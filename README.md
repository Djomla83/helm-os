# HELM OS — dizajn i istraživanje

**Vaš računar. Vaš izbor. Pouzdane aplikacije.**

HELM OS je radni naziv projekta besplatnog, otvorenog desktop sistema zasnovanog
na Linux ekosistemu, sa fokusom na kvalitet svakodnevnih aplikacija, korisničku
kontrolu i proverljivu podršku za odabrani Windows softver.

> **Trenutni status: eksperimentalni proizvodni moduli i prihvaćena prva realna GUI vertikala.**
> Ovo još nije gotov OS niti produkciono izdanje: nema instalacionog ISO-a, produkciono prihvaćenog
> GUI-ja, App Forge-a, Wine/Proton integracije, SDK-a ili novog jezika. Cilj od 95% kompatibilnosti
> nije postignut niti izmeren.

**Implementation update, 2026-09-22:** bounded Rust modules
[helm-evidence](crates/helm-evidence/README.md), [helm-app-spec](crates/helm-app-spec/README.md),
[helm-observe](crates/helm-observe/README.md), [helm-bind](crates/helm-bind/README.md) and the
product-accepted [helm-launch 0.1](crates/helm-launch/README.md) are present in the repository.
The first real native GTK backend-connected G2 vertical is owner-accepted and integrated into
`main`; it remains an experimental vertical, not a production GUI. Current G2 status and limits
are summarised in the [documentation index](docs/INDEX.md). The
[post-G2 product review](docs/implementation/HELM-POST-G2-PRODUCT-REVIEW.md) records proposed next
gates and review findings; it authorises no implementation by itself.

## Počnite ovde

| Dokument | Namena |
|---|---|
| [Glavni projektni dokument](HELM_MASTER_PLAN.md) | Celovita vizija, zahtevi, arhitektura, rizici i plan rada. |
| [Prvi zadatak agentu](AGENT_STARTER.md) | Ograničen početni zadatak, izlazi i kriterijumi završetka. |
| [Pravila za agente](AGENTS.md) | Ovlašćenja, dokazi, bezbednost i review. |
| [Stanje projekta](docs/PROJECT_STATE.md) | Šta zaista postoji i šta nije testirano. |
| [Mapa dokumentacije](docs/INDEX.md) | ADR, RFC, eksperimenti i operativna uputstva. |
| [Backlog](planning/BACKLOG.md) | Pripremljeni zadaci i redosled zavisnosti. |

## Pravac

Prvo testiramo mali aplikacioni tok na postojećoj Linux osnovi. Ponovo koristimo
relevantne komponente, proveravamo pristup datotekama i integritet podataka i
merimo vrednost iznad postojećih rešenja. Tek nakon toga odlučujemo o posebnoj
OS slici, širem katalogu i developer SDK-u.

App Forge je predlog za analizu, pripremu i validaciju aplikacionog okruženja.
Nije obećanje da se proizvoljni `.exe` automatski pretvara u nativnu aplikaciju.
Novi jezik, generalno binarno prepisivanje i Windows VM nisu početni kritični put.

## Provera dokumentacije

Potrebni su Python 3.10 ili noviji i standardna biblioteka:

```bash
python3 tools/validate_docs.py
python3 -m unittest discover -s tools/tests -v
```

Ove komande proveravaju dokumentaciju i zapise. **Ne testiraju kompatibilnost
Windows aplikacija, sandbox ili performanse OS-a.**

[Izveštaj o izvršenim proverama](docs/VALIDATION.md) beleži obim i ograničenja.

## Doprinosi i objavljivanje

Pročitajte [CONTRIBUTING.md](CONTRIBUTING.md). Za novi predlog koristite RFC,
a za usvajanje značajnog izbora ADR. Važeći statusi odluka, uključujući prihvaćene ADR-ove,
nalaze se u [indeksu odluka](docs/DECISIONS.md).

[Postupak objavljivanja](docs/runbooks/PUBLISH.md) čuva operativni proces objavljivanja.
Autoritativni udaljeni repozitorijum ovog radnog stabla je `Djomla83/helm-os`; istorijski
tekstovi koji prethode njegovom kreiranju ostaju istorijski zapisi.

## Licenca i naziv

Licencna politika je [predlog koji treba odobriti](LICENSE-DECISION.md).
Ovaj nacrt još ne predstavljamo kao licencirano open-source izdanje.
HELM je privremen naziv; dostupnost brenda nije proverena. Nema tvrdnje o
povezanosti sa Microsoftom, Wine-om, Valve-om ili proizvođačima navedenih aplikacija.
