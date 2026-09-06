# RFC-0002: Početni threat model i bezbednosne granice

**Status:** Draft. Nije bezbednosni audit.

## Vrednosti koje štitimo

Korisnički dokumenti i nalozi, tajne, integritet sistema i update kanala,
verodostojnost kataloga profila, build infrastruktura i nezavisnost od modelskih grešaka.

## Nepouzdani ulazi

Installer i njegov child proces; plugin; aplikacioni sadržaj; profil iz zajednice;
preuzeti paket; AI predlog; web/issue tekst; crash log; javni PR kod.

## Granice

Aplikacija–host, aplikacija–aplikacija, AI–policy servis, CI–potpisivanje,
lab–korisnički podaci i javni log–privatni trag. Sam Wine prefix nije definisana
sigurnosna granica ovog projekta.

## Scenariji za proveru

Kontrolisani program pokušava da čita canary fajl iz zabranjenog direktorijuma.
Installer pokušava da doda host autostart van odobrenog plana. Profil traži neočekivano
širenje prava. Testna arhiva sadrži putanju van staging područja. Log sadrži instrukciju
agentu da objavi tajnu. Kandidat update-a nema važeći potpis ili je povučen.

Očekivanje je kontrolisano odbijanje, jasna evidencija i nepromenjeno korisničko stanje.
Testovi se izvode sa bezopasnim sintetičkim resursima u odvojenoj laboratoriji.

## Kompatibilnosne tenzije

Clipboard, drag-and-drop, globalne prečice, pristupačnost, screen sharing i inter-process
komunikacija mogu zahtevati dodatni pristup. Svaka dozvola mora imati korisničko
opravdanje i test granice. Ne proglašavati prolaz posle tihog uklanjanja izolacije.

## Otvorene odluke

Sandbox backend; policy protokol; granice privilegovanog servisa; format potpisivanja;
kanal za privatne prijave; zadržavanje logova; procedure revokacije.

Rezultati EXP-004 i pregled HELM-006 treba da prethode konačnom prihvatanju.
