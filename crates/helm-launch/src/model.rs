//! Portable identity and receipt-record model.
//!
//! Everything here is inert data. A value of any type in this module grants no
//! authority, observes nothing, and states no verdict. The fact enums are closed
//! vocabularies with lowercase snake-case spellings; none of them names success,
//! readiness, compatibility, verification, sandboxing or containment, and none
//! offers a conversion to `bool`, `Result<(), _>` or an ordering of outcomes.
//!
//! In P1 no code in this crate observes any of these facts from an operating
//! system. The model fixes the accepted 0.1 receipt contract so that it can be
//! serialised and tested before any backend exists.

use sha2::{Digest as _, Sha256};

/// Maximum bytes in a logical capability identifier.
pub const MAX_ID_BYTES: usize = 80;

/// A 32-byte SHA-256 value, spelled externally as 64 lowercase hexadecimal
/// characters.
///
/// A digest identifies bytes. It does not mean that those bytes are authentic,
/// trusted, approved, compatible or authorised, and it carries no authority.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest([u8; 32]);

const fn hex_char(nibble: u8) -> char {
    let n = nibble & 0x0f;
    if n < 10 {
        (b'0' + n) as char
    } else {
        (b'a' + (n - 10)) as char
    }
}

const fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

impl Digest {
    /// The raw 32 bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// 64 lowercase hexadecimal characters. Identical on every platform.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0 {
            s.push(hex_char(b >> 4));
            s.push(hex_char(b));
        }
        s
    }

    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "no P1 producer builds a digest from raw bytes")
    )]
    pub(crate) const fn from_raw(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// SHA-256 over exactly `bytes`, with no canonicalisation.
    pub(crate) fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Strict grammar: exactly 64 characters from `[0-9a-f]`. Uppercase, a
    /// prefix, whitespace or any other spelling is refused.
    pub(crate) fn parse_hex(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (slot, pair) in out.iter_mut().zip(bytes.chunks_exact(2)) {
            let [hi, lo] = pair else {
                return None;
            };
            *slot = (hex_value(*hi)? << 4) | hex_value(*lo)?;
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

/// A validated logical identifier: `[a-z0-9][a-z0-9._-]{0,79}`.
///
/// Crate-private so that the serializer can rely on the grammar: every accepted
/// spelling is already JSON-safe and needs no escaping or filtering.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CapabilityId(String);

impl CapabilityId {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        let (first, rest) = bytes.split_first()?;
        if bytes.len() > MAX_ID_BYTES {
            return None;
        }
        let head_ok = first.is_ascii_lowercase() || first.is_ascii_digit();
        let tail_ok = rest.iter().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        });
        (head_ok && tail_ok).then(|| Self(text.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// One of the two measured output streams.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Stream {
    /// Descriptor 1 of a future executed image.
    Stdout,
    /// Descriptor 2 of a future executed image.
    Stderr,
}

impl Stream {
    /// Stable spelling, also the receipt key of the stream's facts.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

/// The child environment mode. 0.1 has exactly one: an empty environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EnvironmentMode {
    /// `envp` holds no entry. Hardening, not provenance.
    Empty,
}

impl EnvironmentMode {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
        }
    }
}

/// The closed mechanism identifier a 0.1 receipt names.
///
/// This is receipt vocabulary only. **No backend exists in P1**, and naming the
/// accepted mechanism here implements none of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Backend {
    /// The accepted 0.1 Linux x86_64 mechanism of ADR-0024 section D.
    LinuxX8664Clone3PidfdExecveat,
}

impl Backend {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LinuxX8664Clone3PidfdExecveat => "linux_x86_64_clone3_pidfd_execveat",
        }
    }
}

/// ELF `e_type` of an admitted object, as a header field. A header value proves
/// nothing about the program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ElfType {
    /// `ET_EXEC`.
    EtExec,
    /// `ET_DYN`.
    EtDyn,
}

impl ElfType {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EtExec => "et_exec",
            Self::EtDyn => "et_dyn",
        }
    }
}

/// A stage of the accepted closed child sequence (plan section 8.4), named by a
/// structured pre-exec failure record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ChildStage {
    /// Final stdio mapping.
    Dup2,
    /// Clearing close-on-exec on descriptors 0, 1 and 2.
    ClearCloexec,
    /// Changing to the working-directory capability.
    Chdir,
    /// Closing every non-preserved descriptor.
    CloseRange,
    /// Entering the dedicated process group.
    Setpgid,
    /// Resetting signal dispositions.
    Sigaction,
    /// Installing the final signal mask.
    Sigmask,
    /// Setting `no_new_privs`.
    NoNewPrivs,
    /// The execution attempt itself.
    Exec,
}

impl ChildStage {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dup2 => "dup2",
            Self::ClearCloexec => "clear_cloexec",
            Self::Chdir => "chdir",
            Self::CloseRange => "close_range",
            Self::Setpgid => "setpgid",
            Self::Sigaction => "sigaction",
            Self::Sigmask => "sigmask",
            Self::NoNewPrivs => "no_new_privs",
            Self::Exec => "exec",
        }
    }
}

/// Why nothing more can be said about the execution attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum IndeterminateReason {
    /// The status channel reached EOF with no record. This is what every
    /// ordinary run shows, and also what a child that died before exec shows.
    StatusEofWithoutRecord,
    /// A short or oversized status record.
    StatusRecordMalformed,
    /// Neither a record nor EOF arrived within the pre-exec bound.
    PreExecStatusTimeout,
    /// Reading the status channel failed. Never treated as EOF.
    StatusReadFailed {
        /// Raw error number reported for the read.
        errno: i32,
    },
}

impl IndeterminateReason {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StatusEofWithoutRecord => "status_eof_without_record",
            Self::StatusRecordMalformed => "status_record_malformed",
            Self::PreExecStatusTimeout => "pre_exec_status_timeout",
            Self::StatusReadFailed { .. } => "status_read_failed",
        }
    }
}

/// What the exec-status channel established.
///
/// There is deliberately no variant meaning that execution happened: clean
/// exec-status EOF alone is not positive exec proof, and nothing derives one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExecStatus {
    /// An explicit structured record: the named stage failed before or at the
    /// execution attempt.
    PreExecFailure {
        /// The stage that reported the failure.
        stage: ChildStage,
        /// Raw error number from the record.
        errno: i32,
    },
    /// Nothing more can be said about the execution attempt.
    Indeterminate(IndeterminateReason),
}

impl ExecStatus {
    /// Stable spelling of the kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreExecFailure { .. } => "pre_exec_failure",
            Self::Indeterminate(_) => "indeterminate",
        }
    }
}

/// What was observed about the end of the direct child. No cause is claimed:
/// a signal number is not a sender.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ChildEnd {
    /// The direct child exited with this status. Exit code 0 means only that.
    Exited {
        /// Raw exit status.
        code: i32,
    },
    /// The direct child was terminated by this signal number.
    Signaled {
        /// Raw signal number.
        signal: i32,
        /// Whether the end was reported as a core-dumping termination.
        core_dumped: bool,
    },
    /// The end could not be classified, for example because the child was
    /// reaped elsewhere.
    EndUnobservable,
    /// No end was observed within the bound after `SIGKILL`. No claim is made
    /// that the child is still running at any later moment.
    EndNotObserved,
}

impl ChildEnd {
    /// Stable spelling of the kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exited { .. } => "exited",
            Self::Signaled { .. } => "signaled",
            Self::EndUnobservable => "end_unobservable",
            Self::EndNotObserved => "end_not_observed",
        }
    }
}

/// Whether the one process-group cleanup call was made, or why not.
///
/// `Issued` records that the call was made. It says nothing about which
/// processes, if any, received a signal. It is cleanup, never containment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GroupSweep {
    /// The one group call was made, before the reap.
    Issued,
    /// Group authority was never positively established, so no call was made.
    NotIssuedGroupNotEstablished,
    /// The child was observed already reaped elsewhere, so no call was made.
    NotIssuedChildAlreadyReaped,
}

impl GroupSweep {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Issued => "issued",
            Self::NotIssuedGroupNotEstablished => "not_issued_group_not_established",
            Self::NotIssuedChildAlreadyReaped => "not_issued_child_already_reaped",
        }
    }
}

/// Why reading a stream stopped (plan section 11.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Completeness {
    /// The pipe reached EOF and every byte was drained.
    CompleteAtEof,
    /// The direct child ended and another process still held a write end when
    /// the post-exit drain bound expired. Nothing is claimed about later output.
    WriterRetainedAfterChildExit,
    /// Reading stopped because no end of the direct child was observed within
    /// the post-kill bound.
    ReadStoppedChildEndNotObserved,
    /// A read call reported an error. Bytes before it were counted. Never EOF.
    ReadFailed {
        /// Raw error number reported for the read.
        errno: i32,
    },
}

impl Completeness {
    /// Stable spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CompleteAtEof => "complete_at_eof",
            Self::WriterRetainedAfterChildExit => "writer_retained_after_child_exit",
            Self::ReadStoppedChildEndNotObserved => "read_stopped_child_end_not_observed",
            Self::ReadFailed { .. } => "read_failed",
        }
    }
}

/// Caller-asserted context digests. Recorded only: never compared, fetched or
/// interpreted, and never evidence that anything was consulted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssertedContext {
    pub(crate) subject_spec_sha256: Option<Digest>,
    pub(crate) binding_report_sha256: Option<Digest>,
}

impl AssertedContext {
    /// The caller's asserted application-specification digest, if any.
    #[must_use]
    pub const fn subject_spec_sha256(&self) -> Option<Digest> {
        self.subject_spec_sha256
    }

    /// The caller's asserted binding-report digest, if any.
    #[must_use]
    pub const fn binding_report_sha256(&self) -> Option<Digest> {
        self.binding_report_sha256
    }
}

/// A pre-execution measurement of the main executable file body.
///
/// Never the identity of the bytes that executed, and never a statement about
/// the ELF interpreter, libraries or any loaded code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExecutableMeasurement {
    pub(crate) pre_exec_body_size: u64,
    pub(crate) pre_exec_body_sha256: Digest,
    pub(crate) pre_exec_mode_bits: u16,
    pub(crate) elf_type: ElfType,
}

impl ExecutableMeasurement {
    /// Bytes read before the execution attempt.
    #[must_use]
    pub const fn pre_exec_body_size(&self) -> u64 {
        self.pre_exec_body_size
    }

    /// SHA-256 over the bytes read before the execution attempt.
    #[must_use]
    pub const fn pre_exec_body_sha256(&self) -> Digest {
        self.pre_exec_body_sha256
    }

    /// Permission bits sampled before the execution attempt.
    #[must_use]
    pub const fn pre_exec_mode_bits(&self) -> u16 {
        self.pre_exec_mode_bits
    }

    /// Header `e_type`.
    #[must_use]
    pub const fn elf_type(&self) -> ElfType {
        self.elf_type
    }
}

/// Termination actions recorded as facts, with no causal claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Termination {
    pub(crate) sigterm_sent: bool,
    pub(crate) sigkill_sent: bool,
    pub(crate) group_sweep: GroupSweep,
}

impl Termination {
    /// Whether `SIGTERM` was sent to the direct child.
    #[must_use]
    pub const fn sigterm_sent(&self) -> bool {
        self.sigterm_sent
    }

    /// Whether `SIGKILL` was sent to the direct child.
    #[must_use]
    pub const fn sigkill_sent(&self) -> bool {
        self.sigkill_sent
    }

    /// Whether the group cleanup call was made, or why not.
    #[must_use]
    pub const fn group_sweep(&self) -> GroupSweep {
        self.group_sweep
    }
}

/// Per-stream measurement. No output byte is ever held here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StreamFacts {
    pub(crate) bytes_drained: u64,
    pub(crate) drained_sha256: Digest,
    pub(crate) completeness: Completeness,
}

impl StreamFacts {
    /// Bytes read before reading stopped.
    #[must_use]
    pub const fn bytes_drained(&self) -> u64 {
        self.bytes_drained
    }

    /// SHA-256 over exactly the drained bytes, which the receipt does not
    /// publish.
    #[must_use]
    pub const fn drained_sha256(&self) -> Digest {
        self.drained_sha256
    }

    /// Why reading stopped.
    #[must_use]
    pub const fn completeness(&self) -> Completeness {
        self.completeness
    }
}

/// The complete 0.1 receipt record, in schema field order.
///
/// There is no public constructor, no `Default` and no `Deserialize`: outside
/// this crate a record can only be read. That is an in-process API property. It
/// does not make any serialised receipt authentic.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReceiptRecord {
    pub(crate) backend: Backend,
    pub(crate) plan_sha256: Digest,
    pub(crate) asserted_context: AssertedContext,
    pub(crate) working_directory_id: CapabilityId,
    pub(crate) executable: ExecutableMeasurement,
    pub(crate) argument_count: u32,
    pub(crate) environment_mode: EnvironmentMode,
    pub(crate) exec_status: ExecStatus,
    pub(crate) child_end: ChildEnd,
    pub(crate) run_deadline_expired: bool,
    pub(crate) termination: Termination,
    pub(crate) stdout: StreamFacts,
    pub(crate) stderr: StreamFacts,
}

impl ReceiptRecord {
    /// The closed mechanism identifier.
    #[must_use]
    pub const fn backend(&self) -> Backend {
        self.backend
    }

    /// Exact-byte identity of the launch plan.
    #[must_use]
    pub const fn plan_sha256(&self) -> Digest {
        self.plan_sha256
    }

    /// Caller-asserted, unverified context digests.
    #[must_use]
    pub const fn asserted_context(&self) -> &AssertedContext {
        &self.asserted_context
    }

    /// The caller's working-directory identifier. No path is ever recorded.
    #[must_use]
    pub fn working_directory_id(&self) -> &str {
        self.working_directory_id.as_str()
    }

    /// The pre-execution measurement.
    #[must_use]
    pub const fn executable(&self) -> &ExecutableMeasurement {
        &self.executable
    }

    /// Number of argv elements. The values are fixed by the plan digest.
    #[must_use]
    pub const fn argument_count(&self) -> u32 {
        self.argument_count
    }

    /// The environment mode.
    #[must_use]
    pub const fn environment_mode(&self) -> EnvironmentMode {
        self.environment_mode
    }

    /// What the exec-status channel established.
    #[must_use]
    pub const fn exec_status(&self) -> ExecStatus {
        self.exec_status
    }

    /// What was observed about the direct child's end.
    #[must_use]
    pub const fn child_end(&self) -> ChildEnd {
        self.child_end
    }

    /// Whether the run deadline passed before the child's end was observed.
    #[must_use]
    pub const fn run_deadline_expired(&self) -> bool {
        self.run_deadline_expired
    }

    /// Termination actions.
    #[must_use]
    pub const fn termination(&self) -> &Termination {
        &self.termination
    }

    /// Facts for one stream.
    #[must_use]
    pub const fn stream(&self, stream: Stream) -> &StreamFacts {
        match stream {
            Stream::Stdout => &self.stdout,
            Stream::Stderr => &self.stderr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CapabilityId, Digest};

    #[test]
    fn digest_hex_round_trips_and_is_strict() {
        let d = Digest::from_raw([
            0x00, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76,
            0x54, 0x32, 0x10, 0xff, 0x7f, 0x80, 0x0f, 0xf0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88, 0x99, 0xaa,
        ]);
        let hex = d.to_hex();
        assert_eq!(
            hex,
            "000123456789abcdeffedcba9876543210ff7f800ff0112233445566778899aa"
        );
        assert_eq!(Digest::parse_hex(&hex), Some(d));
        assert_eq!(format!("{d}"), hex);
        assert_eq!(format!("{d:?}"), hex);

        for bad in [
            String::new(),
            hex.to_uppercase(),
            format!("{hex}0"),
            hex[..63].to_owned(),
            format!(" {}", &hex[1..]),
            format!("0x{}", &hex[2..]),
            format!("{}g", &hex[..63]),
            format!("{}\u{e9}", &hex[..62]),
        ] {
            assert_eq!(Digest::parse_hex(&bad), None, "accepted {bad:?}");
        }
    }

    #[test]
    fn digest_of_matches_the_published_sha256_of_empty_input() {
        assert_eq!(
            Digest::of(b"").to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn capability_id_grammar_is_exact() {
        let longest = format!("a{}", "z".repeat(79));
        for good in ["workdir", "0", "a.b_c-d", "9-", longest.as_str()] {
            assert_eq!(
                CapabilityId::parse(good).map(|c| c.as_str().to_owned()),
                Some(good.to_owned())
            );
        }
        let too_long = format!("a{}", "z".repeat(80));
        for bad in [
            "",
            "Workdir",
            ".a",
            "_a",
            "-a",
            "a b",
            "a/b",
            "a\\b",
            "a\"b",
            "a\u{e9}",
            "a\0",
            too_long.as_str(),
        ] {
            assert!(CapabilityId::parse(bad).is_none(), "accepted {bad:?}");
        }
    }
}
