# Planovi eksperimenata

The table began as a planning snapshot. EXP-001 desk research and Gate 0 mechanics evidence now
exist; no application compatibility test has been completed.

| ID | Plan | Status |
|---|---|---|
| EXP-001 | [Pregled prethodnog rada i izbor baseline-a](EXP-001.md) | Planned |
| EXP-002 | [Pouzdanost komunikacione aplikacije](EXP-002.md) | Planned |
| EXP-003 | [Kontrolisana instalacija i aktivacija](EXP-003.md) | Planned |
| EXP-004 | [Izolacija i desktop integracija](EXP-004.md) | Planned |
| EXP-005 | [Ažuriranje, migracija i oporavak](EXP-005.md) | Planned |
| EXP-006 | [Dodatni build target za proizvođača](EXP-006.md) | Planned |
| EXP-007 | [Doprinos ograničenog agent pool-a](EXP-007.md) | Planned |
| EXP-008 | [Uska runtime specijalizacija — odloženo](EXP-008.md) | Planned |
| EXP-009 | [Evidence Loop — Gate 0 i najmanji end-to-end PoC](EXP-009.md) | Gate 0 delimicno izvrsen |

EXP-001 je izvršen kao desk research; rezultat je [foundation audit](../research/FOUNDATION_AUDIT.md).
Nije izvršen nijedan aplikacioni test. EXP-009 je predlog najmanjeg PoC-a koji proizlazi iz tog
audit-a i objedinjuje delove EXP-003, EXP-004 i EXP-005 u jedan merljiv tok; njegov Gate 0 takođe
prikuplja dokaze potrebne za EXP-002.

Current Gate 0 assessment: G0-2 PASS for the controlled, narrow HRESULT comparison across pinned
vanilla, staging and Proton configurations. Historical vanilla and the initial Proton attempt
remain INCONCLUSIVE. G0-1, original G0-3, G0-4 and G0-5 remain BLOCKED. G0-3a and G0-3b separately
PASS only for their recorded mechanics. See
[EXP-009-GATE0-REPORT.md](EXP-009-GATE0-REPORT.md#current-assessment). Work stops at the Gate 0 review
boundary; the PoC is not authorised.
Ostali eksperimenti prema preduslovima i [backlog-u](../../planning/BACKLOG.md).

Subsequent owner decision, 2026-09-07: G0-2 accepted only for its controlled HRESULT comparison.
The separately authorised [A0-7ZIP desktop application baseline](EXP-009.md#application-baseline)
has a [preparation report](EXP-009-APP-BASELINE-REPORT.md). It is BLOCKED on Hyper-V permissions;
no application workflow ran. This does not authorise the larger PoC or any other Gate 0 check.
