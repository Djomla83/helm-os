# OBS-FS-01 execution report

**Experiment verdict: PASS**, with the conditional descendant bind-mount case
**BLOCKED** and its residual claim carried as unverified.\
**Date:** 2026-09-08. **Lab:** `helm-lab-desktop-7zip`, booted and shut down normally.\
**This does not accept [ADR-0022](../adr/ADR-0022-observation-authority.md), which
remains Proposed, and authorises no `helm-observe` implementation.**

## 1. Identities

| Role | Commit |
|---|---|
| Authorising main | `63ac6796296894945dff520423cb1a93351e8524` |
| Original frozen preregistration | `b4ed2e108134eb58a2561403f5da3509aed955ee` |
| Preflight falsification and halt | `90e025890a32b36ccc55a4dc223bbb85046ba152` |
| Amendment 1, operative expectations | `63ac6796296894945dff520423cb1a93351e8524` |
| **Execution definition commit** | **`2496f68cccbf70ac04288992a9945cb34f046b98`** |

The execution definition was committed and pushed **before the first preregistered
trial**. Development and debugging runs that preceded it are preparation, not results,
and are not reported as trials.

Source actually executed, SHA-256 of the committed bytes, verified byte-identical on the
guest before the run:

| File | SHA-256 |
|---|---|
| `spike.c` | `241d9d6d3af533c28576289392d82813b40ae047ff2e0141a9738c24126107f7` |
| `frozen_cases.py` | `c935f421ec35120470faeba34b484726869823c10ce2e60f50a5e5fdf2db37f1` |
| `oracles.py` | `d990cca6e61454e6c82f9388fe518b529aadbe76f60b0c33eb9ec92825605354` |
| `harness.py` | `d72a1e54e2c135c456e95c94b0613ed5da14c32a4b26eb0affc3cc30105f4a05` |
| `checker.py` | `6d1303e6d8a77d6bc160027dde13b4598d068332734b486683cef8ecffde777a` |
| `run_obs_fs_01.py` | `08e68416a330b6c98affdc659063a298cd045b5e2688daa0c1217bee3678aa0e` |
| built `spike` binary | `ad412a80ef2348e1ce6b76dc67bc5fec62c4168a65c071e12cfe3ccb603bd26b` |

`SOURCE-HASHES.json` in the definition commit records the same files as they sit in a
Windows working tree with CRLF endings; the table above is the authoritative LF content
that was compiled and executed.

## 2. Refreshed lab facts

Re-measured on this boot rather than carried over from the preserved preflight:

| Item | Value |
|---|---|
| VM identity | `bd39f424-5b26-479d-846f-2f28f6639637`, matches the recorded lab |
| Guest | Ubuntu 24.04.4 LTS, Linux `7.0.0-31-generic`, x86_64 |
| Identity | `uid=1001(helmlab) gid=1001(helmlab) groups=1001(helmlab)` — no sudo at any point |
| Fixture filesystem | `/dev/sda2`, ext4, `rw,relatime` |
| Free disk | 12,909 MiB before the run |
| Toolchain | gcc 13.3.0, **statically linked**; rustc and cargo absent, nothing installed |
| `kernel.yama.ptrace_scope` | `1` — own-child tracing permitted |
| procfs | mounted, `hidepid` unset, `/proc/self/fd` reachable |
| `STATX_MNT_ID_UNIQUE` | supported; the kernel returned the unique ID and did not set `STATX_MNT_ID` |
| `kernel.apparmor_restrict_unprivileged_userns` | **`1`** |
| `unshare -Urm true` | **fails**: `write failed /proc/self/uid_map: Operation not permitted` |
| `NO_XDEV` fallback boundary | `/run/user/1001`, an existing tmpfs mount |

No sysctl, AppArmor, package, VM, checkpoint, switch or WSL change was made.

## 3. Instrumentation

The spike is linked **statically on purpose**, so the dynamic loader contributes no
syscalls and a negative claim rests on a complete trace rather than on filtering. During
preparation a dynamically linked build put loader opens and reads into the trace; that is
why static linking is part of the definition.

Every child ran under parent-to-child `ptrace` with `PTRACE_TRACEME` in the child, a
deadline, and kill-and-reap on timeout. The tracer logs **every** `openat`, `openat2`,
`statx`, `read`, `pread64`, `readv`, `preadv`, `mmap`, `connect`, `getdents64`, `fstatfs`
and `close`, decoding `open_how.flags` **and** `open_how.resolve` and recording returned
descriptors so reuse is not confused. Race mutations are forced at a named syscall stop,
never by timing. The policy checker imports and links nothing from the spike.

A representative complete trace of the approved mechanism, ten syscalls end to end:

```text
openat(AT_FDCWD, <root>, O_PATH|O_DIRECTORY|O_CLOEXEC)              = 3   authorized root
openat(AT_FDCWD, "/proc/self/fd", O_PATH|O_DIRECTORY|O_CLOEXEC)     = 4   procfs capability
openat(4, "3", O_RDONLY|O_CLOEXEC|O_PATH)                           = 5   self-identity admission probe
openat2(3, "p", {flags=O_NOFOLLOW|O_CLOEXEC|O_PATH,
        resolve=NO_XDEV|NO_MAGICLINKS|NO_SYMLINKS|BENEATH}, 24)     = 5   target pin
statx(5, "", AT_EMPTY_PATH, ...S_IFREG...)                          = 0   classify
openat(4, "5", O_RDONLY|O_NOCTTY|O_NONBLOCK|O_CLOEXEC)              = 6   procfs reopen of the pin
statx(6, "", AT_EMPTY_PATH, ...)                                    = 0   identity verify
read(6, ..., 4)                                                     = 3   bounded stream
read(6, "", 1)                                                      = 0   EOF check
statx(6, "", AT_EMPTY_PATH, ...)                                    = 0   post-read consistency
```

Every target `openat2` in every trial carried all four required RESOLVE bits; the checker
verified this mechanically per trial and found zero exceptions.

## 4. Results: 41 PASS, 1 BLOCKED

### Static mandatory cases

| Case | Result | Evidence |
|---|---|---|
| Regular nonempty / empty | `observed_file` | Digest equals the independent hashlib oracle |
| Missing leaf / missing parent | `absent` | One resolve, no sibling retry, no reopen |
| Wrong bytes, same name | `observed_file` with the *actual* digest | No comparison or mismatch code anywhere |
| Directory in file position | `wrong_kind`, kind `directory` | Zero `getdents64` |
| Regular file in parent position | `wrong_kind` at resolve, `ENOTDIR` | Never `absent` |
| Permission denied, leaf | `permission_denied` at the **reopen** stage | The `O_PATH` pin and classification succeeded on a mode-000 file; the procfs reopen was refused. Bytes stayed unknown |
| Permission denied, parent search | `permission_denied` at resolve, `EACCES` | Never `absent` |
| Sparse file | `observed_file` | Digest hashes the hole as zeros, matching the oracle; no `SEEK_HOLE` skipping |
| Metadata over limit | `file_limit` at the **budget** stage | **Zero** procfs reopens and zero reads |
| Hardlink | `observed_file`, sampled `nlink` 2 | No alias enumeration, no path outside the root opened |
| Directory metadata | `observed_directory` | Metadata only, no enumeration |

### Symlinks — Amendment 1 confirmed on both variants

| Variant | Result | Mechanism actually observed |
|---|---|---|
| Trailing internal | `symlink_forbidden` at `classify`, kind `symlink` | `openat2` **succeeded**, returning an `O_PATH` descriptor to the link; rejection followed classification |
| Trailing escaping | `symlink_forbidden` at `classify` | Same; destination never resolved, opened or read |
| Trailing dangling | `symlink_forbidden` at `classify` | Same |
| **Non-final component** | `symlink_forbidden` at `resolve`, `ELOOP` | `openat2` returned **no descriptor** |

Amendment 1 is exactly right on both variants, and the original architecture proposal's
insistence that post-open handle classification is mandatory is vindicated: under the
mandated flags the safety of a trailing symlink depends entirely on that classification
step, because resolution does not reject it.

### D1–D3: why the mechanism was chosen, now measured

| Arm | Result |
|---|---|
| **D1** direct `O_RDONLY` on a writerless FIFO | **Blocked for the full 8 s until the supervisor deadline** and was killed. The hazard is real |
| **D2** direct `O_RDONLY\|O_NONBLOCK` | Returned descriptor 4 with `o_path=false` in 0.015 s — a genuine **data-open of a FIFO** |
| **D3** same object via `O_PATH` | Returned descriptor 5 with `o_path=true`, classified `fifo`, **zero reads**, 0.019 s |

This is the empirical justification for the architecture's choice: the rejected strategy
either hangs or data-opens the special file, while the chosen strategy classifies it
without opening it.

### Special files: zero data access, trace-proven

`fifo` and `unix_socket` both ended `special_file` at `classify` with **0 procfs reopens,
0 data reads and 0 `connect` calls**. No character or block device node was created or
opened; unprivileged `mknod` is unavailable and no hazardous device class was touched by
any arm.

### procfs admission

| Case | Result |
|---|---|
| Real procfs | Admitted by filesystem-type check plus the self-identity probe |
| Fake tmpfs decoy directory | `procfs_rejected` before any target read |
| Missing procfs directory | `procfs_unavailable`; **zero** reads, and no pathname fallback |
| **Foreign process** `/proc/1/fd` | `procfs_rejected` — the self-identity probe caught a directory that is genuine procfs but belongs to another process |

The self-identity probe must open the magic link **without** `O_NOFOLLOW`. During
preparation an `O_NOFOLLOW` probe pinned the magic link itself rather than its target and
rejected every admission — the same trailing-symlink semantics that Amendment 1 concerns.

### Races: all injections landed, all outcomes inside the frozen sets

| Case | Seed | Stop | Injected | Outcome | Digest |
|---|---|---|---|---|---|
| Leaf regular → symlink swap | 110001 | `openat2` | yes | `symlink_forbidden` at classify | — |
| Parent → symlink swap | 110002 | `openat2` | yes | `symlink_forbidden` at resolve | — |
| Replacement before pin | 110003 | `openat2` | yes | `observed_file` | `race_new`, in the frozen set |
| Replacement after pin | 110004 | reopen | yes | `observed_file` | `race_new`, in the frozen set |
| Directory replacement after pin | 110005 | `openat2` | yes | `observed_file` | retained-root semantics held |
| Growing file | 110006 | first read | yes | `changed_during_read` | no complete digest claimed |
| Truncating file | 110007 | first read | yes | `changed_during_read` | no complete digest claimed |
| In-place overwrite, deterministic | 110008 | first read | yes | `changed_during_read` | `race_mixed_1` |
| In-place overwrite, undetectable arm | 110009 | first read | yes | `changed_during_read` | `race_mixed_1` |
| Hardlink alias mutation | 110010 | first read | yes | `changed_during_read` | `alias_new`, in the frozen set |

**Ten of ten mandatory injections landed.** No trial was INVALID, so no case fell to
INCONCLUSIVE, and nothing was retried. One forced execution per deterministic schedule was
run, as frozen; the 20-repetition stress allowance was not needed and was not used.

The `mmap`-based undetectable arm was in fact detected here, through the `ctime` change
its writeback produced. The frozen definition permits that: non-detection would have been
a PASS demonstrating the limit, and detection is equally inside the safe set. **No
snapshot claim is made either way** — a digest identifies the bytes actually delivered
through the retained descriptor under the measured sequence, nothing more.

### Batch, budget and privacy

| Case | Result |
|---|---|
| Two-file changing environment | Both targets observed; mixed sequential capture representable |
| Budget exhaustion | `budgeta` `observed_file`, `budgetb` `total_limit`. Earlier result retained, **every declared target has an explicit outcome** |
| Private marker and unlisted file | Only the listed target was opened; zero opens naming the private or unlisted objects; no absolute path in any output stream |
| Descriptor-shaped plan field | Plan validation error with **no target I/O** — no `openat2`, no reopen, no target read. Only the untrusted plan file itself was read, which is inherent to validating it |

### Mount

| Arm | Result |
|---|---|
| **Non-namespace `NO_XDEV` fallback** (mandatory) | `mount_crossing` from `EXDEV` resolving `1001` under a `/run/user` root; **no descriptor produced** |
| **Control arm without `NO_XDEV`** | The same resolution **succeeded** (`observed_directory`), with the other three RESOLVE bits still set |

The control arm is what makes the fallback meaningful: it proves `RESOLVE_NO_XDEV`
specifically caused the rejection, rather than some unrelated failure.

## 5. Descendant bind mount: BLOCKED, residual claim carried

`kernel.apparmor_restrict_unprivileged_userns` is `1` and `unshare -Urm true` fails
writing `/proc/self/uid_map`. Creating a private user and mount namespace is therefore
unavailable to the unprivileged lab user. This was **not** worked around: no sudo, no
sysctl change, no AppArmor profile, no retry with privilege.

Exactly one claim remains **unverified**:

> `RESOLVE_NO_XDEV` rejects a bind mount created as a descendant of an authorized ext4
> root.

General mount-traversal rejection **is** evidenced, by the fallback arm and its control.
This report does **not** claim that descendant bind mounts were validated, and any future
use that depends on them stays outside what this evidence supports.

## 6. Resource accounting, isolation and preservation

Whole run 8 seconds of wall clock, far inside the 10-minute allowance. Fixture allocation
stayed trivial: the over-limit fixture is sparse, with an apparent size of 512 MiB plus
4 KiB and effectively no blocks, so the 1 GiB fixture and 5 GiB total budgets were never
approached. Host free space went 98.07 GiB before boot to 98.27 GiB after shutdown; guest
free space was 12,909 MiB before the run.

**A0 isolation.** The A0 tree exists at `<HOME>/exp009-a0-7zip` and was not traversed,
listed, read, hashed or modified. No Wine or 7-Zip process was started. All fixtures live
in a separate disposable directory. **A0-7ZIP remains experimental FAIL.**

**Privacy.** The published evidence replaces the lab work directory with `<WORK>` and the
home directory with `<HOME>`. `/proc/self/fd`, `/proc/1/fd` and `/run/user` are retained
verbatim because they are the subject of the procfs-admission and mount-crossing cases.
No guest address, SSH key, credential or raw private provenance is published; those stay
in the private provenance directory outside the repository. Diagnostics use logical target
IDs and relative scoped paths.

VM shut down gracefully: state **Off**, **0 checkpoints**, unchanged from before. No VM,
WSL lab, branch or private provenance was deleted.

## 7. Verdict and what it does not establish

Applying the frozen precedence — FAIL, else BLOCKED, else INCONCLUSIVE, else PASS — no
valid mandatory trial violated a required property, every mandatory case executed and was
interpretable, and every mandatory injection landed. **OBS-FS-01: PASS**, with
`bind_mount_descendant_namespace` reported separately as BLOCKED.

This is evidence for one cohort only: Linux `7.0.0-31-generic` on x86_64, local ext4
mounted `rw,relatime`, gcc 13.3.0 static binaries, unprivileged UID 1001, and this
specific `O_PATH` plus procfs-reopen mechanism. It does **not** establish arbitrary Linux
filesystem safety, network or FUSE behaviour, future kernel behaviour, Windows semantics,
atomic file or environment snapshots, runtime provenance, or safe execution. Actual block
devices, hostile or privileged mounts and non-x86_64 Linux remain untested.

**A PASS does not accept ADR-0022 and does not authorise implementation.** Both remain
separate owner decisions. The separately tracked implementation-test obligations — exact
plan SHA binding, root logical-ID and count binding, and rejection of an
authorized-then-substituted plan — are not OBS-FS-01 cases and remain unexecuted.

Full sanitized per-trial results, traces and expected digests:
[obs-fs-01-report.json](evidence/obs-fs-01-2026-09-08/obs-fs-01-report.json).
Execution definition sources: [obs-fs-01/](obs-fs-01/).
