# ADR-0020: Documentation language policy

**Status:** **Accepted**\
**Draft date:** 2026-09-07\
**Approver:** Repository owner (Djomla83), by written instruction of 2026-09-07\
**Acceptance date:** 2026-09-07

> This is the **authoritative specification** for the documentation language policy. Any other
> document that describes this topic — including `CONTRIBUTING.md` — links here rather than
> restating the rules, so the two cannot silently diverge.

## Context

`CONTRIBUTING.md` originally required documentation in Serbian, latinica. From 2026-09-06 that rule
was overridden in practice: the foundation audit, the ADR series and the experiment documents were
written in English on the owner's verbal instruction, recorded nowhere in the repository. A rule
that is silently overridden is worse than either following it or changing it, because a contributor
cannot tell which language to use or which document is normative.

An earlier draft of this ADR proposed a different policy (Serbian normative for product documents,
English for technical records). **That draft is superseded by the owner's decision below**, which
differs from it: the decision makes English primary for new technical material and treats the
Serbian corpus as history and originating requirements, rather than as the normative tier.

## Decision

Approved by the owner on 2026-09-07:

1. **English is the primary language** for new technical specifications, ADRs, RFCs, experiment
   reports and developer-facing instructions.
2. **Existing Serbian material is preserved** as project history and as the originating
   requirements. **The repository is not translated now.**
3. **Each topic has exactly one clearly identified authoritative specification.** Translations and
   historical documents must **link to it** rather than restating it and silently diverging.

### Applying rule 3

| Topic | Authoritative document | Status of the other documents |
|---|---|---|
| Product vision, scope, originating requirements | [`HELM_MASTER_PLAN.md`](../../HELM_MASTER_PLAN.md) (Serbian) | Originating requirements. Preserved. Where a later English ADR supersedes a specific point, that ADR is authoritative **for that point** and says so. |
| Architectural decisions | The individual ADR in `docs/adr/` (English) | The index in [`DECISIONS.md`](../DECISIONS.md) is a pointer list, not a specification. |
| Experiments and their results | The `EXP-*` document and its report in `docs/experiments/` (English) | — |
| Research findings | [`FOUNDATION_AUDIT.md`](../research/FOUNDATION_AUDIT.md) (English) | [`PRIOR_ART.md`](../research/PRIOR_ART.md) links to it rather than duplicating it. |
| Contribution rules | [`CONTRIBUTING.md`](../../CONTRIBUTING.md) (Serbian) | Authoritative for contribution process; for the **language policy specifically** it links to this ADR. |
| Agent rules | [`AGENTS.md`](../../AGENTS.md) (Serbian) | Preserved. New agent-facing instructions are written in English. |

### What this does not mean

- It does **not** authorise translating or rewriting existing Serbian documents.
- It does **not** demote the master plan. It remains the originating requirements document.
- It does **not** apply retroactively: no existing document must be rewritten to comply.

## Consequences

New technical material is written in the language of the primary sources it cites and of the
upstream projects it must interoperate with, which lowers the cost of reporting defects and
proposing patches upstream. The Serbian corpus stays readable and authoritative for what it covers.

The cost is a bilingual repository, and a standing obligation that every topic names one
authoritative document. A Serbian-speaking reader who does not read English loses direct access to
new technical records; the owner accepted that trade explicitly.

## Evidence

The owner's written instruction of 2026-09-07 states the three rules verbatim. The prior state —
English documents in a repository whose contribution rules required Serbian — is visible in the
2026-09-06 commits and was flagged in the audit header, `PROJECT_STATE.md` and `CHANGELOG.md` 0.2.0
without ever being proposed as a decision.

## Revisiting

The project acquires contributors for whom a single language is necessary; or the owner decides to
translate the master plan, at which point a single-language policy becomes cheap and this record is
superseded by a new ADR.

**This record carries a named human approval and is accepted.** No other ADR in this repository is
accepted; ADR-0013 to ADR-0019 remain `Proposed` and this decision does not change their status.
