//! End-to-end binding against **real** observations.
//!
//! `ObservationArtifact` has no constructor outside the observation backend, so a
//! genuine artifact can only be produced where that backend exists: Linux x86_64.
//! These tests therefore carry the same gate. Everything they exercise is pure
//! platform-independent comparison logic; the platform is needed only to obtain
//! an input. Synthetic fixtures only: no A0 access, no Wine, no 7-Zip.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use helm_bind::{
    BindingRefusalCode as R, ClaimState, ClaimSubject, Contradiction, Difference, FailureReason,
    OmissionReason, RejectionReason, bind, parse_binding_plan,
};
use helm_observe::{ObservationArtifact, ValidatedPlan};

const NOTES_SPEC: &[u8] = include_bytes!("fixtures/synthetic-notes.json");
const A0_SPEC: &[u8] = include_bytes!("fixtures/a0-7zip.json");

// The literal byte strings the committed fixture's identities were derived from.
const SOURCE_BYTES: &[u8] = b"Synthetic application source; not an executable.\n";
const RUNTIME_BYTES: &[u8] = b"Synthetic runtime artifact; not Wine.\n";
const ENTRY_BYTES: &[u8] = b"Synthetic entry point; not executable.\n";
const DEFINITION_BYTES: &[u8] = b"Synthetic pre-execution definition: preserve note bytes.\n";

fn spec() -> helm_app_spec::ValidatedAppSpec {
    helm_app_spec::parse_spec(NOTES_SPEC).unwrap()
}

fn spec_digest_hex(raw: &[u8]) -> String {
    use sha2::Digest as _;
    let d: [u8; 32] = sha2::Sha256::digest(raw).into();
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn temp_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("helm-bind-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

/// A synthetic tree whose bodies reproduce the committed fixture identities,
/// alongside decoys that exercise every observation outcome.
fn build_fixtures(base: &Path) {
    fs::write(base.join("source.bin"), SOURCE_BYTES).unwrap();
    fs::write(base.join("runtime.bin"), RUNTIME_BYTES).unwrap();
    fs::write(base.join("definition.bin"), DEFINITION_BYTES).unwrap();
    fs::create_dir_all(base.join("drive_c/Notes")).unwrap();
    fs::write(base.join("drive_c/Notes/notes.exe"), ENTRY_BYTES).unwrap();
    // Same bytes, another relative path.
    fs::create_dir_all(base.join("elsewhere")).unwrap();
    fs::write(base.join("elsewhere/notes.exe"), ENTRY_BYTES).unwrap();
    // Different length and different digest.
    fs::write(base.join("wrong.bin"), b"different body entirely\n").unwrap();
    // Same length as the source body, different digest.
    fs::write(base.join("samelen.bin"), vec![b'x'; SOURCE_BYTES.len()]).unwrap();
    fs::create_dir_all(base.join("adir")).unwrap();
    std::os::unix::fs::symlink("source.bin", base.join("link")).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(base.join("fifo"))
            .status()
            .unwrap()
            .success()
    );
    let denied = base.join("denied.bin");
    fs::write(&denied, b"unreadable\n").unwrap();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
    // Sparse and over the observer's per-file ceiling: refused on metadata alone.
    fs::File::create(base.join("over.bin"))
        .unwrap()
        .set_len(helm_observe::MAX_FILE_BYTES + 1)
        .unwrap();
}

fn observation_plan_json(subject_hex: &str) -> Vec<u8> {
    let targets = [
        ("src", "source.bin", "regular_file_sha256"),
        ("rt", "runtime.bin", "regular_file_sha256"),
        ("def", "definition.bin", "regular_file_sha256"),
        ("entry", "drive_c/Notes/notes.exe", "regular_file_sha256"),
        ("elsewhere", "elsewhere/notes.exe", "regular_file_sha256"),
        ("wrong", "wrong.bin", "regular_file_sha256"),
        ("samelen", "samelen.bin", "regular_file_sha256"),
        ("missing", "nope.bin", "regular_file_sha256"),
        ("link", "link", "regular_file_sha256"),
        ("fifo", "fifo", "regular_file_sha256"),
        ("denied", "denied.bin", "regular_file_sha256"),
        ("over", "over.bin", "regular_file_sha256"),
        ("adir", "adir", "directory_metadata"),
    ];
    let body: Vec<String> = targets
        .iter()
        .map(|(id, path, observable)| {
            format!(
                "{{\"id\":\"{id}\",\"root\":\"prefix\",\"path\":\"{path}\",\
                 \"observable\":\"{observable}\"}}"
            )
        })
        .collect();
    format!(
        "{{\"schema\":\"helm-observation-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{subject_hex}\",\
         \"roots\":[{{\"id\":\"prefix\"}},{{\"id\":\"other\"}}],\"targets\":[{}]}}",
        body.join(",")
    )
    .into_bytes()
}

/// Produce a genuine observation over the synthetic tree.
fn observe(base: &Path, plan_bytes: &[u8]) -> (ValidatedPlan, ObservationArtifact) {
    let plan = helm_observe::parse_plan(plan_bytes).unwrap();
    let dir = |p: &Path| OwnedFd::from(fs::File::open(p).unwrap());
    let prefix = helm_observe::root_from_fd("prefix", dir(base)).unwrap();
    let other = helm_observe::root_from_fd("other", dir(base)).unwrap();
    let procfs =
        helm_observe::proc_fd_from_trusted_current_process(dir(Path::new("/proc/self/fd")))
            .unwrap();
    let scope = helm_observe::authorize(plan.clone(), vec![prefix, other], procfs).unwrap();
    let artifact = helm_observe::observe(&scope);
    (plan, artifact)
}

fn binding_plan(subject_hex: &str, extra: &str, claims: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{subject_hex}\"{extra},\"claims\":[{claims}]}}"
    )
    .into_bytes()
}

const FULL_MAPPING: &str = "{\"claim\":\"source_body\",\"target\":\"src\"},\
     {\"claim\":\"runtime_artifact_body\",\"role\":\"synthetic-runtime\",\"target\":\"rt\"},\
     {\"claim\":\"verification_definition_body\",\"role\":\"notes-contract\",\"target\":\"def\"},\
     {\"claim\":\"entry_point_presence\",\"target\":\"entry\"},\
     {\"claim\":\"entry_point_body\",\"target\":\"entry\"}";

fn state_of(report: &helm_bind::BindingReport, wanted: &str, role: Option<&str>) -> ClaimState {
    report
        .claims()
        .iter()
        .find(|c| c.subject().as_str() == wanted && c.subject().role() == role)
        .unwrap_or_else(|| panic!("claim {wanted} missing"))
        .state()
}

// --------------------------------------------------------------- happy path

#[test]
fn a_complete_correct_mapping_contradicts_nothing_and_still_covers_partially() {
    let base = temp_root("match");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        FULL_MAPPING,
    ))
    .unwrap();

    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();

    assert_eq!(state_of(&report, "source_body", None), ClaimState::Match);
    assert_eq!(
        state_of(&report, "runtime_artifact_body", Some("synthetic-runtime")),
        ClaimState::Match
    );
    assert_eq!(
        state_of(
            &report,
            "verification_definition_body",
            Some("notes-contract")
        ),
        ClaimState::Match
    );
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Match
    );
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::Match
    );
    for unsupported in [
        "source_architecture",
        "runtime_family",
        "windows_architecture",
        "prefix_role",
    ] {
        assert_eq!(
            state_of(&report, unsupported, None),
            ClaimState::UnsupportedBinding
        );
    }

    assert_eq!(report.contradiction(), Contradiction::NoClaimContradicted);
    let coverage = report.coverage();
    assert_eq!(coverage.claims, 9);
    assert_eq!(coverage.matched, 5);
    assert_eq!(coverage.unsupported_binding, 4);
    assert!(
        coverage.unsupported_binding >= 4,
        "coverage can never be complete in 0.1"
    );
    // Nothing anywhere says the application is satisfied, compatible or ready.
    let text = String::from_utf8_lossy(report.exact_bytes()).into_owned();
    assert!(!text.contains('/'), "no path in a report: {text}");
    assert!(!text.bytes().any(|b| b.is_ascii_uppercase()));
}

#[test]
fn an_unmapped_bindable_claim_is_not_observed_and_never_a_match() {
    let base = temp_root("unmapped");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"src\"}",
    ))
    .unwrap();

    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();
    assert_eq!(state_of(&report, "source_body", None), ClaimState::Match);
    assert_eq!(
        state_of(&report, "runtime_artifact_body", Some("synthetic-runtime")),
        ClaimState::NotObserved
    );
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::NotObserved
    );
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::NotObserved
    );
    assert_eq!(report.contradiction(), Contradiction::NoClaimContradicted);
    assert_eq!(report.coverage().not_observed, 4);
}

// ------------------------------------------------------------ typed mismatches

#[test]
fn body_mismatches_distinguish_size_from_digest() {
    let base = temp_root("mismatch");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));

    let both = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"wrong\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &both, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "source_body", None),
        ClaimState::Mismatch(Difference::SizeAndDigest)
    );
    assert_eq!(
        report.contradiction(),
        Contradiction::Contradicted { mismatches: 1 }
    );

    let digest_only = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"samelen\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &digest_only, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "source_body", None),
        ClaimState::Mismatch(Difference::DigestValue),
        "an equal length with a different digest is a digest mismatch"
    );

    // A runtime archive mapped where the source body is expected compares under
    // the source domain and disagrees; it is never reported as source identity.
    let crossed = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"rt\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &crossed, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "source_body", None),
        ClaimState::Mismatch(Difference::SizeAndDigest)
    );
}

/// The target ID `entry` carries no meaning: mapping the source body to it
/// compares the source body's desired values, and nothing else.
#[test]
fn a_target_id_spelling_carries_no_semantics() {
    let base = temp_root("noheuristic");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"entry\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "source_body", None),
        ClaimState::Mismatch(Difference::SizeAndDigest),
        "the entry-point body is not the source body, whatever the target is called"
    );
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::NotObserved
    );
}

// ------------------------------------------------------ entry-point path rules

#[test]
fn the_same_content_at_another_relative_path_never_binds_the_entry_point() {
    let base = temp_root("elsewhere");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"elsewhere\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"elsewhere\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Mismatch(Difference::EntryPointPath)
    );
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::Mismatch(Difference::EntryPointPath),
        "identical bytes at another path are not the desired entry point"
    );
    assert_eq!(
        report.contradiction(),
        Contradiction::Contradicted { mismatches: 2 }
    );
}

#[test]
fn a_correct_path_with_wrong_bytes_is_a_digest_mismatch_not_a_path_mismatch() {
    let base = temp_root("entrybytes");
    build_fixtures(&base);
    fs::write(base.join("drive_c/Notes/notes.exe"), b"tampered\n").unwrap();
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"entry\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Match,
        "presence needs no digest"
    );
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::Mismatch(Difference::DigestValue)
    );
}

/// The A0 fixture states no entry-point digest, so nothing is manufactured.
#[test]
fn an_unstated_entry_point_digest_is_never_synthesised() {
    let base = temp_root("a0");
    build_fixtures(&base);
    let subject = spec_digest_hex(A0_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let a0 = helm_app_spec::parse_spec(A0_SPEC).unwrap();
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"entry\"}",
    ))
    .unwrap();
    let report = bind(&a0, &mapping, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::DesiredValueUnspecified,
        "the specification states no digest, so there is nothing to compare"
    );
    // A0 declares four runtime artifacts, two disabled DLLs and three definitions:
    // 1 source body + 4 always-unsupported classes + 4 artifacts + 2 disabled DLLs
    // + 2 entry-point claims + 3 definitions.
    assert_eq!(report.coverage().claims, 1 + 4 + 4 + 2 + 2 + 3);
    assert_eq!(report.coverage().claims, 16);
    assert!(report.coverage().unsupported_binding >= 4);
    assert!(
        report
            .coverage()
            .unsupported_classes
            .contains(&"disabled_dll")
    );
    // The A0 entry-point path is not this synthetic tree's path.
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Mismatch(Difference::EntryPointPath)
    );
}

// ---------------------------------------------- observation outcomes stay apart

#[test]
fn absence_rejection_failure_and_omission_never_collapse() {
    let base = temp_root("outcomes");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));

    for (target, expected) in [
        ("missing", ClaimState::Absent),
        (
            "link",
            ClaimState::ObservationRejected(RejectionReason::SymlinkForbidden),
        ),
        (
            "fifo",
            ClaimState::ObservationRejected(RejectionReason::SpecialFile),
        ),
        (
            "denied",
            ClaimState::ObservationFailed(FailureReason::PermissionDenied),
        ),
        (
            "over",
            ClaimState::ObservationOmitted(OmissionReason::FileLimit),
        ),
    ] {
        let mapping = parse_binding_plan(&binding_plan(
            &subject,
            "",
            &format!("{{\"claim\":\"source_body\",\"target\":\"{target}\"}}"),
        ))
        .unwrap();
        let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();
        assert_eq!(
            state_of(&report, "source_body", None),
            expected,
            "target {target} must keep its own state"
        );
        assert_eq!(
            report.contradiction(),
            Contradiction::NoClaimContradicted,
            "target {target}: only a mismatch contradicts"
        );
    }
    fs::set_permissions(base.join("denied.bin"), fs::Permissions::from_mode(0o600)).unwrap();
}

// ------------------------------------------------------------------- refusals

#[test]
fn every_broken_document_combination_is_refused_without_a_report() {
    let base = temp_root("refusal");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let other_subject = spec_digest_hex(A0_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let spec = spec();

    // The mapping declares another specification.
    let wrong_mapping = parse_binding_plan(&binding_plan(
        &other_subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"src\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &wrong_mapping, &plan, &artifact)
            .unwrap_err()
            .code(),
        R::MappingSubjectSpecMismatch
    );

    // The observation declares another specification.
    let (foreign_plan, foreign_artifact) = observe(&base, &observation_plan_json(&other_subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"src\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &mapping, &foreign_plan, &foreign_artifact)
            .unwrap_err()
            .code(),
        R::ObservationSubjectSpecMismatch
    );

    // An artifact produced by one plan, paired with another plan.
    let mut alternative = observation_plan_json(&subject);
    alternative.push(b' ');
    let alternative = helm_observe::parse_plan(&alternative).unwrap();
    assert_eq!(
        bind(&spec, &mapping, &alternative, &artifact)
            .unwrap_err()
            .code(),
        R::ObservationPlanMismatch
    );

    // A mapping naming a target the observation plan does not declare.
    let unknown_target = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"nosuch\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &unknown_target, &plan, &artifact)
            .unwrap_err()
            .code(),
        R::UnknownTarget
    );

    // An asserted prefix root the observation plan does not declare.
    let unknown_root = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"nosuch\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &unknown_root, &plan, &artifact)
            .unwrap_err()
            .code(),
        R::UnknownRoot
    );

    // Roles the specification does not declare.
    for claim in [
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"nosuch\",\"target\":\"rt\"}",
        "{\"claim\":\"verification_definition_body\",\"role\":\"nosuch\",\"target\":\"def\"}",
    ] {
        let unknown_role = parse_binding_plan(&binding_plan(&subject, "", claim)).unwrap();
        assert_eq!(
            bind(&spec, &unknown_role, &plan, &artifact)
                .unwrap_err()
                .code(),
            R::UnknownRole
        );
    }

    // A directory-metadata target cannot answer any 0.1 comparator.
    let incompatible = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"adir\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &incompatible, &plan, &artifact)
            .unwrap_err()
            .code(),
        R::IncompatibleObservable
    );

    // An entry-point target under a root other than the asserted prefix.
    let wrong_root_plan = String::from_utf8(observation_plan_json(&subject))
        .unwrap()
        .replace(
            "{\"id\":\"entry\",\"root\":\"prefix\"",
            "{\"id\":\"entry\",\"root\":\"other\"",
        )
        .into_bytes();
    let (other_root_plan, other_root_artifact) = observe(&base, &wrong_root_plan);
    let entry_mapping = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(
            &spec,
            &entry_mapping,
            &other_root_plan,
            &other_root_artifact
        )
        .unwrap_err()
        .code(),
        R::EntryPointRootMismatch
    );
}

// ------------------------------------------------------------------- identity

#[test]
fn the_report_binds_exactly_the_four_input_identities() {
    let base = temp_root("identity");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping_bytes = binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        FULL_MAPPING,
    );
    let mapping = parse_binding_plan(&mapping_bytes).unwrap();
    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();

    let inputs = report.inputs();
    assert_eq!(inputs.spec_sha256.to_hex(), subject);
    assert_eq!(
        inputs.observation_plan_sha256.to_hex(),
        plan.sha256().to_hex()
    );
    assert_eq!(
        inputs.binding_plan_sha256.to_hex(),
        mapping.sha256().to_hex()
    );
    assert_eq!(
        inputs.observation_artifact_sha256.to_hex(),
        artifact.sha256().to_hex()
    );

    // Each digest is over the exact bytes of its own document, computed here
    // independently of the crates under test.
    use sha2::Digest as _;
    let independent = |raw: &[u8]| -> String {
        let d: [u8; 32] = sha2::Sha256::digest(raw).into();
        d.iter().map(|b| format!("{b:02x}")).collect()
    };
    assert_eq!(
        inputs.binding_plan_sha256.to_hex(),
        independent(&mapping_bytes)
    );
    assert_eq!(
        inputs.observation_plan_sha256.to_hex(),
        independent(plan.exact_bytes())
    );
    assert_eq!(
        inputs.observation_artifact_sha256.to_hex(),
        independent(artifact.exact_bytes())
    );
    assert_eq!(
        report.sha256().to_hex(),
        independent(report.exact_bytes()),
        "the report digest is over exactly its own bytes"
    );

    // A different mapping over the same three other documents is a different report.
    let other_mapping_bytes = binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"src\"}",
    );
    let other_mapping = parse_binding_plan(&other_mapping_bytes).unwrap();
    let other = bind(&spec(), &other_mapping, &plan, &artifact).unwrap();
    assert_ne!(
        report.sha256(),
        other.sha256(),
        "the mapping is part of the report identity"
    );

    // The same four inputs always give the same bytes.
    let again = bind(&spec(), &mapping, &plan, &artifact).unwrap();
    assert_eq!(again.exact_bytes(), report.exact_bytes());
    assert_eq!(again.sha256(), report.sha256());
}

/// Claim outcomes follow specification declaration order, and the subject list
/// never contains contextual metadata.
#[test]
fn claim_order_follows_the_specification() {
    let base = temp_root("order");
    build_fixtures(&base);
    let subject = spec_digest_hex(NOTES_SPEC);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"src\"}",
    ))
    .unwrap();
    let report = bind(&spec(), &mapping, &plan, &artifact).unwrap();
    let order: Vec<&str> = report
        .claims()
        .iter()
        .map(|c| c.subject().as_str())
        .collect();
    assert_eq!(
        order,
        [
            "source_body",
            "source_architecture",
            "runtime_family",
            "runtime_artifact_body",
            "windows_architecture",
            "prefix_role",
            "entry_point_presence",
            "entry_point_body",
            "verification_definition_body",
        ]
    );
    assert!(
        !report
            .claims()
            .iter()
            .any(|c| matches!(c.subject(), ClaimSubject::DisabledDll(_))),
        "the synthetic specification declares no disabled DLLs"
    );
}
