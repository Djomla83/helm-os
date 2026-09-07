//! Verify existing oracle *evidence*, never execute or duplicate its ZIP content inspection.
use crate::{Artifacts, bundle::digest_shape, model::*, record};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn verify(
    output: &Output,
    contract: &Contract,
    artifacts: &Artifacts,
    workflow: Option<&str>,
    report: &mut Report,
) {
    let manifest: Option<ContentManifest> = record(
        artifacts,
        &output.content_manifest,
        workflow,
        &output.id,
        report,
    );
    let manifest_hash = contract
        .artifacts
        .iter()
        .find(|a| a.id == output.content_manifest)
        .map(|a| a.sha256.as_str());
    let manifest = manifest.filter(|m| {
        let valid = !m.files.is_empty() && m.files.len() <= 32 && m.allowed_directories.len() <= 32
            && m.required_directories.len() <= 32 && m.files.iter().all(|(p, f)| content_path(p)
                && digest_shape(&f.sha256) && f.bytes <= 1024 * 1024)
            && m.allowed_directories.iter().all(|p| p.ends_with('/') && content_path(p.trim_end_matches('/')))
            && m.required_directories.iter().all(|p| m.allowed_directories.contains(p));
        report.check("CONTENT_MANIFEST", if valid { CheckStatus::Satisfied } else { CheckStatus::Invalid },
            workflow, Some(&output.content_manifest), &output.id, "Frozen content manifest must contain bounded, valid file identities and directory requirements.");
        valid
    });
    for (id, hash, expected, code) in [
        (
            &output.content_result,
            &output.sha256,
            None,
            "CONTENT_VERIFIER",
        ),
        (
            &output.known_good.result,
            &output.known_good.sha256,
            Some(ExperimentStatus::Pass),
            "KNOWN_GOOD_CONTROL",
        ),
        (
            &output.deliberately_broken.result,
            &output.deliberately_broken.sha256,
            Some(ExperimentStatus::Fail),
            "BROKEN_CONTROL",
        ),
    ] {
        let oracle: Option<OracleRecord> = record(artifacts, id, workflow, &output.id, report);
        let (Some(oracle), Some(manifest), Some(manifest_hash)) =
            (oracle, &manifest, manifest_hash)
        else {
            report.check(code, CheckStatus::Incomplete, workflow, Some(id), &output.id,
                "Required independent verifier/control evidence or its frozen expectation is unavailable.");
            continue;
        };
        let mut paths = BTreeSet::new();
        let shape = oracle.scope == "ZIP output-content verification only"
            && matches!(
                oracle.verdict,
                ExperimentStatus::Pass | ExperimentStatus::Fail
            )
            && (oracle.verdict == ExperimentStatus::Pass) == oracle.errors.is_empty()
            && oracle.files.len() <= 32
            && oracle.archive_bytes <= 1024 * 1024
            && oracle.files.iter().all(|f| {
                content_path(&f.path)
                    && paths.insert(&f.path)
                    && digest_shape(&f.sha256)
                    && f.bytes <= 1024 * 1024
            });
        let identities =
            oracle.archive_sha256 == *hash && oracle.fixture_manifest_sha256 == manifest_hash;
        report.check("ORACLE_IDENTITY", if shape && identities { CheckStatus::Satisfied } else { CheckStatus::Invalid },
            workflow, Some(id), &output.id, "Verifier record must bind the expected output/control SHA-256 and frozen manifest, with a consistent PASS/FAIL result.");
        let observed: BTreeMap<_, _> = oracle
            .files
            .iter()
            .map(|f| {
                (
                    f.path.clone(),
                    ContentIdentity {
                        bytes: f.bytes,
                        sha256: f.sha256.clone(),
                    },
                )
            })
            .collect();
        // FAIL is an experimental observation, not necessarily missing evidence. PASS must
        // agree with every file identity. Directory/encryption/CRC checks remain the oracle's work.
        let truthful_pass = oracle.verdict != ExperimentStatus::Pass || observed == manifest.files;
        let control_matches = expected.is_none_or(|expected| oracle.verdict == expected)
            && (expected != Some(ExperimentStatus::Fail) || observed != manifest.files);
        let valid = shape && identities && truthful_pass && control_matches;
        report.check(code, if valid { CheckStatus::Satisfied } else { CheckStatus::Invalid },
            workflow, Some(id), &output.id, match expected {
                Some(ExperimentStatus::Pass) => "Known-good control must be accepted with the frozen file contents.",
                Some(_) => "Deliberately corrupted control must be rejected; accepting it invalidates interpretation.",
                None => "Independent content-verifier evidence is present and identity-bound; PASS must agree with the frozen file identities, while an evidenced content FAIL remains an experimental result.",
            });
        if expected.is_none() && valid {
            report.check(if oracle.verdict == ExperimentStatus::Pass { "CONTENT_RESULT_PASS" } else { "CONTENT_RESULT_FAIL" },
                CheckStatus::Satisfied, workflow, Some(id), &output.id,
                "Recorded content result is separate from destination and workflow completeness; the oracle was not rerun.");
        }
    }
}

fn content_path(path: &str) -> bool {
    // Unicode archive member names are evidence data, never opened as host paths.
    !path.is_empty()
        && path.len() <= 1024
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_archive_names_are_data() {
        assert!(content_path("fixture/names/café-漢字.txt"));
        assert!(!content_path("fixture/../canary"));
    }
}
