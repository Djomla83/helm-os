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
import contextlib
import json
import os
import pathlib
import select
import shutil
import socket
import signal
import stat
import subprocess
import time

import checker
import evidence
import harness
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
CH_REJECTED_ARM = "rejected_arm"  # the rejected fork+pidfd_open acquisition

# CH_ACQUISITION is retired by owner amendment M3-T. It named a launcher field
# describing its own acquisition mode, which is the self-assertion M3 exists to
# avoid; M3 now uses CH_TRACE like every other traced case.

ALL_CHANNELS = (CH_RECEIPT, CH_PAYLOAD, CH_REPORT, CH_TRACE, CH_LIVENESS,
                CH_THREADED_LAUNCHER, CH_REJECTED_ARM)

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
# M3 is now supplied by CH_TRACE. Owner amendment M3-T made it a traced case
# before the first valid trial, so the acquisition is established from the
# external syscall record instead of from a launcher field describing itself.
# The traced set is amended prospectively from seven cases to eight.
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
    CH_REJECTED_ARM:
        "launcher_spike.c implements no fork+pidfd_open acquisition arm, so the "
        "rejected arm has no outcome to record",
}


# ================================================================= plan model
_DEFAULT_TIMEOUT_MS = 5000
_DEFAULT_GRACE_MS = 2000


class CasePlan:
    """One frozen case's concrete posing plan. Pure data, no behaviour."""

    # Every slot is a semantic field with a declared consumer in
    # CASEPLAN_FIELD_CONSUMERS; a field nothing reads is a test failure. The
    # Trial #2 delta review found work_dir_kind stored and never read, so S2
    # and S7 ran in an ordinary directory. It is gone: their construction is a
    # setup now, and a setup's keys have consumers.
    __slots__ = ("case", "binary", "setup", "parent", "spike_flags",
                 "helper_args", "argv0", "rule", "streams", "channels",
                 "posed_when", "excused_fds", "repeat", "timeout_ms",
                 "grace_ms", "spawn_confirm_ms", "body_length_changed",
                 "pre_exec_stall", "traced", "baseline_flags",
                 "expected_marker", "assertions", "note")

    def __init__(self, case, binary, rule, channels, setup="none",
                 parent="none", spike_flags=(), helper_args=(), argv0=None,
                 streams=None, posed_when=None, excused_fds=(), repeat=1,
                 timeout_ms=_DEFAULT_TIMEOUT_MS, grace_ms=_DEFAULT_GRACE_MS,
                 spawn_confirm_ms=SPAWN_CONFIRM_TIMEOUT_MS,
                 body_length_changed=False, pre_exec_stall=False,
                 baseline_flags=None, expected_marker=None, assertions=(),
                 note=""):
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
        self.body_length_changed = body_length_changed
        self.pre_exec_stall = pre_exec_stall
        self.traced = BY_NAME[case]["traced"] if case in BY_NAME else False
        # M2 is the only case that needs a second, CONTROL run of the same plan:
        # its expectation is a comparison against the single-threaded arm, so
        # the control's flags are declared here rather than improvised at trial
        # time. None means the case has no control arm.
        self.baseline_flags = (tuple(baseline_flags)
                               if baseline_flags is not None else None)
        # N-2. What the executed image must identify itself as, and the named
        # post-rule assertions that compare what executed with what the case
        # claims. Forced-state landing decides whether a case was POSED; these
        # decide what the posed case SHOWED, so a violation is a FAIL.
        self.expected_marker = expected_marker
        self.assertions = tuple(assertions)
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
            "expected_marker": self.expected_marker,
            "assertions": list(self.assertions),
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


# N-3 / N-4a. A posed check that reads an observation key nothing produces can
# never hold, and Trial #2's E5 and O5 were exactly that. Every check therefore
# DECLARES the observation keys it reads. Tests prove the declaration matches
# the function body and that every declared key has a producer on the path of
# every case that uses the check.
#
# F417. A posed check answers ONLY whether a precondition the mechanism does
# not control was established. It never reads what the mechanism produced for
# the property under test -- that is the case's result -- and it is applied to
# every repetition separately (evaluate()).
POSED_CHECK_READS = {}


def _check(name, reads):
    def register(fn):
        POSED_CHECKS[name] = fn
        POSED_CHECK_READS[name] = tuple(reads)
        return fn
    return register




# ================================================= the closed setup-result schema
# T2-R1. Trial #1's setups produced directives -- post_pin, hold_writer,
# mutated_marker, cleanup -- that NOTHING consumed, so ten cases ran without the
# adversarial condition that defines them and E5's mandatory expectation could
# never be met. A key that nobody reads is worse than a missing feature: it
# looks like the case is doing something.
#
# Every key a setup may return is declared here with its role, and a test fails
# if a setup produces a key this schema does not name or if a semantic key has
# no execution consumer. The schema is the contract; the list is not maintained
# by hand from the keys that happen to exist today.

EXEC_PATH = "EXEC_PATH"
IMMEDIATE_SETUP_FACT = "IMMEDIATE_SETUP_FACT"
POST_PIN_ACTION = "POST_PIN_ACTION"
POST_PIN_INPUT = "POST_PIN_INPUT"
HELD_RESOURCE = "HELD_RESOURCE"
POSED_ASSERTION_INPUT = "POSED_ASSERTION_INPUT"
LAUNCH_ARGUMENT = "LAUNCH_ARGUMENT"
CLEANUP_RESOURCE = "CLEANUP_RESOURCE"

SETUP_RESULT_SCHEMA = {
    "exec_path": EXEC_PATH,
    "not_posed": IMMEDIATE_SETUP_FACT,
    "extra_helper_args": POSED_ASSERTION_INPUT,
    "liveness_fifo": HELD_RESOURCE,
    # O6 and O7 (AB7-B1): the case-private FIFO whose read end the harness
    # opens BEFORE the launcher is spawned, so helper_fork's descendant can
    # signal before the launcher's group sweep ends it.
    "fixture_signal_fifo": HELD_RESOURCE,
    "post_pin": POST_PIN_ACTION,
    "hold_writer": POST_PIN_ACTION,
    # The exact bytes a post-pin marker action writes. The action reads them
    # back through a separate descriptor before the case may be posed, and E6's
    # executed-marker assertion compares the report against the same bytes.
    "mutated_marker": POST_PIN_INPUT,
    # S2/S7: the case-private working directory handed to the launcher as its
    # capability, changed to mode 0000 at the barrier.
    "work_dir": LAUNCH_ARGUMENT,
    "cleanup": CLEANUP_RESOURCE,
}

# Keys whose absence from the execution path changes what a case tests.
SEMANTIC_SETUP_KEYS = frozenset(
    name for name, role in SETUP_RESULT_SCHEMA.items()
    if role in (POST_PIN_ACTION, POST_PIN_INPUT, HELD_RESOURCE,
                POSED_ASSERTION_INPUT, LAUNCH_ARGUMENT, CLEANUP_RESOURCE))


# ===================================================== the post-pin barrier
# The launcher reaches a frozen hook AFTER it has pinned, measured and admitted
# the object and BEFORE it clones or executes anything, announces READY on an
# inherited control socket, and waits. The HARNESS performs the preregistered
# change, proves it landed, and answers CONTINUE. The launcher then closes the
# control descriptor -- before clone3, so it can never reach the child -- and
# proceeds.
#
# The mutation is always the harness's act. The launcher only synchronises: a
# mechanism that mutated its own target would be testing itself.

POST_PIN_READY = b"R"
POST_PIN_CONTINUE = b"C"
POST_PIN_READY_TIMEOUT_S = 30.0
POST_PIN_ACTIONS = {}


def _post_pin(name):
    def register(fn):
        POST_PIN_ACTIONS[name] = fn
        return fn
    return register


def _digest(path):
    try:
        return oracles.digest_of(pathlib.Path(path).read_bytes())
    except OSError:
        return None


def _landed(action, ok, detail, **evidence):
    """One structured harness fact about whether the forced state arrived.

    N-5: every fact is DURABLE, so it carries only normalised values --
    booleans, digests, sizes, mode integers, marker text from a closed set and
    basenames. No absolute path, no pid, no descriptor number and no raw
    (st_dev, st_ino) pair is ever put into one.
    """
    out = {"action": action, "landed": bool(ok), "detail": detail}
    out.update(evidence)
    return out


def _problem(what, exc):
    """A cleanup or restoration problem, normalised for durable evidence.

    ``repr(OSError)`` carries the filename, which is a private host path. Only
    the operation and the errno NAME are kept.
    """
    name = observations.ERRNO_NAMES.get(getattr(exc, "errno", None))
    return "%s: %s" % (what, name or type(exc).__name__)


def _stat_identity(path):
    st = os.stat(str(path))
    return (st.st_dev, st.st_ino)


# ------------------------------------------- the launcher, observed from outside
# At the post-pin barrier the launcher is alive, has pinned and admitted its
# object, and has not yet cloned. The harness reads that process's own
# descriptor table, fd flags, signal masks and environment from /proc. That is
# a direct observation of the state the child will be cloned from, made by the
# harness rather than reported by the mechanism, and nothing in it is published
# raw: only the booleans derived from it reach the durable record.

O_CLOEXEC_FLAG = 0o2000000          # Linux O_CLOEXEC as /proc/<pid>/fdinfo prints it
_ACCESS_MODES = {0: "O_RDONLY", 1: "O_WRONLY", 2: "O_RDWR"}


def _fdinfo_flags(pid, fd):
    """The open-file flags of ``pid``'s descriptor ``fd``, or None."""
    try:
        text = pathlib.Path("/proc/%d/fdinfo/%d" % (pid, fd)).read_text(
            encoding="ascii", errors="replace")
    except OSError:
        return None
    for line in text.splitlines():
        if line.startswith("flags:"):
            try:
                return int(line.split(":", 1)[1].strip(), 8)
            except ValueError:
                return None
    return None


def launcher_descriptors_on(pid, identity):
    """``{fd: flags}`` for every descriptor of ``pid`` on the inode ``identity``.

    Returns None when the descriptor table cannot be read at all, which is never
    the same answer as "no such descriptor".
    """
    base = "/proc/%d/fd" % pid
    try:
        names = os.listdir(base)
    except OSError:
        return None
    found = {}
    for name in names:
        try:
            if _stat_identity(os.path.join(base, name)) != identity:
                continue
        except (OSError, ValueError):
            continue
        found[int(name)] = _fdinfo_flags(pid, int(name))
    return found


def _access_mode(flags):
    return None if flags is None else _ACCESS_MODES.get(flags & 3)


def _proc_status_masks(pid):
    """``(SigBlk, SigIgn)`` of ``pid`` as integers, or ``(None, None)``."""
    try:
        text = pathlib.Path("/proc/%d/status" % pid).read_text(
            encoding="ascii", errors="replace")
    except OSError:
        return None, None
    masks = {}
    for line in text.splitlines():
        key, _, value = line.partition(":")
        if key in ("SigBlk", "SigIgn"):
            try:
                masks[key] = int(value.strip(), 16)
            except ValueError:
                return None, None
    return masks.get("SigBlk"), masks.get("SigIgn")


def _proc_environment_names(pid):
    """The NAMES in ``pid``'s environment, or None. Values are never kept."""
    try:
        raw = pathlib.Path("/proc/%d/environ" % pid).read_bytes()
    except OSError:
        return None
    return {entry.split(b"=", 1)[0].decode("utf-8", "replace")
            for entry in raw.split(b"\0") if entry}


def _signal_bit(signum):
    return 1 << (int(signum) - 1)


def _marker_text(data):
    """A marker as published: the text before the first NUL, closed set only."""
    text = bytes(data).split(b"\0", 1)[0].decode("ascii", "replace")
    return text if text in observations.PUBLISHABLE_MARKERS else "<unrecognised>"


@_post_pin("rename_over")
def _pp_rename_over(ctx, plan, built, arg):
    """E2: a DIFFERENT body is renamed over the pathname after the pin.

    The replacement is a case-private copy: renaming the canonical helper_alt
    would destroy a build artefact every later case still needs.
    """
    target = pathlib.Path(built["exec_path"])
    replacement = pathlib.Path(arg)
    before, incoming = _digest(target), _digest(replacement)
    try:
        os.replace(str(replacement), str(target))
    except OSError as exc:                                  # noqa: BLE001
        return _landed("rename_over", False, "rename failed: %r" % (exc,))
    after = _digest(target)
    return _landed("rename_over", after == incoming and after != before,
                   "the pathname now resolves to the replacement body",
                   pinned_body_sha256=before, replacement_sha256=incoming,
                   path_body_sha256_after=after)


@_post_pin("rename_away_and_unlink")
def _pp_rename_away_and_unlink(ctx, plan, built, arg):
    """E3: the pinned path is renamed away and then unlinked."""
    target = pathlib.Path(built["exec_path"])
    away = target.with_name(target.name + ".renamed-away")
    before = _digest(target)
    try:
        os.replace(str(target), str(away))
        os.unlink(str(away))
    except OSError as exc:                                  # noqa: BLE001
        return _landed("rename_away_and_unlink", False, "failed: %r" % (exc,))
    gone = not target.exists() and not away.exists()
    return _landed("rename_away_and_unlink", gone,
                   "the pathname and the renamed-away name are both gone",
                   pinned_body_sha256=before, path_exists=target.exists(),
                   renamed_away_exists=away.exists())


@_post_pin("retarget_symlink")
def _pp_retarget_symlink(ctx, plan, built, arg):
    """E4: the symlink is retargeted after the launcher resolved and pinned it."""
    link = pathlib.Path(built["exec_path"])
    new_target = pathlib.Path(arg)
    try:
        before = os.readlink(str(link))
    except OSError as exc:                                  # noqa: BLE001
        return _landed("retarget_symlink", False, "not a symlink: %r" % (exc,))
    try:
        link.unlink()
        link.symlink_to(new_target)
        after = os.readlink(str(link))
    except OSError as exc:                                  # noqa: BLE001
        return _landed("retarget_symlink", False, "retarget failed: %r" % (exc,))
    return _landed("retarget_symlink",
                   after == str(new_target) and after != before,
                   "the symlink now resolves elsewhere",
                   target_before=pathlib.PurePath(before).name,
                   target_after=pathlib.PurePath(after).name,
                   new_target_sha256=_digest(new_target))


@_post_pin("pwrite_marker")
def _pp_pwrite_marker(ctx, plan, built, arg):
    """E6: a length-preserving in-place mutation, after the measurement.

    Writes EXACTLY the declared marker bytes and reads the region back through a
    separate read-only descriptor; the case is posed only if the bytes on disk
    are the declared ones. The first Trial #2 correction wrote every byte plus
    one, so the marker E6 claims would run was never the marker it wrote.
    """
    path = pathlib.Path(built["exec_path"])
    offset, declared = int(arg), built.get("mutated_marker")
    if not isinstance(declared, (bytes, bytearray)) or not declared:
        return _landed("pwrite_marker", False,
                       "the setup recorded no marker bytes to write")
    declared = bytes(declared)
    before, size_before = _digest(path), path.stat().st_size
    write_fd = os.open(str(path), os.O_WRONLY)
    try:
        written = os.pwrite(write_fd, declared, offset)
    finally:
        os.close(write_fd)
    read_fd = os.open(str(path), os.O_RDONLY)
    try:
        readback = os.pread(read_fd, len(declared), offset)
    finally:
        os.close(read_fd)
    after, size_after = _digest(path), path.stat().st_size
    return _landed("pwrite_marker",
                   written == len(declared) and readback == declared
                   and after != before and size_after == size_before,
                   "the declared marker bytes were written in place and read "
                   "back through a separate read-only descriptor",
                   marker_written=_marker_text(declared),
                   readback_matches=readback == declared, bytes_written=written,
                   length_preserved=size_after == size_before,
                   body_sha256_before=before, body_sha256_after=after)


@_post_pin("truncate_rewrite")
def _pp_truncate_rewrite(ctx, plan, built, arg):
    """E6b: the body is truncated and rewritten, so its LENGTH changes."""
    path = pathlib.Path(built["exec_path"])
    before, size_before = _digest(path), path.stat().st_size
    fd = os.open(str(path), os.O_WRONLY | os.O_TRUNC)
    try:
        os.write(fd, b"HELM-E6B-REWRITTEN\n")
    finally:
        os.close(fd)
    after, size_after = _digest(path), path.stat().st_size
    return _landed("truncate_rewrite",
                   after != before and size_after != size_before,
                   "the body was truncated and rewritten at a new length",
                   size_before=size_before, size_after=size_after,
                   body_sha256_before=before, body_sha256_after=after)


class _SharedMapping:
    """A MAP_SHARED writable mapping made through libc, holding no descriptor.

    CPython's ``mmap.mmap`` duplicates the descriptor it is given -- its
    ``trackfd`` switch only arrived in 3.13, and the runner's Python is 3.12 --
    so closing our own descriptor left a second one open and E6c's
    ``descriptor_closed=True`` was false. libc's mmap keeps no descriptor: once
    ours is closed, the mapping is the only thing referring to the file, which
    is exactly the state E6c preregisters.
    """

    PROT_READ, PROT_WRITE, MAP_SHARED, MS_SYNC = 1, 2, 1, 4

    def __init__(self, fd, length):
        import ctypes
        self._ctypes = ctypes
        libc = ctypes.CDLL(None, use_errno=True)
        libc.mmap.restype = ctypes.c_void_p
        libc.mmap.argtypes = (ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int,
                              ctypes.c_int, ctypes.c_int, ctypes.c_long)
        libc.munmap.argtypes = (ctypes.c_void_p, ctypes.c_size_t)
        libc.msync.argtypes = (ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int)
        addr = libc.mmap(None, length, self.PROT_READ | self.PROT_WRITE,
                         self.MAP_SHARED, fd, 0)
        if addr is None or addr == ctypes.c_void_p(-1).value:
            raise OSError(ctypes.get_errno(), "mmap failed")
        self._libc, self._addr, self._length = libc, addr, length

    def write(self, offset, data):
        if self._addr is None or offset < 0 or offset + len(data) > self._length:
            raise ValueError("write outside the mapping")
        self._ctypes.memmove(self._addr + offset, bytes(data), len(data))
        if self._libc.msync(self._addr, self._length, self.MS_SYNC) != 0:
            raise OSError(self._ctypes.get_errno(), "msync failed")

    def close(self):
        if self._addr is None:
            return
        addr, self._addr = self._addr, None
        if self._libc.munmap(addr, self._length) != 0:
            raise OSError(self._ctypes.get_errno(), "munmap failed")


def _shared_mapping_of(identity):
    """Whether this process has a SHARED WRITABLE mapping of ``identity``'s inode.

    Read from /proc/self/maps, whose inode column is decimal; None if the maps
    cannot be read.
    """
    try:
        text = pathlib.Path("/proc/self/maps").read_text(
            encoding="utf-8", errors="replace")
    except OSError:
        return None
    for line in text.splitlines():
        parts = line.split()
        if len(parts) >= 5 and parts[4] == str(identity[1]) \
                and "w" in parts[1] and "s" in parts[1]:
            return True
    return False


@_post_pin("mmap_write")
def _pp_mmap_write(ctx, plan, built, arg):
    """E6c: a SHARED writable mapping mutates the body while the fd closes.

    The descriptor is closed immediately and the mapping is kept -- that exact
    lifetime is the case, and both halves are now PROVEN rather than asserted:
    no descriptor of this process refers to the inode, and a shared writable
    mapping of it is present. The mutation lands in the guarded marker region,
    as E6's does, so the body stays ELF-valid; the first correction rewrote ELF
    byte 0, which would have turned "the mapping did not deny the write" into
    ENOEXEC, outside the case's safe set. The mapping is released in cleanup,
    after the launcher has returned.
    """
    path = pathlib.Path(built["exec_path"])
    declared = built.get("mutated_marker")
    if not isinstance(declared, (bytes, bytearray)) or not declared:
        return _landed("mmap_write", False,
                       "the setup recorded no marker bytes to write")
    before, identity = _digest(path), _stat_identity(path)
    fd = os.open(str(path), os.O_RDWR)
    try:
        mapping = _SharedMapping(fd, path.stat().st_size)
    except Exception as exc:                                # noqa: BLE001
        os.close(fd)
        return _landed("mmap_write", False, _problem("mmap failed", exc))
    os.close(fd)                       # the DESCRIPTOR goes; the mapping stays
    try:
        mapping.write(int(arg), declared)
    except Exception as exc:                                # noqa: BLE001
        mapping.close()
        return _landed("mmap_write", False,
                       _problem("write through the mapping failed", exc))
    built.setdefault("_open_mappings", []).append(mapping)
    after = _digest(path)
    held = launcher_descriptors_on(os.getpid(), identity)
    closed = held == {}
    mapped = _shared_mapping_of(identity) is True
    return _landed("mmap_write", after != before and closed and mapped,
                   "the body was mutated through a shared writable mapping, and "
                   "no descriptor of the harness refers to the file any more",
                   descriptor_closed=closed, mapping_retained=mapped,
                   marker_written=_marker_text(declared),
                   body_sha256_before=before, body_sha256_after=after)


@_post_pin("fchmod")
def _pp_fchmod(ctx, plan, built, arg):
    """E6d and X1: the mode is changed after the admission fact was recorded."""
    path = pathlib.Path(built["exec_path"])
    before = path.stat().st_mode & 0o7777
    try:
        os.chmod(str(path), int(arg))
    except OSError as exc:                                  # noqa: BLE001
        return _landed("fchmod", False, "chmod failed: %r" % (exc,))
    after = path.stat().st_mode & 0o7777
    return _landed("fchmod", after == (int(arg) & 0o7777) and after != before,
                   "the mode changed after admission",
                   mode_before=before, mode_after=after)


@_post_pin("chmod_work_dir")
def _pp_chmod_work_dir(ctx, plan, built, arg):
    """S2 / S7: the capability's directory goes to mode 0000 AFTER it was opened.

    The frozen text says the capability's own descriptor is fchmod'ed. Mode is a
    property of the inode, so the harness changes it through the path and first
    proves that the launcher already holds a descriptor on that very inode. A
    change made before the launcher opened the directory would not pose the
    case: the open itself would fail, and nothing about fchdir would be tested.
    """
    work = pathlib.Path(built["work_dir"])
    pid = built.get("_launcher_pid")
    identity = _stat_identity(work)
    held = launcher_descriptors_on(pid, identity) if pid else None
    if not held:
        return _landed("chmod_work_dir", False,
                       "the launcher holds no descriptor on the working "
                       "directory, so a mode change would not follow the "
                       "capability", capability_open_in_launcher=False)
    before = work.stat().st_mode & 0o7777
    try:
        os.chmod(str(work), int(arg))
    except OSError as exc:                                  # noqa: BLE001
        return _landed("chmod_work_dir", False, _problem("chmod failed", exc),
                       capability_open_in_launcher=True)
    after = work.stat().st_mode & 0o7777
    return _landed("chmod_work_dir",
                   after == (int(arg) & 0o7777) and after != before,
                   "the directory the launcher already holds open changed mode",
                   capability_open_in_launcher=True, mode_before=before,
                   mode_after=after)


def _hold_writer(ctx, plan, built):
    """E5: a second O_WRONLY descriptor on the SAME inode, held across exec.

    N-3. The relation that matters is between this writer and the object the
    LAUNCHER pinned, so it is proved against the launcher's own descriptor
    table at the barrier: the writer's (st_dev, st_ino), taken through the
    writer itself, must be the inode of a read-only descriptor the launcher
    holds. Pathname equality is never the evidence, and the raw pair is used
    here and never recorded -- the durable fact is the relation.
    """
    path = pathlib.Path(built["exec_path"])
    pid = built.get("_launcher_pid")
    try:
        fd = os.open(str(path), os.O_WRONLY)
    except OSError as exc:                                  # noqa: BLE001
        return _landed("hold_writer", False,
                       _problem("could not open a writer", exc))
    try:
        st = os.fstat(fd)
    except OSError as exc:                                  # noqa: BLE001
        os.close(fd)
        return _landed("hold_writer", False, _problem("fstat failed", exc))
    held = (launcher_descriptors_on(pid, (st.st_dev, st.st_ino))
            if pid else None)
    modes = sorted({_access_mode(flags) for flags in (held or {}).values()}
                   - {None})
    same = "O_RDONLY" in modes
    if not same:
        os.close(fd)
        return _landed("hold_writer", False,
                       "the writer's inode is not one the launcher holds pinned",
                       access_mode="O_WRONLY", launcher_holds_same_inode=False)
    built.setdefault("_open_writers", []).append(fd)
    return _landed("hold_writer", True,
                   "an O_WRONLY descriptor is held across exec on the inode the "
                   "launcher pinned",
                   access_mode="O_WRONLY", launcher_holds_same_inode=True,
                   launcher_access_modes=modes)


def perform_post_pin(ctx, plan, built):
    """Every forced state this case declares, at the barrier. Structured facts.

    Returns ``(landed, [evidence...])``. A case whose forced state cannot be
    proven to have landed is NOT posed: running it anyway is how Trial #1's
    E-series produced results about manipulations that never happened.
    """
    facts = []
    directive = built.get("post_pin")
    if directive:
        name, arg = directive
        action = POST_PIN_ACTIONS.get(name)
        if action is None:
            facts.append(_landed(name, False, "no implementation for this "
                                              "post-pin action"))
        else:
            try:
                facts.append(action(ctx, plan, built, arg))
            except Exception as exc:                        # noqa: BLE001
                facts.append(_landed(name, False, "action raised: %r" % (exc,)))
    if built.get("hold_writer"):
        try:
            facts.append(_hold_writer(ctx, plan, built))
        except Exception as exc:                            # noqa: BLE001
            facts.append(_landed("hold_writer", False, "raised: %r" % (exc,)))
    return all(f["landed"] for f in facts) and bool(facts), facts


def needs_post_pin(built):
    return bool(built.get("post_pin") or built.get("hold_writer"))


def needs_barrier(built, applied=None):
    """Whether this launch must stop at the post-pin barrier.

    Either the setup declares a change that must land after the pin, or the
    parent state must be PROVEN present in the launcher before clone3.
    """
    return needs_post_pin(built) or bool(applied is not None
                                         and applied.needs_barrier)


def release_setup_resources(built):
    """Deterministic cleanup of everything a setup held open or created.

    Runs after the case's status is final, and its failures are recorded
    separately: a cleanup problem must never rewrite a mechanism result.
    """
    problems = []
    for fd in built.pop("_open_writers", []):
        try:
            os.close(fd)
        except OSError as exc:                              # noqa: BLE001
            problems.append(_problem("writer close failed", exc))
    for mapping in built.pop("_open_mappings", []):
        try:
            mapping.close()
        except (OSError, ValueError) as exc:                # noqa: BLE001
            problems.append(_problem("mapping release failed", exc))
    work = built.get("work_dir")
    if work:
        try:
            os.chmod(str(work), 0o755)
            os.rmdir(str(work))
        except FileNotFoundError:
            pass
        except OSError as exc:                              # noqa: BLE001
            problems.append(_problem("working directory cleanup failed", exc))
    target = built.get("cleanup")
    if target:
        try:
            pathlib.Path(target).unlink()
        except FileNotFoundError:
            pass
        except OSError as exc:                              # noqa: BLE001
            problems.append(_problem("cleanup unlink failed", exc))
    # AB7-B1: the pre-armed fixture signal's reader and FIFO, in case an
    # exception skipped _run_once's disarm. Idempotent.
    problems += disarm_fixture_signal(built)
    return problems


def restore_after_trial(built):
    """Undo a per-trial forced state so the NEXT repeated trial can be posed.

    S7 repeats S2's construction 200 times. A directory left at mode 0000 would
    make the next launcher's open(work_dir) fail, so each trial would test the
    open rather than fchdir. Problems are recorded, never folded into a result.
    """
    problems = []
    work = built.get("work_dir")
    if work:
        try:
            os.chmod(str(work), 0o755)
        except OSError as exc:                              # noqa: BLE001
            problems.append(_problem("working directory mode restore failed", exc))
    return problems


# ---------------------------------------------------------------- setup steps
# Each returns a dict describing what it built: at minimum {"exec_path": ...},
# or {"not_posed": reason}. They touch only the disposable build directory.

def _verified_source(ctx, name):
    """``(path, None)`` for a built object, or ``(None, reason)``.

    The digest is compared against the identity the runner hashed BEFORE the
    first case. Trial #1 could not say which bytes it had run; this makes the
    answer a checked fact at the moment of use rather than a claim made at the
    end. The driver never builds anything -- it only opens what the harness
    produced -- so a mismatch here means the object changed underneath the
    trial, which is not a mechanism result and must not be scored as one.
    """
    path = ctx.build / name
    identity = getattr(ctx, "build_identity", None)
    if not identity:
        return None, ("no build identity was recorded before the first case, "
                      "so the bytes being run cannot be tied to the trial")
    recorded = identity.get(name)
    if recorded is None:
        return None, "the trial recorded no build identity for " + str(name)
    try:
        actual = oracles.digest_of(path.read_bytes())
    except OSError as exc:                                 # noqa: BLE001
        return None, "the built object %s could not be read: %r" % (name, exc)
    if actual != recorded.get("sha256"):
        return None, ("the built object %s is not the one this trial hashed "
                      "before its first case" % name)
    return path, None


def verify_build_identity(ctx):
    """Every recorded artefact, re-hashed. Returns the deviations, if any.

    Called once before the first case, so a build that changed between hashing
    and posing halts BEFORE the immutability boundary rather than producing
    evidence about unknown bytes.
    """
    identity = getattr(ctx, "build_identity", None) or {}
    deviations = []
    for name in sorted(identity):
        path, reason = _verified_source(ctx, name)
        if path is None:
            deviations.append({"artefact": name, "detail": reason})
    return deviations




# ======================================= T2-R3: build identity at point of use
# One central binding rather than a check bolted onto whichever setups happened
# to have one. Every case's starting object is tied to the identity hashed
# before the first case, and its relationship to that identity is CLASSIFIED
# rather than assumed -- because a case that deliberately mutates its own copy
# must not be required to keep the base digest afterwards, and a case that
# quietly ran different bytes must not pass.

DIRECT_BASE = "DIRECT_BASE"
BYTE_IDENTICAL_CASE_COPY = "BYTE_IDENTICAL_CASE_COPY"
INTENTIONAL_MUTATION_TARGET = "INTENTIONAL_MUTATION_TARGET"
NON_BUILD_OBJECT = "NON_BUILD_OBJECT"
SYMLINK_TO_BASE = "SYMLINK_TO_BASE"

# Setups whose object cannot be classified from its name. Everything else is
# derived: a basename the build identity knows is a DIRECT_BASE, and a
# "<CASE>_<artefact>" name is that artefact's case-private copy. A setup that
# matches neither and is not declared here does not pose its case, which is what
# "explicitly classified, never accidentally skipped" has to mean in code.
SETUP_OBJECT_CLASS = {
    "symlink_retarget": (SYMLINK_TO_BASE, "helper_report"),
    "noexec_copy": (BYTE_IDENTICAL_CASE_COPY, "helper_report"),
    "directory_capability": (NON_BUILD_OBJECT, None),
    "privileged_setid": (NON_BUILD_OBJECT, None),
}

# Setups that deliberately change their object AFTER the starting check. The
# starting bytes are still bound; the base digest is simply not required to
# survive the change the case exists to make.
MUTATING_SETUPS = frozenset({
    "rename_alt_over", "rename_then_unlink", "symlink_retarget",
    "mutate_marker_in_place", "truncate_and_rewrite",
    "shared_writable_mapping", "fchmod_zero_after_admission",
    "never_executable",
})


def _classify_object(ctx, plan, path):
    """``(classification, base artefact name)`` for a case's starting object."""
    declared = SETUP_OBJECT_CLASS.get(plan.setup)
    if declared is not None:
        return declared
    name = pathlib.Path(path).name
    identity = getattr(ctx, "build_identity", None) or {}
    if name in identity:
        return (INTENTIONAL_MUTATION_TARGET if plan.setup in MUTATING_SETUPS
                else DIRECT_BASE), name
    prefix = plan.case + "_"
    if name.startswith(prefix) and name[len(prefix):] in identity:
        return (INTENTIONAL_MUTATION_TARGET if plan.setup in MUTATING_SETUPS
                else BYTE_IDENTICAL_CASE_COPY), name[len(prefix):]
    return None, None


def bind_build_identity(ctx, plan, built):
    """Tie this case's starting object to the pre-boundary build identity.

    Returns a structured fact. ``bound`` false means the case is not posed:
    evidence about bytes the trial cannot name is not evidence.
    """
    path = built.get("exec_path")
    if not path:
        return {"bound": False, "classification": None,
                "detail": "the setup produced no exec_path"}
    classification, base = _classify_object(ctx, plan, path)
    if classification is None:
        return {"bound": False, "classification": None,
                "detail": "the object %s is not classified against the build "
                          "identity" % pathlib.Path(path).name}
    fact = {"classification": classification, "base_artefact": base,
            "object": pathlib.Path(path).name}
    if classification == NON_BUILD_OBJECT:
        fact.update(bound=True, detail="not a build artefact, classified "
                                       "explicitly rather than skipped")
        return fact
    identity = (getattr(ctx, "build_identity", None) or {}).get(base)
    if identity is None:
        fact.update(bound=False, detail="the trial recorded no build identity "
                                        "for " + str(base))
        return fact
    resolved = path
    if classification == SYMLINK_TO_BASE:
        try:
            resolved = os.path.realpath(path)
        except OSError as exc:                              # noqa: BLE001
            fact.update(bound=False, detail="unreadable symlink: %r" % (exc,))
            return fact
        if pathlib.Path(resolved).name != base:
            fact.update(bound=False, detail="the symlink does not start at " + base)
            return fact
    actual = _digest(resolved)
    fact.update(base_sha256=identity.get("sha256"), object_sha256=actual,
                bound=actual is not None and actual == identity.get("sha256"))
    fact["detail"] = ("the starting bytes are the artefact this trial hashed"
                      if fact["bound"] else
                      "the starting bytes are not the artefact this trial hashed")
    return fact


@_setup("none")
def _setup_none(ctx, plan):
    path, reason = _verified_source(ctx, plan.binary)
    if path is None:
        return {"not_posed": reason}
    return {"exec_path": str(path)}


@_setup("copy")
def _setup_copy(ctx, plan):
    """A private copy, so a case that mutates an inode cannot disturb another."""
    source, reason = _verified_source(ctx, plan.binary)
    if source is None:
        return {"not_posed": reason}
    target = ctx.build / (plan.case + "_" + plan.binary)
    shutil.copy2(source, target)
    return {"exec_path": str(target)}


@_setup("rename_alt_over")
def _setup_rename_alt_over(ctx, plan):
    """E2: rename helper_alt OVER the pathname after the pin.

    The substitution happens between the launcher's open() and its execveat, and
    the pinned descriptor must keep running helper_report.
    """
    info = _setup_copy(ctx, plan)
    if "not_posed" in info:
        return info
    # A case-private replacement. Renaming the canonical helper_alt would
    # consume a build artefact every later case still needs, and a trial that
    # eats its own inputs is not repeatable.
    alt, reason = _verified_source(ctx, "helper_alt")
    if alt is None:
        return {"not_posed": reason}
    private = ctx.build / (plan.case + "_replacement_helper_alt")
    shutil.copy2(alt, private)
    return dict(info, post_pin=("rename_over", str(private)))


@_setup("rename_then_unlink")
def _setup_rename_then_unlink(ctx, plan):
    """E3: the original pathname is renamed away and then unlinked."""
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("rename_away_and_unlink", None))


@_setup("symlink_retarget")
def _setup_symlink_retarget(ctx, plan):
    """E4: the pinned leaf was a symlink whose target is retargeted after the pin."""
    base, reason = _verified_source(ctx, "helper_report")
    if base is None:
        return {"not_posed": reason}
    alt, reason = _verified_source(ctx, "helper_alt")
    if alt is None:
        return {"not_posed": reason}
    private = ctx.build / (plan.case + "_retarget_helper_alt")
    shutil.copy2(alt, private)
    link = ctx.build / (plan.case + "_link")
    if link.is_symlink() or link.exists():
        link.unlink()
    link.symlink_to(base)
    return {"exec_path": str(link),
            "post_pin": ("retarget_symlink", str(private))}


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
    """E5b: the writer opens O_WRONLY, writes and CLOSES before launch.

    No ETXTBSY is expected, because the write-deny reference is taken at exec
    time and this writer is gone by then.

    **Trial #1 aborted here** with ``OSError: [Errno 9] Bad file descriptor``.
    The byte was read back through the write-only descriptor, and a write-only
    file description carries no ``FMODE_READ``: the kernel refuses the read
    before it reaches the filesystem, which is what ``read(2)`` means by "not
    open for reading". The descriptor was valid; the access mode was wrong.

    The case under test is *a real O_WRONLY writer*, so the fix does not widen
    the descriptor to ``O_RDWR`` -- that would quietly change what E5b tests.
    The byte is read first, through a separate read-only descriptor that is
    closed before the writer is opened, and then written back unchanged. The
    executable stays byte-identical, a genuine write happens through a
    write-only descriptor, and the writer closes before exec.

    A short read or a short write does not continue: the case is not posed, and
    says so.
    """
    info = _setup_copy(ctx, plan)
    path = info["exec_path"]

    read_fd = os.open(path, os.O_RDONLY)
    try:
        original = os.pread(read_fd, 1, 0)
    finally:
        os.close(read_fd)
    if len(original) != 1:
        return {"not_posed": "E5b could not read the byte it rewrites: %d "
                             "bytes read, exactly 1 required" % len(original)}

    write_fd = os.open(path, os.O_WRONLY)
    try:
        written = os.pwrite(write_fd, original, 0)
    finally:
        os.close(write_fd)
    if written != 1:
        return {"not_posed": "E5b's no-op rewrite wrote %d bytes, exactly 1 "
                             "required" % written}
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


MARKER_REGION_BYTES = 16


def _marker_bytes(text):
    """A marker as the 16 NUL-padded bytes of helper_report's guarded region."""
    raw = str(text).encode("ascii")
    if not raw or len(raw) >= MARKER_REGION_BYTES:
        raise ValueError("a marker must fit the guarded region with its NUL")
    return raw.ljust(MARKER_REGION_BYTES, b"\0")


@_setup("mutate_marker_in_place")
def _setup_mutate_marker(ctx, plan):
    """E6: length-preserving, ELF-valid in-place mutation of the marker region."""
    info = _setup_copy(ctx, plan)
    offset = find_marker_region(info["exec_path"])
    if offset is None:
        return {"not_posed": "the guarded marker region is not uniquely "
                             "locatable in the built helper_report image"}
    # The exact bytes the harness writes, derived from the one declared marker
    # E6's executed-marker assertion expects, so the two cannot drift apart.
    return dict(info, post_pin=("pwrite_marker", offset),
                mutated_marker=_marker_bytes(plan.expected_marker))


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
    if "not_posed" in info:
        return info
    offset = find_marker_region(info["exec_path"])
    if offset is None:
        return {"not_posed": "the guarded marker region is not uniquely "
                             "locatable in the built helper_report image"}
    return dict(info, post_pin=("mmap_write", offset),
                mutated_marker=_marker_bytes(observations.E6_MUTATED_MARKER))


@_setup("fchmod_zero_after_admission")
def _setup_fchmod_zero(ctx, plan):
    """E6d / X1: mode bits cleared AFTER admission recorded them."""
    info = _setup_copy(ctx, plan)
    return dict(info, post_pin=("fchmod", 0))


@_setup("work_dir_fchmod_zero")
def _setup_work_dir_fchmod_zero(ctx, plan):
    """S2 / S7: a case-private working directory, taken to mode 0000 post-pin.

    N-4b. These cases used to declare ``work_dir_kind="fchmod_zero"``, which
    nothing read: the launcher got the ordinary build directory, the helper ran
    and reported, and both cases were INVALID by construction. The directory is
    now the launcher's capability and the barrier changes its mode after the
    launcher has opened it, which is the only order in which the child's fchdir
    is what fails.
    """
    info = _setup_none(ctx, plan)
    if "not_posed" in info:
        return info
    work = ctx.build / (plan.case + "_workdir")
    if work.exists():
        os.chmod(str(work), 0o755)
        shutil.rmtree(str(work))
    work.mkdir(mode=0o755)
    return dict(info, work_dir=str(work), post_pin=("chmod_work_dir", 0))


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


# The one byte helper_fork.c's descendant writes, once, on its fixture path.
FIXTURE_SIGNAL_BYTE = b"L"
# How long, after launch() returned, the harness waits for a signal that is not
# already buffered. The byte is normally in the pipe before launch() returns;
# the bound only keeps an absent signal from holding the trial.
FIXTURE_SIGNAL_READ_TIMEOUT_MS = 2000
# Where the armed read end is held between the spawn and the read. Harness
# state, never a setup-result key, never passed to any process.
_FIXTURE_FD = "_fixture_signal_fd"


@_setup("fork_helper_prearmed")
def _setup_fork_helper_prearmed(ctx, plan):
    """O6 and O7: helper_fork with a fixture signal armed BEFORE the launch.

    AB7-B1. The P-series rendezvous above opens its read end only after
    launch() returned, which proves survival. A descendant that stays in the
    direct child's process group never survives that long: the launcher's
    group sweep, issued before its reap, kills it while it is still blocked in
    open(). O6 and O7 do not ask whether the descendant survived. They ask
    whether the fixture exists -- whether helper_fork executed, forked, and its
    descendant reached the signalling path. So the harness opens this FIFO's
    read end before the launcher is spawned (_arm_fixture_signal), the
    descendant's open() completes at once, its one byte waits in the pipe, and
    the harness reads it after launch() returned (_read_fixture_signal). The
    launcher's sweep is untouched, no --setsid is added and helper_fork.c is
    unchanged.

    Before the boundary this only names the case-private path and removes any
    node an earlier run left there. The FIFO itself is created fresh for each
    launcher invocation.
    """
    fifo = ctx.build / (plan.case + ".fixture-signal")
    try:
        _remove_fifo(fifo)
    except OSError as exc:                                  # noqa: BLE001
        return {"not_posed": _problem("a stale fixture signal FIFO could not "
                                      "be removed", exc)}
    return {"exec_path": str(ctx.build / "helper_fork"),
            "fixture_signal_fifo": str(fifo),
            "extra_helper_args": ("--liveness-fifo", str(fifo))}


def _remove_fifo(path):
    try:
        os.unlink(str(path))
    except FileNotFoundError:
        pass


def _arm_fixture_signal(built, pass_fds=()):
    """Create the case-private FIFO afresh and open its read end, before the spawn.

    ``(facts, None)`` once armed, ``(None, reason)`` when it cannot be armed,
    and ``(None, None)`` when the setup declares no fixture signal. The read
    end is harness infrastructure and nothing else: opened O_RDONLY |
    O_NONBLOCK | O_CLOEXEC, checked non-inheritable, checked absent from the
    descriptors handed to the launcher, and held only in this process, so it
    can enter neither launcher_spike nor helper_fork. The node is new for every
    launcher invocation, so a writer of any earlier node cannot reach it, and a
    fresh pipe holds no byte -- which is checked rather than assumed. A reader
    that fails any check stays in ``built`` only until disarm_fixture_signal
    closes it.
    """
    stale_fd = built.pop(_FIXTURE_FD, None)
    if stale_fd is not None:
        with contextlib.suppress(OSError):
            os.close(stale_fd)
    path = built.get("fixture_signal_fifo")
    if not path:
        return None, None
    try:
        _remove_fifo(path)
        os.mkfifo(path, 0o600)
        fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_CLOEXEC)
    except OSError as exc:                                  # noqa: BLE001
        return None, _problem("the fixture signal could not be armed", exc)
    built[_FIXTURE_FD] = fd
    try:
        is_fifo = stat.S_ISFIFO(os.fstat(fd).st_mode)
        inheritable = os.get_inheritable(fd)
        poller = select.poll()
        poller.register(fd, select.POLLIN)
        stale = bool(poller.poll(0))
    except OSError as exc:                                  # noqa: BLE001
        return None, _problem("the fixture signal could not be verified", exc)
    passed = fd in tuple(pass_fds)
    if not is_fifo or inheritable or passed or stale:
        return None, ("the fixture signal reader is not a fresh, "
                      "non-inheritable FIFO held only by the harness")
    return {"armed_before_launch": True, "reader_inheritable": inheritable,
            "reader_passed_to_launcher": passed, "fresh_fifo": not stale}, None


def _read_fixture_signal(built, timeout_ms=FIXTURE_SIGNAL_READ_TIMEOUT_MS):
    """Whether exactly the frozen signal byte arrived; None if nothing was armed.

    Read after launch() returned, from the read end opened before the spawn, so
    a byte the descendant wrote before the launcher's sweep is still buffered.
    True only for exactly FIXTURE_SIGNAL_BYTE; nothing, or anything else, is
    False. Bounded: it never waits past ``timeout_ms`` in total and reads at
    most one byte more than the signal. It only reads; the harness never writes
    the FIFO.
    """
    fd = built.get(_FIXTURE_FD)
    if fd is None:
        return None
    data = b""
    deadline = time.monotonic() + timeout_ms / 1000.0
    try:
        poller = select.poll()
        poller.register(fd, select.POLLIN)
        while len(data) <= len(FIXTURE_SIGNAL_BYTE):
            remaining = max(0, int((deadline - time.monotonic()) * 1000))
            if not poller.poll(remaining):
                break
            try:
                chunk = os.read(fd, len(FIXTURE_SIGNAL_BYTE) + 1)
            except BlockingIOError:
                if remaining == 0:
                    break
                continue
            if not chunk:
                break
            data += chunk
    except OSError:                                         # noqa: BLE001
        return False
    return data == FIXTURE_SIGNAL_BYTE


def disarm_fixture_signal(built):
    """Close the harness's read end and remove the FIFO. Idempotent.

    Called after every launcher invocation (_run_once) and again by
    release_setup_resources, so neither the descriptor nor the node outlives
    the case, and no later case or repetition can read this one's signal.
    """
    problems = []
    fd = built.pop(_FIXTURE_FD, None)
    if fd is not None:
        try:
            os.close(fd)
        except OSError as exc:                              # noqa: BLE001
            problems.append(_problem("fixture signal reader close failed", exc))
    path = built.get("fixture_signal_fifo")
    if path:
        try:
            _remove_fifo(path)
        except OSError as exc:                              # noqa: BLE001
            problems.append(_problem("fixture signal FIFO removal failed", exc))
    return problems


@_setup("privileged_setid")
def _setup_privileged_setid(ctx, plan):
    """N3: expected BLOCKED. No privileged fixture is created, ever.

    Reaching this setup at all would mean an authorised privileged identity
    existed, which this experiment never manufactures. It returns ``not_posed``
    rather than inventing one, so a mis-derived block cannot become a result.
    """
    return {"not_posed": "no privileged identity exists and none is manufactured"}


# ------------------------------------------------------------- parent states
# N-1. The Trial #2 delta review found every parent state except V1's
# environment declared and never applied: F2 had no stray descriptor, F3 no
# CLOEXEC one, F5 and T6 no blocked or ignored signal, F6 its stdio, F7 no
# adjacent pair, R4 and M5 no ignored SIGCHLD. F2 -- mandatory, instant-reject,
# "the ONLY pass" -- would have passed vacuously. The dictionary each of these
# functions returned was never proof of anything, because nothing read it.
#
# A parent state is now a closed contract. Every key is declared in
# PARENT_STATE_SCHEMA, and every key has ONE consumer that creates the state
# (AppliedParentState) and ONE prover that observes it; a test fails if either
# is missing. The state is created in the LAUNCHER process -- the caller whose
# state each case describes -- and, except for M2's threads, proven from
# outside in that process's own /proc entries at the post-pin barrier, before
# clone3. A state that cannot be proven there does not pose its case.

PS_ENV = "env"
PS_INHERIT_FD = "inherit_fd"
PS_BLOCK = "block_signals"
PS_IGNORE = "ignore_signals"
PS_CLOSE_LOW = "close_low_fds"
PS_THREADS = "launcher_threads"

PARENT_STATE_SCHEMA = {
    PS_ENV: "the launcher's spawn environment; proven by the NAMES in "
            "/proc/<launcher>/environ",
    PS_INHERIT_FD: "an unrelated descriptor passed into the launcher, made "
                   "CLOEXEC there by --parent-fd-set-cloexec when declared; "
                   "proven by the launcher's descriptor table and fdinfo flags",
    PS_BLOCK: "signals blocked in the harness only across the spawn and "
              "inherited; proven by the launcher's SigBlk",
    PS_IGNORE: "signals set to SIG_IGN in the harness only across the spawn, "
               "with restore_signals off; proven by the launcher's SigIgn",
    PS_CLOSE_LOW: "descriptors 0..K-1 closed in the launcher by "
                  "--parent-close-low-fds before the pin; proven by what the "
                  "launcher's low descriptors refer to",
    PS_THREADS: "extra live launcher threads from the plan's --extra-threads; "
                "proven by the receipt's parent_shape against the control arm",
}

# Keys proven live at the barrier. PS_THREADS is proven after the run, from the
# receipt, by the threaded_parent_observed posed check.
BARRIER_PROVEN_STATE_KEYS = frozenset({PS_ENV, PS_INHERIT_FD, PS_BLOCK,
                                       PS_IGNORE, PS_CLOSE_LOW})

# Linux x86-64 signal numbers from the one closed table observations.py owns, so
# a parent state never depends on the harness host's own signal module.
_SIG = {name: number for number, name in observations.SIGNAL_NAMES.items()}


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
    return {PS_ENV: {"HELM_LEAK_CANARY": "canary",
                    "LD_LIBRARY_PATH": "/nonexistent",
                    "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
                    "HOME": str(ctx.build)}}


@_parent("non_cloexec_fd")
def _parent_non_cloexec(ctx, plan):
    """F2: an unrelated NON-CLOEXEC descriptor open in the parent at exec time.

    Under frozen D-1 arm (i) the child not seeing it is the ONLY pass; there is
    no documented-failure acceptance path.
    """
    # Passed into the launcher by pass_fds, so it is open and NOT CLOEXEC there
    # at clone3. Opened through harness.open_non_cloexec_descriptor, which
    # existed for exactly this and was never called.
    return {PS_INHERIT_FD: {"path": str(ctx.build / "helper_report"),
                            "cloexec": False}}


@_parent("cloexec_fd")
def _parent_cloexec(ctx, plan):
    """F3: an unrelated descriptor that is CLOEXEC in the launcher at clone3.

    The harness can hand a descriptor over but cannot mark it CLOEXEC inside
    another process, so the launcher does that once, at startup, under the
    TEST/CONTROL-ONLY --parent-fd-set-cloexec; the barrier proves the flag.
    """
    return {PS_INHERIT_FD: {"path": str(ctx.build / "helper_report"),
                            "cloexec": True}}


@_parent("block_and_ignore_signals")
def _parent_signals(ctx, plan):
    """F5: block {SIGTERM, SIGUSR1} and ignore {SIGPIPE, SIGUSR2}."""
    return {PS_BLOCK: [_SIG["SIGTERM"], _SIG["SIGUSR1"]],
            PS_IGNORE: [_SIG["SIGPIPE"], _SIG["SIGUSR2"]]}


@_parent("close_stdio")
def _parent_close_stdio(ctx, plan):
    """F6: close 0, 1 and 2 so the launcher's own pipes and exec fd can land there.

    Closed in the launcher at startup by --parent-close-low-fds 3, which first
    saves the receipt channel above 2. The barrier proves the launcher's exec
    descriptor is 0 and its working-directory capability is 1.
    """
    return {PS_CLOSE_LOW: 3}


@_parent("adjacent_fds")
def _parent_adjacent_fds(ctx, plan):
    """F7: force the exec fd and the status write end to adjacent numbers, so a
    close_range gap computation that inverted its bounds would return EINVAL.

    Descriptor 0 is closed in the launcher before the pin, so the exec
    descriptor opens as 0 and is relocated above 2 AFTER every pipe exists --
    to the number right after the status write end. The barrier proves the exec
    descriptor opened as 0; the launcher's own pre-clone pipe2 and F_DUPFD
    records then establish the pair (posed check exec_status_pair_adjacent).
    The child's close_range calls are F7's result and never its proof: the
    first proof read them, and so recorded the EINVAL F7 exists to catch as a
    state that did not materialise (F417-B4).
    The former "pad_descriptors" directive was never applied and could not have
    produced adjacency: lowest-free allocation puts the status end far above the
    exec descriptor however the table is padded.
    """
    return {PS_CLOSE_LOW: 1}


@_parent("sigchld_ignore")
def _parent_sigchld_ignore(ctx, plan):
    """R4 / M5: SIGCHLD set to SIG_IGN -- a caller precondition a crate cannot
    enforce, which is why R4 is RECORDED and its gates forbid reporting a status
    the launcher never observed."""
    return {PS_IGNORE: [_SIG["SIGCHLD"]]}


@_parent("sigterm_blocked_sigpipe_ignored")
def _parent_t6(ctx, plan):
    """T6: the wall-clock shape must match T2 from a default-signal parent, which
    it can only do if the child's signal state was reset."""
    return {PS_BLOCK: [_SIG["SIGTERM"]], PS_IGNORE: [_SIG["SIGPIPE"]]}


@_parent("multithreaded")
def _parent_multithreaded(ctx, plan):
    """M2: >=3 extra live threads, one allocating, one with a pthread_atfork
    handler registered.

    This describes the LAUNCHER process, not this driver. The plan's own
    --extra-threads creates it inside the launcher, and the receipt's
    parent_shape proves it against the control arm (posed check
    threaded_parent_observed); a threaded Python parent would evidence nothing
    about the launcher.
    """
    return {PS_THREADS: 3}


class AppliedParentState:
    """One declared parent state, CREATED for exactly one launcher and released.

    Construction opens any inherited descriptor and composes the caller-state
    flags; ``spawning()`` holds the harness in a declared signal state ONLY
    across the spawn; ``prove()`` observes the launcher at the barrier;
    ``release()`` closes what construction opened. Nothing it changes in the
    harness outlives the spawn, so no case's process state reaches the next.
    """

    def __init__(self, name, state):
        state = dict(state or {})
        unknown = sorted(set(state) - set(PARENT_STATE_SCHEMA))
        if unknown:
            raise ValueError("parent state %s declares keys the closed schema "
                             "does not name: %s" % (name, unknown))
        self.name = name
        self.state = state
        self.env = dict(state.get(PS_ENV) or {})
        self.block = tuple(state.get(PS_BLOCK) or ())
        self.ignore = tuple(state.get(PS_IGNORE) or ())
        self.close_low = int(state.get(PS_CLOSE_LOW) or 0)
        self.threads = state.get(PS_THREADS)
        self.spike_flags = []
        self.pass_fds = ()
        self._fd, self._fd_identity, self._fd_cloexec = None, None, False
        inherit = state.get(PS_INHERIT_FD)
        if inherit:
            fd = harness.open_non_cloexec_descriptor(inherit["path"])
            st = os.fstat(fd)
            self._fd, self._fd_identity = fd, (st.st_dev, st.st_ino)
            self._fd_cloexec = bool(inherit.get("cloexec"))
            self.pass_fds = (fd,)
            if self._fd_cloexec:
                self.spike_flags += ["--parent-fd-set-cloexec", str(fd)]
        if self.close_low:
            self.spike_flags += ["--parent-close-low-fds", str(self.close_low)]

    @property
    def restore_signals(self):
        # Popen's default resets SIGPIPE to SIG_DFL in the child, which would
        # silently undo F5 and T6's ignored SIGPIPE. It is switched off only for
        # a state that declares ignored signals; every other case keeps it.
        return not self.ignore

    @property
    def needs_barrier(self):
        return bool(set(self.state) & BARRIER_PROVEN_STATE_KEYS)

    @contextlib.contextmanager
    def spawning(self):
        """The declared signal state in the harness, for the spawn and no longer.

        The blocked mask and ignored dispositions are inherited by the launcher
        through fork and execve; the harness is restored before the launcher
        can reach its barrier, and the restoration is unconditional.
        """
        saved_mask, saved = None, []
        try:
            if self.block:
                saved_mask = signal.pthread_sigmask(signal.SIG_BLOCK, self.block)
            for signum in self.ignore:
                saved.append((signum, signal.signal(signum, signal.SIG_IGN)))
            yield
        finally:
            for signum, previous in reversed(saved):
                signal.signal(signum,
                              signal.SIG_DFL if previous is None else previous)
            if saved_mask is not None:
                signal.pthread_sigmask(signal.SIG_SETMASK, saved_mask)

    def prove(self, pid, exec_path, work_dir):
        """One landing fact per barrier-proven key, observed in the launcher.

        The REQUIREMENT is always read from the declared state, never from the
        fields the consumer derived from it, so a consumer that is bypassed or
        emptied cannot also empty what the proof demands.
        """
        facts = []
        state = self.state
        if PS_ENV in state:
            facts.append(_prove_environment(pid, sorted(state[PS_ENV])))
        if PS_INHERIT_FD in state:
            facts.append(_prove_inherited_fd(
                pid, self._fd, self._fd_identity,
                bool(state[PS_INHERIT_FD].get("cloexec"))))
        if PS_BLOCK in state or PS_IGNORE in state:
            facts.append(_prove_signal_state(
                pid, tuple(state.get(PS_BLOCK) or ()),
                tuple(state.get(PS_IGNORE) or ())))
        if PS_CLOSE_LOW in state:
            facts.append(_prove_low_fds_closed(pid, int(state[PS_CLOSE_LOW]),
                                               exec_path, work_dir))
        return facts

    def release(self):
        problems = []
        if self._fd is not None:
            fd, self._fd = self._fd, None
            try:
                os.close(fd)
            except OSError as exc:                          # noqa: BLE001
                problems.append(_problem("inherited descriptor close failed",
                                         exc))
        return problems


def _prove_environment(pid, names):
    seen = _proc_environment_names(pid)
    present = sorted(n for n in names if seen is not None and n in seen)
    ok = seen is not None and present == list(names)
    return _landed("parent_env", ok,
                   "every declared name is in the launcher's environment" if ok
                   else "the launcher's environment lacks a declared name",
                   names_present=present, all_declared_names_present=ok)


def _prove_inherited_fd(pid, fd, identity, want_cloexec):
    held = launcher_descriptors_on(pid, identity) if fd is not None else None
    flags = (held or {}).get(fd)
    present = flags is not None
    cloexec = present and bool(flags & O_CLOEXEC_FLAG)
    ok = present and cloexec == want_cloexec
    return _landed("parent_inherited_fd", ok,
                   "the unrelated descriptor is open in the launcher with the "
                   "declared close-on-exec flag" if ok else
                   "the unrelated descriptor is absent from the launcher or "
                   "carries the wrong close-on-exec flag",
                   present_in_launcher=present, cloexec_in_launcher=cloexec,
                   cloexec_required=bool(want_cloexec))


def _prove_signal_state(pid, block, ignore):
    blocked, ignored = _proc_status_masks(pid)
    names = observations.SIGNAL_NAMES
    readable = blocked is not None and ignored is not None
    missing = []
    if readable:
        missing += [names.get(s, str(s)) for s in block
                    if not blocked & _signal_bit(s)]
        missing += [names.get(s, str(s)) for s in ignore
                    if not ignored & _signal_bit(s)]
    ok = readable and not missing
    return _landed("parent_signal_state", ok,
                   "every declared signal is blocked or ignored in the launcher"
                   if ok else "the launcher's signal state is not the declared "
                              "one",
                   blocked_required=[names.get(s, str(s)) for s in block],
                   ignored_required=[names.get(s, str(s)) for s in ignore],
                   signals_missing=missing, masks_readable=readable)


def _prove_low_fds_closed(pid, k, exec_path, work_dir):
    def target(fd):
        try:
            return _stat_identity("/proc/%d/fd/%d" % (pid, fd))
        except OSError:
            return None

    try:
        pinned = _stat_identity(exec_path)
    except (OSError, TypeError):
        pinned = None
    exec_at_0 = pinned is not None and target(0) == pinned
    facts, ok = {"exec_fd_at_0": exec_at_0}, exec_at_0
    if k >= 3:
        try:
            capability = _stat_identity(work_dir)
        except (OSError, TypeError):
            capability = None
        at_1 = capability is not None and target(1) == capability
        fd2_free = not os.path.lexists("/proc/%d/fd/2" % pid)
        facts.update(work_dir_capability_at_1=at_1, fd2_free=fd2_free)
        ok = ok and at_1 and fd2_free
    return _landed("parent_low_fds_closed", ok,
                   "the launcher's own descriptors occupy the closed low slots"
                   if ok else "the launcher's low descriptors are not its own "
                              "exec descriptor and capability",
                   closed_below=k, **facts)


def _launcher_pid(proc, plan):
    """The launcher's pid: the spawned process, or strace's only child.

    At READY the launcher has not cloned, so under a tracer it is the one child
    of the strace process. Anything else is ambiguous, and an ambiguous launcher
    cannot be observed.
    """
    if not plan.traced:
        return proc.pid
    try:
        entries = os.listdir("/proc")
    except OSError:
        return None
    children = []
    for entry in entries:
        if not entry.isdigit():
            continue
        try:
            text = pathlib.Path("/proc/%s/stat" % entry).read_text(
                encoding="ascii", errors="replace")
        except OSError:
            continue
        fields = text.rsplit(")", 1)[-1].split()
        if len(fields) > 1 and fields[1] == str(proc.pid):
            children.append(int(entry))
    return children[0] if len(children) == 1 else None


# -------------------------------------------------------------- posed checks
# A posed check answers "did the forced state actually materialise?". False means
# the case was never a test of the mechanism, which is INVALID -- not FAIL.

# O7's former posed check, stderr_capture_failed, read the receipt's stderr
# completeness -- the very fact O7 tests -- so it made the result a posing
# condition; and the helper_report construction it guarded never forked, so it
# could never hold and O7 was INVALID on every run. The owner decision after
# f9bbf39 rebuilt O7 on the helper_fork retained-writer fixture, whose capture
# failure is the result assertion observations.assert_stderr_capture_failure_
# reported, beside the separately derived Exited:42.
#
# AB7-B1. Its posing proof then was fixture_descendant_alive, the P-series
# rendezvous read after launch() returned. O7's descendant stays in the direct
# child's process group, so the launcher's own pre-reap group sweep killed it
# before that read and the check could never hold. The proof is now the
# fixture signal the harness arms before the spawn.


@_check("fixture_descendant_signalled", reads=("fixture_descendant_signalled",))
def _check_fixture_descendant_signalled(obs):
    """O6 and O7: helper_fork executed, forked, and its descendant signalled.

    Established ONLY by the pre-armed fixture signal: exactly the frozen byte,
    read after launch() returned from the case-private FIFO whose read end the
    harness opened before the launcher was spawned (_arm_fixture_signal,
    _read_fixture_signal). Nothing the launcher reported is read here -- not
    stdout's or stderr's completeness, not the exit status, not the receipt's
    outcome -- because those are the results O6 and O7 exist to test. The
    signal does not claim the descendant outlived launch(); the launcher's
    normal group sweep ends it. No signal, or the wrong bytes, is not posed.
    """
    return obs.get("fixture_descendant_signalled") is True


@_check("retention_observed", reads=("descendant_alive_after_launch", "spike"))
def _check_retention_observed(obs):
    """P4: a setsid descendant really did retain the inherited write ends.

    That state is created by the executed helper, so its proof must not be only
    what the launcher CONCLUDED about it. The harness's own liveness
    rendezvous, opened only after launch() returned, proves P4's setsid
    descendant -- which the group sweep does not reach -- was alive and still
    held the pipes at that moment. A receipt reporting
    WriterRetainedAfterChildExit establishes it too, as it did before the F417
    correction. What the launcher reported about completeness is then P4's
    gate: CompleteAtEof beside a live retaining descendant fails it. O6 no
    longer uses this check (AB7-M2): its fixture is established by
    fixture_descendant_signalled and its completeness is a result.
    """
    if obs.get("descendant_alive_after_launch") is True:
        return True
    spike = obs.get("spike")
    if not isinstance(spike, dict):
        return False
    return any(isinstance(spike.get(s), dict)
               and spike[s].get("completeness") == "WriterRetainedAfterChildExit"
               for s in ("stdout", "stderr"))


@_check("no_helper_report", reads=("report_state",))
def _check_no_helper_report(obs):
    """S5: the ABSENCE of the report is the evidence the injected death landed.

    S5's forced state is created inside the launcher's own child by the frozen
    --die-before-exec mode, and nothing but a decisive absence of the report can
    show that it landed. S2 and S7 no longer use this check (F417-B3): their
    forced state is proven at the barrier, so whether an image ran is their
    RESULT -- the no_executed_image assertion, which reads the child's explicit
    CHDIR status record and the report sentinel (AB7-I1, AB7-M1) -- and not a
    posing failure.
    """
    # Only a DECISIVE absence counts. A truncated prefix or a retained writer
    # means the stream could not say whether a report was written, and treating
    # that as "no report" would turn a gap in the evidence into a finding.
    return obs.get("report_state") == observations.REPORT_ABSENT


@_check("returned_before_descendant_lifetime", reads=("elapsed_ms",))
def _check_returned_early(obs):
    """O6: launch() must return measurably before the descendant's sleep ends."""
    elapsed = obs.get("elapsed_ms")
    return isinstance(elapsed, int) and elapsed < P_DESCENDANT_LIFETIME_MS


# O3's completed_under_ten_seconds and O8's trial_floor_512 were posed checks
# over what the mechanism PRODUCED -- elapsed time and drained bytes -- so a
# slow drain or a short trial scored INVALID where the frozen rows make it a
# FAIL (F417-B2). O3's bound is now the post-rule assertion
# observations.assert_completed_under_ten_seconds; O8's floor is its own
# stream_exact rule, applied to every repetition.


# O5's former posed check, bounded_poll_and_cpu, read a key nothing produced,
# so O5 was INVALID on every run -- and it treated an over-bound drain as
# "not posed" when the frozen row makes it a failure. Both bounds are now the
# post-rule assertion observations.assert_bounded_drain: a missing measurement
# is unobservable, an exceeded bound is a FAIL.


@_check("same_inode_as_writer", reads=("post_pin_evidence",))
def _check_same_inode(obs):
    """E5: the held writer is on the inode the LAUNCHER pinned.

    Read from the hold_writer landing fact, which the barrier produces by
    comparing the writer's own (st_dev, st_ino) with the launcher's descriptor
    table. The former keys writer_inode and pinned_inode had no producer, so
    E5 could never be posed.
    """
    fact = observations._landing_fact(obs, "hold_writer")
    return bool(fact and fact.get("landed") is True
                and fact.get("launcher_holds_same_inode") is True)


@_check("exec_status_pair_adjacent", reads=("exec_status_pair_adjacent",))
def _check_exec_status_pair_adjacent(obs):
    """F7: the exec descriptor and the exec-status write end were adjacent.

    Established before the behaviour under test and without it: from the
    LAUNCHER's own pre-clone syscalls in the external record, starting from the
    exec descriptor the barrier proved at fd 0. The child's close_range calls
    are never read, so their success or their EINVAL is F7's RESULT, never its
    posing (F417-B4). No established pair is not posed.
    """
    return obs.get("exec_status_pair_adjacent") is True


@_check("threaded_parent_observed",
        reads=("spike", "baseline_observation", "declared_launcher_threads"))
def _check_threaded_parent(obs):
    """M2: the threaded arm really had its extra threads; the control had none."""
    want = obs.get("declared_launcher_threads")
    spike, baseline = obs.get("spike"), obs.get("baseline_observation")
    shape = spike.get("parent_shape") if isinstance(spike, dict) else None
    base_spike = baseline.get("spike") if isinstance(baseline, dict) else None
    base_shape = (base_spike.get("parent_shape")
                  if isinstance(base_spike, dict) else None)
    if not isinstance(want, int) or not isinstance(shape, dict) \
            or not isinstance(base_shape, dict):
        return False
    return (shape.get("extra_threads") == want
            and shape.get("atfork_handler_registered") is True
            and base_shape.get("extra_threads") == 0)


# ============================================================== the 72 plans
def _build_plans():
    p = []
    add = p.append

    # ---- E: executable identity and TOCTOU --------------------------------
    # Every E case except E5 needs the helper's own marker to establish WHICH
    # body ran, so they declare the report channel. N-2: the marker is now
    # ASSERTED, not merely present -- helper_alt prints a report too.
    pinned_body = ("executed_marker", "measured_starting_identity")
    add(CasePlan("E1", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT, CH_TRACE), setup="copy",
                 helper_args=("--exit", "0"), expected_marker="helper_report",
                 assertions=pinned_body,
                 note="the measured digest must equal the independent hashlib "
                      "digest taken before the run"))
    add(CasePlan("E2", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="rename_alt_over",
                 helper_args=("--exit", "0"), expected_marker="helper_report",
                 assertions=pinned_body,
                 note="helper_alt executing is a mechanism FAIL, never INVALID"))
    add(CasePlan("E3", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="rename_then_unlink",
                 helper_args=("--exit", "0"), expected_marker="helper_report",
                 assertions=pinned_body))
    add(CasePlan("E4", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="symlink_retarget",
                 helper_args=("--exit", "0"), expected_marker="helper_report",
                 assertions=pinned_body,
                 note="the retargeted body executing is a mechanism FAIL"))
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
                 helper_args=("--exit", "0"),
                 expected_marker=observations.E6_MUTATED_MARKER,
                 assertions=pinned_body,
                 note="the MUTATED marker runs, and the pre-exec measurement "
                      "is the starting (pre-mutation) identity"))
    add(CasePlan("E6b", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="truncate_and_rewrite",
                 body_length_changed=True))
    add(CasePlan("E6c", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="shared_writable_mapping",
                 body_length_changed=True))
    add(CasePlan("E6d", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="fchmod_zero_after_admission",
                 assertions=("mode_measured_pre_change",),
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
                 (CH_RECEIPT, CH_REPORT, CH_TRACE), parent="adjacent_fds",
                 posed_when="exec_status_pair_adjacent"))

    # ---- X: exec failure and admission ------------------------------------
    add(CasePlan("X1", "helper_report", "process_disposition", (CH_RECEIPT,),
                 setup="fchmod_zero_after_admission",
                 assertions=("mode_measured_pre_change",)))
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
                 assertions=("completed_under_ten_seconds",)))
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
                 assertions=("bounded_drain",)))
    # O6 and O7 (AB7-B1, AB7-M2): the unchanged helper_fork retained-writer
    # fixture, established by the fixture signal the harness arms BEFORE the
    # spawn. Their descendants stay in the direct child's process group and
    # the launcher's normal group sweep ends them; neither case adds --setsid.
    # What the launcher reports about completeness is each case's RESULT.
    add(CasePlan("O6", "helper_fork", "process_disposition",
                 (CH_RECEIPT, CH_PAYLOAD, CH_LIVENESS),
                 setup="fork_helper_prearmed",
                 helper_args=("--prewrite", "512", "--retain-stdio",
                              "--parent-exit", "0", "--lifetime-ms",
                              str(P_DESCENDANT_LIFETIME_MS)),
                 streams=_stream("stdout", 512, "WriterRetainedAfterChildExit"),
                 posed_when="fixture_descendant_signalled",
                 assertions=("stream_completeness_as_declared",),
                 note="posed by the pre-armed fixture signal; CompleteAtEof is "
                      "the frozen FAIL, never a posing failure"))
    # O7, by owner decision after f9bbf39. The former helper_report
    # construction could never produce a capture failure: it never forked, so
    # stderr always reached EOF. The unchanged helper_fork retained-writer
    # fixture does: its direct child exits 42 at once while a descendant keeps
    # descriptors 1 and 2. For Trial #2 the frozen "stderr capture failure" is
    # the receipt's stderr completeness WriterRetainedAfterChildExit. Stdout is
    # retained too, a fixture side-effect O7 does not score, and no payload is
    # declared because O7 freezes none.
    add(CasePlan("O7", "helper_fork", "process_disposition",
                 (CH_RECEIPT, CH_LIVENESS), setup="fork_helper_prearmed",
                 helper_args=("--retain-stdio", "--parent-exit", "42",
                              "--lifetime-ms", str(P_DESCENDANT_LIFETIME_MS)),
                 posed_when="fixture_descendant_signalled",
                 assertions=("stderr_capture_failure_reported",),
                 note="posed by the pre-armed fixture signal; Exited:42 and "
                      "stderr WriterRetainedAfterChildExit are two separate "
                      "receipt facts, and a receipt carrying only one of them "
                      "is a FAIL"))
    add(CasePlan("O8", "helper_report", "stream_exact", (CH_RECEIPT, CH_PAYLOAD),
                 helper_args=("--no-report", "--stdout", "512"),
                 streams=_stream("stdout", 512), repeat=REPEAT_TRIALS,
                 note="each of the 200 repetitions is scored by its own "
                      "stream_exact rule; a posed short or wrong drain in any "
                      "one of them is the case's FAIL"))

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
                 posed_when="retention_observed"))

    # ---- S: spawn/exec confirmation ---------------------------------------
    add(CasePlan("S1", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), helper_args=("--exit", "0"),
                 note="clean EOF alone is not a PASS; the report is the exec "
                      "evidence, which is why this case declares the channel"))
    add(CasePlan("S2", "helper_report", "process_disposition",
                 (CH_RECEIPT, CH_REPORT), setup="work_dir_fchmod_zero",
                 assertions=("no_executed_image",),
                 note="posed by the barrier's proven mode change; the report's "
                      "decisive ABSENCE is then part of the result, so an image "
                      "that ran is a FAIL"))
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
                 (CH_RECEIPT, CH_REPORT), setup="work_dir_fchmod_zero",
                 repeat=REPEAT_TRIALS, assertions=("no_executed_image",),
                 note="S2's construction 200 times; each repetition's landing is "
                      "proven at the barrier and each is scored on its own"))

    # ---- M: mechanism minimality and parent shape -------------------------
    add(CasePlan("M1", "helper_report", "child_syscalls_within_frozen_set",
                 (CH_RECEIPT, CH_TRACE), helper_args=("--exit", "0")))
    add(CasePlan("M2", "helper_report", "identical_to_single_threaded_arm",
                 (CH_RECEIPT, CH_TRACE, CH_THREADED_LAUNCHER),
                 parent="multithreaded", helper_args=("--exit", "0"),
                 spike_flags=("--extra-threads", "3"), baseline_flags=(),
                 posed_when="threaded_parent_observed",
                 note="the frozen arm exactly: >=3 extra live threads in the "
                      "LAUNCHER, one allocating continuously and one with a "
                      "pthread_atfork handler registered. The control run is "
                      "the same plan with no threads, and the child window must "
                      "come out identical"))
    add(CasePlan("M3", "helper_report", "pidfd_acquired_atomically",
                 (CH_RECEIPT, CH_TRACE), helper_args=("--exit", "0"),
                 note="owner amendment M3-T: traced, so the acquisition is "
                      "read from the external syscall record. It carries no "
                      "injection mode and no parent control arm -- in "
                      "particular not --rejected-acquisition-arm, whose "
                      "pidfd_open would put a second acquisition route in the "
                      "very record this case reads"))
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

# Every CasePlan slot and the function that reads it. A slot nothing reads is an
# inert field, and that is how work_dir_kind hid S2/S7's missing construction;
# a test fails on any slot absent here or not actually read by its consumer.
# ``note`` is the only documentation-only field.
CASEPLAN_FIELD_CONSUMERS = {
    "case": "evaluate", "binary": "_setup_none", "setup": "prepare",
    "parent": "pose", "spike_flags": "spike_argv", "helper_args": "spike_argv",
    "argv0": "spike_argv", "rule": "_evaluate_repetition",
    "streams": "_launch_and_observe", "channels": "missing_channels",
    "posed_when": "_evaluate_repetition", "excused_fds": "pose", "repeat": "pose",
    "timeout_ms": "spike_argv", "grace_ms": "spike_argv",
    "spawn_confirm_ms": "spike_argv", "body_length_changed": "pose",
    "pre_exec_stall": "pose", "traced": "_launch_and_observe",
    "baseline_flags": "pose", "expected_marker": "pose",
    "assertions": "_evaluate_repetition", "note": None,
}


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


# ============================================= N-5: durable posing evidence
# The proof that a case was posed as preregistered used to live only in the
# in-memory observation: build-identity binding, every forced-state landing
# fact and every cleanup problem died with the runner, and the durable record
# could not show that any E2-X1 manipulation happened. It now travels in the
# record, normalised: only these keys, only these values, never the raw
# observation.
BINDING_PUBLIC_KEYS = ("classification", "base_artefact", "object", "bound",
                       "detail", "base_sha256", "object_sha256")

FACT_PUBLIC_KEYS = frozenset({
    "action", "landed", "detail", "trial",
    # E-series and X1 post-pin actions
    "pinned_body_sha256", "replacement_sha256", "path_body_sha256_after",
    "path_exists", "renamed_away_exists", "target_before", "target_after",
    "new_target_sha256", "marker_written", "readback_matches",
    "bytes_written", "length_preserved", "body_sha256_before",
    "body_sha256_after", "size_before", "size_after", "descriptor_closed",
    "mapping_retained", "mode_before", "mode_after",
    # E5, S2 and S7: relations to the launcher, never raw identifiers
    "access_mode", "launcher_holds_same_inode", "launcher_access_modes",
    "capability_open_in_launcher",
    # parent-state proofs
    "names_present", "all_declared_names_present", "present_in_launcher",
    "cloexec_in_launcher", "cloexec_required", "blocked_required",
    "ignored_required", "signals_missing", "masks_readable", "closed_below",
    "exec_fd_at_0", "work_dir_capability_at_1", "fd2_free",
})


def _public_fact(fact, trial=None):
    out = {key: value for key, value in fact.items() if key in FACT_PUBLIC_KEYS}
    if trial is not None:
        out["trial"] = trial
    return out


# Plain observation values posing_evidence may carry under ``measured`` when a
# case's posed check or assertions read them. Booleans and integers only.
MEASURED_PUBLIC_KEYS = ("launcher_cpu_ms", "poll_returns",
                        "exec_status_pair_adjacent", "declared_launcher_threads",
                        "descendant_alive_after_launch", "elapsed_ms",
                        "fixture_descendant_signalled")

# The fork_helper fixtures: the P-series rendezvous read after launch()
# returned, and O6's and O7's signal armed before the spawn (AB7-B1).
FORK_HELPER_SETUPS = ("fork_helper", "fork_helper_prearmed")


def posing_evidence(plan, obs, evaluated=(), decided_by=None):
    """The normalised facts posing validity rests on, for the durable record.

    M1: written for EVERY case_completed record, BLOCKED and pre-setup records
    included. ``invocation`` says whether case_pose_started was written -- only
    pose()'s observation carries ``repeat_observations``, and the runner
    journals the boundary immediately before calling pose() -- and whether the
    mechanism was invoked. Nothing is invented for a case never posed.
    """
    trials = obs.get("repeat_observations") or [obs]
    invocation = {
        "pose_started": "repeat_observations" in obs,
        "mechanism_invoked": any(isinstance(t, dict)
                                 and t.get("launch_returned") is not None
                                 for t in trials),
    }
    if obs.get("blocked"):
        invocation.update(pose_started=False, mechanism_invoked=False,
                          blocked_cause=obs["blocked"])
    out = {"invocation": invocation}
    binding = obs.get("build_identity_binding")
    if isinstance(binding, dict):
        out["build_identity_binding"] = {key: binding[key]
                                         for key in BINDING_PUBLIC_KEYS
                                         if key in binding}
    if plan.setup in FORK_HELPER_SETUPS:
        # O7's owner decision: the fixture's construction is part of the posing
        # proof, so the declared helper and its declared arguments travel with
        # the record, beside the binding that names the helper_fork bytes and
        # the fixture fact the posed check read. The FIFO's private path is a
        # setup-time extra argument and never enters it. AB7-B1: for the
        # pre-armed signal, the normalised arming facts travel too.
        out["fixture"] = {"binary": plan.binary,
                          "helper_args": list(plan.helper_args)}
        arming = obs.get("fixture_signal_arming")
        if isinstance(arming, dict):
            out["fixture"]["signal_arming"] = {
                key: arming[key] for key in (
                    "armed_before_launch", "reader_inheritable",
                    "reader_passed_to_launcher", "fresh_fifo")
                if key in arming}
    forced =[_public_fact(fact, index if len(trials) > 1 else None)
              for index, trial in enumerate(trials) if isinstance(trial, dict)
              for fact in trial.get("post_pin_evidence") or ()
              if isinstance(fact, dict)]
    if forced:
        out["forced_state"] = forced
    reads = set(POSED_CHECK_READS.get(plan.posed_when, ()))
    for name in plan.assertions:
        reads |= set(observations.ASSERTION_READS.get(name, ()))
    measured = {}
    for key in MEASURED_PUBLIC_KEYS:
        if key in reads and obs.get(key) is not None:
            measured[key] = obs[key]
    if "report_state" in reads:
        states = {}
        for trial in trials:
            state = str(trial.get("report_state"))
            states[state] = states.get(state, 0) + 1
        measured["report_states"] = states
    if "report_sentinel_seen" in reads and "repeat_observations" in obs:
        # AB7-M1: the normalised sentinel fact, counted over the repetitions
        # the case actually ran. Never the captured bytes, and never on a
        # BLOCKED or pre-setup record, which ran none.
        sentinel = {}
        for trial in trials:
            value = str(trial.get("report_sentinel_seen"))
            sentinel[value] = sentinel.get(value, 0) + 1
        measured["report_sentinel_seen"] = sentinel
    if "baseline_observation" in reads:
        def threads(o):
            spike = o.get("spike") if isinstance(o, dict) else None
            shape = spike.get("parent_shape") if isinstance(spike, dict) else None
            return shape.get("extra_threads") if isinstance(shape, dict) else None
        measured["extra_threads"] = {
            "threaded_arm": threads(obs),
            "control_arm": threads(obs.get("baseline_observation"))}
    if measured:
        out["measured"] = measured
    if plan.posed_when is not None and evaluated:
        held = [entry["held"] for entry in evaluated]
        out["posed_check"] = {
            "name": plan.posed_when,
            "held": (False if False in held
                     else True if all(h is True for h in held) else None)}
    if plan.baseline_flags is not None:
        control = obs.get("baseline_observation")
        out["control_arm"] = {
            "run": isinstance(control, dict),
            "posed": isinstance(control, dict) and not control.get("not_posed"),
            "launch_returned": (control.get("launch_returned")
                                if isinstance(control, dict) else None)}
    if len(evaluated) > 1:
        # B1: no repetition may disappear. Each one's own status and outcome
        # token is durable, and so is the reduction that decided the case.
        out["repetitions"] = [{"trial": entry["trial"], "status": entry["status"],
                               "posed": "not_posed" not in entry["record"],
                               "outcome": entry["record"].get("outcome")}
                              for entry in evaluated]
        counts = {status: 0 for status in STATUS_PRECEDENCE}
        for entry in evaluated:
            counts[entry["status"]] += 1
        out["reduction"] = dict(counts, decided_by_trial=decided_by)
    out["cleanup_problems"] = list(obs.get("cleanup_problems") or ())
    return out


def _is_frozen_expectation(spec, token):
    return token == spec["predict"] or token in (spec["safe"] or ())


# ============================================ F417: posed versus result, centrally
# The F417 review found one defect four ways. A check that read what the
# mechanism PRODUCED -- a short drain, an image that ran, a close_range EINVAL,
# a control arm that hung -- was treated as a check on whether the case was
# POSED, so the preregistered FAIL became INVALID. And the repeated-trial
# reduction replaced trial 0's token with the first LATER token that differed,
# so a mismatch in trial 0 PASSed behind a correct trial 1.
#
# Definition section 9.6 fixes one rule for every launcher invocation and one
# reduction for every case:
#
#   INVALID  the result cannot honestly be evaluated: a forced state or posing
#            precondition was not established, an evidence channel or a required
#            measurement is missing, or the observation is not interpretable;
#   FAIL     the mechanism was invoked under the frozen state and a decisive
#            observation contradicts the frozen prediction, safe set, gate or
#            executed-identity assertion -- or a launch did not return in bound;
#   PASS     otherwise.
#
#   case     any repetition FAIL -> FAIL; else any INVALID -> INVALID; else PASS.
STATUS_PRECEDENCE = (checker.FAIL, checker.INVALID, checker.PASS)

# Everything one launcher invocation produces: _launch_and_observe's return,
# its early not-posed returns, and what _run_once adds. A repetition is judged
# on the case-level observation with exactly these replaced by its OWN values,
# so no repetition can borrow another's evidence.
TRIAL_OBSERVATION_KEYS = frozenset({
    "spike", "report", "report_state", "payload_len", "payload_is_recipe",
    "exec_confirmation", "trace", "acquisition", "acquisition_normalised",
    "trace_sha256", "observed_stage_sequence", "descendant_alive_after_launch",
    "launch_returned", "elapsed_ms", "spike_exit", "spike_stderr",
    "post_pin_evidence", "launcher_cpu_ms", "poll_returns",
    "exec_status_pair_adjacent", "not_posed", "cleanup_problems",
    # AB7: the pre-armed fixture signal, its arming facts and the report
    # sentinel are facts of ONE launcher invocation too.
    "fixture_descendant_signalled", "fixture_signal_arming",
    "report_sentinel_seen",
})


def reduce_repetitions(statuses):
    """The case status from its repetitions: any FAIL, else any INVALID, else PASS."""
    statuses = list(statuses)
    if not statuses:
        return checker.INVALID
    for status in STATUS_PRECEDENCE:
        if status in statuses:
            return status
    raise ValueError("a repetition was scored outside PASS, FAIL and INVALID: "
                     + repr(sorted(set(statuses))))


def _repetition_view(obs, trial):
    view = {key: value for key, value in obs.items()
            if key not in TRIAL_OBSERVATION_KEYS and key != "repeat_observations"}
    view.update(trial)
    return view


def _control_arm_verdict(plan, view):
    """I2: ``None`` when M2's control arm returned in bound, else the verdict.

    ``("not_posed", why)`` when it was never run, not posed, or its return was
    not recorded; ``("non_return", why)`` or ``("late", why, elapsed)`` when the
    production launch it runs did not return in bound -- a FAIL like any other.
    """
    control = view.get("baseline_observation")
    if not isinstance(control, dict):
        return ("not_posed", "the control arm was never run")
    if control.get("not_posed"):
        return ("not_posed", "the control arm was not posed: "
                + str(control["not_posed"]))
    returned = control.get("launch_returned")
    if returned is False:
        return ("non_return", "the control arm's launch() did not return "
                              "within its declared bound")
    if returned is not True:
        return ("not_posed", "the control arm's return was not recorded")
    elapsed, bound = control.get("elapsed_ms"), plan.total_bound_ms()
    if isinstance(elapsed, int) and elapsed > bound:
        return ("late", "the control arm's launch() returned after %d ms, bound "
                        "%d ms" % (elapsed, bound), elapsed)
    return None


def _evaluate_repetition(plan, spec, view, where=""):
    """One launcher invocation's record, for the checker to score on its own.

    Returns ``(record, held)``, where ``held`` is the posed check's result, or
    None when the repetition never reached it.
    """
    record = {
        "case": plan.case,
        "class": spec["cls"],
        "traced": bool(view.get("traced")),
        "launch_returned": view.get("launch_returned"),
        "elapsed_ms": view.get("elapsed_ms"),
        "total_bound_ms": plan.total_bound_ms(),
    }
    if view.get("not_posed"):
        record["not_posed"] = str(view["not_posed"]) + where
        return record, None
    if spec["traced"]:
        # checker.score_case scores a traced case INVALID when its record shows
        # no syscall record, so the trace has to reach the record rather than
        # stopping at the observation -- on EVERY posed path, including a
        # non-return and M2's control arm, or a decisive FAIL would read as
        # INVALID. Only the PARSED window travels: the raw tracer text stays
        # local and is represented by its digest, which keeps a host-specific
        # record out of published evidence while still re-verifiable.
        record["trace"] = view.get("trace")
        record["trace_sha256"] = view.get("trace_sha256")
        if view.get("acquisition_normalised") is not None:
            # M3-T. Normalised only: DIRECT_CHILD and DIRECT_CHILD_PIDFD in
            # place of the raw pid and descriptor number, which are
            # experiment-local identifiers and never part of a receipt.
            record["acquisition_normalised"] = view["acquisition_normalised"]
    if view.get("launch_returned") is False:
        # A launcher-side non-return is never a recordable outcome. Recorded
        # from this driver's own watchdog, so a hang is a FAIL and never an
        # absent record understating a known result as an open question.
        record["outcome"] = None
        record["reason"] = "launch() did not return within the declared bound" + where
        return record, None
    if plan.baseline_flags is not None:
        verdict = _control_arm_verdict(plan, view)
        if verdict is not None:
            if verdict[0] == "not_posed":
                record["not_posed"] = verdict[1] + where
                return record, None
            record["outcome"] = None
            record["reason"] = verdict[1] + where
            if verdict[0] == "non_return":
                record["launch_returned"] = False
            else:
                record["elapsed_ms"] = verdict[2]
            return record, None
    held = None
    if plan.posed_when is not None:
        held = bool(POSED_CHECKS[plan.posed_when](view))
        if not held:
            record["not_posed"] = ("the forced state did not materialise: "
                                   + plan.posed_when + where)
            return record, held

    token, reason = observations.derive(plan.rule, view)
    if token is None:
        # AB7-M1. No token is INVALID -- unless an assertion that proves a
        # FORBIDDEN event on its own is violated. S2/S7's report sentinel
        # beside a report that does not parse proves an image started, whichever
        # body it was: a decisive FAIL, never "not interpretable".
        decisive = [name for name in plan.assertions
                    if name in observations.DECISIVE_WITHOUT_TOKEN_ASSERTIONS]
        results = observations.apply_assertions(decisive, view)
        violated = [name for name in decisive
                    if results[name]["result"] == observations.ASSERTION_VIOLATED]
        if not violated:
            record["not_posed"] = ("observation not interpretable: " + reason
                                   + where)
            return record, held
        record["assertions"] = results
        token = observations.ASSERTION_VIOLATION_TOKENS[violated[0]]
        reason = results[violated[0]]["detail"]
    elif plan.assertions:
        results = observations.apply_assertions(plan.assertions, view)
        record["assertions"] = results
        violated = [name for name in plan.assertions
                    if results[name]["result"] == observations.ASSERTION_VIOLATED]
        unobservable = [name for name in plan.assertions
                        if results[name]["result"]
                        == observations.ASSERTION_UNOBSERVABLE]
        if violated:
            # Posed, and it showed something other than what it claims: FAIL.
            record["mechanism_outcome"] = token
            token = observations.ASSERTION_VIOLATION_TOKENS[violated[0]]
            reason = results[violated[0]]["detail"]
        elif unobservable and _is_frozen_expectation(spec, token):
            # It would PASS on a token alone, and the evidence that it produced
            # what it claims is missing. Missing evidence is never success.
            record["not_posed"] = (
                "an executed-identity assertion could not be observed: " +
                "; ".join(name + ": " + results[name]["detail"]
                          for name in unobservable) + where)
            return record, held
    record["outcome"] = token
    record["reason"] = reason + where

    if spec["traced"]:
        # checker.score_case scores a traced case INVALID when its record shows
        # no syscall record, so the trace has to reach the record rather than
        # stopping at the observation. Only the PARSED window travels: the raw
        # tracer text stays local and is represented by its digest, which is
        # what keeps a host-specific record out of published evidence while
        # still making it re-verifiable.
        record["trace"] = view.get("trace")
        record["trace_sha256"] = view.get("trace_sha256")
        if view.get("acquisition_normalised") is not None:
            # M3-T. Normalised only: DIRECT_CHILD and DIRECT_CHILD_PIDFD in
            # place of the raw pid and descriptor number, which are
            # experiment-local identifiers and never part of a receipt.
            record["acquisition_normalised"] = view["acquisition_normalised"]

    if spec["gates"]:
        record["gates"] = gates_for(plan, view, token)
    if plan.case in DOCUMENTATION_GATES:
        record["documentation_gate"] = _documentation_gate(view.get("spike"))
    offending = _asserted_unobserved_fact(view.get("spike"))
    if offending:
        record["asserted_unobserved_fact"] = offending
    return record, held


def evaluate(plan, obs):
    """Turn one case's observation into the record ``checker.py`` scores.

    The record is data. It never names its own status, and every field the
    checker needs is present or explicitly absent -- ``launch_returned`` in
    particular, whose omission is the P-8 hole and is INVALID by contract.

    Every launcher invocation of the case -- each of O8's and S7's 200
    repetitions, the single run of every other case -- is judged on its own by
    ``_evaluate_repetition`` and scored by the frozen checker. The case record
    is the record of the repetition that decides the reduction: the first FAIL,
    else the first INVALID, else the first PASS (definition section 9.6).
    """
    spec = BY_NAME[plan.case]
    if obs.get("blocked"):
        # A conditional case blocked on its own frozen cause is decided before
        # any repetition, and the checker enforces that only a conditional case
        # may absorb one.
        record = {
            "case": plan.case,
            "class": spec["cls"],
            "traced": bool(obs.get("traced")),
            "launch_returned": obs.get("launch_returned"),
            "elapsed_ms": obs.get("elapsed_ms"),
            "total_bound_ms": plan.total_bound_ms(),
            "blocked": obs["blocked"],
        }
        record["posing_evidence"] = posing_evidence(plan, obs)
        return record

    trials = obs.get("repeat_observations") or [obs]
    evaluated = []
    for index, trial in enumerate(trials):
        where = "" if len(trials) == 1 else " (repeated trial %d)" % index
        record, held = _evaluate_repetition(
            plan, spec, _repetition_view(obs, trial), where)
        evaluated.append({"trial": index,
                          "status": checker.score_case(plan.case, record)[0],
                          "record": record, "held": held})
    decided = reduce_repetitions(entry["status"] for entry in evaluated)
    chosen = next(entry for entry in evaluated if entry["status"] == decided)
    record = dict(chosen["record"])
    record["posing_evidence"] = posing_evidence(plan, obs, evaluated,
                                                chosen["trial"])
    return record


# =================================================================== context
class TrialContext:
    """Everything a case needs that is not in its plan. Built once per trial."""

    def __init__(self, build, work, preflight, freeze, sanitiser,
                 supplied_channels=None, build_identity=None):
        self.build = pathlib.Path(build)
        self.work = pathlib.Path(work)
        self.preflight = preflight
        self.freeze = freeze
        self.sanitiser = sanitiser
        # The exact artefacts hashed before the first case. Every setup that
        # opens a build product checks against this, so the trial can prove
        # which bytes it ran even if it later aborts.
        self.build_identity = dict(build_identity or {})
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
               extra_helper_args=(), parent_flags=()):
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
    # TEST/CONTROL-ONLY caller-state flags, supplied by the applied parent
    # state at spawn time and never by a plan's own spike_flags.
    argv.extend(parent_flags)
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

    Availability is re-probed at preflight and never assumed. Only the eight
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
    prepared = prepare(plan, ctx, auth)
    if not prepared["ready"]:
        return prepared["observation"]
    return pose(plan, ctx, auth, prepared)


def prepare(plan, ctx, auth):
    """Everything a case needs BEFORE the mechanism is invoked. Poses nothing.

    Split out of :func:`observe` so the runner has somewhere to put the
    ``case_pose_started`` record: that event is the Trial #2 immutability
    boundary, and it must be durable before the launcher exists, not after.

    ``ready`` false means the case has a terminal observation already -- a
    channel the mechanism cannot supply, a frozen environment block, a fixture
    that could not be built, or a starting object that is not the one this trial
    hashed. None of those invoke the mechanism, so none of them crosses the
    boundary.
    """
    _require_authorisation(auth)

    missing = plan.missing_channels(ctx.supplied_channels)
    if missing:
        return {"ready": False, "observation": {
            "not_posed": "the mechanism supplies no channel for " +
                         ", ".join(missing) + ": " +
                         "; ".join(CHANNEL_UNAVAILABLE_REASON.get(c, c)
                                   for c in missing),
            "launch_returned": None}}

    cause = blocked_cause_for(plan, ctx)
    if cause is not None:
        return {"ready": False,
                "observation": {"blocked": cause, "launch_returned": None}}

    built = SETUPS[plan.setup](ctx, plan)
    if built.get("not_posed"):
        return {"ready": False, "observation": {
            "not_posed": built["not_posed"], "launch_returned": None}}

    # T2-R3. The starting object is tied to the identity hashed before the
    # first case, and its relationship to that identity is classified.
    binding = bind_build_identity(ctx, plan, built)
    if not binding.get("bound"):
        return {"ready": False, "observation": {
            "not_posed": "build identity: " + str(binding.get("detail")),
            "build_identity_binding": binding, "launch_returned": None}}

    return {"ready": True, "built": built, "binding": binding}


def pose(plan, ctx, auth, prepared):
    """Invoke the mechanism for one prepared case. THIS crosses the boundary."""
    _require_authorisation(auth)
    built, binding = prepared["built"], prepared["binding"]

    # The DECLARATION. It is turned into real state -- once per launcher, and
    # released after it -- by _run_once, and proven at the barrier.
    parent_state = PARENT_STATES[plan.parent](ctx, plan)
    try:
        trials = [_run_once(plan, ctx, built, parent_state)
                  for _ in range(plan.repeat)]
    finally:
        # Deterministic release of everything the setup held. Its problems are
        # recorded beside the case, never folded into its mechanism result.
        cleanup_problems = release_setup_resources(built)

    obs = dict(trials[0])
    obs["repeat_observations"] = trials
    obs["build_identity_binding"] = binding
    obs["cleanup_problems"] = (
        [p for trial in trials for p in trial.get("cleanup_problems") or ()]
        + cleanup_problems)
    obs["declared_launcher_threads"] = parent_state.get(PS_THREADS)
    obs["expected_marker"] = plan.expected_marker

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
    return obs


def gates_for(plan, obs, token=None):
    """Gated sub-assertions for the RECORDED cases, computed from observations.

    A gate is never derived from what the launcher intended, only from what the
    record shows. A gate whose evidence is missing stays False, because a gate
    that cannot be shown to hold has not been shown to hold.

    ``token`` is the case's derived outcome. R4's never_reports_exited_zero used
    to read an ``outcome_token`` key nothing produced, so ``None != "Exited:0"``
    made the gate hold on every run; it is now computed from the real token, in
    evaluate(), and an absent token does not hold it.
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
            out[gate] = token is not None and token != "Exited:0"
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


def _children_cpu_ms():
    """CPU of every reaped child of this process so far, in ms, or None.

    O5's launcher CPU is the difference across one launch, measured OUTSIDE the
    launcher. The harness reaps nothing else while a case runs, so the
    difference is the launcher's own CPU plus that of the helper it reaped: it
    can overstate the launcher's share and never understate it.
    """
    try:
        import resource
    except ImportError:
        return None
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    return int(round((usage.ru_utime + usage.ru_stime) * 1000))


def _run_once(plan, ctx, built, parent_state, flags=None):
    """One launcher invocation, with this driver's own watchdog.

    The watchdog is the P-8 correction in code: ``launch_returned`` is recorded
    from here, always, so a true hang is a FAIL rather than a missing record.

    N-1: the declared parent state is CREATED here, for this one launcher, and
    released afterwards together with any per-trial forced state; problems in
    either are recorded beside the trial and never folded into its result.
    """
    try:
        applied = AppliedParentState(plan.parent, parent_state)
    except OSError as exc:                                  # noqa: BLE001
        return {"not_posed": _problem("the parent state could not be created",
                                      exc),
                "launch_returned": None,
                "cleanup_problems": restore_after_trial(built)}
    try:
        result = _launch_and_observe(plan, ctx, built, applied, flags)
    finally:
        # AB7-B1: the pre-armed fixture signal's read end and FIFO never
        # outlive the invocation that armed them, whatever it returned.
        problems = (applied.release() + restore_after_trial(built)
                    + disarm_fixture_signal(built))
    result["cleanup_problems"] = problems
    return result


def _launch_and_observe(plan, ctx, built, applied, flags):
    argv = spike_argv(plan, built["exec_path"], built.get("work_dir") or ctx.work,
                      ctx.build, flags=flags,
                      extra_helper_args=built.get("extra_helper_args", ()),
                      parent_flags=applied.spike_flags)
    trace_path = None
    if plan.traced:
        trace_path = ctx.build / (plan.case + ".strace")
        tracer = tracer_argv(ctx.preflight, trace_path)
        if tracer is None:
            return {"not_posed": "no tracer is available for a traced case",
                    "launch_returned": None}
        argv = tracer + argv

    # AB7-B1. O6's and O7's fixture signal is armed BEFORE the launcher exists.
    # The read end is open in this process only -- never in pass_fds, never
    # inheritable -- so helper_fork's descendant can signal before the
    # launcher's normal group sweep ends it. _run_once disarms it afterwards.
    arming, unarmed = _arm_fixture_signal(built, applied.pass_fds)
    if unarmed is not None:
        return {"not_posed": unarmed, "launch_returned": None}

    bound_s = (plan.total_bound_ms() + 5000) / 1000.0

    cpu_before = _children_cpu_ms()
    started = time.monotonic()
    post_pin_facts = None
    if needs_barrier(built, applied):
        outcome = _run_with_post_pin(argv, applied, bound_s, ctx, plan, built)
        post_pin_facts = outcome["post_pin"]
        if outcome.get("not_posed"):
            return {"not_posed": outcome["not_posed"],
                    "post_pin_evidence": post_pin_facts,
                    "launch_returned": None}
        returned, stdout, stderr, rc = (outcome["returned"], outcome["stdout"],
                                        outcome["stderr"], outcome["rc"])
        # The handshake and the harness's own action are not launch() time: the
        # launcher's clock starts after CONTINUE, and so does this one.
        started = outcome.get("continued_at") or started
    else:
        try:
            with applied.spawning():
                proc = subprocess.Popen(argv, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE, env=applied.env,
                                        pass_fds=applied.pass_fds,
                                        restore_signals=applied.restore_signals)
        except OSError as exc:                              # noqa: BLE001
            return {"not_posed": _problem("the launcher could not be started",
                                          exc),
                    "launch_returned": None}
        try:
            stdout, stderr = proc.communicate(timeout=bound_s)
            returned, rc = True, proc.returncode
        except subprocess.TimeoutExpired:
            proc.kill()
            stdout, stderr = proc.communicate()
            returned, rc = False, None
    elapsed_ms = int((time.monotonic() - started) * 1000)
    cpu_after = _children_cpu_ms()
    launcher_cpu_ms = (cpu_after - cpu_before
                       if cpu_before is not None and cpu_after is not None
                       else None)

    spike = observations.parse_spike_stdout(stdout)
    # O5. The launcher's own count of drain poll() returns, emitted outside the
    # receipt beside its elapsed time and dropped by evidence.receipt_view.
    poll_returns = (spike.get("poll_returns_not_in_receipt")
                    if isinstance(spike, dict) else None)

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

    trace, acquisition, trace_digest, pair_adjacent = None, None, None, None
    if trace_path is not None and trace_path.exists():
        raw_trace = trace_path.read_bytes()
        trace = observations.parse_strace_child_window(raw_trace)
        if applied.close_low >= 1:
            # F7 (F417-B4). The exec descriptor opened as 0 -- the barrier
            # proved it from /proc -- and the pair is followed through the
            # LAUNCHER's own pre-clone syscalls. The child's close_range calls
            # are never read. Descriptor numbers stay here; only the boolean
            # travels.
            pair_adjacent = observations.parent_pair_is_adjacent(
                observations.parse_parent_descriptor_pair(raw_trace, 0))
        # M3-T. Parsed from the same record the other traced cases use; there is
        # no second tracing framework. The raw text stays local and only its
        # digest is publishable, so a host-specific record cannot become
        # evidence by being useful.
        acquisition = observations.parse_pidfd_acquisition(raw_trace)
        trace_digest = oracles.digest_of(raw_trace)

    # Read ONCE, after launch() returned: the P-series rendezvous consumes its
    # byte. It is the P1/P2/P4 survival fact and nothing more (AB7-B1): a
    # descendant in the direct child's process group never survives the sweep.
    alive = _descendant_alive(built, plan)
    # O6's and O7's pre-armed signal, written by helper_fork's descendant before
    # the sweep and buffered in the FIFO this process kept open since before
    # the spawn. It is their fixture fact and their exec evidence.
    signalled = _read_fixture_signal(built)
    # AB7-M1: whether the report sentinel is positively in descriptor 1's
    # retained prefix. Only the boolean travels; the captured bytes never do.
    sentinel_seen = (observations.report_sentinel_seen(spike.get("stdout"))
                     if isinstance(spike, dict)
                     and spike.get("admission") == "accepted" else None)

    return {
        "spike": spike,
        "report": report,
        "report_state": report_state,
        "payload_len": len(payload),
        "payload_is_recipe": payload_is_recipe,
        "exec_confirmation": observations.exec_confirmation(
            spike, report, payload_is_recipe, report_state,
            fixture_signalled=signalled),
        "trace": trace,
        "acquisition": acquisition,
        "acquisition_normalised": observations.normalise_acquisition(acquisition),
        "trace_sha256": trace_digest,
        "observed_stage_sequence": (trace or {}).get("stage_sequence"),
        "descendant_alive_after_launch": alive,
        "fixture_descendant_signalled": signalled,
        "fixture_signal_arming": arming,
        "report_sentinel_seen": sentinel_seen,
        "launch_returned": returned,
        "elapsed_ms": elapsed_ms,
        "spike_exit": rc,
        "spike_stderr": (stderr or b"").decode("utf-8", "replace")[-2000:],
        "post_pin_evidence": post_pin_facts,
        "launcher_cpu_ms": launcher_cpu_ms,
        "poll_returns": (poll_returns if observations._plain_int(poll_returns)
                         else None),
        "exec_status_pair_adjacent": pair_adjacent,
    }




def _run_with_post_pin(argv, applied, bound_s, ctx, plan, built):
    """Launch, wait for READY, prove and land the forced state, then CONTINUE.

    Every wait is bounded. A control failure is never a mechanism result: if
    READY does not arrive, the launcher cannot be identified, the declared
    parent state is not observed in it, or a harness action cannot be proven to
    have landed, the control socket is closed WITHOUT sending CONTINUE, the
    launcher dies at its own barrier, and the case is reported as not posed.
    """
    if applied is None:
        applied = AppliedParentState("none", {})
    parent_sock, child_sock = socket.socketpair()
    control = child_sock.fileno()
    argv = list(argv) + ["--post-pin-control-fd", str(control)]
    facts, note, continued_at = [], None, None
    try:
        with applied.spawning():
            proc = subprocess.Popen(argv, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, env=applied.env,
                                    pass_fds=(control,) + tuple(applied.pass_fds),
                                    restore_signals=applied.restore_signals)
    except OSError as exc:                                  # noqa: BLE001
        parent_sock.close()
        child_sock.close()
        return {"not_posed": _problem("the launcher could not be started", exc),
                "post_pin": facts}
    child_sock.close()          # the parent drops its copy of the child's end

    try:
        parent_sock.settimeout(POST_PIN_READY_TIMEOUT_S)
        try:
            ready = parent_sock.recv(1)
        except (socket.timeout, OSError) as exc:            # noqa: BLE001
            ready, note = b"", _problem("READY never arrived", exc)
        if ready != POST_PIN_READY:
            note = note or ("the control channel produced %r instead of READY"
                            % (ready,))
        else:
            pid = _launcher_pid(proc, plan)
            if pid is None:
                note = "the launcher process could not be identified at the barrier"
            else:
                # The parent state first: it must already be present in the
                # launcher. Then the setup's own post-pin change, which may need
                # to observe the launcher too (E5, S2, S7).
                # The capability the launcher was actually given: a case-private
                # directory for S2/S7, the trial's work directory otherwise.
                facts = applied.prove(pid, built.get("exec_path"),
                                      built.get("work_dir") or ctx.work)
                built["_launcher_pid"] = pid
                try:
                    _, action_facts = perform_post_pin(ctx, plan, built)
                finally:
                    built.pop("_launcher_pid", None)
                facts = facts + action_facts
                if facts and all(fact["landed"] for fact in facts):
                    try:
                        parent_sock.sendall(POST_PIN_CONTINUE)
                        continued_at = time.monotonic()
                    except OSError as exc:                  # noqa: BLE001
                        note = _problem("CONTINUE could not be delivered", exc)
                else:
                    note = "the preregistered forced state did not land"
    finally:
        parent_sock.close()     # closing without CONTINUE ends the launcher

    try:
        stdout, stderr = proc.communicate(timeout=bound_s)
        returned, rc = True, proc.returncode
    except subprocess.TimeoutExpired:
        proc.kill()
        stdout, stderr = proc.communicate()
        returned, rc = False, None

    if note is not None:
        return {"not_posed": "post-pin control: " + note, "post_pin": facts}
    return {"post_pin": facts, "returned": returned, "stdout": stdout,
            "stderr": stderr, "rc": rc, "continued_at": continued_at}


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


# ============================================ the fixed symbolic registry (M-1)
# ``evidence.py`` needs the names this module registers, and cannot import it at
# module scope because this module imports ``evidence``. The lazy import it used
# instead could observe a HALF-BUILT driver: Python publishes a module object in
# ``sys.modules`` before executing its body, so ``import driver`` succeeds long
# before the decorators below have populated SETUPS, PARENT_STATES and
# POSED_CHECKS. Reading those globals directly cannot tell "empty because this
# freeze registers nothing" apart from "empty because the body has not run yet",
# and the empty answer was then cached permanently.
#
# This accessor is therefore defined LAST, after every registry is complete, so
# its mere EXISTENCE is a positive readiness signal. A partially initialised
# driver does not have the attribute, and evidence.py fails closed rather than
# guessing from globals that may or may not be filled in.
#
# It is not a second source of truth. It returns the registries themselves.

def registry_vocabulary():
    """A snapshot of every fixed symbolic name this module registers.

    Pure: it reads the registries and nothing else. It builds nothing, poses
    nothing and executes nothing, so a sanitiser may call it at any time. The
    values are tuples rather than the live dicts, so a consumer cannot mutate
    the driver's registries through what it is handed.
    """
    return {
        "SETUPS": tuple(sorted(SETUPS)),
        "PARENT_STATES": tuple(sorted(PARENT_STATES)),
        "POSED_CHECKS": tuple(sorted(POSED_CHECKS)),
        "ALL_CHANNELS": tuple(ALL_CHANNELS),
    }
