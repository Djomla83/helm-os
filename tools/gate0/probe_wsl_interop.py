"""Capture one authorised WSL interop negative test; do not infer containment."""

import argparse
import base64
import datetime
import hashlib
import json
import os
from pathlib import Path
import struct
import subprocess
import time


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def capture(argv, *, cwd=None, timeout=20):
    started = now()
    clock = time.monotonic()
    result = {"argv": argv, "cwd": cwd, "started_utc": started}
    try:
        process = subprocess.run(argv, cwd=cwd, capture_output=True, timeout=timeout)
        result.update(exit_code=process.returncode, stdout_base64=base64.b64encode(process.stdout).decode(),
                      stderr_base64=base64.b64encode(process.stderr).decode(),
                      stdout=process.stdout.decode("utf-8", errors="replace"),
                      stderr=process.stderr.decode("utf-8", errors="replace"))
    except subprocess.TimeoutExpired as error:
        result.update(exit_code=None, timeout_seconds=timeout,
                      stdout_base64=base64.b64encode(error.stdout or b"").decode(),
                      stderr_base64=base64.b64encode(error.stderr or b"").decode())
    except OSError as error:
        result.update(exit_code=None, exec_errno=error.errno, exec_error=str(error),
                      stdout_base64="", stderr_base64="")
    result.update(ended_utc=now(), duration_seconds=time.monotonic() - clock)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("exe")
    parser.add_argument("expected_sha256")
    parser.add_argument("output")
    args = parser.parse_args()
    target = Path(args.exe).resolve(strict=True)
    output = Path(args.output)
    if output.exists():
        raise FileExistsError("Refusing to overwrite a previous observation")
    data = target.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != args.expected_sha256.lower():
        raise ValueError("Transferred executable hash does not match the source")
    pe_offset = struct.unpack_from("<I", data, 0x3C)[0]
    if data[:2] != b"MZ" or data[pe_offset:pe_offset + 4] != b"PE\0\0":
        raise ValueError("Expected a valid MZ/PE signature")
    record = {
        "check": "G0-2 completion prerequisite: WSL Windows-process execution",
        "baseline_commit": "ca0aa5ae26a1d3b630f63eda8f87626d125248d1",
        "distro": os.environ.get("WSL_DISTRO_NAME"), "uid": os.getuid(), "started_utc": now(),
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "exe": {"path": str(target), "sha256": digest, "bytes": len(data),
                "pe_machine": hex(struct.unpack_from("<H", data, pe_offset + 4)[0])},
        "environment": {key: os.environ.get(key) for key in
                        ["WSL_INTEROP", "WINEPREFIX", "DISPLAY", "WAYLAND_DISPLAY",
                         "SSH_AUTH_SOCK", "DOCKER_HOST"]},
        "observations": [],
    }
    for command in [
        ["cat", "/etc/wsl.conf"],
        ["cat", "/proc/sys/fs/binfmt_misc/WSLInterop"],
        ["cat", "/proc/sys/fs/binfmt_misc/status"],
        ["mountpoint", "-q", "/mnt/c"],
        ["findmnt", "-rn", "-o", "TARGET,FSTYPE"],
        ["uname", "-r"], ["cat", "/etc/os-release"], ["id"],
        ["df", "-B1", "/home/helmlab"],
    ]:
        record["observations"].append(capture(command))
    record["execution"] = capture(
        [str(target), "/d", "/c", "echo", "HELM_G0_INTEROP_NEGATIVE_TEST"],
        cwd=str(target.parent),
    )
    record["ended_utc"] = now()
    encoded = (json.dumps(record, indent=2) + "\n").encode("utf-8")
    with output.open("xb") as stream:
        stream.write(encoded)
    print(encoded.decode("utf-8"), end="")


if __name__ == "__main__":
    main()
