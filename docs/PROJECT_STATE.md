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
