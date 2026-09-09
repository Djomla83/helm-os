"""LAUNCH-EXEC-01 runner: emits one sanitised report.

Disposable spike tooling.

>>> NOT_RUN <<<
This runner has never been executed against the preregistered case set. It
refuses to run unless BOTH a frozen source-hash manifest exists AND the operator
passes --i-have-owner-authorisation-d7, because D-7 (execution authorisation) is
NOT granted: the definition is frozen for review, not for execution.

Importing this module poses no case and executes no helper.
"""
import argparse
import json
import pathlib
import sys

import checker
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

# Names the report may reproduce verbatim. Anything else observed in a child is
# reported as <UNDECLARED:xxxxxxxx> and its VALUE is never reproduced, so a V1
# leak is detectable without republishing what leaked -- on a hosted runner the
# inherited environment includes ACTIONS_RUNTIME_TOKEN and
# ACTIONS_ID_TOKEN_REQUEST_TOKEN.
DECLARED_ENV_NAMES = {"HELM_LEAK_CANARY", "LD_LIBRARY_PATH", "PATH", "HOME"}


def sanitise_env_name(name):
    import hashlib
    if name in DECLARED_ENV_NAMES:
        return name
    return "<UNDECLARED:" + hashlib.sha256(name.encode()).hexdigest()[:8] + ">"


def sanitise_path(text, work, home):
    if not isinstance(text, str):
        return text
    if work:
        text = text.replace(work, "<WORK>")
    if home:
        text = text.replace(home, "<HOME>")
    return text


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


def main(argv=None):
    parser = argparse.ArgumentParser(description="LAUNCH-EXEC-01 runner")
    parser.add_argument("--preflight-only", action="store_true",
                        help="record the environment inventory and stop")
    parser.add_argument("--verify-freeze", action="store_true",
                        help="check the frozen sources against SOURCE-HASHES.json")
    parser.add_argument("--i-have-owner-authorisation-d7", action="store_true",
                        help="required to pose any preregistered case; D-7 is "
                             "NOT currently granted")
    parser.add_argument("--build-dir", default="target/launch-exec-01")
    args = parser.parse_args(argv)

    if args.verify_freeze:
        ok, detail = verify_freeze()
        print(json.dumps({"freeze_verified": ok, "detail": detail}, indent=2))
        return 0 if ok else 1

    if args.preflight_only:
        print(json.dumps({
            "preflight": harness.preflight(),
            "manifest": summary(),
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

    # Reaching here would pose the frozen cases. Deliberately unimplemented in
    # this commit: the case-posing driver is written only once D-7 is granted,
    # so that no path in this repository can pose a case by accident.
    print(json.dumps({
        "status": "HALT",
        "reason": "the case-posing driver is intentionally not implemented "
                  "until D-7 is granted; freeze verified but nothing was run",
        "checker_ready": bool(checker.MEMBERSHIP_READY),
    }, indent=2))
    return 5


if __name__ == "__main__":
    sys.exit(main())
