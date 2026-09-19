//! P1 and P2 boundary inspection: lint-policy drift, source confinement,
//! platform gating and manifest scope. Every inspected file is embedded at
//! compile time with `include_str!`, so these tests read nothing at run time and
//! run identically on every platform. They execute nothing.
//!
//! The scans distinguish **product code** from the `cfg(test)` region of the
//! same file. Product code may not name host state, a pathname API or any
//! process vocabulary; test code is the trusted caller and may build fixtures.
//!
//! # Scope after P3
//!
//! The inspected set below is the **portable and authority** source of P1 and
//! P2. It deliberately excludes `src/backend/`, which the owner authorised for
//! process creation and scoped `unsafe` in P3: the claims this file makes —
//! that the sources it scans contain no `unsafe` token at all and name no
//! process, foreign-interface or raw-descriptor vocabulary — stay exactly as
//! strong for them as they were.
//!
//! `tests/p3_boundary.rs` makes the crate-wide claims instead: that `unsafe`
//! appears **only** under `src/backend/`, that the one scoped
//! `#![allow(unsafe_code)]` is at that boundary and nowhere else, that the
//! child window's vocabulary is closed, and that no backend item is public.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};

const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CRATE_MANIFEST: &str = include_str!("../Cargo.toml");
const LOCKFILE: &str = include_str!("../../../Cargo.lock");

/// The exact cohort predicate every P2 surface must be gated by.
const COHORT_PREDICATE: &str = r#"all(target_os = "linux", target_arch = "x86_64")"#;

/// That predicate as a complete `cfg` attribute body.
const COHORT: &str = r#"cfg(all(target_os = "linux", target_arch = "x86_64"))"#;

const SOURCES: [(&str, &str); 8] = [
    ("src/lib.rs", include_str!("../src/lib.rs")),
    ("src/plan.rs", include_str!("../src/plan.rs")),
    ("src/model.rs", include_str!("../src/model.rs")),
    ("src/receipt.rs", include_str!("../src/receipt.rs")),
    ("src/error.rs", include_str!("../src/error.rs")),
    ("src/layout.rs", include_str!("../src/layout.rs")),
    ("src/lifecycle.rs", include_str!("../src/lifecycle.rs")),
    ("src/authority.rs", include_str!("../src/authority.rs")),
];

const TESTS: [(&str, &str); 3] = [
    ("tests/plan_contract.rs", include_str!("plan_contract.rs")),
    ("tests/p2_boundary.rs", include_str!("p2_boundary.rs")),
    (
        "tests/linux_admission.rs",
        include_str!("linux_admission.rs"),
    ),
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

fn source(name: &str) -> &'static str {
    SOURCES
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("no such source {name}"))
        .1
}

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

/// The marker that starts a file's test-only region.
const TEST_MARKER: &str = "#[cfg(test)]";

/// Everything before the first `#[cfg(test)]`: the code a release build
/// compiles. Every source file keeps its test-only items last, which
/// [`the_test_only_region_of_every_source_is_last`] verifies.
fn product_code(source: &str) -> &str {
    match source.find(TEST_MARKER) {
        Some(at) => &source[..at],
        None => source,
    }
}

/// Everything from the first `#[cfg(test)]` on.
fn test_code(source: &str) -> &str {
    match source.find(TEST_MARKER) {
        Some(at) => &source[at..],
        None => "",
    }
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
    assert!(
        code_tokens(source("src/lib.rs"))
            .iter()
            .any(|t| t == "parse_launch_plan")
    );
    assert!(
        code_tokens(source("src/plan.rs"))
            .iter()
            .any(|t| t == "strict_tree")
    );
    assert!(
        code_tokens(source("src/authority.rs"))
            .iter()
            .any(|t| t == "admit_executable")
    );
}

#[test]
fn the_test_only_region_of_every_source_is_last() {
    for (name, source) in SOURCES {
        let region = test_code(source);
        if region.is_empty() {
            continue;
        }
        // The split is only meaningful if the region really is the test region.
        assert!(
            region.contains("mod tests"),
            "{name}: the first {TEST_MARKER} does not start the test region"
        );
        // And no product item may follow it.
        let product = product_code(source);
        assert!(
            !product.contains("mod tests"),
            "{name}: a test module appears in the product region"
        );
    }
    // Every file that has unit tests is covered, and `authority.rs` is one of
    // them: its instability seam lives in that region by construction.
    let with_tests: Vec<&str> = SOURCES
        .iter()
        .filter(|(_, s)| !test_code(s).is_empty())
        .map(|(n, _)| *n)
        .collect();
    assert!(with_tests.contains(&"src/authority.rs"), "{with_tests:?}");
    assert!(
        test_code(source("src/authority.rs")).contains("INSTABILITY_SEAM"),
        "the seam must be test-only"
    );
    assert!(
        !product_code(source("src/authority.rs")).contains("INSTABILITY_SEAM"),
        "no release build may contain the seam"
    );
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
    // `deny` leaves room for the one scoped `allow` the owner authorised for
    // `src/backend/` in P3. In the portable and authority sources scanned here
    // no `allow` may exist, and the token is absent from them entirely
    // (`the_portable_and_authority_sources_contain_no_unsafe_token_anywhere`).
    let root: String = strip(source("src/lib.rs")).split_whitespace().collect();
    assert!(
        root.contains("#![deny(unsafe_code,unsafe_op_in_unsafe_fn)]"),
        "the crate root must deny both lints"
    );
    assert!(
        !code_tokens(source("src/lib.rs"))
            .iter()
            .any(|t| t == "forbid"),
        "the crate root must deny, not forbid"
    );
    // Across the portable and authority sources each lint is named as code
    // exactly once: in that root `deny`. No `forbid`, `allow`, `expect` or
    // `warn` of either exists in them, even through `cfg_attr`.
    // `tests/p3_boundary.rs` bounds where the backend may name them.
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
fn no_product_code_relaxes_the_panic_free_lints() {
    // Test modules may `allow` them, as the accepted plan's test strategy
    // expects; product code may not, on any platform.
    for (name, source) in SOURCES {
        let product: String = strip(product_code(source)).split_whitespace().collect();
        for lint in [
            "clippy::unwrap_used",
            "clippy::expect_used",
            "clippy::panic",
        ] {
            let relaxed = format!("allow({lint}");
            assert!(!product.contains(&relaxed), "{name} relaxes {lint}");
            assert!(
                !product.contains(&format!("allow(dead_code,{lint}")),
                "{name} relaxes {lint}"
            );
        }
    }
    // The scan is real: the crate does relax them inside test modules.
    let relaxing: Vec<&str> = SOURCES
        .iter()
        .filter(|(_, s)| {
            strip(test_code(s))
                .split_whitespace()
                .collect::<String>()
                .contains("clippy::unwrap_used")
        })
        .map(|(n, _)| *n)
        .collect();
    assert!(relaxing.contains(&"src/authority.rs"), "{relaxing:?}");
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
fn the_portable_and_authority_sources_contain_no_unsafe_token_anywhere() {
    // The strongest possible form for these files: not as code, not in a
    // string, not even in a comment.
    //
    // The crate root is the one exception, and only in prose: after P3 it has
    // to say where the scoped exception lives and that nothing public reaches
    // it. For the root the claim is therefore the code-level one, which is what
    // actually matters. `tests/p3_boundary.rs` proves the crate-wide form:
    // the token appears as code only under `src/backend/`.
    // Two files describe the rule in prose and are therefore held to the
    // code-level form of it instead: the crate root, which has to say where the
    // scoped exception lives and that nothing public reaches it, and this file,
    // which explains the rule it enforces.
    const DESCRIBES_THE_RULE: [&str; 2] = ["src/lib.rs", "tests/p2_boundary.rs"];
    let word = ["un", "safe"].concat();
    for (name, source) in SOURCES.iter().chain(TESTS.iter()) {
        if DESCRIBES_THE_RULE.contains(name) {
            assert!(
                !code_tokens(source).contains(&word),
                "{name} uses the {word} token as code"
            );
            continue;
        }
        assert!(
            !raw_words(source).any(|w| w == word),
            "{name} contains the {word} token (comments included)"
        );
    }
    // The scan is real: the crate root does describe the exception in prose.
    assert!(raw_words(source("src/lib.rs")).any(|w| w == word));
}

#[test]
fn the_crate_declares_exactly_the_p2_modules() {
    let lib = source("src/lib.rs");
    let tokens = code_tokens(lib);
    let modules: BTreeSet<&str> = tokens
        .windows(2)
        .filter(|w| w[0] == "mod")
        .map(|w| w[1].as_str())
        .collect();
    // `launch` is the P4 lifecycle module. Like `backend` it is declared
    // here and gated on the cohort; unlike `backend` it is re-exported,
    // because P4 is the slice that publishes `launch` and `LaunchOutcome`.
    let expected: BTreeSet<&str> = [
        "authority",
        "backend",
        "error",
        "launch",
        "layout",
        "lifecycle",
        "model",
        "plan",
        "receipt",
    ]
    .into();
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
        // Only the crate root may declare or reach the backend module. The
        // check is on the module, not on the word: `Backend` is also an
        // accepted P1 receipt fact, and `ReceiptRecord` carries it.
        if name != "src/lib.rs" {
            let code: String = strip(source).split_whitespace().collect();
            assert!(
                !code.contains("modbackend"),
                "{name} declares the backend module"
            );
            assert!(
                !code.contains("backend::"),
                "{name} reaches into the backend module"
            );
            assert!(
                !code.contains("crate::backend"),
                "{name} names the backend module"
            );
        }
    }
    // The crate root declares it exactly once, privately, and re-exports
    // nothing from it.
    let root: String = strip(lib).split_whitespace().collect();
    assert_eq!(root.matches("modbackend;").count(), 1);
    assert!(
        !root.contains("pubmodbackend"),
        "the backend module is public"
    );
    assert!(
        !root.contains("pubusebackend"),
        "a backend item is re-exported"
    );
    assert!(!root.contains("pub(crate)usebackend"));
}

// --------------------------------------------------------- platform gating

#[test]
fn every_p2_surface_is_gated_by_exactly_the_linux_x86_64_cohort() {
    let lib: String = source("src/lib.rs")
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join("\n");
    let gate = format!("#[{COHORT}]");
    // The module declaration and the re-export both carry the exact predicate.
    assert!(
        lib.contains(&format!("{gate}\nmod authority;")),
        "the authority module is not gated by {COHORT}"
    );
    assert!(
        lib.contains(&format!("{gate}\npub use authority::{{")),
        "the P2 re-export is not gated by {COHORT}"
    );
    // Nothing else in the crate is gated on a platform at all: the portable
    // model has no platform-specific behaviour to gate.
    for (name, source) in SOURCES {
        let occurrences = source.matches("target_os").count();
        let allowed = match name {
            // The crate root: the authority module gate, the backend module
            // gate, the P4 launch module gate, the authority re-export gate,
            // the P4 launch re-export gate and the two `cfg_attr`
            // documentation blocks that prove presence off and on the cohort.
            "src/lib.rs" => 7,
            // The module's own documentation states its gate in prose only.
            "src/authority.rs" => 1,
            // One `cfg_attr` each that silences dead code off the cohort, where
            // the P4 receipt producer is not compiled: `plan.rs` for the
            // accessor it reads, `receipt.rs` for the serializer it calls.
            "src/plan.rs" | "src/receipt.rs" => 1,
            // The three `LaunchError` constructors: the P4 producer that calls
            // them is not compiled off the cohort.
            "src/error.rs" => 3,
            _ => 0,
        };
        assert_eq!(
            occurrences, allowed,
            "{name} names target_os {occurrences}x"
        );
    }
    // `authority.rs` states its gate in a comment, never as code: the gate
    // lives in `lib.rs`, so the module cannot be compiled off the cohort by
    // some other route.
    assert!(
        !code_tokens(source("src/authority.rs"))
            .iter()
            .any(|t| t == "target_os" || t == "target_arch"),
        "the authority module must not carry its own platform cfg"
    );
}

#[test]
fn the_crate_root_proves_the_off_cohort_absence_of_every_p2_name() {
    // Off the cohort these `compile_fail` doctests are collected and run, so
    // Windows and macOS CI proves the names do not exist there. On the cohort
    // the sibling block proves they do.
    let lib = source("src/lib.rs");
    for name in [
        "admit_executable",
        "admit_working_directory",
        "authorize",
        "ExecutableCapability",
        "WorkingDirectoryCapability",
        "AuthorizedLaunch",
    ] {
        let absence = format!(r#"doc = "use helm_launch::{name};""#);
        assert!(lib.contains(&absence), "no off-cohort proof for {name}");
    }
    assert!(lib.contains(r#"doc = "```compile_fail","#));
    assert!(
        lib.contains(&format!("not({COHORT_PREDICATE}),")),
        "the off-cohort documentation block is missing"
    );
    assert!(
        lib.contains(&format!("{COHORT_PREDICATE},")),
        "the on-cohort documentation block is missing"
    );
    assert_eq!(COHORT, format!("cfg({COHORT_PREDICATE})"));
}

// ------------------------------------------------------------- vocabulary

/// Vocabulary that must not appear as code in the **portable and authority**
/// sources or their tests, product code and test code alike: foreign
/// interfaces, raw descriptor numbers, process creation, signals, waiting, and
/// the surfaces of later, unauthorised slices.
///
/// `src/backend/` is authorised for exactly this vocabulary and is outside the
/// scanned set; `tests/p3_boundary.rs` bounds it there instead.
const FORBIDDEN_EVERYWHERE: &[&str] = &[
    // Foreign interfaces and low-level code.
    "extern",
    "asm",
    "global_asm",
    "naked_asm",
    "no_mangle",
    "transmute",
    "libc",
    "nix",
    "windows_sys",
    "syscall",
    // Raw descriptor numbers and hand-built ownership.
    "RawFd",
    "AsRawFd",
    "FromRawFd",
    "IntoRawFd",
    "borrow_raw",
    "RawHandle",
    "OwnedHandle",
    "forget",
    "ManuallyDrop",
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
    "dup2",
    "dup3",
    "pipe",
    "pipe2",
    "poll",
    "launcher_spike",
    // Surfaces of later slices that are not authorised, and of the one that
    // is authorised but must stay inside `src/backend/`.
    "launch",
    "launch_minimal",
    "LaunchOutcome",
    "PreparedLaunch",
    "ChildPlan",
    // Serde derivation of any model type.
    "Deserialize",
    "Serialize",
];

/// Host state and pathname vocabulary. Product code never names it: the
/// product API receives authority only as an owned descriptor and resolves no
/// name. Test code is the trusted caller and may build fixtures by path.
const FORBIDDEN_IN_PRODUCT_CODE: &[&str] = &[
    "File",
    "OpenOptions",
    "canonicalize",
    "read_link",
    "current_dir",
    "read_dir",
    "metadata",
    "remove_file",
    "create_dir",
    "open",
    "openat",
    "openat2",
    "statx",
    "memfd_create",
    "mkdir",
    "unlink",
    "rmdir",
    "chmod",
    "fchmod",
    "ftruncate",
    "rename",
    "seek",
    "dup",
    "pwrite",
    "Path",
    "PathBuf",
    "OsStr",
    "OsString",
    "CStr",
    "CString",
    "env",
    "thread",
    "Instant",
    "SystemTime",
    "net",
    "TcpStream",
    "UdpSocket",
];

/// Descriptor and `rustix` vocabulary, confined to the one module the owner
/// authorised for it. No other module may touch a descriptor at all.
const AUTHORITY_ONLY: &[&str] = &[
    "rustix",
    "fs",
    "io",
    "OwnedFd",
    "BorrowedFd",
    "AsFd",
    "fcntl_getfl",
    "fstat",
    "pread",
    "retry_on_intr",
    "OFlags",
    "FileType",
    "Mode",
    "RawMode",
    "Errno",
];

/// The P4 public surface is `launch` and `LaunchOutcome`, and the crate root
/// is the only P1/P2 file allowed to name them — once to declare the gated
/// module and once to re-export the pair. Everything else stays forbidden
/// here, and `tests/p4_boundary.rs` makes the claims about `src/launch.rs`
/// itself, which is not a P1/P2 source and is deliberately not in `SOURCES`.
const P4_PUBLIC_NAMES: &[&str] = &["launch", "LaunchOutcome"];

#[test]
fn forbidden_vocabulary_appears_nowhere_outside_the_backend() {
    for (name, source) in SOURCES.iter().chain(TESTS.iter()) {
        for token in code_tokens(source) {
            if *name == "src/lib.rs" && P4_PUBLIC_NAMES.contains(&token.as_str()) {
                continue;
            }
            assert!(
                !FORBIDDEN_EVERYWHERE.contains(&token.as_str()),
                "{name} uses `{token}` as code"
            );
        }
    }
}

#[test]
fn the_crate_root_names_the_p4_public_pair_exactly_as_often_as_it_must() {
    // `mod launch;`, then `pub use launch::{LaunchOutcome, launch};`. Three
    // `launch` tokens and one `LaunchOutcome`, and no fourth appearance that
    // could be a second entry point or a leaked internal.
    let tokens = code_tokens(SOURCES[0].1);
    assert_eq!(SOURCES[0].0, "src/lib.rs");
    assert_eq!(
        tokens.iter().filter(|token| *token == "launch").count(),
        3,
        "the crate root names `launch` an unexpected number of times"
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|token| *token == "LaunchOutcome")
            .count(),
        1,
        "the crate root names `LaunchOutcome` an unexpected number of times"
    );
    // And no other P3/P4 internal is nameable from the root.
    for forbidden in [
        "launch_minimal",
        "PreparedLaunch",
        "SpawnedChild",
        "ChildHandle",
        "MinimalLaunch",
        "ChildPlan",
    ] {
        assert!(
            !tokens.iter().any(|token| token == forbidden),
            "the crate root names `{forbidden}` as code"
        );
    }
}

#[test]
fn product_code_names_no_host_state_and_no_pathname_api() {
    for (name, source) in SOURCES {
        for token in code_tokens(product_code(source)) {
            assert!(
                !FORBIDDEN_IN_PRODUCT_CODE.contains(&token.as_str()),
                "{name} names `{token}` in product code"
            );
        }
    }
    // The scan is real: the Linux tests do use those fixtures APIs.
    let fixtures = code_tokens(test_code(source("src/authority.rs")));
    assert!(fixtures.iter().any(|t| t == "open"), "{fixtures:?}");
    assert!(fixtures.iter().any(|t| t == "unlink"));
}

#[test]
fn only_the_authority_module_names_a_descriptor() {
    for (name, source) in SOURCES {
        if name == "src/authority.rs" {
            continue;
        }
        for token in code_tokens(source) {
            assert!(
                !AUTHORITY_ONLY.contains(&token.as_str()),
                "{name} names `{token}`, which only src/authority.rs may"
            );
        }
    }
    // The scan is real: the authority module names them in product code.
    let authority = code_tokens(product_code(source("src/authority.rs")));
    for expected in ["rustix", "OwnedFd", "fstat", "pread", "fcntl_getfl"] {
        assert!(
            authority.iter().any(|t| t == expected),
            "src/authority.rs stopped using {expected}"
        );
    }
}

#[test]
fn the_authority_module_receives_authority_only_as_a_descriptor() {
    let code: String = strip(product_code(source("src/authority.rs")))
        .split_whitespace()
        .collect();
    // Its only `std` import is the descriptor types; everything else comes from
    // `core`, `rustix`, `sha2` and this crate.
    assert!(
        code.contains("usestd::os::fd::{AsFdas_,BorrowedFd,OwnedFd};"),
        "the descriptor import changed"
    );
    assert_eq!(code.matches("usestd::").count(), 1, "a new std import");
    for forbidden in [
        "std::fs",
        "std::env",
        "std::path",
        "std::io",
        "std::process",
    ] {
        assert!(!code.contains(forbidden), "product code names {forbidden}");
    }
    // No public accessor hands out a descriptor, and the crate-private ones are
    // the only descriptor accessors at all.
    assert_eq!(code.matches("fndescriptor(&self)").count(), 2);
    assert_eq!(code.matches("pub(crate)fndescriptor").count(), 2);
    assert!(!code.contains("pubfndescriptor"));
}

#[test]
fn the_accepted_admission_constants_are_unchanged() {
    let code: String = strip(product_code(source("src/authority.rs")))
        .split_whitespace()
        .collect();
    for constant in [
        "MAX_EXECUTABLE_BYTES:u64=536_870_912;",
        "MEASUREMENT_BUFFER_BYTES:u64=65_536;",
        "ELF_HEADER_BYTES:usize=64;",
        "MODE_BITS_MASK:RawMode=0o7777;",
        "ELFCLASS64:u8=2;",
        "ELFDATA2LSB:u8=1;",
        "ET_EXEC:u16=2;",
        "ET_DYN:u16=3;",
        "EM_X86_64:u16=62;",
    ] {
        assert!(code.contains(constant), "changed or missing: {constant}");
    }
    // Only `MAX_EXECUTABLE_BYTES` is public; the rest are implementation.
    assert!(code.contains("pubconstMAX_EXECUTABLE_BYTES"));
    for private in ["MEASUREMENT_BUFFER_BYTES", "ELF_HEADER_BYTES", "EM_X86_64"] {
        assert!(
            !code.contains(&format!("pubconst{private}")),
            "{private} became public"
        );
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

#[test]
fn no_capability_type_derives_a_forbidden_trait() {
    // `ExecutableCapability`, `WorkingDirectoryCapability` and
    // `AuthorizedLaunch` carry no derive at all: their only trait impls are the
    // three privacy-safe `Debug`s written by hand. The `compile_fail` doctests
    // on each type prove the absence at compile time; this pins the source.
    let code: String = strip(product_code(source("src/authority.rs")))
        .split_whitespace()
        .collect();
    for capability in [
        "ExecutableCapability",
        "WorkingDirectoryCapability",
        "AuthorizedLaunch",
    ] {
        assert!(
            code.contains(&format!("pubstruct{capability}{{")),
            "{capability}"
        );
        assert_eq!(
            code.matches(&format!("implcore::fmt::Debugfor{capability}"))
                .count(),
            1,
            "{capability} must have exactly one hand-written Debug"
        );
        for forbidden in ["Clone", "Copy", "Default", "From", "PartialEq", "Hash"] {
            assert!(
                !code.contains(&format!("impl{forbidden}for{capability}")),
                "{capability} implements {forbidden}"
            );
            assert!(
                !code.contains(&format!("impl{forbidden}<")),
                "{capability}: a generic {forbidden} impl appeared"
            );
        }
    }
    assert_eq!(code.matches("#[derive(").count(), 2, "only the two samples");
    assert!(code.contains("not_sync:PhantomData<Cell<()>>"));
}

// ------------------------------------------------------------ manifest scope

#[test]
fn dependencies_are_exactly_the_p3_set() {
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
        ]),
        "the portable dependency set must not change"
    );

    // Exactly two cohort-gated packages, both at the repository-vetted pins,
    // with default features off. The same `rustix` line appears once as a
    // dependency and once as a dev-dependency.
    // P4 adds exactly one feature, `event`, for the observation loop's
    // `poll`. Nothing else about the pin changed.
    let pinned = concat!(
        r#"rustix = { version = "=1.1.4", default-features = false, "#,
        r#"features = ["std", "fs", "process", "pipe", "event"] }"#
    );
    assert_eq!(CRATE_MANIFEST.matches(pinned).count(), 2);
    assert_eq!(
        CRATE_MANIFEST
            .matches(r#"libc = { version = "=0.2.189", default-features = false }"#)
            .count(),
        1,
        "libc must be declared exactly once, cohort-gated, as a constants-only dependency"
    );
    let gate = format!("[target.'{COHORT}'.dependencies]");
    let dev_gate = format!("[target.'{COHORT}'.dev-dependencies]");
    assert!(CRATE_MANIFEST.contains(&gate), "missing {gate}");
    assert!(CRATE_MANIFEST.contains(&dev_gate), "missing {dev_gate}");

    // P4 enabled `event` for the observation loop's `poll`, and nothing else.
    // `time` in particular stays off: monotonic deadlines are
    // `std::time::Instant`, which is `CLOCK_MONOTONIC` on Linux.
    for feature in ["time", "thread", "mm", "net", "runtime", "use-libc"] {
        assert!(
            !CRATE_MANIFEST.contains(&format!("\"{feature}\"")),
            "rustix feature {feature} must not be enabled"
        );
    }

    // Exactly one feature, non-default and empty, and no build script.
    let features = krate
        .get("features")
        .expect("the test-only injection feature must be declared");
    assert_eq!(
        features.keys().collect::<Vec<&String>>(),
        vec!["test-fault-injection"],
        "an unexpected feature was declared"
    );
    assert!(
        CRATE_MANIFEST.contains("test-fault-injection = []"),
        "the injection feature must enable nothing else"
    );
    assert!(
        !CRATE_MANIFEST.contains(
            "[features]
default"
        ),
        "a default feature set appeared"
    );

    let target_tables: Vec<&String> = krate
        .keys()
        .filter(|table| table.starts_with("target"))
        .collect();
    assert_eq!(
        target_tables.len(),
        2,
        "unexpected target tables: {target_tables:?}"
    );
    for table in krate.keys() {
        assert!(
            !table.contains("build-dependencies"),
            "unexpected manifest table [{table}]"
        );
    }
    let package = &krate["package"];
    assert_eq!(package["name"], "helm-launch");
    assert_eq!(package["version"], "0.1.0");
    assert_eq!(package["edition"], "2024");
    assert_eq!(package["rust-version"], "1.95");
    assert_eq!(package["publish"], "false");
    assert!(!CRATE_MANIFEST.contains("build.rs"), "a build script");
    // `libc` is cohort-gated only: it must never become a portable dependency.
    assert!(
        !krate["dependencies"].contains_key("libc"),
        "libc became a portable dependency"
    );
}

#[test]
fn the_lockfile_entry_has_exactly_the_p3_dependencies() {
    let start = LOCKFILE.find("name = \"helm-launch\"").unwrap();
    let entry = &LOCKFILE[start..];
    let entry = &entry[..entry
        .find(
            "

",
        )
        .unwrap_or(entry.len())];
    let deps: Vec<&str> = entry
        .lines()
        .filter(|l| l.starts_with(" \""))
        .map(|l| l.trim().trim_matches([',', '"']))
        .collect();
    assert_eq!(
        deps,
        ["libc", "rustix", "serde", "serde_json", "sha2"],
        "the lockfile entry changed beyond the one constants-only addition"
    );
    assert!(entry.contains("version = \"0.1.0\""));
    assert!(
        !entry.contains("source ="),
        "helm-launch must be a local package"
    );

    // The two cohort packages are the ones the repository already vetted: the
    // same versions, so no package and no version was added.
    let rustix = LOCKFILE
        .find("name = \"rustix\"")
        .map(|at| &LOCKFILE[at..])
        .unwrap();
    let rustix = &rustix[..rustix
        .find(
            "

",
        )
        .unwrap_or(rustix.len())];
    assert!(rustix.contains("version = \"1.1.4\""), "{rustix}");
    assert_eq!(
        LOCKFILE.matches("name = \"rustix\"").count(),
        1,
        "a second rustix version entered the lockfile"
    );
    let pinned_c_bindings = LOCKFILE
        .find("name = \"libc\"")
        .map(|at| &LOCKFILE[at..])
        .unwrap();
    let pinned_c_bindings = &pinned_c_bindings[..pinned_c_bindings
        .find(
            "

",
        )
        .unwrap_or(pinned_c_bindings.len())];
    assert!(
        pinned_c_bindings.contains("version = \"0.2.189\""),
        "{pinned_c_bindings}"
    );
    assert_eq!(
        LOCKFILE.matches("name = \"libc\"").count(),
        1,
        "a second libc version entered the lockfile"
    );
    assert!(
        !LOCKFILE.contains("name = \"cc\""),
        "a build-time compiler dependency entered the lockfile"
    );
}
