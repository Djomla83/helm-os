# Bezbednost i prijava problema

Trenutno nema produkcionog OS-a niti podržanih runtime izdanja. Repozitorijum sadrži
nacrte i pomoćni validator dokumentacije. **Ne postoji potvrđen security audit niti
ugovoreni rok bezbednosnog odgovora.**

Pre objavljivanja runtime-a vlasnik mora da uspostavi stvaran privatni kanal za
prijave i proces odgovora. Do tada ne objavljujte u javnom issue-u tajne, privatne
crash dump-ove ili detalje propusta koji bi direktno ugrozili aktivne korisnike.
Koristite postojeći privatni kanal sa održavaocem ako je dogovoren.

Za dokumentacione greške bez osetljivih podataka može se otvoriti običan issue.
Za upstream propust sledite pravila odgovarajućeg projekta i koordinirajte prijavu.

Plan bezbednosti je u [master planu, poglavlje 17](HELM_MASTER_PLAN.md#s17),
[ažuriranju i oporavku](HELM_MASTER_PLAN.md#s18) i
[threat-model RFC-u](docs/rfc/RFC-0002-threat-model.md).

Zabranjeno je tretirati Wine prefix kao dokaz sandbox-a, davati agentima potpisne
ključeve ili pokretati nepoznate aplikacije sa host privilegijama radi testiranja.
