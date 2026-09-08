//! Untrusted plan bytes to an inert `ValidatedPlan`.
//!
//! Pure: no filesystem, process, network, environment, registry, package or Wine
//! lookup happens here, on any path including error paths. Validating a plan
//! grants no authority of any kind.

use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::error::{PlanError, PlanErrorCode as C, PlanErrors};

/// Maximum accepted plan document size.
pub const MAX_PLAN_BYTES: usize = 128 * 1024;
/// Maximum declared roots.
pub const MAX_ROOTS: usize = 8;
/// Maximum declared targets.
pub const MAX_TARGETS: usize = 64;
/// Maximum relative target path length in bytes.
pub const MAX_PATH_BYTES: usize = 1024;
/// Maximum path components.
pub const MAX_PATH_COMPONENTS: usize = 32;
/// Maximum bytes in one path component.
pub const MAX_COMPONENT_BYTES: usize = 255;
/// Maximum logical identifier length.
pub const MAX_ID_BYTES: usize = 80;
/// Maximum JSON nesting accepted from an untrusted document.
pub const MAX_JSON_DEPTH: usize = 8;

const SCHEMA: &str = "helm-observation-plan";
const VERSION: &str = "0.1";

/// A 32-byte SHA-256 identity rendered as 64 lowercase hexadecimal characters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest([u8; 32]);

impl Digest {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) const fn from_raw(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn to_hex(self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0 {
            // Two lowercase hexadecimal characters per byte.
            s.push(char::from_digit(u32::from(b >> 4), 16).unwrap_or('0'));
            s.push(char::from_digit(u32::from(b & 0x0f), 16).unwrap_or('0'));
        }
        s
    }

    fn parse_hex(text: &str) -> Option<Self> {
        if text.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        let bytes = text.as_bytes();
        for (i, slot) in out.iter_mut().enumerate() {
            let hi = (bytes[2 * i] as char).to_digit(16)?;
            let lo = (bytes[2 * i + 1] as char).to_digit(16)?;
            if bytes[2 * i].is_ascii_uppercase() || bytes[2 * i + 1].is_ascii_uppercase() {
                return None;
            }
            *slot = u8::try_from(hi * 16 + lo).ok()?;
        }
        Some(Self(out))
    }
}

impl core::fmt::Debug for Digest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl core::fmt::Display for Digest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// The observation operation a target requests. Constrains safe measurement
/// mechanics only; it is not a desired outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Observable {
    /// Metadata of the explicitly targeted directory object. Never enumerated.
    DirectoryMetadata,
    /// SHA-256 of a bounded stream from the explicitly targeted regular file.
    RegularFileSha256,
}

impl Observable {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DirectoryMetadata => "directory_metadata",
            Self::RegularFileSha256 => "regular_file_sha256",
        }
    }
}

/// One inert declared target. Holds no capability and opens nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    id: String,
    root: String,
    path: Option<String>,
    observable: Observable,
}

impl Target {
    /// Logical target identifier, carrying no built-in role semantics.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Logical root identifier this target is declared against.
    #[must_use]
    pub fn root(&self) -> &str {
        &self.root
    }

    /// Exact inert relative spelling, or `None` for the root directory itself.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    #[must_use]
    pub const fn observable(&self) -> Observable {
        self.observable
    }
}

/// A validated, immutable plan. Construction is only possible through
/// [`parse_plan`], so a caller cannot manufacture one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPlan {
    bytes: Vec<u8>,
    sha256: Digest,
    subject_spec_sha256: Digest,
    roots: Vec<String>,
    targets: Vec<Target>,
}

impl ValidatedPlan {
    /// Exact accepted input bytes, retained unchanged.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 over [`Self::exact_bytes`]. No canonicalisation: a formatting
    /// change produces a different plan artifact.
    #[must_use]
    pub const fn sha256(&self) -> Digest {
        self.sha256
    }

    /// Opaque subject context supplied by the caller. The observer never parses,
    /// validates or compares the document it names.
    #[must_use]
    pub const fn subject_spec_sha256(&self) -> Digest {
        self.subject_spec_sha256
    }

    /// Declared logical root IDs in declaration order.
    #[must_use]
    pub fn root_ids(&self) -> &[String] {
        &self.roots
    }

    /// Declared targets in declaration order.
    #[must_use]
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
}

fn valid_id(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_ID_BYTES {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-'))
}

/// Inert relative path grammar. Deliberately narrower than the Linux namespace.
fn validate_path(path: &str, index: usize, out: &mut Vec<PlanError>) {
    let push = |c, out: &mut Vec<PlanError>| out.push(PlanError::at(c, "targets.path", index));
    if path.as_bytes().contains(&0) {
        push(C::PathNulByte, out);
        return;
    }
    if path.len() > MAX_PATH_BYTES {
        push(C::PathTooLong, out);
    }
    if path.starts_with('/') {
        push(C::PathAbsolute, out);
    }
    if path.contains('\\') {
        push(C::PathBackslash, out);
    }
    let components: Vec<&str> = path.split('/').collect();
    if components.len() > MAX_PATH_COMPONENTS {
        push(C::PathComponentCount, out);
    }
    for c in &components {
        if c.is_empty() {
            push(C::PathEmptyComponent, out);
            continue;
        }
        if *c == "." {
            push(C::PathDotComponent, out);
        }
        if *c == ".." {
            push(C::PathParentComponent, out);
        }
        if c.len() > MAX_COMPONENT_BYTES {
            push(C::PathComponentTooLong, out);
        }
        if c.starts_with(' ') {
            push(C::PathLeadingSpace, out);
        }
        if c.ends_with(' ') || c.ends_with('.') {
            push(C::PathTrailingSpaceOrDot, out);
        }
        if !c.bytes().all(|b| (0x20..0x7f).contains(&b)) {
            push(C::PathNonPortableByte, out);
        }
    }
}

/// Reject duplicate object keys before any map insertion can hide one, and bound
/// nesting. `serde_json` keeps the last duplicate silently, which would let two
/// documents with different meaning validate identically.
fn scan_raw(bytes: &[u8], out: &mut Vec<PlanError>) {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    match serde_json::Value::deserialize(&mut de) {
        Ok(_) => {}
        Err(_) => out.push(PlanError::new(C::MalformedJson, "plan")),
    }
    // Structural duplicate-key and depth scan over the raw token stream.
    let mut depth = 0usize;
    let mut stack: Vec<Vec<String>> = Vec::new();
    let iter = bytes.iter().copied();
    let mut in_string = false;
    let mut escaped = false;
    let mut current = String::new();
    let mut expect_key = false;
    for b in iter {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
                if expect_key {
                    if let Some(keys) = stack.last_mut() {
                        if keys.contains(&current) {
                            out.push(PlanError::new(C::DuplicateKey, "plan"));
                        } else {
                            keys.push(current.clone());
                        }
                    }
                    expect_key = false;
                }
                current.clear();
            } else if expect_key {
                current.push(char::from(b));
            }
            continue;
        }
        match b {
            b'"' => {
                in_string = true;
            }
            b'{' => {
                depth += 1;
                if depth > MAX_JSON_DEPTH {
                    out.push(PlanError::new(C::NestingTooDeep, "plan"));
                    return;
                }
                stack.push(Vec::new());
                expect_key = true;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                stack.pop();
                expect_key = false;
            }
            b'[' => {
                depth += 1;
                if depth > MAX_JSON_DEPTH {
                    out.push(PlanError::new(C::NestingTooDeep, "plan"));
                    return;
                }
            }
            b']' => {
                depth = depth.saturating_sub(1);
            }
            b',' => {
                expect_key = !stack.is_empty() && depth == stack.len();
            }
            _ => {}
        }
    }
}

use serde::Deserialize as _;

fn expect_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
    out: &mut Vec<PlanError>,
) -> Option<&'a str> {
    match obj.get(key) {
        None => {
            out.push(PlanError::new(C::MissingField, key));
            None
        }
        Some(Value::String(s)) => Some(s.as_str()),
        Some(_) => {
            out.push(PlanError::new(C::TypeMismatch, key));
            None
        }
    }
}

/// Parse and bound untrusted plan bytes into an inert validated plan.
///
/// # Errors
/// Returns every deterministically ordered validation finding. No filesystem or
/// other host access occurs on any path.
pub fn parse_plan(bytes: &[u8]) -> Result<ValidatedPlan, PlanErrors> {
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(PlanErrors::single(C::InputTooLarge, "plan"));
    }
    let mut errors: Vec<PlanError> = Vec::new();
    scan_raw(bytes, &mut errors);
    if errors
        .iter()
        .any(|e| matches!(e.code(), C::MalformedJson | C::NestingTooDeep))
    {
        return Err(PlanErrors::new(errors));
    }

    let value: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(_) => return Err(PlanErrors::single(C::MalformedJson, "plan")),
    };
    let Value::Object(obj) = value else {
        return Err(PlanErrors::single(C::NotJsonObject, "plan"));
    };

    const ALLOWED: [&str; 5] = [
        "schema",
        "version",
        "subject_spec_sha256",
        "roots",
        "targets",
    ];
    for key in obj.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            errors.push(PlanError::new(C::UnknownField, "plan"));
        }
    }

    if let Some(s) = expect_str(&obj, "schema", &mut errors)
        && s != SCHEMA
    {
        errors.push(PlanError::new(C::UnknownSchema, "schema"));
    }
    if let Some(v) = expect_str(&obj, "version", &mut errors)
        && v != VERSION
    {
        errors.push(PlanError::new(C::UnknownVersion, "version"));
    }

    let mut subject = None;
    if let Some(text) = expect_str(&obj, "subject_spec_sha256", &mut errors) {
        match Digest::parse_hex(text) {
            Some(d) => subject = Some(d),
            None => errors.push(PlanError::new(C::DigestGrammar, "subject_spec_sha256")),
        }
    }

    let mut roots: Vec<String> = Vec::new();
    match obj.get("roots") {
        None => errors.push(PlanError::new(C::MissingField, "roots")),
        Some(Value::Array(items)) => {
            if items.len() > MAX_ROOTS {
                errors.push(PlanError::new(C::RootCountLimit, "roots"));
            }
            for (i, item) in items.iter().enumerate().take(MAX_ROOTS) {
                let Value::Object(r) = item else {
                    errors.push(PlanError::at(C::TypeMismatch, "roots", i));
                    continue;
                };
                for key in r.keys() {
                    if key != "id" {
                        errors.push(PlanError::at(C::UnknownField, "roots", i));
                    }
                }
                match r.get("id") {
                    Some(Value::String(id)) if valid_id(id) => {
                        if roots.contains(id) {
                            errors.push(PlanError::at(C::DuplicateId, "roots", i));
                        } else {
                            roots.push(id.clone());
                        }
                    }
                    Some(Value::String(_)) => errors.push(PlanError::at(C::IdGrammar, "roots", i)),
                    Some(_) => errors.push(PlanError::at(C::TypeMismatch, "roots", i)),
                    None => errors.push(PlanError::at(C::MissingField, "roots", i)),
                }
            }
        }
        Some(_) => errors.push(PlanError::new(C::TypeMismatch, "roots")),
    }

    let mut targets: Vec<Target> = Vec::new();
    match obj.get("targets") {
        None => errors.push(PlanError::new(C::MissingField, "targets")),
        Some(Value::Array(items)) => {
            if items.is_empty() {
                errors.push(PlanError::new(C::EmptyPlan, "targets"));
            }
            if items.len() > MAX_TARGETS {
                errors.push(PlanError::new(C::TargetCountLimit, "targets"));
            }
            for (i, item) in items.iter().enumerate().take(MAX_TARGETS) {
                let Value::Object(t) = item else {
                    errors.push(PlanError::at(C::TypeMismatch, "targets", i));
                    continue;
                };
                const T_ALLOWED: [&str; 4] = ["id", "root", "path", "observable"];
                for key in t.keys() {
                    if !T_ALLOWED.contains(&key.as_str()) {
                        errors.push(PlanError::at(C::UnknownField, "targets", i));
                    }
                }
                let id = match t.get("id") {
                    Some(Value::String(s)) if valid_id(s) => Some(s.clone()),
                    Some(Value::String(_)) => {
                        errors.push(PlanError::at(C::IdGrammar, "targets", i));
                        None
                    }
                    Some(_) => {
                        errors.push(PlanError::at(C::TypeMismatch, "targets", i));
                        None
                    }
                    None => {
                        errors.push(PlanError::at(C::MissingField, "targets", i));
                        None
                    }
                };
                let root = match t.get("root") {
                    Some(Value::String(s)) if valid_id(s) => Some(s.clone()),
                    Some(Value::String(_)) => {
                        errors.push(PlanError::at(C::IdGrammar, "targets", i));
                        None
                    }
                    Some(_) => {
                        errors.push(PlanError::at(C::TypeMismatch, "targets", i));
                        None
                    }
                    None => {
                        errors.push(PlanError::at(C::MissingField, "targets", i));
                        None
                    }
                };
                let path = match t.get("path") {
                    None => None,
                    Some(Value::String(s)) => {
                        validate_path(s, i, &mut errors);
                        Some(s.clone())
                    }
                    Some(_) => {
                        errors.push(PlanError::at(C::TypeMismatch, "targets", i));
                        None
                    }
                };
                let observable = match t.get("observable") {
                    Some(Value::String(s)) if s == "directory_metadata" => {
                        Some(Observable::DirectoryMetadata)
                    }
                    Some(Value::String(s)) if s == "regular_file_sha256" => {
                        Some(Observable::RegularFileSha256)
                    }
                    Some(Value::String(_)) => {
                        errors.push(PlanError::at(C::UnknownObservable, "targets", i));
                        None
                    }
                    Some(_) => {
                        errors.push(PlanError::at(C::TypeMismatch, "targets", i));
                        None
                    }
                    None => {
                        errors.push(PlanError::at(C::MissingField, "targets", i));
                        None
                    }
                };
                if let (Some(id), Some(root), Some(observable)) = (id, root, observable) {
                    if targets.iter().any(|t| t.id == id) {
                        errors.push(PlanError::at(C::DuplicateId, "targets", i));
                    }
                    if !roots.contains(&root) {
                        errors.push(PlanError::at(C::UnknownRootReference, "targets", i));
                    }
                    targets.push(Target {
                        id,
                        root,
                        path,
                        observable,
                    });
                }
            }
        }
        Some(_) => errors.push(PlanError::new(C::TypeMismatch, "targets")),
    }

    if !errors.is_empty() {
        return Err(PlanErrors::new(errors));
    }
    let Some(subject_spec_sha256) = subject else {
        return Err(PlanErrors::single(C::MissingField, "subject_spec_sha256"));
    };
    let sha256 = Digest(Sha256::digest(bytes).into());
    Ok(ValidatedPlan {
        bytes: bytes.to_vec(),
        sha256,
        subject_spec_sha256,
        roots,
        targets,
    })
}
