# Primeri

JSON zapis je sintetički i netestiran. Prazne vrednosti su namerne: nema stvarne
aplikacione verzije, hash-a, runtime-a, hardvera ili testa.

Šema sprečava neke očigledne kontradikcije, ali ne može utvrditi da li su kasniji
log, potpis i rezultat istiniti. To zahteva stvarnu laboratoriju i review.

Lokalni validator proverava dokumente i ključne invarijante bez mreže.
Za punu JSON Schema proveru treba koristiti odgovarajući validator Draft 2020-12;
početna pomoćna skripta nije potpuna implementacija tog standarda.
