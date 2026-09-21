# LAUNCH-EXEC-01 — Trial #1 result review

Independent review of the first and only execution authorised under D-7. The reviewer did not
author the frozen experiment or the trial workflow. Nothing was fixed, re-run or dispatched, and no
frozen artefact was touched.

**Trial #1 crossed the immutability boundary and then aborted on a harness defect. It produced no
evidence document, and no aggregate verdict can be derived from it.**

## 1. Trial identity

| | |
|---|---|
| Workflow run | `34500901306` — run number **1**, attempt **1**, `workflow_dispatch` |
| Job | `102951135361` (`trial`) → **failure** |
| Workflow head | `f487ea3103de8b0916eaaf39a8bb8a604990d7bf` (`main`) |
| **Experiment bytes executed** | **`89c923a147ff16182d4d0ae14a0bd7bb6e62723d`** |
| `SOURCE-HASHES.json` SHA-256 | `9fb861602d477a00f014f14f0a31b5979947af18fa2b620b95ac803ab5bfc365` |
| D-7 decision commit | `5272ad633b0dd73f6e4662c6b50adabf67359dc1` |
| Artifact | id `10161795945`, `launch-exec-01-evidence`, 1662 bytes |
| Artifact ZIP SHA-256 | `76db4ed6c5470aacf76161f84932b9e072fd3bfc4d7f13ec2effeee0167277a8` |

The ZIP was downloaded read-only and its digest recomputed locally; it matches GitHub's own reported
digest exactly. The workflow ref and the experiment bytes are different objects and are recorded
separately on purpose.

## 2. Dispatch and freeze binding — all verified

Every gate the dispatcher was built to enforce fired correctly and is visible in the job log:

| Check | Observed |
|---|---|
| `run_attempt` | `1` |
| Confirmation input | accepted (`RUN-ONE-VALID-TRIAL`) |
| Freeze and manifest inputs | matched the authorised values |
| Checkout resolved to | `89c923a147ff16182d4d0ae14a0bd7bb6e62723d` |
| Checked-out manifest SHA-256 | `9fb8616…f365` — matched |
| Frozen source verification | `"frozen sources match the manifest"`, `freeze_verified: true` |
| Driver completeness | `complete: true`, `driver_total: 72`, `frozen_total: 72`, no missing/unknown/duplicates |
| Static posability | `"unposable": {}` — **72 handlers / 72 posable / 0 unposable** |

The trial executed the authorised bytes, not a branch tip. No finding here.

## 3. Preflight — passed

Environment recorded in the preserved `preflight.json`:

| | |
|---|---|
| Platform | Linux `6.17.0-1022-azure`, `x86_64`, Ubuntu 24.04.5 LTS |
| euid | `1001` (unprivileged) |
| `ptrace_scope` | `1` |
| strace | `/usr/bin/strace`, **6.8** (frozen floor 5.4) |
| clone3 | `available: true`, `EINVAL` on a zero-size argument — implemented and reached |
| gcc / glibc | 13.3.0 / 2.39 |
| `parent_no_new_privs` | `0` |
| binfmt_misc | enabled |
| Block reasons derived | `unprivileged_runner` only |

The workflow's `--preflight-only` step is an inventory and gates nothing. The authoritative gate is
`preflight_gates()` inside `run_trial`, and it is established as passed **structurally**: the
traceback shows execution reached `run_trial` line 205, the case loop, which is only reachable when
`halts` is empty. Had it halted, the function would have returned `HALT_PREFLIGHT` at line 191 with
zero cases posed.

**`PREFLIGHT_PASSED`.**

## 4. The immutability boundary was crossed

The traceback places execution inside the case loop:

```
run_launch_exec_01.py, line 205, in run_trial
    observation = driver.observe(plan, ctx, auth)
driver.py, line 1256, in observe
    built = SETUPS[plan.setup](ctx, plan)
driver.py, line 335, in _setup_writer_open_closed
    os.pwrite(fd, os.pread(fd, 1, 0), 0)     # a no-op rewrite of one byte
OSError: [Errno 9] Bad file descriptor
```

`driver.observe` was reached for real case plans with a real `Authorisation`. Preregistered cases
were executed: fixtures were built, `launcher_spike` was run, and E1 — a traced case — was run under
`strace`.

**`IMMUTABILITY_BOUNDARY_CROSSED`.**

## 5. Exact progress

`E5b` is the only case in the frozen membership whose setup is `writer_open_closed`, and it sits at
index 5 — the **sixth** case in `MEMBERSHIP` order. The loop body is
`observe(...)` → `evaluate(...)` → store, so a loop that has advanced to iteration six completed
both calls for the five before it.

| Case | Execution entered | Record completed in memory | Durable evidence | Claimable status |
|---|---|---|---|---|
| E1 | YES | YES | **NO** | `UNKNOWN_FROM_PRESERVED_EVIDENCE` |
| E2 | YES | YES | **NO** | `UNKNOWN_FROM_PRESERVED_EVIDENCE` |
| E3 | YES | YES | **NO** | `UNKNOWN_FROM_PRESERVED_EVIDENCE` |
| E4 | YES | YES | **NO** | `UNKNOWN_FROM_PRESERVED_EVIDENCE` |
| E5 | YES | YES | **NO** | `UNKNOWN_FROM_PRESERVED_EVIDENCE` |
| **E5b** | YES — setup only | NO | **NO** | `UNKNOWN_FROM_PRESERVED_EVIDENCE` |
| E6 … M5 (66 cases) | NO | NO | NO | `UNKNOWN_FROM_PRESERVED_EVIDENCE` (fact established: never entered) |

**6 case handlers entered. 0 durable case statuses recoverable.** The in-memory records for E1–E5
existed and were lost with the process; the raw per-case evidence, including E1's syscall record,
was written into the ephemeral runner's build directory and was never uploaded.

Wall time from the start of the trial step to the traceback: **8.4 s** (16:15:21.577 → 16:15:29.965),
which covers `harness.build()` compiling six binaries plus the five completed cases.

## 6. The E5b harness defect

Frozen bytes at `89c923a`, `driver.py` lines 328–338:

```python
@_setup("writer_open_closed")
def _setup_writer_open_closed(ctx, plan):
    """E5b: the writer opens, writes and CLOSES before launch -- no ETXTBSY,
    because the write-deny reference is taken at exec time."""
    info = _setup_copy(ctx, plan)
    fd = os.open(info["exec_path"], os.O_WRONLY)
    try:
        os.pwrite(fd, os.pread(fd, 1, 0), 0)     # a no-op rewrite of one byte
    finally:
        os.close(fd)
    return info
```

The descriptor is opened `O_WRONLY` and then **read** from. On Linux a file description carries its
access mode: `O_WRONLY` sets `FMODE_WRITE` and not `FMODE_READ`, and the kernel's read path rejects
the call before it reaches the filesystem — `read(2)` documents `EBADF` as *"fd is not a valid file
descriptor **or is not open for reading**"*, and `pread(2)` inherits `read(2)`'s errors. So the
descriptor is perfectly valid and the failure is categorical: a write-only description can never
serve a read. The intent — "rewrite one byte with itself so the file is genuinely written to before
the writer closes" — needs `O_RDWR`.

**Classification: harness / case-setup defect.** It is a defect in how the experiment *prepares*
E5b's fixture, not an observation about `launch()`. The mechanism under test was never reached for
E5b. It is recorded, not fixed.

**Defect-class audit (read-only).** The frozen `driver.py` has exactly two `os.open` sites. Line 333
is the defective one. The only other, line 1447, opens `O_RDONLY | O_NONBLOCK` and reads — correct.
No second instance of this class exists in the frozen driver.

Worth noting for the pretrial record: `os.pread` does not exist on Windows, so this line could not
have been exercised in any form on the authoring machine — it would have raised `AttributeError`
rather than `EBADF`. The compile-only CI never ran it either, because CI compiles and inspects but
poses no case. The defect was reachable only by the trial itself.

## 7. Three failure layers, kept apart

| Layer | Status |
|---|---|
| **A — Workflow** | `failure`. Steps 1–11 succeeded; step 12, *Report the trial command outcome*, failed deliberately because the runner exited 1. That design is what let the artifact be preserved first. |
| **B — Runner / harness** | **This is what happened.** Python raised `OSError` during E5b's fixture setup, before any evidence document was constructed. |
| **C — Mechanism result** | **Nothing.** No frozen case demonstrated a mismatch with its expectation. The launcher mechanism was not evaluated. |

The red X in the Actions UI is layer A. It says nothing about layer C.

## 8. No aggregate is derivable

The frozen checker maps a missing record to `INVALID, "no status was recorded for a case in the
frozen membership"`, and the definition states that a membership case with no recorded status is
INVALID rather than absent. Feeding an empty record set to `checker.report` would therefore produce
72 × INVALID → `MECHANISM_INCONCLUSIVE`. **That number would be a fabrication, and it was not
computed.** Three reasons:

1. **It would assert something false about five cases.** INVALID means *"could not be posed"*. E1–E5
   **were** posed, scored, and their statuses lost. Recording them as unposable would misrepresent a
   preservation failure as a case-level defect.
2. **It would misattribute the abort to the other 66.** They were never entered. This very run
   established 72/72 static posability, so their posability is not in doubt; the process died before
   reaching them.
3. **The rule presupposes a produced record set.** It exists so a completed trial cannot quietly
   drop a case from its report. `checker.report` was never called, no document exists, and the
   definition's own instruction for this situation is different and explicit: *"a defect discovered
   afterwards ends the trial and starts a new, separately recorded one."*

**`AGGREGATE_NOT_DERIVABLE_FROM_FROZEN_EVIDENCE`.**

## 9. Artifact content

| File | Size | SHA-256 |
|---|---:|---|
| `evidence.json` | **0** | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `preflight.json` | 4005 | `84a6a3a78aa8a627c39ad1ad775bb6098ea48eb486377dbcc911772a9179f757` |

`evidence.json` is empty because the shell created it with `>` before Python started, and the
sanitised document is written only at the very end of a completed trial. Its emptiness proves **no
completed trial document was published**. It does **not** prove that no case executed — §4 and §5
show six case handlers were entered. Those two propositions are separate and must stay separate.

`preflight.json` parses as JSON and carries `preflight`, `manifest` and `driver` blocks.

**Privacy: clean.** Checked directly against the published bytes — no hostname, no `fv-az…` runner
name, no account name, no `/home/` path, no credential shape, and no broad `uname` field. The P-16
fixes are visible working in production: the host descriptor was never collected, and P-14 rendered
the work directory as `<ABSPATH>` inside `noexec_writable`.

## 10. Crash evidence

| | |
|---|---|
| Exception | `OSError` |
| errno | `9` — `EBADF`, *Bad file descriptor* |
| Source file | `docs/experiments/launch-exec-01/driver.py` |
| Function | `_setup_writer_open_closed` |
| Line | 335 — `os.pwrite(fd, os.pread(fd, 1, 0), 0)` |
| Case being prepared | **E5b** |
| Reached via | `run_launch_exec_01.py:205 run_trial` → `driver.py:1256 observe` |
| Runner exit code | **1** |
| Job log SHA-256 | `586bd0222f7e82137cef8cbd8eddf56abd3b634245ab0a881ff6685531bda343` |

The excerpt in §4 is the traceback with the runner's absolute workspace prefix removed; it is
otherwise verbatim and was not rewritten into a cleaner failure. The complete log is retained
local-only.

## 11. Trial #1 status

`NOT_RUN` is now false — the boundary was crossed. `MECHANISM_REJECTED` is also false: nothing was
rejected. No existing repository term covers "boundary crossed, harness aborted, no aggregate".

**Recommended term: `TRIAL_ABORTED_AFTER_BOUNDARY`.** It states both facts that matter — the trial
began, and it produced no verdict — and it does not redefine or borrow a frozen aggregate verdict.
`TRIAL_STARTED_HARNESS_ABORT` is an acceptable synonym; either is preferable to any wording that
implies a mechanism outcome.

## 12. D-7 is consumed

The owner authorised exactly one execution. A preregistered case was executed. The authorisation has
been exercised and **must not be reused**, whatever the trial's usefulness turned out to be — an
aborted trial does not return the authorisation unused.

**`D7_AUTHORIZATION_CONSUMED`.**

## 13. No re-run

* GitHub's **Re-run is not authorised** — and the workflow itself refuses it, halting unless
  `run_attempt == 1`.
* A **second manual `workflow_dispatch` is not authorised** by the existing D-7. The `run_attempt`
  guard cannot prevent one; the one-trial limit is an owner protocol rule as well as a mechanism.
* **Trial #1 cannot be resumed.** The runner has no resume path, the ephemeral workspace is gone,
  and the in-memory records for E1–E5 no longer exist.
* **Trial #1 cannot be patched in place.** The frozen artefacts are immutable from the moment the
  first case executed.

**TRIAL_1 MUST NOT BE RE-RUN OR RESUMED.**

## 14. A second finding, for Trial #2's preregistration

The trial workflow recorded **no digests of the binaries it built**. The definition requires
`BUILD-EVIDENCE.md` to carry one section per build with the SHA-256 of every produced binary and
fixture. Trial #1's build therefore cannot be recorded as the protocol requires, and that gap is not
recoverable now. A Trial #2 dispatcher should hash the build directory before the trial step. Noted,
not fixed.

## 15. What would be required for a Trial #2

Not performed here, and none of it may begin before the owner accepts this review:

1. Correct the E5b setup in a **descendant** experiment version — never in the frozen one.
2. Audit every other setup handler for the same defect class and for other paths that only a real
   trial can reach; the two `os.open` sites are cleared, but the broader class of "setup code no
   local platform can execute" is not.
3. Add built-binary digest capture so `BUILD-EVIDENCE.md` can be completed.
4. Freeze a new trial version and re-preregister what changed.
5. Non-executing validation, then a bounded independent review.
6. A **new** explicit owner D-7 authorisation bound to the new freeze.
7. A **new** manual dispatch.

---

**D-7 is consumed. LAUNCH-EXEC-01 Trial #1 is `TRIAL_ABORTED_AFTER_BOUNDARY` with no derivable
aggregate. This review must be owner-accepted before any Trial #2 fix, preregistration or new D-7
authorisation.**
