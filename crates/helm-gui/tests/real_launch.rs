//! The real launch, end to end, with no window.
//!
//! This is the test that proves the vertical is real: it drives the **same**
//! orchestration adapter the interface uses, against a real ELF subject, and
//! asserts on a real receipt. It needs no display, so hosted Linux CI can run
//! it.
//!
//! It asserts what was observed, and refuses to assert more. In particular it
//! requires that a clean exec-status end-of-file is **not** turned into proof
//! that execution began: there is no `ExecSucceeded` in the accepted model and
//! this test exists partly to keep it that way.

// A failing assertion is how a test reports, and the fixture paths are
// constructed by this file. Panicking here is the mechanism, not a defect.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test code: a failed assertion is the reporting mechanism"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use helm_gui::state;
use helm_gui::vertical;
use helm_launch::{ChildEnd, Completeness, ExecStatus, Stream};
use sha2::{Digest as _, Sha256};

/// The mode the fixture is given, and then asserted against the receipt.
const FIXTURE_MODE: u32 = 0o755;

struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "helm-g2-vertical-{}-{tag}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir_all(&root).expect("create scratch root");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn dir(&self, name: &str) -> PathBuf {
        let dir = self.root.join(name);
        fs::create_dir_all(&dir).expect("create scratch directory");
        dir
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Every artefact this test made is removed, whether it passed or not.
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Copies the purpose-built fixture and gives it an explicit, asserted mode.
fn place_fixture(scratch: &Scratch) -> PathBuf {
    let source = Path::new(env!("CARGO_BIN_EXE_g2-launch-fixture"));
    assert!(source.is_file(), "fixture binary was not built: {source:?}");
    let target = scratch.path("g2-subject");
    fs::copy(source, &target).expect("copy fixture");
    fs::set_permissions(&target, fs::Permissions::from_mode(FIXTURE_MODE))
        .expect("set fixture mode");
    let placed = fs::metadata(&target).expect("stat fixture").permissions();
    assert_eq!(
        placed.mode() & 0o7777,
        FIXTURE_MODE,
        "fixture mode was not applied"
    );
    target
}

#[test]
fn the_gui_adapter_drives_a_real_launch_to_a_real_receipt() {
    let scratch = Scratch::new("ok");
    let subject = place_fixture(&scratch);
    let workdir = scratch.dir("work");

    // --- real admission, through the accepted public API -------------------
    let executable = match vertical::admit_executable_at(&subject) {
        Ok(capability) => capability,
        Err(refusal) => panic!("executable admission refused: {refusal:?}"),
    };
    let measurement = executable.measurement();
    assert_eq!(
        measurement.pre_exec_mode_bits(),
        u16::try_from(FIXTURE_MODE).unwrap(),
        "the receipt must carry the mode the fixture was actually given"
    );
    assert!(measurement.pre_exec_body_size() > 0);

    let working_directory = match vertical::admit_working_directory_at(&workdir) {
        Ok(capability) => capability,
        Err(refusal) => panic!("working-directory admission refused: {refusal:?}"),
    };

    // --- the real plan, through the accepted parser -------------------------
    let argv0 = vertical::argv0_for(&subject);
    let plan = match vertical::parse_plan(&argv0) {
        Ok(plan) => plan,
        Err(errors) => panic!("the fixed plan did not parse: {:?}", errors.as_slice()),
    };
    assert_eq!(plan.argv(), [argv0.clone()].as_slice());
    assert_eq!(plan.working_directory_id(), vertical::WORKING_DIRECTORY_ID);
    assert_eq!(plan.timeout_ms(), vertical::TIMEOUT_MS);
    assert_eq!(plan.grace_ms(), vertical::GRACE_MS);
    let plan_digest = plan.sha256();

    // --- authority is composed only here, and consumed exactly once ---------
    let authorized = match vertical::authorise(plan, executable, working_directory) {
        Ok(authorized) => authorized,
        Err(refusal) => panic!("authorisation refused: {:?}", refusal.code()),
    };

    // --- the real launch ----------------------------------------------------
    let outcome = match helm_launch::launch(authorized) {
        Ok(outcome) => outcome,
        Err(error) => panic!(
            "launch could not begin: {}",
            state::launch_error_detail(&error)
        ),
    };

    // --- what was actually observed ----------------------------------------
    let receipt = outcome.receipt();
    let record = receipt.record();

    assert_eq!(
        record.child_end(),
        ChildEnd::Exited { code: 0 },
        "the fixture ends by exit, reporting 0"
    );
    assert_eq!(record.plan_sha256(), plan_digest);
    assert_eq!(
        record.working_directory_id(),
        vertical::WORKING_DIRECTORY_ID
    );
    assert_eq!(record.argument_count(), 1);
    assert!(!record.run_deadline_expired());

    // The marker proves the subject really ran and really wrote to stdout.
    let stdout = String::from_utf8_lossy(outcome.stdout_prefix()).into_owned();
    assert!(
        stdout.contains("helm-g2-fixture-stdout-marker"),
        "expected stdout marker, got {stdout:?}"
    );
    let stderr = String::from_utf8_lossy(outcome.stderr_prefix()).into_owned();
    assert!(
        stderr.contains("helm-g2-fixture-stderr-marker"),
        "expected stderr marker, got {stderr:?}"
    );
    assert!(!outcome.prefix_truncated(Stream::Stdout));
    assert_eq!(
        record.stream(Stream::Stdout).completeness(),
        Completeness::CompleteAtEof
    );

    // --- the receipt digest is the identity of the receipt's own bytes ------
    let recomputed = Sha256::digest(receipt.exact_bytes());
    assert_eq!(
        recomputed.as_slice(),
        receipt.sha256().as_bytes().as_slice(),
        "the digest shown in the interface must be recomputable from the exact bytes"
    );
    assert!(
        core::str::from_utf8(receipt.exact_bytes()).is_ok(),
        "the receipt's exact bytes are UTF-8 under the accepted schema"
    );

    // --- the nonclaim survives a completely ordinary, successful-looking run
    assert!(
        matches!(record.exec_status(), ExecStatus::Indeterminate(_)),
        "there is no exec-success value; a clean run stays indeterminate"
    );
    let detail = state::exec_status_detail(record.exec_status());
    assert!(
        !detail.contains("succeeded") && !detail.contains("success"),
        "exec status must never be spelled as success: {detail}"
    );
    let normal = state::exec_status_normal(record.exec_status());
    assert!(
        normal.contains("did not establish") || normal.contains("did not begin running"),
        "normal copy must not claim execution was established: {normal}"
    );

    // --- and the words the interface would show are the observed ones -------
    assert_eq!(
        state::child_end_headline(record.child_end()),
        "The program ended and reported 0."
    );
    assert_eq!(
        state::deadline_fact_of(record),
        "The deadline did not expire. HELM issued no stop and no force."
    );
}

#[test]
fn a_non_elf_file_is_refused_by_real_admission() {
    let scratch = Scratch::new("notelf");
    let not_elf = scratch.path("notes.txt");
    fs::write(&not_elf, b"this is not an ELF executable\n").expect("write fixture");

    match vertical::admit_executable_at(&not_elf) {
        Ok(_) => panic!("a text file must not be admitted as an executable"),
        Err(refusal) => {
            let admission = refusal
                .admission()
                .expect("a non-ELF file is refused by helm-launch, not by the caller");
            assert_eq!(admission.code(), helm_launch::AdmissionErrorCode::NotElf);
            // And the interface explains the condition without judging the file.
            let text = state::admission_text(admission.code());
            assert!(text.contains("ELF"));
            assert!(!text.to_lowercase().contains("dangerous"));
        }
    }
}

#[test]
fn a_file_is_refused_as_a_working_directory() {
    let scratch = Scratch::new("notdir");
    let file = scratch.path("plain.txt");
    fs::write(&file, b"not a directory\n").expect("write fixture");

    match vertical::admit_working_directory_at(&file) {
        Ok(_) => panic!("a regular file must not be admitted as a working directory"),
        Err(refusal) => {
            let admission = refusal.admission().expect("refused by helm-launch");
            assert_eq!(
                admission.code(),
                helm_launch::AdmissionErrorCode::NotDirectory
            );
        }
    }
}

#[test]
fn a_directory_is_refused_as_an_executable() {
    let scratch = Scratch::new("dir-as-exe");
    let dir = scratch.dir("somewhere");

    match vertical::admit_executable_at(&dir) {
        Ok(_) => panic!("a directory must not be admitted as an executable"),
        Err(refusal) => {
            let admission = refusal.admission().expect("refused by helm-launch");
            assert_eq!(
                admission.code(),
                helm_launch::AdmissionErrorCode::NotRegularFile
            );
        }
    }
}

#[test]
fn the_fixed_plan_document_is_accepted_by_the_real_parser() {
    // The adapter must not be able to emit a document the accepted parser
    // rejects, including when the selected file name needs JSON escaping.
    for argv0 in [
        "program",
        "a name with spaces",
        "quote\"inside",
        "back\\slash",
        "emoji-🙂",
    ] {
        match vertical::parse_plan(argv0) {
            Ok(plan) => assert_eq!(plan.argv(), [argv0.to_owned()].as_slice()),
            Err(errors) => panic!(
                "argv0 {argv0:?} produced an invalid plan: {:?}",
                errors.as_slice()
            ),
        }
    }
}
