"""LAUNCH-EXEC-01 harness: preflight, fixture builder, descriptor pre-loader.

Disposable spike tooling. It BUILDS and INSPECTS; the runner poses cases.
Importing this module executes nothing.

NOT_RUN: no preregistered trial has been executed.
"""
import json
import os
import pathlib
import shutil
import subprocess
import sys

import make_fixtures
from frozen_cases import BLOCK_REASONS

STATIC_PROBE = b"""int main(void) { return 0; }\n"""

STATIC_CFLAGS = ["-O2", "-Wall", "-Wextra", "-static"]
DYNAMIC_CFLAGS = ["-O2", "-Wall", "-Wextra"]

STATIC_TARGETS = {
    "helper_report": "helper_report.c",
    "helper_alt": "helper_alt.c",
    "helper_fork": "helper_fork.c",
    "helper_setid": "helper_setid.c",
    "launcher_spike": "launcher_spike.c",
}
DYNAMIC_TARGETS = {"helper_dynamic": "helper_dynamic.c"}


def preflight():
    """Everything recorded before the first case. Executes no preregistered case.

    Every value here is an environment fact, not a result. The tracer and mount
    facts decide which conditional cases can be posed at all, and the block
    reasons they map to are frozen in frozen_cases.BLOCK_REASONS.
    """
    def run(cmd):
        try:
            out = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            return out.stdout.strip()
        except Exception as exc:                      # noqa: BLE001
            return f"<unavailable: {exc}>"

    info = {
        "uname": run(["uname", "-a"]),
        "arch": run(["uname", "-m"]),
        "kernel_release": run(["uname", "-r"]),
        "os_release": _read("/etc/os-release"),
        "geteuid": os.geteuid() if hasattr(os, "geteuid") else None,
        "ptrace_scope": _read("/proc/sys/kernel/yama/ptrace_scope"),
        "strace": shutil.which("strace"),
        "gcc": run(["gcc", "--version"]).splitlines()[:1],
        "glibc": run(["ldd", "--version"]).splitlines()[:1],
        "pagesize": run(["getconf", "PAGESIZE"]),
        "binfmt_misc_status": _read("/proc/sys/fs/binfmt_misc/status"),
        "binfmt_misc_entries": _binfmt_entries(),
        "mountinfo_noexec": _mounts_with("noexec"),
        "mountinfo_nosuid": _mounts_with("nosuid"),
    }
    info["block_reasons"] = _derive_blocks(info)
    return info


def _read(path):
    try:
        return pathlib.Path(path).read_text(encoding="utf-8", errors="replace").strip()
    except OSError as exc:
        return f"<unavailable: {exc}>"


def _binfmt_entries():
    """Every binfmt_misc registration, recorded BEFORE the first trial.

    An entry matching \\x7fELF with a masked e_machine is consulted ahead of
    binfmt_elf and resolves its interpreter by pathname, which is the state case
    E8's admission rule exists to make unreachable. A runner with no such entry
    cannot pose the hazard, so its absence must be evidenced, not assumed.
    """
    root = pathlib.Path("/proc/sys/fs/binfmt_misc")
    if not root.is_dir():
        return "<not mounted>"
    entries = {}
    try:
        for child in sorted(root.iterdir()):
            if child.name in ("register", "status"):
                continue
            entries[child.name] = _read(str(child))
    except OSError as exc:
        return f"<unavailable: {exc}>"
    return entries


def _mounts_with(option):
    out = []
    raw = _read("/proc/self/mountinfo")
    if raw.startswith("<unavailable"):
        return raw
    for line in raw.splitlines():
        if f" {option}" in line or f",{option}" in line:
            parts = line.split()
            if len(parts) > 4:
                out.append(parts[4])
    return out


def _derive_blocks(info):
    """Which frozen block reasons this environment actually triggers."""
    blocks = {}
    if info.get("geteuid") == 0:
        blocks["euid_zero"] = BLOCK_REASONS["euid_zero"]
    if not info.get("strace"):
        scope = info.get("ptrace_scope")
        if scope not in ("0", "1"):
            blocks["no_tracer"] = BLOCK_REASONS["no_tracer"]
    noexec = info.get("mountinfo_noexec")
    if not noexec or isinstance(noexec, str):
        blocks["no_noexec_mount"] = BLOCK_REASONS["no_noexec_mount"]
    # N3 is BLOCKED by construction on any unprivileged runner: no privileged
    # fixture is created, and no cross-UID elevation is manufactured.
    if info.get("geteuid") != 0:
        blocks["unprivileged_runner"] = BLOCK_REASONS["unprivileged_runner"]
    return blocks


def static_link_gate(build_dir):
    """Preflight linkability gate. NO SILENT SUBSTITUTION.

    Static linking is a precondition, not a preference: a complete trace is what
    makes "no other descriptor was present" evidence rather than filtering. If
    the static link fails, EVERY case is BLOCKED and the aggregate is
    MECHANISM_INCONCLUSIVE. A dynamic helper must not be substituted, in whole
    or for any individual case; musl-gcc must not be substituted; no package is
    installed and no sudo is used.
    """
    build = pathlib.Path(build_dir)
    build.mkdir(parents=True, exist_ok=True)
    probe_c = build / "probe.c"
    probe_c.write_bytes(STATIC_PROBE)
    probe = build / "probe"
    cmd = ["cc", *STATIC_CFLAGS, "-o", str(probe), str(probe_c)]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except Exception as exc:                          # noqa: BLE001
        return {"ok": False, "reason": f"cc unavailable: {exc}", "command": cmd}
    if proc.returncode != 0:
        return {"ok": False, "reason": "static link failed", "command": cmd,
                "stderr": proc.stderr[-2000:]}
    ldd = subprocess.run(["ldd", str(probe)], capture_output=True, text=True)
    dynamic = "not a dynamic executable" not in (ldd.stdout + ldd.stderr).lower()
    if dynamic:
        return {"ok": False, "reason": "probe linked dynamically", "command": cmd,
                "ldd": ldd.stdout.strip()}
    return {"ok": True, "command": cmd, "ldd": ldd.stdout.strip()}


def build(build_dir):
    """Build every helper and the spike. Returns built-binary digests.

    Compiling is not a preregistered trial: nothing here executes the spike or
    poses a case.
    """
    import hashlib
    build = pathlib.Path(build_dir)
    build.mkdir(parents=True, exist_ok=True)
    here = pathlib.Path(__file__).resolve().parent
    built = {}
    for name, source in sorted(STATIC_TARGETS.items()):
        target = build / name
        cmd = ["cc", *STATIC_CFLAGS, "-o", str(target), str(here / source)]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        if proc.returncode != 0:
            raise RuntimeError(f"{name}: static build failed\n{proc.stderr}")
        built[name] = hashlib.sha256(target.read_bytes()).hexdigest()
    for name, source in sorted(DYNAMIC_TARGETS.items()):
        target = build / name
        cmd = ["cc", *DYNAMIC_CFLAGS, "-o", str(target), str(here / source)]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        if proc.returncode != 0:
            raise RuntimeError(f"{name}: dynamic build failed\n{proc.stderr}")
        built[name] = hashlib.sha256(target.read_bytes()).hexdigest()
    # D-9 fixture: set-user-ID on a file the running user already owns. This
    # manufactures no privilege: the owner IS the caller.
    setid = build / "helper_setid"
    os.chmod(setid, 0o4755)
    make_fixtures.write(str(build))
    return built


def open_non_cloexec_descriptor(path):
    """F2: a descriptor the child must not see. Deliberately NOT CLOEXEC."""
    fd = os.open(path, os.O_RDONLY)
    os.set_inheritable(fd, True)
    return fd


if __name__ == "__main__":
    print(json.dumps(preflight(), indent=2, sort_keys=True, default=str))
    sys.exit(0)
