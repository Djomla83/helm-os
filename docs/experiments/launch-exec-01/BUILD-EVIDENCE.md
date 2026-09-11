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

## Build 2 — 2026-09-09, confirming run after the workflow correction

**Purpose:** Build 1's job failed at a step unrelated to the experiment, so the runner-refusal
assertion never ran and the record was incomplete. This run repeats the whole compile-only job
with `fetch-depth: 0`.

| Field | Value |
|---|---|
| Workflow run | `34385841542` |
| Commit built | `5729458f48a10436167db0f915e2ee1bcf634c64` |
| Runner / OS / kernel / toolchain | as Build 1: `ubuntu-24.04`, Ubuntu 24.04.4, `6.17.0-1022-azure` x86_64, `cc 13.3.0`, glibc 2.39 |
| Result | **every step succeeded**, including the two the first run never reached |

**All 85 non-trial Python and checker tests passed on Linux**, and the **runner-refusal assertion
now ran and held**: `runner exit code: 3`, with `"status": "NOT_RUN"` and the reason `D-7
(execution authorisation) is not granted. … No case was posed.` Still **no binary was executed**
and **no preregistered case was posed**.

**The produced binary digests are byte-identical to Build 1**, for all six binaries and all four
fixtures. That is the independent confirmation of the claim recorded at the end of Build 1: the
corrections made between the two commits touched `harness.py`, the workflow and documentation
only, and **not one C source byte changed**. The Build 1 digests therefore remain valid evidence
for the currently frozen C sources, and the two builds corroborate each other.

## Build 3 — 2026-09-09, pre-trial validation of the case driver

**Purpose:** the case-posing driver, the P-12 mapping and the P-14 sanitiser were implemented. This
section records what was validated and, equally, what was **not rebuilt and why**.

> **No binary was compiled, produced or executed in this section.** Nothing was run but Python.
> LAUNCH-EXEC-01 remains **NOT_RUN**, D-7 is not granted, and no preregistered case has been posed.

### No C source changed, so Builds 1 and 2 remain the compile evidence

The driver work touched Python and documentation only. Verified by digest rather than by claim —
all six C sources are byte-identical to the ones Build 1 and Build 2 compiled, and still equal the
digests the superseded manifest recorded:

```
7993aa631abdca6796171fe4ffafb2d296f2a00b61ea2322ee8e624731b99113  helper_alt.c
b3c398d840c16dfa4e0dbc56593b905b4019bfe2316fc339049b8a69930350a0  helper_dynamic.c
a62a6a1e2bbb9cc6ef8a346ae9322c7b7435b1c7f0b6c3710b3aadb9cc2ba0b0  helper_fork.c
99770ed8cfdd4049f9bef3624e21850496b6c14a169c3895b54ab7a858d54711  helper_report.c
c609755d105122eea304f5fe12685d8dcc24064d77659295a11a76d42b86329f  helper_setid.c
743d3ddc59577c5abff5606b72a7756a6d5e40560e69064890415902d18556d9  launcher_spike.c
```

`git diff --stat 262a983 -- '*.c'` is empty. The Build 1 binary digests therefore remain valid
evidence for the currently frozen C, and **no new compile run is required to keep them valid**.

### Local Linux compile capability, recorded honestly

A fresh compile was attempted anyway and **could not be performed locally**. The ordinary personal
`Ubuntu` WSL distribution has **no C compiler**: `gcc`, `cc`, `clang`, `gcc-13` and `musl-gcc` are
all absent, and only `gcc-14-base` is installed — the base package, not the compiler driver.
Installing one is **not authorised** for this task, and the preserved `helm-lab-g0*` distributions
were not touched, entered or modified. `readelf`, `objdump`, `sha256sum` and `file` are present,
but with nothing newly built there was nothing to inspect. **No generated ELF was `ldd`'d,
executed, traced or loaded.**

### What was validated, on Linux

| Check | Environment | Result |
|---|---|---|
| Byte-compile every experiment module | Ubuntu 24.04 WSL2, kernel `6.6.87.2-microsoft-standard-WSL2` x86_64, Python 3.12.3 | **pass**, syntax only, no import |
| Full non-trial suite | as above | **201 tests, all pass** |
| Driver suite alone | as above | **116 tests, all pass** |
| Driver completeness | as above | `complete: true`, 72/72, 0 missing, 0 duplicate, 0 unknown |
| Unposable-case analysis | as above | **43** cases, computed statically |
| Runner must refuse to pose a case | as above | **exit 3**, `"status": "NOT_RUN"` |

The same 201 tests also pass on the Windows host. Every observation in the suite is fabricated
in-process; no test constructs a `driver.Authorisation`, and a test asserts that the suite file
contains no such construction.

### The finding this validation produced

Implementing the driver established that **`launcher_spike.c` never emits the capture prefix it
retains** — it is `malloc`'d, filled, and `free()`d at the end of `main`. The helper report is
therefore produced inside the launcher and no channel carries it out, which makes 40 cases
unposable, plus M2, M3 and M5 for separate reasons. Full analysis in
[section 10 of the pre-trial review](../../implementation/HELM-LAUNCH-INDEPENDENT-PRETRIAL-REVIEW.md#10-post-review-note-the-case-driver-was-implemented-still-not_run).

**Successful validation does not constitute trial evidence.** No preregistered case ran, so no
experiment verdict of any kind exists.

## Build 4 — 2026-09-09, first compile of the PRE-D7-B1 correction (defect found)

**Purpose:** `launcher_spike.c` changed for the first time since the freeze — PRE-D7-B1 (emit the
retained capture prefix) plus M2's and M5's TEST/CONTROL-ONLY arms — so Build 1 and Build 2 no
longer cover the mechanism source. Fresh Linux compile-only evidence was required.

> **No binary produced here was executed, traced, `ldd`'d or loaded.** Every property below came
> from compiling and from reading the produced files as data. LAUNCH-EXEC-01 remains **NOT_RUN**,
> D-7 is not granted, and no preregistered case was posed.

| Field | Value |
|---|---|
| Workflow run | `34408977185`, `.github/workflows/launch-exec-01-compile-only.yml` |
| Commit built | `a7ae6b9bc44b6db58fd4474fb617ef132e9bc664` |
| Trigger | `push` to `docs/helm-launch-architecture` (path filter), not a dummy change |
| Runner | GitHub-hosted `ubuntu-24.04` (`runnervmejwal`) |
| Kernel | `6.17.0-1022-azure`, `x86_64` |
| Compiler | `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` |
| C library | `GLIBC 2.39-0ubuntu8.8` |
| Source freeze verified before building | **yes** — `freeze_verified: true` |
| Job conclusion | **success** (every step, including the runner-refusal assertion) |

**Local compile capability, again:** the ordinary personal `Ubuntu` WSL distribution still has no C
compiler (`gcc`, `cc`, `clang`, `gcc-13`, `musl-gcc` all absent; only `gcc-14-base`). Installing one
is not authorised and the preserved `helm-lab-g0*` distributions were not touched. The bounded
GitHub compile-only workflow was used on the exact candidate ref, which is what it exists for.

### Link commands

```
cc -O2 -Wall -Wextra -static -o build/<t> <t>.c
    for helper_report, helper_alt, helper_fork, helper_setid
cc -O2 -Wall -Wextra -static -pthread -o build/launcher_spike launcher_spike.c
cc -O2 -Wall -Wextra -o build/helper_dynamic helper_dynamic.c
```

`-pthread` is new and applies to `launcher_spike` alone: M2's control arm starts extra live threads
in the launcher. **It linked statically**, so a static binary with pthread is not a problem on this
toolchain. The four helpers still build with the frozen command exactly as before.

### The defect this build found

**The job succeeded, and it should not be read as a clean result.** The compiler emitted a real
warning, and the standing claim for this experiment has been *zero* warnings at `-Wall -Wextra`:

```
launcher_spike.c: In function 'main':
launcher_spike.c:871:12: warning: format '%d' expects a matching 'int' argument [-Wformat=]
  882 |            "\"wait_errno\":%d,"
```

Splitting the single receipt `printf` into a sequence — needed so the base64 capture prefix could be
streamed out — left **`"wait_errno":%d,` duplicated**. The second `%d` therefore read a variadic
argument that was never passed: undefined behaviour, emitting a garbage duplicate key into every
accepted receipt.

This is a **genuine source defect, not a build-environment artefact**. It is recorded here rather
than quietly rebuilt, and it is corrected in a descendant commit with its own build section. Warnings
do not fail this workflow, which is why the job is green above and why the green must not be taken
as the whole answer.

### Binary digests from this build

Recorded because they establish exactly which helpers did and did not change.

```
57e233b303c8727a55748d76eb52de31471f8f545af4bf7bbfe675ffdacd8739  launcher_spike   (CHANGED)
423ac81e0cbf553a3d8f821d72209ed6f505f4a45494c38d800bfdfc126ca236  helper_report    (unchanged)
d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d  helper_alt       (unchanged)
3232d09ffb089907e2586ac0bffdd1ba5b635c99e80cab1333e4ddb575a13dcb  helper_fork      (unchanged)
500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10  helper_setid     (unchanged)
c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc  helper_dynamic   (unchanged)
415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52  helper_foreign.elf
51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9  unloadable_in_cohort.elf
3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359  magic_only.bin
37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a  script_fixture.sh
```

**Only `launcher_spike` changed**, from `d8392a45…` at Builds 1 and 2 to `57e233b3…` here. All five
helper digests and all four fixture digests are byte-identical to Build 1 — independent confirmation
that the correction touched the launcher alone and left `helper_report.c`'s reporting channel exactly
as the owner required.

### Inspection, as data only

| Binary | `file` | PT_INTERP |
|---|---|---|
| `launcher_spike` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_report` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_alt` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_fork` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_setid` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_dynamic` | ELF 64-bit LSB **pie**, x86-64, **dynamically linked** | present |

Both frozen assertions still hold: no static target carries a `PT_INTERP` — `-pthread` did not make
`launcher_spike` dynamic — and `helper_dynamic` does, which is what E7 requires. The E6 marker guard
is still locatable in the built `helper_report`.

### Other steps

**All 247 non-trial Python and checker tests passed on the runner**, 138 more than the previous
build, and the **runner-refusal assertion held at exit 3** with `"status": "NOT_RUN"`. The freeze
verified on the runner *before* anything was built, which also confirms the manifest hashes are over
the LF bytes a fresh checkout produces.

## Build 5 — 2026-09-09, the corrected PRE-D7-B1 source, clean

**Purpose:** Build 4 surfaced a genuine source defect — the duplicated `wait_errno` conversion.
This run compiles the descendant commit that corrects it. Build 4 is preserved above; this section
does not replace it.

> **No binary produced here was executed, traced, `ldd`'d or loaded.** LAUNCH-EXEC-01 remains
> **NOT_RUN**, D-7 is not granted, and no preregistered case was posed.

| Field | Value |
|---|---|
| Workflow run | `34409337399` |
| Commit built | `002e1e4fe72dce9a141e0dcb5ee85f339bb4a560` |
| Runner / OS / kernel | GitHub-hosted `ubuntu-24.04`, `6.17.0-1022-azure` x86_64 |
| Compiler / libc | `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`, `GLIBC 2.39-0ubuntu8.8` |
| Source freeze verified before building | **yes** — `freeze_verified: true` |
| Result | **every step succeeded** |

**Zero warnings and zero errors at `-Wall -Wextra`.** The `-Wformat=` diagnostic quoted in Build 4
is gone, and no other diagnostic replaced it, so the experiment's standing "zero warnings" claim
holds again for the current frozen source.

### Binary and fixture digests

```
f7548af2574c52f7172e3035a99ffec3cf88535aa0667ab62c87b96a1307be09  launcher_spike   (CHANGED)
423ac81e0cbf553a3d8f821d72209ed6f505f4a45494c38d800bfdfc126ca236  helper_report    (unchanged)
d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d  helper_alt       (unchanged)
3232d09ffb089907e2586ac0bffdd1ba5b635c99e80cab1333e4ddb575a13dcb  helper_fork      (unchanged)
500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10  helper_setid     (unchanged)
c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc  helper_dynamic   (unchanged)
415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52  helper_foreign.elf
51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9  unloadable_in_cohort.elf
3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359  magic_only.bin
37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a  script_fixture.sh
```

`launcher_spike` moved from `57e233b3…` (Build 4) to `f7548af2…`, which is the one-line correction.
**Every other digest is byte-identical across Builds 1, 2, 4 and 5** — four independent builds
agreeing that only the launcher changed and that `helper_report.c`'s reporting channel was never
touched, which is what the owner decision required.

These digests are **not reproducible across toolchains**: the build embeds a BuildID and is
sensitive to compiler and libc version. They identify what this runner produced from this source
freeze.

### Inspection, as data only

| Binary | `file` | PT_INTERP |
|---|---|---|
| `launcher_spike` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_report` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_alt` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_fork` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_setid` | ELF 64-bit LSB executable, x86-64, **statically linked** | none |
| `helper_dynamic` | ELF 64-bit LSB **pie**, x86-64, **dynamically linked** | present |

`-pthread` did not cost the static-linking precondition: `launcher_spike` is still statically linked
and still carries no `PT_INTERP`, so M2's control arm did not weaken the condition that makes a
complete trace evidence rather than filtering. `helper_dynamic` still carries one, which is what E7
requires. The E6 marker guard is still locatable in the built `helper_report`.

### Other steps

**All 247 non-trial Python and checker tests passed on the runner**, and the **runner-refusal
assertion held at exit 3** with `"status": "NOT_RUN"`. The freeze verified on the runner *before*
anything was built, which also confirms the manifest hashes are computed over the LF bytes a fresh
checkout produces rather than over a local working copy.

**Successful compilation is not trial evidence.** No preregistered case ran, so no experiment
verdict of any kind exists.

## Build 6 — 2026-09-10, the Trial #2 C source with the post-pin control hook, clean

Build 5 no longer covers `launcher_spike.c`: the TEST/CONTROL-ONLY post-pin barrier changed it, so
this is the first Linux compile of the Trial #2 source. Nothing was executed.

| | |
|---|---|
| Commit built | `53c00393eb4ff5471042b9c9860f42fc29bab047` |
| Frozen candidate | `e4f49f2bdb77c78cb57354584bc78f08a50531ea` (freeze verification `true` on the runner) |
| Workflow run | `34525876757`, job `103034518998`, **success** |
| Runner | GitHub-hosted `ubuntu-24.04` |
| Kernel / arch | `6.17.0-1022-azure`, `x86_64` |
| Compiler | `gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` |
| C library | `ldd (Ubuntu GLIBC 2.39-0ubuntu8.8) 2.39` |
| Link command | `cc -O2 -Wall -Wextra -static -pthread -o build/launcher_spike launcher_spike.c` |
| Compiler diagnostics | **none** — no warning and no error under `-Wall -Wextra` |

The only two `warning:`/`error:` strings in the whole log are `ldd` broken-pipe messages from the
preflight probe's `| head -1`, not compiler output.

### Produced artefacts

```
d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d  build/helper_alt
c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc  build/helper_dynamic
415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52  build/helper_foreign.elf
3232d09ffb089907e2586ac0bffdd1ba5b635c99e80cab1333e4ddb575a13dcb  build/helper_fork
423ac81e0cbf553a3d8f821d72209ed6f505f4a45494c38d800bfdfc126ca236  build/helper_report
500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10  build/helper_setid
9e9e5803f5e9f3bbf435758ae5ec79349ae80eacc6c1a3f34c3134754cd8e7ba  build/launcher_spike
3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359  build/magic_only.bin
37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a  build/script_fixture.sh
51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9  build/unloadable_in_cohort.elf
```

These are compile-only artefacts from a disposable runner. They are **not** Trial #2 build identity:
the trial hashes the files it will actually consume, in its own run, before its first case.

### Static/dynamic inspection, by `readelf`, never by execution

The five static targets each report **no PT_INTERP**; `helper_dynamic` has one, which is what E7
requires. `helper_report` still carries the guarded marker region E6 locates.

### Nothing ran

No produced ELF was executed: the log contains no `set -x` trace of `build/<binary>` as a command,
only `file` and `readelf` reading them as data. `--post-pin-control-fd` appears **zero** times — the
new hook is inactive without it, and CI never passes it. The runner was invoked without D-7 and
exited **3**. The non-trial suite ran **500 tests, OK**, with none skipped on Linux.

**LAUNCH-EXEC-01 Trial #2 remains NOT_RUN. D-7 is not authorised. The valid trial count is ZERO.**

## Build 6 no longer covers `launcher_spike.c` — 2026-09-11, the Trial #2 delta correction

The correction that followed the
[bounded delta review](../../implementation/HELM-LAUNCH-EXEC-01-TRIAL-002-DELTA-REVIEW.md) changed
`launcher_spike.c`: two TEST/CONTROL-ONLY caller-state flags, `--parent-fd-set-cloexec` (F3) and
`--parent-close-low-fds` (F6, F7), the receipt-channel restore they need, and the O5
`poll_returns_not_in_receipt` counter. **Build 6's binary hashes are not the new source's.** No
helper source changed.

The new source has **not** been compiled here: this host has no C compiler and installing one is not
authorised. Fresh Linux compile-only evidence — Build 7, from the existing compile-only workflow,
with nothing executed — is required before any new D-7 decision.

**LAUNCH-EXEC-01 Trial #2 remains NOT_RUN. D-7 is not authorised. The valid trial count is ZERO.**

## Build 7 — 2026-09-11, the Trial #2 delta-corrected candidate, clean

The first Linux compile of `launcher_spike.c` as changed by the delta correction, from the one
owner-authorised fast-forward push `53c0039..f417984`. This is **not** D-7. Nothing was executed.

| | |
|---|---|
| Commit built | `f41798455662579c3895f7a4af50bac17d47bb43` |
| Source binding | the checkout step ran `git log -1 --format=%H` on the runner and printed `f41798455662579c3895f7a4af50bac17d47bb43`; `--verify-freeze` then reported `freeze_verified: true` against that commit's own manifest, which hashes `launcher_spike.c` as `0a45447b627f16dce2fd7193e26993dcc126ae96dce6e6c5b511b41361e0d622` |
| Frozen candidate | `f417984`, manifest blob `4a7dfbae013afd5876ffe309bd8aed2dcd761efd` |
| Workflow run | `34575558065`, attempt 1, job step list all **success** |
| Runner | GitHub-hosted `ubuntu-24.04`, image version `20260907.300.1`, Ubuntu 24.04.5 LTS |
| Kernel / arch | `6.17.0-1022-azure`, `x86_64` |
| Compiler | `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0` |
| C library | `ldd (Ubuntu GLIBC 2.39-0ubuntu8.8) 2.39` |
| Static command | `cc -O2 -Wall -Wextra -static -o build/<t> <t>.c` for helper_report, helper_alt, helper_fork, helper_setid |
| Spike command | `cc -O2 -Wall -Wextra -static -pthread -o build/launcher_spike launcher_spike.c` |
| Dynamic command | `cc -O2 -Wall -Wextra -o build/helper_dynamic helper_dynamic.c` |
| Compiler diagnostics | **none** — no warning and no error under `-Wall -Wextra` |

The only two `error` strings in the whole log are `ldd` broken-pipe messages from the environment
inventory's `ldd --version | head -1`, not compiler output.

### Produced artefacts

| SHA-256 | Size (bytes) | Artefact | Classification |
|---|---|---|---|
| `d13082b12b26cb35d6707f6575feaab3b17368a13b6fd0ccf6ed7ae10c98e26d` | not emitted | `helper_alt` | ELF, static, no PT_INTERP — unchanged since Build 6 |
| `c854162ddf4074cdfe132492ddc595ae6345f820c22a1cd082e32dd1646850cc` | not emitted | `helper_dynamic` | ELF PIE, dynamic, PT_INTERP `/lib64/ld-linux-x86-64.so.2` — unchanged |
| `415f14b8538c8e59832a107b12c54f2fae9cd2ab5637df2f8a6facb82a9cfd52` | 64 | `helper_foreign.elf` | generated fixture — unchanged |
| `3232d09ffb089907e2586ac0bffdd1ba5b635c99e80cab1333e4ddb575a13dcb` | not emitted | `helper_fork` | ELF, static, no PT_INTERP — unchanged |
| `423ac81e0cbf553a3d8f821d72209ed6f505f4a45494c38d800bfdfc126ca236` | not emitted | `helper_report` | ELF, static, no PT_INTERP; E6 marker guard present — unchanged |
| `500c4af1934be000acefc6daa49ebe8ac984686b23a4358bd134464e02f66c10` | not emitted | `helper_setid` | ELF, static, no PT_INTERP — unchanged |
| `4d42212f3b4d45ca46e415961e36e8ee4d45090a15fb3b77a6f8971e661a6546` | not emitted | `launcher_spike` | ELF, static, no PT_INTERP — **new**; Build 6 was `9e9e5803…` |
| `3bdbb4fe8397cd2b842430b39ccff01a8663c751945ef5e9a09e267fb8b1d359` | 4 | `magic_only.bin` | generated fixture — unchanged |
| `37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a` | 490 | `script_fixture.sh` | generated fixture — unchanged |
| `51224867e5fb13d0c6052397c9f4959c7c87bb8bc7d750c91429728a18b507d9` | 64 | `unloadable_in_cohort.elf` | generated fixture — unchanged |

**Sizes.** The compile-only workflow prints SHA-256 values and never sizes, so no compiled binary's
size exists in this run's record, and none is invented here. The four fixture sizes were recomputed
from `make_fixtures.FIXTURES` at `f417984` without building or executing anything, and those same
bytes hash to exactly the digests above. A trial records every artefact's size in its own
build identity, before its first case.

Only `launcher_spike` changed, which is what the correction predicts: it is the only C source the
correction touched. These are compile-only artefacts from a disposable runner and are **not** Trial #2
build identity.

### Nothing ran

The complete set of `set -x` command verbs in the log is `cat`, `cc`, `command`, `echo`, `file`,
`for`, `getconf`, `grep`, `head`, `id`, `ldd`, `mkdir`, `printf`, `readelf`, `sha256sum`, `sort`,
`true` and `uname`; `ldd` ran on the static-link probe `/tmp/probe`, not on an experiment binary. Every
`build/` path appears only as data, in `for` lists, `file`, `readelf` and `sha256sum`.
`--post-pin-control-fd`, `--parent-fd-set-cloexec`, `--parent-close-low-fds` and the D-7 flag appear
**zero** times. The five static targets have no PT_INTERP and `helper_dynamic` has one. The non-trial
suite ran **562 tests, OK, none skipped**, and the runner, invoked without D-7, exited **3** with
`NOT_RUN`. The Rust workspace run on the same commit, `34575558173`, also succeeded.

**LAUNCH-EXEC-01 Trial #2 remains NOT_RUN. D-7 is not authorised. The valid trial count is ZERO.**

## Build 7 still binds the final classification freeze — 2026-09-11, no compiler run

The final classification freeze that follows `82b8746` supersedes `f417984`. It changes only
Python sources, the definition and the manifest. That covers the classification correction
`f9bbf39` and the owner's O7 decision, which runs O7 on the existing `helper_fork` fixture. Every
C source it hashes is byte-identical to the one Build 7 compiled at `f417984`:

| Source | SHA-256 at `f417984` and at the new freeze |
|---|---|
| `launcher_spike.c` | `0a45447b627f16dce2fd7193e26993dcc126ae96dce6e6c5b511b41361e0d622` |
| `helper_report.c` | `99770ed8cfdd4049f9bef3624e21850496b6c14a169c3895b54ab7a858d54711` |
| `helper_alt.c` | `7993aa631abdca6796171fe4ffafb2d296f2a00b61ea2322ee8e624731b99113` |
| `helper_fork.c` | `a62a6a1e2bbb9cc6ef8a346ae9322c7b7435b1c7f0b6c3710b3aadb9cc2ba0b0` |
| `helper_setid.c` | `c609755d105122eea304f5fe12685d8dcc24064d77659295a11a76d42b86329f` |
| `helper_dynamic.c` | `b3c398d840c16dfa4e0dbc56593b905b4019bfe2316fc339049b8a69930350a0` |

The build commands in `harness.py` and the fixtures in `make_fixtures.py` are byte-identical as
well. Build 7 (run `34575558065`) is therefore the compile-only evidence for this freeze, and no
Build 8 is required. No compiler ran for this record. A trial records its own build identity
before its first case.

**LAUNCH-EXEC-01 Trial #2 remains NOT_RUN. D-7 is not authorised. The valid trial count is ZERO.**
