# Stvarno objavljivanje repozitorijuma

**Predložena destinacija:** `Djomla83/helm-os-design`. Naziv je predlog, ne potvrda
slobodnog imena niti dokaz da udaljeni repo postoji. U ovoj sesiji novi GitHub
repo nije kreiran. Postojeći repozitorijumi nisu menjani.

## Preuslovi

Potvrditi destinaciju i javnu vidljivost, završiti [licencnu odluku](../../LICENSE-DECISION.md),
dodati izabrane licencne tekstove i pregledati javni sadržaj. Potrebni su Git,
Python 3.10+ i instaliran/autorizovan GitHub CLI sa pravom kreiranja repozitorijuma.
Ne deliti token u chatu, issue-u ili fajlu repozitorijuma.

## Put A — raspakovan izvorni ZIP, bez Git istorije

Uđite u direktorijum `helm-os-design`. Prvo završite licencnu odluku, zatim:

```bash
python3 tools/validate_docs.py
python3 -m unittest discover -s tools/tests -v
git init -b main
git add .
git commit -m "docs: initial design and research baseline"
```

Git može tražiti da na svom računaru podesite ime i email autora. Ne izmišljajte
identitet drugog saradnika. Pre objavljivanja pregledajte `git diff --cached` pre
commita ili `git show --stat` posle commita.

Ako CLI nije autorizovan, koristite njegov standardni interaktivni tok:

```bash
gh auth login
```

**Sledeća komanda stvarno kreira JAVNI udaljeni repo i šalje trenutni commit.**
Ne izvršavati je dok sadržaj i licenca nisu pregledani.

```bash
gh repo create Djomla83/helm-os-design --public --source=. --remote=origin --push
```

Sintaksa je proverena prema [zvaničnom CLI uputstvu](https://cli.github.com/manual/gh_repo_create).
Ako repo već postoji, ne pokušavajte da ga pregazite niti koristite force push.
Prvo proverite njegov sadržaj i odlučite o bezbednoj integraciji kroz granu/PR.

## Put B — Git bundle sa pripremljenom lokalnom istorijom

```bash
git clone HELM_OS_DESIGN.bundle helm-os-design
cd helm-os-design
git remote remove origin
```

Bundle ima stvarni dokumentacioni commit sa jasno označenim bootstrap autorom.
Pregledajte ga, završite licencnu odluku i napravite svoj novi commit. Potom
primenite istu `gh repo create` komandu. Bundle nije GitHub objava.

## Provera posle objavljivanja

```bash
gh repo view Djomla83/helm-os-design --json nameWithOwner,url,visibility
git remote -v
git rev-parse HEAD
```

Otvorite README, proverite linkove i vidljivost. U `docs/PROJECT_STATE.md` zabeležite
stvarni URL, SHA i datum objave kroz novi pregledan commit. Tek tada delite URL
kao aktivan repozitorijum.

Podesite pravila grane, obavezni review i odgovorne održavaoce na GitHub-u. Ne
računajte da Markdown pravila ili šablon PR-a sami tehnički štite `main`.

Pripremljena tela issue-a su u `planning/issues/`. Otvarajte ih po zavisnostima,
umesto da svih osamnaest istovremeno dodelite agentima. Nijedan od njih nije već
otvoren na serveru samo zato što postoji `.md` fajl.
