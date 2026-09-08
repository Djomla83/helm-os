"""OBS-FS-01 parent harness: fixtures, complete syscall tracer, deterministic
injector, and case runner.

Unprivileged. Traces its own child only (PTRACE_TRACEME in the child), logs every
relevant syscall with decoded openat2 open_how.flags and open_how.resolve, tracks
descriptor lifetime so reuse is not confused, and forces each race mutation at a
named syscall stop rather than relying on timing. Mutates only its own disposable
fixtures. Every child has a deadline and is killed and reaped on timeout.

Method follows tools/helm_evidence_linux_probe.py, extended per the frozen
instrumentation requirements: all opens and reads are logged, not just the
injected one, because the negative claims need a complete trace.
"""
from __future__ import annotations

import ctypes
import json
import os
from pathlib import Path
import shutil
import signal
import socket
import subprocess
import sys
import time

SYS = {0: "read", 3: "close", 9: "mmap", 17: "pread64", 19: "readv", 42: "connect",
       137: "statfs", 138: "fstatfs", 217: "getdents64", 257: "openat",
       295: "preadv", 332: "statx", 437: "openat2"}
DATA_READS = {"read", "pread64", "readv", "preadv"}

O_PATH = 0o10000000
O_NONBLOCK = 0o4000
RESOLVE_BITS = [(0x01, "NO_XDEV"), (0x02, "NO_MAGICLINKS"),
                (0x04, "NO_SYMLINKS"), (0x08, "BENEATH"), (0x10, "IN_ROOT"),
                (0x20, "CACHED")]


class Regs(ctypes.Structure):
    _fields_ = [(n, ctypes.c_ulonglong) for n in (
        'r15 r14 r13 r12 rbp rbx r11 r10 r9 r8 rax rcx rdx rsi rdi '
        'orig_rax rip cs eflags rsp ss fs_base gs_base ds es fs gs').split()]


def decode_resolve(v):
    return "|".join(n for b, n in RESOLVE_BITS if v & b) or "0"


def run_traced(argv, *, stop=None, mutate=None, timeout=10, cwd=None):
    """Run argv under ptrace, log every relevant syscall, optionally inject once.

    stop: a predicate(event) -> bool selecting the syscall-entry stop at which
    mutate() is called. Returns a record including the complete trace.
    """
    libc = ctypes.CDLL(None, use_errno=True)
    libc.ptrace.restype = ctypes.c_long

    def ptrace(req, pid, addr=0, data=0):
        ctypes.set_errno(0)
        r = libc.ptrace(ctypes.c_uint(req), ctypes.c_uint(pid),
                        ctypes.c_void_p(addr), ctypes.c_void_p(data))
        if r == -1 and ctypes.get_errno():
            raise OSError(ctypes.get_errno(), 'ptrace failed')
        return r

    def peek(pid, addr, length):
        return b''.join((ptrace(2, pid, addr + i) & ((1 << 64) - 1)).to_bytes(8, 'little')
                        for i in range(0, length, 8))[:length]

    def cstring(pid, addr):
        if not addr:
            return ''
        raw = bytearray()
        for i in range(0, 4096, 8):
            word = peek(pid, addr + i, 8)
            raw.extend(word.split(b'\0')[0])
            if b'\0' in word:
                return os.fsdecode(bytes(raw))
        return '<overlong>'

    import tempfile
    with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
        pid = os.fork()
        if pid == 0:
            try:
                os.dup2(out.fileno(), 1)
                os.dup2(err.fileno(), 2)
                if cwd:
                    os.chdir(cwd)
                ptrace(0, 0)                      # PTRACE_TRACEME, own child only
                os.kill(os.getpid(), signal.SIGSTOP)
                os.execv(argv[0], argv)
            finally:
                os._exit(126)

        start = time.monotonic()
        trace, injected, timed_out, exit_code = [], False, False, None
        entering, pending, exited = True, None, False
        try:
            while not os.waitpid(pid, os.WNOHANG)[0]:
                if time.monotonic() - start > timeout:
                    raise TimeoutError('initial stop')
                time.sleep(0.001)
            ptrace(0x4200, pid, 0, 1 | (1 << 20))  # TRACESYSGOOD | EXITKILL
            ptrace(24, pid)
            while True:
                waited, status = os.waitpid(pid, os.WNOHANG)
                if not waited:
                    if time.monotonic() - start > timeout:
                        timed_out = True
                        break
                    time.sleep(0.0002)
                    continue
                if os.WIFEXITED(status) or os.WIFSIGNALED(status):
                    exited = True
                    exit_code = os.waitstatus_to_exitcode(status)
                    break
                sig = os.WSTOPSIG(status)
                deliver = 0
                if sig == (signal.SIGTRAP | 0x80):
                    regs = Regs()
                    ptrace(12, pid, 0, ctypes.addressof(regs))
                    name = SYS.get(regs.orig_rax)
                    if entering:
                        pending = None
                        if name:
                            ev = {"syscall": name, "dirfd": ctypes.c_long(regs.rdi).value}
                            if name in ("openat", "openat2"):
                                ev["path"] = cstring(pid, regs.rsi)
                                if name == "openat2":
                                    how = peek(pid, regs.rdx, 24)
                                    ev["flags"] = int.from_bytes(how[0:8], 'little')
                                    ev["resolve"] = int.from_bytes(how[16:24], 'little')
                                    ev["resolve_names"] = decode_resolve(ev["resolve"])
                                else:
                                    ev["flags"] = regs.rdx
                                ev["o_path"] = bool(ev["flags"] & O_PATH)
                            elif name in DATA_READS:
                                ev["fd"] = ctypes.c_long(regs.rdi).value
                                ev["count"] = regs.rdx
                            elif name == "mmap":
                                ev["fd"] = ctypes.c_long(regs.r8).value
                            elif name in ("statx", "fstatfs", "close", "connect",
                                          "getdents64"):
                                ev["fd"] = ctypes.c_long(regs.rdi).value
                                if name == "statx":
                                    ev["path"] = cstring(pid, regs.rsi)
                            pending = ev
                            if not injected and stop and stop(ev):
                                mutate()
                                injected = True
                                ev["injected_here"] = True
                    elif pending is not None:
                        pending["result"] = ctypes.c_long(regs.rax).value
                        trace.append(pending)
                        pending = None
                    entering = not entering
                elif sig not in (signal.SIGTRAP, signal.SIGSTOP):
                    deliver = sig
                ptrace(24, pid, 0, deliver)
        finally:
            if not exited:
                try:
                    os.kill(pid, signal.SIGKILL)
                    os.waitpid(pid, 0)
                except OSError:
                    pass
        out.seek(0); err.seek(0)
        stdout, stderr = out.read(), err.read()
        results = []
        for line in stdout.decode('utf-8', 'replace').splitlines():
            line = line.strip()
            if line.startswith('{'):
                try:
                    results.append(json.loads(line))
                except ValueError:
                    pass
        return {"argv_basename": os.path.basename(argv[0]),
                "injection_requested": stop is not None,
                "injected": injected, "timeout": timed_out,
                "exit_code": exit_code, "results": results, "trace": trace,
                "stderr_bytes": len(stderr),
                "seconds": round(time.monotonic() - start, 3)}


# --------------------------------------------------------------- fixture build
def build_fixtures(root: Path, oracle_bytes, overlimit_apparent: int):
    """Create the synthetic fixture tree. Only disposable harness files."""
    root.mkdir(parents=True, exist_ok=True)
    outside = root.parent / "outside-canary"
    outside.mkdir(exist_ok=True)

    def put(rel, name):
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(oracle_bytes(name))
        return p

    put("plain", "plain")
    put("empty", "empty")
    put("wrongsame", "wrongsame")
    put("filea", "twofile_a")
    put("fileb", "twofile_b")
    put("budgeta", "budget_a")
    put("budgetb", "budget_b")
    put("private", "private")
    put("unlisted", "unlisted")
    (root / "adir").mkdir(exist_ok=True)

    # sparse: apparent size with a single tail byte, no full allocation
    sp = root / "sparse"
    with open(sp, "wb") as f:
        f.truncate(1024 * 1024 - 1)
        f.seek(1024 * 1024 - 1)
        f.write(b"Z")

    over = root / "overlimit"
    with open(over, "wb") as f:
        f.truncate(overlimit_apparent)

    # hardlink to a canary that lives outside the authorized root
    canary = outside / "canary"
    canary.write_bytes(oracle_bytes("canary"))
    hl = root / "hardlink"
    if hl.exists():
        hl.unlink()
    os.link(canary, hl)

    # symlinks
    for name, dest in (("linkint", "plain"), ("linkesc", str(canary)),
                       ("linkdang", "nowhere-at-all")):
        p = root / name
        if p.is_symlink() or p.exists():
            p.unlink()
        p.symlink_to(dest)
    realdir = root / "realdir"
    realdir.mkdir(exist_ok=True)
    (realdir / "child").write_bytes(oracle_bytes("plain"))
    ld = root / "linkdir"
    if ld.is_symlink() or ld.exists():
        ld.unlink() if ld.is_symlink() else shutil.rmtree(ld)
    ld.symlink_to("realdir", target_is_directory=True)

    # special files
    fifo = root / "fifo"
    if fifo.exists():
        fifo.unlink()
    os.mkfifo(fifo)
    sockp = root / "sock"
    if sockp.exists():
        sockp.unlink()
    s = socket.socket(socket.AF_UNIX)
    s.bind(str(sockp))

    # permission fixtures
    pl = root / "permleaf"
    pl.write_bytes(oracle_bytes("permleaf"))
    os.chmod(pl, 0o000)
    pd = root / "permdir"
    pd.mkdir(exist_ok=True)
    (pd / "child").write_bytes(oracle_bytes("permchild"))
    os.chmod(pd, 0o000)

    # race fixtures
    put("raceleaf", "race_old")
    put("growfile", "grow_base")
    put("truncfile", "trunc_base")
    put("inplace", "race_old")
    put("inplace2", "race_old")
    alias_target = outside / "alias-canary"
    alias_target.write_bytes(oracle_bytes("alias_old"))
    al = root / "aliasfile"
    if al.exists():
        al.unlink()
    os.link(alias_target, al)
    racedir = root / "racedir"
    racedir.mkdir(exist_ok=True)
    (racedir / "child").write_bytes(oracle_bytes("race_old"))
    return {"root": root, "outside": outside, "socket": s}


def cleanup_permissions(root: Path):
    for rel in ("permleaf", "permdir"):
        p = root / rel
        try:
            os.chmod(p, 0o700 if p.is_dir() else 0o600)
        except OSError:
            pass
