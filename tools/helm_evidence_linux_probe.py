"""Bounded Linux x86_64 hostile-filesystem regression for helm-evidence.

Runs an unmodified verifier as an unprivileged ptraced child. Stops at a selected
open syscall, mutates only synthetic fixtures, then resumes the real syscall.
No injection, syscall return rewriting, root, special capabilities or secrets.
Every child has a deadline and is killed/reaped on timeout. Results are JSON.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import socket
import subprocess
import tempfile
import time


class Registers(ctypes.Structure):
    _fields_ = [(n, ctypes.c_ulonglong) for n in (
        'r15 r14 r13 r12 rbp rbx r11 r10 r9 r8 rax rcx rdx rsi rdi '
        'orig_rax rip cs eflags rsp ss fs_base gs_base ds es fs gs').split()]


def traced(binary, root, trigger, mutate, timeout=5):
    libc = ctypes.CDLL(None, use_errno=True)
    libc.ptrace.restype = ctypes.c_long

    def ptrace(request, pid, addr=0, data=0):
        ctypes.set_errno(0)
        r = libc.ptrace(ctypes.c_uint(request), ctypes.c_uint(pid),
                        ctypes.c_void_p(addr), ctypes.c_void_p(data))
        if r == -1 and ctypes.get_errno():
            raise OSError(ctypes.get_errno(), 'ptrace failed')
        return r

    def peek(pid, addr, length):
        return b''.join((ptrace(2, pid, addr+i) & ((1 << 64)-1)).to_bytes(8, 'little')
                        for i in range(0, length, 8))[:length]

    def cstring(pid, addr):
        raw = bytearray()
        for i in range(0, 2048, 8):
            word = peek(pid, addr+i, 8)
            raw.extend(word.split(b'\0')[0])
            if b'\0' in word:
                return os.fsdecode(bytes(raw))
        return '<overlong>'

    with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
        pid = os.fork()
        if pid == 0:
            try:
                os.dup2(output.fileno(), 1)
                os.dup2(errors.fileno(), 2)
                ptrace(0, 0)  # PTRACE_TRACEME, own child only
                os.kill(os.getpid(), signal.SIGSTOP)
                os.execv(str(binary), [str(binary), 'verify', str(root), '--json'])
            finally:
                os._exit(126)
        start = time.monotonic()
        injected = False
        entering = True
        exited = False
        opened = []
        selected = False
        exit_code = None
        timed_out = False
        try:
            # The initial stop also has a deadline.
            while not os.waitpid(pid, os.WNOHANG)[0]:
                if time.monotonic()-start > timeout:
                    raise TimeoutError('initial child stop')
                time.sleep(0.001)
            ptrace(0x4200, pid, 0, 1 | (1 << 20))  # TRACESYSGOOD | EXITKILL
            ptrace(24, pid)  # PTRACE_SYSCALL
            while True:
                waited, status = os.waitpid(pid, os.WNOHANG)
                if not waited:
                    if time.monotonic()-start > timeout:
                        timed_out = True
                        break
                    time.sleep(0.0005)
                    continue
                if os.WIFEXITED(status) or os.WIFSIGNALED(status):
                    exited = True
                    exit_code = os.waitstatus_to_exitcode(status)
                    break
                sig = os.WSTOPSIG(status)
                deliver = 0
                if sig == (signal.SIGTRAP | 0x80):
                    regs = Registers()
                    ptrace(12, pid, 0, ctypes.addressof(regs))  # GETREGS
                    if entering:
                        selected = False
                        if regs.orig_rax in (257, 437):  # openat, openat2
                            path = cstring(pid, regs.rsi)
                            flags = (int.from_bytes(peek(pid, regs.rdx, 8), 'little')
                                     if regs.orig_rax == 437 else regs.rdx)
                            if not injected and trigger(path, flags):
                                mutate()
                                injected = selected = True
                    elif selected:
                        result = ctypes.c_longlong(regs.rax).value
                        opened.append({'syscall': int(regs.orig_rax), 'result': result})
                    entering = not entering
                elif sig not in (signal.SIGTRAP, signal.SIGSTOP):
                    deliver = sig
                ptrace(24, pid, 0, deliver)
        finally:
            if not exited:
                os.kill(pid, signal.SIGKILL)
                os.waitpid(pid, 0)
        output.seek(0)
        errors.seek(0)
        raw, err = output.read(), errors.read()
        report = json.loads(raw) if raw else None
        return {'injected': injected, 'timeout': timed_out, 'exit_code': exit_code,
                'open_results': opened, 'stderr_empty': not err,
                'verdict': report['verdict'] if report else None,
                'failures': failures(report), 'seconds': time.monotonic()-start}


def failures(report):
    return sorted({c['code'] for c in report['checks']
                   if c['status'] in ('INVALID', 'INCOMPLETE')}) if report else []


def run(binary, root):
    r = subprocess.run([str(binary), 'verify', str(root), '--json'],
                       capture_output=True, timeout=5)
    report = json.loads(r.stdout)
    human = subprocess.run([str(binary), 'verify', str(root)], capture_output=True, timeout=5)
    assert str(root).encode() not in human.stdout + r.stdout
    return {'exit_code': r.returncode, 'verdict': report['verdict'],
            'failures': failures(report), 'stderr_empty': not r.stderr,
            'private_absolute_path_leaked': False}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', required=True, type=Path)
    p.add_argument('--output', type=Path)
    p.add_argument('--assert-safe', action='store_true')
    args = p.parse_args()
    assert platform.system() == 'Linux' and platform.machine() == 'x86_64'
    assert os.geteuid() != 0, 'Run filesystem tests without root'
    binary = args.binary.resolve()
    fixture = Path(__file__).resolve().parents[1] / 'crates/helm-evidence/tests/fixtures/synthetic'
    rows = []

    with tempfile.TemporaryDirectory(prefix='helm-evidence-review-') as temp:
        base = Path(temp)

        def fresh(name):
            root = base/name
            shutil.copytree(fixture, root)
            return root

        for name in ['file-internal', 'file-escape', 'directory-internal',
                     'directory-escape', 'dangling', 'fifo', 'socket', 'directory-as-file',
                     'device-symlink', 'hardlink', 'hardlink-modified']:
            root = fresh(name)
            leaf = root/'good.json'
            original = leaf.read_bytes()
            outside = base/(name+'-canary')
            outside.write_bytes(original)
            sock = None
            if name.startswith('directory-') and name != 'directory-as-file':
                target = root/'outputs/before'
                moved = (root/'saved-directory' if name.endswith('internal') else base/(name+'-outside'))
                target.rename(moved)
                target.symlink_to(os.path.relpath(moved, target.parent), target_is_directory=True)
            else:
                leaf.unlink()
                if name == 'file-internal':
                    (root/'saved.json').write_bytes(original)
                    leaf.symlink_to('saved.json')
                elif name == 'file-escape':
                    leaf.symlink_to(outside)
                elif name == 'dangling':
                    leaf.symlink_to('absent')
                elif name == 'fifo':
                    os.mkfifo(leaf)
                elif name == 'socket':
                    sock = socket.socket(socket.AF_UNIX)
                    sock.bind(str(leaf))
                elif name == 'directory-as-file':
                    leaf.mkdir()
                elif name == 'device-symlink':
                    leaf.symlink_to('/dev/null')
                elif name.startswith('hardlink'):
                    os.link(outside, leaf)
                    if name == 'hardlink-modified':
                        outside.write_bytes(b'synthetic mutation')
            result = run(binary, root)
            expected = 'COMPLETE' if name == 'hardlink' else 'INVALID'
            rows.append({'case': name, **result, 'passed': result['verdict'] == expected})
            if sock:
                sock.close()
            if name != 'hardlink-modified':
                assert outside.read_bytes() == original

        # Each race is forced once at the actual open boundary; no scheduler luck.
        for name in ['file-swap-internal', 'file-swap-escape', 'manifest-swap-internal',
                     'manifest-swap-escape', 'directory-swap-internal', 'directory-swap-escape',
                     'fifo-swap', 'delete', 'replace-wrong', 'replace-matching',
                     'manifest-replace', 'directory-replace']:
            root = fresh(name)
            is_dir = name.startswith('directory-')
            is_manifest = name.startswith('manifest-')
            rel = 'outputs/before/output.dat' if is_dir else ('bundle.json' if is_manifest else 'good.json')
            leaf = root/rel
            before = leaf.read_bytes()
            # Match leaf opens on both original whole-path and corrected component traversal.
            # For a directory swap stop before opening that directory, not after pinning it.
            def trigger(path, flags):
                if is_dir and path == 'before':
                    return bool(flags & os.O_DIRECTORY)
                return path in (rel, Path(rel).name) and not (flags & os.O_PATH)

            def mutate():
                if is_dir:
                    folder = root/'outputs/before'
                    moved = (base/(name+'-outside') if name.endswith('escape') else root/'saved-directory')
                    folder.rename(moved)
                    if name == 'directory-replace':
                        folder.mkdir()
                        (folder/'output.dat').write_bytes(b'synthetic wrong replacement')
                    else:
                        folder.symlink_to(os.path.relpath(moved, folder.parent), target_is_directory=True)
                elif name == 'delete':
                    leaf.unlink()
                elif name in ('replace-wrong', 'replace-matching', 'manifest-replace'):
                    replacement = root/'replacement'
                    replacement.write_bytes(b'synthetic wrong replacement' if name == 'replace-wrong' else before)
                    replacement.replace(leaf)
                elif name == 'fifo-swap':
                    leaf.unlink()
                    os.mkfifo(leaf)
                else:
                    destination = (base/(name+'-outside') if name.endswith('escape') else root/'saved.json')
                    leaf.rename(destination)
                    leaf.symlink_to(os.path.relpath(destination, leaf.parent))

            result = traced(binary, root, trigger, mutate)
            expected = ('INCOMPLETE' if name == 'delete' else
                        'COMPLETE' if name in ('replace-matching', 'manifest-replace') else 'INVALID')
            passed = result['injected'] and not result['timeout'] and result['verdict'] == expected
            rows.append({'case': name, **result, 'passed': passed})

    receipt = {'scope': 'Synthetic Linux filesystem objects and deterministic syscall-boundary races',
               'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
               'uid': os.geteuid(), 'cases': rows,
               'device_limit': 'Unprivileged reference via symlink to /dev/null; no mknod or device creation',
               'all_passed': all(r['passed'] for r in rows)}
    raw = json.dumps(receipt, indent=2)+'\n'
    if args.output:
        args.output.write_text(raw)
    print(raw)
    return int(args.assert_safe and not receipt['all_passed'])


if __name__ == '__main__':
    raise SystemExit(main())
