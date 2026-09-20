//! P5 Level 4 — adversarial regressions derived from the Trial #1–#3 defects.
//!
//! # What this file is
//!
//! [Productization plan section 14.4] lists fifteen regressions, each one a
//! defect that really happened in Trial #1, #2 or #3 and each one able to turn
//! a product case into a harness accident. Most of them were already closed by
//! P3 and P4 and are **not** duplicated here. This file carries the four that
//! were open, plus one executable inventory that fails if any of the other
//! eleven loses its test.
//!
//! [Productization plan section 14.4]: ../../../docs/implementation/HELM-LAUNCH-PRODUCTIZATION-PLAN.md
//!
//! # Why it reads the tree instead of running things
//!
//! Three of the four open rows are properties of the **test support** itself —
//! how fixtures are built, how a setup failure ends, how a mutation is checked.
//! A runtime test cannot observe them, because a harness that reads through a
//! write-only descriptor fails *as a test*, exactly as `E5b` did. Reading the
//! source catches that class before it runs, and it runs on every platform
//! because the tree is the same everywhere. This file executes nothing, creates
//! no process and opens no descriptor.
//!
//! # Positive controls
//!
//! Every scanner here is run against a synthetic violation as well as against
//! the tree, so "no findings" can never quietly mean "found nothing".

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: a wrong assumption must fail loudly"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};

const BACKEND_TESTS: &str = "src/backend/tests.rs";

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn rust_files(directory: &str) -> BTreeMap<String, String> {
    let root = crate_root().join(directory);
    let mut found = BTreeMap::new();
    let mut stack = vec![root];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).expect("read the source tree") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|ext| ext == "rs") {
                let relative = path
                    .strip_prefix(crate_root())
                    .expect("inside the crate")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect("read a source file");
                found.insert(relative, text);
            }
        }
    }
    found
}

fn every_rust_file() -> BTreeMap<String, String> {
    let mut all = rust_files("src");
    all.extend(rust_files("tests"));
    all
}

fn read_backend_tests() -> String {
    std::fs::read_to_string(crate_root().join(BACKEND_TESTS)).expect("read the backend test module")
}

// ===========================================================================
// A. Trial #1 E5b — a test helper read back through a write-only descriptor
// ===========================================================================

/// Statements of the form `let [mut] NAME = …;`, joined onto one line.
fn let_statements(source: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut pending: Option<(String, String)> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some((name, text)) = pending.take() {
            let joined = format!("{text} {trimmed}");
            if trimmed.ends_with(';') {
                out.push((name, joined));
            } else {
                pending = Some((name, joined));
            }
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("let ") else {
            continue;
        };
        let rest = rest.strip_prefix("mut ").unwrap_or(rest);
        let Some((binding, _)) = rest.split_once('=') else {
            continue;
        };
        let name = binding.trim().trim_end_matches(':').trim();
        if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            continue;
        }
        let statement = (name.to_owned(), trimmed.to_owned());
        if trimmed.ends_with(';') {
            out.push(statement);
        } else {
            pending = Some(statement);
        }
    }
    out
}

/// Names bound to a descriptor that was opened **without** read access.
fn write_only_bindings(source: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (name, statement) in let_statements(source) {
        let creates = statement.contains("File::create(");
        let write_open = statement.contains("OpenOptions::new()")
            && statement.contains(".write(true)")
            && statement.contains(".open(");
        let readable = statement.contains(".read(true)");
        if (creates || write_open) && !readable {
            out.insert(name);
        }
    }
    out
}

/// Every place a name is used as the source of a read.
fn reads_through(source: &str, name: &str) -> bool {
    let patterns = [
        format!("{name}.read("),
        format!("{name}.read_to_end("),
        format!("{name}.read_to_string("),
        format!("{name}.read_exact("),
        format!("read_to_end(&mut {name}"),
        format!("read_to_string(&mut {name}"),
        format!("read_exact(&mut {name}"),
    ];
    patterns.iter().any(|pattern| source.contains(pattern))
}

fn e5b_violations(source: &str) -> Vec<String> {
    write_only_bindings(source)
        .into_iter()
        .filter(|name| reads_through(source, name))
        .collect()
}

/// **Trial #1 `E5b`.** A mutation helper that verifies its own write through
/// the descriptor it wrote with reads a handle that cannot read, the call
/// fails, and the case aborts in setup — looking, in the log, exactly like the
/// mechanism refusing something. Verification must go through a separately
/// opened, read-capable descriptor.
#[test]
fn level4_e5b_no_test_helper_reads_through_a_write_only_descriptor() {
    // Positive control: the scanner must catch the defect it exists for.
    let violation = r#"
        fn helper() {
            let mut sink = File::create(path).unwrap();
            sink.write_all(b"x").unwrap();
            let mut back = String::new();
            sink.read_to_string(&mut back).unwrap();
        }
    "#;
    assert_eq!(
        e5b_violations(violation),
        vec!["sink".to_owned()],
        "the E5b scanner did not catch a synthetic write-only read-back"
    );

    // Negative control: verifying through a separate read-capable open is the
    // shape the rule asks for, and must not be reported.
    let clean = r#"
        fn helper() {
            let mut sink = File::create(path).unwrap();
            sink.write_all(b"x").unwrap();
            let back = fs::read_to_string(path).unwrap();
        }
    "#;
    assert!(
        e5b_violations(clean).is_empty(),
        "the E5b scanner reported a helper that re-opened for reading"
    );

    for (name, source) in every_rust_file() {
        let found = e5b_violations(&source);
        assert!(
            found.is_empty(),
            "{name}: these bindings were opened write-only and then read back: {found:?}. \
             Trial #1 E5b: verify a mutation through a separately opened read-capable descriptor."
        );
    }
}

// ===========================================================================
// B. Trial #1 / definition 9.6 — a setup failure is not a mechanism result
// ===========================================================================

/// **Trial #1, definition section 9.6.** A test that cannot construct its own
/// state must end as a **harness** failure carrying that label, and must never
/// be able to become an assertion about the launcher, a receipt value or a
/// pass/fail verdict about the mechanism.
#[test]
fn level4_a_setup_failure_is_typed_and_never_a_mechanism_result() {
    let support = read_backend_tests();

    assert!(
        support.contains("pub(crate) struct NotPosed {"),
        "{BACKEND_TESTS}: the typed not-posed result is missing"
    );
    assert!(
        support.contains("pub(crate) fn fail(self) -> !"),
        "{BACKEND_TESTS}: NotPosed::fail must diverge, so no caller can continue with a value"
    );

    // The label a reader sees must say what it is and what it is not.
    for phrase in [
        "NOT POSED",
        "No case was posed",
        "NOT a launcher result",
        "NOT a receipt",
        "NOT a verdict about the mechanism",
    ] {
        assert!(
            support.contains(phrase),
            "{BACKEND_TESTS}: the not-posed message no longer states {phrase:?}"
        );
    }

    // The one setup precondition the suite checks for every tracer and fixture
    // test goes through it, rather than through an ad-hoc panic string.
    assert!(
        support.contains("_ => NotPosed::new("),
        "{BACKEND_TESTS}: require_tool no longer reports a missing tool as not-posed"
    );

    // It is test support only: no product source may name it.
    for (name, source) in rust_files("src") {
        if name == BACKEND_TESTS {
            continue;
        }
        assert!(
            !source.contains("NotPosed"),
            "{name}: NotPosed escaped into product source; it is a harness type only"
        );
    }
}

// ===========================================================================
// C. Trial #2 X2b/X2c/X4 — fixture permission and mode assumptions
// ===========================================================================

/// **Trial #2 `X2b`/`X2c`/`X4`.** `0666 & ~umask` is not a constant. A fixture
/// whose mode was assumed rather than checked can silently make a permission
/// case untestable, and the run still looks like a product result.
#[test]
fn level4_every_generated_fixture_mode_is_asserted_after_writing() {
    let support = read_backend_tests();

    assert!(
        support.contains("pub(crate) fn set_and_assert_mode(path: &Path, mode: u32)"),
        "{BACKEND_TESTS}: the asserting mode helper is missing"
    );
    assert!(
        support.contains("fixture mode was not applied at"),
        "{BACKEND_TESTS}: the mode helper no longer asserts what it applied"
    );

    // Exactly one raw `set_permissions` may remain in the module: the one
    // inside the helper that asserts immediately afterwards.
    let raw = support.matches("fs::set_permissions(").count();
    assert_eq!(
        raw, 1,
        "{BACKEND_TESTS}: {raw} raw set_permissions calls; every fixture mode must go through \
         set_and_assert_mode so the applied mode is checked"
    );

    // The compiled fixture itself is checked for the bit the suite depends on.
    assert!(
        support.contains("is not executable: {mode:o}"),
        "{BACKEND_TESTS}: the compiled fixture no longer asserts its executable bits"
    );
}

// ===========================================================================
// H. Trial #2 O6/O7, E4 — relative fixture paths broken by fchdir
// ===========================================================================

/// **Trial #2 `O6`/`O7` and `E4`.** A child that has already run `fchdir`
/// resolves a relative pathname against the **admitted** directory. A fixture
/// path that is relative, or absolute through a symlink, therefore names a
/// different object in the child than in the test.
#[test]
fn level4_fixture_paths_handed_to_children_are_canonical_and_absolute() {
    let support = read_backend_tests();

    assert!(
        support.contains(r#"fs::canonicalize(&root).expect("canonical fixture root")"#),
        "{BACKEND_TESTS}: the fixture root is no longer canonicalised"
    );
    assert!(
        support.contains("root.is_absolute()"),
        "{BACKEND_TESTS}: the fixture root no longer asserts that it is absolute"
    );

    // Every fixture path is built from that one root, so canonicalising the
    // root canonicalises all of them. A second root would defeat that.
    let roots = support.matches("std::env::temp_dir()").count();
    assert_eq!(
        roots, 1,
        "{BACKEND_TESTS}: {roots} temporary-directory roots; every fixture path must descend \
         from the single canonical fixture root"
    );
}

// ===========================================================================
// J. Trial #3 R3-M1 — a published digest must be recomputable
// ===========================================================================

const VECTORS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/implementation/helm-launch-receipt-0.1-test-vectors.json"
));

fn unhex(text: &str) -> Vec<u8> {
    assert!(text.len().is_multiple_of(2), "odd hex run");
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("hex pair"))
        .collect()
}

/// **Trial #3 `R3-M1`.** A digest published beside an artifact proves nothing
/// unless it can be recomputed **from that artifact**. This check never touches
/// the serializer: it reads the committed file and recomputes every digest from
/// the committed bytes, which is exactly what an outside reader can do.
#[test]
fn level4_every_published_receipt_digest_is_recomputable_from_the_artifact() {
    let document: serde_json::Value =
        serde_json::from_str(VECTORS).expect("the published vector file is strict JSON");

    assert_eq!(
        document["authenticity"].as_str(),
        Some("none"),
        "the vector artifact must keep stating that it carries no authenticity claim"
    );

    let vectors = document["vectors"].as_array().expect("vectors is an array");
    assert!(!vectors.is_empty(), "the published vector set is empty");

    let mut names = BTreeSet::new();
    for vector in vectors {
        let name = vector["name"].as_str().expect("vector name");
        assert!(names.insert(name), "{name}: duplicate vector name");

        let bytes = unhex(vector["exact_bytes_base16"].as_str().expect("base16 bytes"));

        // The readable copy must be the same bytes, so a reviewer reading the
        // file sees the real receipt and not a differently escaped near-miss.
        let readable = vector["exact_bytes_utf8"].as_str().expect("utf8 bytes");
        assert_eq!(
            readable.as_bytes(),
            bytes.as_slice(),
            "{name}: the two published encodings are not the same bytes"
        );

        assert_eq!(
            vector["exact_byte_length"].as_u64(),
            Some(bytes.len() as u64),
            "{name}: published length disagrees with the published bytes"
        );

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let recomputed: String = hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(
            Some(recomputed.as_str()),
            vector["sha256"].as_str(),
            "{name}: the published digest is not sha256 of the published bytes"
        );

        assert!(
            bytes.len() <= helm_launch::MAX_RECEIPT_BYTES,
            "{name}: published vector exceeds MAX_RECEIPT_BYTES"
        );

        let text = String::from_utf8(bytes).expect("receipts are UTF-8");
        for term in ["success", "succeeded", "authentic", "verified", "trusted"] {
            assert!(
                !text.contains(term),
                "{name}: a published vector contains the verdict term {term:?}"
            );
        }
    }

    assert_eq!(
        document["max_receipt_bytes"].as_u64(),
        Some(helm_launch::MAX_RECEIPT_BYTES as u64),
        "the published bound drifted from the product constant"
    );
}

// ===========================================================================
// Current-truth vocabulary scan
// ===========================================================================

/// Phrases that assert a **current** absence of something P4 now has. They are
/// deliberately narrow: `future`, `P4` and `P5` are ordinary words in a
/// historical record, and this scan covers **crate-facing** text only, never
/// `docs/`, where superseded statements are supposed to survive verbatim.
const STALE_CURRENT_CLAIMS: [&str; 7] = [
    "future P4",
    "future launch",
    "a future slice",
    "no public launch",
    "P4 is not authorised",
    "P4 remains not authorised",
    "does not exist here",
];

fn stale_claims(source: &str) -> Vec<&'static str> {
    STALE_CURRENT_CLAIMS
        .into_iter()
        .filter(|claim| source.contains(claim))
        .collect()
}

/// **`P4DOC-01`.** Current crate-facing text must not still say the accepted
/// P4 surface is absent or unauthorised. Historical records elsewhere are
/// untouched by this rule, and must stay that way.
#[test]
fn current_crate_facing_text_does_not_deny_the_accepted_p4_surface() {
    // Positive control.
    let violation = "// recorded for a future P4 sweep; nothing consumes it";
    assert_eq!(
        stale_claims(violation),
        vec!["future P4"],
        "the current-truth scanner did not catch a synthetic stale claim"
    );
    // Negative control: a historical sentence naming P4 is not a stale claim.
    let historical = "// P3 issued no sweep; the accepted P4 slice is the only consumer.";
    assert!(
        stale_claims(historical).is_empty(),
        "the current-truth scanner flagged an ordinary historical mention"
    );

    let mut sources = every_rust_file();
    sources.insert(
        "README.md".to_owned(),
        std::fs::read_to_string(crate_root().join("README.md")).expect("read the crate README"),
    );

    // This file necessarily contains every phrase it forbids, as the data it
    // searches for. It is the only exclusion, and the assertion below keeps it
    // the only one: any other file that needs excusing is a real finding.
    const SCANNER: &str = "tests/p5_regressions.rs";
    assert!(
        sources.contains_key(SCANNER),
        "the scanner no longer sees its own file, so the exclusion below is stale"
    );

    for (name, source) in sources {
        if name == SCANNER {
            continue;
        }
        let found = stale_claims(&source);
        assert!(
            found.is_empty(),
            "{name}: current crate-facing text still claims {found:?}; P4 is accepted"
        );
    }
}

// ===========================================================================
// The executable Level 1–4 inventory
// ===========================================================================

/// Every accepted obligation, and the test that actually proves it.
///
/// `(level, row, test function, file)`. The point is not documentation: the
/// test below fails if any named function stops existing, so an obligation
/// cannot quietly lose its evidence during a refactor.
const INVENTORY: [(&str, &str, &str, &str); 26] = [
    // ---- Level 4, section 14.4, in the accepted table's order -------------
    (
        "4",
        "A: write-only descriptor helper (Trial #1 E5b)",
        "level4_e5b_no_test_helper_reads_through_a_write_only_descriptor",
        "tests/p5_regressions.rs",
    ),
    (
        "4",
        "B: setup failure is not a mechanism result (Trial #1, 9.6)",
        "level4_a_setup_failure_is_typed_and_never_a_mechanism_result",
        "tests/p5_regressions.rs",
    ),
    (
        "4",
        "C: fixture permission and mode assumptions (Trial #2 X2b/X2c/X4)",
        "level4_every_generated_fixture_mode_is_asserted_after_writing",
        "tests/p5_regressions.rs",
    ),
    (
        "4",
        "D: fd collision and layout assumptions (F6/F7)",
        "generated_distinct_layouts_satisfy_every_property",
        "src/layout.rs",
    ),
    (
        "4",
        "D: fd layout, simulated hosts (F6/F7)",
        "simulated_hosts_end_with_exactly_stdio_in_the_image",
        "src/layout.rs",
    ),
    (
        "4",
        "D: fd isolation in a real child (F6/F7)",
        "an_authorised_object_executes_with_exactly_the_intended_descriptors",
        "src/backend/tests.rs",
    ),
    (
        "4",
        "E: CLD_DUMPED keeps its signal (Trial #2 R3)",
        "a_signalled_child_keeps_its_signal_number_and_its_core_flag",
        "src/launch.rs",
    ),
    (
        "4",
        "E: exit and signal stay distinct at reap (Trial #2 R3)",
        "reap_classification_keeps_exit_and_signal_distinct",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "F: clean EOF is never exec success (S5, Trial #2 S4)",
        "a_clean_status_eof_is_indeterminate_and_never_exec_success",
        "src/launch.rs",
    ),
    (
        "4",
        "F: the same in the pure model (S5, Trial #2 S4)",
        "s5_status_eof_then_death_is_indeterminate_and_identical_to_a_normal_run",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "F: no function offers a success reading (S5, Trial #2 S4)",
        "no_function_offers_a_success_reading",
        "tests/p2_boundary.rs",
    ),
    (
        "4",
        "G: clone3 process-vs-thread trace disambiguation (Trial #2 M2)",
        "the_child_window_is_closed_from_a_multithreaded_allocating_parent",
        "src/backend/tests.rs",
    ),
    (
        "4",
        "H: relative fixture paths broken by fchdir (Trial #2 O6/O7, E4)",
        "level4_fixture_paths_handed_to_children_are_canonical_and_absolute",
        "tests/p5_regressions.rs",
    ),
    (
        "4",
        "I: report producer vs evidence consumer (Trial #3 X2c)",
        "report_fixture_reports_without_the_backend",
        "src/backend/tests.rs",
    ),
    (
        "4",
        "I: the report schema is closed (Trial #3 X2c)",
        "the_producer_report_schema_is_closed_and_a_malformed_report_is_refused",
        "src/backend/tests.rs",
    ),
    (
        "4",
        "J: digest recomputable from the published artifact (Trial #3 R3-M1)",
        "level4_every_published_receipt_digest_is_recomputable_from_the_artifact",
        "tests/p5_regressions.rs",
    ),
    (
        "4",
        "J: published vectors are the production serializer's own bytes",
        "published_receipt_vectors_match_the_production_serializer",
        "src/receipt.rs",
    ),
    (
        "4",
        "K: a read error is never EOF (T41)",
        "t41_read_errors_are_their_own_facts_and_never_eof",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "L: no blocking reap after SIGKILL (T40)",
        "t40_no_end_within_the_kill_bound_is_end_not_observed_on_both_kill_paths",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "L: end_not_observed is latched (T40)",
        "t40_end_not_observed_is_latched_and_a_later_reap_never_revises_it",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "L: exactly one post-kill wait exists (T40)",
        "the_total_bound_counts_one_post_kill_wait_and_the_drop_guard_is_handed_back",
        "tests/p4_boundary.rs",
    ),
    (
        "4",
        "M: no sweep after a foreign reap (R4, T31)",
        "a_foreign_reaper_suppresses_the_sweep_and_leaves_the_end_unobservable",
        "src/launch.rs",
    ),
    (
        "4",
        "M: the same in the pure model (R4, T31)",
        "t31_a_child_already_reaped_elsewhere_gets_no_sweep",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "N: no sweep without established group authority (T31)",
        "t31_without_group_authority_no_path_issues_a_sweep",
        "src/lifecycle.rs",
    ),
    (
        "4",
        "N: authority is never inferred from an observed group (T31)",
        "no_source_infers_group_authority_from_an_observed_process_group",
        "tests/p4_boundary.rs",
    ),
    (
        "4",
        "O: full signal mask across clone3, including glibc's own (T21)",
        "the_blocked_mask_covers_every_signal_including_the_two_glibc_keeps_to_itself",
        "src/backend/tests.rs",
    ),
];

/// Every Level 4 obligation names a test that exists, in the file it claims.
///
/// This is the answer to "what proves every accepted obligation?" without
/// reconstructing it from review documents.
#[test]
fn the_level_4_inventory_names_a_real_test_for_every_accepted_obligation() {
    let sources = every_rust_file();
    let mut rows = BTreeSet::new();

    for (level, row, test, file) in INVENTORY {
        assert_eq!(level, "4", "{row}: unexpected level");
        assert!(
            rows.insert((row, test)),
            "{row}: duplicated inventory row for {test}"
        );
        let source = sources
            .get(file)
            .unwrap_or_else(|| panic!("{row}: inventory names a missing file {file}"));
        assert!(
            source.contains(&format!("fn {test}(")),
            "{row}: {file} no longer defines {test}; the obligation lost its evidence"
        );
        // A named test must really be a test, not a helper that happens to
        // share the name.
        let at = source.find(&format!("fn {test}(")).expect("found above");
        let preceding = &source[..at];
        assert!(
            preceding.is_marked_as_a_test(),
            "{row}: {test} in {file} is not marked #[test]"
        );
    }

    // All fifteen accepted 14.4 rows are represented.
    let covered: BTreeSet<char> = INVENTORY
        .iter()
        .map(|(_, row, _, _)| row.chars().next().expect("a row label"))
        .collect();
    let expected: BTreeSet<char> = "ABCDEFGHIJKLMNO".chars().collect();
    assert_eq!(
        covered, expected,
        "the inventory does not cover every accepted section 14.4 row"
    );
}

/// `#[test]` must be the last attribute before the function this inventory
/// names. Kept as an extension so the assertion above reads as one claim.
trait TestAttribute {
    fn is_marked_as_a_test(&self) -> bool;
}

impl TestAttribute for str {
    fn is_marked_as_a_test(&self) -> bool {
        self.lines()
            .rev()
            .take(12)
            .any(|line| line.trim() == "#[test]")
    }
}
