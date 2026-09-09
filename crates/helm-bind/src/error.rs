//! Bounded typed diagnostics. No raw untrusted text, no host path, no free-form
//! message carries machine semantics.

use core::fmt;

/// Maximum binding-plan findings retained in one report.
pub const MAX_PLAN_ERRORS: usize = 32;

/// Why an untrusted binding-plan document was rejected. Fixed codes only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BindingPlanErrorCode {
    InputTooLarge,
    NotJsonObject,
    MalformedJson,
    DuplicateKey,
    NestingTooDeep,
    UnknownField,
    MissingField,
    TypeMismatch,
    UnknownSchema,
    UnknownVersion,
    DigestGrammar,
    IdGrammar,
    UnknownClaimKind,
    /// The claim kind requires a `role` and none was supplied.
    RoleRequired,
    /// The claim kind takes no `role` and one was supplied.
    RoleForbidden,
    /// The same semantic claim key is mapped more than once.
    DuplicateClaim,
    ClaimCountLimit,
    EmptyMapping,
    /// An entry-point claim is mapped but `asserted_prefix_root_id` is absent.
    AssertedPrefixRootRequired,
    /// `asserted_prefix_root_id` is present although no entry-point claim is mapped.
    AssertedPrefixRootForbidden,
    ErrorLimit,
}

impl BindingPlanErrorCode {
    /// Stable machine-readable spelling. Never untrusted text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputTooLarge => "INPUT_TOO_LARGE",
            Self::NotJsonObject => "NOT_JSON_OBJECT",
            Self::MalformedJson => "MALFORMED_JSON",
            Self::DuplicateKey => "DUPLICATE_KEY",
            Self::NestingTooDeep => "NESTING_TOO_DEEP",
            Self::UnknownField => "UNKNOWN_FIELD",
            Self::MissingField => "MISSING_FIELD",
            Self::TypeMismatch => "TYPE_MISMATCH",
            Self::UnknownSchema => "UNKNOWN_SCHEMA",
            Self::UnknownVersion => "UNKNOWN_VERSION",
            Self::DigestGrammar => "DIGEST_GRAMMAR",
            Self::IdGrammar => "ID_GRAMMAR",
            Self::UnknownClaimKind => "UNKNOWN_CLAIM_KIND",
            Self::RoleRequired => "ROLE_REQUIRED",
            Self::RoleForbidden => "ROLE_FORBIDDEN",
            Self::DuplicateClaim => "DUPLICATE_CLAIM",
            Self::ClaimCountLimit => "CLAIM_COUNT_LIMIT",
            Self::EmptyMapping => "EMPTY_MAPPING",
            Self::AssertedPrefixRootRequired => "ASSERTED_PREFIX_ROOT_REQUIRED",
            Self::AssertedPrefixRootForbidden => "ASSERTED_PREFIX_ROOT_FORBIDDEN",
            Self::ErrorLimit => "ERROR_LIMIT",
        }
    }
}

impl fmt::Display for BindingPlanErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding-plan finding. `locator` is a field name from the closed schema.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingPlanError {
    code: BindingPlanErrorCode,
    locator: &'static str,
    index: Option<usize>,
}

impl BindingPlanError {
    pub(crate) const fn new(code: BindingPlanErrorCode, locator: &'static str) -> Self {
        Self {
            code,
            locator,
            index: None,
        }
    }

    pub(crate) const fn at(
        code: BindingPlanErrorCode,
        locator: &'static str,
        index: usize,
    ) -> Self {
        Self {
            code,
            locator,
            index: Some(index),
        }
    }

    /// Stable finding code.
    #[must_use]
    pub const fn code(&self) -> BindingPlanErrorCode {
        self.code
    }

    /// Closed-schema field name. Bounded and non-private.
    #[must_use]
    pub const fn locator(&self) -> &'static str {
        self.locator
    }

    /// Declaration index inside `claims`, when the finding is positional.
    #[must_use]
    pub const fn index(&self) -> Option<usize> {
        self.index
    }
}

impl fmt::Display for BindingPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.index {
            Some(i) => write!(f, "{} at {}[{}]", self.code, self.locator, i),
            None => write!(f, "{} at {}", self.code, self.locator),
        }
    }
}

/// Deterministically ordered binding-plan findings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingPlanErrors {
    errors: Vec<BindingPlanError>,
}

impl BindingPlanErrors {
    pub(crate) fn new(mut errors: Vec<BindingPlanError>) -> Self {
        errors.sort();
        errors.dedup();
        if errors.len() > MAX_PLAN_ERRORS {
            errors.truncate(MAX_PLAN_ERRORS - 1);
            errors.push(BindingPlanError::new(
                BindingPlanErrorCode::ErrorLimit,
                "plan",
            ));
        }
        Self { errors }
    }

    pub(crate) fn single(code: BindingPlanErrorCode, locator: &'static str) -> Self {
        Self {
            errors: vec![BindingPlanError::new(code, locator)],
        }
    }

    /// Findings in deterministic order.
    #[must_use]
    pub fn as_slice(&self) -> &[BindingPlanError] {
        &self.errors
    }

    /// Finding codes in deterministic order.
    #[must_use]
    pub fn codes(&self) -> Vec<BindingPlanErrorCode> {
        self.errors.iter().map(BindingPlanError::code).collect()
    }

    /// Whether a code is present.
    #[must_use]
    pub fn contains(&self, code: BindingPlanErrorCode) -> bool {
        self.errors.iter().any(|e| e.code == code)
    }
}

impl fmt::Display for BindingPlanErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.errors.iter().enumerate() {
            if i > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for BindingPlanErrors {}

/// Why a binding attempt was refused before any claim was evaluated.
///
/// A refusal means the four supplied documents cannot form a valid comparison at
/// all. It is never a desired-versus-observed contradiction, and it produces no
/// report, no claim outcomes and no report identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BindingRefusalCode {
    /// The binding plan declares a different subject specification.
    MappingSubjectSpecMismatch,
    /// The observation declares a different subject specification.
    ObservationSubjectSpecMismatch,
    /// The artifact was not produced by the supplied observation plan.
    ObservationPlanMismatch,
    /// A mapping names a target the observation plan does not declare.
    UnknownTarget,
    /// The asserted prefix root is not a root of the observation plan.
    UnknownRoot,
    /// A mapping names a role the specification does not declare.
    UnknownRole,
    /// The mapped target's observable cannot answer this typed claim.
    IncompatibleObservable,
    /// A mapped entry-point target does not sit under the asserted prefix root.
    EntryPointRootMismatch,
}

impl BindingRefusalCode {
    /// Stable machine-readable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MappingSubjectSpecMismatch => "MAPPING_SUBJECT_SPEC_MISMATCH",
            Self::ObservationSubjectSpecMismatch => "OBSERVATION_SUBJECT_SPEC_MISMATCH",
            Self::ObservationPlanMismatch => "OBSERVATION_PLAN_MISMATCH",
            Self::UnknownTarget => "UNKNOWN_TARGET",
            Self::UnknownRoot => "UNKNOWN_ROOT",
            Self::UnknownRole => "UNKNOWN_ROLE",
            Self::IncompatibleObservable => "INCOMPATIBLE_OBSERVABLE",
            Self::EntryPointRootMismatch => "ENTRY_POINT_ROOT_MISMATCH",
        }
    }
}

/// One typed refusal. Carries only validated logical identifiers, never a path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingRefusal {
    code: BindingRefusalCode,
    claim: Option<&'static str>,
    locator: Option<String>,
}

impl BindingRefusal {
    pub(crate) const fn new(code: BindingRefusalCode) -> Self {
        Self {
            code,
            claim: None,
            locator: None,
        }
    }

    pub(crate) fn about(
        code: BindingRefusalCode,
        claim: &'static str,
        locator: impl Into<String>,
    ) -> Self {
        Self {
            code,
            claim: Some(claim),
            locator: Some(locator.into()),
        }
    }

    /// Stable refusal code.
    #[must_use]
    pub const fn code(&self) -> BindingRefusalCode {
        self.code
    }

    /// The claim kind the refusal concerns, when it is claim-specific.
    #[must_use]
    pub const fn claim(&self) -> Option<&'static str> {
        self.claim
    }

    /// A validated logical identifier from one of the supplied documents.
    #[must_use]
    pub fn locator(&self) -> Option<&str> {
        self.locator.as_deref()
    }
}

impl fmt::Display for BindingRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code.as_str())?;
        if let Some(claim) = self.claim {
            write!(f, " for {claim}")?;
        }
        if let Some(locator) = &self.locator {
            write!(f, " ({locator})")?;
        }
        Ok(())
    }
}

impl std::error::Error for BindingRefusal {}
