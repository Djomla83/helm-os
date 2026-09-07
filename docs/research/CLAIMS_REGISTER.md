# Registar ključnih tvrdnji i hipoteza

> Earlier tables are dated claim history. The [G0-2 completion](#g0-2-completion) below
> supplies the current Gate 0 assessment where it supersedes those entries.

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

## Ispravke posle Gate 0 (2026-09-07)

Statusi ispod menjaju ranije unose iz istog paketa. Puno obrazlozenje je u
[dnevniku ispravki](FOUNDATION_AUDIT.md#s13) i u
[Gate 0 izvestaju](../experiments/EXP-009-GATE0-REPORT.md).

| ID | Tvrdnja | Novi status | Napomena |
|---|---|---|---|
| C-03 | Nepouzdanost komunikacionih aplikacija uzrokovana je Windows compatibility slojem | I dalje nepotkrepljeno — **ali i suprotna tvrdnja je nepotkrepljena** | Uzrok je **neutvrdjen**. Broj bug prijava ne meri upotrebu; raniji zakljucak povucen. Simptom nije reprodukovan. |
| C-05 | Tray regresija objasnjava korisnikova rusenja | Povuceno | Nikada nije reprodukovano, a zahvaceni kod nije prisutan u tom klijentu. Dokumentovani simptom je ikona, ne rusenje. |
| H-12 | Evidence pillar je u celini nova vrednost | Znatno suzeno | openQA vec isporucuje traceability, prikupljanje artefakata, zatvoren recnik ishoda, ponovljivo pokretanje i atribuciju. Ostaju dva stvarna nedostatka i dve korekcije seme. |
| H-13 | Hardlink kopija prefiksa je neupotrebljiva kao snapshot | **VERIFIED_EXECUTION** (mehanika, sinteticki pisac) | Kontaminacija je determinisiticka i ne zahteva pad. Korupcija zivog prefiksa je odvojena tvrdnja i zahteva dodatni kvar. |
| H-14 | Reflink kopije su dostupan bezbedan mehanizam | **Opovrgnuto na testiranom hostu** | `cp --reflink` nije podrzan ni na jednom od dva testirana fajl sistema. ADR-0017 trazi izmenu. |
| H-15 | Pattern-based UIA automatizacija ne radi pod Wine-om | **VERIFIED_SOURCE** na fiksiranom tagu | Vazi za pattern actuation. Nije "ceo ekosistem": winetricks danas vodi GUI instalere pod Wine-om preko AutoHotkey-a. Nacin otkaza po klijentu je neizmeren. |

<a id="publication-review"></a>

## Publication review — 2026-09-07

Current status follows the [Gate 0 report](../experiments/EXP-009-GATE0-REPORT.md#current-assessment).
This is an evidence assessment, not a new experiment or a change to acceptance criteria.

| ID | Claim | Current assessment | Evidence and limit |
|---|---|---|---|
| H-13 | Mutable hardlink copies are not independent prefix snapshots | Recorded execution supports the narrow mechanics claim | G0-3a and G0-3b separately PASS for the recorded filesystem/recovery semantics. Original G0-3 remains BLOCKED; no corruption or successful application recovery was demonstrated. |
| H-14 | Reflink copies are available on this lab filesystem | Refuted on the tested filesystems | Recorded copy errors are preserved; ADR-0017 was amended but remains Proposed. |
| C-06 | The DirectComposition probe decides compatibility of an entire application class | Unsupported; historical inference withdrawn | Vanilla recorded `0x80004001` (`E_NOTIMPL`), but its arm is INCONCLUSIVE because required controls are not evidenced. Overall G0-2 is BLOCKED; staging and Proton/UMU are NOT_RUN. No rendering or application-class conclusion follows. |
| C-07 | The configured lab blocks Windows-process execution | UNVERIFIED | Configuration requests disabled interoperability, while `WSLInterop` registration reads `enabled`. No direct negative execution test was performed. |

The [publication manifest](../experiments/evidence/G0-publication-redaction-2026-09-07.json)
distinguishes private raw hashes from tracked redacted hashes. Privacy redaction changes no
experimental value outside the declared path fields.

<a id="g0-2-completion"></a>

## Controlled G0-2 completion — 2026-09-07

The [controlled report](../experiments/EXP-009-GATE0-REPORT.md#current-assessment) supersedes the
publication-stage G0-2 and interoperability assessments above. No architecture ADR is accepted.

| ID | Claim | Assessment | Evidence and limit |
|---|---|---|---|
| C-06 | This probe decides an application's or application class's compatibility | Still unsupported | No rendering or real application workflow ran. `S_OK` is only a successful return from the tested operation. |
| C-07 | Windows-process execution was unavailable in the tested lab invocation | VERIFIED_EXECUTION, limited to the three recorded negative attempts | Original lab and both clones returned interop connection failure/exit 1 without a marker. Configuration and enabled shared binfmt registration are separately recorded. This is not G0-5 or a sandbox certification. |
| C-08 | The tested DirectComposition call returns different HRESULTs across the pinned configurations | VERIFIED_EXECUTION | Controlled Wine 11.17 and UMU-Proton-10.0-4 returned `E_NOTIMPL`; staging 11.16 returned `S_OK`. All final controls were correct. Overall G0-2 PASS means comparison completion. Versions, dependencies and Proton launch path differ; causal attribution to staging patches alone is not established. |
| C-09 | Initial Proton exit codes 0/2/0 established the probe results | Unsupported; attempt retained INCONCLUSIVE | Console JSON was absent. A preregistered DOS-path repetition with unchanged probe/control bytes supplied the missing observations; it does not retroactively validate the first attempt. |

Original G0-3 remains BLOCKED, independently of the separately reported G0-3a/G0-3b mechanics PASS.
G0-1, G0-4 and G0-5 remain BLOCKED. Application compatibility and production readiness remain
unmeasured. Resource accounting includes capture-method debugging and privacy handling separately
from deterministic execution time.

## A0-7ZIP baseline preparation — 2026-09-07

The owner accepted G0-2 only for its controlled HRESULT-comparison scope at
`3f947888067243ba3cedcc3f77916344d9d10a25`. The separately authorised
[desktop application baseline](../experiments/EXP-009-APP-BASELINE-REPORT.md) is BLOCKED on
Hyper-V permissions. This changes no architecture decision or other Gate 0 status.

| ID | Claim | Assessment | Evidence and limit |
|---|---|---|---|
| C-10 | The prepared ZIP verifier distinguishes the defined correct output from seeded content corruption | VERIFIED_EXECUTION, host helper only | Fifteen helper tests and a labelled post-commit host control run; valid-CRC corrupted content is rejected. No guest or application result follows. |
| C-11 | Windows x64 7-Zip 26.03 completes two GUI workflows across a guest restart under vanilla Wine 11.17 | NOT_RUN; application baseline BLOCKED | Installer downloaded/hashed but never executed; no VM, desktop session, application ZIP or guest restart exists for this task. |
| C-12 | This baseline shows HELM improves compatibility or reduces maintenance cost | Unsupported | No HELM comparison or application workflow ran; even a completed existing-component baseline alone would not establish added value. |
