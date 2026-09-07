"""Capture the three fixed operations of EXP-009 G0-2, once per runtime."""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("spec", help="JSON: arm, source_commit, runtime_argv, environment, directory")
    args = parser.parse_args()
    spec = json.loads(Path(args.spec).read_text())
    directory = Path(spec["directory"]).resolve(strict=True)
    output = directory / (spec["arm"] + "-controlled.json")
    if output.exists():
        raise FileExistsError("Refusing to overwrite a previous arm")
    environment = os.environ.copy()
    environment.update(spec["environment"])
    record = {"spec": spec, "started_utc": now(), "runs": [], "identities": {}}
    for name in ["dcomp_probe.c", "dcomp_probe.exe", "g0_2_control.c", "g0_2_control.exe", "run_g0_2.py"]:
        data = (directory / name).read_bytes()
        record["identities"][name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
    if record["identities"]["dcomp_probe.exe"]["sha256"] != "b74011c0c154e1e742b6a27c2de7259befce4e9e528cb946f9221fdd2f445ce4":
        raise ValueError("Target executable differs from the historical probe")
    record["inherited_environment"] = {key: os.environ.get(key) for key in
        ["DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "WSL_INTEROP", "SSH_AUTH_SOCK", "DOCKER_HOST"]}
    for label, exe, arguments in [
        ("good", "g0_2_control.exe", ["good"]),
        ("broken", "g0_2_control.exe", ["broken"]),
        ("target", "dcomp_probe.exe", []),
    ]:
        command = spec["runtime_argv"] + [str(directory / exe)] + arguments
        run = {"label": label, "argv": command, "cwd": str(directory), "started_utc": now()}
        clock = time.monotonic()
        try:
            process = subprocess.run(command, env=environment, cwd=directory, capture_output=True, timeout=180)
            stdout, stderr = process.stdout, process.stderr
            run["exit_code"] = process.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr = error.stdout or b"", error.stderr or b""
            run.update(exit_code=None, timeout_seconds=180)
        except OSError as error:
            stdout, stderr = b"", b""
            run.update(exit_code=None, exec_errno=error.errno, exec_error=str(error))
        run.update(ended_utc=now(), duration_seconds=time.monotonic() - clock)
        for kind, data in [("stdout", stdout), ("stderr", stderr)]:
            artifact = directory / (spec["arm"] + "-" + label + "-" + kind + ".txt")
            with artifact.open("xb") as stream:
                stream.write(data)
            run[kind] = {"file": artifact.name, "sha256": hashlib.sha256(data).hexdigest(),
                         "bytes": len(data), "text": data.decode("utf-8", errors="replace")}
        parsed = []
        for line in stdout.decode("utf-8", errors="replace").splitlines():
            try:
                value = json.loads(line)
            except ValueError:
                continue
            if isinstance(value, dict) and ("control" in value or "dcomp_dll_loaded" in value):
                parsed.append(value)
        run["probe_records"] = parsed
        record["runs"].append(run)
        record["ended_utc"] = now()
        output.write_bytes((json.dumps(record, indent=2) + "\n").encode())
        print(json.dumps({"arm": spec["arm"], "label": label, "exit_code": run["exit_code"],
                          "seconds": run["duration_seconds"], "probe_records": parsed}), flush=True)


if __name__ == "__main__":
    main()
