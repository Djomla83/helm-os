//! Bounded, machine-stable plan errors.
//!
//! Fixed codes only. No OS message string, host path, process or descriptor
//! number, and no byte of the rejected input ever appears in an error value or in
//! its `Display` output: a locator is a closed-schema field name chosen by this
//! crate, never text taken from the document.
//!
//! P1 and P2 carry the plan-error, capability-admission and
//! authorisation-refusal families. The process-creation family of the
//! productization plan belongs to a `launch` function that does not exist, so
//! it is not represented here.
//!
//! The admission and refusal vocabularies are portable data: they carry no
//! descriptor, no platform authority and no `rustix` type, so they compile
//! everywhere even though their only producer, `crate::authority`, exists on
//! the Linux x86_64 cohort alone.

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

// ---------------------------------------------------------------- admission

/// Why a caller-supplied descriptor was **not** admitted as a capability.
///
/// Every code is a refusal of the accepted admission contract
/// (ADR-0024 section C, productization plan sections 6.2 and 6.3). A refusal
/// produces no capability, no authorisation and no receipt, and the refused
/// descriptor closes when the moved-in value drops.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AdmissionErrorCode {
    /// The descriptor's access mode was inspected and is unsuitable: `O_PATH`,
    /// through which nothing can be measured, or `O_WRONLY` or `O_RDWR`, which
    /// would also carry mutation authority. Only `O_RDONLY` is admitted.
    DescriptorModeUnsuitable,
    /// An executable descriptor does not refer to a regular file.
    NotRegularFile,
    /// A working-directory descriptor does not refer to a directory.
    NotDirectory,
    /// The object carries `S_ISUID` or `S_ISGID`. No privilege transition is
    /// attempted; admission refuses.
    SetIdBitsPresent,
    /// The first metadata sample reports a size above the accepted 512 MiB
    /// bound `MAX_EXECUTABLE_BYTES`, which exists on the Linux x86_64 cohort
    /// alongside its only producer. Refused before any byte of the body is
    /// read.
    ExecutableTooLarge,
    /// Fewer than 64 bytes are readable at offset 0, or the ELF magic is
    /// absent. This is where a `#!` script and every other non-ELF object
    /// stops.
    NotElf,
    /// The ELF magic is present but a field is outside the accepted cohort:
    /// `ELFCLASS64`, `ELFDATA2LSB`, `EM_X86_64`, and `e_type` either `ET_EXEC`
    /// or `ET_DYN`. An in-cohort header still promises nothing about the
    /// program.
    ElfNotInCohort,
    /// **The measurement protocol detected instability** while reading the
    /// object: a sampled field differed between the two metadata samples, or
    /// the byte count read differed from the first sample's size.
    ///
    /// This is the only claim the code makes. It never means that no mutation
    /// occurred, that the inode is immutable, that a snapshot exists, or that
    /// the measured bytes are the bytes any later execution would run.
    MeasurementInstabilityDetected,
    /// The descriptor's flags or metadata could not be obtained, or the
    /// operating system reported them in a form this crate cannot represent.
    /// The mode and the object kind are therefore unknown, so admission
    /// refuses rather than guessing.
    MetadataUnavailable,
    /// A positional read of the header or of the body failed.
    ReadFailed,
    /// The caller's logical working-directory identifier violates the
    /// identifier grammar `[a-z0-9][a-z0-9._-]{0,79}`.
    WorkingDirectoryIdInvalid,
}

impl AdmissionErrorCode {
    /// Stable machine-readable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DescriptorModeUnsuitable => "DESCRIPTOR_MODE_UNSUITABLE",
            Self::NotRegularFile => "NOT_REGULAR_FILE",
            Self::NotDirectory => "NOT_DIRECTORY",
            Self::SetIdBitsPresent => "SET_ID_BITS_PRESENT",
            Self::ExecutableTooLarge => "EXECUTABLE_TOO_LARGE",
            Self::NotElf => "NOT_ELF",
            Self::ElfNotInCohort => "ELF_NOT_IN_COHORT",
            Self::MeasurementInstabilityDetected => "MEASUREMENT_INSTABILITY_DETECTED",
            Self::MetadataUnavailable => "METADATA_UNAVAILABLE",
            Self::ReadFailed => "READ_FAILED",
            Self::WorkingDirectoryIdInvalid => "WORKING_DIRECTORY_ID_INVALID",
        }
    }
}

impl fmt::Display for AdmissionErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable spelling of an operating-system error number, from a **closed table
/// owned by this crate**.
///
/// The numbers are the Linux generic values used by the x86_64 cohort, the only
/// platform on which any producer of an [`AdmissionError`] exists. The table
/// covers the numbers the accepted admission operations can report; any other
/// number yields `None` while [`AdmissionError::errno_number`] still reports it.
/// No `strerror` text, no operating-system message and no localised string is
/// ever consulted or emitted.
const fn errno_spelling(number: i32) -> Option<&'static str> {
    Some(match number {
        1 => "EPERM",
        2 => "ENOENT",
        4 => "EINTR",
        5 => "EIO",
        6 => "ENXIO",
        9 => "EBADF",
        11 => "EAGAIN",
        12 => "ENOMEM",
        13 => "EACCES",
        14 => "EFAULT",
        16 => "EBUSY",
        19 => "ENODEV",
        20 => "ENOTDIR",
        21 => "EISDIR",
        22 => "EINVAL",
        23 => "ENFILE",
        24 => "EMFILE",
        26 => "ETXTBSY",
        27 => "EFBIG",
        28 => "ENOSPC",
        29 => "ESPIPE",
        30 => "EROFS",
        38 => "ENOSYS",
        40 => "ELOOP",
        75 => "EOVERFLOW",
        95 => "EOPNOTSUPP",
        116 => "ESTALE",
        122 => "EDQUOT",
        _ => return None,
    })
}

/// A capability-admission refusal: a fixed code and, where one is relevant, a
/// bounded operating-system error number.
///
/// The value carries **no authority**. It never carries the refused descriptor,
/// so a refusal cannot hand back a capability, and it never carries a host
/// path, a descriptor number, an identifier supplied by the caller or any byte
/// of the inspected object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdmissionError {
    code: AdmissionErrorCode,
    errno: Option<i32>,
}

impl AdmissionError {
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "off the Linux x86_64 cohort no admission producer is compiled"
        )
    )]
    pub(crate) const fn new(code: AdmissionErrorCode) -> Self {
        Self { code, errno: None }
    }

    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "off the Linux x86_64 cohort no admission producer is compiled"
        )
    )]
    pub(crate) const fn with_errno(code: AdmissionErrorCode, errno: i32) -> Self {
        Self {
            code,
            errno: Some(errno),
        }
    }

    /// The fixed refusal code.
    #[must_use]
    pub const fn code(&self) -> AdmissionErrorCode {
        self.code
    }

    /// The operating-system error number, when the refusal came from a failed
    /// operating-system operation rather than from a policy decision.
    ///
    /// A policy refusal such as [`AdmissionErrorCode::SetIdBitsPresent`] has
    /// no error number.
    #[must_use]
    pub const fn errno_number(&self) -> Option<i32> {
        self.errno
    }

    /// Stable symbolic spelling of [`Self::errno_number`], from the closed
    /// table this crate owns.
    ///
    /// `None` means either that there is no error number or that this crate
    /// has no stable spelling for it. The number itself always remains
    /// available.
    #[must_use]
    pub const fn errno_name(&self) -> Option<&'static str> {
        match self.errno {
            Some(number) => errno_spelling(number),
            None => None,
        }
    }
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.code.fmt(f)?;
        match (self.errno_name(), self.errno) {
            (Some(name), _) => write!(f, " errno {name}"),
            (None, Some(number)) => write!(f, " errno {number}"),
            (None, None) => Ok(()),
        }
    }
}

impl std::error::Error for AdmissionError {}

// ------------------------------------------------------------ authorisation

/// Why composing a plan with two capabilities was refused.
///
/// The 0.1 authorisation checks exactly one relation, so this vocabulary has
/// exactly one member. No binding state, observation result or application
/// specification verdict participates: `authorize` cannot read one, because the
/// crate links no HELM crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AuthorizationRefusalCode {
    /// The plan's `working_directory.capability_id` differs from the admitted
    /// working-directory capability's logical identifier.
    WorkingDirectoryIdMismatch,
}

impl AuthorizationRefusalCode {
    /// Stable machine-readable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkingDirectoryIdMismatch => "WORKING_DIRECTORY_ID_MISMATCH",
        }
    }
}

impl fmt::Display for AuthorizationRefusalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An authorisation refusal, raised before any process could exist.
///
/// The value carries **no authority**: the consumed plan and both consumed
/// capabilities are dropped, so their descriptors close, and neither is handed
/// back. No descriptor number, host path or caller identifier appears in the
/// value or in its `Display` output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorizationRefusal {
    code: AuthorizationRefusalCode,
}

impl AuthorizationRefusal {
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "off the Linux x86_64 cohort no authorisation producer is compiled"
        )
    )]
    pub(crate) const fn new(code: AuthorizationRefusalCode) -> Self {
        Self { code }
    }

    /// The fixed refusal code.
    #[must_use]
    pub const fn code(&self) -> AuthorizationRefusalCode {
        self.code
    }
}

impl fmt::Display for AuthorizationRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.code.fmt(f)
    }
}

impl std::error::Error for AuthorizationRefusal {}

#[cfg(test)]
mod tests {
    use super::{
        AdmissionError, AdmissionErrorCode as A, AuthorizationRefusal,
        AuthorizationRefusalCode as R, LaunchPlanError, LaunchPlanErrorCode as C, LaunchPlanErrors,
        MAX_PLAN_ERRORS, errno_spelling,
    };

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

    /// Every admission code, so the spellings below are exhaustive.
    const ADMISSION_CODES: [A; 11] = [
        A::DescriptorModeUnsuitable,
        A::NotRegularFile,
        A::NotDirectory,
        A::SetIdBitsPresent,
        A::ExecutableTooLarge,
        A::NotElf,
        A::ElfNotInCohort,
        A::MeasurementInstabilityDetected,
        A::MetadataUnavailable,
        A::ReadFailed,
        A::WorkingDirectoryIdInvalid,
    ];

    #[test]
    fn admission_spellings_are_the_accepted_vocabulary() {
        let spellings: Vec<&str> = ADMISSION_CODES.iter().map(|c| c.as_str()).collect();
        assert_eq!(
            spellings,
            [
                "DESCRIPTOR_MODE_UNSUITABLE",
                "NOT_REGULAR_FILE",
                "NOT_DIRECTORY",
                "SET_ID_BITS_PRESENT",
                "EXECUTABLE_TOO_LARGE",
                "NOT_ELF",
                "ELF_NOT_IN_COHORT",
                "MEASUREMENT_INSTABILITY_DETECTED",
                "METADATA_UNAVAILABLE",
                "READ_FAILED",
                "WORKING_DIRECTORY_ID_INVALID",
            ]
        );
        // Machine-stable: upper snake case only, and one spelling per code.
        for spelling in &spellings {
            assert!(
                spelling
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b == b'_'),
                "{spelling}"
            );
        }
        let mut unique = spellings.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), spellings.len());
        assert_eq!(
            R::WorkingDirectoryIdMismatch.as_str(),
            "WORKING_DIRECTORY_ID_MISMATCH"
        );
    }

    #[test]
    fn admission_display_is_bounded_and_carries_no_host_text() {
        assert_eq!(
            AdmissionError::new(A::SetIdBitsPresent).to_string(),
            "SET_ID_BITS_PRESENT"
        );
        assert_eq!(
            AdmissionError::with_errno(A::ReadFailed, 5).to_string(),
            "READ_FAILED errno EIO"
        );
        // An unlisted number is still reported, as a number, never as OS text.
        let exotic = AdmissionError::with_errno(A::MetadataUnavailable, 4_242);
        assert_eq!(exotic.errno_name(), None);
        assert_eq!(exotic.errno_number(), Some(4_242));
        assert_eq!(exotic.to_string(), "METADATA_UNAVAILABLE errno 4242");
        assert_eq!(
            AuthorizationRefusal::new(R::WorkingDirectoryIdMismatch).to_string(),
            "WORKING_DIRECTORY_ID_MISMATCH"
        );
    }

    #[test]
    fn a_policy_refusal_carries_no_error_number() {
        for code in ADMISSION_CODES {
            let policy = AdmissionError::new(code);
            assert_eq!(policy.code(), code);
            assert_eq!(policy.errno_number(), None);
            assert_eq!(policy.errno_name(), None);
            assert_eq!(policy.to_string(), code.as_str());
        }
    }

    #[test]
    fn the_errno_table_is_closed_and_ascending() {
        // A crate-owned table: strictly ascending, uppercase `E` spellings, no
        // duplicate spelling, and nothing outside it.
        let mut previous: Option<i32> = None;
        let mut names = Vec::new();
        for number in -5..200 {
            if let Some(name) = errno_spelling(number) {
                assert!(previous < Some(number), "{number} out of order");
                previous = Some(number);
                assert!(name.starts_with('E'), "{name}");
                assert!(name.bytes().all(|b| b.is_ascii_uppercase()), "{name}");
                names.push(name);
            }
        }
        assert_eq!(errno_spelling(0), None);
        assert_eq!(errno_spelling(-1), None);
        assert_eq!(errno_spelling(i32::MIN), None);
        assert_eq!(errno_spelling(i32::MAX), None);
        assert_eq!(errno_spelling(9), Some("EBADF"));
        assert_eq!(errno_spelling(13), Some("EACCES"));
        assert_eq!(errno_spelling(75), Some("EOVERFLOW"));
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), names.len());
        assert_eq!(names.len(), 28);
    }
}
