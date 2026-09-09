//! The comparison. A pure function of four already-validated values.
//!
//! Structure matters for review. Every cross-document precondition is checked
//! **before** any claim is evaluated, so a refusal can never leave a partly bound
//! report behind. The claim universe is enumerated from the specification alone,
//! independently of any observation, so its order, its size and the unavoidable
//! unsupported classes can be proved without one. Each typed comparator selects
//! its own desired field, so a caller cannot compare a runtime archive under a
//! source or entry-point domain.

use helm_app_spec::ValidatedAppSpec;
use helm_observe::{
    BudgetReason, Failure, Observable, ObservationArtifact, Rejection, TargetOutcome, ValidatedPlan,
};

use crate::error::{BindingRefusal, BindingRefusalCode as R};
use crate::model::{
    BindingRecord, BindingReport, ClaimBinding, ClaimState, ClaimSubject, Contradiction, Coverage,
    Difference, FailureReason, InputIdentities, OmissionReason, RejectionReason,
};
use crate::plan::{ClaimKind, Digest, ValidatedBindingPlan};

/// Every semantic desired claim in one specification, in declaration order.
///
/// Contextual and human metadata — the specification digest, the application ID,
/// the application version, artifact labels — and role selector keys are
/// deliberately absent: they receive no outcome and are not part of coverage.
///
/// This depends on the specification only. It is what makes the coverage theorem
/// checkable without an observation: every valid specification necessarily yields
/// `SourceArchitecture`, `RuntimeFamily`, `WindowsArchitecture` and `PrefixRole`,
/// none of which has a comparator in 0.1.
pub(crate) fn semantic_claim_subjects(spec: &ValidatedAppSpec) -> Vec<ClaimSubject> {
    let mut subjects = vec![
        ClaimSubject::SourceBody,
        ClaimSubject::SourceArchitecture,
        ClaimSubject::RuntimeFamily,
    ];
    subjects.extend(
        spec.runtime()
            .artifacts()
            .iter()
            .map(|a| ClaimSubject::RuntimeArtifactBody(a.role().as_str().to_owned())),
    );
    subjects.push(ClaimSubject::WindowsArchitecture);
    subjects.push(ClaimSubject::PrefixRole);
    subjects.extend(
        spec.environment()
            .disabled_dlls()
            .iter()
            .map(|d| ClaimSubject::DisabledDll(d.clone())),
    );
    subjects.push(ClaimSubject::EntryPointPresence);
    subjects.push(ClaimSubject::EntryPointBody);
    subjects.extend(
        spec.verification()
            .iter()
            .map(|d| ClaimSubject::VerificationDefinitionBody(d.role().as_str().to_owned())),
    );
    subjects
}

/// The claim classes with no comparator in 0.1 that this specification exhibits.
pub(crate) fn unsupported_classes(spec: &ValidatedAppSpec) -> Vec<&'static str> {
    let mut classes = vec![
        "source_architecture",
        "runtime_family",
        "windows_architecture",
        "prefix_role",
    ];
    if !spec.environment().disabled_dlls().is_empty() {
        classes.push("disabled_dll");
    }
    classes
}

/// Compare explicitly mapped desired claims with the observations actually made.
///
/// Pure: no filesystem, descriptor, network, process, environment, clock or
/// randomness is touched, and the same four inputs give byte-identical output on
/// every platform.
///
/// The result carries **no** satisfaction, compatibility or readiness verdict. It
/// reports one typed outcome per semantic desired claim, whether any two known
/// values disagreed, and how much of the specification the comparison reached.
///
/// # Errors
/// Returns a typed [`BindingRefusal`] when the four documents cannot form a valid
/// comparison at all. A refusal produces no report, no claim outcomes and no
/// report identity; nothing is partly bound first.
pub fn bind(
    spec: &ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    observation_plan: &ValidatedPlan,
    observation: &ObservationArtifact,
) -> Result<BindingReport, BindingRefusal> {
    precheck(spec, mapping, observation_plan, observation)?;

    let mut coverage = Coverage::default();
    let mut claims: Vec<ClaimBinding> = Vec::new();
    for subject in semantic_claim_subjects(spec) {
        let state = evaluate(&subject, spec, mapping, observation_plan, observation);
        coverage.tally(state);
        claims.push(ClaimBinding::new(subject, state));
    }
    coverage.unsupported_classes = unsupported_classes(spec);

    let contradiction = Contradiction::from_states(claims.iter().map(ClaimBinding::state));

    Ok(BindingReport::new(BindingRecord {
        inputs: InputIdentities {
            // Equal to the specification identity: the prechecks established that.
            spec_sha256: mapping.subject_spec_sha256(),
            observation_plan_sha256: from_observe(&observation_plan.sha256()),
            binding_plan_sha256: mapping.sha256(),
            observation_artifact_sha256: from_observe(&observation.sha256()),
        },
        contradiction,
        coverage,
        claims,
    }))
}

/// What follows for one semantic claim. Precedence is explicit here so a result
/// can never vary by implementation accident.
fn evaluate(
    subject: &ClaimSubject,
    spec: &ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    observation_plan: &ValidatedPlan,
    observation: &ObservationArtifact,
) -> ClaimState {
    match subject {
        // No comparator exists in 0.1. Never mapped, never compared.
        ClaimSubject::SourceArchitecture
        | ClaimSubject::RuntimeFamily
        | ClaimSubject::WindowsArchitecture
        | ClaimSubject::PrefixRole
        | ClaimSubject::DisabledDll(_) => ClaimState::UnsupportedBinding,

        ClaimSubject::SourceBody => {
            let source = spec.application().source();
            body_claim(
                mapping,
                ClaimKind::SourceBody,
                None,
                source.size(),
                source.sha256().as_str(),
                observation,
            )
        }
        ClaimSubject::RuntimeArtifactBody(role) => spec
            .runtime()
            .artifacts()
            .iter()
            .find(|a| a.role().as_str() == role)
            .map_or(ClaimState::UnsupportedBinding, |artifact| {
                body_claim(
                    mapping,
                    ClaimKind::RuntimeArtifactBody,
                    Some(role),
                    artifact.size(),
                    artifact.sha256().as_str(),
                    observation,
                )
            }),
        ClaimSubject::VerificationDefinitionBody(role) => spec
            .verification()
            .iter()
            .find(|d| d.role().as_str() == role)
            .map_or(ClaimState::UnsupportedBinding, |definition| {
                body_claim(
                    mapping,
                    ClaimKind::VerificationDefinitionBody,
                    Some(role),
                    definition.size(),
                    definition.sha256().as_str(),
                    observation,
                )
            }),
        ClaimSubject::EntryPointPresence => {
            entry_point_presence(spec, mapping, observation_plan, observation)
        }
        ClaimSubject::EntryPointBody => {
            entry_point_body(spec, mapping, observation_plan, observation)
        }
    }
}

fn from_observe(digest: &helm_observe::Digest) -> Digest {
    Digest::from_raw(*digest.as_bytes())
}

/// Every cross-document precondition, in a fixed order, before any claim exists.
fn precheck(
    spec: &ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    observation_plan: &ValidatedPlan,
    observation: &ObservationArtifact,
) -> Result<(), BindingRefusal> {
    let spec_hex = spec.spec_sha256().as_str();
    if mapping.subject_spec_sha256().to_hex() != spec_hex {
        return Err(BindingRefusal::new(R::MappingSubjectSpecMismatch));
    }
    if observation.record().subject_spec_sha256.to_hex() != spec_hex {
        return Err(BindingRefusal::new(R::ObservationSubjectSpecMismatch));
    }
    if observation.record().plan_sha256 != observation_plan.sha256() {
        return Err(BindingRefusal::new(R::ObservationPlanMismatch));
    }
    if let Some(root) = mapping.asserted_prefix_root_id()
        && !observation_plan.root_ids().iter().any(|r| r == root)
    {
        return Err(BindingRefusal::about(
            R::UnknownRoot,
            "asserted_prefix_root_id",
            root,
        ));
    }

    for claim in mapping.claims() {
        let kind = claim.kind();
        let Some(target) = observation_plan
            .targets()
            .iter()
            .find(|t| t.id() == claim.target())
        else {
            return Err(BindingRefusal::about(
                R::UnknownTarget,
                kind.as_str(),
                claim.target(),
            ));
        };
        if let Some(role) = claim.role()
            && !role_is_declared(spec, kind, role)
        {
            return Err(BindingRefusal::about(R::UnknownRole, kind.as_str(), role));
        }
        // Every comparator in 0.1 needs the regular-file observable: a regular file
        // under `directory_metadata` is rejected as a wrong kind, so that observable
        // could never answer any of these claims.
        if target.observable() != Observable::RegularFileSha256 {
            return Err(BindingRefusal::about(
                R::IncompatibleObservable,
                kind.as_str(),
                claim.target(),
            ));
        }
        if kind.is_entry_point() && Some(target.root()) != mapping.asserted_prefix_root_id() {
            return Err(BindingRefusal::about(
                R::EntryPointRootMismatch,
                kind.as_str(),
                target.root(),
            ));
        }
    }
    Ok(())
}

fn role_is_declared(spec: &ValidatedAppSpec, kind: ClaimKind, role: &str) -> bool {
    match kind {
        ClaimKind::RuntimeArtifactBody => spec
            .runtime()
            .artifacts()
            .iter()
            .any(|a| a.role().as_str() == role),
        ClaimKind::VerificationDefinitionBody => spec
            .verification()
            .iter()
            .any(|d| d.role().as_str() == role),
        _ => false,
    }
}

/// Find the observation of one mapped target. The prechecks already established
/// that the target exists in the plan, but an artifact records only the targets
/// the observation actually walked, so a missing entry is possible in principle.
fn outcome_for<'a>(
    observation: &'a ObservationArtifact,
    target_id: &str,
) -> Option<&'a TargetOutcome> {
    observation
        .record()
        .targets
        .iter()
        .find(|t| t.target_id == target_id)
        .map(|t| &t.outcome)
}

/// Immutable-body identity: both the declared length and the declared digest.
///
/// A match establishes the identity of **that body** and nothing else: not
/// installation, not provisioning, not loader origin, not execution.
fn body_claim(
    mapping: &ValidatedBindingPlan,
    kind: ClaimKind,
    role: Option<&str>,
    desired_size: u64,
    desired_digest: &str,
    observation: &ObservationArtifact,
) -> ClaimState {
    let Some(entry) = mapping.mapping_for(kind, role) else {
        return ClaimState::NotObserved;
    };
    let Some(outcome) = outcome_for(observation, entry.target()) else {
        return ClaimState::NotObserved;
    };
    match outcome {
        TargetOutcome::ObservedFile(facts) => {
            let size_agrees = facts.bytes_read == desired_size;
            let digest_agrees = facts.sha256.to_hex() == desired_digest;
            match (size_agrees, digest_agrees) {
                (true, true) => ClaimState::Match,
                (false, true) => ClaimState::Mismatch(Difference::Size),
                (true, false) => ClaimState::Mismatch(Difference::DigestValue),
                (false, false) => ClaimState::Mismatch(Difference::SizeAndDigest),
            }
        }
        other => non_file_state(other),
    }
}

/// Translate an outcome that produced no readable body, without collapsing any
/// two distinct meanings into one. A failure never becomes a mismatch, an absence
/// never becomes a failure, and an unknown outcome never becomes agreement.
fn non_file_state(outcome: &TargetOutcome) -> ClaimState {
    match outcome {
        // A directory answer cannot reach a regular-file claim through a validated
        // mapping, because the observable is checked first. Kept explicit so a
        // future observer change cannot make it silently benign.
        TargetOutcome::ObservedDirectory(_) => {
            ClaimState::ObservationRejected(RejectionReason::WrongKind)
        }
        TargetOutcome::Absent => ClaimState::Absent,
        TargetOutcome::Rejected { rejection, .. } => {
            ClaimState::ObservationRejected(rejection_reason(*rejection))
        }
        TargetOutcome::Failed { failure, .. } => {
            ClaimState::ObservationFailed(failure_reason(*failure))
        }
        TargetOutcome::NotObserved(reason) => {
            ClaimState::ObservationOmitted(omission_reason(*reason))
        }
        // `TargetOutcome` is `#[non_exhaustive]`. An outcome this build does not
        // know is never agreement and never benign.
        _ => ClaimState::ObservationNotInterpretable,
    }
}

fn rejection_reason(rejection: Rejection) -> RejectionReason {
    match rejection {
        Rejection::SymlinkForbidden => RejectionReason::SymlinkForbidden,
        Rejection::SpecialFile => RejectionReason::SpecialFile,
        Rejection::WrongKind => RejectionReason::WrongKind,
        Rejection::ScopeViolation => RejectionReason::ScopeViolation,
        _ => RejectionReason::Unrecognised,
    }
}

fn failure_reason(failure: Failure) -> FailureReason {
    match failure {
        Failure::PermissionDenied => FailureReason::PermissionDenied,
        Failure::IoFailure => FailureReason::IoFailure,
        Failure::ResolutionRace => FailureReason::ResolutionRace,
        Failure::ChangedDuringRead => FailureReason::ChangedDuringRead,
        Failure::ReopenUnavailable => FailureReason::ReopenUnavailable,
        Failure::UnsupportedPlatform => FailureReason::UnsupportedPlatform,
        _ => FailureReason::Unrecognised,
    }
}

fn omission_reason(reason: BudgetReason) -> OmissionReason {
    match reason {
        BudgetReason::FileLimit => OmissionReason::FileLimit,
        BudgetReason::TotalLimit => OmissionReason::TotalLimit,
        BudgetReason::NotAttemptedTotalLimit => OmissionReason::NotAttemptedTotalLimit,
        _ => OmissionReason::Unrecognised,
    }
}

/// Whether the mapped target's declared relative path is the desired entry point.
///
/// Exact bytes. No normalisation, no case folding, and never a judgement drawn
/// from the spelling of a target ID.
fn entry_path_agrees(
    spec: &ValidatedAppSpec,
    observation_plan: &ValidatedPlan,
    target_id: &str,
) -> bool {
    observation_plan
        .targets()
        .iter()
        .find(|t| t.id() == target_id)
        .and_then(helm_observe::Target::path)
        == Some(spec.entry_point().path().as_str())
}

/// Existence and regular-file kind at the desired relative path, under the root
/// the caller asserted to be the prefix. Never executability, PE validity, Wine
/// loadability, correct installation or launchability.
fn entry_point_presence(
    spec: &ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    observation_plan: &ValidatedPlan,
    observation: &ObservationArtifact,
) -> ClaimState {
    let Some(entry) = mapping.mapping_for(ClaimKind::EntryPointPresence, None) else {
        return ClaimState::NotObserved;
    };
    if !entry_path_agrees(spec, observation_plan, entry.target()) {
        return ClaimState::Mismatch(Difference::EntryPointPath);
    }
    let Some(outcome) = outcome_for(observation, entry.target()) else {
        return ClaimState::NotObserved;
    };
    match outcome {
        // A digest is not needed to establish presence.
        TargetOutcome::ObservedFile(_) => ClaimState::Match,
        other => non_file_state(other),
    }
}

/// Installed entry-point body identity, only when the specification states one.
///
/// Precedence, fixed here so a result cannot vary by implementation accident:
///
/// 1. A specification with no desired entry-point digest yields
///    `DesiredValueUnspecified` **first**, before mapping and before path binding.
///    The specification made no claim, so there is nothing to contradict, and
///    promoting a path difference here would raise a contradiction for a claim the
///    specification never stated. A path difference is still reported by the
///    presence claim when that is mapped.
/// 2. Otherwise an unmapped claim is `NotObserved`.
/// 3. Otherwise a differing target path is `Mismatch`, before the observation
///    outcome is interpreted at all.
/// 4. Otherwise the observation outcome decides.
///
/// The specification declares no desired entry-point length, so none is invented:
/// only the digest is compared.
fn entry_point_body(
    spec: &ValidatedAppSpec,
    mapping: &ValidatedBindingPlan,
    observation_plan: &ValidatedPlan,
    observation: &ObservationArtifact,
) -> ClaimState {
    let Some(desired) = spec.entry_point().sha256() else {
        return ClaimState::DesiredValueUnspecified;
    };
    let Some(entry) = mapping.mapping_for(ClaimKind::EntryPointBody, None) else {
        return ClaimState::NotObserved;
    };
    if !entry_path_agrees(spec, observation_plan, entry.target()) {
        return ClaimState::Mismatch(Difference::EntryPointPath);
    }
    let Some(outcome) = outcome_for(observation, entry.target()) else {
        return ClaimState::NotObserved;
    };
    match outcome {
        TargetOutcome::ObservedFile(facts) => {
            if facts.sha256.to_hex() == desired.as_str() {
                ClaimState::Match
            } else {
                ClaimState::Mismatch(Difference::DigestValue)
            }
        }
        other => non_file_state(other),
    }
}

#[cfg(test)]
mod tests {
    use super::{semantic_claim_subjects, unsupported_classes};
    use crate::model::ClaimSubject;

    const NOTES: &[u8] = include_bytes!("../tests/fixtures/synthetic-notes.json");
    const A0: &[u8] = include_bytes!("../tests/fixtures/a0-7zip.json");

    /// The claim universe depends on the specification alone, so the coverage
    /// theorem is provable without any observation, on every platform.
    #[test]
    fn every_specification_carries_at_least_four_unsupported_semantic_claims() {
        for raw in [NOTES, A0] {
            let Ok(spec) = helm_app_spec::parse_spec(raw) else {
                unreachable!("committed fixtures parse")
            };
            let subjects = semantic_claim_subjects(&spec);
            let unsupported = subjects.iter().filter(|s| !s.is_supported()).count();
            assert!(
                unsupported >= 4,
                "every valid specification has at least four unsupported claims, got {unsupported}"
            );
            for required in [
                ClaimSubject::SourceArchitecture,
                ClaimSubject::RuntimeFamily,
                ClaimSubject::WindowsArchitecture,
                ClaimSubject::PrefixRole,
            ] {
                assert!(
                    subjects.contains(&required),
                    "{} is mandatory and unsupported",
                    required.as_str()
                );
            }
            assert!(
                subjects.len() <= 39,
                "the maximum semantic claim count is 39, got {}",
                subjects.len()
            );
            let classes = unsupported_classes(&spec);
            assert_eq!(classes[0], "source_architecture");
            assert_eq!(classes[1], "runtime_family");
            assert_eq!(classes[2], "windows_architecture");
            assert_eq!(classes[3], "prefix_role");
        }
    }

    /// Declaration order is fixed, so two runs and two platforms never disagree.
    #[test]
    fn claim_order_is_fixed_and_excludes_contextual_metadata() {
        let Ok(spec) = helm_app_spec::parse_spec(NOTES) else {
            unreachable!("committed fixture parses")
        };
        let order: Vec<&str> = semantic_claim_subjects(&spec)
            .iter()
            .map(ClaimSubject::as_str)
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
        // No outcome exists for the specification digest, the application ID, the
        // application version or an artifact label.
        assert!(!order.iter().any(|s| s.contains("application")));
        assert!(!order.iter().any(|s| s.contains("label")));
        assert!(!order.iter().any(|s| s.contains("spec_sha256")));
    }
}
