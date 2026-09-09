# LAUNCH-EXEC-01 build evidence

**Append-only.** One section per build. Nothing here is ever edited or removed, and **no binary
digest is ever inserted retroactively into an earlier commit**.

This file exists because `SOURCE-HASHES.json` is **immutable once written**: recording binary
digests inside it would change the file, and therefore its own digest, so the artefact that
certifies the freeze would no longer be the artefact that was frozen.

> **No binary recorded here has ever been executed.** Every property below was established by
> compiling and by reading the produced files as **data** — `file`, `readelf`, `sha256sum`,
> `grep`. LAUNCH-EXEC-01 remains **NOT_RUN**, D-7 is not granted, and no preregistered case has
> been posed.

## Build 1 — 2026-09-09, compile-only pre-trial review

**Purpose:** the frozen C sources had never been compiled on Linux. The independent pre-trial
review must not recommend D-7 without real compile evidence for the exact frozen sources.

| Field | Value |
|---|---|
| Workflow | `.github/workflows/launch-exec-01-compile-only.yml`, run `34385261457` |
| Commit built | `8bd1ea00d5ef9153d57e9cd49152649457941ef4` |
| Runner | GitHub-hosted `ubuntu-24.04` (`runnervmejwal`) |
| OS | Ubuntu 24.04.4 LTS (noble) |
| Kernel | `6.17.0-1022-azure`, `x86_64` |
| Compiler | `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` |
| C library | `GLIBC 2.39-0ubuntu8.8` |
| euid | `1001` (unprivileged) |
| Page size | 4096 |
| Source freeze verified before building | **yes** — `freeze_verified: true` |

**Link commands, exactly as frozen.**

```
cc -O2 -Wall -Wextra -static -o build/<target> <target>.c
    for helper_report, helper_alt, helper_fork, helper_setid, launcher_spike
cc -O2 -Wall -Wextra -o build/helper_dynamic helper_dynamic.c
python3 make_fixtures.py build
```

**Link result: all six targets built. Zero warnings and zero errors at `-Wall -Wextra`.** No
source was modified to make them build, and no dynamic build was substituted for a static one.

**Static-link gate:** the probe linked statically and `ldd` reported `not a dynamic executable`,
so the gate passed and no case is BLOCKED for a missing static toolchain.

**Inspection, as data only — never executed.**

| Binary | `file` | PT_INTERP |
|---|---|---|
| `helper_report` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_alt` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_fork` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_setid` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `launcher_spike` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_dynamic` | ELF 64-bit LSB **pie**, x86-64, **dynamically linked** | `/lib64/ld-linux-x86-64.so.2` |

Both assertions held: no static target carries a `PT_INTERP`, and `helper_dynamic` does — which
is what case **E7** requires, since E7 exists to expose the loader activity every other helper
excludes. The **E6 marker guard** (`HELM-MARK`) was located in the built `helper_report` image,
so E6's length-preserving in-place mutation has a findable fixed-length target.

**SHA-256 of every produced binary and fixture.**

```
423ac81e0cbf553a3d8f821d72209ed6f505f4a45494c38d800bfdfc126ca236  helper_report
d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d  helper_alt
3232d09ffb089907e2586ac0bffdd1ba5b635c99e80cab1333e4ddb575a13dcb  helper_fork
500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10  helper_setid
d8392a4577359ae162bf10c26e997ca63c30158ceea2032cebefcaea65c6f8bf  launcher_spike
c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc  helper_dynamic
415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52  helper_foreign.elf
51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9  unloadable_in_cohort.elf
3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359  magic_only.bin
37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a  script_fixture.sh
```

These digests are **not reproducible across toolchains**: the build embeds a BuildID and is
sensitive to compiler and libc version. They identify what this runner produced from this source
freeze, which is what a preflight record is for; they are not a claim of deterministic builds.

### Environment facts this build established

Recorded because several of them decide whether a **conditional** case is posable, and because
the pre-trial review found that assuming any of them would let a host property masquerade as a
mechanism defect.

| Fact | Observed | Consequence |
|---|---|---|
| parent `NoNewPrivs` | **0** | **N2 is posable** on this runner: the control arm can observe 0, so N1's positive result is controlled rather than uncontrolled |
| `Seccomp` / `Seccomp_filters` | **0 / 0** | no filter that could reject a syscall on a supporting kernel |
| `clone3` probe | **available**, `EINVAL` from a zero-size argument | implemented and reached without creating a child, so **M3 is posable** |
| `strace` | **`/usr/bin/strace`, present** | the seven traced cases are posable. This **falsifies** the definition's earlier claim, inferred from the runner-image manifest, that `strace` is absent; the definition is corrected |
| `ptrace_scope` | **1** | a parent-side tracer would also be permitted |
| `binfmt_misc` | **enabled**, 4 registrations: `llvm-16/17/18-runtime` (magic `4243`), `python3.12` (magic `cb0d0d0a`) | **none matches `\x7fELF`**, so this runner cannot pose the foreign-interpreter hazard; E8 still tests that admission refuses a foreign-architecture object |
| noexec mounts | `/dev`, `/proc`, `/sys`, `/proc/sys/fs/binfmt_misc` | none is an unprivileged-writable location for a fixture, so **X8 should be BLOCKED** here. The derivation was corrected to test writability rather than mere existence |
| derived block reasons | `unprivileged_runner` only | N3 is BLOCKED by construction, as designed |

### What failed, and why it is not a mechanism finding

The step **"Non-trial Python and checker tests"** failed, and the subsequent
**"Runner must refuse to pose a case"** step was skipped as a consequence.

**Every one of the 85 tests belonging to this experiment passed**, including the whole checker
algebra, the manifest integrity checks and the prose-matches-manifest check. The single failure
was `test_helm_app_spec_fixture`, a **pre-existing repository test unrelated to LAUNCH-EXEC-01**,
which resolves a blob at a named commit and errored with `fatal: path ... exists on disk, but not
in <sha>` because the workflow used `actions/checkout`'s default **shallow** clone.

That is a defect in the review workflow, not in the frozen sources, and per the correction
boundary it is **a PRETRIAL finding and not a mechanism verdict**: no preregistered case ran, so
no experiment verdict of any kind is produced. The workflow now checks out with `fetch-depth: 0`.
The runner-refusal assertion was re-established locally in the meantime (`exit 3`, "D-7
(execution authorisation) is not granted. … No case was posed.").

### Relationship to the current source freeze

The C sources built here are **byte-identical** to those in the current freeze: the corrections
made after this build touched `harness.py`, the workflow and documentation only. The binary
digests above therefore remain valid evidence for the frozen C sources, and
`SOURCE-HASHES.json` records the source digests they were produced from.
