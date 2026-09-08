//! Pure validation of untrusted JSON into immutable **desired** application state.
//!
//! Success establishes only schema 0.1 declaration validity. It establishes no
//! installation, runtime selection, loader execution, prefix or entry-point
//! existence, verification result, compatibility, ownership or permission.
//! Paths and artifact references are inert data, never opened or resolved.
//! Normal parsing uses no I/O, environment, clock, randomness or host discovery.
//!
//! Document identity is SHA-256 of the exact supplied bytes, including whitespace
//! and field order. There is no canonicalization or model serialization.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod model;
mod parse;
mod validate;

pub use error::{ErrorCode, SpecError, SpecErrors};
pub use model::*;

/// The only supported document schema name.
pub const SCHEMA: &str = "helm-app-spec";
/// Experimental schema version, separate from crate and application versions.
pub const VERSION: &str = "0.1";
/// Maximum exact input length; checked before UTF-8 decoding, parsing or hashing.
pub const MAX_INPUT_BYTES: usize = 64 * 1024;

/// Validate an inert desired-state declaration, or return deterministic diagnostics.
///
/// All construction of validated models passes through this function. Syntax,
/// duplicate-key and parser-budget failures return one document-level diagnostic.
/// Semantic diagnostics follow schema order, then declaration order within arrays.
/// No supplied path/reference is read and no host property affects validation.
///
/// ```
/// use helm_app_spec::{parse_spec, ErrorCode};
/// let errors = parse_spec(b"").unwrap_err();
/// assert_eq!(errors.as_slice()[0].code(), ErrorCode::JsonInvalid);
/// ```
pub fn parse_spec(bytes: &[u8]) -> Result<ValidatedAppSpec, SpecErrors> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(SpecErrors::one(ErrorCode::InputTooLarge, "$"));
    }
    let value = parse::json(bytes)?;
    validate::spec(&value, bytes)
}
