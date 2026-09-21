"""LAUNCH-EXEC-01 durable progress journal.

Trial #1 lost real work. It posed E1 through E5, scored them, and then aborted
during E5b's fixture setup; every one of those five records existed only in the
runner's memory, waiting to be published together at the end, so the abort took
them with it. Their statuses are permanently
``UNKNOWN_FROM_PRESERVED_EVIDENCE`` -- not because the cases failed, but because
nothing wrote them down.

This module is the correction. Each fact is serialised the moment it becomes
final and is flushed to stable storage before the next one is attempted, so a
crash can only ever lose the record currently being written.

**Append-only, one JSON object per line.** A line is written whole, flushed and
``fsync``-ed. A crash can therefore truncate only the tail: every earlier line
is already durable, and a torn final line is dropped by the reader rather than
guessed at. There is no update and no delete, so a completed case status cannot
be revised -- writing one twice raises. This is a record of what happened, never
a retry mechanism.

**Everything written here is P-14 sanitised**, through the same
``evidence.Sanitiser`` and the same boundary the published document uses, because
the journal is uploaded as public evidence. That also means the M-1 fail-closed
rule covers it: if the fixed vocabulary cannot be established the write raises
rather than emitting corrupted symbolic evidence.

Nothing here executes anything. Importing this module poses no case.
"""
import os

import evidence

TRIAL_BEGIN = "trial_begin"
PREFLIGHT = "preflight"
BUILD_IDENTITY = "build_identity"
CASE_ENTERED = "case_entered"
CASE_POSE_STARTED = "case_pose_started"
CASE_COMPLETED = "case_completed"
TRIAL_END = "trial_end"

KINDS = frozenset({TRIAL_BEGIN, PREFLIGHT, BUILD_IDENTITY, CASE_ENTERED,
                   CASE_POSE_STARTED, CASE_COMPLETED, TRIAL_END})

# A case that was entered but never completed. It is deliberately NOT one of the
# frozen case statuses: entering a case says nothing about its outcome, and
# Trial #1 is exactly why the two must not be conflated.
ENTERED_NOT_COMPLETED = "CASE_ENTERED_NOT_COMPLETED"

# What an incomplete case's status IS, at review level. Frozen here so that no
# reader has to choose it after a crash, which is what Trial #1 forced.
UNKNOWN_FROM_PRESERVED_EVIDENCE = "UNKNOWN_FROM_PRESERVED_EVIDENCE"

# The two project-level trial statuses a durable journal can prove, and the
# rule that separates them. Frozen BEFORE the trial, so a crash needs no owner
# semantic decision afterwards.
TRIAL_NOT_STARTED = "TRIAL_NOT_STARTED"
TRIAL_ABORTED_AFTER_BOUNDARY = "TRIAL_ABORTED_AFTER_BOUNDARY"
TRIAL_COMPLETED = "TRIAL_COMPLETED"

AGGREGATE_NOT_DERIVABLE = "AGGREGATE_NOT_DERIVABLE_FROM_FROZEN_EVIDENCE"


class JournalError(RuntimeError):
    """A write that would break the append-only contract."""


class Journal:
    """One trial's durable progress record.

    Constructed before the first case is entered and closed when the trial ends.
    Every method returns the record it wrote, so a caller can assert on it.
    """

    def __init__(self, path, sanitiser, trial="trial-002"):
        self.path = str(path)
        self._sanitiser = sanitiser
        self._trial = trial
        self._n = 0
        self._entered = []
        self._pose_started = []
        self._completed = {}
        self._ended = False
        # Opened for APPEND, so an existing journal is never truncated by
        # reopening it. A trial that finds a non-empty journal here is a trial
        # being started twice, which the runner refuses before it gets this far.
        self._fh = open(self.path, "a", encoding="utf-8", newline="\n")

    # ------------------------------------------------------------- the write
    def _append(self, kind, payload):
        if kind not in KINDS:
            raise JournalError("unknown journal record kind: " + str(kind))
        record = dict(payload)
        record["n"] = self._n
        record["kind"] = kind
        record["trial"] = self._trial
        # Sanitise BEFORE serialising, exactly as the publication boundary
        # does, and let evidence own the only json.dumps in the experiment.
        line = evidence.serialise_line(self._sanitiser.record(record))
        self._fh.write(line)
        self._fh.flush()
        os.fsync(self._fh.fileno())
        self._n += 1
        return record

    # ------------------------------------------------------- the record kinds
    def trial_begin(self, freeze, membership):
        return self._append(TRIAL_BEGIN, {"freeze": freeze,
                                          "membership": membership})

    def preflight(self, preflight):
        return self._append(PREFLIGHT, {"preflight": preflight})

    def build_identity(self, artefacts):
        """The exact files this trial will consume, hashed before the first case.

        Durable here rather than only in the final document, because Trial #1
        proved that a crash after the boundary destroys anything held in memory
        -- and build identity is the one fact that cannot be recovered
        afterwards at all, since the runner is disposable.
        """
        return self._append(BUILD_IDENTITY, {"artefacts": artefacts})

    def case_entered(self, case, index):
        """Written BEFORE the case is posed.

        This is the record Trial #1 could not produce: it entered E5b and died
        in its fixture setup, and only a stack trace showed which case that was.
        """
        if case in self._entered:
            raise JournalError("case entered twice: " + str(case))
        self._entered.append(case)
        return self._append(CASE_ENTERED, {"case": case, "index": index})

    def case_pose_started(self, case, index):
        """Written when the mechanism is about to be invoked for this case.

        **This event is the Trial #2 immutability boundary and the moment D-7 is
        consumed.** It is written and fsynced BEFORE the launcher is started, so
        a process that dies between this record and the actual creation of the
        launcher still counts as having consumed its authorisation. That bias is
        deliberate: it can never accidentally permit a second execution, whereas
        the opposite bias could.

        A case that is BLOCKED, or that cannot be posed as written, never
        reaches here -- and its completed record says so without ever claiming
        the mechanism ran.
        """
        if case not in self._entered:
            raise JournalError("pose started for a case never entered: " + str(case))
        if case in self._pose_started:
            raise JournalError("pose started twice: " + str(case))
        if case in self._completed:
            raise JournalError("pose started after completion: " + str(case))
        self._pose_started.append(case)
        return self._append(CASE_POSE_STARTED, {"case": case, "index": index})

    def case_completed(self, case, status, reason, record):
        """Written the moment a case's frozen status is final.

        A status is assigned once. Completing a case that was never entered, or
        completing one twice, is a contract violation rather than an update.
        """
        if case not in self._entered:
            raise JournalError("case completed without being entered: " + str(case))
        if case in self._completed:
            raise JournalError("case completed twice: " + str(case))
        self._completed[case] = status
        return self._append(CASE_COMPLETED, {"case": case, "status": status,
                                             "reason": reason, "record": record})

    def trial_end(self, status, aggregate=None, detail=None, counts=None,
                  halts=None, membership=None, input_digest=None):
        """The trial ended in a state the frozen protocol recognises.

        A ``trial_end`` carrying an aggregate may be written ONLY when the
        frozen prerequisites hold: every membership case has a durable completed
        record, no case has two, and the aggregate was computed by the frozen
        checker from exactly those records. A crash before that leaves no
        trial_end, and replay never synthesises one.
        """
        if self._ended:
            raise JournalError("trial_end written twice")
        if aggregate is not None:
            missing = [c for c in (membership or []) if c not in self._completed]
            if missing:
                raise JournalError(
                    "an aggregate trial_end needs every membership case "
                    "completed; missing %d, first %s"
                    % (len(missing), missing[0]))
        self._ended = True
        return self._append(TRIAL_END, {"status": status, "aggregate": aggregate,
                                        "detail": detail, "counts": counts,
                                        "halts": halts,
                                        "aggregate_input_digest": input_digest})

    def close(self):
        if self._fh is not None:
            self._fh.close()
            self._fh = None

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()
        return False


# ------------------------------------------------------------------- reading
def read(path):
    """``(records, tail_truncated)`` from a journal file.

    A final line that does not parse is a torn write from a crash. It is
    DROPPED and reported, never repaired: half a record is not evidence, and
    guessing at its missing half is exactly the reconstruction this experiment
    forbids. Every earlier line was flushed before it was written, so it stands.
    """
    try:
        with open(path, "r", encoding="utf-8") as handle:
            raw = handle.read()
    except OSError:
        return [], False
    records, truncated = [], False
    lines = raw.splitlines(True)
    for line in lines:
        text = line.strip()
        if not text:
            continue
        parsed = evidence.parse_line(text)
        if parsed is None or not line.endswith("\n"):
            truncated = True
            break
        records.append(parsed)
    return records, truncated


def replay(records):
    """What the preserved journal proves, and nothing more.

    Deliberately makes no claim about a case that was entered and not
    completed: it names it, and leaves its status to the reader. Turning that
    into a frozen status would be the reconstruction Trial #1's review refused.
    """
    entered, pose_started, completed, order = [], [], {}, []
    freeze = membership = artefacts = preflight = end = None
    for record in records:
        kind = record.get("kind")
        case = record.get("case")
        # T2-R4. An impossible ordering is a corrupt history, not something to
        # be repaired into a plausible one. The writer refuses to create these;
        # the reader refuses to believe them.
        if end is not None and kind != TRIAL_END:
            raise JournalError("journal contains events after trial_end")
        if kind == TRIAL_BEGIN:
            freeze, membership = record.get("freeze"), record.get("membership")
        elif kind == PREFLIGHT:
            preflight = record.get("preflight")
        elif kind == BUILD_IDENTITY:
            artefacts = record.get("artefacts")
        elif kind == CASE_ENTERED:
            if case in entered:
                raise JournalError("journal enters " + str(case) + " twice")
            entered.append(case)
        elif kind == CASE_POSE_STARTED:
            if case not in entered:
                raise JournalError("journal starts posing " + str(case) +
                                   " before entering it")
            if case in pose_started:
                raise JournalError("journal starts posing " + str(case) + " twice")
            if case in completed:
                raise JournalError("journal starts posing " + str(case) +
                                   " after completing it")
            pose_started.append(case)
        elif kind == CASE_COMPLETED:
            if case not in entered:
                raise JournalError("journal completes " + str(case) +
                                   " without entering it")
            if case in completed:
                raise JournalError("journal contains two completions for " + str(case))
            completed[case] = {"status": record.get("status"),
                               "reason": record.get("reason"),
                               "record": record.get("record")}
            order.append(case)
        elif kind == TRIAL_END:
            if end is not None:
                raise JournalError("journal contains two trial_end records")
            end = record
    incomplete = [c for c in entered if c not in completed]
    return {
        "freeze": freeze,
        "membership": membership,
        "build_identity": artefacts,
        "preflight": preflight,
        "entered": entered,
        "pose_started": pose_started,
        "completed": completed,
        "completion_order": order,
        "entered_not_completed": incomplete,
        "incomplete_case_status": {c: UNKNOWN_FROM_PRESERVED_EVIDENCE
                                   for c in incomplete},
        "last_entered": entered[-1] if entered else None,
        "last_pose_started": pose_started[-1] if pose_started else None,
        "last_completed": order[-1] if order else None,
        "boundary_crossed": bool(pose_started),
        "trial_end": end,
    }


def project_status(replayed):
    """The frozen Trial #2 reading of a journal. No choice is left to a reader.

    The rules are fixed BEFORE the trial precisely because Trial #1 had to
    invent them afterwards, under the pressure of a result already in hand.
    """
    end = replayed.get("trial_end")
    crossed = bool(replayed.get("pose_started"))
    if end is not None and end.get("aggregate") is not None:
        return {"trial_status": TRIAL_COMPLETED,
                "d7_consumed": True,
                "aggregate": end.get("aggregate"),
                "detail": "every membership case has a durable completed record "
                          "and the frozen checker produced this aggregate"}
    if not crossed:
        return {"trial_status": TRIAL_NOT_STARTED,
                "d7_consumed": False,
                "aggregate": None,
                "detail": "no case_pose_started record exists, so the mechanism "
                          "was never invoked and the boundary was not crossed"}
    return {"trial_status": TRIAL_ABORTED_AFTER_BOUNDARY,
            "d7_consumed": True,
            "aggregate": AGGREGATE_NOT_DERIVABLE,
            "detail": "at least one case_pose_started is durable and no valid "
                      "trial_end exists; completed cases keep exactly their "
                      "recorded status and incomplete cases have none"}


def recovered_records(replayed):
    """The per-case records a crash left behind, ready for the frozen checker.

    Only cases the journal says COMPLETED are returned. A case that was entered
    and not completed is absent, and an absent case is not silently scored --
    the caller decides, and the frozen protocol says a trial that ended this way
    is closed rather than aggregated.
    """
    return {case: entry["record"] for case, entry in replayed["completed"].items()
            if entry.get("record") is not None}
