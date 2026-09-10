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
import journal
import observations
from frozen_cases import (
    DECISIONS,
    TRACER_REQUIREMENT,
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
    # posable, and the trial must not start while ANY case is in that state --
    # not merely while a mandatory one is.
    #
    # A conditional or recorded case that cannot be posed is still a hole in the
    # preregistered set, and the first valid trial is an immutability boundary:
    # once it starts, the frozen artefacts cannot be edited and a defect found
    # afterwards closes the trial and forces a new preregistration. Spending
    # that boundary on a run already known to be incomplete buys nothing.
    unposable = driver.unposable_cases()
    if unposable:
        by_class = {}
        for name, info in unposable.items():
            by_class.setdefault(info["class"], []).append(name)
        halts.append({
            "gate": "evidence_channels",
            "detail": ("%d case(s) have no evidence channel and cannot be "
                       "posed: %s" % (len(unposable),
                                      {k: sorted(v) for k, v in
                                       sorted(by_class.items())})),
            "evidence": unposable,
        })

    # R-4. A usable tracer is a MANDATORY pretrial requirement, not a per-case
    # condition. Eight cases now depend on the external syscall record, and the
    # definition's parent-side ptrace fallback never existed -- so a host that
    # cannot trace stops the trial here rather than degrading eight results to
    # INVALID one at a time.
    #
    # This is emphatically NOT clone3_unavailable, and it never becomes M3's
    # conditional BLOCKED cause: it is an environment failure that prevents a
    # valid trial from starting at all.
    ok, detail = observations.strace_supported(
        preflight.get("strace"), preflight.get("strace_version"))
    if not ok:
        halts.append({"gate": "tracer", "detail": detail,
                      "requirement": TRACER_REQUIREMENT,
                      "evidence": {"strace": preflight.get("strace"),
                                   "strace_version": preflight.get(
                                       "strace_version")}})
    else:
        # strace itself needs ptrace. A permissive scope is part of the same
        # requirement, and a restrictive one is the same environment failure.
        scope = preflight.get("ptrace_scope")
        if scope is not None and str(scope).strip() not in ("0", "1"):
            halts.append({
                "gate": "tracer",
                "detail": ("ptrace_scope is " + str(scope).strip() + "; strace "
                           "is present but cannot attach, so the traced cases "
                           "have no syscall record"),
                "requirement": TRACER_REQUIREMENT,
                "evidence": {"ptrace_scope": scope}})

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


TRIAL_ID = "trial-002"

PREFLIGHT_FILE = "preflight.json"
BUILD_IDENTITY_FILE = "build-identity.json"
JOURNAL_FILE = "journal.jsonl"
EVIDENCE_FILE = "evidence.json"


def _write(out_dir, name, document, sanitiser):
    """One sanitised file, through the single publication boundary."""
    path = pathlib.Path(out_dir) / name
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        evidence.publish(document, sanitiser, stream=handle)
    return path


def run_trial(build_dir, auth, out_dir):
    """Pose every frozen case once, writing every fact down as it becomes final.

    Reached only with an :class:`driver.Authorisation`. Every case in the
    membership produces a record -- absence is never an omission.

    **P-16: this function does not sanitise.** It returns the document raw with
    the sanitiser it built, and :func:`evidence.publish` applies P-14 once, at
    the boundary. Sanitising at each return site is how ``HALT_PREFLIGHT`` came
    to be published raw.

    **Trial #1's correction: nothing waits for the end.** Trial #1 posed E1
    through E5, scored them, and lost all five when it aborted in E5b's fixture
    setup, because per-case records were held in memory until one final
    document. Here the preflight, the build identity and each case's final
    status are flushed to the journal as they happen, so an abort can destroy
    at most the record being written. The final document is a SUMMARY of
    durable facts, never their only copy.
    """
    sanitiser = evidence.public_sanitiser(build=build_dir)
    out = pathlib.Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    freeze = frozen_manifest()

    with journal.Journal(out / JOURNAL_FILE, sanitiser, trial=TRIAL_ID) as jrnl:
        jrnl.trial_begin(freeze=freeze, membership=summary())

        preflight = harness.preflight()
        _write(out, PREFLIGHT_FILE, {"preflight": preflight,
                                     "manifest": summary()}, sanitiser)
        jrnl.preflight(preflight)

        halts = preflight_gates(preflight, build_dir)
        if halts:
            jrnl.trial_end(status="HALT_PREFLIGHT", halts=halts)
            return 6, {"status": "HALT_PREFLIGHT",
                       "reason": "a mandatory preflight invariant is false; no "
                                 "case was posed and LAUNCH-EXEC-01 remains "
                                 "NOT_RUN",
                       "halts": halts,
                       "preflight": preflight}, sanitiser

        # The build happens ONCE. Its identity is hashed from the files on disk
        # and made durable BEFORE the first case, because a disposable runner
        # cannot be asked afterwards which bytes it ran -- Trial #1 could not
        # answer that question at all.
        built = harness.build(build_dir)
        identity = harness.build_identity(build_dir)
        _write(out, BUILD_IDENTITY_FILE,
               {"trial": TRIAL_ID, "build": built, "artefacts": identity},
               sanitiser)
        jrnl.build_identity(identity)

        ctx = driver.TrialContext(build=build_dir, work=build_dir,
                                  preflight=preflight, freeze=freeze,
                                  sanitiser=sanitiser, build_identity=identity)

        # Still before the boundary: if an artefact changed between hashing and
        # posing, halt rather than produce evidence about unknown bytes.
        deviations = driver.verify_build_identity(ctx)
        if deviations:
            jrnl.trial_end(status="HALT_BUILD_IDENTITY", halts=deviations)
            return 6, {"status": "HALT_BUILD_IDENTITY",
                       "reason": "a built artefact does not match the identity "
                                 "hashed for this trial; no case was posed",
                       "halts": deviations,
                       "preflight": preflight}, sanitiser

        # ---------------------------------------------- the case loop
        records, cases = {}, {}
        for index, name in enumerate(MEMBERSHIP):
            # Durable BEFORE the case is posed. Trial #1 entered E5b and died
            # in its setup, and only a stack trace showed which case that was.
            jrnl.case_entered(name, index)
            plan = driver.CASE_PLANS[name]
            observation = driver.observe(plan, ctx, auth)
            record = driver.evaluate(plan, observation)
            status, reason = checker.score_case(name, record)
            # Durable the moment the frozen status is final, and once only.
            jrnl.case_completed(name, status, reason, record)
            records[name] = record
            cases[name] = {"plan": plan.as_dict(), "record": record,
                           "status": status, "reason": reason}

        report = checker.report(records)
        for name, (status, reason) in checker.score_all(records).items():
            cases[name]["status"] = status
            cases[name]["reason"] = reason

        uncontrolled = []
        if records.get("N2", {}).get("blocked") == "parent_no_new_privs_set":
            # N1 stands alone where its control arm could not be posed, and
            # saying so is the whole point of making N2 conditional rather than
            # dropping it.
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
            "build": {"targets": built, "artefacts": identity},
            "freeze": freeze,
            "uncontrolled": uncontrolled,
        })
        jrnl.trial_end(status="RUN", aggregate=report["aggregate"],
                       detail=report["detail"], counts=report["counts"])

    _write(out, EVIDENCE_FILE, document, sanitiser)
    return 0, document, sanitiser


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
    parser.add_argument("--out-dir", default="trial-output",
                        help="where the durable sanitised trial evidence is "
                             "written: the progress journal, the preflight, "
                             "the build identity and the final document")
    args = parser.parse_args(argv)

    if args.verify_freeze:
        ok, detail = verify_freeze()
        evidence.publish({"freeze_verified": ok, "detail": detail})
        return 0 if ok else 1

    if args.driver_completeness:
        # Static analysis only: it walks the plan table and never calls a
        # handler, so it is safe to run at any time, D-7 or not.
        evidence.publish({
            "completeness": driver.completeness(),
            "unposable": driver.unposable_cases(),
            "supplied_channels": sorted(driver.SPIKE_SUPPLIED_CHANNELS),
        })
        return 0

    if args.preflight_only:
        evidence.publish({
            "preflight": harness.preflight(),
            "manifest": summary(),
            "driver": driver.completeness(),
        })
        return 0

    if not args.i_have_owner_authorisation_d7:
        evidence.publish({
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
        })
        return 3

    ok, detail = verify_freeze()
    if not ok:
        evidence.publish({"status": "HALT", "reason": detail})
        return 4

    # Past this line a case can be posed. The Authorisation object is the single
    # gate every posing path checks, and it cannot be built without the flag.
    auth = driver.Authorisation(True)
    code, document, sanitiser = run_trial(args.build_dir, auth, args.out_dir)
    evidence.publish(document, sanitiser)
    return code


if __name__ == "__main__":
    sys.exit(main())
