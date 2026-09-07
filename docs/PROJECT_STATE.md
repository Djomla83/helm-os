# Stanje projekta

**Datum:** 2026-09-06. **Faza:** G0, dokumentacija i priprema istraživanja.

| Oblast | Stanje |
|---|---|
| Vizija i zahtevi | Nacrt pripremljen; principi izdvojeni iz razgovora. |
| Arhitektonske odluke | Predlozi; imenovana ljudska odobrenja nisu unesena. |
| Master dokument | Postoji u ovom paketu. |
| AI uputstva i backlog | Postoje u ovom paketu. |
| Osam eksperimenata | Planovi, bez izvršenih app testova. |
| Kohorta aplikacija | Kandidati; svi `not_tested`; verzije i hardver nisu fiksirani. |
| OS / ISO / App Forge | Nisu implementirani. |
| Developer SDK / novi jezik | Nisu implementirani. |
| Windows VM integracija | Nije testirana. |
| Kompatibilnost i performanse | Nepoznato; nema projektnih merenja. |
| Lokalni validator | Samo dokumentacija i strukturirani primeri. |
| Udaljeni GitHub repo | Nije kreiran kroz dostupne akcije ove sesije. |
| Open-source licenca | Predlog, čeka odluku vlasnika. |

GitHub veza je proverena čitanjem naloga i dostupnih repozitorijuma. Dostupni skup
akcija nije pružio kreiranje novog repozitorijuma, a lokalno nije dostupan
autorizovan GitHub CLI. Nijedan postojeći projekat nije menjan.

Ovaj zapis je vremenski snapshot. Nakon stvarne objave dodati provereni URL,
commit SHA i datum; ne izmišljati ih unapred. Nakon eksperimenta dodati evidence
referencu i promeniti odgovarajući status, bez brisanja prvobitne istorije.

Izvršene dokumentacione provere opisane su u [VALIDATION.md](VALIDATION.md).

---

## Update 2026-09-06 — foundation audit completed (EXP-001, desk research)

Written in English by explicit owner instruction; this deviates from the single-language rule in
`CONTRIBUTING.md`, which the owner should either amend or override deliberately. The Serbian
snapshot above is unchanged and is retained as project history.

### What changed

| Area | State |
|---|---|
| Prior-art review (HELM-003 / EXP-001) | **Done as desk research.** Result: [FOUNDATION_AUDIT.md](research/FOUNDATION_AUDIT.md) — twelve domains reviewed against primary sources, plus two adversarial cross-checks. |
| Reuse matrix | Done. USE AS-IS / EXTEND / REPLACE LATER / BUILD NEW across roughly sixty components, in [audit §4](research/FOUNDATION_AUDIT.md#s04). |
| Novelty analysis | Done. Four of five claimed pillars already ship elsewhere; six genuinely unoccupied areas identified, in [audit §5](research/FOUNDATION_AUDIT.md#s05). |
| Risk register | Extended. Ten highest-risk assumptions with a cheap falsifying test each, in [audit §8](research/FOUNDATION_AUDIT.md#s08). |
| PoC proposal | Drafted. [EXP-009](experiments/EXP-009.md) with a Gate 0 falsification stage and pre-registered pass and fail criteria. |
| New architecture decisions | Seven drafts, ADR-0013 to ADR-0019, all `Proposed`. No existing ADR was modified. |
| Applications, ISO, App Forge, SDK, VM, language | **Still not implemented.** Unchanged. |
| Application tests, benchmarks, hardware matrix, sandbox audit | **Still not executed.** Unchanged. |
| Compatibility rate | **Still unknown.** No test has been run. |
| Remote GitHub repository | Still not created through the actions available in these sessions. |
| Open-source licence | Still a proposal awaiting the owner. |

### What was actually verified

Two facts were re-fetched by hand and are first-hand verified: the Wine 11.0 release statements on
WoW64 parity, the removed `wine64` loader, deprecated `WINEARCH=win32` prefixes, NTSync and the
"experimental Wayland driver"; and the state of Wine's `uiautomationcore.spec`, where the pattern
provider entry point is a stub while element discovery and property reads are implemented. The
documentation validator and its twenty unit tests pass. **Nothing else in the audit was verified by
running software**, and [audit §11](research/FOUNDATION_AUDIT.md#s11) lists the claims that did not
survive cross-checking, including one load-bearing measurement that is graded LIKELY rather than
VERIFIED.

### Findings that change the plan

1. The compatibility substrate is mature and must not be rebuilt.
2. Four of the five named pillars — per-application environments, dependency management,
   confinement plumbing and automated environment derivation — are already shipped by an incumbent
   under an open licence.
3. The only defensible differentiator is reproducible, version-scoped, expiring evidence, plus
   recovery that attributes a failure to one axis and never reverts user documents.
4. **The motivating example does not support the proposed architecture.** Communication-application
   unreliability on Linux is, on the available evidence, not a Wine problem. See
   [audit §6](research/FOUNDATION_AUDIT.md#s06).
5. The real thesis under test is economic: whether automation can substitute for the QA labour the
   funded incumbents pay for. It is currently unevidenced.

### Decisions now needed from the owner

Base distribution and exact hardware; the language rule for documentation; a named owner, reviewer
and ADR approver; whether the project funds or performs upstream Wine work; the licence policy, now
with additional inputs recorded in the audit; and the Track A versus Track B first-product question,
which [ADR-0019](adr/ADR-0019-scope-boundary.md) proposes to settle with the Gate 0 result rather
than by argument.

### Next step

Run Gate 0 of [EXP-009](experiments/EXP-009.md) — three to five days, one machine, no new spend —
and publish the result whatever it says. Do not begin catalogue, GUI, OS-image or SDK work before
that.

---

## Update 2026-09-07 — audit corrected, Gate 0 partially executed

### Product mission, restated because the audit does not replace it

HELM's goal is a free, open-source, user-controlled desktop with reliable application experiences.
**Windows compatibility is one mechanism toward that goal, not the goal**, and the proposed Evidence
Loop is an enabling subsystem, not a substitute for the product vision. Nothing below changes the
mission; it changes what is claimed to be known.

### Audit corrected to revision 2

Ten corrections are recorded in [audit §13](research/FOUNDATION_AUDIT.md#s13), written **before** any
result was collected. The systematic error was grading "I read this in a source" and "this was
observed to behave this way" identically; the audit now separates **VERIFIED_SOURCE** from
**VERIFIED_EXECUTION**. The substantive corrections:

- A factual error is fixed and, more importantly, the inference built on it is withdrawn:
  **bug-report counts do not measure usage**. The cause of the reported Viber experience is
  **unestablished** — which is not the same as the earlier claim that it "is not a Wine problem".
- The claim that a 2026 tray regression explained the user's instability is **withdrawn**; it was
  never reproduced, and the affected code path is not present in that client.
- "Four of five pillars already shipped" is replaced by the accurate form: **the capabilities exist,
  the verification loop does not**.
- The evidence pillar's novelty is roughly **halved**. openQA already ships traceability, artifact
  collection, a closed result vocabulary, reproducible rerun, non-pixel oracles and last-good
  attribution. Two genuine absences remain, plus two schema-shape fixes.
- The UI Automation conclusion is scoped to a pinned tag, to pattern-based actuation, and is
  **refuted as a universal claim** by a shipping counter-example.
- Snapshot **contamination** is separated from **corruption**; one backup tool was wrongly accused
  and is withdrawn.

### Gate 0: 1 PASS, 4 BLOCKED, 0 FAIL

Full report: [EXP-009-GATE0-REPORT.md](experiments/EXP-009-GATE0-REPORT.md). Environment record:
[`G0-environment-2026-09-07.json`](experiments/evidence/G0-environment-2026-09-07.json).

The available environment is Windows 11 with WSL2 Ubuntu 24.04 and **no Wine, no compiler, no
sandbox tooling, no desktop environment and no portal backend**. Four checks are therefore BLOCKED
on an approved lab, and are recorded as blocked rather than answered by inference.

The one executed check produced two results worth acting on:

1. **Hardlink farms are confirmed unusable as prefix snapshots** — the copy silently tracks live
   data, deterministically, with no crash required.
2. **Reflink copies were unsupported on both filesystems tested**, including a default Ubuntu
   install. **ADR-0017 proposed reflink as the safe mechanism**, so that ADR is contradicted by
   execution and must be amended.

No software was installed, no money spent, no persistent host change made, and zero manual
interventions were required.

### Decision status

| Item | State |
|---|---|
| ADR-0013 to ADR-0020 | **All `Proposed`. None accepted.** ADR-0015 and ADR-0017 carry recorded corrections that must be applied before review. |
| Evidence Loop PoC | **Not authorised.** Four of five Gate 0 checks are blocked on the same precondition. |
| Track A / Track B first-product decision | **Still open**, and must not be settled by argument — its deciding input (G0-1) is blocked. |
| Documentation language | [ADR-0020](adr/ADR-0020-documentation-language.md) now **proposes** a `CONTRIBUTING.md` amendment instead of the current silent override. |
| Application tests, benchmarks, compatibility rate | **Still none. Still unknown.** |

### Next step

One decision from the maintainer: **whether to authorise a lab**, and where it runs. That single
item unblocks four of the five Gate 0 checks. WSL2 can unblock the Wine-mechanics checks; it cannot
unblock the integration or graphics questions, and results obtained there must not be presented as
if it could.


---

## Update 2026-09-07 (later) — bounded lab phase executed

Owner authorised a bounded Gate 0 lab phase. Full detail:
[Gate 0 report](experiments/EXP-009-GATE0-REPORT.md) and
[lab runbook](experiments/LAB-G0-RUNBOOK.md).

### Language policy — decided

[ADR-0020](adr/ADR-0020-documentation-language.md) is **Accepted** with named owner approval, and is
the authoritative specification for the topic. English is primary for new technical specifications,
ADRs, RFCs, experiment reports and developer-facing instructions; existing Serbian material is
preserved as history and originating requirements; the repository is **not** translated now; and
each topic has exactly one authoritative document that others link to. `CONTRIBUTING.md` now
summarises this and points at the ADR. **No architecture ADR was accepted.**

### Lab provisioned

The preferred VM path was **not available**: Hyper-V's role is enabled but the account lacks
permission, and creating a VM would require a host administrator change that the authorisation
excludes. The authorised fallback was used — a single disposable WSL2 distribution `helm-lab-g0`,
isolated from Windows drives and interop, running as an unprivileged user. Host changes: one new
directory and one new WSL distribution. Nothing else.

### Results: 3 PASS, 1 PARTIAL, 3 BLOCKED, 0 FAIL

| Check | Outcome |
|---|---|
| G0-3a filesystem mechanics | PASS (round 1; reported as a newly added subtest, **not** as satisfying the originally registered criterion) |
| G0-3b Wine registry behaviour | **PASS with real Wine 11.17** — hardlink copy contaminated, quiesced full copy independent, both controls correct |
| G0-2 DirectComposition | **PARTIAL** — vanilla Wine returns E_NOTIMPL; the staging and Proton arms were not tested, so the comparison is unresolved |
| G0-1, G0-4, G0-5 | BLOCKED — need authorised accounts, a desktop session, and a reviewed containment boundary respectively |

**The most consequential result:** a synthetic user document created after the recovery point did
**not** survive a whole-prefix restore. That is the failure ADR-0017 rule 2 exists to prevent, now
demonstrated rather than argued.

### Decision status

| Item | State |
|---|---|
| ADR-0020 | **Accepted** 2026-09-07 (documentation language only) |
| ADR-0013 to ADR-0019 | **All still `Proposed`.** ADR-0017 amended on owner instruction and is now the best-evidenced; ADR-0015 still needs its novelty claim narrowed. |
| Evidence Loop PoC | **Still not authorised.** Its central thesis needs real applications and a desktop session. |
| Track A / Track B | **Still open.** G0-1 stayed blocked; a lab does not supply authorised accounts or a reproduction. |
| Application compatibility rate | **Still unknown.** No Windows application was installed or run. |

### Next step

Finish G0-2's remaining two runtime arms in a second disposable lab (about an hour), take amended
ADR-0017 to review, and decide the environment for application-level work — a desktop session on
real hardware, or the Hyper-V group approval recorded in the runbook.
