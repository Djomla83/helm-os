//! Read-only evidence verification. COMPLETE applies only to the supplied contract;
//! it is never an application compatibility rating or authentication of an author.
#![forbid(unsafe_code)]
mod bundle;
pub mod model;
mod oracle;

use bundle::{Bundle, ReadError, digest_shape, identifier, safe_path};
use model::*;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    path::Path,
};

type Artifacts = BTreeMap<String, Result<Vec<u8>, CheckStatus>>;

/// Verify `bundle.json` and its declared references inside `root`, without writing or executing.
pub fn verify(root: &Path) -> Report {
    let mut report = Report::new();
    let mut bundle = match Bundle::open(root) {
        Ok(bundle) => bundle,
        Err(error) => {
            read_failure(&mut report, error, None, "bundle", "root");
            return report.finish();
        }
    };
    let bytes = match bundle.read("bundle.json", bundle::MANIFEST_LIMIT) {
        Ok(bytes) => bytes,
        Err(error) => {
            read_failure(&mut report, error, None, "bundle", "contract");
            return report.finish();
        }
    };
    let contract: Contract = match serde_json::from_slice(&bytes) {
        Ok(contract) => contract,
        Err(_) => {
            report.check(
                "CONTRACT_FORMAT",
                CheckStatus::Invalid,
                None,
                None,
                "contract",
                "Contract is not the required typed JSON shape (unknown fields are rejected).",
            );
            return report.finish();
        }
    };
    if !validate_contract(&contract, &mut report) {
        return report.finish();
    }
    report.experimental_verdict = Some(contract.experimental_verdict);
    report.section("contract", 0);
    let start = report.checks.len();
    let mut artifacts = BTreeMap::new();
    for artifact in &contract.artifacts {
        let data = match bundle.read(&artifact.path, bundle::FILE_LIMIT) {
            Ok(bytes) => {
                report.check(
                    "ARTIFACT_EXISTS",
                    CheckStatus::Satisfied,
                    None,
                    Some(&artifact.id),
                    &artifact.id,
                    "Declared regular artifact is present inside the bundle.",
                );
                let status = if sha256(&bytes) == artifact.sha256 {
                    CheckStatus::Satisfied
                } else {
                    CheckStatus::Invalid
                };
                report.check(
                    "ARTIFACT_HASH",
                    status,
                    None,
                    Some(&artifact.id),
                    &artifact.id,
                    "Artifact SHA-256 must match the declared identity.",
                );
                if status == CheckStatus::Satisfied {
                    Ok(bytes)
                } else {
                    Err(status)
                }
            }
            Err(error) => Err(read_failure(
                &mut report,
                error,
                None,
                &artifact.id,
                &artifact.id,
            )),
        };
        artifacts.insert(artifact.id.clone(), data);
    }
    report.section("artifacts", start);
    let start = report.checks.len();
    let restart = verify_restart(&contract, &artifacts, &mut report);
    let restart_status = match report.checks[start..]
        .iter()
        .map(|c| c.status)
        .find(|s| *s == CheckStatus::Invalid)
    {
        Some(status) => status,
        None if report.checks[start..]
            .iter()
            .any(|c| c.status == CheckStatus::Incomplete) =>
        {
            CheckStatus::Incomplete
        }
        _ => CheckStatus::Satisfied,
    };
    report.section("restart", start);
    for workflow in &contract.workflows {
        let start = report.checks.len();
        verify_workflow(
            workflow,
            &contract,
            &artifacts,
            restart.as_ref(),
            restart_status,
            &mut report,
        );
        report.section(&workflow.id, start);
    }
    report.finish()
}

fn validate_contract(contract: &Contract, report: &mut Report) -> bool {
    if contract.schema != SCHEMA || contract.version != VERSION {
        report.check(
            "SCHEMA_VERSION",
            CheckStatus::Invalid,
            None,
            None,
            "contract",
            "Only helm-evidence schema version 0.1 is supported; unknown versions are rejected.",
        );
        return false;
    }
    let mut valid = identifier(&contract.experiment)
        && identifier(&contract.application.id)
        && identifier(&contract.application.version)
        && digest_shape(&contract.application.source_sha256)
        && (1..=128).contains(&contract.artifacts.len())
        && (1..=16).contains(&contract.workflows.len());
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for artifact in &contract.artifacts {
        let declared = identifier(&artifact.id)
            && ids.insert(artifact.id.as_str())
            && digest_shape(&artifact.sha256)
            && paths.insert(artifact.path.to_ascii_lowercase());
        valid &= declared;
        if !declared {
            report.check(
                "ARTIFACT_DECLARATION",
                CheckStatus::Invalid,
                None,
                Some(public_id(&artifact.id)),
                "artifact",
                "Artifact needs a unique ID/path and a lowercase SHA-256 identity.",
            );
        }
        if !safe_path(&artifact.path) {
            report.check(
                "PATH_UNSAFE",
                CheckStatus::Invalid,
                None,
                Some(if identifier(&artifact.id) {
                    &artifact.id
                } else {
                    "artifact"
                }),
                "path",
                "Artifact path must be a portable relative path without traversal or device names.",
            );
            valid = false;
        }
    }
    let reference = |id: &str| identifier(id) && ids.contains(id);
    let refs = |list: &[String]| {
        !list.is_empty()
            && list.len() <= 128
            && list.iter().all(|id| reference(id))
            && list.iter().collect::<BTreeSet<_>>().len() == list.len()
    };
    let mut workflows = BTreeSet::new();
    for workflow in &contract.workflows {
        let declared = identifier(&workflow.id)
            && !matches!(workflow.id.as_str(), "contract" | "artifacts" | "restart")
            && workflows.insert(workflow.id.as_str())
            && reference(&workflow.record)
            && (1..=32).contains(&workflow.steps.len());
        valid &= declared;
        if !declared {
            report.check(
                "WORKFLOW_DECLARATION",
                CheckStatus::Invalid,
                Some(public_id(&workflow.id)),
                None,
                "workflow",
                "Workflow needs a unique non-reserved ID, a declared record reference, and 1-32 required steps.",
            );
        }
        let mut steps = BTreeSet::new();
        for step in &workflow.steps {
            let declared =
                identifier(&step.id) && steps.insert(step.id.as_str()) && refs(&step.evidence);
            valid &= declared;
            if !declared {
                report.check("STEP_DECLARATION", CheckStatus::Invalid, Some(public_id(&workflow.id)),
                    None, public_id(&step.id), "Step needs a unique ID and a bounded nonempty list of distinct declared evidence references.");
            }
        }
        if workflow.phase != Phase::Standalone {
            valid &= contract.restart.is_some();
            if contract.restart.is_none() {
                report.check(
                    "RESTART_DECLARATION",
                    CheckStatus::Invalid,
                    Some(public_id(&workflow.id)),
                    None,
                    "restart",
                    "A before/after phase requires an explicit restart contract.",
                );
            }
        }
        if let Some(output) = &workflow.output {
            let declared = identifier(&output.id)
                && safe_path(&output.expected_path)
                && digest_shape(&output.sha256)
                && reference(&output.content_result)
                && reference(&output.content_manifest)
                && reference(&output.known_good.result)
                && digest_shape(&output.known_good.sha256)
                && reference(&output.deliberately_broken.result)
                && digest_shape(&output.deliberately_broken.sha256)
                && output.known_good.result != output.deliberately_broken.result
                && output.known_good.sha256 != output.deliberately_broken.sha256;
            valid &= declared;
            if !declared {
                report.check("OUTPUT_DECLARATION", CheckStatus::Invalid, Some(public_id(&workflow.id)),
                    None, public_id(&output.id), "Output needs a safe destination, valid digests and declared content/manifest/distinct-control references.");
            }
            if let Some(local) = &output.local_artifact {
                let declared = reference(local)
                    && contract.artifacts.iter().any(|a| {
                        a.id == *local
                            && a.sha256 == output.sha256
                            && a.path == output.expected_path
                    });
                valid &= declared;
                if !declared {
                    report.check("LOCAL_OUTPUT_DECLARATION", CheckStatus::Invalid, Some(public_id(&workflow.id)),
                        Some(public_id(local)), public_id(&output.id), "Local output artifact must be declared at the exact required path with the expected output digest.");
                }
            }
        }
    }
    if let Some(restart) = &contract.restart {
        let declared = reference(&restart.record)
            && refs(&restart.evidence)
            && contract
                .workflows
                .iter()
                .any(|w| w.phase == Phase::BeforeRestart)
            && contract
                .workflows
                .iter()
                .any(|w| w.phase == Phase::AfterRestart);
        valid &= declared;
        if !declared {
            report.check("RESTART_DECLARATION", CheckStatus::Invalid, None, Some(public_id(&restart.record)),
                "restart", "Restart needs declared record/evidence references and both before/after workflow phases.");
        }
    }
    report.check("CONTRACT_RULES", if valid { CheckStatus::Satisfied } else { CheckStatus::Invalid },
        None, None, "contract", "Contract requires bounded unique identifiers, valid digests, declared references, and explicit before/after phases for a restart.");
    valid
}

fn public_id(id: &str) -> &str {
    if identifier(id) { id } else { "invalid-id" }
}

fn read_failure(
    report: &mut Report,
    error: ReadError,
    workflow: Option<&str>,
    artifact: &str,
    requirement: &str,
) -> CheckStatus {
    let (code, status, explanation) = match error {
        ReadError::Missing => (
            "ARTIFACT_MISSING",
            CheckStatus::Incomplete,
            "Required artifact is absent.",
        ),
        ReadError::Unsafe => (
            "PATH_UNSAFE",
            CheckStatus::Invalid,
            "Symlinks, reparse points, unsafe paths and non-regular artifacts are rejected.",
        ),
        ReadError::Io => (
            "ARTIFACT_UNREADABLE",
            CheckStatus::Invalid,
            "Artifact cannot be read within the permitted bundle capability.",
        ),
        ReadError::TooLarge => (
            "RESOURCE_LIMIT",
            CheckStatus::Invalid,
            "Manifest, artifact or total read size exceeds the fixed limit.",
        ),
    };
    report.check(
        code,
        status,
        workflow,
        Some(artifact),
        requirement,
        explanation,
    );
    status
}

fn required<'a>(
    artifacts: &'a Artifacts,
    id: &str,
    workflow: Option<&str>,
    requirement: &str,
    report: &mut Report,
) -> Option<&'a [u8]> {
    match artifacts.get(id) {
        Some(Ok(bytes)) => {
            report.check(
                "EVIDENCE_REFERENCE",
                CheckStatus::Satisfied,
                workflow,
                Some(id),
                requirement,
                "Required evidence artifact is present and hash-verified.",
            );
            Some(bytes)
        }
        other => {
            let status = match other {
                Some(Err(status)) => *status,
                _ => CheckStatus::Incomplete,
            };
            report.check(
                "EVIDENCE_UNAVAILABLE",
                status,
                workflow,
                Some(id),
                requirement,
                "Required evidence is absent or invalid; it cannot satisfy this requirement.",
            );
            None
        }
    }
}

fn record<T: DeserializeOwned>(
    artifacts: &Artifacts,
    id: &str,
    workflow: Option<&str>,
    requirement: &str,
    report: &mut Report,
) -> Option<T> {
    let bytes = required(artifacts, id, workflow, requirement, report)?;
    match serde_json::from_slice(bytes) {
        Ok(record) => Some(record),
        Err(_) => {
            report.check(
                "RECORD_FORMAT",
                CheckStatus::Invalid,
                workflow,
                Some(id),
                requirement,
                "Required machine record is malformed or lacks a typed mandatory field.",
            );
            None
        }
    }
}

fn verify_restart(
    contract: &Contract,
    artifacts: &Artifacts,
    report: &mut Report,
) -> Option<RestartRecord> {
    let Some(restart) = &contract.restart else {
        report.check(
            "RESTART_NOT_REQUIRED",
            CheckStatus::NotRequired,
            None,
            None,
            "restart",
            "This contract declares no restart boundary.",
        );
        return None;
    };
    for id in &restart.evidence {
        required(artifacts, id, None, "restart", report);
    }
    let record: RestartRecord = record(artifacts, &restart.record, None, "restart", report)?;
    let valid = record.version == VERSION
        && identifier(&record.before_boot_id)
        && identifier(&record.after_boot_id)
        && record.before_boot_id != record.after_boot_id;
    report.check(
        "RESTART_IDENTITIES",
        if valid {
            CheckStatus::Satisfied
        } else {
            CheckStatus::Invalid
        },
        None,
        Some(&restart.record),
        "restart",
        "Restart record must bind distinct, nonempty before/after boot identities in version 0.1.",
    );
    if valid { Some(record) } else { None }
}

fn verify_workflow(
    workflow: &Workflow,
    contract: &Contract,
    artifacts: &Artifacts,
    restart: Option<&RestartRecord>,
    restart_status: CheckStatus,
    report: &mut Report,
) {
    let who = Some(workflow.id.as_str());
    if workflow.phase != Phase::Standalone {
        report.check(
            "RESTART_DEPENDENCY",
            restart_status,
            who,
            None,
            "restart",
            "This workflow requires the declared restart evidence and boot identities.",
        );
    }
    for step in &workflow.steps {
        for id in &step.evidence {
            required(artifacts, id, who, &step.id, report);
        }
    }
    let observed: Option<WorkflowRecord> =
        record(artifacts, &workflow.record, who, "workflow", report);
    let observed = observed.filter(|r| {
        let mut ids = BTreeSet::new();
        let valid = r.version == VERSION && r.workflow == workflow.id && r.steps.len() <= 32
            && r.steps.iter().all(|s| identifier(&s.id) && ids.insert(s.id.as_str()))
            && r.boot_id.as_ref().is_none_or(|id| identifier(id))
            && r.output.as_ref().is_none_or(|o| safe_path(&o.path) && digest_shape(&o.sha256));
        report.check("WORKFLOW_RECORD", if valid { CheckStatus::Satisfied } else { CheckStatus::Invalid },
            who, Some(&workflow.record), "workflow", "Workflow record must match its declared identity/version and contain unambiguous typed observations.");
        valid
    });
    for step in &workflow.steps {
        let status = observed
            .as_ref()
            .and_then(|r| r.steps.iter().find(|s| s.id == step.id))
            .map(|s| s.status);
        report.check(
            "STEP_PRESENT",
            if status.is_some() {
                CheckStatus::Satisfied
            } else {
                CheckStatus::Incomplete
            },
            who,
            Some(&workflow.record),
            &step.id,
            "Every required step must have an explicit machine-readable status.",
        );
        if let Some(status) = status {
            report.recorded_steps.push(RecordedStep {
                workflow: workflow.id.clone(),
                step: step.id.clone(),
                status,
            });
            report.check("STEP_EXECUTED", if matches!(status, ExperimentStatus::NotRun | ExperimentStatus::Blocked) {
                CheckStatus::Incomplete
            } else { CheckStatus::Satisfied }, who, Some(&workflow.record), &step.id,
                "Required step must have run; recorded FAIL or INCONCLUSIVE is not silently changed to PASS.");
        }
    }
    if let Some(observed) = &observed
        && workflow.phase != Phase::Standalone
    {
        let expected = restart.map(|r| {
            if workflow.phase == Phase::BeforeRestart {
                &r.before_boot_id
            } else {
                &r.after_boot_id
            }
        });
        let status = match (&observed.boot_id, expected) {
            (Some(actual), Some(expected)) if actual == expected => CheckStatus::Satisfied,
            (Some(_), Some(_)) => CheckStatus::Invalid,
            _ => CheckStatus::Incomplete,
        };
        report.check(
            "WORKFLOW_BOOT",
            status,
            who,
            Some(&workflow.record),
            "boot",
            "Workflow boot identity must match its declared side of the restart boundary.",
        );
    }
    if let Some(output) = &workflow.output {
        let observation = observed.as_ref().and_then(|r| r.output.as_ref());
        report.check(
            "OUTPUT_OBSERVED",
            if observation.is_some() {
                CheckStatus::Satisfied
            } else {
                CheckStatus::Incomplete
            },
            who,
            Some(&workflow.record),
            &output.id,
            "A machine-readable output observation is required.",
        );
        if let Some(observation) = observation {
            report.check(
                "OUTPUT_IDENTITY",
                if observation.sha256 == output.sha256 {
                    CheckStatus::Satisfied
                } else {
                    CheckStatus::Invalid
                },
                who,
                Some(&workflow.record),
                &output.id,
                "Recorded output identity must match the expected SHA-256.",
            );
            report.check("OUTPUT_DESTINATION", if observation.path == output.expected_path { CheckStatus::Satisfied } else { CheckStatus::Incomplete },
                who, Some(&workflow.record), &output.id, "Observed output must be at the exact required destination; content correctness cannot substitute for location.");
        } else {
            report.check(
                "OUTPUT_DESTINATION",
                CheckStatus::Incomplete,
                who,
                Some(&workflow.record),
                &output.id,
                "No output observation establishes the required destination.",
            );
        }
        if let Some(local) = &output.local_artifact {
            required(artifacts, local, who, &output.id, report);
        } else {
            report.check("OUTPUT_BYTES_NOT_REQUIRED", CheckStatus::NotRequired, who, None, &output.id,
                "Contract requires recorded output evidence only; output bytes were not locally rehashed.");
        }
        oracle::verify(output, contract, artifacts, who, report);
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Human formatting never echoes filesystem paths, raw evidence strings, or parser/OS errors.
pub fn write_human(out: &mut impl Write, report: &Report) -> io::Result<()> {
    writeln!(out, "Evidence bundle: {}", verdict_name(report.verdict))?;
    if let Some(experiment) = report.experimental_verdict {
        writeln!(out, "Recorded experiment: {}", experiment.as_str())?;
    }
    for section in &report.sections {
        writeln!(out, "{}: {}", section.id, verdict_name(section.verdict))?;
    }
    for check in &report.checks {
        if matches!(check.status, CheckStatus::Incomplete | CheckStatus::Invalid) {
            writeln!(
                out,
                "{} workflow={} artifact={} requirement={}: {}",
                check.code,
                check.workflow.as_deref().unwrap_or("-"),
                check.artifact.as_deref().unwrap_or("-"),
                check.requirement,
                check.explanation
            )?;
        }
    }
    writeln!(
        out,
        "Completeness is scoped to this contract; no general application compatibility verdict."
    )
}

fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Complete => "COMPLETE",
        Verdict::Incomplete => "INCOMPLETE",
        Verdict::Invalid => "INVALID",
    }
}
