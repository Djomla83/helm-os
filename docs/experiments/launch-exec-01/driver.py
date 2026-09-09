"""LAUNCH-EXEC-01 case-posing driver.

The piece the frozen definition deliberately left unimplemented until D-7. It
declares, for every one of the 72 preregistered cases, a concrete plan: which
object is pinned, how the forced state is constructed, exactly which frozen spike
mode is passed, what argv the executed image should see, what the oracle expects,
which **evidence channel** carries the answer, and which frozen outcome rule
interprets it.

**Nothing here executes anything unless an Authorisation instance is passed in,
and Authorisation can only be constructed from an explicit owner flag.** The
plans are inert data; the completeness suite walks them without calling a single
handler. Importing this module poses no case.

Three properties the rest of the file exists to keep:

* **Declared, not discovered.** A case's expectation, schedule, forced state and
  evidence channel are written down before the trial. The driver never chooses a
  bound, a volume or a token after seeing a result.
* **Absence is recorded, never omitted.** Every case in the membership produces a
  record. A case whose forced state did not materialise, or whose declared
  evidence channel the mechanism cannot supply, is ``not_posed`` and scores
  INVALID. A launcher that did not return is recorded from this driver's own
  watchdog as ``launch_returned: false`` and scores FAIL. Neither may be a
  missing key -- that is the P-8 hole, and the checker rejects it.
* **Host properties are separated from mechanism defects.** N2 and M3 are
  conditional on frozen causes preflight settles BEFORE any case is posed, so an
  inherited ``no_new_privs`` bit or a seccomp-blocked ``clone3`` can never be
  scored as a falsified mechanism claim.

**Evidence channels are declared and checked statically.** A case is only posable
if the mechanism can actually deliver the observation its frozen expectation is
written against. ``unposable_cases()`` reports, without running anything, every
case whose channel the frozen spike does not supply. That is a machine-checkable
answer to "does a posing path exist", rather than a claim.

NOT_RUN: no trial has been executed and no case has been posed.
"""
import json
import os
import pathlib
import select
import shutil
import signal
import subprocess
import time

import evidence
import observations
import oracles
from frozen_cases import (
    BY_NAME,
    CONDITIONAL,
    DOCUMENTATION_GATES,
    EXEC_RACE_DELAY_MS,
    MAX_ARG_BYTES,
    MAX_CAPTURE_BYTES,
    MEMBERSHIP,
    P_DESCENDANT_LIFETIME_MS,
    POST_EXIT_DRAIN_MS,
    REPEAT_TRIALS,
    SPAWN_CONFIRM_TIMEOUT_MS,
    T3_GRACE_EXIT_DELAY_MS,
    T4_DELTA_MS,
)


# ============================================================== authorisation
class Authorisation:
    """The single object that permits a case to be posed.

    It exists so that "can this code path exec the spike?" is answerable by
    inspection rather than by tracing flags through call sites. The runner
    constructs exactly one, only after the operator passed
    ``--i-have-owner-authorisation-d7``. No test constructs one and no default
    argument supplies one, so the unit suite cannot reach an exec even by
    mistake.
    """

    __slots__ = ("granted", "reason")

    def __init__(self, owner_flag_passed, reason="owner D-7 flag"):
        if owner_flag_passed is not True:
            raise PermissionError(
                "D-7 execution authorisation was not granted; no case may be "
                "posed. The flag is the owner's decision and nothing else may "
                "stand in for it.")
        self.granted = True
        self.reason = reason


def _require_authorisation(auth):
    if not isinstance(auth, Authorisation) or auth.granted is not True:
        raise PermissionError(
            "posing a case requires an Authorisation built from the owner's "
            "D-7 flag")


# ========================================================== evidence channels
# What kind of observation a case's frozen expectation is written against. A
# channel the mechanism cannot supply makes the case unposable, which is a
# pre-trial finding rather than an experiment result.
CH_RECEIPT = "receipt"            # fields the spike prints on its own stdout
CH_PAYLOAD = "payload"            # stream counts/digests vs the frozen recipe
CH_REPORT = "report"              # the helper's own report, behind the sentinel
CH_TRACE = "trace"                # an external tracer over the child window
CH_LIVENESS = "liveness"          # harness-side descendant liveness probe
CH_THREADED_LAUNCHER = "threaded_launcher"   # a multi-threaded launcher process
CH_ACQUISITION = "acquisition"    # which pidfd acquisition the launcher used
CH_REJECTED_ARM = "rejected_arm"  # the rejected fork+pidfd_open acquisition

ALL_CHANNELS = (CH_RECEIPT, CH_PAYLOAD, CH_REPORT, CH_TRACE, CH_LIVENESS,
                CH_THREADED_LAUNCHER, CH_ACQUISITION, CH_REJECTED_ARM)

# Channels the CURRENT frozen sources can actually deliver.
#
# CH_REPORT is now supplied. PRE-D7-B1 -- the launcher retained the bounded
# capture prefix and free()d it without emitting it, so the helper's own report,
# the FIRST item in the definition's evidence preference order, never reached
# the harness. The launcher now emits that prefix base64-encoded inside each
# stream block. No descriptor was added to the child, no count or digest
# changed, and the executed image still sees exactly {0,1,2}.
#
# CH_THREADED_LAUNCHER and CH_REJECTED_ARM are now supplied by the two
# TEST/CONTROL-ONLY arms M2 and M5 name in their frozen contracts.
#
# CH_ACQUISITION is STILL ABSENT, and deliberately so. M3 asks for evidence that
# the pidfd was acquired ATOMICALLY. The definition's evidence preference order
# permits a syscall record for exactly seven cases -- E1, E7, F4, F7, M1, M2,
# M4 -- and M3 is not one of them; the helper cannot observe its parent's
# acquisition; /proc/<pid>/fd can show that a pidfd exists but not that it was
# obtained in the same syscall as the child; and the oracles compute recipe
# digests only. A receipt field naming the acquisition would be the launcher
# asserting the very thing M3 exists to evidence, which the owner instruction
# rules insufficient. So M3 stays unposable and is returned as an owner
# question rather than answered with a self-assertion.
SPIKE_SUPPLIED_CHANNELS = frozenset({
    CH_RECEIPT, CH_PAYLOAD, CH_REPORT, CH_TRACE, CH_LIVENESS,
    CH_THREADED_LAUNCHER, CH_REJECTED_ARM,
})

CHANNEL_UNAVAILABLE_REASON = {
    CH_REPORT:
        "no channel emits the helper report, so no observation written against "
        "the report can be made",
    CH_THREADED_LAUNCHER:
        "launcher_spike.c has no threading mode, so a multi-threaded launcher "
        "parent cannot be constructed",
    CH_ACQUISITION:
        "the frozen definition permits a syscall record only for the seven "
        "cases declared traced:true, and M3 is not among them; no other frozen "
        "evidence source can show that the pidfd was acquired in the same "
        "syscall that created the child, and a receipt field naming the "
        "acquisition would be the launcher asserting what M3 exists to "
        "evidence. OWNER DECISION REQUIRED",
    CH_REJECTED_ARM:
        "launcher_spike.c implements no fork+pidfd_open acquisition arm, so the "
        "rejected arm has no outcome to record",
}


# ================================================================= plan model
_DEFAULT_TIMEOUT_MS = 5000
_DEFAULT_GRACE_MS = 2000


class CasePlan:
    """One frozen case's concrete posing plan. Pure data, no behaviour."""

    __slots__ = ("case", "binary", "setup", "parent", "spike_flags",
                 "helper_args", "argv0", "rule", "streams", "channels",
                 "posed_when", "excused_fds", "repeat", "timeout_ms",
                 "grace_ms", "spawn_confirm_ms", "work_dir_kind",
                 "body_length_changed", "pre_exec_stall", "traced",
                 "baseline_flags", "note")

    def __init__(self, case, binary, rule, channels, setup="none",
                 parent="none", spike_flags=(), helper_args=(), argv0=None,
                 streams=None, posed_when=None, excused_fds=(), repeat=1,
                 timeout_ms=_DEFAULT_TIMEOUT_MS, grace_ms=_DEFAULT_GRACE_MS,
                 spawn_confirm_ms=SPAWN_CONFIRM_TIMEOUT_MS,
                 work_dir_kind="build", body_length_changed=False,
                 pre_exec_stall=False, baseline_flags=None, note=""):
        self.case = case
        self.binary = binary
        self.rule = rule
        self.channels = tuple(channels)
        self.setup = setup
        self.parent = parent
        self.spike_flags = tuple(spike_flags)
        self.helper_args = tuple(helper_args)
        self.argv0 = argv0 if argv0 is not None else binary
        self.streams = dict(streams or {})
        self.posed_when = posed_when
        self.excused_fds = tuple(excused_fds)
        self.repeat = repeat
        self.timeout_ms = timeout_ms
        self.grace_ms = grace_ms
        self.spawn_confirm_ms = spawn_confirm_ms
        self.work_dir_kind = work_dir_kind
        self.body_length_changed = body_length_changed
        self.pre_exec_stall = pre_exec_stall
        self.traced = BY_NAME[case]["traced"] if case in BY_NAME else False
        # M2 is the only case that needs a second, CONTROL run of the same plan:
        # its expectation is a comparison against the single-threaded arm, so
        # the control's flags are declared here rather than improvised at trial
        # time. None means the case has no control arm.
        self.baseline_flags = (tuple(baseline_flags)
                               if baseline_flags is not None else None)
        self.note = note

    def total_bound_ms(self):
        return oracles.total_bound_ms(self.timeout_ms, self.grace_ms,
                                      self.spawn_confirm_ms, POST_EXIT_DRAIN_MS)

    def missing_channels(self, supplied=None):
        # Resolved at CALL time, not at definition time. A default argument
        # would freeze the module constant into the signature, and the preflight
        # gate would then be unable to react to a channel that stopped being
        # supplied -- which is precisely the condition it exists to catch.
        if supplied is None:
            supplied = SPIKE_SUPPLIED_CHANNELS
        return tuple(c for c in self.channels if c not in supplied)

    def as_dict(self):
        return {
            "case": self.case, "binary": self.binary, "rule": self.rule,
            "channels": list(self.channels), "setup": self.setup,
            "parent": self.parent, "spike_flags": list(self.spike_flags),
            "helper_args": list(self.helper_args), "argv0": self.argv0,
            "streams": self.streams, "posed_when": self.posed_when,
            "repeat": self.repeat, "traced": self.traced,
            "baseline_flags": (list(self.baseline_flags)
                               if self.baseline_flags is not None else None),
            "total_bound_ms": self.total_bound_ms(),
            "missing_channels": list(self.missing_channels()),
        }


def _stream(name, length, completeness="CompleteAtEof"):
    """A declared stream expectation, digest computed from the frozen recipe."""
    return {name: {"bytes": length, "sha256": oracles.stream_digest(name, length),
                   "completeness": completeness}}


def _both_streams(length, completeness="CompleteAtEof"):
    out = _stream("stdout", length, completeness)
    out.update(_stream("stderr", length, completeness))
    return out


# ======================================================= registries (by name)
# Every plan names its setup, parent preparation and posed-check by string, so a
# typo is a test failure rather than a case that silently never runs.
SETUPS = {}
PARENT_STATES = {}
POSED_CHECKS = {}


def _setup(name):
    def register(fn):
        SETUPS[name] = fn
        return fn
    return register


def _parent(name):
    def register(fn):
        PARENT_STATES[name] = fn
        return fn
    return register


def _check(name):
    def register(fn):
        POSED_CHECKS[name] = fn
        return fn
    return register


# ---------------------------------------------------------------- setup steps
# Each returns a dict describing what it built: at minimum {"exec_path": ...},
# or {"not_posed": reason}. They touch only the disposable build directory.

@_setup("none")
def _setup_none(ctx, plan):
    return {"exec_path": str(ctx.build / plan.binary)}


@_setup("copy")
def _setup_copy(ctx, plan):
    """A private copy, so a case that mutates an inode cannot disturb another."""
    target = ctx.build / (plan.case + "_" + plan.binary)
    shutil.copy2(ctx.build / plan.binary, target)
    return {"exec_path": str(target)}


@_setup("rename_alt_over")
def _setup_rename_alt_over(ctx, plan):
    """E2: rename helper_alt OVER the pathname after the pin.

    The substitution happens between the launcher's open() and its execveat, and
    the pinned descriptor must keep running helper_report.
    """
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("rename_over", str(ctx.build / "helper_alt")))


@_setup("rename_then_unlink")
def _setup_rename_then_unlink(ctx, plan):
    """E3: the original pathname is renamed away and then unlinked."""
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("rename_away_and_unlink", None))


@_setup("symlink_retarget")
def _setup_symlink_retarget(ctx, plan):
    """E4: the pinned leaf was a symlink whose target is retargeted after the pin."""
    link = ctx.build / (plan.case + "_link")
    if link.is_symlink() or link.exists():
        link.unlink()
    link.symlink_to(ctx.build / "helper_report")
    return {"exec_path": str(link),
            "post_pin": ("retarget_symlink", str(ctx.build / "helper_alt"))}


@_setup("writer_open_held")
def _setup_writer_open_held(ctx, plan):
    """E5: the HARNESS holds an O_WRONLY descriptor on the same inode across launch.

    The (st_dev, st_ino) pair is recorded so the oracle proves it is the same
    inode; pathname equality is never the evidence.
    """
    info = _setup_copy(ctx, plan)
    return dict(info, hold_writer=True)


@_setup("writer_open_closed")
def _setup_writer_open_closed(ctx, plan):
    """E5b: the writer opens, writes and CLOSES before launch -- no ETXTBSY,
    because the write-deny reference is taken at exec time."""
    info = _setup_copy(ctx, plan)
    fd = os.open(info["exec_path"], os.O_WRONLY)
    try:
        os.pwrite(fd, os.pread(fd, 1, 0), 0)     # a no-op rewrite of one byte
    finally:
        os.close(fd)
    return info


def find_marker_region(path):
    """Locate helper_report's guarded fixed-length marker in the built image.

    ``helper_report.c`` declares HELM-MARK<16 bytes>KRAM-MLEH precisely so a
    length-preserving mutation has a findable, unique target. The guard is what
    makes E6 a mutation of a known 16 bytes rather than a search for a bare
    string literal, which was finding P-10. A non-unique or absent guard yields
    None and the case refuses to guess.
    """
    data = pathlib.Path(path).read_bytes()
    lo, hi = b"HELM-MARK", b"KRAM-MLEH"
    start = data.find(lo)
    if start < 0:
        return None
    marker_at = start + len(lo)
    if data[marker_at + 16:marker_at + 16 + len(hi)] != hi:
        return None
    if data.find(lo, start + 1) >= 0:
        return None
    return marker_at


@_setup("mutate_marker_in_place")
def _setup_mutate_marker(ctx, plan):
    """E6: length-preserving, ELF-valid in-place mutation of the marker region."""
    info = _setup_copy(ctx, plan)
    offset = find_marker_region(info["exec_path"])
    if offset is None:
        return {"not_posed": "the guarded marker region is not uniquely "
                             "locatable in the built helper_report image"}
    return dict(info, post_pin=("pwrite_marker", offset),
                mutated_marker=b"MUTATED\x00\x00\x00\x00\x00\x00\x00\x00\x00")


@_setup("truncate_and_rewrite")
def _setup_truncate_and_rewrite(ctx, plan):
    """E6b: ftruncate + rewrite to a DIFFERENT length after the pin."""
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("truncate_rewrite", None))


@_setup("shared_writable_mapping")
def _setup_shared_writable_mapping(ctx, plan):
    """E6c: a shared writable mapping that survives the descriptor close.

    The one genuinely unresolved kernel question in the manifest -- whether a
    surviving ``i_mmap_writable`` mapping leaves ``i_writecount`` at zero --
    which is why the case is RECORDED and carved out of its own gate.
    """
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("mmap_write", None))


@_setup("fchmod_zero_after_admission")
def _setup_fchmod_zero(ctx, plan):
    """E6d / X1: mode bits cleared AFTER admission recorded them."""
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("fchmod", 0))


@_setup("never_executable")
def _setup_never_executable(ctx, plan):
    """X3: a file that has never had an execute bit; no post-admission change."""
    info = _setup_copy(ctx, plan)
    os.chmod(info["exec_path"], 0o644)
    return info


@_setup("dynamic_helper")
def _setup_dynamic_helper(ctx, plan):
    return {"exec_path": str(ctx.build / "helper_dynamic")}


@_setup("foreign_elf")
def _setup_foreign_elf(ctx, plan):
    return {"exec_path": str(ctx.build / "helper_foreign.elf")}


@_setup("unloadable_in_cohort")
def _setup_unloadable(ctx, plan):
    """X4: passes the cohort rule, ``e_phnum = 0``, so it reaches execveat with
    nothing for the loader to map."""
    return {"exec_path": str(ctx.build / "unloadable_in_cohort.elf")}


@_setup("script_fixture")
def _setup_script(ctx, plan):
    return {"exec_path": str(ctx.build / "script_fixture.sh")}


@_setup("directory_capability")
def _setup_directory(ctx, plan):
    """X5: the capability names a directory descriptor."""
    target = ctx.build / (plan.case + "_dir")
    target.mkdir(exist_ok=True)
    return {"exec_path": str(target)}


@_setup("setid_fixture")
def _setup_setid(ctx, plan):
    """X7 (D-9): set-user-ID on a file the running user already owns.

    No privilege is manufactured -- the owner IS the caller -- so the bit changes
    nothing at exec and the case is purely an admission test on ``st_mode``.
    """
    return {"exec_path": str(ctx.build / "helper_setid")}


@_setup("noexec_copy")
def _setup_noexec_copy(ctx, plan):
    """X8: a copy on an unprivileged-writable noexec mount, if preflight found one.

    No mount is created and no sudo is used; without such a mount the case is
    BLOCKED on its frozen cause rather than failed for an environment reason.
    """
    mounts = ctx.preflight.get("noexec_writable") or []
    if not mounts:
        return {"not_posed": "no unprivileged-writable noexec mount exists"}
    target = pathlib.Path(mounts[0]) / "helm_launch_exec_01_x8"
    shutil.copy2(ctx.build / "helper_report", target)
    os.chmod(target, 0o755)
    return {"exec_path": str(target), "cleanup": str(target)}


@_setup("fork_helper")
def _setup_fork_helper(ctx, plan):
    """The P-series negative controls, with an out-of-band liveness rendezvous.

    ``helper_fork``'s descendant blocks in ``open(fifo, O_WRONLY)`` until a
    reader appears. That blocking open is what makes survival OBSERVABLE: the
    harness opens the read end only AFTER launch() has returned, so a byte that
    arrives proves the descendant was still alive at that moment rather than
    merely that it once started. Liveness travels by pathname because this is a
    negative-control fixture, not the mechanism, and no descriptor above 2 is
    ever passed to a helper.
    """
    fifo = ctx.build / (plan.case + ".liveness")
    if fifo.exists():
        fifo.unlink()
    os.mkfifo(str(fifo), 0o600)
    return {"exec_path": str(ctx.build / "helper_fork"),
            "liveness_fifo": str(fifo),
            "extra_helper_args": ("--liveness-fifo", str(fifo))}


@_setup("privileged_setid")
def _setup_privileged_setid(ctx, plan):
    """N3: expected BLOCKED. No privileged fixture is created, ever.

    Reaching this setup at all would mean an authorised privileged identity
    existed, which this experiment never manufactures. It returns ``not_posed``
    rather than inventing one, so a mis-derived block cannot become a result.
    """
    return {"not_posed": "no privileged identity exists and none is manufactured"}


# ------------------------------------------------------------- parent states
@_parent("none")
def _parent_none(ctx, plan):
    return {}


@_parent("env_canaries")
def _parent_env_canaries(ctx, plan):
    """V1: the host sets the four declared names before launch.

    Under D-10 the child's environ must be EXACTLY empty, so any of these
    arriving in the image is a failure. The values are inert markers, and the
    sanitiser never republishes an environment value in any case.
    """
    return {"env": {"HELM_LEAK_CANARY": "canary",
                    "LD_LIBRARY_PATH": "/nonexistent",
                    "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
                    "HOME": str(ctx.build)}}


@_parent("non_cloexec_fd")
def _parent_non_cloexec(ctx, plan):
    """F2: an unrelated NON-CLOEXEC descriptor open in the parent at exec time.

    Under frozen D-1 arm (i) the child not seeing it is the ONLY pass; there is
    no documented-failure acceptance path.
    """
    return {"open_non_cloexec": str(ctx.build / "helper_report")}


@_parent("cloexec_fd")
def _parent_cloexec(ctx, plan):
    return {"open_cloexec": str(ctx.build / "helper_report")}


@_parent("block_and_ignore_signals")
def _parent_signals(ctx, plan):
    """F5: block {SIGTERM, SIGUSR1} and ignore {SIGPIPE, SIGUSR2}."""
    return {"block": [int(signal.SIGTERM), int(signal.SIGUSR1)],
            "ignore": [int(signal.SIGPIPE), int(signal.SIGUSR2)]}


@_parent("close_stdio")
def _parent_close_stdio(ctx, plan):
    """F6: close 0, 1 and 2 so the launcher's own pipes and exec fd can land there."""
    return {"close_stdio": True}


@_parent("adjacent_fds")
def _parent_adjacent_fds(ctx, plan):
    """F7: force the exec fd and the status write end to adjacent numbers, so a
    close_range gap computation that inverted its bounds would return EINVAL."""
    return {"pad_descriptors": True}


@_parent("sigchld_ignore")
def _parent_sigchld_ignore(ctx, plan):
    """R4 / M5: SIGCHLD set to SIG_IGN -- a caller precondition a crate cannot
    enforce, which is why R4 is RECORDED and its gates forbid reporting a status
    the launcher never observed."""
    return {"sigchld_ignore": True}


@_parent("sigterm_blocked_sigpipe_ignored")
def _parent_t6(ctx, plan):
    """T6: the wall-clock shape must match T2 from a default-signal parent, which
    it can only do if the child's signal state was reset."""
    return {"block": [int(signal.SIGTERM)], "ignore": [int(signal.SIGPIPE)]}


@_parent("multithreaded")
def _parent_multithreaded(ctx, plan):
    """M2: >=3 extra live threads, one allocating, one with a pthread_atfork
    handler registered.

    This describes the LAUNCHER process, not this driver, and the frozen spike
    has no threading mode -- which is why M2 declares CH_THREADED_LAUNCHER and is
    reported unposable rather than approximated with a threaded Python parent
    that would evidence nothing about the launcher.
    """
    return {"threads": 3, "atfork": True, "allocation_in_flight": True}


# -------------------------------------------------------------- posed checks
# A posed check answers "did the forced state actually materialise?". False means
# the case was never a test of the mechanism, which is INVALID -- not FAIL.

@_check("stderr_capture_failed")
def _check_stderr_capture_failed(obs):
    """O7: an exit status AND a stderr capture failure simultaneously true."""
    spike = obs.get("spike")
    if not isinstance(spike, dict):
        return False
    block = spike.get("stderr")
    if not isinstance(block, dict):
        return False
    return (block.get("completeness") == "WriterRetainedAfterChildExit"
            and isinstance(spike.get("exit_code"), int)
            and spike.get("exit_code") >= 0)


@_check("writer_retained")
def _check_writer_retained(obs):
    """O6 / P4: a descendant really did retain the inherited write ends."""
    spike = obs.get("spike")
    if not isinstance(spike, dict):
        return False
    return any(isinstance(spike.get(s), dict)
               and spike[s].get("completeness") == "WriterRetainedAfterChildExit"
               for s in ("stdout", "stderr"))


@_check("no_helper_report")
def _check_no_helper_report(obs):
    """S2 / S5 / S7: the ABSENCE of the report is the independent evidence that
    the image never ran.

    This check is only meaningful where the report channel EXISTS: if no channel
    can carry a report, absence is uninformative and proves nothing. The driver
    therefore refuses to evaluate it unless the observation says the channel was
    available, so a missing channel can never masquerade as evidence of a
    missing report.
    """
    # Only a DECISIVE absence counts. A truncated prefix or a retained writer
    # means the stream could not say whether a report was written, and treating
    # that as "no report" would turn a gap in the evidence into a finding.
    return obs.get("report_state") == observations.REPORT_ABSENT


@_check("returned_before_descendant_lifetime")
def _check_returned_early(obs):
    """O6: launch() must return measurably before the descendant's sleep ends."""
    elapsed = obs.get("elapsed_ms")
    return isinstance(elapsed, int) and elapsed < P_DESCENDANT_LIFETIME_MS


@_check("completed_under_ten_seconds")
def _check_under_ten_seconds(obs):
    """O3: 16 MiB across two streams must complete well inside the 60 s timeout."""
    elapsed = obs.get("elapsed_ms")
    return isinstance(elapsed, int) and elapsed < 10000


@_check("bounded_poll_and_cpu")
def _check_bounded_poll_and_cpu(obs):
    """O5: a spinning launcher also satisfies "no hang", so the drain must be
    bounded in CPU as well as in wall clock.

    The poll() return count is not instrumented by the frozen spike; the CPU
    bound is measured by this driver from the launcher process it spawned, which
    is a harness-side observation requiring no change to the mechanism.
    """
    cpu_ms = obs.get("spike_cpu_ms")
    if cpu_ms is None:
        return False
    polls = obs.get("poll_returns")
    if polls is not None and polls > 10000:
        return False
    return cpu_ms < 200


@_check("same_inode_as_writer")
def _check_same_inode(obs):
    """E5: the write-deny reference must be proven against (st_dev, st_ino)."""
    writer, pinned = obs.get("writer_inode"), obs.get("pinned_inode")
    return writer is not None and pinned is not None and writer == pinned


@_check("trial_floor_512")
def _check_trial_floor(obs):
    """O8: any repeat trial short of 512 bytes is a failure of the drain, so the
    floor is checked per trial rather than on an average."""
    trials = obs.get("repeat_observations")
    if not trials:
        return False
    for trial in trials:
        spike = trial.get("spike")
        if not isinstance(spike, dict):
            return False
        block = spike.get("stdout")
        if not isinstance(block, dict) or block.get("bytes_drained", 0) < 512:
            return False
    return True


# ============================================================== the 72 plans
def _build_plans():
    p = []
    add = p.append

    # ---- E: executable identity and TOCTOU --------------------------------
    # Every E case except E5 needs the helper's own marker to establish WHICH
    # body ran, so they declare the report channel.
    add(CasePlan("E1", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT, CH_TRACE), setup="copy",
                 helper_args=("--exit", "0"),
                 note="the measured digest must equal the independent hashlib "
                      "digest taken before the run"))
    add(CasePlan("E2", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="rename_alt_over",
                 helper_args=("--exit", "0")))
    add(CasePlan("E3", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="rename_then_unlink",
                 helper_args=("--exit", "0")))
    add(CasePlan("E4", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="symlink_retarget",
                 helper_args=("--exit", "0")))
    add(CasePlan("E5", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="writer_open_held", helper_args=("--exit", "0"),
                 posed_when="same_inode_as_writer",
                 note="a pre-exec ETXTBSY never reaches the image, so the "
                      "receipt alone carries the whole result"))
    add(CasePlan("E5b", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="writer_open_closed",
                 helper_args=("--exit", "0")))
    add(CasePlan("E6", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="mutate_marker_in_place",
                 helper_args=("--exit", "0")))
    add(CasePlan("E6b", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="truncate_and_rewrite",
                 body_length_changed=True))
    add(CasePlan("E6c", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="shared_writable_mapping",
                 body_length_changed=True))
    add(CasePlan("E6d", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="fchmod_zero_after_admission",
                 note="a pre-exec EACCES; the receipt carries the whole result"))
    add(CasePlan("E7", "helper_dynamic", "process_disposition",
                 (CH_RECEIPT, CH_REPORT, CH_TRACE), setup="dynamic_helper",
                 note="the one dynamically linked helper; the trace shows the "
                      "kernel resolving PT_INTERP and ld.so resolving DT_NEEDED"))
    add(CasePlan("E8", "helper_foreign.elf", "admission", (CH_RECEIPT,),
                 setup="foreign_elf"))

    # ---- A: argv ----------------------------------------------------------
    add(CasePlan("A1", "helper_report", "argv_exact", (CH_RECEIPT, CH_REPORT),
                 helper_args=("alpha beta gamma",)))
    add(CasePlan("A2", "helper_report", "argv_exact", (CH_RECEIPT, CH_REPORT),
                 helper_args=("a;b|c&d\ne$f`g*h?i",)))
    add(CasePlan("A3", "helper_report", "argv_exact", (CH_RECEIPT, CH_REPORT),
                 helper_args=("",)))
    add(CasePlan("A4", "helper_report", "argv_exact", (CH_RECEIPT, CH_REPORT),
                 helper_args=("x" * MAX_ARG_BYTES,)))
    add(CasePlan("A6", "helper_report", "argv_exact", (CH_RECEIPT, CH_REPORT),
                 argv0="not-a-pathname"))

    # ---- V: environment (D-10) --------------------------------------------
    add(CasePlan("V1", "helper_report", "environ_empty",
                 (CH_RECEIPT, CH_REPORT), parent="env_canaries"))

    # ---- F: descriptor inheritance ----------------------------------------
    add(CasePlan("F1", "helper_report", "fds_exactly_012",
                 (CH_RECEIPT, CH_REPORT)))
    add(CasePlan("F2", "helper_report", "fds_exactly_012",
                 (CH_RECEIPT, CH_REPORT), parent="non_cloexec_fd"))
    add(CasePlan("F3", "helper_report", "fds_exactly_012",
                 (CH_RECEIPT, CH_REPORT), parent="cloexec_fd"))
    add(CasePlan("F4", "helper_report", "fds_exactly_012",
                 (CH_RECEIPT, CH_REPORT, CH_TRACE)))
    add(CasePlan("F5", "helper_report", "signals_reset",
                 (CH_RECEIPT, CH_REPORT), parent="block_and_ignore_signals"))
    add(CasePlan("F6", "helper_report", "fds_exactly_012",
                 (CH_RECEIPT, CH_REPORT), parent="close_stdio"))
    add(CasePlan("F7", "helper_report", "fds_exactly_012",
                 (CH_RECEIPT, CH_REPORT, CH_TRACE), parent="adjacent_fds"))

    # ---- X: exec failure and admission ------------------------------------
    add(CasePlan("X1", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="fchmod_zero_after_admission"))
    add(CasePlan("X2", "script_fixture.sh", "admission", (CH_RECEIPT,),
                 setup="script_fixture"))
    add(CasePlan("X2b", "script_fixture.sh", "process_disposition",
                 (CH_RECEIPT,), setup="script_fixture",
                 spike_flags=("--bypass-admission",),
                 note="ENOENT per execveat(2) BUGS, not ENOEXEC: the "
                      "interpreter is never invoked"))
    add(CasePlan("X2c", "script_fixture.sh", "interpreter_ran_with_devfd",
                 (CH_RECEIPT, CH_REPORT), setup="script_fixture",
                 spike_flags=("--bypass-admission", "--exec-fd-no-cloexec")))
    add(CasePlan("X3", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="never_executable"))
    add(CasePlan("X4", "unloadable_in_cohort.elf", "process_disposition",
                 (CH_RECEIPT,), setup="unloadable_in_cohort",
                 spike_flags=("--bypass-admission",),
                 note="admitted by the cohort rule, e_phnum = 0, so execveat is "
                      "reached with nothing for the loader to map"))
    add(CasePlan("X5", "helper_report", "admission", (CH_RECEIPT,),
                 setup="directory_capability"))
    add(CasePlan("X6", "helper_report", "admission", (CH_RECEIPT,),
                 setup="copy", spike_flags=("--exec-fd-o-path",)))
    add(CasePlan("X7", "helper_setid", "admission", (CH_RECEIPT,),
                 setup="setid_fixture"))
    add(CasePlan("X8", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="noexec_copy"))

    # ---- O: output --------------------------------------------------------
    # The O series runs --no-report, so its exec evidence is the payload itself:
    # only the pinned helper can produce the frozen recipe, and the receipt
    # carries the count and digest that prove it. No report channel is needed.
    add(CasePlan("O1", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stdout", "4096"),
                 streams=_stream("stdout", 4096)))
    add(CasePlan("O2", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stderr", "4096"),
                 streams=_stream("stderr", 4096)))
    add(CasePlan("O3", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stdout", str(8 * 1024 * 1024),
                              "--stderr", str(8 * 1024 * 1024)),
                 streams=_both_streams(8 * 1024 * 1024), timeout_ms=60000,
                 posed_when="completed_under_ten_seconds"))
    add(CasePlan("O4", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stdout",
                              str(MAX_CAPTURE_BYTES * 2)),
                 streams=_stream("stdout", MAX_CAPTURE_BYTES * 2),
                 note="digest over ALL drained bytes; the retained prefix stops "
                      "at the bound and the truncated flag stays out of the "
                      "receipt"))
    add(CasePlan("O5", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stdout", "4096",
                              "--stderr", "4096", "--close-stdout-early"),
                 streams=_both_streams(4096),
                 posed_when="bounded_poll_and_cpu"))
    add(CasePlan("O6", "helper_fork", "process_disposition",
                 (CH_RECEIPT, CH_PAYLOAD, CH_LIVENESS), setup="fork_helper",
                 helper_args=("--prewrite", "512", "--retain-stdio",
                              "--parent-exit", "0", "--lifetime-ms",
                              str(P_DESCENDANT_LIFETIME_MS)),
                 streams=_stream("stdout", 512, "WriterRetainedAfterChildExit"),
                 posed_when="writer_retained"))
    add(CasePlan("O7", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stderr", "4096", "--exit", "42"),
                 streams=_stream("stderr", 4096),
                 posed_when="stderr_capture_failed"))
    add(CasePlan("O8", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stdout", "512"),
                 streams=_stream("stdout", 512), repeat=REPEAT_TRIALS,
                 posed_when="trial_floor_512"))

    # ---- R: exit and signal -----------------------------------------------
    add(CasePlan("R1", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), helper_args=("--exit", "0")))
    add(CasePlan("R2", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), helper_args=("--exit", "42")))
    add(CasePlan("R3", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT),
                 helper_args=("--rlimit-core-zero", "--raise-segv")))
    add(CasePlan("R4", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), parent="sigchld_ignore",
                 helper_args=("--exit", "42")))

    # ---- T: timeout -------------------------------------------------------
    # helper_report emits its report BEFORE it sleeps, so a timeout case still
    # has exec evidence -- which is why every T case declares the report channel.
    add(CasePlan("T1", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), helper_args=("--sleep-ms", "60000"),
                 timeout_ms=2000, grace_ms=2000))
    add(CasePlan("T2", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT),
                 helper_args=("--ignore-sigterm", "--sleep-ms", "60000"),
                 timeout_ms=2000, grace_ms=2000))
    add(CasePlan("T3", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT),
                 helper_args=("--grace-exit-delay-ms",
                              str(T3_GRACE_EXIT_DELAY_MS),
                              "--sleep-ms", "60000", "--exit", "9"),
                 timeout_ms=2000, grace_ms=5000))
    add(CasePlan("T4", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT),
                 helper_args=("--sleep-ms", str(5000 - T4_DELTA_MS),
                              "--exit", "9"), timeout_ms=5000, grace_ms=2000))
    add(CasePlan("T5", "helper_fork", "process_disposition",
                 (CH_RECEIPT, CH_PAYLOAD, CH_LIVENESS), setup="fork_helper",
                 helper_args=("--prewrite", "512", "--retain-stdio",
                              "--parent-exit", "7", "--lifetime-ms",
                              str(P_DESCENDANT_LIFETIME_MS)),
                 streams=_stream("stdout", 512, "WriterRetainedAfterChildExit"),
                 timeout_ms=5000, grace_ms=2000))
    add(CasePlan("T6", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT),
                 parent="sigterm_blocked_sigpipe_ignored",
                 helper_args=("--ignore-sigterm", "--sleep-ms", "60000"),
                 timeout_ms=2000, grace_ms=2000))

    # ---- N: no_new_privs (D-11) -------------------------------------------
    add(CasePlan("N1", "helper_report", "no_new_privs",
                 (CH_RECEIPT, CH_REPORT)))
    add(CasePlan("N2", "helper_report", "no_new_privs",
                 (CH_RECEIPT, CH_REPORT), spike_flags=("--skip-no-new-privs",)))
    add(CasePlan("N3", "helper_setid", "privilege_transition_suppressed",
                 (CH_RECEIPT, CH_REPORT), setup="privileged_setid"))

    # ---- P: process-tree negative controls --------------------------------
    add(CasePlan("P1", "helper_fork", "descendant_lifecycle",
                 (CH_RECEIPT, CH_LIVENESS), setup="fork_helper",
                 helper_args=("--release-stdio", "--parent-exit", "0",
                              "--lifetime-ms", str(P_DESCENDANT_LIFETIME_MS))))
    add(CasePlan("P2", "helper_fork", "descendant_lifecycle",
                 (CH_RECEIPT, CH_LIVENESS), setup="fork_helper",
                 helper_args=("--release-stdio", "--setsid", "--parent-exit",
                              "0", "--lifetime-ms",
                              str(P_DESCENDANT_LIFETIME_MS))))
    add(CasePlan("P3", "helper_fork", "sweep", (CH_RECEIPT,),
                 setup="fork_helper",
                 helper_args=("--release-stdio", "--parent-exit", "0",
                              "--lifetime-ms", str(P_DESCENDANT_LIFETIME_MS))))
    add(CasePlan("P4", "helper_fork", "descendant_lifecycle",
                 (CH_RECEIPT, CH_LIVENESS), setup="fork_helper",
                 helper_args=("--retain-stdio", "--setsid", "--parent-exit", "0",
                              "--lifetime-ms", str(P_DESCENDANT_LIFETIME_MS)),
                 posed_when="writer_retained"))

    # ---- S: spawn/exec confirmation ---------------------------------------
    add(CasePlan("S1", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), helper_args=("--exit", "0"),
                 note="clean EOF alone is not a PASS; the report is the exec "
                      "evidence, which is why this case declares the channel"))
    add(CasePlan("S2", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), work_dir_kind="fchmod_zero",
                 posed_when="no_helper_report",
                 note="the report's ABSENCE is the evidence, which is only "
                      "informative where the channel exists"))
    add(CasePlan("S3", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), helper_args=("--exit", "127")))
    add(CasePlan("S4", "helper_report", "process_disposition", (CH_RECEIPT,),
                 helper_args=("--exit-immediately", "7"),
                 spike_flags=("--post-fork-delay-ms", str(EXEC_RACE_DELAY_MS)),
                 note="--exit-immediately is the helper's first observable act, "
                      "so no report can exist and the exit status is the whole "
                      "of the evidence"))
    add(CasePlan("S5", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), spike_flags=("--die-before-exec",),
                 posed_when="no_helper_report",
                 note="the discriminating case: byte-identical to S1 at the "
                      "parent, and only the report's absence separates them"))
    add(CasePlan("S6", "helper_report", "process_disposition", (CH_RECEIPT,),
                 spike_flags=("--stall-pre-exec-ms",
                              str(SPAWN_CONFIRM_TIMEOUT_MS * 2)),
                 pre_exec_stall=True))
    add(CasePlan("S7", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), work_dir_kind="fchmod_zero",
                 repeat=REPEAT_TRIALS, posed_when="no_helper_report"))

    # ---- M: mechanism minimality and parent shape -------------------------
    add(CasePlan("M1", "helper_report", "child_syscalls_within_frozen_set",
                 (CH_RECEIPT, CH_TRACE), helper_args=("--exit", "0")))
    add(CasePlan("M2", "helper_report", "identical_to_single_threaded_arm",
                 (CH_RECEIPT, CH_TRACE, CH_THREADED_LAUNCHER),
                 parent="multithreaded", helper_args=("--exit", "0"),
                 spike_flags=("--extra-threads", "3"), baseline_flags=(),
                 note="the frozen arm exactly: >=3 extra live threads in the "
                      "LAUNCHER, one allocating continuously and one with a "
                      "pthread_atfork handler registered. The control run is "
                      "the same plan with no threads, and the child window must "
                      "come out identical"))
    add(CasePlan("M3", "helper_report", "pidfd_acquired_atomically",
                 (CH_RECEIPT, CH_ACQUISITION), helper_args=("--exit", "0")))
    add(CasePlan("M4", "helper_report", "sequence_matches_frozen_stages",
                 (CH_RECEIPT, CH_TRACE), helper_args=("--exit", "0")))
    add(CasePlan("M5", "helper_report", "rejected_acquisition",
                 (CH_RECEIPT, CH_REJECTED_ARM), parent="sigchld_ignore",
                 helper_args=("--exit", "42"),
                 spike_flags=("--rejected-acquisition-arm",),
                 note="the REJECTED fork+pidfd_open arm, recorded to evidence "
                      "why clone3(CLONE_PIDFD) is primary. Not a candidate "
                      "mechanism"))
    return p


_PLAN_LIST = _build_plans()
DRIVER_CASE_IDS = [plan.case for plan in _PLAN_LIST]
CASE_PLANS = {plan.case: plan for plan in _PLAN_LIST}


# ============================================================== completeness
def completeness():
    """Static completeness of the driver against the frozen membership.

    Calls no handler and poses nothing: it walks the plan table as data. This is
    the machine-checkable proof that no case exists only in ``frozen_cases.py``,
    and it rejects a missing case, a duplicate handler and an unknown id alike.
    """
    frozen = set(MEMBERSHIP)
    declared = list(DRIVER_CASE_IDS)
    seen, duplicates = set(), []
    for name in declared:
        if name in seen:
            duplicates.append(name)
        seen.add(name)
    return {
        "frozen_total": len(frozen),
        "driver_total": len(declared),
        "missing": sorted(frozen - seen),
        "unknown": sorted(seen - frozen),
        "duplicates": sorted(set(duplicates)),
        "unresolved_setups": sorted(
            {p.setup for p in _PLAN_LIST if p.setup not in SETUPS}),
        "unresolved_parents": sorted(
            {p.parent for p in _PLAN_LIST if p.parent not in PARENT_STATES}),
        "unresolved_rules": sorted(
            {p.rule for p in _PLAN_LIST if p.rule not in observations.RULES}),
        "unresolved_checks": sorted(
            {p.posed_when for p in _PLAN_LIST
             if p.posed_when is not None and p.posed_when not in POSED_CHECKS}),
        "unknown_channels": sorted(
            {c for p in _PLAN_LIST for c in p.channels if c not in ALL_CHANNELS}),
        "complete": (seen == frozen and not duplicates
                     and len(declared) == len(frozen)),
    }


def unposable_cases(supplied=None):
    """Cases the frozen mechanism cannot pose, with the reason, computed statically.

    A case definition existing in the manifest is not enough, and neither is a
    plan existing here: the mechanism has to be able to deliver the observation
    the frozen expectation is written against. This function is the honest
    answer to that question and it runs nothing.
    """
    if supplied is None:
        supplied = SPIKE_SUPPLIED_CHANNELS
    out = {}
    for plan in _PLAN_LIST:
        missing = plan.missing_channels(supplied)
        if missing:
            out[plan.case] = {
                "class": BY_NAME[plan.case]["cls"],
                "expected": (BY_NAME[plan.case]["predict"]
                             or BY_NAME[plan.case]["safe"]),
                "missing_channels": list(missing),
                "reasons": [CHANNEL_UNAVAILABLE_REASON.get(c, c)
                            for c in missing],
            }
    return out


# ================================================================ evaluation
def _documentation_gate(spike):
    """E6 / E6b / E6d: the measurement must be named as a PRE-execution one.

    A case cannot be PASSed by observing the expected kernel behaviour while a
    field still presents the pre-execution measurement as the identity of the
    executed body. The receipt field is checked BY NAME, and a field claiming the
    executed body fails the gate.
    """
    if not isinstance(spike, dict):
        return False
    if "pre_exec_body_sha256" not in spike:
        return False
    forbidden = ("executed_body_sha256", "body_sha256", "ran_sha256",
                 "executed_sha256")
    return not any(key in spike for key in forbidden)


def _asserted_unobserved_fact(spike):
    """D-8: no timestamp and no duration may appear inside the receipt.

    ``launcher_spike.c`` already keeps both outside, in fields whose names say
    so. This is the parent-side check that a published receipt did not regain
    one; it returns the offending key so the failure names itself.
    """
    receipt = evidence.receipt_view(spike)
    if receipt is None:
        return None
    for key in receipt:
        low = key.lower()
        if "elapsed" in low or "timestamp" in low or "duration" in low:
            return "the receipt carries a temporal field: " + key
    return None


def evaluate(plan, obs):
    """Turn one case's observation into the record ``checker.py`` scores.

    The record is data. It never names its own status, and every field the
    checker needs is present or explicitly absent -- ``launch_returned`` in
    particular, whose omission is the P-8 hole and is INVALID by contract.
    """
    spec = BY_NAME[plan.case]
    record = {
        "case": plan.case,
        "class": spec["cls"],
        "traced": bool(obs.get("traced")),
        "launch_returned": obs.get("launch_returned"),
        "elapsed_ms": obs.get("elapsed_ms"),
        "total_bound_ms": plan.total_bound_ms(),
    }

    # A conditional case blocked on its own frozen cause is expected, and the
    # checker enforces that only a conditional case may absorb one.
    if obs.get("blocked"):
        record["blocked"] = obs["blocked"]
        return record
    if obs.get("not_posed"):
        record["not_posed"] = obs["not_posed"]
        return record

    if obs.get("launch_returned") is False:
        # A launcher-side non-return is never a recordable outcome. Recorded
        # from this driver's own watchdog, so a hang can never be an absent
        # record understating a known result as an open question.
        record["outcome"] = None
        record["reason"] = "launch() did not return within the declared bound"
        return record

    if plan.posed_when is not None:
        if not POSED_CHECKS[plan.posed_when](obs):
            record["not_posed"] = ("the forced state did not materialise: " +
                                   plan.posed_when)
            return record

    token, reason = observations.derive(plan.rule, obs)
    if token is None:
        record["not_posed"] = "observation not interpretable: " + reason
        return record
    record["outcome"] = token
    record["reason"] = reason

    if spec["gates"]:
        record["gates"] = {gate: obs.get("gates", {}).get(gate)
                           for gate in spec["gates"]}
    if plan.case in DOCUMENTATION_GATES:
        record["documentation_gate"] = _documentation_gate(obs.get("spike"))
    offending = _asserted_unobserved_fact(obs.get("spike"))
    if offending:
        record["asserted_unobserved_fact"] = offending
    return record


# =================================================================== context
class TrialContext:
    """Everything a case needs that is not in its plan. Built once per trial."""

    def __init__(self, build, work, preflight, freeze, sanitiser,
                 supplied_channels=None):
        self.build = pathlib.Path(build)
        self.work = pathlib.Path(work)
        self.preflight = preflight
        self.freeze = freeze
        self.sanitiser = sanitiser
        self.supplied_channels = frozenset(
            SPIKE_SUPPLIED_CHANNELS if supplied_channels is None
            else supplied_channels)
        self.blocks = dict(preflight.get("block_reasons") or {})


def blocked_cause_for(plan, ctx):
    """The frozen block cause this case must absorb here, or None.

    ONLY a conditional case may absorb a block, and only with the cause its own
    manifest entry names -- the P-3 correction. A mandatory case in a blocking
    environment is not quietly retired: it stays mandatory and the aggregate
    becomes MECHANISM_INCONCLUSIVE, which is the honest outcome.
    """
    spec = BY_NAME[plan.case]
    if spec["cls"] != CONDITIONAL:
        return None
    cause = spec["blocked_if"]
    return cause if cause in ctx.blocks else None


def spike_argv(plan, exec_path, work_dir, build, flags=None,
               extra_helper_args=()):
    """The exact spike invocation a case uses. Pure string construction.

    Exposed so the command can be reviewed, diffed and tested WITHOUT running
    it: the suite asserts every plan produces a well-formed invocation, and no
    test ever hands the result to a process. ``flags`` overrides the plan's own
    spike flags, which is how M2's control arm runs the same plan without the
    extra threads.
    """
    argv = [str(pathlib.Path(build) / "launcher_spike"),
            "--exec-path", str(exec_path),
            "--work-dir", str(work_dir),
            "--timeout-ms", str(plan.timeout_ms),
            "--grace-ms", str(plan.grace_ms),
            "--spawn-confirm-ms", str(plan.spawn_confirm_ms),
            "--post-exit-drain-ms", str(POST_EXIT_DRAIN_MS),
            "--max-capture-bytes", str(MAX_CAPTURE_BYTES)]
    argv.extend(plan.spike_flags if flags is None else flags)
    argv.extend(["--arg", plan.argv0])
    for item in list(plan.helper_args) + list(extra_helper_args):
        argv.extend(["--arg", item])
    return argv


def declared_injection_modes(plan):
    """The declared test-only injection modes this plan carries.

    M1, M2 and M4 trace the PRODUCTION configuration, so this must be empty for
    them -- enforced by the rule itself and asserted statically in the tests.
    ``--extra-threads`` is deliberately NOT one of these: it changes the
    PARENT's shape, not the child's syscall sequence, which is exactly why M2
    can carry it and still trace a production child window.
    """
    return observations.injection_modes_in(plan.spike_flags)


def tracer_argv(preflight, output_path):
    """How this environment collects a syscall record, or None if it cannot.

    Availability is re-probed at preflight and never assumed. Only the seven
    cases declared ``traced: true`` ever reach here, which is the definition's
    own restriction on evidence item 2 rather than a driver convention.
    """
    strace = (preflight or {}).get("strace")
    if not strace:
        return None
    return [str(strace), "-f", "-qq", "-o", str(output_path)]


# =================================================================== posing
def observe(plan, ctx, auth):
    """Pose ONE preregistered case and return its observation.

    **The only path in this repository that executes the mechanism.** It requires
    an :class:`Authorisation`, which only the runner constructs and only from the
    owner's D-7 flag.

    The channel check comes FIRST and is deliberate: a case whose declared
    evidence channel the mechanism cannot supply is never posed at all. Running
    it anyway would produce a receipt that looks like a result, and the honest
    record is that the case could not be posed.
    """
    _require_authorisation(auth)

    missing = plan.missing_channels(ctx.supplied_channels)
    if missing:
        return {"not_posed": "the mechanism supplies no channel for " +
                             ", ".join(missing) + ": " +
                             "; ".join(CHANNEL_UNAVAILABLE_REASON.get(c, c)
                                       for c in missing),
                "launch_returned": None}

    cause = blocked_cause_for(plan, ctx)
    if cause is not None:
        return {"blocked": cause, "launch_returned": None}

    built = SETUPS[plan.setup](ctx, plan)
    if built.get("not_posed"):
        return {"not_posed": built["not_posed"], "launch_returned": None}

    parent_state = PARENT_STATES[plan.parent](ctx, plan)
    trials = [_run_once(plan, ctx, built, parent_state)
              for _ in range(plan.repeat)]

    obs = dict(trials[0])
    obs["repeat_observations"] = trials

    # M2's control arm: the SAME plan with no extra threads. Its child window is
    # the baseline the threaded arm must match, and it is collected here rather
    # than borrowed from another case so that both arms share every other
    # condition -- which is what makes the comparison mean anything.
    if plan.baseline_flags is not None:
        baseline = _run_once(plan, ctx, built, parent_state,
                             flags=plan.baseline_flags)
        obs["baseline_observation"] = baseline
        trace = baseline.get("trace")
        obs["single_threaded_child_syscalls"] = (
            trace.get("child_syscalls") if isinstance(trace, dict) else None)

    obs["declared_pre_exec_stall"] = plan.pre_exec_stall
    obs["declared_body_length_changed"] = plan.body_length_changed
    obs["declared_injection_modes"] = declared_injection_modes(plan)
    obs["expected_streams"] = plan.streams or None
    obs["expected_argv"] = ([plan.argv0] + list(plan.helper_args)
                            + list(built.get("extra_helper_args", ())))
    obs["excused_descriptors"] = plan.excused_fds
    obs["traced"] = plan.traced
    obs["gates"] = gates_for(plan, obs)
    return obs


def gates_for(plan, obs):
    """Gated sub-assertions for the RECORDED cases, computed from observations.

    A gate is never derived from what the launcher intended, only from what the
    record shows. A gate whose evidence is missing stays False, because a gate
    that cannot be shown to hold has not been shown to hold.
    """
    spec = BY_NAME[plan.case]
    if not spec["gates"]:
        return {}
    spike = obs.get("spike") or {}
    elapsed = obs.get("elapsed_ms")
    bound = plan.total_bound_ms()
    out = {}
    for gate in spec["gates"]:
        if gate == "launch_returns_within_total_bound":
            out[gate] = (obs.get("launch_returned") is True
                         and isinstance(elapsed, int) and elapsed <= bound)
        elif gate == "completeness_reported":
            out[gate] = all(
                isinstance(spike.get(s), dict)
                and spike[s].get("completeness") in (
                    "CompleteAtEof", "WriterRetainedAfterChildExit")
                for s in ("stdout", "stderr"))
        elif gate == "writer_retained_after_child_exit_reported":
            out[gate] = any(
                isinstance(spike.get(s), dict)
                and spike[s].get("completeness")
                == "WriterRetainedAfterChildExit"
                for s in ("stdout", "stderr"))
        elif gate == "sweep_strictly_before_reap":
            # The launcher issues the sweep before waitid by construction, and
            # the receipt records whether it was issued at all. A sweep that was
            # never issued cannot have been issued out of order.
            out[gate] = spike.get("group_sweep_issued") in (True, False)
        elif gate == "never_reports_exited_zero":
            out[gate] = obs.get("outcome_token") != "Exited:0"
        elif gate == "never_hangs":
            out[gate] = obs.get("launch_returned") is True
        elif gate == "no_claim_that_measured_bytes_ran":
            out[gate] = _documentation_gate(spike)
        elif gate == "never_reports_unobserved_exit_status":
            # M5. The arm may report an exit status ONLY when it observed one.
            arm = spike.get("rejected_acquisition_arm")
            if not isinstance(arm, dict):
                out[gate] = False
            elif arm.get("exit_status_observed") is True:
                out[gate] = (isinstance(arm.get("exit_status"), int)
                             and arm["exit_status"] >= 0)
            else:
                out[gate] = arm.get("exit_status") in (-1, None)
        else:
            out[gate] = False
    return out


def _run_once(plan, ctx, built, parent_state, flags=None):
    """One launcher invocation, with this driver's own watchdog.

    The watchdog is the P-8 correction in code: ``launch_returned`` is recorded
    from here, always, so a true hang is a FAIL rather than a missing record.
    """
    argv = spike_argv(plan, built["exec_path"], ctx.work, ctx.build, flags=flags,
                      extra_helper_args=built.get("extra_helper_args", ()))
    trace_path = None
    if plan.traced:
        trace_path = ctx.build / (plan.case + ".strace")
        tracer = tracer_argv(ctx.preflight, trace_path)
        if tracer is None:
            return {"not_posed": "no tracer is available for a traced case",
                    "launch_returned": None}
        argv = tracer + argv

    env = dict(parent_state.get("env") or {})
    bound_s = (plan.total_bound_ms() + 5000) / 1000.0

    started = time.monotonic()
    try:
        proc = subprocess.run(argv, capture_output=True, env=env,
                              timeout=bound_s)
        returned = True
        stdout, stderr, rc = proc.stdout, proc.stderr, proc.returncode
    except subprocess.TimeoutExpired as expired:
        returned = False
        stdout = expired.stdout or b""
        stderr = expired.stderr or b""
        rc = None
    elapsed_ms = int((time.monotonic() - started) * 1000)

    spike = observations.parse_spike_stdout(stdout)

    # PRE-D7-B1: the helper report now travels out inside the receipt, as the
    # base64 capture prefix of descriptor 1. It is decoded HERE, in this process,
    # and never republished -- evidence.py withholds the field by key, because
    # scanning an encoded blob for secrets is a game the scanner loses.
    report_state = observations.REPORT_STREAM_INCOMPLETE
    report, payload = None, b""
    if isinstance(spike, dict) and spike.get("admission") == "accepted":
        report_state, report, payload = observations.helper_report_state(
            spike.get("stdout"))

    payload_is_recipe = None
    if plan.streams and isinstance(spike, dict) and \
            spike.get("admission") == "accepted":
        payload_is_recipe = all(
            isinstance(spike.get(name), dict)
            and spike[name].get("bytes_drained") == want["bytes"]
            and spike[name].get("drained_sha256") == want["sha256"]
            for name, want in plan.streams.items())

    trace = None
    if trace_path is not None and trace_path.exists():
        trace = observations.parse_strace_child_window(trace_path.read_bytes())

    return {
        "spike": spike,
        "report": report,
        "report_state": report_state,
        "payload_len": len(payload),
        "payload_is_recipe": payload_is_recipe,
        "exec_confirmation": observations.exec_confirmation(
            spike, report, payload_is_recipe, report_state),
        "trace": trace,
        "observed_stage_sequence": (trace or {}).get("stage_sequence"),
        "descendant_alive_after_launch": _descendant_alive(built, plan),
        "launch_returned": returned,
        "elapsed_ms": elapsed_ms,
        "spike_exit": rc,
        "spike_stderr": (stderr or b"").decode("utf-8", "replace")[-2000:],
    }


def _descendant_alive(built, plan, timeout_ms=3000):
    """Whether the P-series descendant outlived launch(), or None if not asked.

    The read end is opened only after launch() has returned. ``helper_fork``'s
    descendant is blocked in ``open(fifo, O_WRONLY)`` until then, so a byte that
    arrives is proof it was alive at that moment. Nothing arriving inside the
    bound means it is gone -- and BOTH answers are honest: D-4 makes the launcher
    responsible for the direct child only, so it claims no process-tree
    containment and either observation is a recorded fact rather than a verdict.
    """
    fifo = built.get("liveness_fifo")
    if not fifo:
        return None
    try:
        fd = os.open(fifo, os.O_RDONLY | os.O_NONBLOCK)
    except OSError:
        return None
    try:
        poller = select.poll()
        poller.register(fd, select.POLLIN)
        if not poller.poll(timeout_ms):
            return False
        return os.read(fd, 1) == b"L"
    except OSError:
        return None
    finally:
        os.close(fd)
