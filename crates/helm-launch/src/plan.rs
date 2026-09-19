//! Untrusted launch-plan bytes to an inert [`ValidatedLaunchPlan`].
//!
//! Pure. No filesystem, process, network, environment, clock or registry access
//! happens here, on any path including error paths. A validated plan is caller
//! intent: it holds no descriptor, names no executable, and grants no authority
//! of any kind.

use std::collections::BTreeMap;

use crate::error::{LaunchPlanError, LaunchPlanErrorCode as C, LaunchPlanErrors};
use crate::model::{CapabilityId, Digest, EnvironmentMode, Stream};

/// Maximum accepted plan document size in bytes, checked before parsing.
pub const MAX_PLAN_BYTES: usize = 32_768;
/// Maximum `argv` elements.
pub const MAX_ARGS: usize = 64;
/// Maximum bytes in one `argv` element, after escape decoding.
pub const MAX_ARG_BYTES: usize = 4_096;
/// Maximum sum of the byte lengths of all `argv` elements, after escape decoding.
pub const MAX_ARGV_TOTAL_BYTES: usize = 131_072;
/// Maximum `capture_prefix_bytes` per stream.
pub const MAX_CAPTURE_BYTES: u32 = 65_536;
/// Smallest accepted `timeout_ms`.
pub const MIN_TIMEOUT_MS: u32 = 1;
/// Largest accepted `timeout_ms`.
pub const MAX_TIMEOUT_MS: u32 = 600_000;
/// Largest accepted `termination.grace_ms`.
pub const MAX_GRACE_MS: u32 = 60_000;
/// Maximum container nesting accepted from an untrusted document. The 0.1
/// schema itself needs two levels.
pub const MAX_JSON_DEPTH: usize = 8;

const SCHEMA: &str = "helm-launch-plan";
const VERSION: &str = "0.1";

/// The kind of execution a plan asks for. 0.1 has exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExecutionKind {
    /// One already-open, admitted Linux executable descriptor. The plan never
    /// names it.
    LinuxExactExecutable,
}

impl ExecutionKind {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LinuxExactExecutable => "linux_exact_executable",
        }
    }
}

/// The stdin arrangement a plan asks for. 0.1 has exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StdinMode {
    /// A pipe whose write end is closed, so the reader sees immediate EOF.
    ClosedPipeEof,
}

impl StdinMode {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClosedPipeEof => "closed_pipe_eof",
        }
    }
}

/// The first termination signal a plan asks for. 0.1 has exactly one, and
/// `SIGKILL` always follows the grace period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TerminationSignal {
    /// `SIGTERM`.
    Sigterm,
}

impl TerminationSignal {
    /// Stable spelling, as written in the plan.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sigterm => "SIGTERM",
        }
    }
}

/// A validated, immutable launch plan: caller intent and nothing more.
///
/// Construction is only possible through [`parse_launch_plan`]. There is no
/// `Default`, setter, `From`, `Serialize` or `Deserialize`.
///
/// ```compile_fail
/// fn forge(real: helm_launch::ValidatedLaunchPlan) -> helm_launch::ValidatedLaunchPlan {
///     helm_launch::ValidatedLaunchPlan { argv: Vec::new(), ..real }
/// }
/// ```
///
/// ```compile_fail
/// let _ = helm_launch::ValidatedLaunchPlan::default();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedLaunchPlan {
    bytes: Vec<u8>,
    sha256: Digest,
    execution_kind: ExecutionKind,
    argv: Vec<String>,
    environment_mode: EnvironmentMode,
    working_directory_id: CapabilityId,
    stdin_mode: StdinMode,
    stdout_capture_prefix_bytes: u32,
    stderr_capture_prefix_bytes: u32,
    timeout_ms: u32,
    termination_signal: TerminationSignal,
    grace_ms: u32,
    asserted_subject_spec_sha256: Option<Digest>,
    asserted_binding_report_sha256: Option<Digest>,
}

impl ValidatedLaunchPlan {
    /// Exact accepted input bytes, retained unchanged.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 over exactly [`Self::exact_bytes`]. No canonicalisation: any
    /// formatting change produces a different identity.
    #[must_use]
    pub const fn sha256(&self) -> Digest {
        self.sha256
    }

    /// The requested execution kind.
    #[must_use]
    pub const fn execution_kind(&self) -> ExecutionKind {
        self.execution_kind
    }

    /// Literal argument vector, `argv[0]` included exactly as the caller wrote
    /// it. Readable data; no quoting, splitting or expansion exists.
    #[must_use]
    pub fn argv(&self) -> &[String] {
        &self.argv
    }

    /// The requested environment mode.
    #[must_use]
    pub const fn environment_mode(&self) -> EnvironmentMode {
        self.environment_mode
    }

    /// Logical identifier a working-directory capability must carry. An
    /// identifier, never a path.
    #[must_use]
    pub fn working_directory_id(&self) -> &str {
        self.working_directory_id.as_str()
    }

    /// The same identifier as the parsed capability value, for the **P4**
    /// receipt.
    ///
    /// The receipt records what plan validation already admitted; nothing
    /// re-parses a string that was validated once.
    #[cfg_attr(
        not(all(target_os = "linux", target_arch = "x86_64")),
        allow(
            dead_code,
            reason = "off the Linux x86_64 cohort no launch producer builds a receipt"
        )
    )]
    pub(crate) const fn working_directory_capability_id(&self) -> &CapabilityId {
        &self.working_directory_id
    }

    /// The requested stdin arrangement.
    #[must_use]
    pub const fn stdin_mode(&self) -> StdinMode {
        self.stdin_mode
    }

    /// Requested in-memory prefix bound for one stream.
    #[must_use]
    pub const fn capture_prefix_bytes(&self, stream: Stream) -> u32 {
        match stream {
            Stream::Stdout => self.stdout_capture_prefix_bytes,
            Stream::Stderr => self.stderr_capture_prefix_bytes,
        }
    }

    /// Requested run timeout in milliseconds.
    #[must_use]
    pub const fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    /// Requested first termination signal.
    #[must_use]
    pub const fn termination_signal(&self) -> TerminationSignal {
        self.termination_signal
    }

    /// Requested grace period in milliseconds.
    #[must_use]
    pub const fn grace_ms(&self) -> u32 {
        self.grace_ms
    }

    /// Caller-asserted specification digest. Never compared or interpreted.
    #[must_use]
    pub const fn asserted_subject_spec_sha256(&self) -> Option<Digest> {
        self.asserted_subject_spec_sha256
    }

    /// Caller-asserted binding-report digest. Never compared or interpreted.
    #[must_use]
    pub const fn asserted_binding_report_sha256(&self) -> Option<Digest> {
        self.asserted_binding_report_sha256
    }
}

// ---------------------------------------------------------------- strict tree

/// A private bounded JSON tree.
///
/// Never deserialize untrusted bytes straight into `serde_json::Value`: its
/// object visitor keeps the last of two duplicate keys, so two documents with
/// different meaning would validate identically. Duplicates are refused here, on
/// **decoded** keys, while the tree is built. A private tree also keeps map order
/// independent of `serde_json`'s optional `preserve_order` feature.
enum Json {
    Null,
    Bool,
    Integer(i128),
    Float,
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

struct Tree<'a> {
    failure: &'a mut Option<C>,
    depth: usize,
}

fn reject<E: serde::de::Error>(failure: &mut Option<C>, code: C) -> E {
    *failure = Some(code);
    E::custom("bounded JSON rejected")
}

impl<'de> serde::de::DeserializeSeed<'de> for Tree<'_> {
    type Value = Json;
    fn deserialize<D: serde::Deserializer<'de>>(self, parser: D) -> Result<Json, D::Error> {
        parser.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for Tree<'_> {
    type Value = Json;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("a bounded launch plan document")
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Json, E> {
        Ok(Json::Null)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Json, E> {
        Ok(Json::Null)
    }
    fn visit_bool<E: serde::de::Error>(self, _v: bool) -> Result<Json, E> {
        Ok(Json::Bool)
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Json, E> {
        Ok(Json::Integer(i128::from(v)))
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Json, E> {
        Ok(Json::Integer(i128::from(v)))
    }
    fn visit_i128<E: serde::de::Error>(self, v: i128) -> Result<Json, E> {
        Ok(Json::Integer(v))
    }
    fn visit_u128<E: serde::de::Error>(self, v: u128) -> Result<Json, E> {
        Ok(Json::Integer(i128::try_from(v).unwrap_or(i128::MAX)))
    }
    fn visit_f64<E: serde::de::Error>(self, _v: f64) -> Result<Json, E> {
        Ok(Json::Float)
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Json, E> {
        Ok(Json::String(v.to_owned()))
    }

    fn visit_map<M: serde::de::MapAccess<'de>>(self, mut input: M) -> Result<Json, M::Error> {
        if self.depth >= MAX_JSON_DEPTH {
            return Err(reject(self.failure, C::NestingTooDeep));
        }
        let mut object = BTreeMap::new();
        // Never reserve from an untrusted size hint.
        while let Some(key) = input.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(reject(self.failure, C::DuplicateKey));
            }
            let value = input.next_value_seed(Tree {
                failure: &mut *self.failure,
                depth: self.depth + 1,
            })?;
            object.insert(key, value);
        }
        Ok(Json::Object(object))
    }

    fn visit_seq<S: serde::de::SeqAccess<'de>>(self, mut input: S) -> Result<Json, S::Error> {
        if self.depth >= MAX_JSON_DEPTH {
            return Err(reject(self.failure, C::NestingTooDeep));
        }
        let mut items = Vec::new();
        while let Some(value) = input.next_element_seed(Tree {
            failure: &mut *self.failure,
            depth: self.depth + 1,
        })? {
            items.push(value);
        }
        Ok(Json::Array(items))
    }
}

fn strict_tree(bytes: &[u8]) -> Result<Json, C> {
    let mut failure: Option<C> = None;
    let mut parser = serde_json::Deserializer::from_slice(bytes);
    let tree = serde::de::DeserializeSeed::deserialize(
        Tree {
            failure: &mut failure,
            depth: 0,
        },
        &mut parser,
    );
    let Ok(tree) = tree else {
        return Err(failure.unwrap_or(C::MalformedJson));
    };
    if parser.end().is_err() {
        return Err(C::MalformedJson);
    }
    Ok(tree)
}

/// Whether bytes pass the strict structural admission alone.
#[cfg(test)]
pub(crate) fn is_strict_json(bytes: &[u8]) -> bool {
    strict_tree(bytes).is_ok()
}

// ---------------------------------------------------------------- validation

type Object = BTreeMap<String, Json>;

struct Findings(Vec<LaunchPlanError>);

impl Findings {
    fn push(&mut self, code: C, locator: &'static str) {
        self.0.push(LaunchPlanError::new(code, locator));
    }

    fn push_at(&mut self, code: C, locator: &'static str, index: usize) {
        self.0.push(LaunchPlanError::at(code, locator, index));
    }

    /// Report every key outside `allowed` against the containing object. The
    /// unknown key's text is never echoed.
    fn closed_keys(&mut self, object: &Object, allowed: &[&str], locator: &'static str) {
        if object.keys().any(|k| !allowed.contains(&k.as_str())) {
            self.push(C::UnknownField, locator);
        }
    }

    fn required<'a>(
        &mut self,
        object: &'a Object,
        key: &str,
        locator: &'static str,
    ) -> Option<&'a Json> {
        let value = object.get(key);
        if value.is_none() {
            self.push(C::MissingField, locator);
        }
        value
    }

    fn string<'a>(&mut self, value: Option<&'a Json>, locator: &'static str) -> Option<&'a str> {
        match value? {
            Json::String(s) => Some(s.as_str()),
            _ => {
                self.push(C::TypeMismatch, locator);
                None
            }
        }
    }

    fn object<'a>(&mut self, value: Option<&'a Json>, locator: &'static str) -> Option<&'a Object> {
        match value? {
            Json::Object(o) => Some(o),
            _ => {
                self.push(C::TypeMismatch, locator);
                None
            }
        }
    }

    /// A JSON integer inside `min..=max`. A non-integer number is a type
    /// mismatch; an integer outside the range, negative included, is `range`.
    fn integer(
        &mut self,
        value: Option<&Json>,
        locator: &'static str,
        min: u32,
        max: u32,
        range: C,
    ) -> Option<u32> {
        match value? {
            Json::Integer(n) => match u32::try_from(*n) {
                Ok(v) if (min..=max).contains(&v) => Some(v),
                _ => {
                    self.push(range, locator);
                    None
                }
            },
            _ => {
                self.push(C::TypeMismatch, locator);
                None
            }
        }
    }

    /// A string that must equal exactly `expected`, otherwise `mismatch`.
    fn constant(
        &mut self,
        value: Option<&Json>,
        locator: &'static str,
        expected: &str,
        mismatch: C,
    ) -> bool {
        match self.string(value, locator) {
            Some(s) if s == expected => true,
            Some(_) => {
                self.push(mismatch, locator);
                false
            }
            None => false,
        }
    }

    /// A single-member object `{ key: value }` from the closed schema.
    fn member<'a>(
        &mut self,
        root: &'a Object,
        name: &str,
        name_locator: &'static str,
        key: &str,
        key_locator: &'static str,
    ) -> Option<&'a Json> {
        let value = self.required(root, name, name_locator);
        let object = self.object(value, name_locator)?;
        self.closed_keys(object, &[key], name_locator);
        self.required(object, key, key_locator)
    }

    fn argv(&mut self, value: Option<&Json>) -> Option<Vec<String>> {
        let items = match value? {
            Json::Array(items) => items,
            _ => {
                self.push(C::TypeMismatch, "argv");
                return None;
            }
        };
        let mut strings = Vec::with_capacity(items.len().min(MAX_ARGS));
        let mut well_typed = true;
        for (i, item) in items.iter().enumerate().take(MAX_ARGS) {
            match item {
                Json::String(s) => strings.push(s.clone()),
                _ => {
                    self.push_at(C::TypeMismatch, "argv", i);
                    well_typed = false;
                }
            }
        }
        let count_ok = validate_argv_shape(items.len(), &strings, self);
        (well_typed && count_ok).then_some(strings)
    }
}

/// argv rules that do not depend on JSON types, kept separate so the total-size
/// bound is testable even though [`MAX_PLAN_BYTES`] makes it unreachable through
/// a document: escape decoding never lengthens a string.
fn validate_argv_shape(declared: usize, strings: &[String], findings: &mut Findings) -> bool {
    let mut ok = true;
    if declared == 0 {
        findings.push(C::ArgvEmpty, "argv");
        ok = false;
    }
    if declared > MAX_ARGS {
        findings.push(C::ArgvTooMany, "argv");
        ok = false;
    }
    let mut total: usize = 0;
    for (i, s) in strings.iter().enumerate() {
        if s.as_bytes().contains(&0) {
            findings.push_at(C::ArgContainsNul, "argv", i);
            ok = false;
        }
        if s.len() > MAX_ARG_BYTES {
            findings.push_at(C::ArgTooLong, "argv", i);
            ok = false;
        }
        total = total.saturating_add(s.len());
    }
    if total > MAX_ARGV_TOTAL_BYTES {
        findings.push(C::ArgvTooLarge, "argv");
        ok = false;
    }
    ok
}

/// Parse and bound untrusted plan bytes into inert caller intent.
///
/// Structural failures (size, malformed JSON, duplicate decoded keys, nesting)
/// return exactly one document-level finding, because such a document has no
/// reliable meaning to report field findings against. Otherwise every field
/// finding is returned, deterministically ordered and capped.
///
/// No input/output of any kind happens on any path.
///
/// ```
/// let bytes = br#"{"schema":"helm-launch-plan","version":"0.1","execution_kind":"linux_exact_executable","argv":["tool"],"environment":{"mode":"empty"},"working_directory":{"capability_id":"workdir"},"stdin":{"mode":"closed_pipe_eof"},"stdout":{"capture_prefix_bytes":0},"stderr":{"capture_prefix_bytes":4096},"timeout_ms":30000,"termination":{"signal":"SIGTERM","grace_ms":5000}}"#;
/// let plan = helm_launch::parse_launch_plan(bytes).unwrap();
/// assert_eq!(plan.argv(), ["tool"]);
/// assert_eq!(plan.exact_bytes(), bytes);
/// ```
///
/// # Errors
/// [`LaunchPlanErrors`] with at least one finding.
pub fn parse_launch_plan(bytes: &[u8]) -> Result<ValidatedLaunchPlan, LaunchPlanErrors> {
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(LaunchPlanErrors::single(C::InputTooLarge, "plan"));
    }
    let tree = strict_tree(bytes).map_err(|code| LaunchPlanErrors::single(code, "plan"))?;
    let Json::Object(root) = tree else {
        return Err(LaunchPlanErrors::single(C::NotJsonObject, "plan"));
    };

    let mut f = Findings(Vec::new());
    f.closed_keys(
        &root,
        &[
            "schema",
            "version",
            "execution_kind",
            "argv",
            "environment",
            "working_directory",
            "stdin",
            "stdout",
            "stderr",
            "timeout_ms",
            "termination",
            "asserted_context",
        ],
        "plan",
    );

    let v = f.required(&root, "schema", "schema");
    let schema_ok = f.constant(v, "schema", SCHEMA, C::UnknownSchema);
    let v = f.required(&root, "version", "version");
    let version_ok = f.constant(v, "version", VERSION, C::UnknownVersion);
    let v = f.required(&root, "execution_kind", "execution_kind");
    let kind = f
        .constant(
            v,
            "execution_kind",
            ExecutionKind::LinuxExactExecutable.as_str(),
            C::ExecutionKindUnsupported,
        )
        .then_some(ExecutionKind::LinuxExactExecutable);

    let v = f.required(&root, "argv", "argv");
    let argv = f.argv(v);

    let v = f.member(
        &root,
        "environment",
        "environment",
        "mode",
        "environment.mode",
    );
    let environment = f
        .constant(
            v,
            "environment.mode",
            EnvironmentMode::Empty.as_str(),
            C::EnvironmentModeUnsupported,
        )
        .then_some(EnvironmentMode::Empty);

    let v = f.member(
        &root,
        "working_directory",
        "working_directory",
        "capability_id",
        "working_directory.capability_id",
    );
    let working_directory = f
        .string(v, "working_directory.capability_id")
        .and_then(|s| {
            let id = CapabilityId::parse(s);
            if id.is_none() {
                f.push(C::IdGrammar, "working_directory.capability_id");
            }
            id
        });

    let v = f.member(&root, "stdin", "stdin", "mode", "stdin.mode");
    let stdin = f
        .constant(
            v,
            "stdin.mode",
            StdinMode::ClosedPipeEof.as_str(),
            C::StdinModeUnsupported,
        )
        .then_some(StdinMode::ClosedPipeEof);

    let v = f.member(
        &root,
        "stdout",
        "stdout",
        "capture_prefix_bytes",
        "stdout.capture_prefix_bytes",
    );
    let stdout = f.integer(
        v,
        "stdout.capture_prefix_bytes",
        0,
        MAX_CAPTURE_BYTES,
        C::CaptureBoundOutOfRange,
    );
    let v = f.member(
        &root,
        "stderr",
        "stderr",
        "capture_prefix_bytes",
        "stderr.capture_prefix_bytes",
    );
    let stderr = f.integer(
        v,
        "stderr.capture_prefix_bytes",
        0,
        MAX_CAPTURE_BYTES,
        C::CaptureBoundOutOfRange,
    );

    let v = f.required(&root, "timeout_ms", "timeout_ms");
    let timeout = f.integer(
        v,
        "timeout_ms",
        MIN_TIMEOUT_MS,
        MAX_TIMEOUT_MS,
        C::TimeoutOutOfRange,
    );

    let v = f.required(&root, "termination", "termination");
    let mut signal = None;
    let mut grace = None;
    if let Some(t) = f.object(v, "termination") {
        f.closed_keys(t, &["signal", "grace_ms"], "termination");
        let v = f.required(t, "signal", "termination.signal");
        signal = f
            .constant(
                v,
                "termination.signal",
                TerminationSignal::Sigterm.as_str(),
                C::TerminationSignalUnsupported,
            )
            .then_some(TerminationSignal::Sigterm);
        let v = f.required(t, "grace_ms", "termination.grace_ms");
        grace = f.integer(
            v,
            "termination.grace_ms",
            0,
            MAX_GRACE_MS,
            C::GraceOutOfRange,
        );
    }

    let mut subject = None;
    let mut binding = None;
    let mut context_ok = true;
    if let Some(value) = root.get("asserted_context") {
        match f.object(Some(value), "asserted_context") {
            Some(ctx) => {
                f.closed_keys(
                    ctx,
                    &["subject_spec_sha256", "binding_report_sha256"],
                    "asserted_context",
                );
                let before = f.0.len();
                subject = asserted_digest(
                    &mut f,
                    ctx,
                    "subject_spec_sha256",
                    "asserted_context.subject_spec_sha256",
                );
                binding = asserted_digest(
                    &mut f,
                    ctx,
                    "binding_report_sha256",
                    "asserted_context.binding_report_sha256",
                );
                context_ok = f.0.len() == before;
            }
            None => context_ok = false,
        }
    }

    if !f.0.is_empty() {
        return Err(LaunchPlanErrors::new(f.0));
    }
    // With no finding recorded every required value is present; this match only
    // proves it to the compiler, and never yields a plan with a default.
    let (
        true,
        true,
        true,
        Some(execution_kind),
        Some(argv),
        Some(environment_mode),
        Some(working_directory_id),
        Some(stdin_mode),
        Some(stdout_capture_prefix_bytes),
        Some(stderr_capture_prefix_bytes),
        Some(timeout_ms),
        Some(termination_signal),
        Some(grace_ms),
    ) = (
        schema_ok,
        version_ok,
        context_ok,
        kind,
        argv,
        environment,
        working_directory,
        stdin,
        stdout,
        stderr,
        timeout,
        signal,
        grace,
    )
    else {
        return Err(LaunchPlanErrors::single(C::MissingField, "plan"));
    };

    Ok(ValidatedLaunchPlan {
        bytes: bytes.to_vec(),
        sha256: Digest::of(bytes),
        execution_kind,
        argv,
        environment_mode,
        working_directory_id,
        stdin_mode,
        stdout_capture_prefix_bytes,
        stderr_capture_prefix_bytes,
        timeout_ms,
        termination_signal,
        grace_ms,
        asserted_subject_spec_sha256: subject,
        asserted_binding_report_sha256: binding,
    })
}

/// An optional asserted digest. Absence is the only way not to assert one;
/// `null` is a type mismatch.
fn asserted_digest(
    f: &mut Findings,
    ctx: &Object,
    key: &str,
    locator: &'static str,
) -> Option<Digest> {
    let text = f.string(ctx.get(key), locator)?;
    let digest = Digest::parse_hex(text);
    if digest.is_none() {
        f.push(C::DigestGrammar, locator);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::{Findings, MAX_ARG_BYTES, MAX_ARGS, MAX_ARGV_TOTAL_BYTES, validate_argv_shape};
    use crate::error::LaunchPlanErrorCode as C;

    /// The document size bound makes the total bound unreachable through
    /// `parse_launch_plan`, so the rule itself is exercised directly.
    #[test]
    fn the_total_argv_bound_is_enforced_by_the_rule_itself() {
        let element = "x".repeat(MAX_ARG_BYTES);
        let count = MAX_ARGV_TOTAL_BYTES / MAX_ARG_BYTES;
        let at_bound: Vec<String> = vec![element.clone(); count];
        let mut f = Findings(Vec::new());
        assert!(validate_argv_shape(at_bound.len(), &at_bound, &mut f));
        assert!(f.0.is_empty());

        let mut over = at_bound;
        over.push("y".to_owned());
        assert!(over.len() <= MAX_ARGS);
        let mut f = Findings(Vec::new());
        assert!(!validate_argv_shape(over.len(), &over, &mut f));
        assert_eq!(f.0.len(), 1);
        assert_eq!(f.0[0].code(), C::ArgvTooLarge);
    }
}
