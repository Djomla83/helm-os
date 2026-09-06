# Doprinos projektu

Projekat je u fazi nacrta. Pre prihvatanja spoljnih doprinosa potrebno je usvojiti
[licencnu politiku](LICENSE-DECISION.md). Ne podrazumevamo preneto autorstvo niti
pravo da objavimo tuđ poverljiv sadržaj.

## Tok promene

Najpre proverite [backlog](planning/BACKLOG.md) i [odluke](docs/DECISIONS.md).
Otvorite usko pitanje ili preuzmite dodeljen zadatak. Značajnu promenu predložite
kroz RFC; rezultat odluke zapišite u ADR. Mali ispravci teksta ne zahtevaju veliki RFC.

Jedan PR treba da rešava jednu jasnu stvar. Navedite izvor, pretpostavke, provere,
uticaj na povezane dokumente i status odluke. Ne mešajte eksperimentalnu hipotezu
sa rezultatima implementacije.

## Dokazi

Koristite zvaničnu dokumentaciju, kod projekta ili ponovljiv eksperiment.
Kompatibilnost važi za konkretno izdanje, hardver i tok, ne za ime aplikacije zauvek.
Sintetički primeri moraju biti označeni. Preskočen ili blokiran test nije uspeh.

Izvor i licenca svakog preuzetog dela moraju biti poznati. AI pomoć ne ukida obavezu
pregleda i provere porekla. Ne doprinosite procureli vlasnički kod, lične podatke
ili komercijalne artefakte koje nemate pravo da distribuirate.

## Provere pre PR-a

```bash
python3 tools/validate_docs.py
python3 -m unittest discover -s tools/tests -v
```

Dokumentaciju održavamo na srpskom, latinicom, u ovoj početnoj reviziji.
Prevod za širu zajednicu može doći kroz poseban usklađen proces, bez održavanja
više kontradiktornih normativnih specifikacija.

## Review

Predlozi koji menjaju licence, autorizaciju, sigurnosne granice ili javna obećanja
zahtevaju imenovano ljudsko odobrenje. PR nije odobren zato što ga je agent napisao
ili zato što lokalni validator prolazi. Pravila grane na GitHub-u moraju posebno
biti podešena; ovaj tekst ih ne aktivira.
