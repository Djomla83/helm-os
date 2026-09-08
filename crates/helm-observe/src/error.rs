//! Bounded typed errors. No raw OS strings, no host absolute paths.

use core::fmt;

/// Maximum plan-validation errors retained in one report.
pub const MAX_PLAN_ERRORS: usize = 64;

/// Why an untrusted plan document was rejected. Fixed codes only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PlanErrorCode {
    InputTooLarge,
    NotJsonObject,
    MalformedJson,
    DuplicateKey,
    UnknownField,
    MissingField,
    UnknownSchema,
    UnknownVersion,
    TypeMismatch,
    DigestGrammar,
    IdGrammar,
    DuplicateId,
    UnknownRootReference,
    RootCountLimit,
    TargetCountLimit,
    EmptyPlan,
    UnknownObservable,
    PathAbsolute,
    PathEmptyComponent,
    PathDotComponent,
    PathParentComponent,
    PathBackslash,
    PathNulByte,
    PathNonPortableByte,
    PathLeadingSpace,
    PathTrailingSpaceOrDot,
    PathTooLong,
    PathComponentTooLong,
    PathComponentCount,
    NestingTooDeep,
    ErrorLimit,
}

impl PlanErrorCode {
    /// Stable machine-readable spelling. Never a host path or OS message.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputTooLarge => "INPUT_TOO_LARGE",
            Self::NotJsonObject => "NOT_JSON_OBJECT",
            Self::MalformedJson => "MALFORMED_JSON",
            Self::DuplicateKey => "DUPLICATE_KEY",
            Self::UnknownField => "UNKNOWN_FIELD",
            Self::MissingField => "MISSING_FIELD",
            Self::UnknownSchema => "UNKNOWN_SCHEMA",
            Self::UnknownVersion => "UNKNOWN_VERSION",
            Self::TypeMismatch => "TYPE_MISMATCH",
            Self::DigestGrammar => "DIGEST_GRAMMAR",
            Self::IdGrammar => "ID_GRAMMAR",
            Self::DuplicateId => "DUPLICATE_ID",
            Self::UnknownRootReference => "UNKNOWN_ROOT_REFERENCE",
            Self::RootCountLimit => "ROOT_COUNT_LIMIT",
            Self::TargetCountLimit => "TARGET_COUNT_LIMIT",
            Self::EmptyPlan => "EMPTY_PLAN",
            Self::UnknownObservable => "UNKNOWN_OBSERVABLE",
            Self::PathAbsolute => "PATH_ABSOLUTE",
            Self::PathEmptyComponent => "PATH_EMPTY_COMPONENT",
            Self::PathDotComponent => "PATH_DOT_COMPONENT",
            Self::PathParentComponent => "PATH_PARENT_COMPONENT",
            Self::PathBackslash => "PATH_BACKSLASH",
            Self::PathNulByte => "PATH_NUL_BYTE",
            Self::PathNonPortableByte => "PATH_NON_PORTABLE_BYTE",
            Self::PathLeadingSpace => "PATH_LEADING_SPACE",
            Self::PathTrailingSpaceOrDot => "PATH_TRAILING_SPACE_OR_DOT",
            Self::PathTooLong => "PATH_TOO_LONG",
            Self::PathComponentTooLong => "PATH_COMPONENT_TOO_LONG",
            Self::PathComponentCount => "PATH_COMPONENT_COUNT",
            Self::NestingTooDeep => "NESTING_TOO_DEEP",
            Self::ErrorLimit => "ERROR_LIMIT",
        }
    }
}

impl fmt::Display for PlanErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan-validation finding. `locator` is a validated logical ID or a bounded
/// field name from the closed schema, never a host path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlanError {
    code: PlanErrorCode,
    locator: &'static str,
    index: Option<usize>,
}

impl PlanError {
    pub(crate) const fn new(code: PlanErrorCode, locator: &'static str) -> Self {
        Self {
            code,
            locator,
            index: None,
        }
    }

    pub(crate) const fn at(code: PlanErrorCode, locator: &'static str, index: usize) -> Self {
        Self {
            code,
            locator,
            index: Some(index),
        }
    }

    #[must_use]
    pub const fn code(&self) -> PlanErrorCode {
        self.code
    }

    /// Closed-schema field or collection name. Bounded and non-private.
    #[must_use]
    pub const fn locator(&self) -> &'static str {
        self.locator
    }

    /// Declaration index inside its collection, when the finding is positional.
    #[must_use]
    pub const fn index(&self) -> Option<usize> {
        self.index
    }
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.index {
            Some(i) => write!(f, "{} at {}[{}]", self.code, self.locator, i),
            None => write!(f, "{} at {}", self.code, self.locator),
        }
    }
}

/// Deterministically ordered plan-validation findings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanErrors {
    errors: Vec<PlanError>,
}

impl PlanErrors {
    pub(crate) fn new(mut errors: Vec<PlanError>) -> Self {
        errors.sort();
        errors.dedup();
        if errors.len() > MAX_PLAN_ERRORS {
            errors.truncate(MAX_PLAN_ERRORS - 1);
            errors.push(PlanError::new(PlanErrorCode::ErrorLimit, "plan"));
        }
        Self { errors }
    }

    pub(crate) fn single(code: PlanErrorCode, locator: &'static str) -> Self {
        Self {
            errors: vec![PlanError::new(code, locator)],
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[PlanError] {
        &self.errors
    }

    #[must_use]
    pub fn codes(&self) -> Vec<PlanErrorCode> {
        self.errors.iter().map(PlanError::code).collect()
    }

    #[must_use]
    pub fn contains(&self, code: PlanErrorCode) -> bool {
        self.errors.iter().any(|e| e.code == code)
    }
}

impl fmt::Display for PlanErrors {
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

impl std::error::Error for PlanErrors {}

/// Why a capability or an authorisation was refused. No target access occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum AdmissionErrorCode {
    /// The descriptor is not a directory.
    RootNotDirectory,
    /// The root filesystem is outside the supported local-ext4 0.1 cohort.
    RootUnsupportedFilesystem,
    /// Root metadata could not be established.
    RootMetadataUnavailable,
    /// The supplied capability is not a procfs directory.
    ProcfsWrongFilesystem,
    /// The supplied directory does not map this process's descriptor numbers.
    ProcfsForeignOrUnusable,
    /// A plan root has no supplied capability.
    MissingRoot,
    /// A supplied capability is not declared by the plan.
    ExtraRoot,
    /// The same logical root ID was supplied more than once.
    DuplicateRoot,
    /// This build has no observation backend for the running platform.
    UnsupportedPlatform,
}

impl AdmissionErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RootNotDirectory => "ROOT_NOT_DIRECTORY",
            Self::RootUnsupportedFilesystem => "ROOT_UNSUPPORTED_FILESYSTEM",
            Self::RootMetadataUnavailable => "ROOT_METADATA_UNAVAILABLE",
            Self::ProcfsWrongFilesystem => "PROCFS_WRONG_FILESYSTEM",
            Self::ProcfsForeignOrUnusable => "PROCFS_FOREIGN_OR_UNUSABLE",
            Self::MissingRoot => "MISSING_ROOT",
            Self::ExtraRoot => "EXTRA_ROOT",
            Self::DuplicateRoot => "DUPLICATE_ROOT",
            Self::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
        }
    }
}

/// One admission failure, optionally naming the validated logical root ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionError {
    code: AdmissionErrorCode,
    root: Option<String>,
}

impl AdmissionError {
    pub(crate) const fn new(code: AdmissionErrorCode) -> Self {
        Self { code, root: None }
    }

    pub(crate) fn for_root(code: AdmissionErrorCode, root: &str) -> Self {
        Self {
            code,
            root: Some(root.to_owned()),
        }
    }

    #[must_use]
    pub const fn code(&self) -> AdmissionErrorCode {
        self.code
    }

    /// Validated logical root ID, never a host path.
    #[must_use]
    pub fn root_id(&self) -> Option<&str> {
        self.root.as_deref()
    }
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            Some(r) => write!(f, "{} for root {r}", self.code.as_str()),
            None => f.write_str(self.code.as_str()),
        }
    }
}

impl std::error::Error for AdmissionError {}

/// Deterministically ordered admission failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionErrors {
    errors: Vec<AdmissionError>,
}

impl AdmissionErrors {
    pub(crate) fn new(mut errors: Vec<AdmissionError>) -> Self {
        errors.sort_by(|a, b| (a.code, &a.root).cmp(&(b.code, &b.root)));
        Self { errors }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[AdmissionError] {
        &self.errors
    }

    #[must_use]
    pub fn codes(&self) -> Vec<AdmissionErrorCode> {
        self.errors.iter().map(AdmissionError::code).collect()
    }

    #[must_use]
    pub fn contains(&self, code: AdmissionErrorCode) -> bool {
        self.errors.iter().any(|e| e.code == code)
    }
}

impl fmt::Display for AdmissionErrors {
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

impl std::error::Error for AdmissionErrors {}
