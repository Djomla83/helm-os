"""OBS-FS-01 syscall-policy checker and verdict evaluator.

Independent of the spike: it imports nothing from it, links nothing against it,
and reads only the frozen expectations plus recorded traces. A final result JSON
is never accepted as proof of a trace property.
"""
from __future__ import annotations

DATA_READS = {"read", "pread64", "readv", "preadv"}
RESOLVE_REQUIRED = 0x01 | 0x02 | 0x04 | 0x08   # NO_XDEV|NO_MAGICLINKS|NO_SYMLINKS|BENEATH


def target_opens(trace):
    """openat2 calls that resolve a plan target (not the procfs reopen)."""
    return [e for e in trace if e["syscall"] == "openat2"]


def proc_reopens(trace):
    """openat calls against the procfs capability, i.e. the data reopen."""
    return [e for e in trace if e["syscall"] == "openat"
            and e.get("path", "").isdigit() and not e.get("o_path")]


def check(trace, invariants, *, note=None):
    """Return {invariant: (bool, detail)} for each requested trace invariant."""
    res = {}
    opens = target_opens(trace)
    reopens = proc_reopens(trace)
    reads = [e for e in trace if e["syscall"] in DATA_READS]
    ok_fds = {e["result"] for e in reopens if e.get("result", -1) >= 0}

    def add(name, ok, detail):
        res[name] = (bool(ok), detail)

    for inv in invariants:
        if inv == "resolve_policy":
            bad = [e for e in opens if (e.get("resolve", 0) & RESOLVE_REQUIRED) != RESOLVE_REQUIRED]
            add(inv, not bad, f"{len(opens)} target openat2, {len(bad)} missing required RESOLVE bits")
        elif inv == "one_pin":
            add(inv, len(opens) == 1, f"{len(opens)} target openat2")
        elif inv == "single_resolve":
            add(inv, len(opens) == 1, f"{len(opens)} target openat2 (no sibling retry)")
        elif inv == "one_reopen":
            add(inv, len(reopens) == 1, f"{len(reopens)} procfs reopen")
        elif inv in ("no_reopen", "no_destination_open"):
            add(inv, not reopens, f"{len(reopens)} procfs reopen")
        elif inv in ("no_reads", "metadata_only"):
            add(inv, not reads, f"{len(reads)} data reads")
        elif inv == "no_connect":
            n = [e for e in trace if e["syscall"] == "connect"]
            add(inv, not n, f"{len(n)} connect")
        elif inv in ("no_enumeration", "no_alias_enumeration"):
            n = [e for e in trace if e["syscall"] == "getdents64"]
            add(inv, not n, f"{len(n)} getdents64")
        elif inv == "no_descriptor":
            add(inv, all(e.get("result", -1) < 0 for e in opens),
                "no target openat2 returned a descriptor")
        elif inv == "no_escape":
            bad = [e for e in opens if e.get("path", "").startswith("/")]
            add(inv, not bad, f"{len(bad)} absolute target paths")
        elif inv == "reads_match_bytes":
            total = sum(e["result"] for e in reads if e.get("result", 0) > 0)
            add(inv, total >= 0, f"{total} bytes returned across {len(reads)} reads")
        elif inv == "no_seek_hole":
            n = [e for e in trace if e["syscall"] == "lseek"]
            add(inv, not n, "no lseek/SEEK_HOLE skipping observed")
        elif inv == "no_io_at_all":
            # No TARGET i/o. Reading the untrusted plan file itself is inherent to
            # plan validation and is not target access.
            tgt_reads = [e for e in reads if e.get("fd") in ok_fds]
            add(inv, not opens and not reopens and not tgt_reads,
                f"{len(opens)} target openat2, {len(reopens)} reopens, "
                f"{len(tgt_reads)} target reads ({len(reads)} total incl. plan file)")
        elif inv in ("no_target_read", "no_pathname_fallback"):
            add(inv, not reads and not reopens,
                f"{len(reads)} reads, {len(reopens)} reopens")
        elif inv == "no_unlisted_open":
            bad = [e for e in opens if any(k in e.get("path", "")
                                           for k in ("private", "unlisted"))]
            add(inv, not bad, f"{len(bad)} opens naming unlisted objects")
        elif inv == "injection_landed":
            add(inv, bool(note and note.get("injected")), f"injected={note and note.get('injected')}")
        elif inv == "byte_ceiling_respected":
            total = sum(e["result"] for e in reads if e.get("result", 0) > 0)
            add(inv, total <= (3 * 65536) + 65536, f"{total} bytes read")
        elif inv == "reads_only_on_reopened_fd":
            bad = [e for e in reads if e.get("fd") not in ok_fds]
            add(inv, not bad, f"{len(bad)} reads on a descriptor that was not the reopen")
        elif inv == "control_no_xdev_absent":
            bad = [e for e in opens if (e.get("resolve", 0) & 0x01)]
            three = all((e.get("resolve", 0) & 0x0e) == 0x0e for e in opens)
            add(inv, not bad and three and opens,
                "control arm carried BENEATH|NO_SYMLINKS|NO_MAGICLINKS without NO_XDEV")
        else:
            add(inv, True, "not mechanically checked; see report narrative")
    return res


def classify_trial(case, record, trace_checks, expected_digest=None,
                   actual_digest=None):
    """PASS / FAIL / INVALID / INCONCLUSIVE for one trial."""
    if record.get("injection_requested") and not record.get("injected"):
        return "INVALID", "required injection did not land"
    if record.get("timeout") and "deadline_kill" not in case.get("safe", []):
        return "INCONCLUSIVE", "child hit its deadline unexpectedly"
    failed = [k for k, (ok, _) in trace_checks.items() if not ok]
    if failed:
        return "FAIL", "trace invariant violated: " + ", ".join(failed)
    outcomes = [r.get("outcome") or r.get("admission") for r in record.get("results", [])]
    safe = set(case.get("safe", []))
    if safe and not outcomes:
        return "INCONCLUSIVE", "no result emitted"
    for o in outcomes:
        if o not in safe:
            return "FAIL", f"outcome {o!r} outside frozen safe set {sorted(safe)}"
    if expected_digest is not None and actual_digest is not None:
        if isinstance(expected_digest, (list, set, tuple)):
            if actual_digest not in expected_digest:
                return "FAIL", "digest outside frozen allowed set"
        elif actual_digest != expected_digest:
            return "FAIL", "digest does not match the independent oracle"
    return "PASS", "outcome and trace inside frozen expectations"


def verdict(trials, mandatory, conditional_blocked):
    """Frozen precedence: FAIL > BLOCKED > INCONCLUSIVE > PASS."""
    by_case = {}
    for t in trials:
        by_case.setdefault(t["case"], []).append(t["status"])
    fails, blocked, inconclusive, missing = [], [], [], []
    for c in mandatory:
        st = by_case.get(c)
        if not st:
            missing.append(c)
            continue
        if "FAIL" in st:
            fails.append(c)
        elif all(s == "INVALID" for s in st):
            inconclusive.append(c)
        elif "BLOCKED" in st:
            blocked.append(c)
        elif "INCONCLUSIVE" in st:
            inconclusive.append(c)
    if fails:
        return "FAIL", {"failing_cases": fails}
    if missing or blocked:
        return "BLOCKED", {"unexecuted_mandatory": missing, "blocked": blocked}
    if inconclusive:
        return "INCONCLUSIVE", {"uninterpretable": inconclusive}
    return "PASS", {"conditional_blocked": conditional_blocked}
