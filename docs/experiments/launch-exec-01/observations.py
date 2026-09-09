"""LAUNCH-EXEC-01 observation parsing and frozen outcome-token derivation (P-12).

This module is the function from OBSERVATION to the frozen outcome vocabulary.
The independent pre-trial review recorded its absence as **P-12**: the manifest
freezes expectations as tokens, the spike emits component fields, and nothing
mapped one to the other. A mapping written after results could be known is where
goalpost movement happens, so it is written and frozen BEFORE the first trial.

Three rules govern everything here:

1. **Nothing in this module executes anything.** It parses bytes and returns
   tokens. It imports no subprocess machinery, spawns nothing, and reads no file.
   Importing it poses no case.

2. **Missing information can never become success.** Every derivation returns
   ``None`` when the evidence it needs is absent or unparseable, and ``None``
   becomes INVALID at the checker, never PASS and never a silently plausible
   token. The rule that matters most: a clean exec-status EOF is *not* exec
   confirmation, because S5 produces one that is byte-identical to S1's.

3. **Tokens are built from closed vocabularies, never from free text.** The
   composite forms ``Exited:0``, ``Signaled:SIGSEGV``, ``ExecFailed:ETXTBSY``,
   ``refused:ElfNotInCohort`` and ``TimedOut:KilledByLauncher:SIGKILL`` are the
   frozen contract's own representation, reproduced here from a fixed
   disposition set, a fixed signal table, a fixed errno table and a fixed
   refusal set. An observation outside those vocabularies yields ``None``.

A well-formed token outside the case's frozen expectation is a **FAIL**, not an
INVALID: ``Exited:3`` where ``Exited:42`` was predicted is a real, posed result.
Only unparseable or missing evidence is INVALID. Conflating the two would
understate a known result as an open question, which is the defect P-8 closed on
the checker side.

NOT_RUN: no trial has been executed and no case has been posed.
"""
import json
import re

from frozen_cases import (
    CHILD_FORBIDDEN_SYSCALLS,
    CHILD_INJECTION_MODES,
    CHILD_PERMITTED_SYSCALLS,
    CHILD_TEST_INJECTION_SYSCALLS,
    STAGES,
)

# --------------------------------------------------------- closed vocabularies
# Exactly the process dispositions launcher_spike.c can print. Anything else in
# that field means the spike is not the frozen spike, which is INVALID rather
# than a mechanism result.
SPIKE_DISPOSITIONS = frozenset({
    "ExecFailed",
    "ExecStatusIndeterminate",
    "ExitStatusUnobservable",
    "TimedOut",
    "Exited",
    "Signaled",
})

# Exactly the timeout sub-dispositions the spike can print, "" included.
SPIKE_TIMEOUT_DISPOSITIONS = frozenset({
    "", "TerminationFailed", "ExitedDuringGrace", "KilledByLauncher",
})

# Exactly the admission refusals launcher_spike.c can emit. A refusal outside
# this set is not a HELM refusal.
SPIKE_REFUSALS = frozenset({
    "NotRegularFile",
    "SetIdBitsPresent",
    "ElfNotInCohort",
    "DescriptorModeUnsuitable",
})

# The frozen child-setup stage vocabulary, reused rather than restated so a
# stage rename cannot desynchronise the mapping from the manifest.
SPIKE_STAGES = frozenset(STAGES)

# Signal numbers to names. Closed on purpose: the mechanism can only deliver or
# observe these, and an unknown number must not be rendered as "Signaled:47".
SIGNAL_NAMES = {
    1: "SIGHUP", 2: "SIGINT", 3: "SIGQUIT", 4: "SIGILL", 5: "SIGTRAP",
    6: "SIGABRT", 7: "SIGBUS", 8: "SIGFPE", 9: "SIGKILL", 10: "SIGUSR1",
    11: "SIGSEGV", 12: "SIGUSR2", 13: "SIGPIPE", 14: "SIGALRM", 15: "SIGTERM",
    16: "SIGSTKFLT", 17: "SIGCHLD", 18: "SIGCONT", 19: "SIGSTOP", 20: "SIGTSTP",
    21: "SIGTTIN", 22: "SIGTTOU", 23: "SIGURG", 24: "SIGXCPU", 25: "SIGXFSZ",
    26: "SIGVTALRM", 27: "SIGPROF", 28: "SIGWINCH", 29: "SIGIO", 30: "SIGPWR",
    31: "SIGSYS",
}

# errno numbers to names, restricted to what execveat(2), fchdir(2) and the
# child-setup syscalls can return in this experiment. Linux x86-64 values.
ERRNO_NAMES = {
    1: "EPERM", 2: "ENOENT", 4: "EINTR", 5: "EIO", 8: "ENOEXEC", 9: "EBADF",
    12: "ENOMEM", 13: "EACCES", 14: "EFAULT", 20: "ENOTDIR", 21: "EISDIR",
    22: "EINVAL", 24: "EMFILE", 26: "ETXTBSY", 36: "ENAMETOOLONG",
    38: "ENOSYS", 40: "ELOOP",
}

# The exit-status range execveat and waitid can report. Outside it, the field is
# not an exit status.
EXIT_CODE_MIN, EXIT_CODE_MAX = 0, 255

# The frozen report sentinel, restated here for splitting fd-1 bytes. oracles.py
# owns the recipe; this module owns only the split.
REPORT_SENTINEL = b"HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n"

# /proc/<pid>/status renders signal masks as 16 lowercase hex digits.
_MASK_RE = re.compile(r"\A[0-9a-fA-F]{1,16}\Z")


# ------------------------------------------------------------------- failures
class Unparseable(Exception):
    """Raised only by the strict parsers; callers convert it to ``None``.

    It exists so a malformed observation is impossible to mistake for a valid
    one carrying default values -- the failure mode that would let a missing
    field read as a zero exit code.
    """


# ---------------------------------------------------------------- spike output
def parse_spike_stdout(raw):
    """Parse the ONE JSON object launcher_spike.c writes to its own stdout.

    Returns a dict, or ``None`` if the bytes are not a single well-formed
    receipt built from the closed vocabularies above. ``None`` is not a result:
    it means the mechanism was not observed, which the checker scores INVALID.
    """
    if raw is None:
        return None
    if isinstance(raw, bytes):
        try:
            raw = raw.decode("utf-8")
        except UnicodeDecodeError:
            return None
    text = raw.strip()
    if not text:
        return None
    # The spike prints exactly one object and nothing else. Extra lines mean
    # something else wrote to the channel, and the receipt is then not
    # attributable to the mechanism.
    lines = [ln for ln in text.splitlines() if ln.strip()]
    if len(lines) != 1:
        return None
    try:
        obj = json.loads(lines[0])
    except (ValueError, TypeError):
        return None
    if not isinstance(obj, dict):
        return None

    admission = obj.get("admission")
    if admission == "refused":
        refusal = obj.get("refusal")
        if refusal not in SPIKE_REFUSALS:
            return None
        if obj.get("exec_reached") is not False:
            return None
        return {"admission": "refused", "refusal": refusal,
                "exec_reached": False}
    if admission != "accepted":
        return None

    disposition = obj.get("process_disposition")
    if disposition not in SPIKE_DISPOSITIONS:
        return None
    timeout_disposition = obj.get("timeout_disposition")
    if timeout_disposition not in SPIKE_TIMEOUT_DISPOSITIONS:
        return None
    stage = obj.get("exec_failed_stage")
    if stage not in SPIKE_STAGES and stage != "":
        return None
    for key in ("pre_exec_body_sha256", "pre_exec_body_size",
                "pre_exec_mode_bits", "exec_failed_errno", "exit_code",
                "term_signal", "wait_errno", "stdout", "stderr"):
        if key not in obj:
            return None
    for stream in ("stdout", "stderr"):
        block = obj[stream]
        if not isinstance(block, dict):
            return None
        if block.get("completeness") not in (
                "CompleteAtEof", "WriterRetainedAfterChildExit"):
            return None
        if not isinstance(block.get("bytes_drained"), int):
            return None
        if not isinstance(block.get("drained_sha256"), str):
            return None
    return obj


def parse_helper_report(payload_and_report):
    """Parse the helper report that follows the frozen sentinel on descriptor 1.

    Returns ``(payload_bytes, report_dict_or_None)``. A report that is present
    but malformed yields ``None`` for the report, never a partial dict: the
    difference between "the image ran and told us X" and "we could not tell what
    ran" is exactly what S5 exists to expose.
    """
    if payload_and_report is None:
        return b"", None
    raw = payload_and_report
    if isinstance(raw, str):
        raw = raw.encode("utf-8", "surrogateescape")
    index = raw.find(REPORT_SENTINEL)
    if index < 0:
        return raw, None
    payload = raw[:index]
    tail = raw[index + len(REPORT_SENTINEL):]
    try:
        report = json.loads(tail.decode("utf-8").strip() or "null")
    except (ValueError, UnicodeDecodeError):
        return payload, None
    if not isinstance(report, dict):
        return payload, None
    if "marker" not in report:
        # Every frozen helper that emits a report emits a marker first. Its
        # absence means the bytes after the sentinel are not one of our reports.
        return payload, None
    return payload, report


# ----------------------------------------------------------- exec confirmation
# The four states section 11 of the trial contract requires be distinguishable.
EXEC_PRE_EXEC_ERROR = "pre_exec_error"        # the child wrote a status record
EXEC_DIED_BEFORE_EXEC = "died_before_exec"    # clean EOF, and NOTHING ran
EXEC_REACHED = "exec_reached"                 # positive evidence the image ran
EXEC_UNINTERPRETABLE = "uninterpretable"      # a report arrived and made no sense


def exec_confirmation(spike, report, payload_is_recipe=None):
    """Which of the four exec states the observation actually supports.

    **Clean EOF is deliberately not enough.** launcher_spike.c sets
    ``exec_confirmed`` when the status pipe closes with no record, and S5 makes
    a child die by SIGKILL immediately before ``execveat`` so that exactly this
    happens with nothing executed. The parent's view there is byte-identical to
    S1's. Positive evidence that the image ran is therefore required, and there
    are exactly two admissible kinds:

      * a parseable helper report behind the frozen sentinel, or
      * for the ``--no-report`` O-series, payload bytes that match the frozen
        recipe -- only the pinned helper can produce them.

    ``payload_is_recipe`` is that second kind, computed by the caller against
    ``oracles.py``. It is passed in rather than computed here so this module
    never needs the recipe, and so a case that declares no payload evidence
    cannot accidentally acquire some.
    """
    if spike is None:
        return None
    if spike.get("admission") == "refused":
        # Nothing was executed and nothing claims to have been.
        return EXEC_DIED_BEFORE_EXEC
    if spike.get("process_disposition") == "ExecFailed":
        return EXEC_PRE_EXEC_ERROR
    if report is not None:
        return EXEC_REACHED
    if payload_is_recipe is True:
        return EXEC_REACHED
    if payload_is_recipe is False:
        # The case declared payload evidence and the payload did not match. The
        # image that ran, if any, is not the pinned helper.
        return EXEC_UNINTERPRETABLE
    return EXEC_DIED_BEFORE_EXEC


# -------------------------------------------------------- token constructors
def _exited(code):
    if not isinstance(code, int) or not EXIT_CODE_MIN <= code <= EXIT_CODE_MAX:
        return None
    return "Exited:" + str(code)


def _signaled(number):
    name = SIGNAL_NAMES.get(number)
    if name is None:
        return None
    return "Signaled:" + name


def _errno_name(number):
    return ERRNO_NAMES.get(number)


def _exec_failed(stage, errno_number):
    """``ExecFailed:<ERRNO>``, or ``ExecFailed:CHDIR:<ERRNO>`` at the CHDIR stage.

    The stage qualifier is frozen for exactly one stage: S2 and S7 predict
    ``ExecFailed:CHDIR:EACCES`` because the working-directory capability, not the
    executable, is what was refused. Every other stage renders unqualified, which
    is what X1/X3/X4/X5/X8 predict.
    """
    name = _errno_name(errno_number)
    if name is None:
        return None
    if stage == "CHDIR":
        return "ExecFailed:CHDIR:" + name
    if stage in ("", "EXEC"):
        return "ExecFailed:" + name
    # A failure at RELOCATE, DUP2, CLEAR_CLOEXEC, CLOSE_RANGE, SETPGID, SIGMASK,
    # SIGACTION or NO_NEW_PRIVS is a real, posed mechanism failure, and it is
    # rendered with its stage so it can never be mistaken for an exec refusal.
    return "ExecFailed:" + stage + ":" + name


def _timed_out(timeout_disposition, exit_code, term_signal):
    """The three frozen timeout renderings, and nothing else."""
    if timeout_disposition == "":
        return "TimedOut"
    if timeout_disposition == "KilledByLauncher":
        name = SIGNAL_NAMES.get(term_signal)
        if name is None:
            return None
        return "TimedOut:KilledByLauncher:" + name
    if timeout_disposition == "ExitedDuringGrace":
        if not isinstance(exit_code, int) or not (
                EXIT_CODE_MIN <= exit_code <= EXIT_CODE_MAX):
            return None
        return "TimedOut:ExitedDuringGrace:" + str(exit_code)
    if timeout_disposition == "TerminationFailed":
        return "TimedOut:TerminationFailed"
    return None


# ------------------------------------------------------------ derivation rules
# Every case names exactly one rule. A rule returns (token, reason) with token
# None whenever the evidence it needs is absent -- never a fallback token.

def rule_admission(obs):
    """``refused:<Refusal>`` for the admission cases: E8, X2, X5, X6, X7."""
    spike = obs.get("spike")
    if spike is None:
        return None, "no parseable spike receipt"
    if spike.get("admission") != "refused":
        return None, ("the object was ADMITTED; a case that exists to observe a "
                      "refusal cannot be scored from an admitted run")
    return "refused:" + spike["refusal"], "admission refused at the frozen rule"


def rule_process_disposition(obs):
    """The direct-child lifecycle token, gated on real exec evidence.

    Used by every case whose frozen expectation is an ``Exited:``, ``Signaled:``,
    ``ExecFailed:``, ``TimedOut``, ``ExitStatusUnobservable`` or
    ``ExecStatusIndeterminate`` token.
    """
    spike = obs.get("spike")
    if spike is None:
        return None, "no parseable spike receipt"
    if spike.get("admission") == "refused":
        return ("refused:" + spike["refusal"],
                "refused at admission where a lifecycle outcome was expected")

    disposition = spike["process_disposition"]
    confirmation = obs.get("exec_confirmation")

    if disposition == "ExecFailed":
        token = _exec_failed(spike.get("exec_failed_stage"),
                             spike.get("exec_failed_errno"))
        if token is None:
            return None, ("the child reported a pre-exec failure with an errno "
                          "outside the frozen table; the stage record is not "
                          "interpretable")
        return token, "the child wrote an explicit pre-exec status record"

    if disposition == "ExecStatusIndeterminate":
        # The spike emits this for a short record and for a pre-exec timeout. The
        # two are distinguished by the declared schedule: only a case that passes
        # --stall-pre-exec-ms can time out before exec.
        if obs.get("declared_pre_exec_stall"):
            return ("ExecStatusIndeterminate:PreExecTimeout",
                    "the confirmation bound elapsed with no exec status")
        return ("ExecStatusIndeterminate",
                "the exec status could not be determined from the record")

    if disposition == "ExitStatusUnobservable":
        return ("ExitStatusUnobservable",
                "the child was reaped elsewhere; no status was observed")

    # Exited / Signaled / TimedOut all assert something about a body that ran.
    # Without exec evidence the launcher cannot honestly say which body, so the
    # honest token is the indeterminate one. This is the S5-vs-S1 rule and the
    # single most load-bearing line in this module.
    if confirmation == EXEC_UNINTERPRETABLE:
        return None, ("a helper report or payload was present and could not be "
                      "interpreted; the executed body is unidentified")
    if confirmation != EXEC_REACHED:
        return ("ExecStatusIndeterminate",
                "clean exec-status EOF with no positive evidence that the "
                "pinned image ever ran; EOF alone never means exec")

    if disposition == "TimedOut":
        token = _timed_out(spike.get("timeout_disposition"),
                           spike.get("exit_code"), spike.get("term_signal"))
        if token is None:
            return None, "timeout sub-disposition outside the frozen renderings"
        return token, "the deadline expired with the child still running"

    if disposition == "Exited":
        if obs.get("declared_body_length_changed"):
            # E6b/E6c rewrite the inode to a DIFFERENT length, so whatever ran is
            # not a frozen helper and its exit code carries no meaning. The
            # frozen vocabulary renders that as Exited:mutated rather than
            # inventing significance for an arbitrary number.
            return ("Exited:mutated",
                    "a body of changed length exited; its status is not the "
                    "pinned helper's status")
        token = _exited(spike.get("exit_code"))
        if token is None:
            return None, "exit code outside the observable range"
        return token, "the direct child exited and the status was observed"

    if disposition == "Signaled":
        token = _signaled(spike.get("term_signal"))
        if token is None:
            return None, "termination signal outside the frozen signal table"
        return token, "the direct child was terminated by a signal"

    return None, "unreachable disposition"


def rule_argv_exact(obs):
    """``argv_exact`` for A1-A4 and A6, from the report's element-wise argv."""
    report = obs.get("report")
    if report is None:
        return None, "no helper report; argv was never observed in the image"
    expected = obs.get("expected_argv")
    if expected is None:
        return None, "the case declared no expected argv"
    elements = report.get("argv")
    if not isinstance(elements, list):
        return None, "the report carries no argv array"
    observed = []
    for item in elements:
        if not isinstance(item, dict) or "value" not in item or "len" not in item:
            return None, "an argv element is not a {len,value} pair"
        value = item["value"]
        if not isinstance(value, str) or item["len"] != len(value.encode(
                "utf-8", "surrogateescape")):
            return None, "an argv element's declared length is not its length"
        observed.append(value)
    if observed == list(expected):
        return "argv_exact", "every argv element arrived byte-identical"
    return ("argv_mismatch",
            "argv differed from the frozen construction element-wise")


def rule_environ_empty(obs):
    """``environ_empty`` for V1. Under D-10 an empty array is the only pass."""
    report = obs.get("report")
    if report is None:
        return None, "no helper report; environ was never observed in the image"
    environ = report.get("environ")
    if not isinstance(environ, list):
        return None, "the report carries no environ array"
    if environ == []:
        return "environ_empty", "the executed image saw an exactly empty environ"
    return ("environ_inherited",
            "the executed image inherited " + str(len(environ)) + " name(s)")


def rule_fds_exactly_012(obs):
    """``fds_exactly_012`` for the F series.

    The descriptor set is read from the EXECUTED IMAGE. F1 and F4 additionally
    read ``/proc/self/fd`` as a cross-check, and the descriptor that reading
    consumes is the only one excused -- by number, declared by the case, never
    by pattern.
    """
    report = obs.get("report")
    if report is None:
        return None, "no helper report; the descriptor set was never observed"
    descriptors = report.get("descriptors")
    if not isinstance(descriptors, list):
        return None, "the report carries no descriptor array"
    numbers, cloexec_on_stdio = set(), []
    for item in descriptors:
        if not isinstance(item, dict) or "fd" not in item or "cloexec" not in item:
            return None, "a descriptor entry is not a {fd,cloexec} pair"
        numbers.add(item["fd"])
        if item["fd"] in (0, 1, 2) and item["cloexec"] is True:
            cloexec_on_stdio.append(item["fd"])
    for excused in obs.get("excused_descriptors") or ():
        numbers.discard(excused)
    if cloexec_on_stdio:
        # F6: a child stdio descriptor left FD_CLOEXEC is a real failure, not a
        # bookkeeping detail -- it is the dup2(fd,fd) no-op the spike clears.
        return ("stdio_cloexec_set",
                "descriptor(s) " + repr(sorted(cloexec_on_stdio)) +
                " survived exec with FD_CLOEXEC set")
    if numbers == {0, 1, 2}:
        return "fds_exactly_012", "exactly 0, 1 and 2 survived into the image"
    return ("fds_" + "_".join(str(n) for n in sorted(numbers)) if numbers
            else "fds_none",
            "the surviving descriptor set was " + repr(sorted(numbers)))


def _mask_is_empty(value):
    if not isinstance(value, str):
        return None
    text = value.strip()
    if not _MASK_RE.match(text):
        return None
    return int(text, 16) == 0


def rule_signals_reset(obs):
    """``signals_reset`` for F5.

    ``SigBlk`` must be empty and ``SigIgn`` must contain nothing: the spike
    resets every disposition to ``SIG_DFL`` before exec, because ``execve``
    clears handled dispositions but leaves IGNORED ones ignored. The helper
    reports the masks separately so blocked, ignored and caught are never
    conflated.
    """
    report = obs.get("report")
    if report is None:
        return None, "no helper report; signal state was never observed"
    signals = report.get("signals")
    if not isinstance(signals, dict):
        return None, "the report carries no signals object"
    blocked = _mask_is_empty(signals.get("SigBlk"))
    ignored = _mask_is_empty(signals.get("SigIgn"))
    if blocked is None or ignored is None:
        return None, "a signal mask is not a 64-bit hex mask"
    if blocked and ignored:
        return ("signals_reset",
                "the executed image had an empty blocked set and no inherited "
                "ignored disposition")
    detail = []
    if not blocked:
        detail.append("SigBlk=" + str(signals.get("SigBlk")))
    if not ignored:
        detail.append("SigIgn=" + str(signals.get("SigIgn")))
    return "signals_inherited", "; ".join(detail)


def rule_no_new_privs(obs):
    """``no_new_privs_1`` (N1) and ``no_new_privs_0`` (N2's control arm).

    Kernel-backed: the value is ``NoNewPrivs`` read from ``/proc/self/status``
    inside the executed image, never inferred from the fact that the spike
    called ``prctl``.
    """
    report = obs.get("report")
    if report is None:
        return None, "no helper report; NoNewPrivs was never observed"
    raw = report.get("no_new_privs")
    if not isinstance(raw, str):
        return None, "the report carries no NoNewPrivs field"
    text = raw.strip()
    if text == "1":
        return "no_new_privs_1", "the executed image observed NoNewPrivs: 1"
    if text == "0":
        return "no_new_privs_0", "the executed image observed NoNewPrivs: 0"
    return None, "NoNewPrivs was neither 0 nor 1"


def rule_stream_exact(obs):
    """``stream_exact`` for the O series.

    Every declared stream must match the frozen-recipe oracle on BOTH count and
    digest, and must carry the completeness the case declared. Capture state and
    process state are kept separate here on purpose: O7 requires a receipt that
    can carry an exit status and a capture failure simultaneously, so neither may
    absorb the other.
    """
    spike = obs.get("spike")
    if spike is None:
        return None, "no parseable spike receipt"
    if spike.get("admission") == "refused":
        return None, "refused at admission; no stream was ever produced"
    expected = obs.get("expected_streams")
    if not expected:
        return None, "the case declared no expected streams"
    mismatches = []
    for stream, want in sorted(expected.items()):
        block = spike.get(stream)
        if not isinstance(block, dict):
            return None, "the receipt carries no " + stream + " block"
        if block.get("bytes_drained") != want["bytes"]:
            mismatches.append(stream + ": drained " +
                              str(block.get("bytes_drained")) + " of " +
                              str(want["bytes"]))
        elif block.get("drained_sha256") != want["sha256"]:
            mismatches.append(stream + ": digest differs from the frozen recipe")
        if block.get("completeness") != want["completeness"]:
            mismatches.append(stream + ": completeness " +
                              str(block.get("completeness")) + " != " +
                              want["completeness"])
    if mismatches:
        return "stream_mismatch", "; ".join(mismatches)
    return ("stream_exact",
            "every declared stream matched the frozen recipe in count, digest "
            "and completeness")


def rule_interpreter_ran_with_devfd(obs):
    """``interpreter_ran_with_devfd`` for X2c.

    The script's interpreter runs and receives ``/dev/fd/N``. The evidence is the
    interpreter's own argv, which only exists if the exec fd was NOT CLOEXEC --
    X2b is the same fixture with the flag, and it must produce ENOENT instead.
    """
    report = obs.get("report")
    if report is None:
        return None, ("no interpreter report; whether the interpreter ran "
                      "cannot be established")
    elements = report.get("argv")
    if not isinstance(elements, list) or not elements:
        return None, "the interpreter report carries no argv"
    values = [e.get("value") for e in elements
              if isinstance(e, dict) and isinstance(e.get("value"), str)]
    if len(values) != len(elements):
        return None, "an interpreter argv element is not a string"
    if any(v.startswith("/dev/fd/") for v in values):
        return ("interpreter_ran_with_devfd",
                "the interpreter ran and received the exec descriptor as "
                "/dev/fd/N")
    return ("interpreter_ran_without_devfd",
            "the interpreter ran and no argument was a /dev/fd/N path")


def rule_privilege_transition_suppressed(obs):
    """``privilege_transition_suppressed`` for N3.

    N3 is CONDITIONAL on ``unprivileged_runner`` and is expected to be BLOCKED.
    It can only be posed where a real privileged identity exists, and this
    experiment manufactures none. Reaching this rule at all means a privileged
    fixture existed; the token then requires the executed image to have observed
    an unchanged effective uid alongside NoNewPrivs 1.
    """
    report = obs.get("report")
    if report is None:
        return None, "no helper report; no transition could be observed"
    euid, uid = report.get("euid"), report.get("uid")
    if not isinstance(euid, int) or not isinstance(uid, int):
        return None, "the report carries no uid/euid pair"
    nnp, _ = rule_no_new_privs(obs)
    if nnp != "no_new_privs_1":
        return None, ("NoNewPrivs was not observed as 1, so no suppression claim "
                      "is supported")
    if euid == uid:
        return ("privilege_transition_suppressed",
                "a set-id object did not raise the effective uid, with "
                "NoNewPrivs 1 observed in the image")
    return ("privilege_transition_occurred",
            "the effective uid changed across exec")


def rule_descendant_lifecycle(obs):
    """``descendant_survived`` / ``descendant_died`` for P1, P2 and P4.

    Both members are honest. Direct-child-only ownership is the frozen D-4
    contract, so the launcher makes NO process-tree containment claim and either
    observation is a recorded fact rather than a pass or a failure.
    """
    liveness = obs.get("descendant_alive_after_launch")
    if liveness is None:
        return None, ("the harness did not record whether the descendant "
                      "outlived launch()")
    if liveness is True:
        return ("descendant_survived",
                "the descendant was still alive after launch() returned")
    return ("descendant_died",
            "the descendant was no longer alive after launch() returned")


def rule_sweep(obs):
    """``sweep_issued`` / ``sweep_not_issued`` for P3.

    Both members are honest; the GATE is what matters, and it requires the sweep
    to appear strictly before the reap. After the reap the process-group id may
    have been recycled, and signalling it would reach processes the launcher
    never created.
    """
    spike = obs.get("spike")
    if spike is None:
        return None, "no parseable spike receipt"
    issued = spike.get("group_sweep_issued")
    if issued is True:
        return "sweep_issued", "the launcher issued a process-group sweep"
    if issued is False:
        return "sweep_not_issued", "the launcher issued no process-group sweep"
    return None, "the receipt does not record whether a sweep was issued"


def _trace(obs):
    trace = obs.get("trace")
    if trace is None:
        return None, "a traced case produced no syscall record"
    calls = trace.get("child_syscalls")
    if not isinstance(calls, list) or not all(isinstance(c, str) for c in calls):
        return None, "the trace carries no child syscall sequence"
    return calls, None


def rule_child_syscalls_within_frozen_set(obs):
    """``child_syscalls_within_frozen_set`` for M1.

    This is the evidence that REPLACES the architecture's bare assertion of "no
    allocation, no locking, no formatting, no panic path". M1 traces the
    PRODUCTION configuration, so a declared test-only injection syscall
    appearing here is a failure rather than an exemption -- without that split
    the minimality claim would hold only because the injecting cases happen not
    to be traced.
    """
    calls, why = _trace(obs)
    if calls is None:
        return None, why
    if obs.get("declared_injection_modes"):
        return None, ("M1 traces the production configuration; this run "
                      "declared a test-only injection mode")
    permitted = set(CHILD_PERMITTED_SYSCALLS)
    injections = set(CHILD_TEST_INJECTION_SYSCALLS)
    forbidden = set(CHILD_FORBIDDEN_SYSCALLS)
    strays = [c for c in calls if c not in permitted]
    if strays:
        seen_injection = sorted({c for c in strays if c in injections})
        seen_forbidden = sorted({c for c in strays if c in forbidden})
        detail = "syscalls outside the frozen child set: " + repr(sorted(set(strays)))
        if seen_injection:
            detail += "; declared test-only injections present: " + repr(seen_injection)
        if seen_forbidden:
            detail += "; explicitly forbidden: " + repr(seen_forbidden)
        return "child_syscalls_outside_frozen_set", detail
    return ("child_syscalls_within_frozen_set",
            "every traced child syscall is in the frozen permitted set")


def rule_identical_to_single_threaded_arm(obs):
    """``identical_to_single_threaded_arm`` for M2.

    The multi-threaded parent's child window must be the same sequence as the
    single-threaded arm's. Without this the mechanism is evidenced only for a
    single-threaded parent, which no real HELM caller is.
    """
    calls, why = _trace(obs)
    if calls is None:
        return None, why
    baseline = obs.get("single_threaded_child_syscalls")
    if baseline is None:
        return None, ("no single-threaded baseline sequence was recorded to "
                      "compare against")
    if not isinstance(baseline, list):
        return None, "the recorded baseline is not a syscall sequence"
    if calls == baseline:
        return ("identical_to_single_threaded_arm",
                "the multi-threaded parent produced the same child sequence")
    return ("differs_from_single_threaded_arm",
            "the child sequence differed from the single-threaded arm")


def rule_sequence_matches_frozen_stages(obs):
    """``sequence_matches_frozen_stages`` for M4.

    Matched against the frozen STAGES order, including that CHDIR precedes
    CLOSE_RANGE -- the working-directory descriptor is used before the range
    close destroys it -- and that NO_NEW_PRIVS precedes EXEC (D-11).
    """
    observed = obs.get("observed_stage_sequence")
    if observed is None:
        return None, "the trace recorded no stage sequence"
    if not isinstance(observed, list) or not all(
            isinstance(s, str) for s in observed):
        return None, "the recorded stage sequence is not a list of stage names"
    unknown = [s for s in observed if s not in SPIKE_STAGES]
    if unknown:
        return None, ("the trace contains stages outside the frozen vocabulary: "
                      + repr(unknown))
    order = {name: i for i, name in enumerate(STAGES)}
    positions = [order[s] for s in observed]
    if positions != sorted(positions):
        return ("sequence_differs_from_frozen_stages",
                "the observed stage order is not the frozen order: " +
                repr(observed))
    if "CHDIR" in observed and "CLOSE_RANGE" in observed:
        if observed.index("CHDIR") > observed.index("CLOSE_RANGE"):
            return ("sequence_differs_from_frozen_stages",
                    "CHDIR did not precede CLOSE_RANGE")
    if "NO_NEW_PRIVS" in observed and "EXEC" in observed:
        if observed.index("NO_NEW_PRIVS") > observed.index("EXEC"):
            return ("sequence_differs_from_frozen_stages",
                    "NO_NEW_PRIVS did not precede EXEC, which D-11 requires")
    return ("sequence_matches_frozen_stages",
            "the child sequence matched the frozen stage order")


def rule_pidfd_acquired_atomically(obs):
    """``pidfd_acquired_atomically`` for M3.

    M3 is CONDITIONAL on ``clone3_unavailable``. Reaching this rule means
    preflight found clone3 permitted, so an absent atomic acquisition here is a
    genuine mechanism result and NOT the host condition: the host condition was
    already settled, without creating a child, before any case was posed.
    """
    acquisition = obs.get("pidfd_acquisition")
    if acquisition is None:
        return None, "the harness recorded no pidfd acquisition mode"
    if acquisition == "clone3_pidfd":
        return ("pidfd_acquired_atomically",
                "the pidfd was obtained atomically from clone3(CLONE_PIDFD)")
    if acquisition in ("fork_pidfd_open", "none"):
        return ("pidfd_not_acquired_atomically",
                "the pidfd was not obtained atomically: " + acquisition)
    return None, "unrecognised pidfd acquisition mode: " + repr(acquisition)


def rule_rejected_acquisition(obs):
    """M5's three recorded outcomes for the REJECTED fork+pidfd_open arm.

    Not a candidate mechanism. It is recorded to evidence why
    ``clone3(CLONE_PIDFD)`` is primary rather than asserting it, and its gate
    forbids reporting a status that was never observed.
    """
    outcome = obs.get("rejected_acquisition_outcome")
    if outcome is None:
        return None, ("the harness recorded no outcome for the rejected "
                      "acquisition arm")
    if outcome in ("pidfd_open_esrch", "waitid_echild", "pidfd_open_succeeded"):
        return outcome, "recorded outcome of the rejected fork+pidfd_open arm"
    return None, "unrecognised rejected-acquisition outcome: " + repr(outcome)


# The complete rule registry. driver.py names one of these per case, and the
# completeness test proves every name resolves and every frozen token is
# reachable from at least one rule.
RULES = {
    "admission": rule_admission,
    "process_disposition": rule_process_disposition,
    "argv_exact": rule_argv_exact,
    "environ_empty": rule_environ_empty,
    "fds_exactly_012": rule_fds_exactly_012,
    "signals_reset": rule_signals_reset,
    "no_new_privs": rule_no_new_privs,
    "stream_exact": rule_stream_exact,
    "interpreter_ran_with_devfd": rule_interpreter_ran_with_devfd,
    "privilege_transition_suppressed": rule_privilege_transition_suppressed,
    "descendant_lifecycle": rule_descendant_lifecycle,
    "sweep": rule_sweep,
    "child_syscalls_within_frozen_set": rule_child_syscalls_within_frozen_set,
    "identical_to_single_threaded_arm": rule_identical_to_single_threaded_arm,
    "sequence_matches_frozen_stages": rule_sequence_matches_frozen_stages,
    "pidfd_acquired_atomically": rule_pidfd_acquired_atomically,
    "rejected_acquisition": rule_rejected_acquisition,
}


def derive(rule_name, obs):
    """Apply one named rule. Returns ``(token_or_None, reason)``.

    An unknown rule name is a driver defect, not an experiment result, so it
    raises rather than returning a token.
    """
    if rule_name not in RULES:
        raise KeyError("no such outcome rule: " + repr(rule_name))
    token, reason = RULES[rule_name](obs)
    if token is not None and not isinstance(token, str):
        raise TypeError("rule " + rule_name + " returned a non-string token")
    return token, reason


def injection_modes_in(flags):
    """Which declared test-only injection modes a spike invocation carries.

    Used by the M-series rules and by the driver's own self-check, so that a
    production trace can never be taken from an injecting run.
    """
    declared = set(CHILD_INJECTION_MODES)
    return sorted({f for f in flags if f in declared})
