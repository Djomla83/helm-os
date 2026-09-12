# LAUNCH-EXEC-01 — Trial #2 bounded postmortem diagnostics

**THIS DOCUMENT DOES NOT MODIFY TRIAL #2.**

Trial #2 remains the one completed execution in GitHub run `34640280964` of freeze
`ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`: 59 PASS / 6 FAIL / 6 INVALID / 1 BLOCKED,
aggregate `MECHANISM_REJECTED`. Its D-7 is consumed, its valid trial count is one, it must not be
rerun, and no Trial #3 or Trial #3 D-7 exists.

This is the single bounded pre-correction diagnostic pass authorised by the
[owner's postmortem decisions](../DECISIONS.md#trial-002-postmortem-decisions). It inspected E6/E6c
statically and ran one isolated Python `fork`/`waitid` process probe for R3. It changed no frozen
source, evidence, result status, checker, case definition, workflow or result review.

## 1. Owner decisions carried forward

* `TRIAL3_CORRECTION_SCOPE_BOUNDED_TO_TRIAL2_FINDINGS`
* `S4_CONSERVATIVE_EXEC_EVIDENCE_POLICY_SELECTED`
* `E6C_REMAINS_RECORDED_AND_IN_SCOPE`
* `P1_P2_P4_REQUIRE_VALID_LIVENESS_REVALIDATION`
* `N3_PRIVILEGED_TEST_DEFERRED`

Clean exec-status EOF remains insufficient proof of exec. E6c stays recorded and its runtime
shared-mapping result is not pre-answered here. The historical P1/P2/P4 statuses remain PASS but
are not positive architectural liveness evidence. N3 remains an allowed conditional environment
BLOCK until a future controlled privileged environment is separately authorised.

## 2. Settled correction targets — no further diagnosis in this pass

| Cases | Settled prospective requirement |
|---|---|
| X2b / X2c / X4 | `X2b/X2c/X4_FIXTURE_EXEC_MODE_CORRECTION_REQUIRED`: generated execution fixtures must receive executable mode before use |
| T1 | `T1_FROZEN_EXPECTATION_CORRECTION_REQUIRED`: a future expectation must use the qualified timeout result rather than frozen Trial #2's unreachable bare `TimedOut` |
| S4 | `S4_CONSERVATIVE_EXEC_EVIDENCE_POLICY_SELECTED`: rewrite the future contract around independent positive test evidence; do not teach the launcher that clean EOF proves exec |
| M2 | `M2_PROCESS_CLONE_CORRELATION_CORRECTION_REQUIRED`: select and correlate the direct-child `CLONE_PIDFD` clone rather than the first pthread `clone3` |
| E4 | `E4_PATH_CORRECTION_REQUIRED`: anchor or resolve the symlink target so it cannot self-nest as a relative target |
| O6 / O7 | `O6_O7_ABSOLUTE_FIFO_CORRECTION_REQUIRED`: pass a canonical absolute case-private FIFO path and make helper-side open/write failure diagnosable |
| P1 / P2 / P4 | `P1_P2_P4_LIVENESS_REVALIDATION_REQUIRED`: re-observe with the corrected liveness fixture in a future valid trial |

No correction above was implemented.

## 3. E6/E6c marker-location static diagnostic

### 3.1 Method and environment

The exact `helper_report.c` and `driver.py` blobs were extracted with `git archive` from
`ba41a3f12be411058ed50e78bcd1c7e22afb7ae4`. The helper source had SHA-256
`99770ed8cfdd4049f9bef3624e21850496b6c14a169c3895b54ab7a858d54711` and Git blob
`af9af5e5d1cabbfe302dd49d2c12fcd0164135d9`.

The available WSL Ubuntu environment was Linux `6.6.87.2-microsoft-standard-WSL2`, x86-64, with
Python 3.12.3. It had no installed C compiler. Official Ubuntu packages were downloaded and
unpacked into `$DIAG` without installation or privilege; the successful compiler was
`x86_64-linux-gnu-gcc-13 (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`, matching Trial #2's recorded
GCC major/minor/patch version. The frozen command was reproduced:

```text
gcc -O2 -Wall -Wextra -static -o $DIAG/helper_report-static-inspection-only \
  $DIAG/freeze/docs/experiments/launch-exec-01/helper_report.c
```

The resulting static x86-64 ELF was never executed. Its diagnostic SHA-256 was
`1ca23f083f088a304503a1e778fff019ed9dc86a640457012e9b5254fda81a6a`.

### 3.2 Source objects and locator contract

The frozen source declares three independent, externally visible objects:

```c
volatile char g_marker_guard_lo[9] = "HELM-MARK";
volatile char g_marker[16] = "helper_report\0\0\0";
volatile char g_marker_guard_hi[9] = "KRAM-MLEH";
```

The frozen `find_marker_region` searches for exactly one `HELM-MARK` occurrence and then requires
`KRAM-MLEH` exactly 16 bytes after the end of that low guard. Its accepted byte layout is therefore:

```text
HELM-MARK | exactly 16 marker bytes | KRAM-MLEH
```

It returns the first marker-byte offset only if that complete 34-byte guarded region exists and the
low guard is unique. Absence of the low guard, wrong placement of the high guard, or a duplicate low
guard all return the same `None`.

### 3.3 Observed linked layout

`nm -S -n`, `readelf -Ws`, `objdump -s -j .data`, `strings -a -t x` and direct Python byte scanning
agreed:

| Object | Virtual address | File offset | Size |
|---|---:|---:|---:|
| `g_marker_guard_hi` (`KRAM-MLEH`) | `0x4ab0d8` | `0xaa0d8` | 9 |
| `g_marker` (`helper_report\0\0\0`) | `0x4ab0f0` | `0xaa0f0` | 16 |
| `g_marker_guard_lo` (`HELM-MARK`) | `0x4ab100` | `0xaa100` | 9 |

Each literal occurred exactly once, but the compiler/linker emitted the three independent objects
in reverse source order. There were 15 padding bytes between the end of the high guard and the
marker; the marker immediately preceded the low guard. Consequently the required forward region
occurred **zero** times. Direct invocation of the frozen `driver.find_marker_region` on the bytes
returned `None`.

### 3.4 Root cause and future construction

The posing design incorrectly inferred adjacency and order from three separate C objects. Neither
the declarations nor the build supplied a layout mechanism that made the locator's assumed
34-byte representation true. This is the precise shared setup failure for E6 and E6c.

The smallest robust future construction is one deliberately contiguous byte array, for example one
34-byte `volatile unsigned char` object containing the low guard, the 16-byte mutable region and the
high guard. The helper should read the marker by a fixed offset within that one object, and the
locator should locate and validate that one explicitly declared region with a unique frozen guard.
An array provides one contiguous object and removes dependence on ordering or padding between
separate globals. The locator should also report which invariant failed.

No source change was made, and E6c's runtime mapping question remains for a future honestly posed
recorded case.

**`E6_E6C_STATIC_ROOT_CAUSE_CONFIRMED`**

## 4. R3 `waitid` / `CLD_DUMPED` diagnostic

### 4.1 Frozen observation path

The frozen launcher reaps the direct child with `waitid(P_PIDFD, ..., WEXITED)`. It classifies
`CLD_EXITED` as `Exited` and every other successful non-timeout termination as `Signaled`, but its
receipt writes:

```c
info.si_code == CLD_KILLED ? info.si_status : -1
```

for `term_signal`. The frozen normalizer then looks that integer up in its closed signal table
(1–31, including `SIGSEGV` 11). It cannot render `-1`, so the case becomes INVALID with
"termination signal outside the frozen signal table".

Linux [`waitid(2)`](https://man7.org/linux/man-pages/man2/wait.2.html) defines `CLD_KILLED` as
signal termination without the core-dump classification and `CLD_DUMPED` as signal termination
with it. For either signal termination, `si_status` contains the terminating signal; `si_code`
determines how to interpret the field. The launcher's `CLD_KILLED`-only condition therefore loses
a valid signal when `si_code == CLD_DUMPED`.

### 4.2 Isolated probe

One Python 3.12.3 process called `os.fork()`. The disposable child set `RLIMIT_CORE` soft and hard
limits to zero and sent itself `SIGSEGV`; the parent called `os.waitid(os.P_PID, child,
os.WEXITED)`. No launcher, HELM helper, frozen runner or LAUNCH-EXEC case participated, and no
retained core file was required or inspected.

Observed:

```text
SI_CODE=CLD_DUMPED
SI_STATUS=11
EXPECTED_SIGNAL=11
RLIMIT_CORE_CHILD=0
RETAINED_CORE_FILE_REQUIRED=NO
```

This reproduces the exact semantic combination implicated by R3: a zero core-size limit does not
prevent `waitid` from classifying the `SIGSEGV` termination as `CLD_DUMPED`, and `si_status` still
contains 11. The frozen launcher would publish `term_signal: -1`; the normalizer would then produce
R3's preserved INVALID reason.

The future correction must preserve `si_code` and use `si_status` for both `CLD_KILLED` and
`CLD_DUMPED`, while keeping the two classifications distinguishable. It must not merely add an
entry for `-1` to the signal vocabulary. No correction was made here.

**`R3_CLD_DUMPED_OBSERVATION_DEFECT_CONFIRMED`**

## 5. Commands and failed setup attempts

Commands are shown with `$REPO` and `$DIAG`; no private host path is published.

```text
git archive ba41a3f12be411058ed50e78bcd1c7e22afb7ae4 \
  docs/experiments/launch-exec-01/helper_report.c \
  docs/experiments/launch-exec-01/driver.py | tar -xf - -C $DIAG/freeze

apt-get -o Dir::State::lists=$DIAG/lists -o Dir::Cache=$DIAG/cache update
apt-get -o Dir::State::lists=$DIAG/lists -o Dir::Cache=$DIAG/cache download \
  gcc-13-x86-64-linux-gnu cpp-13-x86-64-linux-gnu libgcc-13-dev libc6-dev \
  linux-libc-dev libcrypt-dev rpcsvc-proto libc-dev-bin \
  libisl23 libmpc3 libmpfr6 libgmp10 zlib1g libzstd1 libcc1-0
dpkg-deb -x PACKAGE.deb $DIAG/root

gcc -O2 -Wall -Wextra -static -o $DIAG/helper_report-static-inspection-only \
  $DIAG/freeze/docs/experiments/launch-exec-01/helper_report.c
file $DIAG/helper_report-static-inspection-only
sha256sum $DIAG/helper_report-static-inspection-only
nm -S -n $DIAG/helper_report-static-inspection-only
readelf -Ws $DIAG/helper_report-static-inspection-only
objdump -s -j .data --start-address=0x4ab0c0 --stop-address=0x4ab120 \
  $DIAG/helper_report-static-inspection-only
strings -a -t x $DIAG/helper_report-static-inspection-only
python3 -c '<bounded byte-offset scan>'
python3 -c '<import frozen driver; call find_marker_region on bytes>'

python3 -c '<fork one child; set RLIMIT_CORE=0; send SIGSEGV; parent waitid>'
```

Failures were retained rather than hidden:

* No compiler existed in the default WSL environment. Checks of two dormant alternative local WSL
  environments timed out; both were returned to their prior stopped state.
* The first scratch command failed before extraction because its shell wrapper lost the temporary
  variable; it produced no binary.
* An initial unprivileged package download used a stale package list and received three HTTP 404s;
  it produced no binary. Refreshing package metadata into `$DIAG`, not the system package database,
  resolved this.
* The first unpacked GCC invocation stopped before compilation because `libisl.so.23` was absent.
  Its declared runtime dependencies were then downloaded and unpacked into `$DIAG`; the frozen
  static compile completed.

All scratch packages, extracted sources and the unexecuted binary were removed after inspection.
These setup failures changed no conclusion and are not LAUNCH-EXEC experiments.

## 6. Boundary, implications and unresolved items

| Boundary | Count / result |
|---|---|
| HELM experimental ELF executions | **ZERO** |
| HELM helper executions | **ZERO** |
| `launcher_spike` executions | **ZERO** |
| LAUNCH-EXEC cases posed | **ZERO** |
| Trial #2 reruns | **ZERO** |
| D-7 uses or grants | **ZERO** |
| Trial #3 freezes or executions | **ZERO** |
| Isolated non-HELM process probes | **ONE** |

Both authorised diagnostic questions are conclusive and require no new owner decision before the
bounded correction is prepared. Still unresolved by design, not by diagnostic failure:

* E6c's actual shared-writable-mapping kernel result awaits a future valid, honestly posed recorded
  case.
* N3 privileged-transition behavior remains deferred.
* Trial #3 does not yet exist; correction preparation is not a freeze, execution authorisation or
  D-7.

**TRIAL #2 D-7 IS CONSUMED**

**LAUNCH-EXEC-01 TRIAL #2 VALID TRIAL COUNT IS ONE**

**TRIAL #2 MUST NOT BE RERUN**

**NO TRIAL #3 IS AUTHORISED**

**`TRIAL_2_POSTMORTEM_CLOSED_READY_FOR_BOUNDED_CORRECTION`**
