"""LAUNCH-EXEC-01 runner: emits one sanitised report.

Disposable spike tooling.

>>> NOT_RUN <<<
This runner has never been executed against the preregistered case set. It
refuses to run unless BOTH a frozen source-hash manifest exists AND the operator
passes --i-have-owner-authorisation-d7, because D-7 (execution authorisation) is
NOT granted: the definition is frozen for review, not for execution.

The case-posing driver now EXISTS -- it lives in ``driver.py``, with the P-12
observation-to-token mapping in ``observations.py`` and the P-14 sanitiser in
``evidence.py``. Its existence changes nothing about authorisation: the flag is
still required, the default invocation still refuses, and this module still
poses nothing on import.

**Preflight gates run before the first case, and a failed gate HALTS.** A trial
that is known in advance to be inconclusive must not be started at all, because
starting it would consume the first-valid-trial immutability boundary for a
result nobody can use. That is why an unposable MANDATORY case is a halt and not
a per-case INVALID.

Importing this module poses no case and executes no helper.
"""
import argparse
import json
import pathlib
import sys

import checker
import driver
import evidence
import harness
from frozen_cases import (
    DECISIONS,
    MEMBERSHIP,
    POST_EXIT_DRAIN_MS,
    SPAWN_CONFIRM_TIMEOUT_MS,
    summary,
)

HERE = pathlib.Path(__file__).resolve().parent
SOURCE_HASHES = HERE / "SOURCE-HASHES.json"

# Kept for compatibility with the names the independent review cited by name in
# P-14. The implementation now lives in evidence.py, which sanitises VALUES as
# well as names -- see that module for why the original pair was insufficient.
DECLARED_ENV_NAMES = set(evidence.DECLARED_ENV_NAMES)


def sanitise_env_name(name):
    return evidence.Sanitiser().env_name(name)


def sanitise_path(text, work, home):
    return evidence.Sanitiser(work=work, home=home).text(text)


def frozen_manifest():
    if not SOURCE_HASHES.exists():
        return None
    return json.loads(SOURCE_HASHES.read_text(encoding="utf-8"))


def verify_freeze():
    """The source freeze must exist and match before any trial may occur."""
    import hashlib
    manifest = frozen_manifest()
    if manifest is None:
        return False, "SOURCE-HASHES.json does not exist; no freeze has occurred"
    drift = []
    for name, expected in sorted(manifest.get("sha256", {}).items()):
        path = HERE / name
        if not path.exists():
            drift.append(f"{name}: missing")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            drift.append(f"{name}: {actual} != {expected}")
    if drift:
        return False, "frozen sources drifted:\n  " + "\n  ".join(drift)
    return True, "frozen sources match the manifest"


def preflight_gates(preflight, build_dir):
    """Every mandatory invariant that must hold BEFORE the first case is posed.

    Returns a list of halt reasons; empty means the trial may proceed. Each entry
    says what was false and why it stops the trial rather than blocking one case.
    """
    halts = []

    complete = driver.completeness()
    if not complete["complete"]:
        halts.append({
            "gate": "driver_completeness",
            "detail": "the driver does not cover the frozen membership exactly",
            "evidence": complete,
        })

    # A case whose declared evidence channel the mechanism cannot supply is not
    # posable. One that is MANDATORY makes the aggregate INCONCLUSIVE no matter
    # what else happens, so the trial must not start.
    unposable = driver.unposable_cases()
    mandatory_unposable = {name: info for name, info in unposable.items()
                           if info["class"] == "mandatory"}
    if mandatory_unposable:
        halts.append({
            "gate": "evidence_channels",
            "detail": ("%d mandatory case(s) have no evidence channel; the "
                       "aggregate would be MECHANISM_INCONCLUSIVE before the "
                       "first case is posed" % len(mandatory_unposable)),
            "evidence": mandatory_unposable,
        })

    # Static linking is a precondition, not a preference: a complete trace is
    # what makes "no other descriptor was present" evidence rather than
    # filtering. A dynamic helper must never be substituted.
    gate = harness.static_link_gate(build_dir)
    if not gate.get("ok"):
        halts.append({"gate": "static_link", "detail": gate.get("reason"),
                      "evidence": gate})

    # clone3 unavailability disables the whole mechanism, so it is a preflight
    # gate like the static-link gate rather than a single case result.
    if (preflight.get("clone3") or {}).get("available") is False:
        halts.append({
            "gate": "clone3",
            "detail": "clone3 is unavailable, which disables the mechanism",
            "evidence": preflight.get("clone3"),
        })
    return halts


def run_trial(build_dir, auth):
    """Pose every frozen case once and emit one sanitised evidence document.

    Reached only with an :class:`driver.Authorisation`. Every case in the
    membership produces a record -- absence is never an omission.
    """
    preflight = harness.preflight()
    halts = preflight_gates(preflight, build_dir)
    if halts:
        return 6, {"status": "HALT_PREFLIGHT",
                   "reason": "a mandatory preflight invariant is false; no case "
                             "was posed and LAUNCH-EXEC-01 remains NOT_RUN",
                   "halts": halts,
                   "preflight": preflight}

    built = harness.build(build_dir)
    sanitiser = evidence.Sanitiser(
        work=str(pathlib.Path(build_dir).resolve()),
        home=str(pathlib.Path.home()), build=str(pathlib.Path(build_dir).resolve()))
    ctx = driver.TrialContext(build=build_dir, work=build_dir,
                              preflight=preflight, freeze=frozen_manifest(),
                              sanitiser=sanitiser)

    records, cases = {}, {}
    for name in MEMBERSHIP:
        plan = driver.CASE_PLANS[name]
        observation = driver.observe(plan, ctx, auth)
        record = driver.evaluate(plan, observation)
        records[name] = record
        cases[name] = {"plan": plan.as_dict(), "record": record}

    report = checker.report(records)
    for name, (status, reason) in checker.score_all(records).items():
        cases[name]["status"] = status
        cases[name]["reason"] = reason

    uncontrolled = []
    if records.get("N2", {}).get("blocked") == "parent_no_new_privs_set":
        # N1 stands alone where its control arm could not be posed, and saying so
        # is the whole point of making N2 conditional rather than dropping it.
        uncontrolled.append(
            "N1 is UNCONTROLLED: its control arm N2 was BLOCKED because the "
            "launcher's parent already had no_new_privs set")

    document = evidence.evidence_document({
        "status": "RUN",
        "aggregate": report["aggregate"],
        "detail": report["detail"],
        "counts": report["counts"],
        "preflight": preflight,
        "membership": summary(),
        "cases": cases,
        "build": built,
        "freeze": frozen_manifest(),
        "uncontrolled": uncontrolled,
    })
    return 0, sanitiser.record(document)


def main(argv=None):
    parser = argparse.ArgumentParser(description="LAUNCH-EXEC-01 runner")
    parser.add_argument("--preflight-only", action="store_true",
                        help="record the environment inventory and stop")
    parser.add_argument("--verify-freeze", action="store_true",
                        help="check the frozen sources against SOURCE-HASHES.json")
    parser.add_argument("--driver-completeness", action="store_true",
                        help="report driver coverage of the frozen membership "
                             "and the cases the mechanism cannot pose; poses "
                             "nothing")
    parser.add_argument("--i-have-owner-authorisation-d7", action="store_true",
                        help="required to pose any preregistered case; D-7 is "
                             "NOT currently granted")
    parser.add_argument("--build-dir", default="target/launch-exec-01")
    args = parser.parse_args(argv)

    if args.verify_freeze:
        ok, detail = verify_freeze()
        print(json.dumps({"freeze_verified": ok, "detail": detail}, indent=2))
        return 0 if ok else 1

    if args.driver_completeness:
        # Static analysis only: it walks the plan table and never calls a
        # handler, so it is safe to run at any time, D-7 or not.
        print(json.dumps({
            "completeness": driver.completeness(),
            "unposable": driver.unposable_cases(),
            "supplied_channels": sorted(driver.SPIKE_SUPPLIED_CHANNELS),
        }, indent=2, sort_keys=True))
        return 0

    if args.preflight_only:
        print(json.dumps({
            "preflight": harness.preflight(),
            "manifest": summary(),
            "driver": driver.completeness(),
        }, indent=2, sort_keys=True, default=str))
        return 0

    if not args.i_have_owner_authorisation_d7:
        print(json.dumps({
            "status": "NOT_RUN",
            "reason": "D-7 (execution authorisation) is not granted. This "
                      "definition is frozen for independent pre-trial review, "
                      "not for execution. No case was posed.",
            "decisions": DECISIONS,
            "membership": len(MEMBERSHIP),
            "bounds": {
                "SPAWN_CONFIRM_TIMEOUT_MS": SPAWN_CONFIRM_TIMEOUT_MS,
                "POST_EXIT_DRAIN_MS": POST_EXIT_DRAIN_MS,
            },
        }, indent=2, sort_keys=True))
        return 3

    ok, detail = verify_freeze()
    if not ok:
        print(json.dumps({"status": "HALT", "reason": detail}, indent=2))
        return 4

    # Past this line a case can be posed. The Authorisation object is the single
    # gate every posing path checks, and it cannot be built without the flag.
    auth = driver.Authorisation(True)
    code, document = run_trial(args.build_dir, auth)
    sys.stdout.write(evidence.serialise(document))
    return code


if __name__ == "__main__":
    sys.exit(main())
