//! Immutable binding results and the deterministic report serializer.
//!
//! No satisfaction, compatibility or readiness vocabulary appears here. There is
//! deliberately no global success token of any kind: the report states which
//! values disagreed and how much of the specification was reached, and stops.

use sha2::{Digest as _, Sha256};

use crate::plan::Digest;

/// Upper bound on a serialized binding report. It is a proven bound, not a
/// truncation point: the widest report the schema and limits can express is far
/// below it, and nothing is ever cut to fit.
pub const MAX_REPORT_BYTES: usize = 16 * 1024;

/// Which semantic requirement a claim outcome concerns.
///
/// Contextual and human metadata — the specification digest, application ID,
/// application version, artifact labels — and role selector keys are deliberately
/// absent: they receive no outcome and do not enter coverage.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClaimSubject {
    /// Immutable application-source body identity.
    SourceBody,
    /// Declared source architecture. No comparator exists in 0.1.
    SourceArchitecture,
    /// Declared runtime family. No comparator exists in 0.1.
    RuntimeFamily,
    /// Immutable runtime-artifact body identity for one declared role.
    RuntimeArtifactBody(String),
    /// Declared Windows architecture. No comparator exists in 0.1.
    WindowsArchitecture,
    /// Declared prefix role. No comparator exists in 0.1.
    PrefixRole,
    /// One declared disabled-DLL requirement. No comparator exists in 0.1.
    DisabledDll(String),
    /// Existence and regular-file kind at the desired relative path.
    EntryPointPresence,
    /// Installed entry-point body identity.
    EntryPointBody,
    /// Frozen verification-definition body identity for one declared role.
    VerificationDefinitionBody(String),
}

impl ClaimSubject {
    /// Stable wire spelling of the claim class.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SourceBody => "source_body",
            Self::SourceArchitecture => "source_architecture",
            Self::RuntimeFamily => "runtime_family",
            Self::RuntimeArtifactBody(_) => "runtime_artifact_body",
            Self::WindowsArchitecture => "windows_architecture",
            Self::PrefixRole => "prefix_role",
            Self::DisabledDll(_) => "disabled_dll",
            Self::EntryPointPresence => "entry_point_presence",
            Self::EntryPointBody => "entry_point_body",
            Self::VerificationDefinitionBody(_) => "verification_definition_body",
        }
    }

    /// The declared role or name this instance selects, when the class has instances.
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        match self {
            Self::RuntimeArtifactBody(r)
            | Self::DisabledDll(r)
            | Self::VerificationDefinitionBody(r) => Some(r),
            _ => None,
        }
    }

    /// Whether 0.1 has any comparator for this class.
    #[must_use]
    pub const fn is_supported(&self) -> bool {
        !matches!(
            self,
            Self::SourceArchitecture
                | Self::RuntimeFamily
                | Self::WindowsArchitecture
                | Self::PrefixRole
                | Self::DisabledDll(_)
        )
    }
}

/// Which known values disagreed. Never a judgement about desired state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Difference {
    /// Observed length differs from the declared length.
    Size,
    /// Observed digest differs from the declared digest.
    DigestValue,
    /// Both differ.
    SizeAndDigest,
    /// The mapped target's declared relative path is not the desired path.
    EntryPointPath,
}

impl Difference {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Size => "size",
            Self::DigestValue => "digest",
            Self::SizeAndDigest => "size_and_digest",
            Self::EntryPointPath => "entry_point_path",
        }
    }
}

/// Why the observer refused the mapped object. A fact about the object, not a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum RejectionReason {
    SymlinkForbidden,
    SpecialFile,
    WrongKind,
    ScopeViolation,
    /// The observer reported a rejection this build does not recognise. The
    /// outcome kind is known; only the reason is not.
    Unrecognised,
}

impl RejectionReason {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SymlinkForbidden => "symlink_forbidden",
            Self::SpecialFile => "special_file",
            Self::WrongKind => "wrong_kind",
            Self::ScopeViolation => "scope_violation",
            Self::Unrecognised => "unrecognised",
        }
    }
}

/// Why the observation attempt could not complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FailureReason {
    PermissionDenied,
    IoFailure,
    ResolutionRace,
    ChangedDuringRead,
    ReopenUnavailable,
    UnsupportedPlatform,
    /// The observer reported a failure this build does not recognise.
    Unrecognised,
}

impl FailureReason {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PermissionDenied => "permission_denied",
            Self::IoFailure => "io_failure",
            Self::ResolutionRace => "resolution_race",
            Self::ChangedDuringRead => "changed_during_read",
            Self::ReopenUnavailable => "reopen_unavailable",
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::Unrecognised => "unrecognised",
        }
    }
}

/// Why the observer bounded the target away without attempting it fully.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum OmissionReason {
    FileLimit,
    TotalLimit,
    NotAttemptedTotalLimit,
    /// The observer reported a budget reason this build does not recognise.
    Unrecognised,
}

impl OmissionReason {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileLimit => "file_limit",
            Self::TotalLimit => "total_limit",
            Self::NotAttemptedTotalLimit => "not_attempted_total_limit",
            Self::Unrecognised => "unrecognised",
        }
    }
}

/// What actually followed from comparing one desired claim with the observations.
///
/// Ten distinct states. None collapses into another, and none of them is a
/// judgement about whether the application is satisfied, compatible or ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClaimState {
    /// Both sides were known and agreed.
    Match,
    /// Both sides were known and disagreed.
    Mismatch(Difference),
    /// The specification declines to state the value, so there is nothing to compare.
    DesiredValueUnspecified,
    /// A comparator exists but the binding plan supplied no mapping.
    NotObserved,
    /// No comparator exists in 0.1 for this claim class.
    UnsupportedBinding,
    /// The observer established that the mapped target is absent.
    Absent,
    /// The observer refused the mapped object.
    ObservationRejected(RejectionReason),
    /// The observation attempt could not complete.
    ObservationFailed(FailureReason),
    /// The observer bounded the target away.
    ObservationOmitted(OmissionReason),
    /// The artifact carried an outcome this build cannot interpret. Never
    /// agreement, and never silently benign.
    ObservationNotInterpretable,
}

impl ClaimState {
    /// Stable wire spelling of the state itself.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch(_) => "mismatch",
            Self::DesiredValueUnspecified => "desired_value_unspecified",
            Self::NotObserved => "not_observed",
            Self::UnsupportedBinding => "unsupported_binding",
            Self::Absent => "absent",
            Self::ObservationRejected(_) => "observation_rejected",
            Self::ObservationFailed(_) => "observation_failed",
            Self::ObservationOmitted(_) => "observation_omitted",
            Self::ObservationNotInterpretable => "observation_not_interpretable",
        }
    }

    /// Only a mismatch of two known values participates in the contradiction axis.
    #[must_use]
    pub const fn is_mismatch(self) -> bool {
        matches!(self, Self::Mismatch(_))
    }
}

/// One desired claim and what followed for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimBinding {
    subject: ClaimSubject,
    state: ClaimState,
}

impl ClaimBinding {
    pub(crate) const fn new(subject: ClaimSubject, state: ClaimState) -> Self {
        Self { subject, state }
    }

    /// Which semantic requirement this outcome concerns.
    #[must_use]
    pub const fn subject(&self) -> &ClaimSubject {
        &self.subject
    }

    /// What followed for it.
    #[must_use]
    pub const fn state(&self) -> ClaimState {
        self.state
    }
}

/// Whether any two known values disagreed.
///
/// Absence, observer rejection, observer failure, budget omission, unsupported
/// binding, an unmapped claim, an unspecified desired value and an
/// uninterpretable outcome are **never** promoted into contradiction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Contradiction {
    /// No compared pair disagreed. This is **not** a success statement.
    NoClaimContradicted,
    /// At least one compared pair disagreed.
    Contradicted {
        /// How many claims are in the mismatch state.
        mismatches: u32,
    },
}

impl Contradiction {
    /// The contradiction axis over a set of claim states.
    ///
    /// Only a `Mismatch` of two known values participates. Absence, observer
    /// rejection, observer failure, budget omission, unsupported binding, an
    /// unmapped claim, an unspecified desired value and an uninterpretable outcome
    /// are never promoted.
    pub(crate) fn from_states(states: impl Iterator<Item = ClaimState>) -> Self {
        let mismatches =
            u32::try_from(states.filter(|s| s.is_mismatch()).count()).unwrap_or(u32::MAX);
        if mismatches == 0 {
            Self::NoClaimContradicted
        } else {
            Self::Contradicted { mismatches }
        }
    }

    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoClaimContradicted => "no_claim_contradicted",
            Self::Contradicted { .. } => "contradicted",
        }
    }
}

/// How much of the specification the comparison actually reached.
///
/// There is no completeness token: in 0.1 at least four mandatory semantic
/// requirements have no comparator at all, so coverage is always partial.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Coverage {
    /// Total semantic claim instances, which is the denominator.
    pub claims: u32,
    pub matched: u32,
    pub mismatched: u32,
    pub desired_value_unspecified: u32,
    pub not_observed: u32,
    pub unsupported_binding: u32,
    pub absent: u32,
    pub observation_rejected: u32,
    pub observation_failed: u32,
    pub observation_omitted: u32,
    pub observation_not_interpretable: u32,
    /// Distinct claim classes with no comparator in 0.1, in fixed order.
    pub unsupported_classes: Vec<&'static str>,
}

impl Coverage {
    pub(crate) fn tally(&mut self, state: ClaimState) {
        self.claims = self.claims.saturating_add(1);
        let slot = match state {
            ClaimState::Match => &mut self.matched,
            ClaimState::Mismatch(_) => &mut self.mismatched,
            ClaimState::DesiredValueUnspecified => &mut self.desired_value_unspecified,
            ClaimState::NotObserved => &mut self.not_observed,
            ClaimState::UnsupportedBinding => &mut self.unsupported_binding,
            ClaimState::Absent => &mut self.absent,
            ClaimState::ObservationRejected(_) => &mut self.observation_rejected,
            ClaimState::ObservationFailed(_) => &mut self.observation_failed,
            ClaimState::ObservationOmitted(_) => &mut self.observation_omitted,
            ClaimState::ObservationNotInterpretable => &mut self.observation_not_interpretable,
        };
        *slot = slot.saturating_add(1);
    }
}

/// The four input identities this comparison was performed against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputIdentities {
    /// Exact-byte identity of the application specification.
    pub spec_sha256: Digest,
    /// Exact-byte identity of the observation plan.
    pub observation_plan_sha256: Digest,
    /// Exact-byte identity of the binding plan.
    pub binding_plan_sha256: Digest,
    /// Exact-byte identity of the observation artifact.
    pub observation_artifact_sha256: Digest,
}

/// The immutable binding record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingRecord {
    pub inputs: InputIdentities,
    pub contradiction: Contradiction,
    pub coverage: Coverage,
    pub claims: Vec<ClaimBinding>,
}

/// Serialized comparison with exact-byte identity.
///
/// There is no public constructor: a report can only come from a real comparison.
///
/// ```compile_fail
/// # use helm_bind::BindingReport;
/// fn forge(bytes: Vec<u8>, real: BindingReport) -> BindingReport {
///     BindingReport { bytes, ..real }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingReport {
    bytes: Vec<u8>,
    sha256: Digest,
    record: BindingRecord,
}

/// Escape a validated logical identifier. Roles and DLL names are already
/// restricted to lowercase ASCII alphanumerics, dot, underscore and hyphen, so
/// this cannot widen the output alphabet; it exists so the serializer stays total.
fn push_id(out: &mut String, id: &str) {
    out.push('"');
    for ch in id.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-') {
            out.push(ch);
        }
    }
    out.push('"');
}

impl BindingReport {
    pub(crate) fn new(record: BindingRecord) -> Self {
        // Fixed field order within schema 0.1. A deterministic serializer, not a
        // canonicalisation standard: reformatting produces a different identity.
        let mut s = String::with_capacity(2048);
        s.push_str("{\"schema\":\"helm-binding-report\",\"version\":\"0.1\",\"inputs\":{");
        s.push_str("\"spec_sha256\":\"");
        s.push_str(&record.inputs.spec_sha256.to_hex());
        s.push_str("\",\"observation_plan_sha256\":\"");
        s.push_str(&record.inputs.observation_plan_sha256.to_hex());
        s.push_str("\",\"binding_plan_sha256\":\"");
        s.push_str(&record.inputs.binding_plan_sha256.to_hex());
        s.push_str("\",\"observation_artifact_sha256\":\"");
        s.push_str(&record.inputs.observation_artifact_sha256.to_hex());
        s.push_str("\"},\"contradiction\":{\"state\":\"");
        s.push_str(record.contradiction.as_str());
        s.push('"');
        if let Contradiction::Contradicted { mismatches } = record.contradiction {
            s.push_str(",\"mismatches\":");
            s.push_str(&mismatches.to_string());
        }
        s.push_str("},\"coverage\":{");
        let c = &record.coverage;
        for (i, (name, value)) in [
            ("claims", c.claims),
            ("match", c.matched),
            ("mismatch", c.mismatched),
            ("desired_value_unspecified", c.desired_value_unspecified),
            ("not_observed", c.not_observed),
            ("unsupported_binding", c.unsupported_binding),
            ("absent", c.absent),
            ("observation_rejected", c.observation_rejected),
            ("observation_failed", c.observation_failed),
            ("observation_omitted", c.observation_omitted),
            (
                "observation_not_interpretable",
                c.observation_not_interpretable,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                s.push(',');
            }
            s.push('"');
            s.push_str(name);
            s.push_str("\":");
            s.push_str(&value.to_string());
        }
        s.push_str(",\"unsupported_classes\":[");
        for (i, class) in c.unsupported_classes.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('"');
            s.push_str(class);
            s.push('"');
        }
        s.push_str("]},\"claims\":[");
        for (i, claim) in record.claims.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("{\"claim\":\"");
            s.push_str(claim.subject.as_str());
            s.push('"');
            if let Some(role) = claim.subject.role() {
                s.push_str(",\"role\":");
                push_id(&mut s, role);
            }
            s.push_str(",\"state\":\"");
            s.push_str(claim.state.as_str());
            s.push('"');
            match claim.state {
                ClaimState::Mismatch(d) => {
                    s.push_str(",\"difference\":\"");
                    s.push_str(d.as_str());
                    s.push('"');
                }
                ClaimState::ObservationRejected(r) => {
                    s.push_str(",\"reason\":\"");
                    s.push_str(r.as_str());
                    s.push('"');
                }
                ClaimState::ObservationFailed(r) => {
                    s.push_str(",\"reason\":\"");
                    s.push_str(r.as_str());
                    s.push('"');
                }
                ClaimState::ObservationOmitted(r) => {
                    s.push_str(",\"reason\":\"");
                    s.push_str(r.as_str());
                    s.push('"');
                }
                _ => {}
            }
            s.push('}');
        }
        s.push_str("],\"origin\":\"desired_versus_observed_comparison_only\"}");
        let bytes = s.into_bytes();
        let sha256 = Digest::from_raw(Sha256::digest(&bytes).into());
        Self {
            bytes,
            sha256,
            record,
        }
    }

    /// Exact serialized bytes. Store unchanged; reformatting makes another report.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 of exactly [`Self::exact_bytes`].
    #[must_use]
    pub const fn sha256(&self) -> Digest {
        self.sha256
    }

    /// Whether any two known values disagreed.
    #[must_use]
    pub const fn contradiction(&self) -> Contradiction {
        self.record.contradiction
    }

    /// How much of the specification the comparison reached.
    #[must_use]
    pub const fn coverage(&self) -> &Coverage {
        &self.record.coverage
    }

    /// Every semantic claim outcome, in specification declaration order.
    #[must_use]
    pub fn claims(&self) -> &[ClaimBinding] {
        &self.record.claims
    }

    /// The four input identities this comparison was performed against.
    #[must_use]
    pub const fn inputs(&self) -> &InputIdentities {
        &self.record.inputs
    }

    /// Borrowed view of the whole record.
    #[must_use]
    pub const fn record(&self) -> &BindingRecord {
        &self.record
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BindingRecord, BindingReport, ClaimBinding, ClaimState, ClaimSubject, Contradiction,
        Coverage, Difference, FailureReason, InputIdentities, MAX_REPORT_BYTES, OmissionReason,
        RejectionReason,
    };
    use crate::plan::{Digest, MAX_ID_BYTES};
    use sha2::{Digest as _, Sha256};

    /// The serializer never truncates, so the widest report the schema and the
    /// declared limits can express must fit inside the stated ceiling. Prove it
    /// numerically rather than assuming it.
    #[test]
    fn the_widest_expressible_report_stays_inside_the_ceiling() {
        let widest = Digest::from_raw([0xff; 32]);
        let role = "d".repeat(MAX_ID_BYTES);
        let mut claims = Vec::new();
        // 1 source body, 16 runtime artifact bodies, 8 verification definitions,
        // 4 always-unsupported classes, 8 disabled DLLs, 2 entry-point claims: 39.
        claims.push(ClaimBinding::new(
            ClaimSubject::SourceBody,
            ClaimState::Mismatch(Difference::SizeAndDigest),
        ));
        claims.push(ClaimBinding::new(
            ClaimSubject::SourceArchitecture,
            ClaimState::UnsupportedBinding,
        ));
        claims.push(ClaimBinding::new(
            ClaimSubject::RuntimeFamily,
            ClaimState::UnsupportedBinding,
        ));
        for _ in 0..16 {
            claims.push(ClaimBinding::new(
                ClaimSubject::RuntimeArtifactBody(role.clone()),
                ClaimState::Mismatch(Difference::SizeAndDigest),
            ));
        }
        claims.push(ClaimBinding::new(
            ClaimSubject::WindowsArchitecture,
            ClaimState::UnsupportedBinding,
        ));
        claims.push(ClaimBinding::new(
            ClaimSubject::PrefixRole,
            ClaimState::UnsupportedBinding,
        ));
        for _ in 0..8 {
            claims.push(ClaimBinding::new(
                ClaimSubject::DisabledDll("d".repeat(64)),
                ClaimState::UnsupportedBinding,
            ));
        }
        claims.push(ClaimBinding::new(
            ClaimSubject::EntryPointPresence,
            ClaimState::Mismatch(Difference::EntryPointPath),
        ));
        claims.push(ClaimBinding::new(
            ClaimSubject::EntryPointBody,
            ClaimState::Mismatch(Difference::EntryPointPath),
        ));
        for _ in 0..8 {
            claims.push(ClaimBinding::new(
                ClaimSubject::VerificationDefinitionBody(role.clone()),
                ClaimState::Mismatch(Difference::SizeAndDigest),
            ));
        }
        assert_eq!(claims.len(), 39, "the maximum semantic claim count is 39");

        let mut coverage = Coverage {
            unsupported_classes: vec![
                "source_architecture",
                "runtime_family",
                "windows_architecture",
                "prefix_role",
                "disabled_dll",
            ],
            ..Coverage::default()
        };
        for claim in &claims {
            coverage.tally(claim.state());
        }
        let report = BindingReport::new(BindingRecord {
            inputs: InputIdentities {
                spec_sha256: widest,
                observation_plan_sha256: widest,
                binding_plan_sha256: widest,
                observation_artifact_sha256: widest,
            },
            contradiction: Contradiction::Contradicted { mismatches: 27 },
            coverage,
            claims,
        });
        assert!(
            report.exact_bytes().len() < MAX_REPORT_BYTES,
            "the widest report is {} bytes against a {MAX_REPORT_BYTES} byte ceiling",
            report.exact_bytes().len()
        );
    }

    /// A fixed record, serialized. The bytes and their digest must be identical on
    /// every platform: the serializer has no clock, no host value, no map iteration
    /// and no platform-dependent formatting. The expected digest below was measured,
    /// not invented, and cross-platform CI re-measures it on Linux, Windows and macOS.
    fn fixture_report() -> BindingReport {
        let d = |b: u8| Digest::from_raw([b; 32]);
        let claims = vec![
            ClaimBinding::new(ClaimSubject::SourceBody, ClaimState::Match),
            ClaimBinding::new(
                ClaimSubject::SourceArchitecture,
                ClaimState::UnsupportedBinding,
            ),
            ClaimBinding::new(ClaimSubject::RuntimeFamily, ClaimState::UnsupportedBinding),
            ClaimBinding::new(
                ClaimSubject::RuntimeArtifactBody("synthetic-runtime".to_owned()),
                ClaimState::Mismatch(Difference::SizeAndDigest),
            ),
            ClaimBinding::new(
                ClaimSubject::WindowsArchitecture,
                ClaimState::UnsupportedBinding,
            ),
            ClaimBinding::new(ClaimSubject::PrefixRole, ClaimState::UnsupportedBinding),
            ClaimBinding::new(ClaimSubject::EntryPointPresence, ClaimState::Absent),
            ClaimBinding::new(
                ClaimSubject::EntryPointBody,
                ClaimState::DesiredValueUnspecified,
            ),
            ClaimBinding::new(
                ClaimSubject::VerificationDefinitionBody("notes-contract".to_owned()),
                ClaimState::ObservationFailed(FailureReason::PermissionDenied),
            ),
        ];
        let mut coverage = Coverage {
            unsupported_classes: vec![
                "source_architecture",
                "runtime_family",
                "windows_architecture",
                "prefix_role",
            ],
            ..Coverage::default()
        };
        for claim in &claims {
            coverage.tally(claim.state());
        }
        BindingReport::new(BindingRecord {
            inputs: InputIdentities {
                spec_sha256: d(0x11),
                observation_plan_sha256: d(0x22),
                binding_plan_sha256: d(0x33),
                observation_artifact_sha256: d(0x44),
            },
            contradiction: Contradiction::from_states(claims.iter().map(ClaimBinding::state)),
            coverage,
            claims,
        })
    }

    /// The cross-platform determinism anchor. If this digest ever differs between
    /// runners, the serializer is not platform-independent and that is a defect,
    /// not a reason to record three expected values.
    #[test]
    fn the_fixture_report_has_one_platform_independent_identity() {
        let report = fixture_report();
        println!(
            "HELM-BIND-FIXTURE-REPORT: bytes={} sha256={}",
            report.exact_bytes().len(),
            report.sha256().to_hex()
        );
        assert_eq!(
            report.sha256().to_hex(),
            "d63d04e2a55a61fa149729f48bc355fb18f28e309f8aa2c973589391bf80e923",
            "report bytes were {}",
            String::from_utf8_lossy(report.exact_bytes())
        );
        // The digest is over exactly these bytes, computed independently here.
        let independent: [u8; 32] = Sha256::digest(report.exact_bytes()).into();
        assert_eq!(report.sha256().as_bytes(), &independent);
        assert_eq!(fixture_report().exact_bytes(), report.exact_bytes());
    }

    /// Privacy and vocabulary.
    ///
    /// Independent review correction: this test previously also asserted that the
    /// report bytes never *contain* `snapshot`, `verified`, `ready_to_launch` or
    /// `installed`. That is not an invariant of the module and cannot be, for two
    /// reasons. Validated role and DLL selectors are caller data written verbatim,
    /// and every one of those words is a legal `helm-app-spec` identifier; and the
    /// binder's own coverage key `observation_failed` contains `fail`.
    ///
    /// The property that does hold, and that this test now states, is that no term
    /// **the binder itself emits** is a verdict word, and that no selector can
    /// reach a slot the binder controls. The end-to-end case, with a specification
    /// whose selectors deliberately are verdict words, is
    /// `caller_selectors_reach_the_report_bytes_verbatim`.
    #[test]
    fn the_report_carries_no_verdict_vocabulary_and_no_path() {
        let report = fixture_report();
        let bytes = report.exact_bytes();
        assert!(
            !bytes.iter().any(u8::is_ascii_uppercase),
            "the report vocabulary is lowercase, so no uppercase verdict token can appear"
        );
        let text = String::from_utf8_lossy(bytes).into_owned();
        assert!(!text.contains('/'), "no path may appear: {text}");

        const VERDICT_WORDS: [&str; 11] = [
            "pass",
            "fail",
            "satisfied",
            "unsatisfied",
            "compatible",
            "ready",
            "ready_to_launch",
            "installed_correctly",
            "complete",
            "snapshot",
            "verified",
        ];
        // Every term the binder chooses for itself.
        let mut emitted: Vec<&str> = vec![
            "helm-binding-report",
            report.contradiction().as_str(),
            "desired_versus_observed_comparison_only",
        ];
        emitted.extend(report.claims().iter().map(|c| c.subject().as_str()));
        emitted.extend(report.claims().iter().map(|c| c.state().as_str()));
        emitted.extend(report.coverage().unsupported_classes.iter().copied());
        for term in emitted {
            assert!(
                !VERDICT_WORDS.contains(&term),
                "the binder emitted a verdict word as its own term: {term}"
            );
        }
        // No verdict word can reach a slot the binder controls, whatever a caller
        // names a role: `push_id` cannot emit a quote, so a selector cannot close
        // its string.
        for word in VERDICT_WORDS {
            assert!(!text.contains(&format!("\"state\":\"{word}\"")));
            assert!(!text.contains(&format!("\"claim\":\"{word}\"")));
        }
    }

    /// The contradiction axis depends on mismatches and on nothing else.
    #[test]
    fn only_a_mismatch_contradicts() {
        let others = [
            ClaimState::Match,
            ClaimState::DesiredValueUnspecified,
            ClaimState::NotObserved,
            ClaimState::UnsupportedBinding,
            ClaimState::Absent,
            ClaimState::ObservationRejected(RejectionReason::SpecialFile),
            ClaimState::ObservationFailed(FailureReason::IoFailure),
            ClaimState::ObservationOmitted(OmissionReason::FileLimit),
            ClaimState::ObservationNotInterpretable,
        ];
        assert_eq!(
            Contradiction::from_states(others.iter().copied()),
            Contradiction::NoClaimContradicted,
            "no state other than a mismatch may contradict"
        );

        // Every subset of the non-mismatch states, with and without mismatches.
        let differences = [
            Difference::Size,
            Difference::DigestValue,
            Difference::SizeAndDigest,
            Difference::EntryPointPath,
        ];
        for mask in 0u32..(1 << others.len()) {
            let mut states: Vec<ClaimState> = Vec::new();
            for (i, state) in others.iter().enumerate() {
                if mask & (1 << i) != 0 {
                    states.push(*state);
                }
            }
            assert_eq!(
                Contradiction::from_states(states.iter().copied()),
                Contradiction::NoClaimContradicted
            );
            for (n, difference) in differences.iter().enumerate() {
                let mut with = states.clone();
                for _ in 0..=n {
                    with.push(ClaimState::Mismatch(*difference));
                }
                let expected = u32::try_from(n + 1).unwrap_or(u32::MAX);
                assert_eq!(
                    Contradiction::from_states(with.iter().copied()),
                    Contradiction::Contradicted {
                        mismatches: expected
                    },
                    "a mismatch count must survive any accompanying states"
                );
            }
        }
    }

    /// Coverage counts partition the claim set exactly.
    #[test]
    fn coverage_counts_sum_to_the_claim_count() {
        let states = [
            ClaimState::Match,
            ClaimState::Mismatch(Difference::Size),
            ClaimState::DesiredValueUnspecified,
            ClaimState::NotObserved,
            ClaimState::UnsupportedBinding,
            ClaimState::Absent,
            ClaimState::ObservationRejected(RejectionReason::WrongKind),
            ClaimState::ObservationFailed(FailureReason::Unrecognised),
            ClaimState::ObservationOmitted(OmissionReason::TotalLimit),
            ClaimState::ObservationNotInterpretable,
        ];
        assert_eq!(states.len(), 10, "there are exactly ten claim states");
        let mut coverage = Coverage::default();
        for (i, state) in states.iter().enumerate() {
            for _ in 0..=i {
                coverage.tally(*state);
            }
        }
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
        assert_eq!(coverage.claims, 55);
        // Every state has its own counter: none shares a slot with another.
        assert_eq!(coverage.matched, 1);
        assert_eq!(coverage.observation_not_interpretable, 10);
    }
}
