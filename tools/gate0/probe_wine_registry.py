#!/usr/bin/env python3
"""Gate 0 / G0-3b probe: does Wine's registry save reach a hardlinked copy, and is a quiesced
full copy independent of subsequent writes?

Scope. This probe runs REAL Wine against a disposable prefix with synthetic data only. It tests
the data-safety mechanics that ADR-0017 depends on. It does NOT test application recovery, GUI
behaviour, graphics, devices or sandbox containment.

Pre-registered questions (fixed here before execution):

  Q1  After a hardlink "snapshot" of the prefix, does a subsequent Wine registry write reach the
      copy? Expected: YES for at least one hive -> the copy is CONTAMINATED and is not a backup.
  Q2  Does a quiesced full copy stay byte-identical after further Wine registry writes?
      Expected: YES -> a quiesced full copy is an independent recoverable state.
  Q3  Does a synthetic user document created AFTER the copy survive a restore of that copy?
      Expected: NO for a naive whole-prefix restore -> restoring the prefix wholesale destroys
      work created after the copy. This is the failure ADR-0017 rule 2 exists to prevent.
  Q4  Which hive actually records a per-user setting written via `reg add HKCU\\...`?
      (Audit correction C-9: revision 1 inspected system.reg, which is the wrong hive.)

Controls:
  KNOWN-GOOD    a copy taken of a quiesced prefix with no subsequent write must compare equal.
  DELIBERATELY  a copy that is deliberately mutated must compare unequal, proving the comparison
  BROKEN        can actually detect a difference.

Independence is VERIFIED by comparison, never assumed. Reflink is capability-tested and never
inferred from a command that can silently fall back.

Output: a JSON record on stdout. Exit 0 if the probe ran to completion.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

WINE_TIMEOUT = 180


def sha256(path: Path) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return None


def hive_digests(prefix: Path) -> dict:
    out = {}
    for name in ("system.reg", "user.reg", "userdef.reg"):
        p = prefix / name
        out[name] = {"sha256": sha256(p), "nlink": (p.stat().st_nlink if p.exists() else None),
                     "size": (p.stat().st_size if p.exists() else None)}
    return out


def run(cmd: list[str], env: dict, label: str, cwd: str | None = None) -> dict:
    started = time.time()
    try:
        proc = subprocess.run(cmd, env=env, cwd=cwd, capture_output=True, text=True,
                              timeout=WINE_TIMEOUT)
        return {"label": label, "cmd": " ".join(cmd), "rc": proc.returncode,
                "seconds": round(time.time() - started, 2),
                "stderr_tail": proc.stderr.strip().splitlines()[-3:] if proc.stderr else []}
    except subprocess.TimeoutExpired:
        return {"label": label, "cmd": " ".join(cmd), "rc": None, "timeout": True,
                "seconds": round(time.time() - started, 2)}


def wine_env(prefix: Path) -> dict:
    env = dict(os.environ)
    env["WINEPREFIX"] = str(prefix)
    env["WINEARCH"] = "win64"
    env["WINEDEBUG"] = "-all"
    env.setdefault("DISPLAY", ":0")
    return env


def quiesce(prefix: Path, log: list) -> dict:
    """Stop wineserver and VERIFY, per ADR-0017 rule 1: issuing the command is not evidence."""
    env = wine_env(prefix)
    log.append(run(["wineserver", "-k"], env, "wineserver -k"))
    # -w waits for the server to actually exit; that is the verification step.
    waited = run(["wineserver", "-w"], env, "wineserver -w (wait for exit)")
    log.append(waited)
    still_running = subprocess.run(["pgrep", "-f", f"wineserver.*{prefix.name}"],
                                   capture_output=True, text=True).returncode == 0
    return {"wait_rc": waited.get("rc"), "wineserver_still_running": still_running,
            "verified_stopped": (waited.get("rc") == 0 and not still_running)}


def full_copy(src: Path, dst: Path) -> None:
    """Quiesced full copy into a FRESH destination. Never hardlinks."""
    shutil.copytree(src, dst, symlinks=True)


def hardlink_copy(src: Path, dst: Path) -> None:
    subprocess.run(["cp", "-al", str(src), str(dst)], check=True)


def probe_reflink(workdir: Path) -> dict:
    src, dst = workdir / "rl_src", workdir / "rl_dst"
    src.write_bytes(b"x" * 4096)
    r = subprocess.run(["cp", "--reflink=always", str(src), str(dst)],
                       capture_output=True, text=True)
    src.unlink(missing_ok=True)
    dst.unlink(missing_ok=True)
    return {"supported": r.returncode == 0, "rc": r.returncode,
            "stderr": r.stderr.strip()[:200],
            "note": "capability-tested explicitly; never inferred from a fallback-capable command"}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--workdir", type=Path, required=True)
    ap.add_argument("--keep", action="store_true")
    args = ap.parse_args(argv)

    work = args.workdir
    work.mkdir(parents=True, exist_ok=True)
    prefix = work / "pfx"
    log: list = []
    result: dict = {"probe": "wine_registry_semantics", "probe_version": "1", "checks": {},
                    "commands": log}

    wv = subprocess.run(["wine", "--version"], capture_output=True, text=True)
    result["wine_version"] = wv.stdout.strip() or wv.stderr.strip()
    result["kernel"] = os.uname().release
    fs = subprocess.run(["stat", "-f", "-c", "%T", str(work)], capture_output=True, text=True)
    result["filesystem"] = fs.stdout.strip()
    result["reflink"] = probe_reflink(work)

    env = wine_env(prefix)
    log.append(run(["wineboot", "--init"], env, "wineboot --init"))
    q0 = quiesce(prefix, log)
    result["checks"]["quiesce_after_init"] = q0
    if not (prefix / "user.reg").exists():
        result["error"] = "prefix creation did not produce user.reg"
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0

    baseline = hive_digests(prefix)

    # ---- copies: one hardlink farm, one quiesced full copy ----
    hard = work / "copy_hardlink"
    full = work / "copy_full"
    hardlink_copy(prefix, hard)
    full_copy(prefix, full)

    after_copy = {"prefix": hive_digests(prefix), "hardlink": hive_digests(hard),
                  "full": hive_digests(full)}

    # ---- CONTROL: known-good (no write yet) ----
    result["checks"]["control_known_good"] = {
        "description": "with no intervening write, the full copy must equal the prefix",
        "equal": all(after_copy["full"][h]["sha256"] == after_copy["prefix"][h]["sha256"]
                     for h in after_copy["prefix"]),
    }

    # ---- Q4/Q1/Q2: perform a real Wine registry write ----
    marker = f"helm-g0-{int(os.getpid())}"
    log.append(run(["wine", "reg", "add", r"HKCU\Software\HelmGate0", "/v", "Probe",
                    "/t", "REG_SZ", "/d", marker, "/f"], env, "reg add HKCU"))
    q1 = quiesce(prefix, log)
    result["checks"]["quiesce_after_write"] = q1

    after_write = {"prefix": hive_digests(prefix), "hardlink": hive_digests(hard),
                   "full": hive_digests(full)}

    def changed(where: str, hive: str) -> bool:
        return after_copy[where][hive]["sha256"] != after_write[where][hive]["sha256"]

    hives = ["system.reg", "user.reg", "userdef.reg"]
    result["checks"]["Q4_which_hive_records_hkcu"] = {
        "question": "Which hive changes when a per-user value is written?",
        "changed_hives": [h for h in hives if changed("prefix", h)],
        "note": ("audit correction C-9: revision 1 inspected system.reg only, which is the wrong "
                 "hive for an HKCU write"),
    }
    result["checks"]["Q1_hardlink_copy_contaminated"] = {
        "question": "Does a Wine registry write reach a hardlink copy of the prefix?",
        "changed_hives_in_copy": [h for h in hives if changed("hardlink", h)],
        "contaminated": any(changed("hardlink", h) for h in hives),
        "tracks_live": {h: after_write["hardlink"][h]["sha256"] == after_write["prefix"][h]["sha256"]
                        for h in hives},
        "nlink_before": {h: after_copy["hardlink"][h]["nlink"] for h in hives},
    }
    result["checks"]["Q2_full_copy_independent"] = {
        "question": "Does a quiesced full copy stay unchanged after further writes?",
        "changed_hives_in_copy": [h for h in hives if changed("full", h)],
        "independent": not any(changed("full", h) for h in hives),
    }

    # ---- CONTROL: deliberately broken ----
    broken = work / "copy_full_broken"
    full_copy(full, broken)
    (broken / "user.reg").write_bytes(b"DELIBERATELY CORRUPTED CONTROL\n")
    result["checks"]["control_deliberately_broken"] = {
        "description": "a mutated copy must compare unequal, proving the comparison detects change",
        "detected": sha256(broken / "user.reg") != sha256(full / "user.reg"),
    }

    # ---- Q3: does a document created after the copy survive a whole-prefix restore? ----
    docs = prefix / "drive_c" / "users" / os.environ.get("USER", "helmlab") / "Documents"
    docs.mkdir(parents=True, exist_ok=True)
    newdoc = docs / "created_after_copy.txt"
    newdoc.write_text("synthetic user work created AFTER the recovery point\n", encoding="utf-8")
    doc_digest = sha256(newdoc)

    restored = work / "restored_from_full"
    full_copy(full, restored)
    survived = (restored / "drive_c" / "users" / os.environ.get("USER", "helmlab")
                / "Documents" / "created_after_copy.txt").exists()
    result["checks"]["Q3_post_copy_document_survives_restore"] = {
        "question": "Does a user document created after the copy survive restoring that copy?",
        "document_digest": doc_digest,
        "survived_whole_prefix_restore": survived,
        "interpretation": ("If false, a naive whole-prefix restore silently destroys work created "
                           "after the recovery point. This is the exact failure ADR-0017 rule 2 "
                           "(tier separation) exists to prevent, and it is why runtime, prefix and "
                           "user-document recovery must be separate operations."),
    }

    result["scope_limits"] = [
        "Real Wine was executed; results are for the pinned Wine build and filesystem recorded above.",
        "WSL2 environment: valid only for this WSL configuration. Not generalisable to bare-metal "
        "graphics, device support, battery or physical sleep/resume.",
        "Synthetic data only. No application, no user account, no licensed software.",
        "This probe does NOT demonstrate application recovery: it tests state independence, not "
        "whether an application still works after a restore.",
    ]
    if not args.keep:
        for d in (hard, full, broken, restored):
            shutil.rmtree(d, ignore_errors=True)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
