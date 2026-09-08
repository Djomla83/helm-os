use std::fmt;

/// Stable experimental machine-readable categories; no parser error text is exposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// Input exceeds 64 KiB.
    InputTooLarge,
    /// Invalid JSON, UTF-8, number encoding or trailing input.
    JsonInvalid,
    /// Structural JSON budget exceeded before semantic validation.
    JsonLimit,
    /// An object repeats a decoded JSON key, even with the same value.
    FieldDuplicate,
    /// A closed schema object contains an unknown field.
    FieldUnknown,
    /// A required field is absent.
    FieldMissing,
    /// A field has the wrong JSON type (including explicit null).
    FieldType,
    /// Unsupported schema name.
    SchemaUnknown,
    /// Unsupported schema version.
    SchemaVersion,
    /// Unsupported value in the narrow typed vocabulary.
    ValueUnsupported,
    /// Identifier violates its length or character grammar.
    IdentifierInvalid,
    /// Digest is not exactly 64 lowercase hexadecimal characters.
    DigestInvalid,
    /// Metadata is empty, excessive or outside printable ASCII.
    MetadataInvalid,
    /// Size is not a positive bounded JSON integer.
    SizeInvalid,
    /// Collection cardinality is outside its schema bound.
    CollectionLimit,
    /// Runtime artifact role is repeated.
    DuplicateRuntimeArtifact,
    /// Frozen definition role is repeated.
    DuplicateVerificationDefinition,
    /// Disabled DLL basename is repeated.
    DuplicateDll,
    /// Disabled DLL basename violates the narrow grammar.
    DllInvalid,
    /// Entry-point spelling is unsafe or outside the portable subset.
    PathUnsafe,
    /// First 63 diagnostics retained; additional semantic errors were omitted.
    ErrorLimit,
}

impl ErrorCode {
    /// Stable code string, independent of Rust's Debug representation.
    pub fn as_str(self) -> &'static str {
        self.details().0
    }

    fn details(self) -> (&'static str, &'static str) {
        match self {
            Self::InputTooLarge => ("INPUT_TOO_LARGE", "Input exceeds 65536 bytes."),
            Self::JsonInvalid => (
                "JSON_INVALID",
                "Input must be one valid UTF-8 JSON document.",
            ),
            Self::JsonLimit => (
                "JSON_LIMIT",
                "JSON structure exceeds a fixed parser budget.",
            ),
            Self::FieldDuplicate => ("FIELD_DUPLICATE", "An object repeats a decoded JSON key."),
            Self::FieldUnknown => (
                "FIELD_UNKNOWN",
                "Object contains a field outside the closed schema.",
            ),
            Self::FieldMissing => ("FIELD_MISSING", "Required field is absent."),
            Self::FieldType => (
                "FIELD_TYPE",
                "Field has the wrong JSON type; null is not supported.",
            ),
            Self::SchemaUnknown => ("SCHEMA_UNKNOWN", "Schema must be helm-app-spec."),
            Self::SchemaVersion => ("SCHEMA_VERSION", "Only schema version 0.1 is supported."),
            Self::ValueUnsupported => (
                "VALUE_UNSUPPORTED",
                "Value is outside the schema 0.1 vocabulary.",
            ),
            Self::IdentifierInvalid => (
                "IDENTIFIER_INVALID",
                "Identifier must match [a-z0-9][a-z0-9._-]{0,79}.",
            ),
            Self::DigestInvalid => (
                "DIGEST_INVALID",
                "SHA-256 must contain exactly 64 lowercase hex digits.",
            ),
            Self::MetadataInvalid => (
                "METADATA_INVALID",
                "Metadata must contain 1-256 printable ASCII bytes.",
            ),
            Self::SizeInvalid => (
                "SIZE_INVALID",
                "Size must be a JSON integer from 1 through 1099511627776 bytes.",
            ),
            Self::CollectionLimit => (
                "COLLECTION_LIMIT",
                "Collection count is outside its schema bound.",
            ),
            Self::DuplicateRuntimeArtifact => (
                "DUPLICATE_RUNTIME_ARTIFACT",
                "Runtime artifact roles must be unique.",
            ),
            Self::DuplicateVerificationDefinition => (
                "DUPLICATE_VERIFICATION_DEFINITION",
                "Verification definition roles must be unique.",
            ),
            Self::DuplicateDll => ("DUPLICATE_DLL", "Disabled DLL basenames must be unique."),
            Self::DllInvalid => (
                "DLL_INVALID",
                "DLL basename must match [a-z][a-z0-9_]{0,63}, without extension.",
            ),
            Self::PathUnsafe => (
                "PATH_UNSAFE",
                "Entry point must use bounded portable prefix-relative drive_c path spelling.",
            ),
            Self::ErrorLimit => (
                "ERROR_LIMIT",
                "Additional semantic errors omitted after the first 63.",
            ),
        }
    }
}

/// Bounded diagnostic. Locations contain only fixed schema names and array indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecError {
    code: ErrorCode,
    location: String,
}

impl SpecError {
    /// Machine-readable error category.
    pub fn code(&self) -> ErrorCode {
        self.code
    }
    /// Logical location, for example `$.runtime.artifacts[0].sha256`.
    pub fn location(&self) -> &str {
        &self.location
    }
    /// Fixed explanation; never echoes attacker-controlled field names or values.
    pub fn explanation(&self) -> &'static str {
        self.code.details().1
    }
}

/// Nonempty deterministic diagnostics, at most 64 entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecErrors(pub(crate) Vec<SpecError>);

impl SpecErrors {
    /// Read-only diagnostics in their defined order.
    pub fn as_slice(&self) -> &[SpecError] {
        &self.0
    }

    pub(crate) fn one(code: ErrorCode, location: &str) -> Self {
        let mut errors = Self(Vec::new());
        errors.push(code, location);
        errors
    }

    pub(crate) fn push(&mut self, code: ErrorCode, location: &str) {
        if self.0.len() < 63 {
            self.0.push(SpecError {
                code,
                location: location.to_owned(),
            });
        } else if self.0.len() == 63 {
            self.0.push(SpecError {
                code: ErrorCode::ErrorLimit,
                location: "$".to_owned(),
            });
        }
    }
}

impl fmt::Display for SpecErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index != 0 {
                f.write_str("; ")?;
            }
            write!(
                f,
                "{} at {}: {}",
                error.code.as_str(),
                error.location,
                error.explanation()
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for SpecErrors {}
