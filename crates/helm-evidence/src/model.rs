//! The unstable 0.1 contract. All listed artifacts, steps and workflows are required.
use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = "helm-evidence";
pub const VERSION: &str = "0.1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub schema: String,
    pub version: String,
    pub experiment: String,
    pub application: Application,
    pub experimental_verdict: ExperimentStatus,
    pub artifacts: Vec<Artifact>,
    pub workflows: Vec<Workflow>,
    pub restart: Option<Restart>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    pub id: String,
    pub version: String,
    pub source_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workflow {
    pub id: String,
    pub phase: Phase,
    pub record: String,
    pub steps: Vec<RequiredStep>,
    pub output: Option<Output>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredStep {
    pub id: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Standalone,
    BeforeRestart,
    AfterRestart,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub id: String,
    /// Relative to the experiment's declared working directory, never opened on the host.
    pub expected_path: String,
    pub sha256: String,
    pub content_result: String,
    pub content_manifest: String,
    pub known_good: Control,
    pub deliberately_broken: Control,
    /// If declared, these bytes must also be present in the bundle and rehashed.
    pub local_artifact: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub result: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Restart {
    pub record: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowRecord {
    pub version: String,
    pub workflow: String,
    pub boot_id: Option<String>,
    pub steps: Vec<StepRecord>,
    pub output: Option<OutputRecord>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StepRecord {
    pub id: String,
    pub status: ExperimentStatus,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OutputRecord {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RestartRecord {
    pub version: String,
    pub before_boot_id: String,
    pub after_boot_id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExperimentStatus {
    Pass,
    Fail,
    Inconclusive,
    Blocked,
    NotRun,
}

impl ExperimentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Inconclusive => "INCONCLUSIVE",
            Self::Blocked => "BLOCKED",
            Self::NotRun => "NOT_RUN",
        }
    }
}

// These three types consume the existing app_baseline.py format, not a replacement oracle.
// Historical auxiliary metadata is allowed; required semantic fields remain typed.
#[derive(Debug, Deserialize)]
pub(crate) struct OracleRecord {
    pub scope: String,
    pub verdict: ExperimentStatus,
    pub errors: Vec<String>,
    pub files: Vec<FileIdentity>,
    pub archive_sha256: String,
    pub archive_bytes: u64,
    pub fixture_manifest_sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FileIdentity {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct ContentIdentity {
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentManifest {
    pub files: std::collections::BTreeMap<String, ContentIdentity>,
    pub required_directories: Vec<String>,
    pub allowed_directories: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Complete,
    Incomplete,
    Invalid,
}

impl Verdict {
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Complete => 0,
            Self::Incomplete => 1,
            Self::Invalid => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Satisfied,
    Incomplete,
    Invalid,
    NotRequired,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub code: &'static str,
    pub status: CheckStatus,
    pub workflow: Option<String>,
    pub artifact: Option<String>,
    pub requirement: String,
    pub explanation: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Section {
    pub id: String,
    pub verdict: Verdict,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub report_version: &'static str,
    pub verdict: Verdict,
    pub experimental_verdict: Option<ExperimentStatus>,
    pub recorded_steps: Vec<RecordedStep>,
    pub sections: Vec<Section>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Serialize)]
pub struct RecordedStep {
    pub workflow: String,
    pub step: String,
    pub status: ExperimentStatus,
}

impl Report {
    pub(crate) fn new() -> Self {
        Self {
            report_version: VERSION,
            verdict: Verdict::Incomplete,
            experimental_verdict: None,
            recorded_steps: Vec::new(),
            sections: Vec::new(),
            checks: Vec::new(),
        }
    }

    pub(crate) fn check(
        &mut self,
        code: &'static str,
        status: CheckStatus,
        workflow: Option<&str>,
        artifact: Option<&str>,
        requirement: &str,
        explanation: &'static str,
    ) {
        self.checks.push(Check {
            code,
            status,
            workflow: workflow.map(str::to_owned),
            artifact: artifact.map(str::to_owned),
            requirement: requirement.to_owned(),
            explanation,
        });
    }

    pub(crate) fn section(&mut self, id: &str, start: usize) {
        self.sections.push(Section {
            id: id.to_owned(),
            verdict: aggregate(&self.checks[start..]),
        });
    }

    pub(crate) fn finish(mut self) -> Self {
        self.verdict = aggregate(&self.checks);
        self
    }
}

fn aggregate(checks: &[Check]) -> Verdict {
    if checks.iter().any(|c| c.status == CheckStatus::Invalid) {
        Verdict::Invalid
    } else if checks.iter().any(|c| c.status == CheckStatus::Incomplete) {
        Verdict::Incomplete
    } else {
        Verdict::Complete
    }
}
