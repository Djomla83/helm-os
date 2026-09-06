# Prvi zadatak razvojnom agentu

**Radni paket:** HELM-003 / EXP-001.\
**Cilj:** proveriti prethodni rad i predložiti najmanji eksperiment koji može dokazati
korist HELM pristupa iznad postojećeg Linux/Wine workflow-a.\
**Status:** spreman za dodelu; nije izvršen.

## Kontekst

Želimo besplatan, otvoren desktop sistem sa pouzdanim aplikacijama i kontrolom
korisnika. Korisnički problem je nestabilno ili nedovoljno uglađeno iskustvo nekih
aplikacija. Nismo dokazali uzrok problema. Ne pretpostavljaj da native Linux Viber
radi kroz Wine. Ne pretpostavljaj da Windows izdanje nužno radi bolje.

Pročitaj [AGENTS.md](AGENTS.md), [master plan](HELM_MASTER_PLAN.md),
[stanje projekta](docs/PROJECT_STATE.md), [odluke](docs/DECISIONS.md),
[backlog](planning/BACKLOG.md) i [eksperiment](docs/experiments/EXP-001.md).

## Dopušteni obim

Istraživanje zvanične dokumentacije, pregled javnog upstream koda kada je potreban,
ažuriranje istraživačke mape i planiranje testa. Bez nepoznatih instalera,
produkcijskih promena, novih troškova i javnog izdavanja.

Predloženi fajlovi za promene: `docs/research/PRIOR_ART.md`,
`docs/experiments/EXP-001.md` i novi `docs/experiments/EXP-001-REPORT.md`.
Ne prihvataj ADR samostalno.

## Isporuka

Napravi poređenje direktnog Wine workflow-a sa najmanje jednim postojećim manager-om.
Obuhvati runtime verzionisanje, konfiguraciju, izolaciju, portal integraciju,
ažuriranje, evidence model i mogućnost proširenja. Navedi šta je zaista potvrđeno
izvorom, a šta zahteva praktičan test.

Predloži reuse/extend/build mapu. Izdvoji jednu korisničku vrednost koja ne ostaje
samo drugačiji launcher ili ime paketa. Definiši najmanji test za tu vrednost:
ulazi, oprema, očekivanje, kriterijum neuspeha i budžetska granica.

Navedi tačnu listu nedostajućih odluka: baza, app verzija, hardver, test nalog/licenca
ili prava pristupa. Ne izmišljaj ih da bi izveštaj izgledao kompletno.

## Kriterijum završetka

Druga osoba može iz izveštaja razumeti zašto predlog ima smisla, šta preuzimamo iz
postojećeg ekosistema i šta bi eksperiment mogao da opovrgne. Nema tvrdnje da smo
implementirali App Forge, postigli kompatibilnost ili izmerili performanse.

Pokreni `python3 tools/validate_docs.py` i dostupne testove validatora.
To navedi kao proveru dokumentacije, ne aplikacija. Predaj mali PR/diff i stani.
