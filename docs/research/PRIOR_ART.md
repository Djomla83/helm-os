# Prethodni rad — početna mapa za EXP-001

**Status:** istraživački nacrt; nije izvršeno praktično poređenje.

| Oblast | Kandidat | Šta treba proveriti |
|---|---|---|
| Windows API/runtime | Wine | Aktuelni API obim, održavanje, testovi, minimalni patchset. |
| Steam igre | Proton | Integracija kroz Steam i dozvoljene kombinacije komponenti. |
| Grafika | DXVK / vkd3d-proton | Koji grafički backend traži konkretna aplikacija. |
| Upravljanje okruženjem | Bottles | Profili, runner verzije, integracije i mogućnost proširenja. |
| Pakovanje i sandbox | Flatpak i portali | Neophodne dozvole, GUI tokovi i njihove granice. |
| Source porting | Clang/CMake, Winelib/winegcc | Prenosivi build naspram Windows API zavisnosti. |
| Compiler istraživanje | MLIR | Da li postoji uska potreba koja opravdava novi IR. |
| OS image/update | Postojeće distribucione tehnologije | Izbor nakon zahteva i laboratorijskog baseline-a. |

**Ažurirano 2026-09-06:** ova mapa je popunjena u
[FOUNDATION_AUDIT.md](FOUNDATION_AUDIT.md), koji za svaki red dodaje primarni izvor, datum provere,
verziju, USE AS-IS / EXTEND / REPLACE LATER / BUILD NEW ocenu i poznata ograničenja. Audit je
desk research; nijedan aplikacioni test nije izvršen. Tabela ispod ostaje kao prvobitni nacrt
obima i nije brisana.

Za svaki red EXP-001 treba da doda: primarni izvor, datum provere, verziju kada je
važna, mogućnost reuse/extension, poznata ograničenja i plan praktičnog testa.

Ne tvrdimo originalnost same ideje upravljanja prefixima, profilima ili verzijama
runtime-a. HELM mora pokazati dodatnu vrednost u pouzdanosti, održavanju i
korisničkom toku. Izvori su u [SOURCES.md](../SOURCES.md).
