//! Independent adversarial review of the binding semantics, against **genuine**
//! observations.
//!
//! Written by the independent reviewer of candidate
//! `832a112decaa333ee3b618d52e966a20c443513e`. `ObservationArtifact` has no public
//! constructor, so binding can only be exercised where `helm-observe`'s backend
//! exists; that gate is a property of the input type, not of the comparison logic.
//! Synthetic fixtures only: no A0 access, no Wine, no 7-Zip, no lab.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};

use helm_bind::{
    BindingRefusalCode as R, ClaimState, ClaimSubject, Contradiction, Difference, bind,
    parse_binding_plan,
};
use helm_observe::{ObservationArtifact, ValidatedPlan};

const BODY_A: &[u8] = b"independent review body A\n";
const BODY_B: &[u8] = b"independent review body BB\n";

fn sha256_hex(raw: &[u8]) -> String {
    use sha2::Digest as _;
    let d: [u8; 32] = sha2::Sha256::digest(raw).into();
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn temp_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("helm-bind-ir-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn build_tree(base: &Path) {
    fs::write(base.join("a.bin"), BODY_A).unwrap();
    fs::write(base.join("b.bin"), BODY_B).unwrap();
    fs::create_dir_all(base.join("drive_c/app")).unwrap();
    fs::write(base.join("drive_c/app/main.exe"), BODY_A).unwrap();
    fs::create_dir_all(base.join("other")).unwrap();
    fs::write(base.join("other/main.exe"), BODY_A).unwrap();
    fs::create_dir_all(base.join("adir")).unwrap();
}

/// A valid `helm-app-spec` document, generated across the current schema
/// boundaries. Digests are arbitrary but well formed; the universe properties do
/// not depend on whether they agree with anything observed.
fn generated_spec(
    artifacts: usize,
    definitions: usize,
    dlls: usize,
    entry_digest: bool,
    entry_path: &str,
) -> Vec<u8> {
    let digest = |n: usize| format!("{:064x}", n as u128 * 0x1234_5678_9abc_def0);
    let arts: Vec<String> = (0..artifacts)
        .map(|i| {
            format!(
                "{{\"role\":\"art{i}\",\"size\":{},\"sha256\":\"{}\"}}",
                i + 1,
                digest(i + 1)
            )
        })
        .collect();
    let defs: Vec<String> = (0..definitions)
        .map(|i| {
            format!(
                "{{\"role\":\"def{i}\",\"size\":{},\"sha256\":\"{}\"}}",
                i + 100,
                digest(i + 100)
            )
        })
        .collect();
    let dll_list: Vec<String> = (0..dlls).map(|i| format!("\"dll{i}\"")).collect();
    let entry = if entry_digest {
        format!(
            "{{\"path\":\"{entry_path}\",\"sha256\":\"{}\"}}",
            sha256_hex(BODY_A)
        )
    } else {
        format!("{{\"path\":\"{entry_path}\"}}")
    };
    format!(
        "{{\"schema\":\"helm-app-spec\",\"version\":\"0.1\",\
         \"application\":{{\"id\":\"gen-app\",\"version\":\"v{artifacts}.{definitions}\",\
         \"source\":{{\"size\":{},\"sha256\":\"{}\",\"architecture\":\"x86_64\"}}}},\
         \"runtime\":{{\"family\":\"wine\",\"artifacts\":[{}]}},\
         \"environment\":{{\"windows_architecture\":\"win64\",\
         \"prefix\":{{\"role\":\"dedicated\"}},\"disabled_dlls\":[{}]}},\
         \"entry_point\":{entry},\
         \"verification\":{{\"definitions\":[{}]}}}}",
        BODY_A.len(),
        sha256_hex(BODY_A),
        arts.join(","),
        dll_list.join(","),
        defs.join(",")
    )
    .into_bytes()
}

fn observation_plan_json(subject_hex: &str, entry_root: &str) -> Vec<u8> {
    let fixed = [
        ("a", "a.bin", "regular_file_sha256", "prefix"),
        ("b", "b.bin", "regular_file_sha256", "prefix"),
        ("missing", "nope.bin", "regular_file_sha256", "prefix"),
        ("adir", "adir", "directory_metadata", "prefix"),
        (
            "elsewhere",
            "other/main.exe",
            "regular_file_sha256",
            "prefix",
        ),
    ];
    let mut body: Vec<String> = fixed
        .iter()
        .map(|(id, path, observable, root)| {
            format!(
                "{{\"id\":\"{id}\",\"root\":\"{root}\",\"path\":\"{path}\",\
                 \"observable\":\"{observable}\"}}"
            )
        })
        .collect();
    body.push(format!(
        "{{\"id\":\"entry\",\"root\":\"{entry_root}\",\"path\":\"drive_c/app/main.exe\",\
         \"observable\":\"regular_file_sha256\"}}"
    ));
    format!(
        "{{\"schema\":\"helm-observation-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{subject_hex}\",\
         \"roots\":[{{\"id\":\"prefix\"}},{{\"id\":\"other\"}}],\"targets\":[{}]}}",
        body.join(",")
    )
    .into_bytes()
}

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

fn state_of(report: &helm_bind::BindingReport, class: &str, role: Option<&str>) -> ClaimState {
    report
        .claims()
        .iter()
        .find(|c| c.subject().as_str() == class && c.subject().role() == role)
        .unwrap_or_else(|| panic!("claim {class} {role:?} missing"))
        .state()
}

// ------------------------------------------- universal claim-universe theorem

/// The author suite checks the coverage theorem against two committed fixtures.
/// This checks it against generated specifications spanning the schema
/// boundaries, so the property is established for the schema and not for two
/// documents.
#[test]
fn every_valid_specification_has_at_least_four_unsupported_claims() {
    let base = temp_root("universe");
    build_tree(&base);
    let mut checked = 0usize;
    for &artifacts in &[1usize, 2, 16] {
        for &definitions in &[1usize, 3, 8] {
            for &dlls in &[0usize, 1, 8] {
                for &entry_digest in &[false, true] {
                    let raw = generated_spec(
                        artifacts,
                        definitions,
                        dlls,
                        entry_digest,
                        "drive_c/app/main.exe",
                    );
                    let spec = helm_app_spec::parse_spec(&raw)
                        .unwrap_or_else(|e| panic!("generated spec must be valid: {e}"));
                    let subject = sha256_hex(&raw);
                    let (plan, artifact) =
                        observe(&base, &observation_plan_json(&subject, "prefix"));
                    let mapping = parse_binding_plan(&binding_plan(
                        &subject,
                        "",
                        "{\"claim\":\"source_body\",\"target\":\"a\"}",
                    ))
                    .unwrap();
                    let report = bind(&spec, &mapping, &plan, &artifact).unwrap();

                    // The four mandatory unsupported classes are always present.
                    for class in [
                        "source_architecture",
                        "runtime_family",
                        "windows_architecture",
                        "prefix_role",
                    ] {
                        assert_eq!(
                            state_of(&report, class, None),
                            ClaimState::UnsupportedBinding,
                            "{class} must be unsupported for every specification"
                        );
                    }
                    let coverage = report.coverage();
                    assert!(
                        coverage.unsupported_binding >= 4,
                        "artifacts={artifacts} definitions={definitions} dlls={dlls}: \
                         unsupported was {}",
                        coverage.unsupported_binding
                    );
                    // Unsupported count is exactly the four classes plus the DLLs.
                    assert_eq!(
                        coverage.unsupported_binding,
                        4 + u32::try_from(dlls).unwrap()
                    );

                    // The claim-instance formula, independently recomputed here.
                    let expected = 1 + 4 + artifacts + dlls + 2 + definitions;
                    assert_eq!(
                        coverage.claims,
                        u32::try_from(expected).unwrap(),
                        "artifacts={artifacts} definitions={definitions} dlls={dlls}"
                    );
                    assert!(coverage.claims <= 39, "the universe ceiling is 39");

                    // Counters partition the claim set exactly.
                    let sum = coverage.matched
                        + coverage.mismatched
                        + coverage.desired_value_unspecified
                        + coverage.not_observed
                        + coverage.unsupported_binding
                        + coverage.absent
                        + coverage.observation_rejected
                        + coverage.observation_failed
                        + coverage.observation_omitted
                        + coverage.observation_not_interpretable;
                    assert_eq!(sum, coverage.claims);

                    // No outcome for contextual metadata or a selector key.
                    for class in report.claims().iter().map(|c| c.subject().as_str()) {
                        assert!(
                            !matches!(class, "application" | "spec_sha256" | "label" | "role"),
                            "contextual metadata must not receive an outcome"
                        );
                    }
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 3 * 3 * 3 * 2, "every generated combination ran");

    // The maximum-size specification reaches exactly 39 claim instances.
    let raw = generated_spec(16, 8, 8, true, "drive_c/app/main.exe");
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"a\"}",
    ))
    .unwrap();
    let report = bind(&spec, &mapping, &plan, &artifact).unwrap();
    assert_eq!(report.coverage().claims, 39);
    assert!(report.exact_bytes().len() < helm_bind::MAX_REPORT_BYTES);
}

// -------------------------------------------------- selector-vocabulary attack

/// The serializer writes validated role and DLL selectors straight into the
/// report. Those selectors are caller data from a valid specification, and the
/// `helm-app-spec` identifier grammar permits words such as `ready` and
/// `verified`. This establishes what actually reaches the exact bytes.
#[test]
fn caller_selectors_reach_the_report_bytes_verbatim() {
    let base = temp_root("selector");
    build_tree(&base);
    let loaded = [
        "ready",
        "pass",
        "fail",
        "verified",
        "complete",
        "snapshot",
        "compatible",
        "satisfied",
        "installed",
    ];
    let arts: Vec<String> = loaded
        .iter()
        .enumerate()
        .map(|(i, role)| {
            format!(
                "{{\"role\":\"{role}\",\"size\":{},\"sha256\":\"{:064x}\"}}",
                i + 1,
                i + 1
            )
        })
        .collect();
    let raw = format!(
        "{{\"schema\":\"helm-app-spec\",\"version\":\"0.1\",\
         \"application\":{{\"id\":\"ready\",\"version\":\"ready\",\
         \"source\":{{\"size\":{},\"sha256\":\"{}\",\"architecture\":\"x86_64\"}}}},\
         \"runtime\":{{\"family\":\"wine\",\"artifacts\":[{}]}},\
         \"environment\":{{\"windows_architecture\":\"win64\",\
         \"prefix\":{{\"role\":\"dedicated\"}},\
         \"disabled_dlls\":[\"verified\",\"complete\",\"snapshot\"]}},\
         \"entry_point\":{{\"path\":\"drive_c/app/main.exe\"}},\
         \"verification\":{{\"definitions\":[{{\"role\":\"satisfied\",\"size\":9,\
         \"sha256\":\"{:064x}\"}}]}}}}",
        BODY_A.len(),
        sha256_hex(BODY_A),
        arts.join(","),
        7
    )
    .into_bytes();
    let spec = helm_app_spec::parse_spec(&raw)
        .unwrap_or_else(|e| panic!("these selectors are legal helm-app-spec identifiers: {e}"));

    let subject = sha256_hex(&raw);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"ready\",\"target\":\"a\"}",
    ))
    .unwrap();
    let report = bind(&spec, &mapping, &plan, &artifact).unwrap();
    let text = String::from_utf8(report.exact_bytes().to_vec()).unwrap();

    // Selector data does reach the bytes, verbatim and unescaped.
    assert!(text.contains("\"role\":\"ready\""));
    assert!(text.contains("\"role\":\"verified\""));
    assert!(text.contains("\"role\":\"satisfied\""));

    // What must remain true regardless: every such word appears only as selector
    // data, never as a claim class, a state, a contradiction value or a coverage
    // key. That is the property the accepted architecture actually requires.
    let is_selector_slot = |needle: &str| {
        text.match_indices(needle).all(|(at, _)| {
            let before = &text[..at];
            before.ends_with("\"role\":\"")
                || before.ends_with("\"id\":\"")
                || before.ends_with("_body\",\"role\":\"")
        })
    };
    for word in loaded {
        if text.contains(word) {
            assert!(
                is_selector_slot(word),
                "{word} appears outside a selector slot: {text}"
            );
        }
    }
    // The binder's own vocabulary never contains a verdict word.
    for state_word in report.claims().iter().map(|c| c.state().as_str()) {
        assert!(
            !loaded.contains(&state_word),
            "a claim state must not be a verdict word: {state_word}"
        );
    }
    assert!(
        !text.contains("\"state\":\"ready\"") && !text.contains("\"state\":\"pass\""),
        "no state may be a verdict word"
    );
    assert!(!text.contains('/'), "no path may appear");
    assert!(!text.bytes().any(|b| b.is_ascii_uppercase()));
}

// ------------------------------------------------------ refusal atomicity

/// Every refusal must be atomic: an error, and no report of any kind.
#[test]
fn refusals_are_atomic_and_deterministically_ordered() {
    let base = temp_root("refusal");
    build_tree(&base);
    let raw = generated_spec(2, 1, 0, true, "drive_c/app/main.exe");
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let other_raw = generated_spec(1, 1, 0, false, "drive_c/app/main.exe");
    let other_subject = sha256_hex(&other_raw);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));

    let cases: Vec<(&str, Vec<u8>, bool)> = vec![
        (
            "mapping subject",
            binding_plan(
                &other_subject,
                "",
                "{\"claim\":\"source_body\",\"target\":\"a\"}",
            ),
            false,
        ),
        (
            "unknown target",
            binding_plan(
                &subject,
                "",
                "{\"claim\":\"source_body\",\"target\":\"nosuch\"}",
            ),
            false,
        ),
        (
            "unknown role",
            binding_plan(
                &subject,
                "",
                "{\"claim\":\"runtime_artifact_body\",\"role\":\"nosuch\",\"target\":\"a\"}",
            ),
            false,
        ),
        (
            "incompatible observable",
            binding_plan(
                &subject,
                "",
                "{\"claim\":\"source_body\",\"target\":\"adir\"}",
            ),
            false,
        ),
        (
            "unknown root",
            binding_plan(
                &subject,
                ",\"asserted_prefix_root_id\":\"nosuch\"",
                "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"}",
            ),
            false,
        ),
        (
            "valid control",
            binding_plan(&subject, "", "{\"claim\":\"source_body\",\"target\":\"a\"}"),
            true,
        ),
    ];
    for (label, bytes, should_bind) in cases {
        let mapping = parse_binding_plan(&bytes).unwrap();
        let result = bind(&spec, &mapping, &plan, &artifact);
        assert_eq!(result.is_ok(), should_bind, "{label}");
        if let Err(refusal) = &result {
            // A refusal carries no path and no untrusted text beyond a validated ID.
            let rendered = refusal.to_string();
            assert!(!rendered.contains('/'), "{label}: {rendered}");
            assert!(refusal.code().as_str().is_ascii());
        }
    }

    // Precedence when several conditions are wrong at once: the subject check
    // wins over every per-claim check, and the claim checks follow mapping order.
    let many = parse_binding_plan(&binding_plan(
        &other_subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"nosuch\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &many, &plan, &artifact).unwrap_err().code(),
        R::MappingSubjectSpecMismatch,
        "the subject check precedes the per-claim checks"
    );
    let target_before_role = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"nosuch\",\"target\":\"nosuch\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &target_before_role, &plan, &artifact)
            .unwrap_err()
            .code(),
        R::UnknownTarget,
        "within one claim the target check precedes the role check"
    );
    let first_claim_wins = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"adir\"},\
         {\"claim\":\"runtime_artifact_body\",\"role\":\"nosuch\",\"target\":\"a\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &first_claim_wins, &plan, &artifact)
            .unwrap_err()
            .code(),
        R::IncompatibleObservable,
        "claims are checked in mapping declaration order"
    );
}

// ------------------------------------------------------------ typed domains

/// A body that agrees in one domain must never be reported under another.
#[test]
fn a_matching_body_never_crosses_its_semantic_domain() {
    let base = temp_root("domain");
    build_tree(&base);
    // The source and the first runtime artifact declare the *same* body identity,
    // so only the claim kind can keep them apart.
    let raw = format!(
        "{{\"schema\":\"helm-app-spec\",\"version\":\"0.1\",\
         \"application\":{{\"id\":\"cross\",\"version\":\"1\",\
         \"source\":{{\"size\":{},\"sha256\":\"{}\",\"architecture\":\"x86_64\"}}}},\
         \"runtime\":{{\"family\":\"wine\",\"artifacts\":[{{\"role\":\"same\",\"size\":{},\
         \"sha256\":\"{}\"}}]}},\
         \"environment\":{{\"windows_architecture\":\"win64\",\
         \"prefix\":{{\"role\":\"dedicated\"}},\"disabled_dlls\":[]}},\
         \"entry_point\":{{\"path\":\"drive_c/app/main.exe\"}},\
         \"verification\":{{\"definitions\":[{{\"role\":\"same\",\"size\":{},\
         \"sha256\":\"{}\"}}]}}}}",
        BODY_A.len(),
        sha256_hex(BODY_A),
        BODY_A.len(),
        sha256_hex(BODY_A),
        BODY_A.len(),
        sha256_hex(BODY_A)
    )
    .into_bytes();
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));

    // Map only the runtime artifact. The source and definition claims must stay
    // NotObserved even though the very same observed body would satisfy them.
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"same\",\"target\":\"a\"}",
    ))
    .unwrap();
    let report = bind(&spec, &mapping, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "runtime_artifact_body", Some("same")),
        ClaimState::Match
    );
    assert_eq!(
        state_of(&report, "source_body", None),
        ClaimState::NotObserved,
        "an agreeing runtime artifact must not answer the source claim"
    );
    assert_eq!(
        state_of(&report, "verification_definition_body", Some("same")),
        ClaimState::NotObserved,
        "an agreeing runtime artifact must not answer a definition claim"
    );
    // A role shared between the runtime and verification namespaces stays apart.
    let both = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"same\",\"target\":\"a\"},\
         {\"claim\":\"verification_definition_body\",\"role\":\"same\",\"target\":\"b\"}",
    ))
    .unwrap();
    let report = bind(&spec, &both, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "runtime_artifact_body", Some("same")),
        ClaimState::Match
    );
    assert_eq!(
        state_of(&report, "verification_definition_body", Some("same")),
        ClaimState::Mismatch(Difference::SizeAndDigest),
        "the two namespaces are distinguished by claim kind, not by role"
    );
}

// ------------------------------------------------------------- body matrix

#[test]
fn body_comparison_reports_size_and_digest_independently() {
    let base = temp_root("body");
    build_tree(&base);
    // Four specifications, one per cell of the size/digest truth table.
    let cases: [(u64, String, Option<Difference>); 4] = [
        (BODY_A.len() as u64, sha256_hex(BODY_A), None),
        (
            BODY_A.len() as u64 + 1,
            sha256_hex(BODY_A),
            Some(Difference::Size),
        ),
        (
            BODY_A.len() as u64,
            sha256_hex(BODY_B),
            Some(Difference::DigestValue),
        ),
        (
            BODY_A.len() as u64 + 1,
            sha256_hex(BODY_B),
            Some(Difference::SizeAndDigest),
        ),
    ];
    for (size, digest, expected) in cases {
        let raw = format!(
            "{{\"schema\":\"helm-app-spec\",\"version\":\"0.1\",\
             \"application\":{{\"id\":\"body\",\"version\":\"1\",\
             \"source\":{{\"size\":{size},\"sha256\":\"{digest}\",\
             \"architecture\":\"x86_64\"}}}},\
             \"runtime\":{{\"family\":\"wine\",\"artifacts\":[{{\"role\":\"r\",\"size\":1,\
             \"sha256\":\"{:064x}\"}}]}},\
             \"environment\":{{\"windows_architecture\":\"win64\",\
             \"prefix\":{{\"role\":\"dedicated\"}},\"disabled_dlls\":[]}},\
             \"entry_point\":{{\"path\":\"drive_c/app/main.exe\"}},\
             \"verification\":{{\"definitions\":[{{\"role\":\"d\",\"size\":1,\
             \"sha256\":\"{:064x}\"}}]}}}}",
            1, 2
        )
        .into_bytes();
        let spec = helm_app_spec::parse_spec(&raw).unwrap();
        let subject = sha256_hex(&raw);
        let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));
        let mapping = parse_binding_plan(&binding_plan(
            &subject,
            "",
            "{\"claim\":\"source_body\",\"target\":\"a\"}",
        ))
        .unwrap();
        let report = bind(&spec, &mapping, &plan, &artifact).unwrap();
        let want = expected.map_or(ClaimState::Match, ClaimState::Mismatch);
        assert_eq!(
            state_of(&report, "source_body", None),
            want,
            "size={size} digest={digest}"
        );
        let contradicted = expected.is_some();
        assert_eq!(
            report.contradiction() != Contradiction::NoClaimContradicted,
            contradicted
        );
    }
}

// ------------------------------------------------- entry point root and path

#[test]
fn entry_point_root_and_path_rules_are_exact() {
    let base = temp_root("entry");
    build_tree(&base);
    let raw = generated_spec(1, 1, 0, true, "drive_c/app/main.exe");
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let entry_claims = "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"entry\"}";

    // Correct root and path.
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));
    let mapping = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        entry_claims,
    ))
    .unwrap();
    let report = bind(&spec, &mapping, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Match
    );
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::Match
    );

    // Same bytes at another relative path.
    let elsewhere = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"elsewhere\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"elsewhere\"}",
    ))
    .unwrap();
    let report = bind(&spec, &elsewhere, &plan, &artifact).unwrap();
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Mismatch(Difference::EntryPointPath)
    );
    assert_eq!(
        state_of(&report, "entry_point_body", None),
        ClaimState::Mismatch(Difference::EntryPointPath),
        "identical bytes at another path never bind the entry point"
    );

    // A case-only path difference is a mismatch: no folding.
    let cased = generated_spec(1, 1, 0, true, "drive_c/app/Main.exe");
    let cased_spec = helm_app_spec::parse_spec(&cased).unwrap();
    let cased_subject = sha256_hex(&cased);
    let (cased_plan, cased_artifact) =
        observe(&base, &observation_plan_json(&cased_subject, "prefix"));
    let cased_mapping = parse_binding_plan(&binding_plan(
        &cased_subject,
        ",\"asserted_prefix_root_id\":\"prefix\"",
        entry_claims,
    ))
    .unwrap();
    let report = bind(&cased_spec, &cased_mapping, &cased_plan, &cased_artifact).unwrap();
    assert_eq!(
        state_of(&report, "entry_point_presence", None),
        ClaimState::Mismatch(Difference::EntryPointPath),
        "a case-only difference must not be folded away"
    );

    // The entry target under a different root is a refusal, not a claim result.
    let (other_plan, other_artifact) = observe(&base, &observation_plan_json(&subject, "other"));
    assert_eq!(
        bind(&spec, &mapping, &other_plan, &other_artifact)
            .unwrap_err()
            .code(),
        R::EntryPointRootMismatch
    );
}

// -------------------------------------- optional entry-point digest precedence

/// A specification that states no entry-point digest can never contradict on the
/// body claim, whatever the mapping or the observation says, and a structurally
/// broken mapping is still refused globally.
#[test]
fn an_unstated_entry_digest_never_contradicts_but_never_hides_a_refusal() {
    let base = temp_root("optional");
    build_tree(&base);
    let raw = generated_spec(1, 1, 0, false, "drive_c/app/main.exe");
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));

    for (label, claims) in [
        ("unmapped", "{\"claim\":\"source_body\",\"target\":\"a\"}"),
        (
            "mapped correctly",
            "{\"claim\":\"entry_point_body\",\"target\":\"entry\"}",
        ),
        (
            "mapped to the wrong path",
            "{\"claim\":\"entry_point_body\",\"target\":\"elsewhere\"}",
        ),
        (
            "mapped to an absent target",
            "{\"claim\":\"entry_point_body\",\"target\":\"missing\"}",
        ),
    ] {
        let extra = if claims.contains("entry_point") {
            ",\"asserted_prefix_root_id\":\"prefix\""
        } else {
            ""
        };
        let mapping = parse_binding_plan(&binding_plan(&subject, extra, claims)).unwrap();
        let report = bind(&spec, &mapping, &plan, &artifact).unwrap();
        assert_eq!(
            state_of(&report, "entry_point_body", None),
            ClaimState::DesiredValueUnspecified,
            "{label}: an unstated desired digest is never compared"
        );
        assert_eq!(
            report.contradiction(),
            Contradiction::NoClaimContradicted,
            "{label}: an unstated claim must never contradict"
        );
    }

    // A structurally broken mapping is still refused, unspecified digest or not.
    let broken = parse_binding_plan(&binding_plan(
        &subject,
        ",\"asserted_prefix_root_id\":\"nosuch\"",
        "{\"claim\":\"entry_point_body\",\"target\":\"entry\"}",
    ))
    .unwrap();
    assert_eq!(
        bind(&spec, &broken, &plan, &artifact).unwrap_err().code(),
        R::UnknownRoot,
        "an unspecified desired value must not suppress a cross-document refusal"
    );
}

// ------------------------------------------------ observation state distinctness

#[test]
fn every_observation_outcome_keeps_its_own_state() {
    let base = temp_root("states");
    build_tree(&base);
    let raw = generated_spec(1, 1, 0, true, "drive_c/app/main.exe");
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let (plan, artifact) = observe(&base, &observation_plan_json(&subject, "prefix"));

    let absent = parse_binding_plan(&binding_plan(
        &subject,
        "",
        "{\"claim\":\"source_body\",\"target\":\"missing\"}",
    ))
    .unwrap();
    let report = bind(&spec, &absent, &plan, &artifact).unwrap();
    let absent_state = state_of(&report, "source_body", None);
    assert_eq!(absent_state, ClaimState::Absent);
    assert!(!absent_state.is_mismatch(), "absence is not a mismatch");
    assert_eq!(report.contradiction(), Contradiction::NoClaimContradicted);
    assert_eq!(report.coverage().absent, 1);
    assert_eq!(report.coverage().mismatched, 0);
    assert_eq!(report.coverage().not_observed, 3);

    // An unmapped claim and an absent target are different states with their own
    // counters, and neither is the other.
    assert_ne!(absent_state, ClaimState::NotObserved);
    assert_ne!(absent_state, ClaimState::UnsupportedBinding);
}

// ---------------------------------------------------------------- identity

#[test]
fn report_identity_is_exact_and_moves_with_every_input() {
    let base = temp_root("identity");
    build_tree(&base);
    let raw = generated_spec(1, 1, 0, true, "drive_c/app/main.exe");
    let spec = helm_app_spec::parse_spec(&raw).unwrap();
    let subject = sha256_hex(&raw);
    let plan_bytes = observation_plan_json(&subject, "prefix");
    let (plan, artifact) = observe(&base, &plan_bytes);
    let mapping_bytes = binding_plan(&subject, "", "{\"claim\":\"source_body\",\"target\":\"a\"}");
    let mapping = parse_binding_plan(&mapping_bytes).unwrap();
    let report = bind(&spec, &mapping, &plan, &artifact).unwrap();

    // Each recorded identity equals an independently computed digest of the exact
    // bytes of its own document.
    let inputs = report.inputs();
    assert_eq!(inputs.spec_sha256.to_hex(), sha256_hex(&raw));
    assert_eq!(
        inputs.observation_plan_sha256.to_hex(),
        sha256_hex(plan.exact_bytes())
    );
    assert_eq!(
        inputs.binding_plan_sha256.to_hex(),
        sha256_hex(&mapping_bytes)
    );
    assert_eq!(
        inputs.observation_artifact_sha256.to_hex(),
        sha256_hex(artifact.exact_bytes())
    );
    assert_eq!(report.sha256().to_hex(), sha256_hex(report.exact_bytes()));

    // Changing only the mapping's bytes, with the same semantic result, moves the
    // report identity.
    let mut spaced = mapping_bytes.clone();
    let at = spaced.iter().position(|b| *b == b',').unwrap();
    spaced.insert(at + 1, b' ');
    let spaced_mapping = parse_binding_plan(&spaced).unwrap();
    let spaced_report = bind(&spec, &spaced_mapping, &plan, &artifact).unwrap();
    assert_eq!(
        spaced_report.claims(),
        report.claims(),
        "the semantic result is unchanged"
    );
    assert_ne!(
        spaced_report.sha256(),
        report.sha256(),
        "but the mapping identity is part of the report identity"
    );

    // Changing only the observation artifact's bytes, with the same semantic
    // result, also moves the report identity.
    let mut other_plan_bytes = plan_bytes.clone();
    let at = other_plan_bytes.iter().position(|b| *b == b',').unwrap();
    other_plan_bytes.insert(at + 1, b' ');
    let (other_plan, other_artifact) = observe(&base, &other_plan_bytes);
    let other_report = bind(&spec, &mapping, &other_plan, &other_artifact).unwrap();
    assert_eq!(other_report.claims(), report.claims());
    assert_ne!(
        other_report.sha256(),
        report.sha256(),
        "the observation identity is part of the report identity"
    );
    assert_ne!(
        other_report.inputs().observation_artifact_sha256,
        inputs.observation_artifact_sha256
    );

    // Determinism.
    assert_eq!(
        bind(&spec, &mapping, &plan, &artifact)
            .unwrap()
            .exact_bytes(),
        report.exact_bytes()
    );
}
