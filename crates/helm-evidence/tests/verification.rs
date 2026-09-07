mod support;
use helm_evidence::{
    model::{CheckStatus, ExperimentStatus, Verdict},
    verify,
};
use serde_json::json;
use std::{fs, process::Command};
use support::*;

fn check(root: &std::path::Path, verdict: Verdict, code: &str, status: CheckStatus) {
    let report = verify(root);
    assert_eq!(report.verdict, verdict, "{report:?}");
    assert!(
        report
            .checks
            .iter()
            .any(|c| c.code == code && c.status == status),
        "{report:?}"
    );
}

#[test]
fn fully_complete_bundle() {
    check(
        &fixture_path("synthetic"),
        Verdict::Complete,
        "OUTPUT_DESTINATION",
        CheckStatus::Satisfied,
    );
}

#[test]
fn missing_required_artifact() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::remove_file(f.root.join("outputs/before/output.dat"))?;
    check(
        &f.root,
        Verdict::Incomplete,
        "ARTIFACT_MISSING",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn wrong_artifact_hash() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::write(f.root.join("outputs/before/output.dat"), b"changed")?;
    check(
        &f.root,
        Verdict::Invalid,
        "ARTIFACT_HASH",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn content_pass_cannot_override_wrong_destination() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("before.json", |r| {
        r["output"]["path"] = "wrong/output.dat".into()
    })?;
    let r = verify(&f.root);
    assert_eq!(r.verdict, Verdict::Incomplete);
    for (code, status) in [
        ("CONTENT_RESULT_PASS", CheckStatus::Satisfied),
        ("OUTPUT_DESTINATION", CheckStatus::Incomplete),
    ] {
        assert!(
            r.checks.iter().any(|c| c.workflow.as_deref() == Some("W1")
                && c.code == code
                && c.status == status)
        );
    }
    Ok(())
}

#[test]
fn missing_output_observation_does_not_infer_from_content() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("before.json", |r| r["output"] = serde_json::Value::Null)?;
    check(
        &f.root,
        Verdict::Incomplete,
        "OUTPUT_DESTINATION",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn missing_known_good_control() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::remove_file(f.root.join("good.json"))?;
    check(
        &f.root,
        Verdict::Incomplete,
        "KNOWN_GOOD_CONTROL",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn known_good_rejected_is_invalid() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("good.json", |r| {
        r["verdict"] = "FAIL".into();
        r["errors"] = json!(["synthetic failure"]);
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "KNOWN_GOOD_CONTROL",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn broken_control_incorrectly_accepted() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("bad.json", |r| {
        r["verdict"] = "PASS".into();
        r["errors"] = json!([]);
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "BROKEN_CONTROL",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn broken_control_must_evidence_content_defect() -> TestResult {
    let f = Fixture::synthetic()?;
    let good = f.json("good.json")?;
    f.edit("bad.json", |r| r["files"] = good["files"].clone())?;
    check(
        &f.root,
        Verdict::Invalid,
        "BROKEN_CONTROL",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn missing_broken_control() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::remove_file(f.root.join("bad.json"))?;
    check(
        &f.root,
        Verdict::Incomplete,
        "BROKEN_CONTROL",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn restart_evidence_missing() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::remove_file(f.root.join("restart.json"))?;
    check(
        &f.root,
        Verdict::Incomplete,
        "RESTART_DEPENDENCY",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn unchanged_boot_is_invalid() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("restart.json", |r| {
        r["after_boot_id"] = r["before_boot_id"].clone()
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "RESTART_IDENTITIES",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn workflow_on_wrong_boot_is_invalid() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("after.json", |r| r["boot_id"] = "boot-before".into())?;
    check(
        &f.root,
        Verdict::Invalid,
        "WORKFLOW_BOOT",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn missing_workflow_boot_is_incomplete() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("after.json", |r| r["boot_id"] = serde_json::Value::Null)?;
    check(
        &f.root,
        Verdict::Incomplete,
        "WORKFLOW_BOOT",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn unknown_schema_version_rejected() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("bundle.json", |b| b["version"] = "999".into())?;
    check(
        &f.root,
        Verdict::Invalid,
        "SCHEMA_VERSION",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn unknown_mandatory_field_rejected() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("bundle.json", |b| {
        b["execute"] = json!(["never", "run", "this"])
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "CONTRACT_FORMAT",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn unexpected_optional_artifact_ignored() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::write(f.root.join("optional.json"), b"not even JSON")?;
    assert_eq!(verify(&f.root).verdict, Verdict::Complete);
    Ok(())
}

#[test]
fn missing_step_is_incomplete() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("before.json", |r| r["steps"] = json!([]))?;
    check(
        &f.root,
        Verdict::Incomplete,
        "STEP_PRESENT",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn not_run_and_blocked_steps_are_incomplete() -> TestResult {
    for status in ["NOT_RUN", "BLOCKED"] {
        let f = Fixture::synthetic()?;
        f.edit("before.json", |r| r["steps"][0]["status"] = status.into())?;
        check(
            &f.root,
            Verdict::Incomplete,
            "STEP_EXECUTED",
            CheckStatus::Incomplete,
        );
    }
    Ok(())
}

#[test]
fn complete_evidence_can_record_experimental_fail() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("bundle.json", |b| b["experimental_verdict"] = "FAIL".into())?;
    f.edit("before.json", |r| r["steps"][0]["status"] = "FAIL".into())?;
    f.edit("result.json", |r| {
        r["verdict"] = "FAIL".into();
        r["errors"] = json!(["synthetic content defect"]);
    })?;
    let r = verify(&f.root);
    assert_eq!(r.verdict, Verdict::Complete, "{r:?}");
    assert_eq!(r.experimental_verdict, Some(ExperimentStatus::Fail));
    assert!(r.checks.iter().any(|c| c.code == "CONTENT_RESULT_FAIL"));
    Ok(())
}

#[test]
fn duplicate_step_or_artifact_id_rejected() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("before.json", |r| {
        r["steps"] = json!([r["steps"][0], r["steps"][0]])
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "WORKFLOW_RECORD",
        CheckStatus::Invalid,
    );
    let f = Fixture::synthetic()?;
    f.edit("bundle.json", |b| {
        b["artifacts"][1]["id"] = b["artifacts"][0]["id"].clone()
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "CONTRACT_RULES",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn oracle_hash_must_bind_the_actual_output() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("result.json", |r| {
        r["archive_sha256"] = "0".repeat(64).into()
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "ORACLE_IDENTITY",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn oracle_pass_with_wrong_content_is_invalid() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("result.json", |r| {
        r["files"][0]["sha256"] = "0".repeat(64).into()
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "CONTENT_VERIFIER",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn malformed_inputs_fail_without_panic() -> TestResult {
    for raw in [
        b"not json".as_slice(),
        b"null",
        b"[]",
        b"{\"schema\":\"helm-evidence\"}",
        b"\xff",
    ] {
        let f = Fixture::synthetic()?;
        fs::write(f.root.join("bundle.json"), raw)?;
        assert_eq!(verify(&f.root).verdict, Verdict::Invalid);
    }
    let f = Fixture::synthetic()?;
    fs::write(f.root.join("before.json"), b"{}")?;
    f.repin("before.json")?;
    check(
        &f.root,
        Verdict::Invalid,
        "RECORD_FORMAT",
        CheckStatus::Invalid,
    );
    Ok(())
}

#[test]
fn invalid_takes_precedence_over_incomplete() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::remove_file(f.root.join("good.json"))?;
    fs::write(f.root.join("outputs/before/output.dat"), b"changed")?;
    assert_eq!(verify(&f.root).verdict, Verdict::Invalid);
    Ok(())
}

#[test]
fn a0_preserves_failure_and_w2_independent_success() {
    let r = verify(&fixture_path("a0-7zip"));
    assert_eq!(r.verdict, Verdict::Incomplete, "{r:?}");
    assert_eq!(r.experimental_verdict, Some(ExperimentStatus::Fail));
    assert!(
        r.sections
            .iter()
            .any(|s| s.id == "W2" && s.verdict == Verdict::Complete)
    );
    assert!(
        r.sections
            .iter()
            .any(|s| s.id == "W1" && s.verdict == Verdict::Incomplete)
    );
    assert!(
        r.checks
            .iter()
            .any(|c| c.workflow.as_deref() == Some("W1") && c.code == "CONTENT_RESULT_PASS")
    );
    let failures: Vec<_> = r
        .checks
        .iter()
        .filter(|c| c.status == CheckStatus::Incomplete || c.status == CheckStatus::Invalid)
        .collect();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].code, "OUTPUT_DESTINATION");
}

#[test]
fn cli_json_deterministic_and_exit_codes() -> TestResult {
    for (fixture, exit) in [("synthetic", 0), ("a0-7zip", 1)] {
        let run = || {
            Command::new(env!("CARGO_BIN_EXE_helm-evidence"))
                .arg("verify")
                .arg(fixture_path(fixture))
                .arg("--json")
                .output()
        };
        let a = run()?;
        let b = run()?;
        assert_eq!(a.status.code(), Some(exit));
        assert_eq!(a.stdout, b.stdout);
        assert!(a.stderr.is_empty());
        let parsed: serde_json::Value = serde_json::from_slice(&a.stdout)?;
        assert_eq!(parsed["report_version"], "0.1");
    }
    let f = Fixture::synthetic()?;
    fs::write(f.root.join("bundle.json"), b"{}")?;
    let result = Command::new(env!("CARGO_BIN_EXE_helm-evidence"))
        .arg("verify")
        .arg(&f.root)
        .arg("--json")
        .output()?;
    assert_eq!(result.status.code(), Some(2));
    for args in [
        vec![],
        vec!["verify"],
        vec!["other", "x"],
        vec!["verify", "x", "--unknown"],
    ] {
        assert_eq!(
            Command::new(env!("CARGO_BIN_EXE_helm-evidence"))
                .args(args)
                .status()?
                .code(),
            Some(64)
        );
    }
    Ok(())
}

#[test]
fn verification_is_read_only() -> TestResult {
    let f = Fixture::synthetic()?;
    fn snapshot(root: &std::path::Path) -> TestResult<Vec<(std::path::PathBuf, Vec<u8>)>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                entries.extend(snapshot(&entry.path())?);
            } else {
                entries.push((entry.path(), fs::read(entry.path())?));
            }
        }
        entries.sort();
        Ok(entries)
    }
    let before = snapshot(&f.root)?;
    assert_eq!(verify(&f.root).verdict, Verdict::Complete);
    assert_eq!(snapshot(&f.root)?, before);
    Ok(())
}

#[test]
fn equivalent_file_elsewhere_does_not_replace_missing_destination() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::rename(
        f.root.join("outputs/before/output.dat"),
        f.root.join("equivalent.dat"),
    )?;
    check(
        &f.root,
        Verdict::Incomplete,
        "ARTIFACT_MISSING",
        CheckStatus::Incomplete,
    );
    Ok(())
}

#[test]
fn local_output_reference_cannot_declare_a_different_destination() -> TestResult {
    let f = Fixture::synthetic()?;
    f.edit("bundle.json", |b| {
        b["workflows"][0]["output"]["local_artifact"] = "output-after".into()
    })?;
    check(
        &f.root,
        Verdict::Invalid,
        "LOCAL_OUTPUT_DECLARATION",
        CheckStatus::Invalid,
    );
    Ok(())
}
