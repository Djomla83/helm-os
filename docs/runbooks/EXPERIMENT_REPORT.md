# Kako zabeležiti rezultat eksperimenta

Napravite poseban `EXP-XXX-REPORT.md`, ne prepisujte plan tako da prikrije prvobitno
očekivanje. Sačuvajte identitet plana i promene koje su nastale tokom izvršavanja.

## Obavezna polja

ID i datum; izvršilac i reviewer; odobren obim; app izdanje/hash; host i reference
OS; runtime build; GPU/drajver i uređaji; ulazi/test nalog bez tajni; koraci; očekivanje;
stvarni rezultat; broj ponavljanja; svi neuspehi; izuzeti/blokirani koraci i razlozi.

Priložite komande i redigovane artefakte ili njihove proverljive reference. Napišite
šta rezultat podržava i šta ne podržava. Pozitivan test jednog toka ne dokazuje
sve funkcije aplikacije niti celu klasu softvera.

## Zaključak

Koristite `supported`, `not_supported` ili `inconclusive` kao zaključak o hipotezi,
uz objašnjenje. To nije automatski aplikacioni status `verified`.
Navedite preporuku: nastaviti, izmeniti pristup, ponoviti bolji test ili obustaviti
usku putanju. Za promenu arhitekture otvorite ADR review.

Pre javne objave redigujte privatne poruke, putanje, tokene i identifikatore naloga.
Ne objavljujte komercijalne instalere ili Windows slike zajedno sa izveštajem.
