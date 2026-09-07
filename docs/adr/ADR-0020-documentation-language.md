# ADR-0020: Documentation language policy — proposed amendment to CONTRIBUTING.md

**Status:** Proposed\
**Draft date:** 2026-09-07\
**Approver:** not entered\
**Acceptance date:** not entered

## Context

`CONTRIBUTING.md` states that documentation is maintained in Serbian, latinica, in this initial
revision, and that translation for a wider community should come through a separate coordinated
process rather than maintaining several contradictory normative specifications.

Since 2026-09-06 that rule has been overridden in practice: the foundation audit, eight ADRs, two
experiment documents and part of `PROJECT_STATE.md` are in English, on the owner's instruction given
in conversation. The repository is therefore currently in a state its own contribution rules
prohibit, on the strength of an instruction recorded nowhere in the repository.

That is the problem this record exists to fix. A rule that is silently overridden is worse than
either following it or changing it, because a later contributor cannot tell which language a new
document should be written in, and cannot tell whether the Serbian documents are still normative.

## Options considered

1. **Translate the new English documents to Serbian and restore the rule.** Consistent, and it
   preserves a single normative language. Costs a substantial translation of technical material
   whose primary sources are all English, and the translation would need re-doing on every revision.
2. **Switch the project to English entirely.** Consistent, and it matches the language of every
   upstream project, specification and bug tracker the work depends on. Costs a retranslation of the
   existing Serbian corpus, including the master plan, which is the largest document in the
   repository.
3. **Adopt an explicit two-tier policy**: Serbian for the founding product documents, English for
   research and technical records, with one rule stating which is normative when they disagree.
4. **Leave it undefined.** Rejected: this is the current state, and it is the defect.

## Proposed decision

Adopt option 3, and amend `CONTRIBUTING.md` accordingly. This is a proposal; the owner decides, and
option 1 or 2 remains open.

Proposed replacement for the language paragraph in `CONTRIBUTING.md`:

> Dokumentaciju održavamo dvojezično, sa jasnom podelom i jednim pravilom prvenstva.
>
> **Srpski, latinica** — proizvodni i upravljački dokumenti: `README.md`, `HELM_MASTER_PLAN.md`,
> `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, `LICENSE-DECISION.md`, `CHANGELOG.md` i indeksi u
> `docs/`. Ovi dokumenti su **normativni**.
>
> **Engleski** — istraživački i tehnički zapisi: `docs/research/`, `docs/adr/`,
> `docs/experiments/` i `tools/`. Ovi dokumenti citiraju primarne izvore koji su na engleskom, i
> namenjeni su i saradnji sa upstream projektima.
>
> **Pravilo prvenstva:** ako se normativni dokument i tehnički zapis razlikuju, važi normativni
> dokument, a razlika se rešava u istom PR-u. Prevod postojećih dokumenata nije uslov za doprinos.

Rationale for the split. The technical records cite primary sources that are exclusively in English,
are the documents most likely to be shared with upstream projects when reporting a defect or
proposing a patch, and change most often. The product documents define what the project promises and
who decides, are read first by anyone new, and should stay in the founder's language. The precedence
rule prevents the two tiers from becoming two competing specifications, which is the outcome the
original rule was written to avoid.

## Consequences

The repository becomes bilingual by design rather than by accident. A contributor gets an
unambiguous answer to "which language does my document use". The existing English documents become
compliant without retranslation, and the existing Serbian documents remain normative and unchanged.

The cost is real: a Serbian-speaking reader who does not read English loses direct access to the
technical records, and the project takes on a small permanent obligation to keep the two tiers from
contradicting each other. If the owner judges that cost too high, option 1 is the correct choice and
this ADR should be rejected in favour of translating the English documents.

This ADR does not itself modify `CONTRIBUTING.md`. The amendment applies only on acceptance.

## Evidence

The current state of the repository: [FOUNDATION_AUDIT.md](../research/FOUNDATION_AUDIT.md),
ADR-0013 to ADR-0019 and this record are in English; `CONTRIBUTING.md` and the master plan are in
Serbian. The deviation is flagged in the audit header, in
[PROJECT_STATE.md](../PROJECT_STATE.md) and in `CHANGELOG.md` 0.2.0, but was never proposed as a
decision until now.

## Revisiting

The project acquires contributors who need a single language; or the volume of Serbian technical
material grows enough that the split stops paying for itself; or the owner decides the master plan
should be translated, at which point option 2 becomes cheap and this record is superseded.

This record is not human approval. Changing the status to `Accepted` requires the name and role of
an approver, a date and a review reference; the draft date above is not an acceptance date.
