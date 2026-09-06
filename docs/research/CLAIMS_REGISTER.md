# Registar ključnih tvrdnji i hipoteza

| ID | Tvrdnja | Status | Potrebna provera |
|---|---|---|---|
| H-01 | Profili smanjuju ručno podešavanje aplikacija | Hipoteza | Poređenje identičnog toka sa baseline manager-om. |
| H-02 | Verzionisanje i transakcije smanjuju regresije | Hipoteza | Update/recovery test i duže praćenje. |
| H-03 | SDK olakšava dodatni target prenosivim projektima | Hipoteza | Stvarni diff, build i funkcionalni test. |
| H-04 | Statička specijalizacija donosi dodatne performanse | Odložena hipoteza | Profiling i kontrolisano poređenje. |
| H-05 | Novi jezik je bolji izbor od postojećih alata | Odložena hipoteza | Definisan problem i benchmark razvoja/ispravnosti. |
| H-06 | Windows VM može dati prihvatljivo iskustvo | Odvojena hipoteza | Licenca, uređaji, performanse i sigurnosna integracija. |
| H-07 | Mali agent pool povećava prihvaćeni učinak | Hipoteza | Trošak, review vreme, regresije i zadržane promene. |
| G-01 | 95% definisane kohorte je dugoročni cilj | Cilj, ne rezultat | Zaključana kohorta i izvršeni testovi. |
| C-01 | HELM već pokreće Photoshop/Office/Viber | Nepotkrepljena tvrdnja | Ne objavljivati; nema naših testova. |
| C-02 | Svaki EXE se može bez rada nativno prevesti | Neusvojeno obećanje | Ne koristiti u specifikaciji ili marketingu. |

Raniji brojevi vremena, veličine runtime-a, procenti i buduće verzije u razgovoru
bili su ilustracije. Ne predstavljaju početni benchmark ili podatke projekta.

## Dodato posle foundation audit-a (2026-09-06)

Sve stavke ispod potiču iz [FOUNDATION_AUDIT.md](FOUNDATION_AUDIT.md) i predstavljaju desk research,
ne rezultat testa. Nazivi su na engleskom radi poklapanja sa audit-om.

| ID | Tvrdnja | Status | Potrebna provera |
|---|---|---|---|
| H-08 | Automation can substitute for the paid per-application QA labour of the incumbents | Hipoteza — glavna produktna teza | Izmeriti false-pass rate najslabijeg signala na tri aplikacije (EXP-009, kriterijum P2). |
| H-09 | Version-scoped profiles reduce silent breakage of applications that auto-update | Hipoteza | Izmeriti poluživot recepta: 10 workaround-a sa Wine 9.x ponovo testirati na 11.17, sa i bez tweak-a. |
| H-10 | Attributable, tiered recovery can revert one axis without touching user documents | Hipoteza | EXP-009, kriterijumi P4 i P5. |
| H-11 | Business/productivity applications are a harder target than games, not an easier one | Radna hipoteza suprotna master planu | Klasifikovati 20 ciljanih aplikacija po toolkit-u, .NET meti, instaleru, licenciranju i servisima. |
| C-03 | Communication-app unreliability on Linux is caused by the Windows compatibility layer | Nepotkrepljena tvrdnja | Dokazi ukazuju na suprotno; videti [audit, poglavlje 6](FOUNDATION_AUDIT.md#s06) i Gate 0 / G0-1. |
| C-04 | Automated derivation of an application environment from its binary is HELM-ova novina | Odbačena kao novina | Postojeća implementacija objavljena 2026-01-24 kod druge alatke; proveriti praktično kroz G0-4. |
