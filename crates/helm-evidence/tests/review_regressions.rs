mod support;
use helm_evidence::{model::Verdict, verify};
use serde_json::json;
use std::fs;
use support::*;

#[test]
fn duplicate_content_expectations_are_invalid() -> TestResult {
    // Encode duplicate keys directly: going through Value would lose the defect.
    for second_key in ["payload.txt", "payload\\u002etxt"] {
        let f = Fixture::synthetic()?;
        let manifest = f.json("manifest.json")?;
        let matching = serde_json::to_string(&manifest["files"]["payload.txt"])?;
        let conflicting = json!({"bytes": 18, "sha256": "0".repeat(64)});
        let raw = format!(
            "{{\"files\":{{\"payload.txt\":{conflicting},\"{second_key}\":{matching}}},\"required_directories\":[],\"allowed_directories\":[]}}"
        );
        fs::write(f.root.join("manifest.json"), raw.as_bytes())?;
        f.repin("manifest.json")?;
        for name in ["result.json", "good.json", "bad.json"] {
            f.edit(name, |r| r["fixture_manifest_sha256"] = sha256(raw.as_bytes()).into())?;
        }
        let report = verify(&f.root);
        assert_eq!(report.verdict, Verdict::Invalid, "duplicate {second_key}: {report:?}");
        assert!(report.checks.iter().any(|c| c.code == "RECORD_FORMAT"));
    }
    Ok(())
}

#[test]
fn workflow_cannot_shadow_builtin_report_sections() -> TestResult {
    for id in ["contract", "artifacts", "restart"] {
        let f = Fixture::synthetic()?;
        f.edit("bundle.json", |b| b["workflows"][0]["id"] = id.into())?;
        f.edit("before.json", |r| r["workflow"] = id.into())?;
        let report = verify(&f.root);
        assert_eq!(report.verdict, Verdict::Invalid, "reserved {id}: {report:?}");
        assert!(report.checks.iter().any(|c| c.code == "WORKFLOW_DECLARATION"));
    }
    Ok(())
}
