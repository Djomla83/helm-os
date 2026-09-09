//! Untrusted binding-plan bytes to an inert `ValidatedBindingPlan`.
//!
//! Pure: no filesystem, process, network, environment or clock access happens
//! here, on any path including error paths. A validated mapping grants no
//! authority and carries no expected value; expected values always come from the
//! application specification.

use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::error::{BindingPlanError, BindingPlanErrorCode as C, BindingPlanErrors};

/// Maximum accepted binding-plan document size.
pub const MAX_PLAN_BYTES: usize = 16 * 1024;
/// Maximum mapping entries: 1 source, 16 runtime artifacts, 8 verification
/// definitions, entry-point presence and entry-point body.
pub const MAX_CLAIMS: usize = 27;
/// Maximum logical identifier length, matching the other HELM contracts.
pub const MAX_ID_BYTES: usize = 80;
/// Maximum JSON nesting accepted from an untrusted document.
pub const MAX_JSON_DEPTH: usize = 4;

const SCHEMA: &str = "helm-binding-plan";
const VERSION: &str = "0.1";

/// A 32-byte SHA-256 identity rendered as 64 lowercase hexadecimal characters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest([u8; 32]);

impl Digest {
    /// The 32 identity bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) const fn from_raw(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// 64 lowercase hexadecimal characters.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0 {
            s.push(char::from_digit(u32::from(b >> 4), 16).unwrap_or('0'));
            s.push(char::from_digit(u32::from(b & 0x0f), 16).unwrap_or('0'));
        }
        s
    }

    pub(crate) fn parse_hex(text: &str) -> Option<Self> {
        if text.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        let bytes = text.as_bytes();
        for (i, slot) in out.iter_mut().enumerate() {
            let (hi, lo) = (bytes[2 * i], bytes[2 * i + 1]);
            if hi.is_ascii_uppercase() || lo.is_ascii_uppercase() {
                return None;
            }
            let hi = (hi as char).to_digit(16)?;
            let lo = (lo as char).to_digit(16)?;
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

/// The closed set of mappable comparison domains.
///
/// Each kind fixes which desired field it reads and which observation
/// `Observable` can answer it. There is deliberately no generic
/// "compare field X to target Y" mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ClaimKind {
    /// Immutable application-source body identity.
    SourceBody,
    /// Immutable runtime-artifact body identity, per declared role.
    RuntimeArtifactBody,
    /// Frozen verification-definition body identity, per declared role.
    VerificationDefinitionBody,
    /// Existence and regular-file kind at the desired relative path.
    EntryPointPresence,
    /// Installed entry-point body identity.
    EntryPointBody,
}

impl ClaimKind {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceBody => "source_body",
            Self::RuntimeArtifactBody => "runtime_artifact_body",
            Self::VerificationDefinitionBody => "verification_definition_body",
            Self::EntryPointPresence => "entry_point_presence",
            Self::EntryPointBody => "entry_point_body",
        }
    }

    const fn parse(text: &str) -> Option<Self> {
        match text.as_bytes() {
            b"source_body" => Some(Self::SourceBody),
            b"runtime_artifact_body" => Some(Self::RuntimeArtifactBody),
            b"verification_definition_body" => Some(Self::VerificationDefinitionBody),
            b"entry_point_presence" => Some(Self::EntryPointPresence),
            b"entry_point_body" => Some(Self::EntryPointBody),
            _ => None,
        }
    }

    /// Whether this kind names one instance of a declared collection.
    #[must_use]
    pub const fn takes_role(self) -> bool {
        matches!(
            self,
            Self::RuntimeArtifactBody | Self::VerificationDefinitionBody
        )
    }

    /// Whether this kind concerns the entry point, and so needs a prefix-root assertion.
    #[must_use]
    pub const fn is_entry_point(self) -> bool {
        matches!(self, Self::EntryPointPresence | Self::EntryPointBody)
    }
}

/// One inert mapping from a typed desired claim to one observation target ID.
///
/// It carries no expected size, digest, predicate, operator, path or verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimMapping {
    kind: ClaimKind,
    role: Option<String>,
    target: String,
}

impl ClaimMapping {
    /// The typed comparison domain.
    #[must_use]
    pub const fn kind(&self) -> ClaimKind {
        self.kind
    }

    /// The declared role this mapping selects, for role-bearing kinds only.
    #[must_use]
    pub fn role(&self) -> Option<&str> {
        self.role.as_deref()
    }

    /// The observation target ID that answers this claim. Never a path.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
}

/// A validated, immutable binding plan. Construction is only possible through
/// [`parse_binding_plan`], so a caller cannot manufacture one.
///
/// ```compile_fail
/// # use helm_bind::ValidatedBindingPlan;
/// fn forge(real: ValidatedBindingPlan) -> ValidatedBindingPlan {
///     ValidatedBindingPlan { claims: Vec::new(), ..real }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedBindingPlan {
    bytes: Vec<u8>,
    sha256: Digest,
    subject_spec_sha256: Digest,
    asserted_prefix_root_id: Option<String>,
    claims: Vec<ClaimMapping>,
}

impl ValidatedBindingPlan {
    /// Exact accepted input bytes, retained unchanged.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 over [`Self::exact_bytes`]. No canonicalisation: whitespace, field
    /// order and JSON escaping differences produce a different mapping identity.
    #[must_use]
    pub const fn sha256(&self) -> Digest {
        self.sha256
    }

    /// The specification this mapping declares itself to be about.
    #[must_use]
    pub const fn subject_spec_sha256(&self) -> Digest {
        self.subject_spec_sha256
    }

    /// The root the **caller asserts** is the application's prefix.
    ///
    /// This is an assertion and never an attestation. It proves neither
    /// dedication, nor ownership, nor that the root belongs to this application,
    /// and no binding report upgrades it. It is present exactly when an
    /// entry-point claim is mapped.
    #[must_use]
    pub fn asserted_prefix_root_id(&self) -> Option<&str> {
        self.asserted_prefix_root_id.as_deref()
    }

    /// Mappings in declaration order.
    #[must_use]
    pub fn claims(&self) -> &[ClaimMapping] {
        &self.claims
    }

    pub(crate) fn mapping_for(&self, kind: ClaimKind, role: Option<&str>) -> Option<&ClaimMapping> {
        self.claims
            .iter()
            .find(|c| c.kind == kind && c.role.as_deref() == role)
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

/// Reject duplicate **decoded** object keys before any map insertion can hide one,
/// and bound nesting, at every object depth including objects nested in arrays.
///
/// Never deserialize straight into `serde_json::Value`: its ordinary object
/// visitor silently keeps the last duplicate. A raw byte scan is not enough either,
/// because JSON permits several spellings of one key. `helm-app-spec` and
/// `helm-observe` bound their own untrusted JSON the same way.
struct StrictScan<'a> {
    failure: &'a mut Option<C>,
    depth: usize,
}

impl StrictScan<'_> {
    fn reject<E: serde::de::Error>(failure: &mut Option<C>, code: C) -> E {
        *failure = Some(code);
        E::custom("bounded JSON rejected")
    }
}

impl<'de> serde::de::DeserializeSeed<'de> for StrictScan<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, parser: D) -> Result<(), D::Error> {
        parser.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for StrictScan<'_> {
    type Value = ();

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("a bounded binding-plan document")
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<(), E> {
        Ok(())
    }
    fn visit_bool<E: serde::de::Error>(self, _v: bool) -> Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: serde::de::Error>(self, _v: i64) -> Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: serde::de::Error>(self, _v: u64) -> Result<(), E> {
        Ok(())
    }
    fn visit_i128<E: serde::de::Error>(self, _v: i128) -> Result<(), E> {
        Ok(())
    }
    fn visit_u128<E: serde::de::Error>(self, _v: u128) -> Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: serde::de::Error>(self, _v: f64) -> Result<(), E> {
        Ok(())
    }
    fn visit_str<E: serde::de::Error>(self, _v: &str) -> Result<(), E> {
        Ok(())
    }

    fn visit_map<M: serde::de::MapAccess<'de>>(self, mut input: M) -> Result<(), M::Error> {
        if self.depth >= MAX_JSON_DEPTH {
            return Err(Self::reject(self.failure, C::NestingTooDeep));
        }
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        while let Some(key) = input.next_key::<String>()? {
            if !seen.insert(key) {
                return Err(Self::reject(self.failure, C::DuplicateKey));
            }
            input.next_value_seed(StrictScan {
                failure: &mut *self.failure,
                depth: self.depth + 1,
            })?;
        }
        Ok(())
    }

    fn visit_seq<S: serde::de::SeqAccess<'de>>(self, mut input: S) -> Result<(), S::Error> {
        if self.depth >= MAX_JSON_DEPTH {
            return Err(Self::reject(self.failure, C::NestingTooDeep));
        }
        while input
            .next_element_seed(StrictScan {
                failure: &mut *self.failure,
                depth: self.depth + 1,
            })?
            .is_some()
        {}
        Ok(())
    }
}

/// Structural admission for untrusted binding-plan bytes. Retains nothing.
fn scan_strict(bytes: &[u8]) -> Result<(), C> {
    let mut failure: Option<C> = None;
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let scanned = serde::de::DeserializeSeed::deserialize(
        StrictScan {
            failure: &mut failure,
            depth: 0,
        },
        &mut parser,
    );
    if scanned.is_err() {
        return Err(failure.unwrap_or(C::MalformedJson));
    }
    if parser.end().is_err() {
        return Err(C::MalformedJson);
    }
    Ok(())
}

fn expect_str<'a>(
    obj: &'a serde_json::Map<String, Value>,
    key: &'static str,
    out: &mut Vec<BindingPlanError>,
) -> Option<&'a str> {
    match obj.get(key) {
        None => {
            out.push(BindingPlanError::new(C::MissingField, key));
            None
        }
        Some(Value::String(s)) => Some(s.as_str()),
        Some(_) => {
            out.push(BindingPlanError::new(C::TypeMismatch, key));
            None
        }
    }
}

/// Parse and bound untrusted binding-plan bytes into an inert validated mapping.
///
/// # Errors
/// Returns every deterministically ordered validation finding. No filesystem or
/// other host access occurs on any path.
pub fn parse_binding_plan(bytes: &[u8]) -> Result<ValidatedBindingPlan, BindingPlanErrors> {
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(BindingPlanErrors::single(C::InputTooLarge, "plan"));
    }
    if let Err(code) = scan_strict(bytes) {
        return Err(BindingPlanErrors::single(code, "plan"));
    }

    let mut errors: Vec<BindingPlanError> = Vec::new();
    let value: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(_) => return Err(BindingPlanErrors::single(C::MalformedJson, "plan")),
    };
    let Value::Object(obj) = value else {
        return Err(BindingPlanErrors::single(C::NotJsonObject, "plan"));
    };

    const ALLOWED: [&str; 5] = [
        "schema",
        "version",
        "subject_spec_sha256",
        "asserted_prefix_root_id",
        "claims",
    ];
    for key in obj.keys() {
        if !ALLOWED.contains(&key.as_str()) {
            errors.push(BindingPlanError::new(C::UnknownField, "plan"));
        }
    }

    if let Some(s) = expect_str(&obj, "schema", &mut errors)
        && s != SCHEMA
    {
        errors.push(BindingPlanError::new(C::UnknownSchema, "schema"));
    }
    if let Some(v) = expect_str(&obj, "version", &mut errors)
        && v != VERSION
    {
        errors.push(BindingPlanError::new(C::UnknownVersion, "version"));
    }

    let mut subject = None;
    if let Some(text) = expect_str(&obj, "subject_spec_sha256", &mut errors) {
        match Digest::parse_hex(text) {
            Some(d) => subject = Some(d),
            None => errors.push(BindingPlanError::new(
                C::DigestGrammar,
                "subject_spec_sha256",
            )),
        }
    }

    let mut asserted_prefix_root_id: Option<String> = None;
    match obj.get("asserted_prefix_root_id") {
        None => {}
        Some(Value::String(s)) if valid_id(s) => asserted_prefix_root_id = Some(s.clone()),
        Some(Value::String(_)) => errors.push(BindingPlanError::new(
            C::IdGrammar,
            "asserted_prefix_root_id",
        )),
        Some(_) => errors.push(BindingPlanError::new(
            C::TypeMismatch,
            "asserted_prefix_root_id",
        )),
    }

    let mut claims: Vec<ClaimMapping> = Vec::new();
    match obj.get("claims") {
        None => errors.push(BindingPlanError::new(C::MissingField, "claims")),
        Some(Value::Array(items)) => {
            if items.is_empty() {
                errors.push(BindingPlanError::new(C::EmptyMapping, "claims"));
            }
            if items.len() > MAX_CLAIMS {
                errors.push(BindingPlanError::new(C::ClaimCountLimit, "claims"));
            }
            for (i, item) in items.iter().enumerate().take(MAX_CLAIMS) {
                let Value::Object(entry) = item else {
                    errors.push(BindingPlanError::at(C::TypeMismatch, "claims", i));
                    continue;
                };
                const ENTRY_ALLOWED: [&str; 3] = ["claim", "role", "target"];
                for key in entry.keys() {
                    if !ENTRY_ALLOWED.contains(&key.as_str()) {
                        errors.push(BindingPlanError::at(C::UnknownField, "claims", i));
                    }
                }
                let kind = match entry.get("claim") {
                    Some(Value::String(s)) => match ClaimKind::parse(s) {
                        Some(k) => Some(k),
                        None => {
                            errors.push(BindingPlanError::at(C::UnknownClaimKind, "claims", i));
                            None
                        }
                    },
                    Some(_) => {
                        errors.push(BindingPlanError::at(C::TypeMismatch, "claims", i));
                        None
                    }
                    None => {
                        errors.push(BindingPlanError::at(C::MissingField, "claims", i));
                        None
                    }
                };
                let role = match entry.get("role") {
                    None => None,
                    Some(Value::String(s)) if valid_id(s) => Some(s.clone()),
                    Some(Value::String(_)) => {
                        errors.push(BindingPlanError::at(C::IdGrammar, "claims", i));
                        None
                    }
                    Some(_) => {
                        errors.push(BindingPlanError::at(C::TypeMismatch, "claims", i));
                        None
                    }
                };
                let target = match entry.get("target") {
                    Some(Value::String(s)) if valid_id(s) => Some(s.clone()),
                    Some(Value::String(_)) => {
                        errors.push(BindingPlanError::at(C::IdGrammar, "claims", i));
                        None
                    }
                    Some(_) => {
                        errors.push(BindingPlanError::at(C::TypeMismatch, "claims", i));
                        None
                    }
                    None => {
                        errors.push(BindingPlanError::at(C::MissingField, "claims", i));
                        None
                    }
                };
                if let Some(kind) = kind {
                    let has_role = entry.contains_key("role");
                    if kind.takes_role() && !has_role {
                        errors.push(BindingPlanError::at(C::RoleRequired, "claims", i));
                    }
                    if !kind.takes_role() && has_role {
                        errors.push(BindingPlanError::at(C::RoleForbidden, "claims", i));
                    }
                    if let Some(target) = target {
                        let role = if kind.takes_role() { role } else { None };
                        if claims
                            .iter()
                            .any(|c| c.kind == kind && c.role.as_deref() == role.as_deref())
                        {
                            errors.push(BindingPlanError::at(C::DuplicateClaim, "claims", i));
                        }
                        claims.push(ClaimMapping { kind, role, target });
                    }
                }
            }
        }
        Some(_) => errors.push(BindingPlanError::new(C::TypeMismatch, "claims")),
    }

    // The prefix-root assertion exists exactly when an entry-point claim is mapped.
    let has_entry_claim = claims.iter().any(|c| c.kind.is_entry_point());
    if has_entry_claim && asserted_prefix_root_id.is_none() {
        errors.push(BindingPlanError::new(
            C::AssertedPrefixRootRequired,
            "asserted_prefix_root_id",
        ));
    }
    if !has_entry_claim && asserted_prefix_root_id.is_some() {
        errors.push(BindingPlanError::new(
            C::AssertedPrefixRootForbidden,
            "asserted_prefix_root_id",
        ));
    }

    if !errors.is_empty() {
        return Err(BindingPlanErrors::new(errors));
    }
    let Some(subject_spec_sha256) = subject else {
        return Err(BindingPlanErrors::single(
            C::MissingField,
            "subject_spec_sha256",
        ));
    };
    let sha256 = Digest(Sha256::digest(bytes).into());
    Ok(ValidatedBindingPlan {
        bytes: bytes.to_vec(),
        sha256,
        subject_spec_sha256,
        asserted_prefix_root_id,
        claims,
    })
}
