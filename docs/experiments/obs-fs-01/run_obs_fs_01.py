"""OBS-FS-01 execution runner.

Executes the frozen case set against the disposable spike, collects complete
syscall traces, scores every trial with the plan-independent checker, and emits
one sanitized JSON report. Runs unprivileged; mutates only its own fixtures.

Usage: python3 run_obs_fs_01.py --spike ./spike --work DIR --out report.json
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

import checker
import frozen_cases as F
import harness
import oracles

EXPECT = oracles.expectations()


def sha_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def dump_digest(prefix, target):
    p = Path(f"{prefix}{target}.bin")
    return sha_file(p) if p.exists() else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--spike", required=True)
    ap.add_argument("--work", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    spike = str(Path(args.spike).resolve())
    work = Path(args.work).resolve()
    if work.exists():
        shutil.rmtree(work)
    root = work / "root"
    dumps = work / "dumps"
    dumps.mkdir(parents=True, exist_ok=True)
    built = harness.build_fixtures(root, oracles.materialise, F.OVERLIMIT_APPARENT)
    prefix = str(dumps) + "/"

    trials = []

    def record(case_name, rec, invariants, safe, expected=None, actual=None,
               extra=None):
        # The control arm deliberately omits RESOLVE_NO_XDEV, so it is scored on
        # the complementary invariant instead of the standard resolve policy.
        if case_name == "xdev_control_without_flag":
            extra_inv = ["control_no_xdev_absent"]
        else:
            extra_inv = ["resolve_policy"]
        checks = checker.check(rec["trace"], invariants + extra_inv, note=rec)
        status, why = checker.classify_trial({"safe": safe}, rec, checks,
                                             expected, actual)
        trials.append({
            "case": case_name, "status": status, "why": why,
            "results": rec["results"], "injected": rec["injected"],
            "injection_requested": rec["injection_requested"],
            "timeout": rec["timeout"], "exit_code": rec["exit_code"],
            "seconds": rec["seconds"],
            "trace_checks": {k: {"ok": v[0], "detail": v[1]} for k, v in checks.items()},
            "trace_summary": summarise(rec["trace"]),
            **(extra or {}),
        })
        return trials[-1]

    def summarise(trace):
        return [
            {k: v for k, v in e.items()
             if k in ("syscall", "path", "resolve_names", "o_path", "fd",
                      "result", "count", "injected_here")}
            for e in trace
        ]

    # ---------------------------------------------------------- static cases
    for c in F.STATIC_CASES:
        rel = c["target"].replace(".", "/")
        argv = [spike, "--root", str(root), "--rel", rel, "--op", c["op"],
                "--id", c["target"].split(".")[0], "--dump-prefix", prefix]
        rec = harness.run_traced(argv, timeout=15)
        exp = act = None
        if c.get("digest"):
            exp = EXPECT[c["digest"]]
            act = dump_digest(prefix, c["target"].split(".")[0])
        record(c["case"], rec, c.get("trace", []), c["safe"], exp, act,
               extra={"expected_digest": exp, "actual_digest": act,
                      "expected_stage": c.get("stage"),
                      "expected_kind": c.get("kind")})

    # ----------------------------------------------------------- batch cases
    for c in F.BATCH_CASES:
        planp = work / f"plan-{c['case']}.txt"
        planp.write_text("".join(f"target {tid} {op} {tid}\n" for tid, op, _ in c["plan"]))
        argv = [spike, "--root", str(root), "--plan", str(planp),
                "--dump-prefix", prefix]
        if c.get("budget"):
            argv += ["--budget", str(c["budget"])]
        rec = harness.run_traced(argv, timeout=20)
        safe = sorted({o for v in c["safe_per_target"].values() for o in v})
        t = record(c["case"], rec, c.get("trace", []), safe)
        t["per_target"] = {r.get("target"): r.get("outcome") for r in rec["results"]}
        t["expected_per_target"] = c["safe_per_target"]
        for r in rec["results"]:
            allowed = c["safe_per_target"].get(r.get("target"))
            if allowed and r.get("outcome") not in allowed and t["status"] == "PASS":
                t["status"], t["why"] = "FAIL", f"{r.get('target')} outcome outside frozen set"
        if len(rec["results"]) != len(c["plan"]) and t["status"] == "PASS":
            t["status"], t["why"] = "FAIL", "not every declared target has an outcome"

    # ------------------------------------------------------- admission cases
    for c in F.ADMISSION_CASES:
        if c["case"] == "descriptor_shaped_plan_field":
            planp = work / "plan-bad.txt"
            planp.write_text(c["plan_text"])
            argv = [spike, "--root", str(root), "--plan", str(planp)]
        else:
            if c["proc"] == "FAKE":
                fake = work / "fakeproc"
                fake.mkdir(exist_ok=True)
                (fake / "3").write_bytes(b"decoy\n")
                proc = str(fake)
            elif c["proc"] == "MISSING":
                proc = str(work / "no-such-proc-dir")
            else:  # FOREIGN: another live process's descriptor directory
                proc = "/proc/1/fd"
            argv = [spike, "--root", str(root), "--rel", "plain", "--op", "file",
                    "--proc", proc]
        rec = harness.run_traced(argv, timeout=15)
        record(c["case"], rec, c.get("trace", []), c["safe"])

    # -------------------------------------------------------- D1..D3 arm
    for c in F.DIRECT_CASES:
        argv = [spike, "--root", str(root), "--rel", c["target"], "--op", "file"]
        if c["case"].startswith("D1") or c["case"].startswith("D2"):
            argv.append("--direct")
            if c.get("nonblock"):
                argv.append("--nonblock")
        rec = harness.run_traced(argv, timeout=8)
        safe = list(c["safe"])
        if c["case"].startswith("D1"):
            # Blocking is demonstrated by the deadline, not by a returned code.
            status_extra = {"blocked_until_deadline": rec["timeout"]}
            checks = checker.check(rec["trace"], ["resolve_policy"], note=rec)
            trials.append({
                "case": c["case"],
                "status": "PASS" if rec["timeout"] else "FAIL",
                "why": ("direct O_RDONLY on a writerless FIFO blocked until the "
                        "supervisor deadline, as predicted")
                       if rec["timeout"] else
                       "direct open did not block; the frozen prediction is falsified",
                "results": rec["results"], "injected": rec["injected"],
                "injection_requested": False, "timeout": rec["timeout"],
                "exit_code": rec["exit_code"], "seconds": rec["seconds"],
                "trace_checks": {k: {"ok": v[0], "detail": v[1]} for k, v in checks.items()},
                "trace_summary": summarise(rec["trace"]), **status_extra})
        else:
            record(c["case"], rec, c.get("trace", []), safe,
                   extra={"expected_kind": c.get("kind")})

    # ------------------------------------------------------------ mount arms
    xdev_root = Path("/run/user") if Path("/run/user/1001").exists() else None
    if xdev_root:
        rec = harness.run_traced([spike, "--root", str(xdev_root), "--rel", "1001",
                                  "--op", "dir"], timeout=10)
        record("xdev_fallback_existing_mount", rec,
               ["no_descriptor", "no_reads"], ["mount_crossing"])
        rec = harness.run_traced([spike, "--root", str(xdev_root), "--rel", "1001",
                                  "--op", "dir", "--allow-xdev"], timeout=10)
        record("xdev_control_without_flag", rec, [], ["observed_directory"])
    else:
        trials.append({"case": "xdev_fallback_existing_mount", "status": "BLOCKED",
                       "why": "no reachable existing mount boundary"})
        trials.append({"case": "xdev_control_without_flag", "status": "BLOCKED",
                       "why": "no reachable existing mount boundary"})

    # conditional bind-mount case
    probe = subprocess.run(["unshare", "-Urm", "true"], capture_output=True, timeout=20)
    bind_blocked = probe.returncode != 0
    trials.append({
        "case": "bind_mount_descendant_namespace",
        "status": "BLOCKED" if bind_blocked else "PENDING",
        "why": ("unprivileged user+mount namespace unavailable: "
                + probe.stderr.decode('utf-8', 'replace').strip())
               if bind_blocked else "namespace unexpectedly available",
        "conditional": True,
        "residual_unverified_claim": F.MOUNT_CASES[2]["residual"]})

    # ------------------------------------------------------------ race cases
    for c in F.RACE_CASES:
        run_race(spike, root, built["outside"], prefix, c, record, work)

    harness.cleanup_permissions(root)
    built["socket"].close()

    mandatory = [c for c in F.MANDATORY_CASES]
    v, detail = checker.verdict(trials, mandatory, ["bind_mount_descendant_namespace"]
                               if bind_blocked else [])
    report = {
        "experiment": "OBS-FS-01",
        "amendment_1": F.AMENDMENT,
        "original_freeze": F.ORIGINAL_FREEZE,
        "preflight_halt": F.PREFLIGHT_HALT,
        "verdict": v, "verdict_detail": detail,
        "counts": tally(trials),
        "mandatory_case_count": len(mandatory),
        "trials": trials,
        "expected_digests": EXPECT,
        "scope": ("Synthetic OBS-FS-01 fixtures only. No A0 access, no Wine or "
                  "7-Zip, no privilege, no product code."),
    }
    Path(args.out).write_text(json.dumps(report, indent=2, sort_keys=False) + "\n")
    print(json.dumps({"verdict": v, "counts": report["counts"],
                      "detail": detail}, indent=2))
    return 0


def tally(trials):
    out = {}
    for t in trials:
        out[t["status"]] = out.get(t["status"], 0) + 1
    return out


def run_race(spike, root, outside, prefix, c, record, work):
    """One forced execution per deterministic schedule."""
    leaf = {"leaf_regular_to_symlink_swap": "raceleaf",
            "parent_to_symlink_swap": "racedir/child",
            "replacement_before_pin": "raceleaf",
            "replacement_after_pin": "raceleaf",
            "directory_replacement_after_pin": "racedir/child",
            "growing_file": "growfile",
            "truncating_file": "truncfile",
            "inplace_overwrite_deterministic": "inplace",
            "inplace_overwrite_undetectable": "inplace2",
            "hardlink_alias_mutation": "aliasfile"}[c["case"]]
    tid = leaf.split("/")[-1] if "/" in leaf else leaf
    target_path = root / leaf
    base = leaf.split("/")[0]

    def reset():
        if c["case"] in ("leaf_regular_to_symlink_swap", "replacement_before_pin",
                         "replacement_after_pin"):
            (root / "raceleaf").unlink(missing_ok=True)
            (root / "raceleaf").write_bytes(oracles.materialise("race_old"))
        if c["case"] in ("parent_to_symlink_swap", "directory_replacement_after_pin"):
            d = root / "racedir"
            if d.is_symlink():
                d.unlink()
            if not d.exists():
                d.mkdir()
            (d / "child").write_bytes(oracles.materialise("race_old"))

    reset()

    def mutate():
        if c["mutate"] == "swap_leaf_to_symlink":
            p = root / "raceleaf"
            p.unlink()
            p.symlink_to("plain")
        elif c["mutate"] == "swap_parent_to_symlink":
            d = root / "racedir"
            saved = root / "racedir-saved"
            if saved.exists():
                shutil.rmtree(saved)
            d.rename(saved)
            d.symlink_to("racedir-saved", target_is_directory=True)
        elif c["mutate"] == "replace_regular":
            tmp = root / "raceleaf.new"
            tmp.write_bytes(oracles.materialise("race_new"))
            tmp.replace(root / "raceleaf")
        elif c["mutate"] == "replace_directory":
            d = root / "racedir"
            saved = root / "racedir-saved2"
            if saved.exists():
                shutil.rmtree(saved)
            d.rename(saved)
            d.mkdir()
            (d / "child").write_bytes(oracles.materialise("race_new"))
        elif c["mutate"] == "grow_file":
            with open(root / "growfile", "ab") as f:
                f.write(oracles.materialise("race_new"))
        elif c["mutate"] == "truncate_file":
            os.truncate(root / "truncfile", 64 * 1024)
        elif c["mutate"] == "overwrite_tail_same_length":
            with open(root / "inplace", "r+b") as f:
                f.seek(64 * 1024)
                f.write(oracles.materialise("race_new")[64 * 1024:])
        elif c["mutate"] == "overwrite_mmap_same_length":
            import mmap
            with open(root / "inplace2", "r+b") as f:
                mm = mmap.mmap(f.fileno(), 0)
                mm[64 * 1024:] = oracles.materialise("race_new")[64 * 1024:]
                mm.close()
        elif c["mutate"] == "mutate_alias":
            (outside / "alias-canary").write_bytes(oracles.materialise("alias_new"))

    seen = {"reads": 0}

    def stop(ev):
        want = c["stop"]
        if want == "openat2_target":
            return ev["syscall"] == "openat2"
        if want == "reopen":
            return ev["syscall"] == "openat" and ev.get("path", "").isdigit()
        if want == "first_read":
            if ev["syscall"] == "read":
                seen["reads"] += 1
                return seen["reads"] == 1
        return False

    argv = [spike, "--root", str(root), "--rel", leaf, "--op", "file",
            "--id", tid, "--dump-prefix", prefix]
    rec = harness.run_traced(argv, stop=stop, mutate=mutate, timeout=20)
    actual = dump_digest(prefix, tid)
    allowed = [EXPECT[d] for d in c.get("digest_set", [])] or None
    t = record(c["case"], rec, c.get("trace", []), c["safe"],
               allowed, actual if allowed else None,
               extra={"seed": c["seed"], "stop": c["stop"], "mutation": c["mutate"],
                      "allowed_digests": c.get("digest_set"),
                      "actual_digest": actual,
                      "undetectable_arm": c.get("undetectable_ok", False)})
    reset()
    return t


if __name__ == "__main__":
    sys.exit(main())
