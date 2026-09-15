# LAUNCH-EXEC-01 — Trial #3 X2c postmortem

> **THIS IS POST-TRIAL ENGINEERING DIAGNOSIS.
> IT DOES NOT CHANGE THE FROZEN TRIAL #3 RESULT.**
>
> Trial #3's frozen result stays **`MECHANISM_REJECTED`**, 70 PASS / 1 FAIL / 1 BLOCKED, with X2c
> the sole FAIL. Trial #3 D-7 is consumed, the valid Trial #3 count is one, and Trial #3 must not
> be rerun. No Trial #4 is authorised, proposed or prepared here.

This postmortem is bounded to X2c. It changes no code, frozen expectation, frozen evidence,
checker, launcher, driver, observation module or fixture. Nothing was dispatched, re-run or
posed, and the frozen runner was not invoked. Two scratch diagnostics were run outside the
repository (Section 9). They are **non-authoritative post-trial diagnostics, not Trial #3
evidence**, and no conclusion below that is marked load-bearing depends on them.

Sources were read from the Trial #3 freeze `bebd8a5f83d4d0daebe9b068050cb5436289c75e` (a
`git archive` extraction; every experiment source is unchanged at `8dc9e96`, and `--verify-freeze`
passes), from the preserved evidence at `11bddc5`, and from the
[result review](HELM-LAUNCH-EXEC-01-TRIAL-003-RESULT-REVIEW.md) at `8dc9e96`. Line numbers below are
the frozen files'.

## 1. Starting state

| Check | Observed |
|---|---|
| Worktree | clean |
| Branch | `docs/helm-launch-architecture` |
| `HEAD` | `8dc9e96c64c2e590434e6e210875ce6d13fefc32` |
| `origin/docs/helm-launch-architecture` | `8dc9e96c64c2e590434e6e210875ce6d13fefc32` |
| `origin/main` | `501a7fa95c4884da4fec9a20a512c2d63f2b30cc` |

## 2. The frozen X2c contract

| Field | Frozen value | Source |
|---|---|---|
| Classification | **mandatory** | [`frozen_cases.py`](../experiments/launch-exec-01/frozen_cases.py) 359 |
| Frozen prediction | **`interpreter_ran_with_devfd`** (single prediction, no safe set) | `frozen_cases.py` 359 |
| Binary | **`script_fixture.sh`** | [`driver.py`](../experiments/launch-exec-01/driver.py) 2470 |
| Setup | **`script_fixture`** (the generated fixture at its declared mode `0755`) | `driver.py` 1445–1447, 2471 |
| Spike flags | **`--bypass-admission`**, **`--exec-fd-no-cloexec`** | `driver.py` 2472 |
| Declared channels | **`receipt`**, **`report`** | `driver.py` 2471 |
| Frozen rule | **`interpreter_ran_with_devfd`** → `observations.rule_interpreter_ran_with_devfd` | `driver.py` 2470; [`observations.py`](../experiments/launch-exec-01/observations.py) 2171 |
| argv | `argv0 = "script_fixture.sh"`, no helper arguments | `driver.py` 3354–3356 |
| Traced / posed check / assertions | `false` / none / none | the preserved plan |

The frozen note (`frozen_cases.py` 359–362 and
[definition](../experiments/LAUNCH-EXEC-01-DEFINITION.md) line 396) says: *"same script without
O_CLOEXEC: the interpreter runs and receives /dev/fd/N. Recording X2b without X2c would attribute a
CLOEXEC artefact to scripts as a class."*

**What the token was intended to mean.** `interpreter_ran_with_devfd` asserts one positive fact:
the `#!` interpreter was actually executed, and among its arguments was a `/dev/fd/N` pathname
standing for the exec descriptor. The rule's own docstring (`observations.py` 1705–1707) names the
evidence it expects for that fact: *"The evidence is the interpreter's own argv."* The token is an
observation about the executed interpreter, not a claim the launcher makes about itself.

## 3. X2b / X2c pair semantics

The pair was introduced by finding **A-8** of the
[pre-execution review](HELM-LAUNCH-PRE-EXECUTION-REVIEW.md): `execveat` on an interpreter-requiring
file fails `ENOENT` when the exec descriptor is `O_CLOEXEC`, so a single script observation "would
record a CLOEXEC artefact and attribute it to scripts as a class". The review's X-series row says
X2b/X2c "split the counterfactual into its `O_CLOEXEC` and non-`CLOEXEC` arms".

| | X2b | X2c |
|---|---|---|
| Fixture | `script_fixture.sh` | same |
| Admission | bypassed | bypassed |
| Exec descriptor | `O_CLOEXEC` (`launcher_spike.c` 691) | no `O_CLOEXEC`, kept non-CLOEXEC by `move_above_2(exec_fd, 0)` (789) |
| Channels | `receipt` | `receipt`, `report` |
| Frozen outcome | `ExecFailed:ENOENT` | `interpreter_ran_with_devfd` |
| Trial #3 | **PASS**, `ExecFailed:ENOENT` | **FAIL**, `ExecStatusIndeterminate` |

The pair exists so that script refusal in 0.1 is attributed to its real causes (a `/dev/fd/N`
procfs dependency and a required non-CLOEXEC descriptor, [architecture](../research/HELM-LAUNCH-ARCHITECTURE.md)
section 6 lines 278–299 and the table at 364–367) and not to "scripts cannot execute".

Two claims have to be separated:

* **CLAIM 1** — in the non-CLOEXEC arm the script path *proceeds* rather than failing with the
  documented `ENOENT`. That is a fact about `execveat`'s return, and the launcher's receipt can speak
  to it: no explicit pre-exec status record, and a clean exec-status EOF.
* **CLAIM 2** — the experiment can *positively observe* that the interpreter ran and received
  `/dev/fd/N`. That is a fact about the executed image, and it needs evidence from the image.

**The frozen material conflated them.** The frozen prediction and rule demand CLAIM 2. The
reasoning that approved the case argued only CLAIM 1: the Trial #3
[correction review](HELM-LAUNCH-EXEC-01-TRIAL-003-CORRECTION-REVIEW.md) (7.1, lines 165–169, "reasoned
and not executed") wrote that "X2c's non-CLOEXEC descriptor lets the interpreter receive and read
`/dev/fd/N`". No record in the lineage checked that X2c's executed object can emit the evidence its
rule reads.

## 4. Fixture capability audit

`script_fixture.sh` is `make_fixtures.SCRIPT_FIXTURE` (`make_fixtures.py` 44–53), 490 bytes, SHA-256
`37f800b1a77f026dbf2ee2724829458ddf78dd8329527deee3d086025959208a` (equal to Trial #3's build
identity), mode `0755`:

```sh
#!/bin/sh
# (five comment lines)
echo "LAUNCH-EXEC-01 script fixture ran with argv: $0 $@"
exit 0
```

| Question | Answer, from the bytes |
|---|---|
| Shebang interpreter | `/bin/sh` |
| Emits the frozen report sentinel `HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n` | **no** — absent from the file, and no output line can contain it (below) |
| Emits JSON | **no** — the file contains no `{` |
| Emits a helper_report-compatible document | **no** — no sentinel, no JSON object, no `marker` key |
| Prints `$0` | **yes** |
| Prints `$@` | **yes** (empty for X2c: no helper arguments) |
| Could its output contain `/dev/fd/N` | **yes** — `execveat(2)` makes the interpreter's script argument `/dev/fd/N`, which the shell exposes as `$0` |

Its only output is one free-text line: `LAUNCH-EXEC-01 script fixture ran with argv: <$0> <$@>\n`.

**What the rule requires** (all frozen):

1. `rule_interpreter_ran_with_devfd` (1702–1724) returns its success token **only** at line 1720,
   after `_report_or_decisive(obs)` returned a non-`None` report whose `argv` is a non-empty list of
   `{"value": <string>}` objects, one of which starts with `/dev/fd/`.
2. `_report_or_decisive` (1354–1365) yields a report only through `_usable_report` (1288–1313),
   which requires `report_state == "complete"` and a parsed report.
3. That state comes from `helper_report_state(spike["stdout"])` (319–343): the launcher's retained
   **stdout** prefix, decoded from base64 (`decode_capture`, 272–296), must parse through
   `parse_helper_report` (217–245).
4. `parse_helper_report` requires the exact sentinel, then a UTF-8 JSON **object** after it, and a
   `marker` key in that object (241–244). Otherwise the report is `None`.

**Proof the fixture cannot meet it.**

* The fixture's output is one line beginning `LAUNCH-EXEC-01 script fixture ran with argv: `.
* `$0` is set by the kernel, not by the plan (`execve(2)`: "there is no way to get the argv[0] that
  was passed"), and is `/dev/fd/N` or `/dev/fd/N/P` (`execveat(2)`). `$@` is empty.
* Diagnostic A (Section 9) enumerated both forms for every N in 0–4095 — 8,192 possible lines. None
  contains the sentinel and none parses as a helper report.
* The rule reads stdout only, and `/bin/sh` emits no sentinel of its own.

> **`X2C_FIXTURE_CANNOT_SUPPLY_RULE_EVIDENCE`**

## 5. Static posability

The pretrial gate (`run_launch_exec_01.preflight_gates` → `driver.unposable_cases`, 2767–2789)
calls `CasePlan.missing_channels` (224–231), which returns the plan's declared channels that are not
in `SPIKE_SUPPLIED_CHANNELS` (141–144). `CH_REPORT` is in that set because the launcher *transports*
descriptor 1's retained prefix (the PRE-D7-B1 correction). X2c declares `receipt` and `report`, so
`missing_channels()` is empty and X2c counted among the 72 posable cases.

That check answers **"can the launcher carry a report out?"**. It never asks **"can this executed
object produce one?"**. No frozen code relates a plan's `binary` to its declared channels
(`completeness()`, 2731–2764, checks setups, parents, rules, posed checks and channel *names* only).

Diagnostic A measured the class of the gap:

* 41 plans declare `CH_REPORT`, across the binaries `helper_dynamic`, `helper_report`,
  `helper_setid` and `script_fixture.sh`;
* only `helper_report.c`, `helper_alt.c` and `helper_dynamic.c` contain the sentinel literal;
* **X2c is the only `CH_REPORT` plan whose binary is not a compiled helper.**

*(`helper_setid`, used by N3, contains no sentinel literal either. N3 was BLOCKED in both trials,
and whether it can deliver a report by another route was **not assessed** here: it is outside this
postmortem's scope and is noted only as the same unchecked property.)*

**Was X2c posable against its frozen success rule? No.** Transport availability was treated as
evidence availability. The false-positive path is:
`CH_REPORT ∈ SPIKE_SUPPLIED_CHANNELS` → `missing_channels() == ()` → absent from `unposable_cases()`
→ `ok 72 posable, 0 unposable` in the trial job log.

> **`X2C_STATIC_POSABILITY_FALSE_POSITIVE`**

## 6. The actual Trial #3 result path

**Preserved record** (`evidence.json`; journal `n = 87` entered, `n = 88` pose started, `n = 89`
completed): `mechanism_invoked: true`, `pose_started: true`, binding `DIRECT_BASE` bound, no
`not_posed`, `launch_returned: true`, `elapsed_ms: 1`, `outcome: "ExecStatusIndeterminate"`,
reason *"the stream that carries the report is complete and holds none: clean exec-status EOF with
no positive evidence that the pinned image ever ran; EOF alone never means exec"*.

The path, step by step. Each step is the only frozen code that can produce the preserved text:

1. **Receipt.** `driver.observe` parses the launcher's stdout with `parse_spike_stdout` (143–214). The
   admission was `accepted`: a refusal would take `refused:…` at `decisive_without_report` 1335.
2. **Capture and report state.** `helper_report_state(spike["stdout"])` (driver 3685). No sentinel
   in the retained prefix gives `parse_helper_report → None`. Because the prefix was decodable and
   decisive (`CompleteAtEof`, not truncated, fully retained, 299–316), the state is
   **`REPORT_ABSENT`** (341–342). The phrase "the stream that carries the report is complete and
   holds none" is emitted only for that state (1346–1350).
3. **Exec confirmation.** `exec_confirmation` (413–468) gets no report, `payload_is_recipe = None`
   (X2c declares no streams, driver 3688–3695) and no fixture signal. Its disposition was not
   `ExecFailed`, and `REPORT_ABSENT` is not an uninterpretable state, so it returns
   **`EXEC_DIED_BEFORE_EXEC`** (468).
4. **The rule.** `rule_interpreter_ran_with_devfd` calls `_report_or_decisive`. `_usable_report`
   fails, so `decisive_without_report` (1327–1351) runs. Not refused, and not `ExecFailed` — that
   would produce "the child wrote an explicit pre-exec status record" (1344), which X2b got and X2c
   did not. `report_state == REPORT_ABSENT`, so it calls `rule_process_disposition`.
5. **Process disposition.** `rule_process_disposition` (1386–1444):
   * the disposition was not `ExecStatusIndeterminate` — that branch's reason is "the exec status
     could not be determined from the record" (1419–1420);
   * it was not `ExitStatusUnobservable` (1422–1424);
   * confirmation is not `EXEC_UNINTERPRETABLE`.

   So it reaches 1441–1444 and returns **`ExecStatusIndeterminate`** with "clean exec-status EOF
   with no positive evidence that the pinned image ever ran; EOF alone never means exec".
6. **Wrapping.** `decisive_without_report` prefixes step 2's phrase. `driver._evaluate_repetition`
   (3182, 3221–3222) records the token as `outcome`.
7. **Checker.** `checker.score_case` (138–141) compares with the single frozen prediction and
   returns **FAIL**.

**Why not `interpreter_ran_with_devfd`:** no parsed report existed, so the only return that
produces it (1720) was unreachable. **Why not INVALID:** the rule did return a token, because
`REPORT_ABSENT` on a complete stream is decisive by the frozen design (definition lines 936–943,
"a complete stdout that holds no report"), and a token that is not the prediction is FAIL.
Diagnostic A's S6 shows the INVALID branch: the same bytes on a `WriterRetainedAfterChildExit`
stream are INVALID.

This path is correct execution of the frozen code as written. Whether that code posed the question
it was meant to pose is Sections 7, 12 and 13.

## 7. The conservative exec-proof boundary

The Trial #2 owner decision ([DECISIONS.md](../DECISIONS.md), decision 2, lines 258–262) fixed
**`S4_CONSERVATIVE_EXEC_EVIDENCE_POLICY_SELECTED`**: "clean exec-status EOF alone is not positive
proof that an executable image ran", and the launcher must not infer exec success from EOF alone.
Trial #3 kept it (`exec_confirmation` 417–422; `rule_process_disposition` 1426–1444). S5 PASSed in
Trial #3, which is the case that makes that policy necessary.

**X2c did not require the launcher receipt to claim exec success.** Its rule reads the helper
report first and uses the receipt only through the decisive-without-report fallback. X2c was meant
to obtain **independent positive evidence** through the `report` channel: *the interpreter's own
argv*. That channel is one of exactly three admissible positive-evidence kinds
(`exec_confirmation` 422–434: a parsed helper report, a recipe-matching payload, or O6/O7's
pre-armed FIFO signal). None of the three is producible by a `#!/bin/sh` script as frozen
(Section 4). X2c therefore had a correct evidence *policy* and an evidence *source* incapable of
satisfying it.

**The S4 policy is not the defect, and weakening it would not honestly fix X2c.** If clean EOF
were accepted as exec proof, X2c would still not get `interpreter_ran_with_devfd`: the rule still
needs the argv report, and the fallback would yield `Exited:0`, which is also not the prediction.
It would also re-open the S1/S5 indistinguishability the policy exists to close.

## 8. Execution versus observability — four categories

**A. What the preserved Trial #3 evidence proves.** From the reason text and the frozen path in
Section 6:

* X2c was genuinely posed through the launcher, and admission was bypassed as frozen;
* the child wrote **no** explicit pre-exec status record, so `execveat` did not report a failure
  through the launcher's status channel;
* the launcher recorded a clean exec-status EOF (`exec_confirmed`), with a disposition of
  `Exited`, `Signaled` or `TimedOut`. `TimedOut` is excluded by `elapsed_ms: 1` against a 5,000 ms
  timeout, so it was `Exited` or `Signaled`;
* descriptor 1 was complete at EOF, fully retained, and held no report sentinel;
* the frozen rule and checker produced FAIL exactly as recorded.

**B. What it leaves indeterminate.**

* whether `/bin/sh` ran;
* whether it received and opened `/dev/fd/N`;
* whether descriptor 1 held zero bytes or the fixture's line;
* the exit code, or which of `Exited`/`Signaled`;
* anything on stderr.

The published record carries none of these. The capture prefix is internal-only by the frozen
publication contract, and the record does not reproduce the receipt. Section 9's S1–S4 show that
"the script ran and printed its `/dev/fd/N` line", "the interpreter failed to open `/dev/fd/N`" and
"nothing was written" all produce the byte-identical record. **The Trial #3 evidence does not show
that the script ran, and it does not show that it did not.**

**C. What source-level reasoning predicts.**

* The launcher child's only non-exec exits write a full status record (`child_fail`, 249–265). The
  exception, `die_before_exec` (355–362), is S5-only and not set for X2c. So a record-free EOF is
  predicted to mean `execveat` passed the kernel's point of no return and replaced the child image.
* By `execveat(2)` and `execve(2)` (Section 10), that image is `/bin/sh` invoked as
  `/bin/sh /dev/fd/N`, with the non-CLOEXEC descriptor N still open.
* It then prints the fixture line and exits 0, provided `/proc` is mounted.

This is reasoning, not Trial #3 evidence.

**D. What a later diagnostic may demonstrate.** Diagnostic B demonstrates C's prediction outside
the launcher on one kernel (Section 9). A faithful demonstration *through `launcher_spike`* would
be a new observation of the experiment's mechanism. It was not run here, and it is not needed for
any classification below.

## 9. Scratch diagnostics — NON-AUTHORITATIVE, NOT TRIAL #3 EVIDENCE

Both ran under WSL 2 (Ubuntu, kernel `6.6.87.2-microsoft-standard-WSL2`, Python 3.12.3,
`/bin/sh → /usr/bin/dash`, `/dev/fd → /proc/self/fd`) from a disposable scratch directory outside
the repository, against a `git archive` of `bebd8a5`.

* They wrote nothing into `docs/experiments/evidence/`, `trial-output/` or
  `target/launch-exec-01/`.
* They did not import or invoke `run_launch_exec_01.py`, `driver.observe`/`pose` or `harness`, and
  did not execute `launcher_spike` or any helper ELF.
* The scratch scripts and their JSON outputs are not committed.
* The trial runner used kernel `6.17.0-1022-azure`, so kernel-level results here are indicative
  only.

### Diagnostic A — frozen parser and rule over synthetic receipts (no process executed)

**Method.**

1. Build a launcher receipt that `parse_spike_stdout` accepts, with a stdout block carrying the
   given bytes as `capture_prefix_base64`.
2. Derive the observation exactly as `driver.observe` does after launch (3672–3748) for X2c's plan.
3. Run `observations.derive("interpreter_ran_with_devfd", obs)`, then
   `driver._evaluate_repetition(CASE_PLANS["X2c"], …)`, then `checker.score_case("X2c", record)`.

Command: `python3 -I -B diag_a_parser.py <bebd8a5 launch-exec-01> diag_b_result.json out.json`.

| Scenario | Receipt | Report state | Exec confirmation | Outcome | Status |
|---|---|---|---|---|---|
| S1 | Exited 0; stdout = Diagnostic B's non-CLOEXEC line `…argv: /dev/fd/3 \n`; complete | `absent` | `died_before_exec` | `ExecStatusIndeterminate` | **FAIL**, with **the exact Trial #3 record reason** |
| S2 | as S1 with `/dev/fd/7` | `absent` | `died_before_exec` | `ExecStatusIndeterminate` | FAIL, same reason |
| S3 | Exited 2; stderr `cannot open /dev/fd/3`; empty stdout | `absent` | `died_before_exec` | `ExecStatusIndeterminate` | FAIL, same reason |
| S4 | Exited 0; nothing written | `absent` | `died_before_exec` | `ExecStatusIndeterminate` | FAIL, same reason |
| S5 | `ExecFailed` at `EXEC`, errno `ENOENT` (the CLOEXEC arm) | `absent` | `pre_exec_error` | `ExecFailed:ENOENT` | FAIL |
| S6 | S1's bytes, `WriterRetainedAfterChildExit` | `stream_incomplete` | `uninterpretable` | none | INVALID |
| S7 **control, not producible by the fixture** | Exited 0; stdout = sentinel + `{"marker": …, "argv": [{"value": "/bin/sh"}, {"value": "/dev/fd/3"}]}` | `complete` | `exec_reached` | `interpreter_ran_with_devfd` | **PASS** |

The fixture audit covered 8,192 possible output lines (`/dev/fd/N` and `/dev/fd/N/script_fixture.sh`,
N = 0–4095): **0** contained the sentinel and **0** parsed as a helper report. The static-posability
facts are those in Section 5.

**What A shows.**

* S1–S4: every stdout the fixture can produce, and every other record-free outcome, collapses to
  the preserved record.
* S7: the rule does work, but only when a structured report exists, and the fixture cannot write
  one.

### Diagnostic B — standalone `execveat` on the frozen script bytes

A Python `ctypes` reproducer:

* writes `make_fixtures.SCRIPT_FIXTURE` (SHA-256 checked) at mode `0755`;
* opens it, sets or clears `FD_CLOEXEC`, and forks;
* in the child: binds stdout/stderr to pipes and stdin to `/dev/null`, calls `chdir` and
  `PR_SET_NO_NEW_PRIVS`, then `syscall(322 /* execveat */, fd, "", ["script_fixture.sh"], [], AT_EMPTY_PATH)`
  with an **empty environment**. On failure it writes the errno to an `O_CLOEXEC` status pipe.

Command: `python3 -I -B diag_b_execveat.py <bebd8a5 launch-exec-01> out.json`. It did not replicate
the launcher's `clone3`, `close_range`, signal resets or `setpgid`.

| Arm | Exec fd | Status pipe | Exit | stdout | stderr |
|---|---|---|---|---|---|
| `FD_CLOEXEC` | 3, CLOEXEC | record `ENOENT` | 127 (reproducer's own) | empty | empty |
| no `FD_CLOEXEC` | 3, not CLOEXEC | **clean EOF, no record** | **0** | **`LAUNCH-EXEC-01 script fixture ran with argv: /dev/fd/3 \n`** (56 bytes) | empty |

**What B shows** (indicative, one kernel): the documented split holds. The CLOEXEC arm fails
`ENOENT`, which is what X2b PASSed on. The non-CLOEXEC arm proceeds, `/bin/sh` runs, and `$0` is
`/dev/fd/3`. So the X2c *phenomenon* occurs, and the fixture's own line reports it — as free text
the frozen rule does not read.

### Diagnostic C — not run

`strace` is not installed in this WSL environment, and B left no ambiguity that a syscall trace
would resolve.

## 10. Linux ABI and documentation

No man pages are installed locally (`manpages-dev` absent), so the authoritative text was taken
from **man7.org, Linux man-pages 6.19 (2026-02-08)**. This is external documentation, separate from
repository evidence.

* **[`execveat(2)`](https://man7.org/linux/man-pages/man2/execveat.2.html)**:
  * `AT_EMPTY_PATH`: "If *path* is an empty string, operate on the file referred to by *dirfd*".
  * On scripts: "the argv[0] that is passed to the script interpreter is a string of the form
    /dev/fd/N or /dev/fd/N/P … A string of the first form occurs when AT_EMPTY_PATH is employed."
  * ERRORS, `ENOENT`: the program "requires the use of an interpreter program (such as a script
    starting with "#!"), but the file descriptor *dirfd* was opened with the O_CLOEXEC flag, with
    the result that the program file is inaccessible to the launched interpreter."
  * BUGS: "it is not possible to set the close-on-exec flag on the file descriptor given to a call
    of the form: execveat(fd, "", argv, envp, AT_EMPTY_PATH)", and the descriptor then "leaks
    through to the script itself".
  * Available since Linux 3.19.
* **[`execve(2)`](https://man7.org/linux/man-pages/man2/execve.2.html)**, *Interpreter scripts*: the
  interpreter is invoked as "*interpreter* [*optional-arg*] *pathname* *arg*...", where *arg* starts
  at `argv[1]`, and "there is no way to get the argv[0] that was passed to the **execve**() call".

**What Linux documents** for `execveat(fd, "", …, AT_EMPTY_PATH)` on a `#!` script:

* with `O_CLOEXEC` the call fails `ENOENT`;
* without it, the interpreter is launched with `/dev/fd/N` as its script argument;
* that name only works if the descriptor leaks into the interpreter, and `/dev/fd` is conventionally
  procfs-backed.

The man pages do not promise that the interpreter will succeed in opening that name. That depends
on procfs, and it is the product architecture's stated reason to exclude scripts. **None of this is
a HELM product contract.** HELM 0.1 refuses scripts at admission (X2, PASS in Trial #3), and X2c's
behaviour is recorded as a counterfactual, not adopted.

## 11. Product requirement versus control arm

* **Product.** ADR-0024 (lines 98–104) authorises exactly one already-open regular ELF in the
  cohort, executed by `execveat(exec_fd, "", …, AT_EMPTY_PATH)` with an `O_RDONLY | O_CLOEXEC`
  descriptor. Scripts are refused at admission in 0.1
  ([architecture](../research/HELM-LAUNCH-ARCHITECTURE.md) section 6: procfs dependency, a required
  non-CLOEXEC descriptor contradicting the inheritance invariant, and pathname-resolved
  `binfmt_misc` interpreters). The product mechanism **never** runs X2c's configuration: X2c needs
  two test-only spike flags, `--bypass-admission` and `--exec-fd-no-cloexec`, each of which removes
  a product invariant.
* **X2c is a counterfactual control arm.** With X2b it supports one explanatory claim behind the
  script refusal: *scripts are not refused because they cannot execute. Under the product's
  mandatory `O_CLOEXEC` they fail `ENOENT`, and making them work requires a leaked, non-CLOEXEC
  descriptor reached through `/dev/fd/N`.* X2 (refused, PASS) tests the product behaviour. X2b
  (ENOENT, PASS) tests the CLOEXEC arm. X2c was to show the other arm working.
* **Does failing to observe the counterfactual arm mean the product mechanism is defective? No.**
  * The product's refusal is established by X2.
  * Its reason is documented by `execveat(2)` and corroborated in-trial by X2b.
  * X2c's FAIL was produced by an evidence pipeline that could not return the predicted token
    whatever the mechanism did (Section 13).
  * An unobserved control arm weakens the *explanation's* in-trial corroboration; it does not show
    that the admitted-ELF mechanism misbehaved.

## 12. Layer-by-layer classification

| Layer | Classification | Boundary and proof |
|---|---|---|
| **A. Product mechanism** | **`MECHANISM_NOT_IMPLICATED`** | X2c exercises a configuration the product forbids (two test-only flags). Its FAIL is invariant to launcher behaviour across every record-free outcome, whether the script ran, failed to open `/dev/fd/N` or wrote nothing (Section 13; Diagnostic A S1–S4). What the preserved record does show — no pre-exec failure record and a clean EOF — is consistent with the documented kernel behaviour and with CLAIM 1. |
| **B. Frozen expectation** | **`EXPECTATION_EVIDENCE_MISMATCH`** | The predicted *phenomenon* is sound: `execveat(2)` documents it, and Diagnostic B shows it. The *expectation as frozen* binds that phenomenon to a structured helper report with an argv list, and X2c's executed object cannot produce one. It is not `EXPECTATION_TOO_STRONG`: the fact is observable, just not through the declared evidence source. |
| **C. Observability contract** | **`OBSERVABILITY_INCOMPLETE`** | The conservative exec-proof policy is sound and must stay. The admissible positive-evidence set (helper report, recipe payload, O6/O7 FIFO signal) has no member a non-helper image such as a `#!` script can supply, and the contract names no alternative for X2c. The parsers, the decisive-without-report rule and the sanitiser all behaved exactly as specified. |
| **D. Harness / case construction** | **`STATIC_POSABILITY_DEFECT`** and **`FIXTURE_DEFECT`** | *Static posability:* the gate treats "the launcher transports descriptor 1" as "this object emits a report", so it certified an unposable success rule (Section 5). *Fixture:* relative to its declared `report` channel and rule, `script_fixture.sh` emits free text, not the frozen report. Its bytes are correct for Linux semantics (Diagnostic B) and wrong for its own case contract. The harness's runtime behaviour — setup, mode, posing, observation derivation — was sound. |

## 13. Counterfactual (load-bearing)

**Question.** If the launcher mechanism and Linux behaved perfectly according to X2c's intended
semantics, could the frozen X2c evidence pipeline have produced `interpreter_ran_with_devfd` from
`script_fixture.sh` as written?

**Answer: NO.** Proof, from frozen code and fixture bytes only, independent of Diagnostic B:

1. The only return of `interpreter_ran_with_devfd` in the frozen tree is `observations.py` 1720.
   It requires a non-`None` report from `_report_or_decisive`.
2. A report exists only if descriptor 1's retained prefix contains the sentinel followed by a JSON
   object with a `marker` key (`parse_helper_report` 230–245), in `complete` state.
3. Under perfect behaviour, descriptor 1 carries only what `/bin/sh` writes while executing the
   fixture: one line of the form in Section 4.
   * That line cannot contain the sentinel: 8,192 variants checked, and structurally none begins
     with `HELM-`.
   * It cannot be a JSON object.
   * `$0` is kernel-assigned and `$@` is empty.
4. So the report is `None` for every possible correct execution. The rule then returns a lifecycle
   token (`ExecStatusIndeterminate` on a clean exit, an `ExecFailed:` token on a status record),
   `refused:…`, or `None` → INVALID. None of those is the frozen prediction.
5. Diagnostic A S1 executes steps 1–4 on the bytes a correct execution produced in Diagnostic B,
   and reproduces Trial #3's outcome and reason string exactly.

Only a launcher that injected bytes into the child's descriptor 1 could have produced the
predicted token, and that would itself be a defect.

**Therefore Trial #3's X2c FAIL cannot, by itself, establish a product launch-mechanism defect.**

## 14. Bounded future options (none implemented, none authorised)

| Option | Defect addressed | Product semantics | Frozen expectation | Launcher mechanism | New formal trial for a new acceptance claim? |
|---|---|---|---|---|---|
| **A.** The script emits its own structured argv evidence (sentinel + JSON object with its own non-helper `marker` and `argv` values), without pretending to be `helper_report` | fixture defect; closes the evidence mismatch for X2c | unchanged | token unchanged. The report contract gains a declared script marker, and the rule should require that marker rather than accept any object | unchanged | **yes**, as part of a new freeze. The gate should also check emitter capability, or the same class stays open |
| **B.** A purpose-built compiled interpreter helper (`#!<absolute build path>/helper_interp`) that writes a report with its argv | fixture defect; stronger than A, because the emitter is a measured, compiled helper | unchanged. The absolute interpreter path is a control-only dependency, never product | token unchanged; new helper in the report contract | unchanged. New C source, so a new compile-only build | **yes** |
| **C.** An independent external observation: trace X2c, and observe `execve` of the interpreter with a `/dev/fd/N` argument | observability incomplete, without trusting the script's output | unchanged | rule changes to a trace rule; traced set 8 → 9 | unchanged | **yes** |
| **D.** Remove X2c, or reclassify it as a recorded/documentary control, if the owner judges that X2 + X2b + `execveat(2)` suffice for the script-refusal claim | removes an unposable mandatory case | unchanged | membership or class changes (prospective only) | unchanged | **yes** for any new aggregate. The claim itself would then rest on documentation plus X2b |
| **E.** Change the product mechanism, e.g. infer exec from clean EOF or admit scripts | **none demonstrated** — X2c does not implicate the mechanism | would change product semantics and weaken the S4 policy | — | changes | not supported by these findings |

**Independent of the option chosen:**

* the static-posability gate should verify that each declared `CH_REPORT` plan's executed object
  is a known report emitter. This is the class fix for Section 5's false positive, and should be
  checked against every such plan, including `helper_setid`'s;
* the S4 conservative exec-proof policy should stay unchanged.

A new acceptance claim for LAUNCH-EXEC-01 requires a new freeze, independent review and a new owner
authorisation in every option. **This postmortem does not recommend Trial #4 because Trial #3 was
REJECTED**, and it does not choose among A–D.

## 15. R3-M1 boundary

R3-M1 is a future evidence-contract issue and did not cause X2c.

## 16. Conclusion

**X2C_PRIMARY_DIAGNOSIS:**
X2c's frozen success rule can be satisfied only by a structured helper report behind the frozen
sentinel, which `script_fixture.sh` as frozen can never emit, and the static-posability gate
certified the case because the launcher transports a report channel. X2c's Trial #3 FAIL was
therefore fixed by case construction for every launcher behaviour that writes no pre-exec status
record, and it carries no information about the launch mechanism.

**PRODUCT_MECHANISM:**
`MECHANISM_NOT_IMPLICATED`

**FROZEN_EXPECTATION:**
`EXPECTATION_EVIDENCE_MISMATCH`

**OBSERVABILITY:**
`OBSERVABILITY_INCOMPLETE`

**HARNESS:**
`STATIC_POSABILITY_DEFECT` (the gate), with `FIXTURE_DEFECT` (the fixture relative to its declared
evidence channel)

**TRIAL3_FROZEN_RESULT_REMAINS:**
`MECHANISM_REJECTED`

No postmortem finding changes Trial #3's frozen verdict, statuses, reasons, journal, evidence or
review.

## 17. Owner decision input

> **A. DO NOT AUTHORISE TRIAL #4.
> THE PRODUCT MECHANISM IS NOT IMPLICATED BY X2c; CORRECT THE TEST / OBSERVABILITY CONTRACT BEFORE
> ANY FURTHER ACCEPTANCE TRIAL IS CONSIDERED.**

No Trial #4 authority is granted by this recommendation. The owner's open choices are which of
options A–D to adopt for X2c, whether to adopt the emitter-capability posability check, and whether
the unassessed `helper_setid` report question belongs in the same prospective correction.

**TRIAL #3 FROZEN RESULT REMAINS MECHANISM_REJECTED**
**TRIAL #3 D-7 IS CONSUMED**
**LAUNCH-EXEC-01 TRIAL #3 VALID TRIAL COUNT IS ONE**
**TRIAL #3 MUST NOT BE RERUN**
**NO TRIAL #4 IS AUTHORISED**

## Validation

| Check | Result |
|---|---|
| `git diff --check` | clean |
| `python tools/validate_docs.py` | PASS |
| `python -m unittest discover -s tools/tests` (Windows) | 783 tests OK, 74 skipped |
| `cargo fmt --check` | pass |
| `run_launch_exec_01.py --verify-freeze` | `freeze_verified: true` |
| Scratch diagnostics | A and B run as recorded in Section 9, non-authoritative and not committed; C not run |
| New Trial #3 runs / reruns / retries | **0 / 0 / 0** |
| Formal trials / frozen cases posed | **0 / 0** |
| Frozen runner with D-7 / `launcher_spike` / helper ELF executions | **0 / 0 / 0** |
