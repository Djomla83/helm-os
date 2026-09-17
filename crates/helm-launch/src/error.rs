//! Bounded, machine-stable plan errors.
//!
//! Fixed codes only. No OS message string, host path, process or descriptor
//! number, and no byte of the rejected input ever appears in an error value or in
//! its `Display` output: a locator is a closed-schema field name chosen by this
//! crate, never text taken from the document.
//!
//! P1 carries only the plan-error family. The admission, authorisation and
//! process-creation families of the productization plan belong to APIs that do
//! not exist in P1, so they are not represented here.

use core::fmt;

/// Maximum plan-validation findings retained in one [`LaunchPlanErrors`].
pub const MAX_PLAN_ERRORS: usize = 64;

/// Why untrusted launch-plan bytes were rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LaunchPlanErrorCode {
    /// The input exceeds [`crate::MAX_PLAN_BYTES`]; nothing else was examined.
    InputTooLarge,
    /// Not well-formed JSON, trailing content, or a string that is not valid
    /// Unicode after escape decoding.
    MalformedJson,
    /// Two object keys decode to the same text, at any depth.
    DuplicateKey,
    /// Containers nest deeper than [`crate::MAX_JSON_DEPTH`].
    NestingTooDeep,
    /// The document is JSON but not a JSON object.
    NotJsonObject,
    /// A key outside the closed 0.1 schema.
    UnknownField,
    /// A required key is absent.
    MissingField,
    /// A value has the wrong JSON type, including a non-integer number.
    TypeMismatch,
    /// `schema` is not `helm-launch-plan`.
    UnknownSchema,
    /// `version` is not `0.1`.
    UnknownVersion,
    /// `execution_kind` is not a member of the closed 0.1 set.
    ExecutionKindUnsupported,
    /// `argv` has no element.
    ArgvEmpty,
    /// `argv` has more than [`crate::MAX_ARGS`] elements.
    ArgvTooMany,
    /// One `argv` element exceeds [`crate::MAX_ARG_BYTES`].
    ArgTooLong,
    /// The `argv` elements together exceed [`crate::MAX_ARGV_TOTAL_BYTES`].
    ArgvTooLarge,
    /// An `argv` element contains NUL after escape decoding.
    ArgContainsNul,
    /// `environment.mode` is not `empty`.
    EnvironmentModeUnsupported,
    /// `stdin.mode` is not `closed_pipe_eof`.
    StdinModeUnsupported,
    /// A `capture_prefix_bytes` value is outside `0..=MAX_CAPTURE_BYTES`.
    CaptureBoundOutOfRange,
    /// `timeout_ms` is outside `MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS`.
    TimeoutOutOfRange,
    /// `termination.grace_ms` is outside `0..=MAX_GRACE_MS`.
    GraceOutOfRange,
    /// `termination.signal` is not `SIGTERM`.
    TerminationSignalUnsupported,
    /// `working_directory.capability_id` violates the identifier grammar.
    IdGrammar,
    /// An asserted digest is not 64 lowercase hexadecimal characters.
    DigestGrammar,
    /// More findings existed than [`MAX_PLAN_ERRORS`]; the rest were dropped.
    ErrorLimit,
}

impl LaunchPlanErrorCode {
    /// Stable machine-readable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputTooLarge => "INPUT_TOO_LARGE",
            Self::MalformedJson => "MALFORMED_JSON",
            Self::DuplicateKey => "DUPLICATE_KEY",
            Self::NestingTooDeep => "NESTING_TOO_DEEP",
            Self::NotJsonObject => "NOT_JSON_OBJECT",
            Self::UnknownField => "UNKNOWN_FIELD",
            Self::MissingField => "MISSING_FIELD",
            Self::TypeMismatch => "TYPE_MISMATCH",
            Self::UnknownSchema => "UNKNOWN_SCHEMA",
            Self::UnknownVersion => "UNKNOWN_VERSION",
            Self::ExecutionKindUnsupported => "EXECUTION_KIND_UNSUPPORTED",
            Self::ArgvEmpty => "ARGV_EMPTY",
            Self::ArgvTooMany => "ARGV_TOO_MANY",
            Self::ArgTooLong => "ARG_TOO_LONG",
            Self::ArgvTooLarge => "ARGV_TOO_LARGE",
            Self::ArgContainsNul => "ARG_CONTAINS_NUL",
            Self::EnvironmentModeUnsupported => "ENVIRONMENT_MODE_UNSUPPORTED",
            Self::StdinModeUnsupported => "STDIN_MODE_UNSUPPORTED",
            Self::CaptureBoundOutOfRange => "CAPTURE_BOUND_OUT_OF_RANGE",
            Self::TimeoutOutOfRange => "TIMEOUT_OUT_OF_RANGE",
            Self::GraceOutOfRange => "GRACE_OUT_OF_RANGE",
            Self::TerminationSignalUnsupported => "TERMINATION_SIGNAL_UNSUPPORTED",
            Self::IdGrammar => "ID_GRAMMAR",
            Self::DigestGrammar => "DIGEST_GRAMMAR",
            Self::ErrorLimit => "ERROR_LIMIT",
        }
    }
}

impl fmt::Display for LaunchPlanErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan-validation finding.
///
/// `locator` is a closed-schema field path such as `stdout.capture_prefix_bytes`,
/// chosen by this crate. It never repeats a key or value from the document.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LaunchPlanError {
    code: LaunchPlanErrorCode,
    locator: &'static str,
    index: Option<usize>,
}

impl LaunchPlanError {
    pub(crate) const fn new(code: LaunchPlanErrorCode, locator: &'static str) -> Self {
        Self {
            code,
            locator,
            index: None,
        }
    }

    pub(crate) const fn at(code: LaunchPlanErrorCode, locator: &'static str, index: usize) -> Self {
        Self {
            code,
            locator,
            index: Some(index),
        }
    }

    /// The fixed finding code.
    #[must_use]
    pub const fn code(&self) -> LaunchPlanErrorCode {
        self.code
    }

    /// Closed-schema field path. Bounded and never document text.
    #[must_use]
    pub const fn locator(&self) -> &'static str {
        self.locator
    }

    /// Element index inside `argv`, when the finding is positional.
    #[must_use]
    pub const fn index(&self) -> Option<usize> {
        self.index
    }
}

impl fmt::Display for LaunchPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.index {
            Some(i) => write!(f, "{} at {}[{}]", self.code, self.locator, i),
            None => write!(f, "{} at {}", self.code, self.locator),
        }
    }
}

/// Deterministically ordered, de-duplicated and capped plan findings.
///
/// Order is by code, then locator, then index, so the same input yields the same
/// value on every platform and in every run.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LaunchPlanErrors {
    errors: Vec<LaunchPlanError>,
}

impl LaunchPlanErrors {
    pub(crate) fn new(mut errors: Vec<LaunchPlanError>) -> Self {
        errors.sort();
        errors.dedup();
        if errors.len() > MAX_PLAN_ERRORS {
            errors.truncate(MAX_PLAN_ERRORS - 1);
            errors.push(LaunchPlanError::new(
                LaunchPlanErrorCode::ErrorLimit,
                "plan",
            ));
        }
        Self { errors }
    }

    pub(crate) fn single(code: LaunchPlanErrorCode, locator: &'static str) -> Self {
        Self {
            errors: vec![LaunchPlanError::new(code, locator)],
        }
    }

    /// Every retained finding, in deterministic order. Never empty.
    #[must_use]
    pub fn as_slice(&self) -> &[LaunchPlanError] {
        &self.errors
    }

    /// The codes of [`Self::as_slice`], in the same order.
    #[must_use]
    pub fn codes(&self) -> Vec<LaunchPlanErrorCode> {
        self.errors.iter().map(LaunchPlanError::code).collect()
    }

    /// Whether any retained finding carries `code`.
    #[must_use]
    pub fn contains(&self, code: LaunchPlanErrorCode) -> bool {
        self.errors.iter().any(|e| e.code == code)
    }
}

impl fmt::Display for LaunchPlanErrors {
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

impl std::error::Error for LaunchPlanErrors {}

#[cfg(test)]
mod tests {
    use super::{LaunchPlanError, LaunchPlanErrorCode as C, LaunchPlanErrors, MAX_PLAN_ERRORS};

    #[test]
    fn findings_are_sorted_deduplicated_and_capped() {
        let mut raw = Vec::new();
        for i in (0..200).rev() {
            raw.push(LaunchPlanError::at(C::ArgTooLong, "argv", i));
            raw.push(LaunchPlanError::at(C::ArgTooLong, "argv", i));
        }
        raw.push(LaunchPlanError::new(C::UnknownField, "plan"));
        let errors = LaunchPlanErrors::new(raw);
        assert_eq!(errors.as_slice().len(), MAX_PLAN_ERRORS);
        assert_eq!(errors.codes().last(), Some(&C::ErrorLimit));
        let kept = &errors.as_slice()[..MAX_PLAN_ERRORS - 1];
        assert!(kept.windows(2).all(|w| w[0] < w[1]));
        // Code order is declaration order: UNKNOWN_FIELD sorts before ARG_TOO_LONG.
        assert_eq!(kept[0].code(), C::UnknownField);
        assert_eq!(kept[1].index(), Some(0));
    }

    #[test]
    fn display_is_bounded_and_machine_stable() {
        let e = LaunchPlanErrors::new(vec![
            LaunchPlanError::new(C::TimeoutOutOfRange, "timeout_ms"),
            LaunchPlanError::at(C::ArgContainsNul, "argv", 3),
        ]);
        assert_eq!(
            e.to_string(),
            "ARG_CONTAINS_NUL at argv[3]; TIMEOUT_OUT_OF_RANGE at timeout_ms"
        );
    }
}
