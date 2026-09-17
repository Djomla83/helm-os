//! P1 boundary inspection: lint-policy drift, source confinement and manifest
//! scope. Every inspected file is embedded at compile time with `include_str!`,
//! so these tests read nothing at run time and run identically on every
//! platform. They execute nothing.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CRATE_MANIFEST: &str = include_str!("../Cargo.toml");
const LOCKFILE: &str = include_str!("../../../Cargo.lock");

const SOURCES: [(&str, &str); 7] = [
    ("src/lib.rs", include_str!("../src/lib.rs")),
    ("src/plan.rs", include_str!("../src/plan.rs")),
    ("src/model.rs", include_str!("../src/model.rs")),
    ("src/receipt.rs", include_str!("../src/receipt.rs")),
    ("src/error.rs", include_str!("../src/error.rs")),
    ("src/layout.rs", include_str!("../src/layout.rs")),
    ("src/lifecycle.rs", include_str!("../src/lifecycle.rs")),
];

const TESTS: [(&str, &str); 2] = [
    ("tests/plan_contract.rs", include_str!("plan_contract.rs")),
    ("tests/p1_boundary.rs", include_str!("p1_boundary.rs")),
];

/// Other workspace members, which must keep inheriting the workspace lints.
const OTHER_MEMBERS: [(&str, &str); 4] = [
    (
        "crates/helm-evidence",
        include_str!("../../helm-evidence/Cargo.toml"),
    ),
    (
        "crates/helm-app-spec",
        include_str!("../../helm-app-spec/Cargo.toml"),
    ),
    (
        "crates/helm-observe",
        include_str!("../../helm-observe/Cargo.toml"),
    ),
    (
        "crates/helm-bind",
        include_str!("../../helm-bind/Cargo.toml"),
    ),
];

// ------------------------------------------------------------- manifest parsing

/// Minimal TOML reader for `[table]` headers and `key = "string"` lines. Enough
/// for lint tables and member lists; anything it cannot read is ignored, and the
/// assertions below fail closed on missing entries.
fn tables(manifest: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut current = String::new();
    for raw in manifest.lines() {
        let line = raw.split('#').next().unwrap().trim();
        if line.starts_with('[') && line.ends_with(']') {
            current = line.trim_matches(['[', ']']).trim().to_owned();
            out.entry(current.clone()).or_default();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            out.entry(current.clone()).or_default().insert(
                key.trim().to_owned(),
                value.trim().trim_matches('"').to_owned(),
            );
        }
    }
    out
}

fn strength(level: &str) -> u8 {
    match level {
        "allow" => 0,
        "warn" => 1,
        "deny" => 2,
        "forbid" => 3,
        other => panic!("unknown lint level {other:?}"),
    }
}

fn workspace_members() -> BTreeSet<String> {
    let start = WORKSPACE_MANIFEST.find("members = [").unwrap();
    let rest = &WORKSPACE_MANIFEST[start..];
    let end = rest.find(']').unwrap();
    rest[..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

// ------------------------------------------------------------ source lexing

/// Rust code with comments blanked and the contents of string, raw-string and
/// character literals blanked. Structure (`fn`, `->`, `{`) is kept, so
/// documentation that says what the crate does *not* do cannot trip a scan.
fn strip(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    let at = |k: usize| chars.get(k).copied();
    let ident_before = |out: &String| {
        out.chars()
            .last()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    };
    while let Some(c) = at(i) {
        if c == '/' && at(i + 1) == Some('/') {
            while at(i).is_some_and(|c| c != '\n') {
                i += 1;
            }
            out.push(' ');
            continue;
        }
        if c == '/' && at(i + 1) == Some('*') {
            let mut depth = 0;
            loop {
                match (at(i), at(i + 1)) {
                    (Some('/'), Some('*')) => {
                        depth += 1;
                        i += 2;
                    }
                    (Some('*'), Some('/')) => {
                        depth -= 1;
                        i += 2;
                        if depth == 0 {
                            break;
                        }
                    }
                    (Some(_), _) => i += 1,
                    (None, _) => break,
                }
            }
            out.push(' ');
            continue;
        }
        // Raw strings: r"..", r#".."#, br#".."#, only at a token start.
        let b_prefix = c == 'b' && at(i + 1) == Some('r');
        if (c == 'r' || b_prefix) && !ident_before(&out) {
            let mut j = if b_prefix { i + 2 } else { i + 1 };
            let mut hashes = 0;
            while at(j) == Some('#') {
                hashes += 1;
                j += 1;
            }
            if at(j) == Some('"') {
                j += 1;
                while at(j).is_some() {
                    if at(j) == Some('"') && (1..=hashes).all(|h| at(j + h) == Some('#')) {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                out.push_str("\"\"");
                i = j;
                continue;
            }
        }
        if c == '"' {
            i += 1;
            while let Some(s) = at(i) {
                i += 1;
                if s == '\\' {
                    i += 1;
                } else if s == '"' {
                    break;
                }
            }
            out.push_str("\"\"");
            continue;
        }
        if c == '\'' {
            if at(i + 1) == Some('\\') {
                i += 2;
                while at(i).is_some_and(|c| c != '\'') {
                    i += 1;
                }
                i += 1;
                out.push_str("' '");
                continue;
            }
            if at(i + 2) == Some('\'') {
                i += 3;
                out.push_str("' '");
                continue;
            }
            // A lifetime or label: keep it as code.
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Identifier tokens of [`strip`]ped code.
fn code_tokens(source: &str) -> Vec<String> {
    raw_words(&strip(source))
        .filter(|t| !t.starts_with(|c: char| c.is_ascii_digit()))
        .map(str::to_owned)
        .collect()
}

/// Every identifier-shaped word of the raw text, comments and strings included.
fn raw_words(source: &str) -> impl Iterator<Item = &str> {
    source
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
}

#[test]
fn the_lexer_drops_comments_and_literals_but_keeps_code() {
    let sample = "fn keep() { let a = \"hidden_in_string\"; let q = '\"'; // hidden_in_comment\n\
                  let r = r#\"hidden \" raw\"#; /* hidden /* nested */ block */ let t: &'static str = x; }";
    let tokens = code_tokens(sample);
    for want in [
        "fn", "keep", "let", "a", "q", "r", "t", "static", "str", "x",
    ] {
        assert!(tokens.iter().any(|t| t == want), "lost {want}: {tokens:?}");
    }
    for hidden in [
        "hidden_in_string",
        "hidden_in_comment",
        "hidden",
        "raw",
        "nested",
        "block",
    ] {
        assert!(
            !tokens.iter().any(|t| t == hidden),
            "kept {hidden}: {tokens:?}"
        );
    }
    // The real sources lex to their real entry points.
    let lib = code_tokens(SOURCES[0].1);
    assert!(lib.iter().any(|t| t == "parse_launch_plan"));
    let plan = code_tokens(SOURCES[1].1);
    assert!(plan.iter().any(|t| t == "strict_tree"));
}

// -------------------------------------------------------------------- lints

#[test]
fn lint_policy_restates_every_workspace_lint_without_weakening() {
    let workspace = tables(WORKSPACE_MANIFEST);
    let krate = tables(CRATE_MANIFEST);

    assert!(
        !krate.contains_key("lints"),
        "helm-launch must not inherit the workspace lint table"
    );
    assert!(
        krate.values().all(|t| !t.contains_key("workspace")),
        "no manifest table may set workspace inheritance"
    );

    let mut workspace_lints = 0;
    for group in ["rust", "clippy"] {
        let ws = &workspace[&format!("workspace.lints.{group}")];
        let local = krate
            .get(&format!("lints.{group}"))
            .unwrap_or_else(|| panic!("[lints.{group}] missing"));
        for (lint, level) in ws {
            workspace_lints += 1;
            let mine = local
                .get(lint)
                .unwrap_or_else(|| panic!("workspace lint {group}::{lint} not restated"));
            if group == "rust" && lint == "unsafe_code" {
                // The one accepted exception (plan 7.4): `deny` in the manifest,
                // restated as `deny` at the crate root (tested below).
                assert_eq!(level, "forbid");
                assert_eq!(mine, "deny");
            } else {
                assert!(
                    strength(mine) >= strength(level),
                    "{group}::{lint} weakened from {level} to {mine}"
                );
            }
        }
    }
    assert!(
        workspace_lints >= 4,
        "workspace lint table unexpectedly small"
    );

    // The accepted table of plan section 7.4, exactly.
    let expected: [(&str, &str); 7] = [
        ("lints.rust", "unsafe_code"),
        ("lints.rust", "unsafe_op_in_unsafe_fn"),
        ("lints.clippy", "unwrap_used"),
        ("lints.clippy", "expect_used"),
        ("lints.clippy", "panic"),
        ("lints.clippy", "undocumented_unsafe_blocks"),
        ("lints.clippy", "multiple_unsafe_ops_per_block"),
    ];
    let mut found = 0;
    for (table, entries) in &krate {
        if table.starts_with("lints") {
            for (lint, level) in entries {
                assert!(
                    expected.contains(&(table.as_str(), lint.as_str())),
                    "unexpected {table}::{lint}"
                );
                assert_eq!(level, "deny", "{table}::{lint}");
                found += 1;
            }
        }
    }
    assert_eq!(found, expected.len());
}

#[test]
fn the_crate_root_denies_rather_than_forbids_the_unsafe_lints() {
    // Plan 7.4 restates the policy in source as `deny`, not `forbid`: only
    // `deny` leaves room for a scoped `allow` in a later, separately authorised
    // slice. P1 authorises none, so no `allow` may exist, and with no `unsafe`
    // token in `src/` (`the_source_contains_no_unsafe_token_anywhere`) P1 has
    // zero such code.
    let root: String = strip(SOURCES[0].1).split_whitespace().collect();
    assert!(
        root.contains("#![deny(unsafe_code,unsafe_op_in_unsafe_fn)]"),
        "the crate root must deny both lints"
    );
    assert!(
        !code_tokens(SOURCES[0].1).iter().any(|t| t == "forbid"),
        "the crate root must deny, not forbid"
    );
    // Each lint is named as code exactly once in all of `src/`: in that root
    // `deny`. No `forbid`, `allow`, `expect` or `warn` of either exists, even
    // through `cfg_attr`.
    for lint in ["unsafe_code", "unsafe_op_in_unsafe_fn"] {
        let named: Vec<&str> = SOURCES
            .iter()
            .flat_map(|(name, source)| {
                code_tokens(source)
                    .into_iter()
                    .filter(|t| t == lint)
                    .map(|_| *name)
            })
            .collect();
        assert_eq!(named, ["src/lib.rs"], "{lint} named as code: {named:?}");
    }
}

#[test]
fn every_other_member_still_inherits_the_workspace_lints() {
    let members = workspace_members();
    let mut expected: BTreeSet<String> =
        OTHER_MEMBERS.iter().map(|(n, _)| (*n).to_owned()).collect();
    expected.insert("crates/helm-launch".to_owned());
    assert_eq!(
        members, expected,
        "a new member must be added to this drift test"
    );
    for (name, manifest) in OTHER_MEMBERS {
        let t = tables(manifest);
        assert_eq!(
            t.get("lints")
                .and_then(|l| l.get("workspace"))
                .map(String::as_str),
            Some("true"),
            "{name} stopped inheriting the workspace lints"
        );
    }
}

// ------------------------------------------------------------- confinement

#[test]
fn the_source_contains_no_unsafe_token_anywhere() {
    let word = ["un", "safe"].concat();
    for (name, source) in SOURCES {
        assert!(
            !raw_words(source).any(|w| w == word),
            "{name} contains the {word} token (comments included)"
        );
    }
}

#[test]
fn the_crate_declares_exactly_the_p1_modules() {
    let tokens = code_tokens(SOURCES[0].1);
    let modules: BTreeSet<&str> = tokens
        .windows(2)
        .filter(|w| w[0] == "mod")
        .map(|w| w[1].as_str())
        .collect();
    let expected: BTreeSet<&str> =
        ["error", "layout", "lifecycle", "model", "plan", "receipt"].into();
    assert_eq!(modules, expected);
    for (name, source) in SOURCES {
        let tokens = code_tokens(source);
        let nested: Vec<&[String]> = tokens
            .windows(2)
            .filter(|w| w[0] == "mod" && w[1] != "tests")
            .collect();
        if name != "src/lib.rs" {
            assert!(nested.is_empty(), "{name} declares a module: {nested:?}");
        }
        let code: String = strip(source).split_whitespace().collect();
        for forbidden in [
            "#[path",
            "include!(",
            "include_bytes!(",
            "include_str!(",
            "cfg_if!",
        ] {
            assert!(!code.contains(forbidden), "{name} uses {forbidden}");
        }
    }
}

/// Operating-system, execution and later-slice vocabulary that must not appear
/// as code in P1. Comments and string contents are excluded.
const FORBIDDEN_CODE: &[&str] = &[
    // FFI and low-level code.
    "extern",
    "asm",
    "global_asm",
    "naked_asm",
    "link",
    "no_mangle",
    "transmute",
    "libc",
    "rustix",
    "nix",
    "windows_sys",
    "syscall",
    // Descriptors.
    "OwnedFd",
    "BorrowedFd",
    "RawFd",
    "AsRawFd",
    "AsFd",
    "FromRawFd",
    "IntoRawFd",
    "RawHandle",
    "OwnedHandle",
    // Processes, signals, waiting.
    "process",
    "Command",
    "Child",
    "Stdio",
    "fork",
    "vfork",
    "clone3",
    "execve",
    "execveat",
    "fexecve",
    "posix_spawn",
    "pidfd",
    "pidfd_open",
    "pidfd_send_signal",
    "waitid",
    "waitpid",
    "kill",
    "setpgid",
    "setsid",
    "sigaction",
    "sigprocmask",
    "rt_sigprocmask",
    "pthread_sigmask",
    "prctl",
    "close_range",
    "fchdir",
    "chdir",
    "fcntl",
    "dup",
    "dup2",
    "dup3",
    "pipe",
    "pipe2",
    "poll",
    // Host state, I/O, time, threads.
    "fs",
    "File",
    "OpenOptions",
    "io",
    "net",
    "TcpStream",
    "UdpSocket",
    "env",
    "thread",
    "Instant",
    "SystemTime",
    "target_os",
    "target_family",
    "target_arch",
    // P2+ surfaces that do not exist in P1.
    "ExecutableCapability",
    "WorkingDirectoryCapability",
    "AuthorizedLaunch",
    "LaunchOutcome",
    "AdmissionError",
    "AuthorizationRefusal",
    "LaunchError",
    "PreparedLaunch",
    "ChildPlan",
    "admit_executable",
    "admit_working_directory",
    "authorize",
    "launch",
    "launch_minimal",
    // Serde derivation of any model type.
    "Deserialize",
    "Serialize",
];

#[test]
fn no_os_execution_or_later_slice_vocabulary_appears_as_code() {
    for (name, source) in SOURCES.iter().chain(TESTS.iter()) {
        let tokens = code_tokens(source);
        for t in &tokens {
            assert!(
                !FORBIDDEN_CODE.contains(&t.as_str()),
                "{name} uses `{t}` as code"
            );
            assert!(!t.starts_with("admit_"), "{name} names `{t}`");
        }
    }
}

#[test]
fn no_function_offers_a_success_reading() {
    // Public functions allowed to return `bool`: three accessors, each reading
    // one accepted boolean receipt fact of the same name, and the membership
    // query on a list of plan findings. None is a verdict about an outcome.
    const BOOL_ALLOWED: [&str; 4] = [
        "run_deadline_expired",
        "sigterm_sent",
        "sigkill_sent",
        "contains",
    ];
    const VERDICT_NAMES: [&str; 9] = [
        "is_success",
        "success",
        "succeeded",
        "ok",
        "is_ok",
        "passed",
        "is_ready",
        "is_compatible",
        "is_verified",
    ];
    let mut public_bool = BTreeSet::new();
    for (name, source) in SOURCES {
        let code = strip(source);
        let tokens = code_tokens(source);
        for w in tokens.windows(2) {
            if w[0] == "fn" {
                assert!(
                    !VERDICT_NAMES.contains(&w[1].as_str()),
                    "{name}: fn {}",
                    w[1]
                );
            }
        }
        for (pos, _) in code.match_indices("fn ") {
            let before = code[..pos].trim_end();
            let public = before.ends_with("pub") || before.ends_with("pub const");
            let rest = &code[pos + 3..];
            let fname: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let signature = &rest[..rest.find(['{', ';']).unwrap_or(rest.len())];
            let compact: String = signature.split_whitespace().collect();
            if public && compact.ends_with("->bool") {
                assert!(
                    BOOL_ALLOWED.contains(&fname.as_str()),
                    "{name}: pub fn {fname} -> bool"
                );
                public_bool.insert(fname);
            }
        }
        // No trait of any kind is implemented for `bool`, so no crate type
        // converts into one.
        for w in tokens.windows(2) {
            assert!(
                !(w[0] == "for" && w[1] == "bool"),
                "{name}: impl .. for bool"
            );
        }
    }
    // The scan saw the accessors it allows, so it is actually inspecting methods.
    assert_eq!(
        public_bool,
        BOOL_ALLOWED.iter().map(|s| (*s).to_owned()).collect()
    );
}

// ------------------------------------------------------------ manifest scope

#[test]
fn dependencies_are_exactly_the_p1_set() {
    let krate = tables(CRATE_MANIFEST);
    let deps: BTreeMap<&str, &str> = krate["dependencies"]
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    assert_eq!(
        deps,
        BTreeMap::from([
            ("serde", "=1.0.228"),
            ("serde_json", "=1.0.149"),
            ("sha2", "=0.10.9")
        ])
    );
    for table in krate.keys() {
        assert!(
            !table.contains("dev-dependencies")
                && !table.contains("build-dependencies")
                && !table.starts_with("target")
                && !table.contains("features"),
            "unexpected manifest table [{table}]"
        );
    }
    let package = &krate["package"];
    assert_eq!(package["name"], "helm-launch");
    assert_eq!(package["version"], "0.1.0");
    assert_eq!(package["edition"], "2024");
    assert_eq!(package["rust-version"], "1.95");
    assert_eq!(package["publish"], "false");
    for word in ["libc", "rustix", "build.rs"] {
        assert!(!CRATE_MANIFEST.contains(word), "manifest mentions {word}");
    }
}

#[test]
fn the_lockfile_entry_has_exactly_the_p1_dependencies() {
    let start = LOCKFILE.find("name = \"helm-launch\"").unwrap();
    let entry = &LOCKFILE[start..];
    let entry = &entry[..entry.find("\n\n").unwrap_or(entry.len())];
    let deps: Vec<&str> = entry
        .lines()
        .filter(|l| l.starts_with(" \""))
        .map(|l| l.trim().trim_matches([',', '"']))
        .collect();
    assert_eq!(deps, ["serde", "serde_json", "sha2"]);
    assert!(entry.contains("version = \"0.1.0\""));
    assert!(
        !entry.contains("source ="),
        "helm-launch must be a local package"
    );
}
