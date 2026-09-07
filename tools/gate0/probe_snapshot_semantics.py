#!/usr/bin/env python3
"""Gate 0 / G0-3 probe: filesystem snapshot semantics for a mutable state directory.

WHAT THIS PROBE DOES AND DOES NOT SHOW
--------------------------------------
This probe tests FILESYSTEM MECHANICS ONLY, with a synthetic writer. It does not run Wine and
therefore says nothing about which save path Wine's registry code actually takes. It answers:

  Q1  If a "snapshot" is a hardlink farm, does a later in-place rewrite of the live file change
      the snapshot too? (snapshot CONTAMINATION)
  Q2  Does a write-temp-then-rename writer leave the hardlink snapshot intact?
  Q3  Is there a window in which the live file is observably truncated/partial, i.e. the window
      in which a crash or power loss would leave a damaged file? (this demonstrates the WINDOW,
      not an actual corruption event)
  Q4  Does this filesystem support reflink copies (cp --reflink), which are the proposed
      alternative because they do not raise st_nlink?

Contamination (Q1) and corruption of the live file (Q3) are different failures and are reported
separately. Contamination alone is sufficient to disqualify hardlink farms as snapshots, because
the "backup" silently tracks the live data.

Output: a JSON record on stdout. Exit code 0 if the probe ran, 1 if it could not run.
Safe: operates only inside a temporary directory it creates, and removes it unless --keep.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

V1 = b"[live]\nvalue=1\n" + b"x" * 4096
V2 = b"[live]\nvalue=2\n" + b"y" * 8192


def sha256(path: Path) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return None


def detect_fs(path: Path) -> dict:
    info = {"path": str(path), "fstype": "unknown", "device": "unknown"}
    try:
        out = subprocess.run(["stat", "-f", "-c", "%T", str(path)],
                             capture_output=True, text=True, timeout=10)
        if out.returncode == 0:
            info["fstype"] = out.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        pass
    try:
        out = subprocess.run(["df", "--output=source", str(path)],
                             capture_output=True, text=True, timeout=10)
        if out.returncode == 0:
            lines = [line.strip() for line in out.stdout.splitlines() if line.strip()]
            if len(lines) > 1:
                info["device"] = lines[-1]
    except (OSError, subprocess.SubprocessError):
        pass
    return info


def write_in_place(target: Path, data: bytes, hold_seconds: float = 0.0,
                   observer: Path | None = None) -> dict | None:
    """Truncate the existing inode and rewrite it. This is the unsafe pattern."""
    observed_mid_write = None
    with open(target, "r+b") as handle:
        handle.truncate(0)
        handle.flush()
        os.fsync(handle.fileno())
        if observer is not None:
            observed_mid_write = {
                "observer_size_bytes": observer.stat().st_size,
                "observer_sha256": sha256(observer),
            }
        if hold_seconds:
            time.sleep(hold_seconds)
        handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())
    return observed_mid_write


def write_atomic(target: Path, data: bytes) -> None:
    """Write a sibling temp file and rename over the target. This is the safe pattern."""
    tmp = target.with_suffix(target.suffix + ".tmp")
    with open(tmp, "wb") as handle:
        handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(tmp, target)


def probe_reflink(workdir: Path) -> dict:
    src = workdir / "reflink_src.bin"
    dst = workdir / "reflink_dst.bin"
    src.write_bytes(V1)
    try:
        out = subprocess.run(["cp", "--reflink=always", str(src), str(dst)],
                             capture_output=True, text=True, timeout=30)
        supported = out.returncode == 0
        return {
            "supported": supported,
            "returncode": out.returncode,
            "stderr": out.stderr.strip()[:300],
            "note": ("reflink copies do not raise st_nlink, so an in-place rewrite of the live "
                     "file cannot reach the copy" if supported else
                     "reflink unavailable on this filesystem; the proposed snapshot mechanism "
                     "cannot be used here"),
        }
    except (OSError, subprocess.SubprocessError) as exc:
        return {"supported": False, "returncode": None, "stderr": str(exc)[:300],
                "note": "cp --reflink could not be executed"}


def run(workdir: Path) -> dict:
    result: dict = {
        "probe": "snapshot_semantics",
        "probe_version": "1",
        "host": {
            "platform": platform.platform(),
            "python": sys.version.split()[0],
        },
        "filesystem": detect_fs(workdir),
        "checks": {},
    }

    # --- Q1: hardlink farm + in-place rewrite -> contamination? ---
    live = workdir / "hive_a.reg"
    live.write_bytes(V1)
    snap_dir = workdir / "snapshot_hardlink"
    snap_dir.mkdir()
    snap = snap_dir / "hive_a.reg"
    os.link(live, snap)

    before = {"live": sha256(live), "snapshot": sha256(snap),
              "nlink": live.stat().st_nlink}
    mid = write_in_place(live, V2, hold_seconds=0.05, observer=snap)
    after = {"live": sha256(live), "snapshot": sha256(snap),
             "nlink": live.stat().st_nlink}

    result["checks"]["Q1_hardlink_in_place_rewrite"] = {
        "question": "Does an in-place rewrite of the live file change a hardlink 'snapshot'?",
        "before": before,
        "after": after,
        "snapshot_changed": before["snapshot"] != after["snapshot"],
        "snapshot_tracks_live": after["snapshot"] == after["live"],
        "verdict": ("CONTAMINATED: the hardlink copy is not an independent snapshot"
                    if after["snapshot"] == after["live"] and before["snapshot"] != after["snapshot"]
                    else "snapshot retained its own content"),
    }

    # --- Q3: is the live file observably truncated during the rewrite? ---
    result["checks"]["Q3_truncation_window"] = {
        "question": ("Is there a window in which the file is observably empty/partial, i.e. the "
                     "window in which a crash would leave it damaged?"),
        "observed_during_write": mid,
        "window_observed": bool(mid and mid.get("observer_size_bytes") == 0),
        "verdict_note": ("This demonstrates only that the window EXISTS. It is not a demonstration "
                         "of an actual corruption event, which additionally requires a crash, "
                         "power loss or kill inside that window."),
    }

    # --- Q2: hardlink farm + atomic replace -> intact? ---
    live2 = workdir / "hive_b.reg"
    live2.write_bytes(V1)
    snap2_dir = workdir / "snapshot_hardlink_atomic"
    snap2_dir.mkdir()
    snap2 = snap2_dir / "hive_b.reg"
    os.link(live2, snap2)

    before2 = {"live": sha256(live2), "snapshot": sha256(snap2),
               "nlink": live2.stat().st_nlink}
    write_atomic(live2, V2)
    after2 = {"live": sha256(live2), "snapshot": sha256(snap2),
              "nlink": live2.stat().st_nlink}

    result["checks"]["Q2_hardlink_atomic_replace"] = {
        "question": "Does a write-temp-then-rename writer leave the hardlink snapshot intact?",
        "before": before2,
        "after": after2,
        "snapshot_preserved": before2["snapshot"] == after2["snapshot"],
        "verdict": ("snapshot preserved: rename breaks the link, the copy keeps the old inode"
                    if before2["snapshot"] == after2["snapshot"]
                    else "snapshot changed"),
    }

    # --- Q4: reflink support ---
    result["checks"]["Q4_reflink_support"] = probe_reflink(workdir)

    result["scope_limits"] = [
        "Synthetic writer. Wine was not installed or executed; this says nothing about which "
        "save path Wine's registry code actually takes.",
        "Filesystem mechanics only. Results are specific to the filesystem reported above.",
        "Q3 demonstrates the existence of a truncation window, not an observed corruption event.",
    ]
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workdir", type=Path, default=None,
                        help="Directory to create the disposable test tree in (default: system temp)")
    parser.add_argument("--keep", action="store_true", help="Do not delete the test tree")
    args = parser.parse_args(argv)

    base = tempfile.mkdtemp(prefix="helm-g0-3-", dir=str(args.workdir) if args.workdir else None)
    workdir = Path(base)
    try:
        result = run(workdir)
        result["workdir"] = str(workdir)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    finally:
        if not args.keep:
            shutil.rmtree(workdir, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())
