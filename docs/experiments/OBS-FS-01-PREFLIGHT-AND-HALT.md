# OBS-FS-01: preflight complete, execution halted before the first trial

**Status:** **HALTED at the preregistration.** Preflight ran; **no preregistered trial
was executed**, so no goalpost has moved.\
**Date:** 2026-09-08.\
**Authorisation:** owner-authorised VM boot, synthetic fixtures, openat2/O_PATH/procfs
spike, race injection, syscall tracing and normal shutdown. Forbidden and not performed:
A0 read/rerun, Wine/7-Zip, sudo/root, AppArmor or sysctl change, helm-observe product
code, and changing expectations after the first trial.\
**Frozen definition:** `b4ed2e108134eb58a2561403f5da3509aed955ee`
([independent review](../implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md)).\
**Repository base:** `1875ada4f7178be70961f4007d1ab233cbf940e2`.

## 1. Why execution halted

Build validation on scratch fixtures — before the frozen fixture recipes or the harness
existed, and before any preregistered case was run — showed that a **material frozen
expectation is factually wrong about Linux behaviour**.

The frozen definition asserts in three places that a symbolic-link target is rejected at
*path resolution* with `ELOOP` and that no descriptor is produced. That is true only for
a symlink in a **non-final** component. For a **trailing** symlink under the mandated
flag set, `openat2` **succeeds** and returns an `O_PATH` descriptor to the link itself;
rejection then happens at classification.

This is documented kernel behaviour, not a surprise of this lab:

> "If the trailing component (i.e., basename) of path is a symbolic link, how.resolve
> contains RESOLVE_NO_SYMLINKS, and how.flags contains both O_PATH and O_NOFOLLOW, then
> an O_PATH file descriptor referencing the symbolic link will be returned."
> — [openat2(2)](https://man7.org/linux/man-pages/man2/openat2.2.html)

The same page explains why: `RESOLVE_NO_SYMLINKS` governs the *resolution* of symlinks in
all components, whereas `O_NOFOLLOW` governs the *final* component only. A trailing
symlink under `O_PATH|O_NOFOLLOW` is never resolved, so there is nothing for
`RESOLVE_NO_SYMLINKS` to reject.

Measured on the lab, with `strace` showing all four RESOLVE flags in effect:

| Case | `openat2` result | Spike stage and outcome |
|---|---|---|
| Trailing symlink (internal) | **succeeds**, returns `O_PATH` fd to the link | `classify` → `symlink_forbidden`, kind `symlink` |
| Trailing symlink (absolute target) | **succeeds**, fd to the link | `classify` → `symlink_forbidden` |
| Trailing symlink (dangling) | **succeeds**, fd to the link | `classify` → `symlink_forbidden` |
| **Parent-component symlink** | `-1 ELOOP` | `resolve` → `symlink_forbidden` |

```text
openat2(3, "link", {flags=O_RDONLY|O_NOFOLLOW|O_CLOEXEC|O_PATH,
        resolve=RESOLVE_NO_XDEV|RESOLVE_NO_MAGICLINKS|RESOLVE_NO_SYMLINKS|
                RESOLVE_BENEATH}, 24) = 4
openat2(3, "linkdir/target.txt", {flags=..., resolve=...}, 24) = -1 ELOOP
```

**The safety property is not in question.** In every case the link destination was never
resolved, opened or read, nothing escaped the authorised root, and no data-open occurred.
Only the predicted *mechanism and errno class* were wrong.

**Attribution.** The original
[architecture proposal](../research/HELM-OBSERVE-ARCHITECTURE.md) was right: it states
that "`O_PATH|O_NOFOLLOW` can return the link object itself, so handle type checking is
still mandatory", and its threat table says `O_PATH` link metadata can identify the
rejection. The independent review's finding **M3 was wrong** to call that sentence
inapplicable, and the review then propagated the error into the expected outcome sets.
The defect originates in the review, not in the proposal.

## 2. Exactly what is wrong, and the minimum correction

Three locations in the frozen definition need correction, all in
[the review](../implementation/HELM-OBSERVE-INDEPENDENT-REVIEW.md):

| Location | Current frozen text | Minimum correction |
|---|---|---|
| Section 7, object-kind table, symlink row | "Rejected at resolution with `ELOOP`; no descriptor is produced" | Split: trailing symlink → `O_PATH` descriptor to the link, rejected at classification; non-final symlink → `ELOOP` at resolution, no descriptor |
| Section 9, finding M3 | Claims the proposal's `O_PATH` link-metadata sentence "does not hold" | Withdraw M3; the proposal's sentence is correct |
| Section 12, static-symlink row | `R = {rejected: ELOOP class}`, `T = no descriptor for the link` | `R = {symlink_forbidden}` for both variants; `T` = destination never resolved/opened/read, no escape, no data-open. Errno `ELOOP` expected only for non-final components |

Section 8's race row ("leaf regular and symlink swap ... rejected in the `ELOOP` class")
needs the same split: after a swap to a trailing symlink the safe outcome is
`symlink_forbidden` from classification, not `ELOOP`.

No other case is affected. The correction **narrows nothing and weakens no safety
requirement**: it replaces a wrong mechanism prediction with the documented one while
keeping the identical no-traversal, no-data-open, no-escape properties.

## 3. Preflight results (authorised, not a trial)

| Item | Measured |
|---|---|
| VM identity | `helm-lab-desktop-7zip`, `bd39f424-5b26-479d-846f-2f28f6639637`, Gen 2, 4 vCPU, 8 GiB — matches the recorded lab |
| Guest | Ubuntu 24.04.4 LTS, Linux `7.0.0-31-generic`, x86_64 |
| Identity | `uid=1001(helmlab) gid=1001(helmlab) groups=1001(helmlab)` — no supplementary groups, no sudo used |
| Fixture filesystem | `/dev/sda2`, **ext4**, `rw,relatime` |
| Free disk | 12,929 MiB, above the 1 GiB fixture and 5 GiB total budgets |
| Toolchain | gcc 13.3.0 present; **rustc and cargo ABSENT** → spike written in C, no packages installed |
| `kernel.yama.ptrace_scope` | `1` — own-child tracing permitted, sufficient for the harness |
| procfs | `rw,nosuid,nodev,noexec,relatime`; **hidepid unset**; `/proc/self/fd` reachable |
| `STATX_MNT_ID_UNIQUE` | **Supported**; the kernel returned the unique ID and did *not* set `STATX_MNT_ID` in the result mask, confirming review finding M1 |
| strace | `/usr/bin/strace` present |
| `kernel.apparmor_restrict_unprivileged_userns` | **`1`** |
| `unshare -Urm true` | **fails**: `write failed /proc/self/uid_map: Operation not permitted` (`exit 1`); `unshare -U` alone succeeds |
| `NO_XDEV` fallback | `/run/user/1001` is a tmpfs mount point — the mandatory non-namespace fallback arm is executable |

**The bind-mount case is BLOCKED, as predicted.** Review finding I8 anticipated this from
[ADR-0016](../adr/ADR-0016-host-side-sandbox.md) and the
[foundation audit](../research/FOUNDATION_AUDIT.md); this is its first *measured*
confirmation on the lab. It was not bypassed: no sudo, no sysctl change, no AppArmor
profile. Per the frozen rules, exactly one claim will remain unverified — **that
`RESOLVE_NO_XDEV` rejects a bind mount created as a descendant of an authorised ext4
root** — while the mandatory non-namespace fallback still exercises the `NO_XDEV` path.

## 4. What was and was not done

Done: VM boot and graceful shutdown; unprivileged SSH as `helmlab`; preflight; a
disposable C spike (292 lines, built with `-Wall -Wextra`, no warnings) implementing the
frozen sequence including the corrected order that bounds file size **before** any
reopen; ad-hoc build validation on scratch fixtures under `~/obs-fs-01/smoke/`.

Not done: **no preregistered case**, no frozen fixture recipes, no harness, no race
injection, no repetition, no verdict. No `helm-observe` product code. The A0 tree at
`/home/helmlab/exp009-a0-7zip` was observed to exist and was **not read, hashed, copied
or modified**. No Wine or 7-Zip process. No package installed. No checkpoint created or
removed; the VM returned to `Off` with 0 checkpoints, as before.

The spike and its scratch fixtures live in the guest under `~/obs-fs-01/`, outside the A0
tree. Private receipts, including the guest address and preflight JSON, are retained
outside the repository under the existing private-provenance directory.

## 5. Decision requested

Execution cannot honestly continue under the current frozen text: running the static
symlink and symlink-swap cases as written would score a safe, correct mechanism against a
wrong prediction, and correcting the prediction myself after seeing lab behaviour is
exactly the goalpost movement this preregistration exists to prevent — more so because
the error is the reviewer's own.

The owner's acceptance requires a **new owner review before execution** for any material
expectation change, and expected safe outcome sets are material. The requested decision
is whether to approve the section 2 correction, after which OBS-FS-01 can be executed in
full against the amended frozen definition.

ADR-0022 remains **Proposed**. OBS-FS-01 remains **NOT_RUN**. A0-7ZIP remains
experimental **FAIL**.
