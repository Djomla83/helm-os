"""LAUNCH-EXEC-01 verdict evaluator.

Links nothing from the spike. It consumes recorded trial dicts and the frozen
manifest, and emits per-case statuses and one aggregate verdict. It never
executes the mechanism, and it never repairs a record.

Per-case status is one of PASS / FAIL / INVALID / BLOCKED, assigned once and
never revised. A case in the frozen membership with no recorded status is not
silently ignored: it becomes INVALID, because absence is not a recorded
environmental cause.
"""
from frozen_cases import (
    BLOCK_REASONS,
    BY_NAME,
    CONDITIONAL,
    CONDITIONAL_CASES,
    DOCUMENTATION_GATES,
    MANDATORY,
    MANDATORY_CASES,
    MEMBERSHIP,
    RECORDED,
    RECORDED_CASES,
)

PASS, FAIL, INVALID, BLOCKED = "PASS", "FAIL", "INVALID", "BLOCKED"

MEMBERSHIP_READY = len(MEMBERSHIP) > 0

REJECTED = "MECHANISM_REJECTED"
INCONCLUSIVE = "MECHANISM_INCONCLUSIVE"
ACCEPTED = "MECHANISM_ACCEPTED"


def score_case(name, record):
    """Status and reason for one case from one recorded trial.

    ``record`` is a dict produced by the runner. It is data, never code, and no
    field of it is trusted to name its own status.
    """
    spec = BY_NAME[name]

    if record is None:
        return INVALID, "no status was recorded for a case in the frozen membership"

    # Invalidity first: a case that could not be POSED as written was never a
    # test of the mechanism, whatever it appears to have produced.
    if record.get("not_posed"):
        return INVALID, "case could not be posed as written: " + str(
            record.get("not_posed"))
    if record.get("traced") and not spec["traced"]:
        return INVALID, ("observed under a tracer while declared traced:false; "
                         "ptrace reports a tracee's exit to the tracer before "
                         "the real parent, and the launcher is the real parent")
    if spec["traced"] and record.get("trace") is None and not record.get("blocked"):
        return INVALID, "a traced case produced no syscall record"

    # Environment blocks, which only a conditional case may absorb, and only
    # with a cause named in the frozen reason set.
    blocked = record.get("blocked")
    if blocked:
        if blocked not in BLOCK_REASONS:
            return INVALID, f"BLOCKED with an unfrozen cause {blocked!r}"
        return BLOCKED, BLOCK_REASONS[blocked]

    # A launcher-side non-return is never a recordable outcome, in any class.
    if record.get("launch_returned") is False:
        return FAIL, "launch() did not return within its declared total bound"
    bound = record.get("total_bound_ms")
    elapsed = record.get("elapsed_ms")
    if bound is not None and elapsed is not None and elapsed > bound:
        return FAIL, f"launch() returned after {elapsed} ms, bound {bound} ms"

    outcome = record.get("outcome")
    if outcome is None:
        return INVALID, "no outcome recorded"

    if spec["predict"] is not None:
        if outcome != spec["predict"]:
            return FAIL, (f"outcome {outcome!r} is not the single frozen "
                          f"prediction {spec['predict']!r}")
    else:
        if outcome not in spec["safe"]:
            return FAIL, (f"outcome {outcome!r} outside the frozen safe set "
                          f"{sorted(spec['safe'])}")

    # Gated sub-assertions on recorded cases. Their outcome is not predicted;
    # these are the parts that are nevertheless never acceptable to fail.
    for gate in spec["gates"]:
        if record.get("gates", {}).get(gate) is not True:
            return FAIL, f"gated sub-assertion {gate!r} did not hold"

    # Documentation gates: a case cannot be PASSed by observing the expected
    # kernel behaviour while a document still presents the pre-execution
    # measurement as the identity of the executed body.
    if name in DOCUMENTATION_GATES:
        if record.get("documentation_gate") is not True:
            return FAIL, ("kernel behaviour matched, but a document or receipt "
                          "field still presents the pre-execution measurement "
                          "as the executable body that ran")

    if record.get("asserted_unobserved_fact"):
        return FAIL, ("the receipt asserted a temporal or causal fact the "
                      "launcher did not observe: " +
                      str(record.get("asserted_unobserved_fact")))

    return PASS, "outcome and every gated invariant inside frozen expectations"


def score_all(records):
    """Map every frozen case to exactly one status. Missing is never dropped."""
    return {name: score_case(name, records.get(name)) for name in MEMBERSHIP}


def verdict(statuses):
    """Ordered, total, disjoint aggregate precedence.

    Evaluation stops at the first rule that fires:

      1 REJECTED     any mandatory or conditional case FAIL (a conditional case
                     that was posed is a real test), including every
                     instant-rejection case and every launcher non-return.
      2 INCONCLUSIVE any mandatory case INVALID or BLOCKED, or any conditional
                     or recorded case INVALID. A conditional case BLOCKED with a
                     recorded cause is expected and does NOT fire this rule --
                     that is what "conditional" means, and a rule that treated
                     it as inconclusive would make the class meaningless.
      3 ACCEPTED     only otherwise.
    """
    def st(name):
        return statuses[name][0]

    fails = [n for n in MANDATORY_CASES + CONDITIONAL_CASES + RECORDED_CASES
             if st(n) == FAIL]
    if fails:
        return REJECTED, {"failing_cases": fails}

    unposed = [n for n in MANDATORY_CASES if st(n) in (INVALID, BLOCKED)]
    unposed += [n for n in CONDITIONAL_CASES + RECORDED_CASES if st(n) == INVALID]
    if unposed:
        return INCONCLUSIVE, {"unposed_cases": sorted(set(unposed))}

    blocked = [n for n in CONDITIONAL_CASES if st(n) == BLOCKED]
    return ACCEPTED, {"conditional_blocked_with_cause": blocked}


def report(records):
    statuses = score_all(records)
    aggregate, detail = verdict(statuses)
    counts = {}
    for name in MEMBERSHIP:
        counts[statuses[name][0]] = counts.get(statuses[name][0], 0) + 1
    return {
        "aggregate": aggregate,
        "detail": detail,
        "counts": counts,
        "statuses": {n: {"status": s, "reason": r}
                     for n, (s, r) in statuses.items()},
    }
