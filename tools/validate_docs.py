#!/usr/bin/env python3
"""Offline provera HELM dokumentacije; nije provera aplikacija ili bezbednosti.

Podržava Markdown oblike koje ovaj repo koristi. Nije pun CommonMark parser
niti puna JSON Schema implementacija. Ne pristupa mreži i ne izvršava aplikacije.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime
from pathlib import Path
from urllib.parse import unquote, urlsplit

EXCLUDED = {".git", "__pycache__", ".venv", "venv", "build", "dist", "target", "private", "secrets"}
STATUSES = {"not_tested", "blocked", "failed", "partial", "verified", "stale"}
ROUTES = {"undecided", "native", "wine", "steam-proton", "windows-vm"}
SHA256 = re.compile(r"^[0-9a-f]{64}$")


def prose_only(text: str) -> tuple[str, list[str]]:
    """Uklanja fenced code blocks; prijavljuje nezatvoren blok."""
    output: list[str] = []
    fence_char, fence_len, start = "", 0, 0
    for number, line in enumerate(text.splitlines(), 1):
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if not fence_char and marker:
            fence_char, fence_len, start = marker[1][0], len(marker[1]), number
            output.append("")
        elif fence_char:
            if (marker and marker[1][0] == fence_char
                    and len(marker[1]) >= fence_len and not marker[2].strip()):
                fence_char, fence_len = "", 0
            output.append("")
        else:
            output.append(line)
    errors = [f"Nezatvoren code fence od reda {start}."] if fence_char else []
    return "\n".join(output), errors


def markdown_anchors(text: str) -> set[str]:
    """Eksplicitni HTML ID i osnovni GitHub-stil ATX naslovni slugovi."""
    prose, _ = prose_only(text)
    anchors = set(re.findall(r'<[^>]+\bid=[\"\']([^\"\']+)[\"\']', prose))
    counts: dict[str, int] = {}
    for heading in re.findall(r"^ {0,3}#{1,6}\s+(.+?)\s*#*\s*$", prose, re.MULTILINE):
        heading = re.sub(r"<[^>]+>", "", heading).lower()
        slug = re.sub(r"[^\w\- ]", "", heading).replace(" ", "-")
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        anchors.add(slug if count == 0 else f"{slug}-{count}")
    return anchors


def check_destination(destination: str, source: Path, root: Path) -> list[str]:
    """Proverava lokalnu putanju i .md anchor; spoljne URL ne otvara."""
    parts = urlsplit(destination.strip("<>"))
    if parts.scheme or parts.netloc:
        return []
    root = root.resolve()
    target = (source.parent / unquote(parts.path)).resolve() if parts.path else source.resolve()
    if not target.is_relative_to(root):
        return [f"Link izlazi iz repozitorijuma: {destination}"]
    if not target.exists():
        return [f"Nedostaje cilj linka: {destination}"]
    if parts.fragment and target.is_file() and target.suffix.lower() == ".md":
        fragment = unquote(parts.fragment)
        if fragment not in markdown_anchors(target.read_text(encoding="utf-8")):
            return [f"Nedostaje anchor: {destination}"]
    return []


def check_markdown(path: Path, root: Path) -> tuple[list[str], int]:
    text = path.read_text(encoding="utf-8")
    prose, errors = prose_only(text)
    # Jednolinijski inline kod ne sadrži stvarne Markdown linkove.
    prose = re.sub(r"(`+)(.*?)\1", "", prose)
    definitions = {
        key.casefold(): value
        for key, value in re.findall(r"^ {0,3}\[([^\]]+)\]:\s*(\S+)", prose, re.MULTILINE)
    }
    body = re.sub(r"^ {0,3}\[[^\]]+\]:.*$", "", prose, flags=re.MULTILINE)
    targets = list(definitions.values())
    targets += re.findall(r"!?\[[^\]\n]*\]\(\s*(<?[^\s)]+>?)\s*(?:\"[^\"]*\")?\)", body)
    for label, key in re.findall(r"\[([^\]\n]+)\]\[([^\]\n]*)\]", body):
        key = (key or label).casefold()
        if key not in definitions:
            errors.append(f"Nedostaje definicija reference: [{key}]")
    # Projekat koristi Sxx kratke oznake izvora i unutar proze.
    for key in re.findall(r"\[(S\d{2})\](?![(:])", body):
        if key.casefold() not in definitions:
            errors.append(f"Nedostaje izvor: [{key}]")
    for target in targets:
        errors.extend(check_destination(target, path, root))
    if not text.endswith("\n"):
        errors.append("Fajl nema završni novi red.")
    return errors, len(targets)


def valid_timestamp(value: object) -> bool:
    if not isinstance(value, str):
        return False
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
        return "T" in value and parsed.tzinfo is not None
    except ValueError:
        return False


def string_list(value: object, *, nonempty: bool = False, unique: bool = False) -> bool:
    return (isinstance(value, list)
            and (not nonempty or len(value) > 0)
            and all(isinstance(item, str) and item.strip() for item in value)
            and (not unique or len(set(value)) == len(value)))


def check_app_record(record: object) -> list[str]:
    """Ključne strukturne invarijante; sadržaj dokaza se ne verifikuje."""
    if not isinstance(record, dict):
        return ["App zapis mora biti objekat."]
    expected = {"schema_version", "record_id", "synthetic", "application", "execution_route",
                "runtime_id", "hardware_profile_id", "status", "tested_at",
                "mandatory_workflows", "evidence", "limitations"}
    errors: list[str] = []
    if set(record) != expected:
        errors.append("App zapis ima nedostajuća ili nepoznata polja.")
    if record.get("schema_version") != "0.1-draft":
        errors.append("Nepoznata revizija app šeme.")
    if not isinstance(record.get("record_id"), str) or not record["record_id"].strip():
        errors.append("record_id mora biti neprazan tekst.")
    if not isinstance(record.get("synthetic"), bool):
        errors.append("synthetic mora biti boolean.")
    status, route = record.get("status"), record.get("execution_route")
    if not isinstance(status, str) or status not in STATUSES:
        errors.append("Nepoznat app status.")
    if not isinstance(route, str) or route not in ROUTES:
        errors.append("Nepoznata execution_route putanja.")
    for key in ("runtime_id", "hardware_profile_id"):
        value = record.get(key)
        if value is not None and (not isinstance(value, str) or not value.strip()):
            errors.append(f"{key} mora biti null ili neprazan tekst.")
    app = record.get("application")
    if not isinstance(app, dict) or set(app) != {"id", "version", "source_sha256"}:
        errors.append("application nema očekivani oblik.")
        app = {}
    if not isinstance(app.get("id"), str) or not app["id"].strip():
        errors.append("application.id mora biti neprazan tekst.")
    version, sha = app.get("version"), app.get("source_sha256")
    if version is not None and (not isinstance(version, str) or not version.strip()):
        errors.append("application.version mora biti null ili neprazan tekst.")
    if sha is not None and (not isinstance(sha, str) or not SHA256.fullmatch(sha)):
        errors.append("Neispravan source_sha256.")
    for key in ("mandatory_workflows", "evidence", "limitations"):
        if not string_list(record.get(key), nonempty=(key == "mandatory_workflows"),
                           unique=(key != "limitations")):
            errors.append(f"Neispravna lista {key}.")
    tested = record.get("tested_at")
    if tested is not None and not valid_timestamp(tested):
        errors.append("tested_at mora imati ISO datum, vreme i vremensku zonu.")
    if status == "not_tested" and (tested is not None or record.get("evidence") != []):
        errors.append("not_tested ne sme imati tested_at ili evidence.")
    if status == "verified":
        if record.get("synthetic") is not False:
            errors.append("verified ne sme biti sintetički app rezultat.")
        if not valid_timestamp(tested):
            errors.append("verified zahteva datum stvarnog testa.")
        if route == "undecided":
            errors.append("verified zahteva konkretnu execution_route putanju.")
        for key in ("runtime_id", "hardware_profile_id"):
            if not isinstance(record.get(key), str) or not record[key].strip():
                errors.append(f"verified zahteva {key}.")
        if not isinstance(version, str) or not version.strip():
            errors.append("verified zahteva application.version.")
        if not isinstance(sha, str) or not SHA256.fullmatch(sha):
            errors.append("verified zahteva source_sha256.")
        if not string_list(record.get("evidence"), nonempty=True, unique=True):
            errors.append("verified zahteva evidence reference.")
    return errors


def check_cohort(cohort: object) -> list[str]:
    if not isinstance(cohort, dict):
        return ["Kohorta mora biti objekat."]
    errors: list[str] = []
    if cohort.get("status") == "draft-not-locked" and cohort.get("compatibility_rate") is not None:
        errors.append("Nacrt kohorte ne sme prikazivati izmeren compatibility_rate.")
    applications = cohort.get("applications")
    if not isinstance(applications, list) or not applications:
        return errors + ["Kohorta mora imati listu kandidata."]
    seen: set[str] = set()
    for app in applications:
        if not isinstance(app, dict):
            errors.append("Kandidat mora biti objekat.")
            continue
        app_id = app.get("app_id")
        if not isinstance(app_id, str) or not app_id or app_id in seen:
            errors.append("Kandidat ima neispravan ili dupliran app_id.")
        else:
            seen.add(app_id)
        status = app.get("status")
        route = app.get("execution_route")
        if not isinstance(status, str) or status not in STATUSES:
            errors.append(f"Nepoznat status kandidata {app_id}.")
        if not isinstance(route, str) or route not in ROUTES:
            errors.append(f"Nepoznata putanja kandidata {app_id}.")
        if not string_list(app.get("mandatory_workflows"), nonempty=True, unique=True):
            errors.append(f"Kandidat {app_id} nema ispravne obavezne tokove.")
    return errors


def validate_repository(root: Path) -> tuple[list[str], dict[str, int]]:
    root = root.resolve()
    if not root.is_dir():
        return [f"Ne postoji direktorijum: {root}"], {}
    errors: list[str] = []
    stats = {"markdown_files": 0, "json_files": 0, "link_targets": 0}
    for path in sorted(root.rglob("*")):
        if not path.is_file() or any(part in EXCLUDED for part in path.relative_to(root).parts):
            continue
        local_errors: list[str] = []
        try:
            if path.suffix.lower() == ".md":
                stats["markdown_files"] += 1
                local_errors, links = check_markdown(path, root)
                stats["link_targets"] += links
            elif path.suffix.lower() == ".json":
                stats["json_files"] += 1
                data = json.loads(path.read_text(encoding="utf-8"))
                if path.name.endswith(".example.json"):
                    local_errors = check_app_record(data)
                elif path.name == "app-cohort.json":
                    local_errors = check_cohort(data)
        except (OSError, UnicodeError, ValueError) as exc:
            local_errors.append(f"Neuspešno čitanje/provera: {exc}")
        errors.extend(f"{path.relative_to(root)}: {error}" for error in local_errors)
    return errors, stats


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args(argv)
    errors, stats = validate_repository(args.root)
    print(json.dumps(stats, ensure_ascii=False, sort_keys=True))
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("PASS: lokalna struktura dokumentacije i osnovne JSON invarijante.")
    print("Nisu provereni web URL dostupnost, istinitost dokaza, OS, aplikacije ili sandbox.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
