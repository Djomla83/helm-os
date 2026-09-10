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
    STRACE_MIN_VERSION,
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
    1: "EPERM", 2: "ENOENT", 3: "ESRCH", 4: "EINTR", 5: "EIO", 8: "ENOEXEC",
    9: "EBADF", 10: "ECHILD", 12: "ENOMEM", 13: "EACCES", 14: "EFAULT",
    20: "ENOTDIR", 21: "EISDIR", 22: "EINVAL", 24: "EMFILE", 26: "ETXTBSY",
    36: "ENAMETOOLONG", 38: "ENOSYS", 40: "ELOOP",
}

# Named because M5's whole outcome vocabulary turns on exactly these two.
ESRCH, ECHILD = 3, 10

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


# --------------------------------------------------- the retained capture
# PRE-D7-B1. The launcher now emits the bounded capture prefix base64-encoded,
# so the helper's own report -- the first item in the definition's evidence
# preference order -- finally reaches the harness. These bytes are INTERNAL:
# they may contain anything the executed image wrote, so they are decoded here,
# reduced to structured facts, and never published. A secret in base64 is still
# a secret, and evidence.py refuses to serialise this field at all.

# The five states section 3 of the correction requires be distinguishable, plus
# the one that says the stream itself could not settle the question.
REPORT_COMPLETE = "complete"
REPORT_TRUNCATED = "truncated"
REPORT_MALFORMED = "malformed"
REPORT_ABSENT = "absent"
REPORT_STREAM_INCOMPLETE = "stream_incomplete"

REPORT_STATES = frozenset({REPORT_COMPLETE, REPORT_TRUNCATED, REPORT_MALFORMED,
                           REPORT_ABSENT, REPORT_STREAM_INCOMPLETE})

# Only a COMPLETE report is evidence. Everything else is missing information,
# and missing information never becomes a token.
_USABLE_REPORT_STATES = frozenset({REPORT_COMPLETE})


def decode_capture(stream_block):
    """Decode one stream's retained prefix. Returns bytes, or None if absent.

    ``None`` means the launcher emitted no prefix field at all -- an older spike,
    or a receipt that is not the frozen one. It is never an empty capture: an
    empty capture is ``b""`` and is a real observation.
    """
    import base64
    if not isinstance(stream_block, dict):
        return None
    encoded = stream_block.get("capture_prefix_base64")
    if encoded is None:
        return None
    if not isinstance(encoded, str):
        return None
    try:
        raw = base64.b64decode(encoded, validate=True)
    except Exception:                                  # noqa: BLE001
        return None
    declared = stream_block.get("capture_prefix_length")
    if isinstance(declared, int) and declared != len(raw):
        # The launcher's own count and its own bytes disagree, so neither can be
        # trusted to describe the stream.
        return None
    return raw


def _capture_is_decisive(stream_block):
    """Whether this stream's prefix can settle a question about the whole stream.

    It cannot when the prefix stopped at the bound while more bytes were drained,
    and it cannot when a descendant retained the writer, because in both cases
    what is missing is unknown rather than known to be nothing.
    """
    if not isinstance(stream_block, dict):
        return False
    if stream_block.get("capture_prefix_truncated") is True:
        return False
    if stream_block.get("completeness") != "CompleteAtEof":
        return False
    drained = stream_block.get("bytes_drained")
    kept = stream_block.get("capture_prefix_length")
    if isinstance(drained, int) and isinstance(kept, int) and kept < drained:
        return False
    return True


def helper_report_state(stream_block):
    """Classify descriptor 1's retained prefix. Returns ``(state, report, payload)``.

    The distinction that matters is between "the image ran and did not report"
    and "we cannot tell". S2, S5 and S7 rest on the ABSENCE of a report, and an
    absence is only evidence when the stream that would have carried it is known
    to be complete.
    """
    raw = decode_capture(stream_block)
    if raw is None:
        return REPORT_STREAM_INCOMPLETE, None, b""
    payload, report = parse_helper_report(raw)
    decisive = _capture_is_decisive(stream_block)
    if report is not None:
        return REPORT_COMPLETE, report, payload
    if REPORT_SENTINEL in raw:
        # The sentinel arrived and what followed did not parse. If the prefix was
        # cut at the bound the report was truncated; otherwise it is malformed,
        # and the two are not the same finding.
        if decisive:
            return REPORT_MALFORMED, None, payload
        return REPORT_TRUNCATED, None, payload
    if decisive:
        return REPORT_ABSENT, None, payload
    return REPORT_STREAM_INCOMPLETE, None, payload


# ----------------------------------------------------------- exec confirmation
# The four states section 11 of the trial contract requires be distinguishable.
EXEC_PRE_EXEC_ERROR = "pre_exec_error"        # the child wrote a status record
EXEC_DIED_BEFORE_EXEC = "died_before_exec"    # clean EOF, and NOTHING ran
EXEC_REACHED = "exec_reached"                 # positive evidence the image ran
EXEC_UNINTERPRETABLE = "uninterpretable"      # a report arrived and made no sense


def exec_confirmation(spike, report, payload_is_recipe=None,
                      report_state=None):
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
    if report_state in (REPORT_TRUNCATED, REPORT_MALFORMED,
                        REPORT_STREAM_INCOMPLETE):
        # A report may or may not have arrived; the stream cannot say. Calling
        # that "nothing ran" would turn a gap in the evidence into a finding,
        # which is exactly the move S5 exists to make impossible.
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

# ------------------------------------------------------------ syscall records
# Evidence item 2 in the definition's preference order, collected ONLY for the
# eight cases declared traced: true. The parser reads a tracer's text and never
# runs one; the driver owns invocation.

# Which frozen stage each child syscall belongs to. RELOCATE is absent on
# purpose: it happens in the PARENT before the clone, so it can never appear in
# the child window this maps.
_SYSCALL_STAGE = {
    "dup2": "DUP2", "dup3": "DUP2",
    "fcntl": "CLEAR_CLOEXEC",
    "fchdir": "CHDIR",
    "close_range": "CLOSE_RANGE",
    "setpgid": "SETPGID",
    "rt_sigprocmask": "SIGMASK",
    "rt_sigaction": "SIGACTION",
    "prctl": "NO_NEW_PRIVS",
    "execveat": "EXEC",
}

# ------------------------------------------------- R-1: fragment joining
# Under -f, strace splits a syscall whenever another traced task produces output
# before it returns. A clone-family call is the routine case, because the child
# starts running while the parent is still inside the syscall:
#
#   111 clone3({...} <unfinished ...>
#   222 execveat(...)             = 0
#   111 <... clone3 resumed> => {pidfd=[4]}, 88) = 222
#
# Reading the unfinished half alone sees no return value, which the bounded
# review found would be reported as clone3_failed -- a FAIL turning a FORMATTING
# artefact into MECHANISM_REJECTED. Fragments are therefore rejoined into one
# logical observation BEFORE any clone3 semantics are interpreted, and a
# fragment that cannot be rejoined is INVALID, never a mechanism failure.
_TASK_PREFIX = re.compile(r"\A(?:\[pid\s+(?P<p1>\d+)\]|(?P<p2>\d+))\s+")
_UNFINISHED = re.compile(r"\A(?P<head>.*?)\s*<unfinished\s*\.\.\.>\s*\Z")
_RESUMED = re.compile(r"\A<\.\.\.\s+(?P<call>[a-z_][a-z0-9_]*)\s+resumed>"
                      r"(?P<tail>.*)\Z")
_CALL_NAME = re.compile(r"\A(?P<call>[a-z_][a-z0-9_]*)\(")


def _split_task(line):
    """``(task_id, remainder, malformed)`` for one raw tracer line.

    The well-formed prefix is tried FIRST and the malformed check only runs when
    it fails. Testing "malformed" first is what a earlier draft did, and
    ``\\s+`` backtracking then let ``[pid   222]`` satisfy the malformed pattern
    by matching one space as the "non-digit" character -- flagging every
    correctly prefixed line as unreadable.
    """
    text = line.rstrip("\n").rstrip()
    match = _TASK_PREFIX.match(text)
    if match:
        task = int(match.group("p1") or match.group("p2"))
        return task, text[match.end():], False
    if text.startswith("[pid"):
        # It announced a task attribution and did not supply a readable one.
        return None, text, True
    return None, text, False


# ------------------------------------------------- V-1: logical record bounds
# A reconstructed syscall must be parsed as ONE logical record. The final
# bounded review found two ways this went wrong:
#
#   * a truncated record has no return value at all, and deriving "success"
#     from `return > 0` silently turned MISSING EVIDENCE into an observed
#     failure -- clone3_failed, a FAIL, and therefore MECHANISM_REJECTED;
#   * a return-value regex that scans the whole text can wander past the end of
#     the syscall and adopt the NEXT line's "= N", inventing a child pid.
#
# Both are closed by finding the syscall's own closing parenthesis and
# requiring everything after it to be exactly the result. Anything else is an
# unbounded record, which is incomplete evidence rather than a mechanism fact.
_RESULT_TAIL = re.compile(
    r"\A\s*=\s*(?P<ret>-?\d+|\?)"
    r"(?:\s+(?P<errno>[A-Z][A-Z0-9_]*))?"
    r"(?:\s*\([^)]*\))?\s*\Z")

# How the syscall's return value was observed. Three states, never two: the
# difference between "the kernel said no" and "we never saw what it said" is
# the difference between a result and a gap in the evidence.
RETURN_OBSERVED_SUCCESS = "observed_success"
RETURN_OBSERVED_ERROR = "observed_error"
RETURN_NOT_OBSERVED = "not_observed"


def split_syscall_record(text):
    """``(name, args, result_text)`` for ONE logical syscall record, or None.

    The argument list is delimited by counting parentheses from the syscall's
    own opening one, skipping quoted strings so a path containing ``(`` cannot
    unbalance it. Everything after the matching close must be the result and
    nothing else, so a record that ran into a following syscall -- or that
    stops before its own close -- is rejected rather than half-read.
    """
    if not isinstance(text, str):
        return None
    match = _CALL_NAME.match(text.strip())
    if not match:
        return None
    body = text.strip()
    name = match.group("call")
    i = match.end() - 1            # index of the opening parenthesis
    depth, in_string, escaped = 0, False, False
    close = -1
    while i < len(body):
        ch = body[i]
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
        elif ch == '"':
            in_string = True
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                close = i
                break
        i += 1
    if close < 0:
        return None                # never closed: truncated or absorbed
    return name, body[match.end():close], body[close + 1:]


def parse_return_state(result_text):
    """``(state, value, errno_name)`` from the text after the closing paren."""
    if result_text is None:
        return RETURN_NOT_OBSERVED, None, None
    match = _RESULT_TAIL.match(result_text)
    if not match:
        return RETURN_NOT_OBSERVED, None, None
    raw = match.group("ret")
    if raw == "?":
        return RETURN_NOT_OBSERVED, None, None
    value = int(raw)
    errno_name = match.group("errno")
    if value < 0 or errno_name:
        return RETURN_OBSERVED_ERROR, value, errno_name
    return RETURN_OBSERVED_SUCCESS, value, None


def join_trace_fragments(text):
    """Rejoin ``<unfinished ...>`` / ``<... call resumed>`` pairs per task.

    Returns ``{"records": [...], "unmatched_unfinished": [...],
    "orphan_resumed": [...], "ambiguous": [...], "malformed": [...]}``.

    Joining is keyed on the TASK the tracer attributed the line to, so fragments
    belonging to different tasks are never spliced together, and a second
    unfinished call from the same task before its resume is recorded as
    ambiguous rather than guessed at.
    """
    if text is None:
        return None
    if isinstance(text, bytes):
        text = text.decode("utf-8", "replace")

    records, pending = [], {}
    unmatched, orphans, ambiguous, malformed = [], [], [], []

    for raw in text.splitlines():
        task, body, bad_prefix = _split_task(raw)
        if bad_prefix:
            malformed.append(raw)
            continue
        if not body:
            continue

        resumed = _RESUMED.match(body)
        if resumed:
            held = pending.pop(task, None)
            if held is None:
                orphans.append({"task": task, "call": resumed.group("call")})
                continue
            if held["call"] != resumed.group("call"):
                # The tracer resumed a different call than the one this task
                # left unfinished. Splicing them would fabricate an observation.
                orphans.append({"task": task, "call": resumed.group("call")})
                unmatched.append({"task": task, "call": held["call"]})
                continue
            records.append({"task": task, "call": held["call"],
                            "text": held["head"] + resumed.group("tail"),
                            "joined": True})
            continue

        unfinished = _UNFINISHED.match(body)
        if unfinished:
            head = unfinished.group("head")
            name = _CALL_NAME.match(head)
            if name is None:
                malformed.append(raw)
                continue
            if task in pending:
                ambiguous.append({"task": task, "call": name.group("call")})
                unmatched.append(pending.pop(task))
            pending[task] = {"task": task, "call": name.group("call"),
                             "head": head}
            continue

        name = _CALL_NAME.match(body)
        records.append({"task": task,
                        "call": name.group("call") if name else None,
                        "text": body, "joined": False})

    for held in pending.values():
        unmatched.append({"task": held["task"], "call": held["call"]})

    return {"records": records, "unmatched_unfinished": unmatched,
            "orphan_resumed": orphans, "ambiguous": ambiguous,
            "malformed": malformed}


def parse_strace_child_window(text):
    """The child's syscall window, from the clone3 return to execveat.

    Returns ``{"child_syscalls": [...], "stage_sequence": [...], "child_pid": N}``
    or ``None`` when the window cannot be identified. ``None`` is not an empty
    window: a case declared traced whose trace produced no record is INVALID, and
    conflating the two would let a tracer failure read as a minimal child.
    """
    joined = join_trace_fragments(text)
    if joined is None:
        return None

    # V-3/V-4: the integrity of the record travels WITH the window, so one
    # shared checker gate can reject a malformed trace for every traced case,
    # including the four whose outcome rules never read the window.
    integrity_ok = not any(joined[k] for k in
                           ("unmatched_unfinished", "orphan_resumed",
                            "ambiguous", "malformed"))

    child_pid = None
    for record in joined["records"]:
        if record["call"] != "clone3":
            continue
        parts = split_syscall_record(record["text"])
        if parts is None:
            # Unbounded or truncated: no return value was observed, which is
            # not the same as a clone3 that returned an error.
            integrity_ok = False
            break
        state, value, _ = parse_return_state(parts[2])
        if state == RETURN_OBSERVED_SUCCESS:
            child_pid = value
        else:
            integrity_ok = integrity_ok and state == RETURN_OBSERVED_ERROR
        break
    if child_pid is None:
        return None

    calls, seen_clone = [], False
    for record in joined["records"]:
        if not seen_clone:
            if record["call"] == "clone3":
                seen_clone = True
            continue
        if record["task"] != child_pid or record["call"] is None:
            continue
        calls.append(record["call"])
        if record["call"] == "execveat":
            break

    if not calls:
        return None
    stages, last = [], None
    for call in calls:
        stage = _SYSCALL_STAGE.get(call)
        # Consecutive syscalls of one stage are one stage entry: three dup2 calls
        # are one DUP2, not three.
        if stage is not None and stage != last:
            stages.append(stage)
            last = stage
    return {"child_syscalls": calls, "stage_sequence": stages,
            "child_pid": child_pid, "integrity_ok": integrity_ok}


# ------------------------------------------------- M3-T: pidfd acquisition
# Owner amendment M3-T. M3 is a traced case, and its expectation rests on the
# EXTERNAL syscall record. Nothing here reads a launcher field naming its own
# acquisition mode: a receipt saying "I used clone3" is precisely the
# self-assertion M3 exists to avoid, and this module refuses to accept one.
#
# The bounded claim is about THIS execution's direct child only. It is not a
# claim about Linux pidfds in general and not a claim that the kernel is atomic;
# frozen_cases.M3_CLAIM_EXCLUSIONS states that in the manifest itself.
#
# R-2: the CLOSURE evidence form is REMOVED. It concluded "no other route could
# exist" from the absence of pidfd_open, which the bounded review showed depends
# on frozen-source invariants the trace never observes -- dup2, pidfd_getfd,
# /proc/<pid>, legacy clone(CLONE_PIDFD), a second fork() child and SCM_RIGHTS
# each satisfied it. M3 now requires the tracer-rendered pidfd output, which
# upstream strace prints via printnum_fd() and is therefore available.

# strace renders clone3 with its clone_args struct decoded, followed by an
# output block for the fields the kernel writes back.
_CLONE3_LINE = re.compile(r"clone3\(\{(?P<args>[^}]*)\}"
                          r"(?:\s*=>\s*\{(?P<out>[^}]*)\})?")
_CLONE3_RESULT = re.compile(r"\)\s*=\s*(?P<ret>-?\d+)")
_FLAGS = re.compile(r"flags\s*=\s*(?P<flags>[A-Za-z0-9_|]+)")
_PIDFD_PTR = re.compile(r"pidfd\s*=\s*(?P<ptr>0x[0-9a-fA-F]+)")
_PIDFD_OUT = re.compile(r"pidfd\s*=\s*\[(?P<fd>\d+)\]")
_PIDFD_OPEN = re.compile(r"pidfd_open\(\s*(?P<pid>\d+)")
_WAITID_PIDFD = re.compile(r"waitid\(\s*P_PIDFD\s*,\s*(?P<fd>\d+)")
_SI_PID = re.compile(r"si_pid\s*=\s*(?P<pid>\d+)")
_PIDFD_SEND = re.compile(r"pidfd_send_signal\(\s*(?P<fd>\d+)")
_POLL_FD = re.compile(r"fd\s*=\s*(?P<fd>\d+)")

# The frozen lifecycle operations that can correlate a descriptor with the
# launcher's direct-child handle. waitid(P_PIDFD, N) is the decisive one, and
# only when the siginfo it renders names the direct child (R-3).
LIFECYCLE_WAITID = "waitid_p_pidfd"
LIFECYCLE_SEND_SIGNAL = "pidfd_send_signal"
LIFECYCLE_POLL = "poll"

# Retained so an evidence record says how fact E was established. After R-2
# there is exactly one admissible form.
EVIDENCE_DIRECT = "direct"


def parse_pidfd_acquisition(text):
    """Extract the M3 acquisition facts from a syscall record.

    Returns a dict of RAW facts, or ``None`` when the record cannot be read at
    all. Raw pids and descriptor numbers are experiment-local identifiers: they
    are used here for correlation and are normalised away before publication by
    :func:`normalise_acquisition`.

    This parser reads text. It runs no tracer and executes nothing.
    """
    joined = join_trace_fragments(text)
    if joined is None:
        return None
    if not joined["records"] and not any(
            joined[k] for k in ("unmatched_unfinished", "orphan_resumed",
                                "ambiguous", "malformed")):
        return None

    facts = {
        "clone3_seen": False,
        "clone3_flags": [],
        "clone_pidfd_flag": False,
        "pidfd_output_pointer_supplied": False,
        "clone3_return": None,
        "clone3_return_state": RETURN_NOT_OBSERVED,
        "clone3_errno": None,
        "clone3_record_bounded": None,
        "pidfd_from_clone3": None,
        "pidfd_open_calls": [],
        "lifecycle_uses": [],
        "evidence_form": None,
        "clone3_call_count": 0,
        # R-1: trace integrity. A record whose fragments did not rejoin is
        # incomplete evidence, and the rule refuses it rather than reading the
        # surviving half as a mechanism result.
        "fragments_unmatched": len(joined["unmatched_unfinished"]),
        "fragments_orphaned": len(joined["orphan_resumed"]),
        "fragments_ambiguous": len(joined["ambiguous"]),
        "lines_malformed": len(joined["malformed"]),
        "clone3_was_joined": False,
    }

    for record in joined["records"]:
        line = record["text"]
        match = _CLONE3_LINE.search(line)
        if match:
            facts["clone3_call_count"] += 1
            if not facts["clone3_seen"]:
                facts["clone3_seen"] = True
                facts["clone3_was_joined"] = bool(record["joined"])
                args = match.group("args") or ""
                flags = _FLAGS.search(args)
                if flags:
                    facts["clone3_flags"] = [
                        f for f in flags.group("flags").split("|") if f]
                    facts["clone_pidfd_flag"] = (
                        "CLONE_PIDFD" in facts["clone3_flags"])
                facts["pidfd_output_pointer_supplied"] = bool(
                    _PIDFD_PTR.search(args))
                out = match.group("out")
                if out:
                    written = _PIDFD_OUT.search(out)
                    if written:
                        facts["pidfd_from_clone3"] = int(written.group("fd"))
                        facts["evidence_form"] = EVIDENCE_DIRECT
                # V-1. The return value is read from the syscall's OWN
                # logical record: everything after its matching close
                # parenthesis, and nothing else. A record that never closed --
                # truncated, or run together with the following line -- yields
                # RETURN_NOT_OBSERVED, never an observed failure.
                parts = split_syscall_record(line)
                facts["clone3_record_bounded"] = parts is not None
                if parts is not None:
                    state, value, errno_name = parse_return_state(parts[2])
                    facts["clone3_return_state"] = state
                    facts["clone3_errno"] = errno_name
                    if state == RETURN_OBSERVED_SUCCESS:
                        facts["clone3_return"] = value
                    elif state == RETURN_OBSERVED_ERROR:
                        facts["clone3_return"] = value

        opened = _PIDFD_OPEN.search(line)
        if opened:
            result = _CLONE3_RESULT.search(line)
            facts["pidfd_open_calls"].append({
                "target_pid": int(opened.group("pid")),
                "returned": int(result.group("ret")) if result else None,
            })

        reaped = _WAITID_PIDFD.search(line)
        if reaped:
            si_pid = _SI_PID.search(line)
            facts["lifecycle_uses"].append({
                "op": LIFECYCLE_WAITID, "fd": int(reaped.group("fd")),
                "si_pid": int(si_pid.group("pid")) if si_pid else None})
        signalled = _PIDFD_SEND.search(line)
        if signalled:
            facts["lifecycle_uses"].append(
                {"op": LIFECYCLE_SEND_SIGNAL, "fd": int(signalled.group("fd")),
                 "si_pid": None})
        if "poll(" in line:
            for polled in _POLL_FD.finditer(line):
                facts["lifecycle_uses"].append(
                    {"op": LIFECYCLE_POLL, "fd": int(polled.group("fd")),
                     "si_pid": None})
    return facts


def _correlated_pidfd(facts):
    """``(descriptor, reason)`` for the lifecycle handle of the DIRECT CHILD.

    R-3: a ``waitid(P_PIDFD, N)`` only correlates when the siginfo it renders
    names the direct child. Entries with no rendered ``si_pid`` -- an ECHILD
    retry, for instance -- are ignored rather than fatal, so a later failed reap
    cannot erase an earlier correct one.

    V-2: the evidence set must first be INTERNALLY CONSISTENT. A pidfd refers to
    exactly one process, so a descriptor that reaps two different pids is a
    contradictory record. Filtering to the wanted child and discarding the
    contradiction would hide a tracer or parser fault behind a PASS, so the
    contradiction is detected BEFORE any filtering.
    """
    child_pid = facts.get("clone3_return")
    if child_pid is None:
        return None, "no observed clone3 child pid to correlate against"

    by_fd = {}
    for use in facts.get("lifecycle_uses", ()):
        if use.get("op") != LIFECYCLE_WAITID:
            continue
        pid = use.get("si_pid")
        if pid is None:
            continue
        by_fd.setdefault(use["fd"], set()).add(pid)

    contradictory = [fd for fd, pids in by_fd.items() if len(pids) > 1]
    if contradictory:
        return None, ("one descriptor reaped more than one distinct child; a "
                      "pidfd refers to exactly one process, so this record is "
                      "self-contradictory")

    candidates = [fd for fd, pids in by_fd.items()
                  if next(iter(pids)) == child_pid]
    if len(candidates) == 1:
        return candidates[0], None
    if not candidates:
        return None, ("no waitid(P_PIDFD, ...) reports si_pid == the clone3 "
                      "return, so nothing ties a descriptor to the direct child")
    return None, ("more than one descriptor claims the direct child; the "
                  "correlation is ambiguous")


def normalise_acquisition(facts):
    """The publishable form: no raw pid, no raw descriptor number.

    Raw pids and fds are experiment-local identifiers and must not become part
    of a receipt's identity. They are replaced by the two roles that carry the
    meaning, and everything else is a boolean, a count or a decoded flag name.
    """
    if not isinstance(facts, dict):
        return None
    correlated, _ = _correlated_pidfd(facts)
    return {
        "clone3_seen": bool(facts.get("clone3_seen")),
        "clone3_call_count": facts.get("clone3_call_count", 0),
        "clone3_flags": list(facts.get("clone3_flags") or ()),
        "clone_pidfd_flag": bool(facts.get("clone_pidfd_flag")),
        "pidfd_output_pointer_supplied":
            bool(facts.get("pidfd_output_pointer_supplied")),
        "clone3_return_state": facts.get("clone3_return_state"),
        "clone3_record_bounded": facts.get("clone3_record_bounded"),
        "clone3_errno": facts.get("clone3_errno"),
        "clone3_was_joined": bool(facts.get("clone3_was_joined")),
        "direct_child": ("DIRECT_CHILD" if facts.get("clone3_return_state")
                         == RETURN_OBSERVED_SUCCESS else None),
        "direct_child_pidfd": (
            "DIRECT_CHILD_PIDFD" if correlated is not None else None),
        "pidfd_from_same_syscall": facts.get("pidfd_from_clone3") is not None,
        "pidfd_open_call_count": len(facts.get("pidfd_open_calls") or ()),
        "lifecycle_ops": sorted({use["op"]
                                 for use in facts.get("lifecycle_uses") or ()}),
        "evidence_form": facts.get("evidence_form"),
        "trace_integrity": {
            "fragments_unmatched": facts.get("fragments_unmatched", 0),
            "fragments_orphaned": facts.get("fragments_orphaned", 0),
            "fragments_ambiguous": facts.get("fragments_ambiguous", 0),
            "lines_malformed": facts.get("lines_malformed", 0),
        },
    }


# --------------------------------------------------- R-4: tracer supportedness
# Pure text helpers. They decide whether the environment can produce the trace
# contract the eight traced cases depend on; the RUNNER turns a negative answer
# into a preflight HALT, because a missing tracer stops the trial rather than
# blocking a case.
_STRACE_VERSION = re.compile(r"(?:\A|\s)v?(?P<maj>\d+)\.(?P<min>\d+)")


def parse_strace_version(reported):
    """``(major, minor)`` from ``strace --version`` output, or None.

    Accepts the list preflight records or a bare string. ``None`` means the
    version could not be read, which is treated as unsupported rather than
    assumed good.
    """
    if reported is None:
        return None
    if isinstance(reported, (list, tuple)):
        reported = reported[0] if reported else ""
    if not isinstance(reported, str):
        return None
    if reported.startswith("<unavailable"):
        return None
    match = _STRACE_VERSION.search(reported)
    if not match:
        return None
    return int(match.group("maj")), int(match.group("min"))


def strace_supported(path, reported_version, minimum=None):
    """``(ok, reason)`` for the frozen tracer requirement.

    Unsupported is never silently downgraded into a case-level block: the caller
    HALTS preflight, which is the whole point of R-4.
    """
    if minimum is None:
        minimum = STRACE_MIN_VERSION
    if not path:
        return False, ("no strace on PATH; a usable tracer is a mandatory "
                       "pretrial requirement and no fallback tracer exists")
    version = parse_strace_version(reported_version)
    if version is None:
        return False, ("strace is present at " + str(path) + " but its version "
                       "could not be read, so the trace contract cannot be "
                       "established")
    if version < tuple(minimum):
        return False, ("strace %d.%d is older than the frozen floor %d.%d, "
                       "below which clone3 clone_args decoding cannot be "
                       "relied on" % (version + tuple(minimum)))
    return True, "strace %d.%d at %s meets the frozen floor" % (
        version + (str(path),))


def _usable_report(obs):
    """The helper report, but ONLY when it is complete and interpretable.

    Returns ``(report, None)`` or ``(None, reason)``. Every rule written against
    the report goes through here, so a truncated or malformed report can never
    be read as a partial answer: section 3 of the correction requires that
    incomplete report evidence stays non-success, and this is where that is
    enforced once rather than seven times.
    """
    state = obs.get("report_state")
    report = obs.get("report")
    if report is not None and state in _USABLE_REPORT_STATES:
        return report, None
    if state == REPORT_TRUNCATED:
        return None, ("the helper report was cut off at the capture bound; a "
                      "truncated report is not a partial answer")
    if state == REPORT_MALFORMED:
        return None, ("a helper report arrived and did not parse; the executed "
                      "image is unidentified")
    if state == REPORT_ABSENT:
        return None, ("no helper report was written, and the stream that would "
                      "have carried it was complete")
    if state == REPORT_STREAM_INCOMPLETE:
        return None, ("the stream could not settle whether a report was "
                      "written; absence here is not evidence")
    return None, "no helper report reached the harness"


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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    report, why = _usable_report(obs)
    if report is None:
        return None, why
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
    facts = obs.get("acquisition")
    if not isinstance(facts, dict):
        return None, ("a traced case produced no syscall record of the "
                      "acquisition")

    # A launcher field naming its own acquisition mode is never consulted. If
    # one ever appears in a receipt it is ignored here on purpose: M3 exists to
    # evidence the acquisition externally, not to relay the launcher's account
    # of itself.

    # R-1 -- trace integrity comes FIRST. A record whose <unfinished ...> and
    # <... resumed> fragments did not rejoin is incomplete evidence, and reading
    # the surviving half would turn a tracer FORMATTING artefact into a
    # mechanism verdict. Every one of these is INVALID, never FAIL.
    if facts.get("lines_malformed"):
        return None, ("the record contains lines whose task attribution could "
                      "not be read; fragments cannot be rejoined safely")
    if facts.get("fragments_ambiguous"):
        return None, ("a task left two syscalls unfinished before either "
                      "resumed; the fragments cannot be paired unambiguously")
    if facts.get("fragments_orphaned"):
        return None, ("the record contains a resumed fragment with no matching "
                      "unfinished half")
    if facts.get("fragments_unmatched"):
        return None, ("the record contains an unfinished syscall that never "
                      "resumed; the trace is incomplete")

    # A -- exactly one parent-side clone3 call.
    if not facts.get("clone3_seen"):
        return None, ("no clone3 call is present in the record; a tracer that "
                      "did not decode it and a launcher that did not call it "
                      "are indistinguishable from here")
    if facts.get("clone3_call_count", 0) > 1:
        return None, ("the record contains more than one clone3 call, so the "
                      "direct child cannot be identified unambiguously")

    # V-1 -- the record must be BOUNDED before anything in it means anything.
    # An unclosed argument list means the record was truncated, or ran together
    # with the following line; either way its "= N" is not this syscall's.
    if facts.get("clone3_record_bounded") is False:
        return None, ("the clone3 record does not close its own argument list, "
                      "so it was truncated or ran into another record; no "
                      "return value can be attributed to it")

    # D -- THREE states, never two. "The kernel said no" and "we never saw what
    # it said" are different facts, and only the first is a mechanism result.
    state = facts.get("clone3_return_state")
    if state == RETURN_NOT_OBSERVED:
        return None, ("the clone3 record carries no observed return value; "
                      "missing evidence is not an observed failure")
    if state == RETURN_OBSERVED_ERROR:
        errno_name = facts.get("clone3_errno")
        return ("clone3_failed",
                "clone3 returned an observed error" +
                ((" (" + errno_name + ")") if errno_name else "") +
                ", so no child was created and no acquisition occurred")

    # B -- its flags contain CLONE_PIDFD.
    if not facts.get("clone_pidfd_flag"):
        return ("pidfd_not_acquired_atomically",
                "clone3 was called without CLONE_PIDFD, so no pidfd could be "
                "returned by the syscall that created the child")

    # C -- an output location was supplied for the kernel to write into.
    if not facts.get("pidfd_output_pointer_supplied"):
        return None, ("CLONE_PIDFD was requested but no clone_args.pidfd output "
                      "location was decoded; the record cannot show where the "
                      "kernel would have written the descriptor")

    # E -- the tracer rendered the descriptor the kernel wrote back. R-2 removed
    # the closure alternative: inferring the descriptor's origin from the
    # absence of pidfd_open depended on frozen-source invariants the trace never
    # observes, and dup2, pidfd_getfd, /proc/<pid>, legacy clone(CLONE_PIDFD), a
    # second fork() child and SCM_RIGHTS all satisfied it. Direct rendering is
    # available -- upstream strace prints it via printnum_fd -- so it is now
    # required.
    written = facts.get("pidfd_from_clone3")
    if written is None:
        return None, ("the tracer did not render the pidfd clone3 wrote back, "
                      "so the descriptor's origin is not externally observed")

    child_pid = facts.get("clone3_return")

    # H -- no separate pidfd_open acquired THIS child's handle. A pidfd_open
    # aimed at some other process is not this case's concern, which is why the
    # target pid is compared rather than the mere presence of the call.
    for call in facts.get("pidfd_open_calls") or ():
        if call.get("target_pid") == child_pid:
            return ("pidfd_acquired_by_pidfd_open",
                    "the direct child's handle was acquired by pidfd_open on "
                    "its numeric pid after creation, which is exactly the "
                    "acquisition the primary mechanism rejects")

    # G -- the descriptor is correlated to the launcher's DIRECT-CHILD handle.
    # R-3: waitid(P_PIDFD, N) only correlates when the siginfo it renders names
    # the direct child; otherwise N could refer to any process.
    correlated, why = _correlated_pidfd(facts)
    if correlated is None:
        return None, why

    # F -- and it is the same descriptor clone3 yielded.
    if written != correlated:
        return None, ("the descriptor clone3 returned is not the one the "
                      "lifecycle used to reap the direct child; the "
                      "correlation is ambiguous")

    return ("pidfd_acquired_atomically",
            "clone3 created the direct child with CLONE_PIDFD and an output "
            "location, the tracer rendered the descriptor it wrote back, that "
            "descriptor is the one waitid(P_PIDFD) used to reap this exact "
            "child, and no pidfd_open acquired it (evidence form: " +
            EVIDENCE_DIRECT + ")")


def rule_rejected_acquisition(obs):
    """M5's three recorded outcomes for the REJECTED fork+pidfd_open arm.

    Not a candidate mechanism. It is recorded to evidence why
    ``clone3(CLONE_PIDFD)`` is primary rather than asserting it, and its gate
    forbids reporting a status that was never observed.
    """
    spike = obs.get("spike")
    if spike is None:
        return None, "no parseable spike receipt"
    arm = spike.get("rejected_acquisition_arm")
    if not isinstance(arm, dict):
        return None, "the receipt records no rejected-acquisition arm"
    if arm.get("attempted") is not True:
        return None, ("the rejected-acquisition arm was not run, so it has no "
                      "outcome to record")
    open_errno = arm.get("pidfd_open_errno")
    wait_errno = arm.get("waitid_errno")
    if not isinstance(open_errno, int) or not isinstance(wait_errno, int):
        return None, "the arm's errno fields are not interpretable"

    if open_errno == ESRCH:
        # The child was reaped between fork() and pidfd_open(). This is the
        # exact window clone3(CLONE_PIDFD) closes, observed rather than argued.
        return ("pidfd_open_esrch",
                "pidfd_open failed with ESRCH: the child was already reaped "
                "between fork() and the acquisition")
    if open_errno == 0:
        if wait_errno == ECHILD:
            return ("waitid_echild",
                    "the pidfd was acquired but the exit status was "
                    "unobservable: waitid returned ECHILD")
        return ("pidfd_open_succeeded",
                "the acquisition raced ahead of the reaper on this run")
    name = _errno_name(open_errno)
    return None, ("pidfd_open failed with " + (name or str(open_errno)) +
                  ", which is outside this case's frozen safe set")


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
