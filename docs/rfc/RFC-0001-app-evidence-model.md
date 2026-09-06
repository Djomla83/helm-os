# RFC-0001: Aplikacioni identitet, profil i testni dokaz

**Status:** Draft. Nije stabilan aplikacioni ABI niti paketni standard.

## Problem

Moramo sprečiti da sintetički manifest, app ime ili jedan launch test postane tvrdnja
da aplikacija pouzdano radi. Identitet artefakta, status profila i rezultat testa su
različiti objekti. Ovaj početni RFC obrađuje samo evidenciju testiranja.

## Predlog

Testni zapis vezuje tačnu aplikaciju i verziju za hardver, runtime i plan funkcija.
Ima status `not_tested`, `blocked`, `failed`, `partial`, `verified` ili `stale`.
Dokaz je lista reviewable artifact referenci i datum izvršavanja. `verified`
ne sme biti dozvoljen bez proverenog identiteta i dokaza.

Nacrt [JSON šeme](../../schemas/app-test-record.schema.json) i
[sintetičkog primera](../../examples/app-test-record.example.json) postoje radi
razgovora. Validator proverava neke konzervativne invarijante. Sintaktički validan
zapis ne dokazuje da je njegov sadržaj istinit; reviewer proverava rezultate.

## Minimalne invarijante

`not_tested` nema datum izvršavanja niti tvrdi da ima merenja. Pozitivan status ne
sme referencirati nepoznatu aplikacionu verziju, runtime ili hardver. VM rezultat
ne postaje binarna Windows kompatibilnost. Promena ključnog identiteta invalidira
stari dokaz i traži obnovu testa.

## Otvorena pitanja

Kako potpisati rezultat; koji autoritativni identitet ima hardware profile; kako
vezati redigovani dokaz za privatni sirovi log; kako izračunati kohortu kada više
hardverskih profila važi za istu aplikaciju; koji period čini rezultat zastarelim.

## Plan validacije

HELM-008 definiše obavezna polja i testira primer, nedostajući dokaz, pogrešan hash,
VM/native razdvajanje i `stale` tranziciju. Posle prvog laboratorijskog iskustva
revidirati šemu. Ne graditi veliki servis pre razumevanja toka podataka.
