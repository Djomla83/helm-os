//! The deterministic receipt serializer.
//!
//! A receipt is exact bytes plus SHA-256 over exactly those bytes. The bytes
//! come from a fixed-order, hand-written serializer with no map iteration, no
//! clock, no host value and no platform-dependent formatting. Nothing is
//! withheld, redacted or sanitised after serialisation, and the record holds no
//! output byte, so the digest is always recomputable from the published bytes.
//!
//! **P1 has no producer.** No function in this crate creates a receipt from an
//! observed launch, because no launch exists. The crate-private constructor
//! below is exercised only by tests, with deterministic test data that is not a
//! record of anything.

use crate::model::{
    ChildEnd, Completeness, Digest, ExecStatus, IndeterminateReason, ReceiptRecord, StreamFacts,
};

/// Proven upper bound on a serialised receipt. Not a truncation point: the
/// widest receipt the model can express is far below it, and nothing is cut.
pub const MAX_RECEIPT_BYTES: usize = 8_192;

/// A serialised 0.1 launch receipt: deterministic data with **zero authority**.
///
/// Receipt bytes may be copied, and may be fabricated outside this crate. They
/// are not signed and are not proof of provenance by themselves; this crate
/// makes no receipt-authenticity claim. What the API does prevent is building an
/// in-memory `LaunchReceipt` from fields or from bytes:
///
/// ```compile_fail
/// fn forge(bytes: Vec<u8>, real: helm_launch::LaunchReceipt) -> helm_launch::LaunchReceipt {
///     helm_launch::LaunchReceipt { bytes, ..real }
/// }
/// ```
///
/// ```compile_fail
/// let bytes: &[u8] = b"{}";
/// let _: helm_launch::LaunchReceipt = serde_json::from_slice(bytes).unwrap();
/// ```
///
/// ```compile_fail
/// let bytes: &[u8] = b"{}";
/// let _: helm_launch::ReceiptRecord = serde_json::from_slice(bytes).unwrap();
/// ```
///
/// The same deserialisation into a type that does implement `Deserialize`
/// compiles, so the two failures above are about the receipt types only:
///
/// ```
/// let bytes: &[u8] = b"{}";
/// let _: serde_json::Value = serde_json::from_slice(bytes).unwrap();
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LaunchReceipt {
    bytes: Vec<u8>,
    sha256: Digest,
    record: ReceiptRecord,
}

impl core::fmt::Debug for LaunchReceipt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LaunchReceipt")
            .field("sha256", &self.sha256)
            .field("byte_len", &self.bytes.len())
            .field("record", &self.record)
            .finish()
    }
}

fn push_digest(out: &mut String, d: Digest) {
    out.push('"');
    out.push_str(&d.to_hex());
    out.push('"');
}

fn push_optional_digest(out: &mut String, d: Option<Digest>) {
    match d {
        Some(d) => push_digest(out, d),
        None => out.push_str("null"),
    }
}

fn push_token(out: &mut String, token: &str) {
    out.push('"');
    out.push_str(token);
    out.push('"');
}

const fn bool_token(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn push_exec_status(out: &mut String, status: ExecStatus) {
    out.push_str("{\"kind\":");
    push_token(out, status.as_str());
    match status {
        ExecStatus::PreExecFailure { stage, errno } => {
            out.push_str(",\"stage\":");
            push_token(out, stage.as_str());
            out.push_str(",\"errno\":");
            out.push_str(&errno.to_string());
        }
        ExecStatus::Indeterminate(reason) => {
            out.push_str(",\"reason\":");
            push_token(out, reason.as_str());
            if let IndeterminateReason::StatusReadFailed { errno } = reason {
                out.push_str(",\"errno\":");
                out.push_str(&errno.to_string());
            }
        }
    }
    out.push('}');
}

fn push_child_end(out: &mut String, end: ChildEnd) {
    out.push_str("{\"kind\":");
    push_token(out, end.as_str());
    match end {
        ChildEnd::Exited { code } => {
            out.push_str(",\"code\":");
            out.push_str(&code.to_string());
        }
        ChildEnd::Signaled {
            signal,
            core_dumped,
        } => {
            out.push_str(",\"signal\":");
            out.push_str(&signal.to_string());
            out.push_str(",\"core_dumped\":");
            out.push_str(bool_token(core_dumped));
        }
        ChildEnd::EndUnobservable | ChildEnd::EndNotObserved => {}
    }
    out.push('}');
}

fn push_stream(out: &mut String, facts: &StreamFacts) {
    out.push_str("{\"bytes_drained\":");
    out.push_str(&facts.bytes_drained.to_string());
    out.push_str(",\"drained_sha256\":");
    push_digest(out, facts.drained_sha256);
    out.push_str(",\"completeness\":");
    push_token(out, facts.completeness.as_str());
    if let Completeness::ReadFailed { errno } = facts.completeness {
        out.push_str(",\"errno\":");
        out.push_str(&errno.to_string());
    }
    out.push('}');
}

/// Serialise a record in the fixed 0.1 field order (plan section 11.2).
fn serialize(record: &ReceiptRecord) -> Vec<u8> {
    let mut s = String::with_capacity(1024);
    s.push_str("{\"schema\":\"helm-launch-receipt\",\"version\":\"0.1\",\"backend\":");
    push_token(&mut s, record.backend.as_str());
    s.push_str(",\"plan_sha256\":");
    push_digest(&mut s, record.plan_sha256);
    s.push_str(",\"asserted_context\":{\"subject_spec_sha256\":");
    push_optional_digest(&mut s, record.asserted_context.subject_spec_sha256);
    s.push_str(",\"binding_report_sha256\":");
    push_optional_digest(&mut s, record.asserted_context.binding_report_sha256);
    s.push_str("},\"working_directory_id\":");
    // The identifier grammar admits only JSON-safe ASCII; no escaping is needed.
    push_token(&mut s, record.working_directory_id.as_str());
    s.push_str(",\"executable\":{\"pre_exec_body_size\":");
    s.push_str(&record.executable.pre_exec_body_size.to_string());
    s.push_str(",\"pre_exec_body_sha256\":");
    push_digest(&mut s, record.executable.pre_exec_body_sha256);
    s.push_str(",\"pre_exec_mode_bits\":");
    s.push_str(&record.executable.pre_exec_mode_bits.to_string());
    s.push_str(",\"elf_type\":");
    push_token(&mut s, record.executable.elf_type.as_str());
    s.push_str("},\"argument_count\":");
    s.push_str(&record.argument_count.to_string());
    s.push_str(",\"environment_mode\":");
    push_token(&mut s, record.environment_mode.as_str());
    s.push_str(",\"exec_status\":");
    push_exec_status(&mut s, record.exec_status);
    s.push_str(",\"child_end\":");
    push_child_end(&mut s, record.child_end);
    s.push_str(",\"run_deadline_expired\":");
    s.push_str(bool_token(record.run_deadline_expired));
    s.push_str(",\"termination\":{\"sigterm_sent\":");
    s.push_str(bool_token(record.termination.sigterm_sent));
    s.push_str(",\"sigkill_sent\":");
    s.push_str(bool_token(record.termination.sigkill_sent));
    s.push_str(",\"group_sweep\":");
    push_token(&mut s, record.termination.group_sweep.as_str());
    s.push_str("},\"stdout\":");
    push_stream(&mut s, &record.stdout);
    s.push_str(",\"stderr\":");
    push_stream(&mut s, &record.stderr);
    s.push('}');
    s.into_bytes()
}

impl LaunchReceipt {
    #[cfg_attr(
        all(not(test), not(all(target_os = "linux", target_arch = "x86_64"))),
        expect(
            dead_code,
            reason = "off the Linux x86_64 cohort no launch producer exists; only tests serialise                       deterministic test records"
        )
    )]
    pub(crate) fn from_record(record: ReceiptRecord) -> Self {
        let bytes = serialize(&record);
        let sha256 = Digest::of(&bytes);
        Self {
            bytes,
            sha256,
            record,
        }
    }

    /// Exact serialised bytes. Store unchanged: reformatting makes other bytes
    /// with another digest.
    #[must_use]
    pub fn exact_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// SHA-256 over exactly [`Self::exact_bytes`]. It identifies these bytes,
    /// not their origin.
    #[must_use]
    pub const fn sha256(&self) -> Digest {
        self.sha256
    }

    /// The record the bytes were serialised from.
    #[must_use]
    pub const fn record(&self) -> &ReceiptRecord {
        &self.record
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    use sha2::{Digest as _, Sha256};

    use super::{LaunchReceipt, MAX_RECEIPT_BYTES};
    use crate::error::LaunchPlanErrorCode;
    use crate::model::{
        AssertedContext, Backend, CapabilityId, ChildEnd, ChildStage, Completeness, Digest,
        ElfType, EnvironmentMode, ExecStatus, ExecutableMeasurement, GroupSweep,
        IndeterminateReason, MAX_ID_BYTES, ReceiptRecord, Stream, StreamFacts, Termination,
    };
    use crate::plan::{ExecutionKind, StdinMode, TerminationSignal};

    const fn d(b: u8) -> Digest {
        Digest::from_raw([b; 32])
    }

    fn id(text: &str) -> CapabilityId {
        CapabilityId::parse(text).unwrap()
    }

    // ------------------------------------------------ exhaustive variant lists
    //
    // Each list is guarded by a `match` without a wildcard, so adding a variant
    // to the model fails compilation here until the variant is enumerated and
    // therefore covered by the vocabulary, bound and injectivity tests.

    fn all_stages() -> Vec<ChildStage> {
        let all = vec![
            ChildStage::Dup2,
            ChildStage::ClearCloexec,
            ChildStage::Chdir,
            ChildStage::CloseRange,
            ChildStage::Setpgid,
            ChildStage::Sigaction,
            ChildStage::Sigmask,
            ChildStage::NoNewPrivs,
            ChildStage::Exec,
        ];
        for s in &all {
            match s {
                ChildStage::Dup2
                | ChildStage::ClearCloexec
                | ChildStage::Chdir
                | ChildStage::CloseRange
                | ChildStage::Setpgid
                | ChildStage::Sigaction
                | ChildStage::Sigmask
                | ChildStage::NoNewPrivs
                | ChildStage::Exec => {}
            }
        }
        all
    }

    fn all_exec_statuses(errno: i32) -> Vec<ExecStatus> {
        let mut all: Vec<ExecStatus> = all_stages()
            .into_iter()
            .map(|stage| ExecStatus::PreExecFailure { stage, errno })
            .collect();
        for reason in [
            IndeterminateReason::StatusEofWithoutRecord,
            IndeterminateReason::StatusRecordMalformed,
            IndeterminateReason::PreExecStatusTimeout,
            IndeterminateReason::StatusReadFailed { errno },
        ] {
            match reason {
                IndeterminateReason::StatusEofWithoutRecord
                | IndeterminateReason::StatusRecordMalformed
                | IndeterminateReason::PreExecStatusTimeout
                | IndeterminateReason::StatusReadFailed { .. } => {}
            }
            all.push(ExecStatus::Indeterminate(reason));
        }
        for s in &all {
            match s {
                ExecStatus::PreExecFailure { .. } | ExecStatus::Indeterminate(_) => {}
            }
        }
        all
    }

    fn all_child_ends(value: i32) -> Vec<ChildEnd> {
        let all = vec![
            ChildEnd::Exited { code: value },
            ChildEnd::Signaled {
                signal: value,
                core_dumped: false,
            },
            ChildEnd::Signaled {
                signal: value,
                core_dumped: true,
            },
            ChildEnd::EndUnobservable,
            ChildEnd::EndNotObserved,
        ];
        for e in &all {
            match e {
                ChildEnd::Exited { .. }
                | ChildEnd::Signaled { .. }
                | ChildEnd::EndUnobservable
                | ChildEnd::EndNotObserved => {}
            }
        }
        all
    }

    fn all_sweeps() -> Vec<GroupSweep> {
        let all = vec![
            GroupSweep::Issued,
            GroupSweep::NotIssuedGroupNotEstablished,
            GroupSweep::NotIssuedChildAlreadyReaped,
        ];
        for g in &all {
            match g {
                GroupSweep::Issued
                | GroupSweep::NotIssuedGroupNotEstablished
                | GroupSweep::NotIssuedChildAlreadyReaped => {}
            }
        }
        all
    }

    fn all_completeness(errno: i32) -> Vec<Completeness> {
        let all = vec![
            Completeness::CompleteAtEof,
            Completeness::WriterRetainedAfterChildExit,
            Completeness::ReadStoppedChildEndNotObserved,
            Completeness::ReadFailed { errno },
        ];
        for c in &all {
            match c {
                Completeness::CompleteAtEof
                | Completeness::WriterRetainedAfterChildExit
                | Completeness::ReadStoppedChildEndNotObserved
                | Completeness::ReadFailed { .. } => {}
            }
        }
        all
    }

    fn all_elf_types() -> Vec<ElfType> {
        let all = vec![ElfType::EtExec, ElfType::EtDyn];
        for e in &all {
            match e {
                ElfType::EtExec | ElfType::EtDyn => {}
            }
        }
        all
    }

    fn all_error_codes() -> Vec<LaunchPlanErrorCode> {
        use LaunchPlanErrorCode as C;
        let all = vec![
            C::InputTooLarge,
            C::MalformedJson,
            C::DuplicateKey,
            C::NestingTooDeep,
            C::NotJsonObject,
            C::UnknownField,
            C::MissingField,
            C::TypeMismatch,
            C::UnknownSchema,
            C::UnknownVersion,
            C::ExecutionKindUnsupported,
            C::ArgvEmpty,
            C::ArgvTooMany,
            C::ArgTooLong,
            C::ArgvTooLarge,
            C::ArgContainsNul,
            C::EnvironmentModeUnsupported,
            C::StdinModeUnsupported,
            C::CaptureBoundOutOfRange,
            C::TimeoutOutOfRange,
            C::GraceOutOfRange,
            C::TerminationSignalUnsupported,
            C::IdGrammar,
            C::DigestGrammar,
            C::ErrorLimit,
        ];
        for c in &all {
            match c {
                C::InputTooLarge
                | C::MalformedJson
                | C::DuplicateKey
                | C::NestingTooDeep
                | C::NotJsonObject
                | C::UnknownField
                | C::MissingField
                | C::TypeMismatch
                | C::UnknownSchema
                | C::UnknownVersion
                | C::ExecutionKindUnsupported
                | C::ArgvEmpty
                | C::ArgvTooMany
                | C::ArgTooLong
                | C::ArgvTooLarge
                | C::ArgContainsNul
                | C::EnvironmentModeUnsupported
                | C::StdinModeUnsupported
                | C::CaptureBoundOutOfRange
                | C::TimeoutOutOfRange
                | C::GraceOutOfRange
                | C::TerminationSignalUnsupported
                | C::IdGrammar
                | C::DigestGrammar
                | C::ErrorLimit => {}
            }
        }
        all
    }

    /// Every token this crate chooses for itself, other than schema keys.
    fn every_emitted_spelling() -> Vec<String> {
        let mut out: Vec<String> = vec![
            "helm-launch-receipt".into(),
            "helm-launch-plan".into(),
            "0.1".into(),
            Backend::LinuxX8664Clone3PidfdExecveat.as_str().into(),
            EnvironmentMode::Empty.as_str().into(),
            ExecutionKind::LinuxExactExecutable.as_str().into(),
            StdinMode::ClosedPipeEof.as_str().into(),
            TerminationSignal::Sigterm.as_str().into(),
            Stream::Stdout.as_str().into(),
            Stream::Stderr.as_str().into(),
        ];
        out.extend(all_stages().iter().map(|s| s.as_str().to_owned()));
        for status in all_exec_statuses(0) {
            out.push(status.as_str().into());
            if let ExecStatus::Indeterminate(r) = status {
                out.push(r.as_str().into());
            }
        }
        out.extend(all_child_ends(0).iter().map(|e| e.as_str().to_owned()));
        out.extend(all_sweeps().iter().map(|g| g.as_str().to_owned()));
        out.extend(all_completeness(0).iter().map(|c| c.as_str().to_owned()));
        out.extend(all_elf_types().iter().map(|e| e.as_str().to_owned()));
        out.extend(all_error_codes().iter().map(|c| c.as_str().to_owned()));
        out
    }

    // ------------------------------------------------------------- fixtures

    fn fixture_record() -> ReceiptRecord {
        ReceiptRecord {
            backend: Backend::LinuxX8664Clone3PidfdExecveat,
            plan_sha256: d(0x11),
            asserted_context: AssertedContext {
                subject_spec_sha256: Some(d(0x22)),
                binding_report_sha256: None,
            },
            working_directory_id: id("workdir"),
            executable: ExecutableMeasurement {
                pre_exec_body_size: 123_456,
                pre_exec_body_sha256: d(0x33),
                pre_exec_mode_bits: 0o755,
                elf_type: ElfType::EtDyn,
            },
            argument_count: 3,
            environment_mode: EnvironmentMode::Empty,
            exec_status: ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord),
            child_end: ChildEnd::Exited { code: 0 },
            run_deadline_expired: false,
            termination: Termination {
                sigterm_sent: false,
                sigkill_sent: false,
                group_sweep: GroupSweep::Issued,
            },
            stdout: StreamFacts {
                bytes_drained: 512,
                drained_sha256: d(0x44),
                completeness: Completeness::CompleteAtEof,
            },
            stderr: StreamFacts {
                bytes_drained: 0,
                drained_sha256: Digest::of(b""),
                completeness: Completeness::CompleteAtEof,
            },
        }
    }

    const FIXTURE_BYTES: &str = concat!(
        r#"{"schema":"helm-launch-receipt","version":"0.1","#,
        r#""backend":"linux_x86_64_clone3_pidfd_execveat","#,
        r#""plan_sha256":"1111111111111111111111111111111111111111111111111111111111111111","#,
        r#""asserted_context":{"subject_spec_sha256":"2222222222222222222222222222222222222222222222222222222222222222","binding_report_sha256":null},"#,
        r#""working_directory_id":"workdir","#,
        r#""executable":{"pre_exec_body_size":123456,"pre_exec_body_sha256":"3333333333333333333333333333333333333333333333333333333333333333","pre_exec_mode_bits":493,"elf_type":"et_dyn"},"#,
        r#""argument_count":3,"environment_mode":"empty","#,
        r#""exec_status":{"kind":"indeterminate","reason":"status_eof_without_record"},"#,
        r#""child_end":{"kind":"exited","code":0},"#,
        r#""run_deadline_expired":false,"#,
        r#""termination":{"sigterm_sent":false,"sigkill_sent":false,"group_sweep":"issued"},"#,
        r#""stdout":{"bytes_drained":512,"drained_sha256":"4444444444444444444444444444444444444444444444444444444444444444","completeness":"complete_at_eof"},"#,
        r#""stderr":{"bytes_drained":0,"drained_sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","completeness":"complete_at_eof"}}"#,
    );

    /// SHA-256 of `FIXTURE_BYTES`, computed outside Rust (Python `hashlib`) from
    /// the hand-written expected bytes above, not from serializer output.
    const FIXTURE_SHA256: &str = "c2e58e8825b8398fd4b2492f13109caa39a52b461de01c43abe804498525e949";

    fn fixture_record_with_data_variants() -> ReceiptRecord {
        ReceiptRecord {
            backend: Backend::LinuxX8664Clone3PidfdExecveat,
            plan_sha256: d(0xaa),
            asserted_context: AssertedContext {
                subject_spec_sha256: Some(d(0xbb)),
                binding_report_sha256: Some(d(0xcc)),
            },
            working_directory_id: id("cwd.0_a-z"),
            executable: ExecutableMeasurement {
                pre_exec_body_size: 0,
                pre_exec_body_sha256: Digest::of(b""),
                pre_exec_mode_bits: 0,
                elf_type: ElfType::EtExec,
            },
            argument_count: 64,
            environment_mode: EnvironmentMode::Empty,
            exec_status: ExecStatus::PreExecFailure {
                stage: ChildStage::Chdir,
                errno: 13,
            },
            child_end: ChildEnd::Signaled {
                signal: 9,
                core_dumped: true,
            },
            run_deadline_expired: true,
            termination: Termination {
                sigterm_sent: true,
                sigkill_sent: true,
                group_sweep: GroupSweep::NotIssuedChildAlreadyReaped,
            },
            stdout: StreamFacts {
                bytes_drained: 18_446_744_073_709_551_615,
                drained_sha256: d(0xdd),
                completeness: Completeness::ReadFailed { errno: -5 },
            },
            stderr: StreamFacts {
                bytes_drained: 7,
                drained_sha256: d(0xee),
                completeness: Completeness::WriterRetainedAfterChildExit,
            },
        }
    }

    const DATA_VARIANT_BYTES: &str = concat!(
        r#"{"schema":"helm-launch-receipt","version":"0.1","#,
        r#""backend":"linux_x86_64_clone3_pidfd_execveat","#,
        r#""plan_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","#,
        r#""asserted_context":{"subject_spec_sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","binding_report_sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"},"#,
        r#""working_directory_id":"cwd.0_a-z","#,
        r#""executable":{"pre_exec_body_size":0,"pre_exec_body_sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","pre_exec_mode_bits":0,"elf_type":"et_exec"},"#,
        r#""argument_count":64,"environment_mode":"empty","#,
        r#""exec_status":{"kind":"pre_exec_failure","stage":"chdir","errno":13},"#,
        r#""child_end":{"kind":"signaled","signal":9,"core_dumped":true},"#,
        r#""run_deadline_expired":true,"#,
        r#""termination":{"sigterm_sent":true,"sigkill_sent":true,"group_sweep":"not_issued_child_already_reaped"},"#,
        r#""stdout":{"bytes_drained":18446744073709551615,"drained_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","completeness":"read_failed","errno":-5},"#,
        r#""stderr":{"bytes_drained":7,"drained_sha256":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","completeness":"writer_retained_after_child_exit"}}"#,
    );

    /// SHA-256 of `DATA_VARIANT_BYTES`, computed outside Rust (Python `hashlib`).
    const DATA_VARIANT_SHA256: &str =
        "19771d9cd6216735a4d9145ad4e06b6f2ba070b365dfc46004a50436f7bfc52a";

    fn assert_recomputable(receipt: &LaunchReceipt) {
        let independent: [u8; 32] = Sha256::digest(receipt.exact_bytes()).into();
        assert_eq!(receipt.sha256().as_bytes(), &independent);
    }

    // ---------------------------------------------------------------- vectors

    /// The cross-platform determinism anchor. Exact bytes and digest are asserted
    /// here, so a platform-dependent serializer fails rather than producing
    /// different "expected" values per runner. The record is deterministic test
    /// data; it is not a record of any launch and is not authentic.
    #[test]
    fn the_fixture_receipt_has_one_platform_independent_identity() {
        let receipt = LaunchReceipt::from_record(fixture_record());
        println!(
            "HELM-LAUNCH-FIXTURE-RECEIPT: bytes={} sha256={}",
            receipt.exact_bytes().len(),
            receipt.sha256()
        );
        assert_eq!(
            String::from_utf8(receipt.exact_bytes().to_vec()).unwrap(),
            FIXTURE_BYTES
        );
        assert_eq!(receipt.sha256().to_hex(), FIXTURE_SHA256);
        assert_recomputable(&receipt);
        assert_eq!(
            LaunchReceipt::from_record(fixture_record()).exact_bytes(),
            receipt.exact_bytes()
        );
        assert_eq!(receipt.record(), &fixture_record());
    }

    #[test]
    fn data_carrying_variants_have_one_platform_independent_identity() {
        let receipt = LaunchReceipt::from_record(fixture_record_with_data_variants());
        println!(
            "HELM-LAUNCH-DATA-VARIANT-RECEIPT: bytes={} sha256={}",
            receipt.exact_bytes().len(),
            receipt.sha256()
        );
        assert_eq!(
            String::from_utf8(receipt.exact_bytes().to_vec()).unwrap(),
            DATA_VARIANT_BYTES
        );
        assert_eq!(receipt.sha256().to_hex(), DATA_VARIANT_SHA256);
        assert_recomputable(&receipt);
    }

    // ------------------------------------------------------ bound, injectivity

    fn widest_base() -> ReceiptRecord {
        let widest = d(0xff);
        ReceiptRecord {
            backend: Backend::LinuxX8664Clone3PidfdExecveat,
            plan_sha256: widest,
            asserted_context: AssertedContext {
                subject_spec_sha256: Some(widest),
                binding_report_sha256: Some(widest),
            },
            working_directory_id: id(&"w".repeat(MAX_ID_BYTES)),
            executable: ExecutableMeasurement {
                pre_exec_body_size: u64::MAX,
                pre_exec_body_sha256: widest,
                pre_exec_mode_bits: u16::MAX,
                elf_type: ElfType::EtExec,
            },
            argument_count: u32::MAX,
            environment_mode: EnvironmentMode::Empty,
            exec_status: ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord),
            child_end: ChildEnd::EndNotObserved,
            run_deadline_expired: false,
            termination: Termination {
                sigterm_sent: false,
                sigkill_sent: false,
                group_sweep: GroupSweep::Issued,
            },
            stdout: StreamFacts {
                bytes_drained: u64::MAX,
                drained_sha256: widest,
                completeness: Completeness::CompleteAtEof,
            },
            stderr: StreamFacts {
                bytes_drained: u64::MAX,
                drained_sha256: widest,
                completeness: Completeness::CompleteAtEof,
            },
        }
    }

    /// Every combination of fact variants, at the widest numeric values, stays
    /// inside the ceiling; and distinct records never share bytes.
    #[test]
    fn every_variant_combination_is_bounded_injective_and_recomputable() {
        let mut seen_bytes: HashSet<Vec<u8>> = HashSet::new();
        let mut widest = 0usize;
        let mut count = 0usize;
        for exec_status in all_exec_statuses(i32::MIN) {
            for child_end in all_child_ends(i32::MIN) {
                for group_sweep in all_sweeps() {
                    for out in all_completeness(i32::MIN) {
                        for err in all_completeness(i32::MIN) {
                            let mut r = widest_base();
                            r.exec_status = exec_status;
                            r.child_end = child_end;
                            r.termination.group_sweep = group_sweep;
                            r.stdout.completeness = out;
                            r.stderr.completeness = err;
                            let receipt = LaunchReceipt::from_record(r);
                            widest = widest.max(receipt.exact_bytes().len());
                            assert!(seen_bytes.insert(receipt.exact_bytes().to_vec()));
                            count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(count, 13 * 5 * 3 * 4 * 4);
        println!("HELM-LAUNCH-WIDEST-RECEIPT: bytes={widest} ceiling={MAX_RECEIPT_BYTES}");
        assert!(
            widest < MAX_RECEIPT_BYTES,
            "widest receipt is {widest} bytes"
        );

        // The remaining axes, each changed on its own, also change the bytes.
        let base = LaunchReceipt::from_record(widest_base());
        let mut variants: Vec<ReceiptRecord> = Vec::new();
        let mut r = widest_base();
        r.run_deadline_expired = true;
        variants.push(r);
        let mut r = widest_base();
        r.termination.sigterm_sent = true;
        variants.push(r);
        let mut r = widest_base();
        r.termination.sigkill_sent = true;
        variants.push(r);
        let mut r = widest_base();
        r.executable.elf_type = ElfType::EtDyn;
        variants.push(r);
        let mut r = widest_base();
        r.asserted_context.subject_spec_sha256 = None;
        variants.push(r);
        let mut r = widest_base();
        r.asserted_context.binding_report_sha256 = None;
        variants.push(r);
        let mut r = widest_base();
        std::mem::swap(&mut r.stdout, &mut r.stderr);
        r.stdout.bytes_drained = 1;
        variants.push(r);
        let mut all = HashSet::new();
        all.insert(base.exact_bytes().to_vec());
        for v in variants {
            let receipt = LaunchReceipt::from_record(v);
            assert_recomputable(&receipt);
            assert!(all.insert(receipt.exact_bytes().to_vec()));
        }
    }

    // ------------------------------------------------------------- vocabulary

    /// Forbidden semantic vocabulary (ADR-0024 section L, plan section 10.4,
    /// and the P1 owner instruction). Compared against whole emitted terms and
    /// against each `_`/`-`/`.` separated word inside them. Words such as
    /// `failure` or `failed`, which name an observed failed operation in the
    /// accepted vocabulary (`pre_exec_failure`, `read_failed`), are distinct
    /// words and are not verdicts.
    const FORBIDDEN: [&str; 18] = [
        "pass",
        "passed",
        "fail",
        "ok",
        "success",
        "successful",
        "succeeded",
        "ready",
        "compatible",
        "verified",
        "worked",
        "launched",
        "sandboxed",
        "contained",
        "safe",
        "authentic",
        "signed",
        "trusted",
    ];

    fn is_verdict(term: &str) -> bool {
        let lower = term.to_ascii_lowercase();
        FORBIDDEN.contains(&lower.as_str())
            || lower
                .split(['_', '-', '.'])
                .any(|word| FORBIDDEN.contains(&word))
    }

    fn assert_not_verdict(term: &str) {
        assert!(!is_verdict(term), "verdict vocabulary emitted: {term}");
    }

    /// Every JSON object key in serialised bytes, in order of appearance.
    fn keys_of(bytes: &[u8]) -> BTreeSet<String> {
        fn walk(v: &serde_json::Value, out: &mut BTreeSet<String>) {
            match v {
                serde_json::Value::Object(map) => {
                    for (k, child) in map {
                        out.insert(k.clone());
                        walk(child, out);
                    }
                }
                serde_json::Value::Array(items) => items.iter().for_each(|i| walk(i, out)),
                _ => {}
            }
        }
        let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        let mut out = BTreeSet::new();
        walk(&value, &mut out);
        out
    }

    #[test]
    fn no_emitted_spelling_or_schema_key_is_a_verdict() {
        for term in every_emitted_spelling() {
            assert_not_verdict(&term);
        }
        let mut keys = BTreeSet::new();
        for exec_status in all_exec_statuses(1) {
            for child_end in all_child_ends(1) {
                for completeness in all_completeness(1) {
                    let mut r = widest_base();
                    r.exec_status = exec_status;
                    r.child_end = child_end;
                    r.stdout.completeness = completeness;
                    keys.extend(keys_of(LaunchReceipt::from_record(r).exact_bytes()));
                }
            }
        }
        for key in &keys {
            assert_not_verdict(key);
        }
        // The closed key set: no timestamp, duration, pid, descriptor, path,
        // output payload, prefix or authenticity field exists.
        let expected: BTreeSet<String> = [
            "schema",
            "version",
            "backend",
            "plan_sha256",
            "asserted_context",
            "subject_spec_sha256",
            "binding_report_sha256",
            "working_directory_id",
            "executable",
            "pre_exec_body_size",
            "pre_exec_body_sha256",
            "pre_exec_mode_bits",
            "elf_type",
            "argument_count",
            "environment_mode",
            "exec_status",
            "kind",
            "stage",
            "errno",
            "reason",
            "child_end",
            "code",
            "signal",
            "core_dumped",
            "run_deadline_expired",
            "termination",
            "sigterm_sent",
            "sigkill_sent",
            "group_sweep",
            "stdout",
            "stderr",
            "bytes_drained",
            "drained_sha256",
            "completeness",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(keys, expected);
    }

    #[test]
    fn the_guard_itself_rejects_verdict_terms() {
        for term in [
            "success",
            "exec_succeeded",
            "ready-to-run",
            "PASS",
            "result.ok",
            "not_contained",
        ] {
            assert!(is_verdict(term), "guard accepted {term}");
        }
        for term in [
            "pre_exec_failure",
            "read_failed",
            "complete_at_eof",
            "passthrough",
        ] {
            assert!(
                !is_verdict(term),
                "guard rejected accepted vocabulary {term}"
            );
        }
    }

    #[test]
    fn serialised_receipts_are_strict_json_without_duplicate_keys() {
        for record in [
            fixture_record(),
            fixture_record_with_data_variants(),
            widest_base(),
        ] {
            let receipt = LaunchReceipt::from_record(record);
            // The plan parser's strict tree refuses duplicate decoded keys and
            // over-deep nesting; the receipt writer must satisfy it.
            assert!(crate::plan::is_strict_json(receipt.exact_bytes()));
            let text = String::from_utf8(receipt.exact_bytes().to_vec()).unwrap();
            assert!(text.is_ascii());
            assert!(!text.contains(char::is_whitespace));
        }
    }

    #[test]
    fn debug_output_names_the_digest_and_never_dumps_bytes() {
        let receipt = LaunchReceipt::from_record(fixture_record());
        let debug = format!("{receipt:?}");
        assert!(debug.contains(&receipt.sha256().to_hex()));
        assert!(!debug.contains("123, 34"), "raw byte dump in {debug}");
    }

    // =====================================================================
    // P5 — published receipt evidence contract
    //
    // `docs/implementation/helm-launch-receipt-0.1-test-vectors.json` is the
    // published artifact. These records are its only source of truth: the
    // vectors are produced by the **production** serializer below, never by a
    // shadow writer, and the test recomputes SHA-256 from the committed bytes
    // themselves rather than trusting a stored digest.
    //
    // Level 1: no process, no descriptor, no host state, so it runs unchanged
    // on Linux, Windows and macOS and must produce identical bytes on each.
    // =====================================================================

    /// The published vector file, relative to this crate's manifest.
    const VECTORS_JSON: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/implementation/helm-launch-receipt-0.1-test-vectors.json"
    ));

    fn hex_lower(bytes: &[u8]) -> String {
        let mut text = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            text.push_str(&format!("{byte:02x}"));
        }
        text
    }

    fn unhex(text: &str) -> Vec<u8> {
        assert!(text.len().is_multiple_of(2), "odd hex run in a vector");
        (0..text.len() / 2)
            .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("hex pair"))
            .collect()
    }

    /// A fact shape with no exec-success reading anywhere; `base` is the only
    /// place the shared, non-varying fields are written.
    fn vector_base() -> ReceiptRecord {
        ReceiptRecord {
            backend: Backend::LinuxX8664Clone3PidfdExecveat,
            plan_sha256: d(0x01),
            asserted_context: AssertedContext {
                subject_spec_sha256: None,
                binding_report_sha256: None,
            },
            working_directory_id: id("vector-workdir"),
            executable: ExecutableMeasurement {
                pre_exec_body_size: 4_096,
                pre_exec_body_sha256: d(0x02),
                pre_exec_mode_bits: 0o755,
                elf_type: ElfType::EtDyn,
            },
            argument_count: 2,
            environment_mode: EnvironmentMode::Empty,
            exec_status: ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord),
            child_end: ChildEnd::Exited { code: 0 },
            run_deadline_expired: false,
            termination: Termination {
                sigterm_sent: false,
                sigkill_sent: false,
                group_sweep: GroupSweep::NotIssuedGroupNotEstablished,
            },
            stdout: StreamFacts {
                bytes_drained: 11,
                drained_sha256: d(0x03),
                completeness: Completeness::CompleteAtEof,
            },
            stderr: StreamFacts {
                bytes_drained: 0,
                drained_sha256: Digest::of(b""),
                completeness: Completeness::CompleteAtEof,
            },
        }
    }

    /// The published vector set, in published order.
    ///
    /// There is deliberately **no** exec-success vector: no such value exists
    /// in the model, and a vector claiming one could not be produced here.
    fn published_vectors() -> Vec<(&'static str, ReceiptRecord)> {
        let mut out: Vec<(&'static str, ReceiptRecord)> = Vec::new();

        out.push(("normal_direct_child_exit", vector_base()));

        let mut pre_exec = vector_base();
        pre_exec.exec_status = ExecStatus::PreExecFailure {
            stage: ChildStage::Exec,
            errno: 13,
        };
        pre_exec.child_end = ChildEnd::Exited { code: 127 };
        pre_exec.stdout = StreamFacts {
            bytes_drained: 0,
            drained_sha256: Digest::of(b""),
            completeness: Completeness::CompleteAtEof,
        };
        out.push(("pre_exec_failure_exec_eacces", pre_exec));

        let mut eof = vector_base();
        eof.exec_status = ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord);
        eof.child_end = ChildEnd::Exited { code: 3 };
        out.push(("status_eof_without_record_indeterminate", eof));

        let mut deadline = vector_base();
        deadline.run_deadline_expired = true;
        deadline.termination = Termination {
            sigterm_sent: true,
            sigkill_sent: true,
            group_sweep: GroupSweep::NotIssuedGroupNotEstablished,
        };
        deadline.child_end = ChildEnd::Signaled {
            signal: 9,
            core_dumped: false,
        };
        out.push(("run_deadline_sigterm_then_sigkill", deadline));

        let mut swept = vector_base();
        swept.termination = Termination {
            sigterm_sent: true,
            sigkill_sent: false,
            group_sweep: GroupSweep::Issued,
        };
        swept.child_end = ChildEnd::Exited { code: 0 };
        out.push(("group_sweep_issued", swept));

        // The deliberate contrast with `group_sweep_issued`: the same
        // termination facts, and the sweep withheld only because the parent's
        // own `setpgid` never established authority (T31).
        let mut no_authority = vector_base();
        no_authority.termination = Termination {
            sigterm_sent: true,
            sigkill_sent: false,
            group_sweep: GroupSweep::NotIssuedGroupNotEstablished,
        };
        out.push(("group_sweep_not_issued_group_not_established", no_authority));

        let mut foreign = vector_base();
        foreign.child_end = ChildEnd::EndUnobservable;
        foreign.termination = Termination {
            sigterm_sent: false,
            sigkill_sent: false,
            group_sweep: GroupSweep::NotIssuedChildAlreadyReaped,
        };
        out.push(("foreign_reaped_end_unobservable", foreign));

        let mut not_observed = vector_base();
        not_observed.child_end = ChildEnd::EndNotObserved;
        not_observed.termination = Termination {
            sigterm_sent: true,
            sigkill_sent: true,
            group_sweep: GroupSweep::Issued,
        };
        not_observed.stdout = StreamFacts {
            bytes_drained: 7,
            drained_sha256: d(0x04),
            completeness: Completeness::ReadStoppedChildEndNotObserved,
        };
        out.push(("end_not_observed_latched", not_observed));

        let mut streams = vector_base();
        streams.stdout = StreamFacts {
            bytes_drained: 64,
            drained_sha256: d(0x05),
            completeness: Completeness::WriterRetainedAfterChildExit,
        };
        streams.stderr = StreamFacts {
            bytes_drained: 32,
            drained_sha256: d(0x06),
            completeness: Completeness::ReadFailed { errno: 5 },
        };
        out.push(("stream_completeness_variants", streams));

        let mut signaled = vector_base();
        signaled.child_end = ChildEnd::Signaled {
            signal: 11,
            core_dumped: true,
        };
        out.push(("signaled_child_keeps_core_flag", signaled));

        let mut widest = widest_base();
        widest.exec_status =
            ExecStatus::Indeterminate(IndeterminateReason::StatusReadFailed { errno: i32::MIN });
        widest.child_end = ChildEnd::Signaled {
            signal: i32::MIN,
            core_dumped: true,
        };
        widest.run_deadline_expired = true;
        widest.termination = Termination {
            sigterm_sent: true,
            sigkill_sent: true,
            group_sweep: GroupSweep::NotIssuedChildAlreadyReaped,
        };
        widest.stdout = StreamFacts {
            bytes_drained: u64::MAX,
            drained_sha256: d(0xff),
            completeness: Completeness::ReadFailed { errno: i32::MIN },
        };
        out.push(("widest_near_max_receipt", widest));

        out
    }

    /// Every published vector is exactly what the production serializer emits,
    /// and its digest is recomputable from the published bytes alone.
    ///
    /// This is the `R3-M1` regression: a published digest that cannot be
    /// recomputed from the published artifact proves nothing.
    #[test]
    fn published_receipt_vectors_match_the_production_serializer() {
        let document: serde_json::Value =
            serde_json::from_str(VECTORS_JSON).expect("the published vector file is strict JSON");

        assert_eq!(
            document["schema"].as_str(),
            Some("helm-launch-receipt-test-vectors"),
            "the vector file changed schema"
        );
        assert_eq!(document["version"].as_str(), Some("0.1"));
        assert_eq!(
            document["receipt_schema"].as_str(),
            Some("helm-launch-receipt")
        );
        assert_eq!(document["receipt_version"].as_str(), Some("0.1"));
        assert_eq!(
            document["max_receipt_bytes"].as_u64(),
            Some(MAX_RECEIPT_BYTES as u64),
            "the published bound drifted from the product constant"
        );
        assert_eq!(
            document["authenticity"].as_str(),
            Some("none"),
            "the vector file must keep stating that it carries no authenticity claim"
        );

        let published = document["vectors"]
            .as_array()
            .expect("vectors is an array")
            .clone();
        let expected = published_vectors();
        assert_eq!(
            published.len(),
            expected.len(),
            "the published vector count and the in-crate set disagree"
        );

        let published_names: Vec<&str> = published
            .iter()
            .map(|v| v["name"].as_str().expect("vector name"))
            .collect();
        let expected_names: Vec<&str> = expected.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            published_names, expected_names,
            "published vector names and order must match the in-crate set exactly; \
             no vector may be withheld"
        );

        let mut digests: BTreeSet<String> = BTreeSet::new();
        for value in &published {
            let digest = value["sha256"].as_str().expect("sha256").to_owned();
            assert!(
                digests.insert(digest.clone()),
                "two published vectors share the digest {digest}; a vector is redundant"
            );
        }

        for ((name, record), value) in expected.into_iter().zip(published.iter()) {
            let receipt = LaunchReceipt::from_record(record);
            let produced = receipt.exact_bytes();

            // 1. The unambiguous encoding is authoritative.
            let committed_hex = value["exact_bytes_base16"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: missing exact_bytes_base16"));
            let committed = unhex(committed_hex);
            assert_eq!(
                committed,
                produced.to_vec(),
                "{name}: published bytes differ from the production serializer"
            );

            // 2. The readable copy must decode to exactly the same bytes, so a
            //    reviewer reading the JSON sees the real receipt and not a
            //    differently escaped near-miss.
            let readable = value["exact_bytes_utf8"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: missing exact_bytes_utf8"));
            assert_eq!(
                readable.as_bytes(),
                produced,
                "{name}: the readable copy is not the same bytes as the base16 copy"
            );

            // 3. The length is published and is the real one.
            assert_eq!(
                value["exact_byte_length"].as_u64(),
                Some(produced.len() as u64),
                "{name}: published length is wrong"
            );

            // 4. The digest is recomputed from the COMMITTED bytes, not taken
            //    on trust and not taken from the in-memory receipt.
            let recomputed = hex_lower(&Sha256::digest(&committed));
            let committed_digest = value["sha256"]
                .as_str()
                .unwrap_or_else(|| panic!("{name}: missing sha256"));
            assert_eq!(
                recomputed, committed_digest,
                "{name}: the published digest is not sha256 of the published bytes"
            );
            assert_eq!(
                recomputed,
                receipt.sha256().to_hex(),
                "{name}: the product digest and the published digest disagree"
            );

            // 5. The published bound holds, with no truncation.
            assert!(
                produced.len() <= MAX_RECEIPT_BYTES,
                "{name}: {} bytes exceeds MAX_RECEIPT_BYTES {MAX_RECEIPT_BYTES}",
                produced.len()
            );

            // 6. Nothing in a published vector may read as success.
            let text = String::from_utf8(produced.to_vec()).expect("receipts are ASCII");
            for term in ["success", "succeeded", "verified", "authentic", "trusted"] {
                assert!(
                    !text.contains(term),
                    "{name}: published vector contains the verdict term {term:?}"
                );
            }
        }
    }

    /// The widest vector the published set carries still fits the accepted
    /// bound, and the bound itself is the accepted constant.
    #[test]
    fn the_published_vectors_respect_the_accepted_receipt_bound() {
        assert_eq!(MAX_RECEIPT_BYTES, 8_192, "the accepted bound changed");
        let widest = published_vectors()
            .into_iter()
            .map(|(name, record)| (name, LaunchReceipt::from_record(record).exact_bytes().len()))
            .max_by_key(|(_, len)| *len)
            .expect("the vector set is not empty");
        assert!(
            widest.1 <= MAX_RECEIPT_BYTES,
            "widest published vector {} is {} bytes",
            widest.0,
            widest.1
        );
    }

    // =====================================================================
    // P5 — the published schema's machine-checkable normative contract
    //
    // `docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md` publishes the receipt
    // evidence contract. Its prose is documentation; the delimited contract
    // block inside it is not. This section reads the **committed document**,
    // parses that block, and checks every line against product truth: the
    // production constants, the production `as_str()` vocabularies enumerated
    // by wildcard-free `match`es, and the key order read back out of the
    // production serializer's own bytes and out of the committed vectors.
    //
    // There is no second serializer here. The reader below consumes receipt
    // bytes and reports the order of the keys it finds; nothing in this
    // section can produce a receipt byte.
    //
    // Level 1: no process, no descriptor, no host state, so it runs on Linux,
    // Windows and macOS in the ordinary suite.
    // =====================================================================

    /// The published schema document, relative to this crate's manifest.
    const SCHEMA_DOC: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/implementation/HELM-LAUNCH-RECEIPT-0.1.md"
    ));

    const CONTRACT_BEGIN: &str = "<!-- HELM-LAUNCH-RECEIPT-CONTRACT:BEGIN -->";
    const CONTRACT_END: &str = "<!-- HELM-LAUNCH-RECEIPT-CONTRACT:END -->";

    /// Every key the contract block must carry, and no other.
    const CONTRACT_KEYS: [&str; 34] = [
        "schema",
        "version",
        "max_receipt_bytes",
        "encoding",
        "formatting_whitespace",
        "trailing_newline",
        "bool_literals",
        "null_literal",
        "digest_encoding",
        "argument_count_encoding",
        "backend_values",
        "environment_mode_values",
        "elf_type_values",
        "child_stage_values",
        "indeterminate_reason_values",
        "exec_status_kinds",
        "child_end_kinds",
        "group_sweep_values",
        "stream_completeness_values",
        "top_level_order",
        "asserted_context_order",
        "asserted_context_nullable",
        "executable_order",
        "termination_order",
        "termination_bool_fields",
        "stream_order",
        "stream_order_read_failed",
        "exec_status_order_pre_exec_failure",
        "exec_status_order_indeterminate",
        "exec_status_order_indeterminate_status_read_failed",
        "child_end_order_exited",
        "child_end_order_signaled",
        "child_end_order_end_unobservable",
        "child_end_order_end_not_observed",
    ];

    /// Parse the one contract block out of the published document.
    ///
    /// Fails on a missing or repeated marker, a malformed line, a duplicate
    /// key, an unknown key or a missing key, so the block cannot be quietly
    /// extended, reordered or hollowed out.
    fn contract() -> BTreeMap<String, String> {
        assert_eq!(
            SCHEMA_DOC.matches(CONTRACT_BEGIN).count(),
            1,
            "the schema document must carry exactly one contract begin marker"
        );
        assert_eq!(
            SCHEMA_DOC.matches(CONTRACT_END).count(),
            1,
            "the schema document must carry exactly one contract end marker"
        );
        let after = SCHEMA_DOC
            .split_once(CONTRACT_BEGIN)
            .expect("begin marker")
            .1;
        let body = after.split_once(CONTRACT_END).expect("end marker").0;

        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for line in body.lines() {
            let line = line.trim_end_matches('\r');
            if line.is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .unwrap_or_else(|| panic!("contract line is not `key=value`: {line:?}"));
            assert!(
                !key.is_empty() && !value.is_empty(),
                "contract line has an empty side: {line:?}"
            );
            assert!(
                CONTRACT_KEYS.contains(&key),
                "the contract block carries the unknown key {key:?}"
            );
            assert!(
                map.insert(key.to_owned(), value.to_owned()).is_none(),
                "the contract block repeats the key {key:?}"
            );
        }
        for key in CONTRACT_KEYS {
            assert!(
                map.contains_key(key),
                "the contract block is missing the required key {key:?}"
            );
        }
        map
    }

    /// One contract value as its comma-separated list.
    fn listed<'a>(contract: &'a BTreeMap<String, String>, key: &str) -> Vec<&'a str> {
        contract
            .get(key)
            .unwrap_or_else(|| panic!("missing contract key {key}"))
            .split(',')
            .collect()
    }

    /// The ordered `(key, value offset)` members of the compact object that
    /// starts at `at`, and the offset just past that object.
    ///
    /// This is a **reader**. Receipts are compact ASCII with no array and no
    /// escape sequence — which [`the_published_schema_contract_matches_the_production_serializer`]
    /// asserts separately, from the same bytes — so it needs no general JSON
    /// machinery, and anything it does not recognise fails loudly rather than
    /// being guessed at.
    fn members(bytes: &[u8], at: usize) -> (Vec<(String, usize)>, usize) {
        assert_eq!(bytes[at], b'{', "an object must open with a brace");
        let mut found: Vec<(String, usize)> = Vec::new();
        let mut i = at + 1;
        if bytes[i] == b'}' {
            return (found, i + 1);
        }
        loop {
            assert_eq!(bytes[i], b'"', "a member must open with a quoted key");
            let start = i + 1;
            let mut end = start;
            while bytes[end] != b'"' {
                assert_ne!(bytes[end], b'\\', "no escape is reachable in a receipt");
                end += 1;
            }
            let key = String::from_utf8(bytes[start..end].to_vec()).expect("an ASCII key");
            i = end + 1;
            assert_eq!(bytes[i], b':', "a key must be followed by a colon");
            i += 1;
            found.push((key, i));
            i = match bytes[i] {
                b'{' => members(bytes, i).1,
                b'"' => {
                    let mut j = i + 1;
                    while bytes[j] != b'"' {
                        assert_ne!(bytes[j], b'\\', "no escape is reachable in a receipt");
                        j += 1;
                    }
                    j + 1
                }
                b'[' => panic!("no array is part of the receipt contract"),
                _ => {
                    let mut j = i;
                    while !matches!(bytes[j], b',' | b'}') {
                        j += 1;
                    }
                    j
                }
            };
            match bytes[i] {
                b',' => i += 1,
                b'}' => return (found, i + 1),
                other => panic!("unexpected byte {other:?} after a member"),
            }
        }
    }

    /// The ordered keys of the object reached by `path` from the receipt root.
    fn keys_at(bytes: &[u8], path: &[&str]) -> Vec<String> {
        let mut at = 0usize;
        for step in path {
            let (found, _) = members(bytes, at);
            let (_, value) = found
                .iter()
                .find(|(key, _)| key == step)
                .unwrap_or_else(|| panic!("the receipt has no field {step:?}"));
            at = *value;
        }
        members(bytes, at).0.into_iter().map(|(k, _)| k).collect()
    }

    /// The raw value text of a top-level field, exactly as emitted.
    fn raw_value(bytes: &[u8], field: &str) -> String {
        let (found, _) = members(bytes, 0);
        let (_, at) = found
            .iter()
            .find(|(key, _)| key == field)
            .unwrap_or_else(|| panic!("the receipt has no field {field:?}"));
        let end = members(bytes, 0).0;
        let _ = end;
        let mut j = *at;
        let stop = match bytes[j] {
            b'"' => {
                let mut k = j + 1;
                while bytes[k] != b'"' {
                    k += 1;
                }
                k + 1
            }
            b'{' => members(bytes, j).1,
            _ => {
                let mut k = j;
                while !matches!(bytes[k], b',' | b'}') {
                    k += 1;
                }
                k
            }
        };
        j = *at;
        String::from_utf8(bytes[j..stop].to_vec()).expect("ASCII value")
    }

    /// Every record shape the contract names an order for, each one built from
    /// the helpers the accepted tests already use. No new receipt shape is
    /// invented here.
    fn contract_witnesses() -> Vec<ReceiptRecord> {
        let mut out: Vec<ReceiptRecord> = published_vectors()
            .into_iter()
            .map(|(_, record)| record)
            .collect();
        // Both asserted-context states, and every exec-status, child-end and
        // completeness shape, at the widest numeric values.
        out.push(vector_base());
        out.push(widest_base());
        for status in all_exec_statuses(i32::MIN) {
            let mut r = widest_base();
            r.exec_status = status;
            out.push(r);
        }
        for end in all_child_ends(i32::MIN) {
            let mut r = widest_base();
            r.child_end = end;
            out.push(r);
        }
        for completeness in all_completeness(i32::MIN) {
            let mut r = widest_base();
            r.stdout.completeness = completeness;
            r.stderr.completeness = completeness;
            out.push(r);
        }
        for sweep in all_sweeps() {
            let mut r = widest_base();
            r.termination.group_sweep = sweep;
            out.push(r);
        }
        for elf_type in all_elf_types() {
            let mut r = widest_base();
            r.executable.elf_type = elf_type;
            out.push(r);
        }
        out
    }

    /// The published schema's normative contract block is true of the product.
    ///
    /// This is the `P5R-01B` enforcement: a published evidence contract whose
    /// machine-checkable core nothing checks is not a contract. Every line of
    /// the block is compared against a production constant, a production
    /// `as_str()` vocabulary, or the key order read back out of bytes the
    /// production serializer emitted — never against prose.
    #[test]
    fn the_published_schema_contract_matches_the_production_serializer() {
        let contract = contract();

        // ---------------------------------------------- 1. constants
        assert_eq!(
            contract["max_receipt_bytes"],
            MAX_RECEIPT_BYTES.to_string(),
            "the published bound drifted from the product constant"
        );

        // ---------------------------------------------- 2. vocabularies
        //
        // Each expectation comes from the exhaustive, wildcard-free list the
        // accepted tests already use, so a new model variant fails compilation
        // before it can go unlisted here.
        let vocabularies: [(&str, Vec<String>); 9] = [
            (
                "backend_values",
                vec![Backend::LinuxX8664Clone3PidfdExecveat.as_str().to_owned()],
            ),
            (
                "environment_mode_values",
                vec![EnvironmentMode::Empty.as_str().to_owned()],
            ),
            (
                "elf_type_values",
                all_elf_types()
                    .iter()
                    .map(|e| e.as_str().to_owned())
                    .collect(),
            ),
            (
                "child_stage_values",
                all_stages().iter().map(|s| s.as_str().to_owned()).collect(),
            ),
            (
                "indeterminate_reason_values",
                all_exec_statuses(0)
                    .into_iter()
                    .filter_map(|s| match s {
                        ExecStatus::Indeterminate(r) => Some(r.as_str().to_owned()),
                        ExecStatus::PreExecFailure { .. } => None,
                    })
                    .collect(),
            ),
            (
                "exec_status_kinds",
                vec![
                    ExecStatus::PreExecFailure {
                        stage: ChildStage::Exec,
                        errno: 0,
                    }
                    .as_str()
                    .to_owned(),
                    ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord)
                        .as_str()
                        .to_owned(),
                ],
            ),
            (
                "child_end_kinds",
                all_child_ends(0)
                    .iter()
                    .map(|e| e.as_str().to_owned())
                    .fold(Vec::new(), |mut acc, s| {
                        if !acc.contains(&s) {
                            acc.push(s);
                        }
                        acc
                    }),
            ),
            (
                "group_sweep_values",
                all_sweeps().iter().map(|g| g.as_str().to_owned()).collect(),
            ),
            (
                "stream_completeness_values",
                all_completeness(0)
                    .iter()
                    .map(|c| c.as_str().to_owned())
                    .collect(),
            ),
        ];
        for (key, expected) in vocabularies {
            assert_eq!(
                listed(&contract, key),
                expected.iter().map(String::as_str).collect::<Vec<_>>(),
                "the contract's {key} disagrees with the production vocabulary"
            );
        }

        // ---------------------------------------------- 3. field order
        //
        // Read back out of production bytes, for every shape, and separately
        // out of the committed vector bytes further down.
        for record in contract_witnesses() {
            let exec_status = record.exec_status;
            let child_end = record.child_end;
            let stdout = record.stdout.completeness;
            let stderr = record.stderr.completeness;
            let nullable = record.asserted_context.subject_spec_sha256.is_none();
            let receipt = LaunchReceipt::from_record(record);
            let bytes = receipt.exact_bytes();

            assert_eq!(
                keys_at(bytes, &[]),
                listed(&contract, "top_level_order"),
                "top-level field order"
            );
            assert_eq!(
                keys_at(bytes, &["asserted_context"]),
                listed(&contract, "asserted_context_order")
            );
            assert_eq!(
                keys_at(bytes, &["executable"]),
                listed(&contract, "executable_order")
            );
            assert_eq!(
                keys_at(bytes, &["termination"]),
                listed(&contract, "termination_order")
            );
            for stream in [Stream::Stdout, Stream::Stderr] {
                let completeness = match stream {
                    Stream::Stdout => stdout,
                    Stream::Stderr => stderr,
                };
                let key = if matches!(completeness, Completeness::ReadFailed { .. }) {
                    "stream_order_read_failed"
                } else {
                    "stream_order"
                };
                assert_eq!(
                    keys_at(bytes, &[stream.as_str()]),
                    listed(&contract, key),
                    "{} field order for {completeness:?}",
                    stream.as_str()
                );
            }
            let exec_key = match exec_status {
                ExecStatus::PreExecFailure { .. } => "exec_status_order_pre_exec_failure",
                ExecStatus::Indeterminate(IndeterminateReason::StatusReadFailed { .. }) => {
                    "exec_status_order_indeterminate_status_read_failed"
                }
                ExecStatus::Indeterminate(_) => "exec_status_order_indeterminate",
            };
            assert_eq!(
                keys_at(bytes, &["exec_status"]),
                listed(&contract, exec_key),
                "exec_status field order for {exec_status:?}"
            );
            let end_key = match child_end {
                ChildEnd::Exited { .. } => "child_end_order_exited",
                ChildEnd::Signaled { .. } => "child_end_order_signaled",
                ChildEnd::EndUnobservable => "child_end_order_end_unobservable",
                ChildEnd::EndNotObserved => "child_end_order_end_not_observed",
            };
            assert_eq!(
                keys_at(bytes, &["child_end"]),
                listed(&contract, end_key),
                "child_end field order for {child_end:?}"
            );

            // ------------------------------------------ 4. byte contract
            assert!(
                bytes.iter().all(u8::is_ascii),
                "the contract declares UTF-8 and the serializer emits ASCII"
            );
            assert_eq!(contract["encoding"], "utf8");
            assert!(
                !bytes.iter().any(u8::is_ascii_whitespace),
                "the contract declares no formatting whitespace"
            );
            assert_eq!(contract["formatting_whitespace"], "none");
            assert_ne!(
                bytes.last(),
                Some(&b'\n'),
                "the contract declares no trailing newline"
            );
            assert_eq!(contract["trailing_newline"], "absent");

            // Literals, read out of the emitted values themselves.
            assert_eq!(listed(&contract, "bool_literals"), vec!["true", "false"]);
            for field in listed(&contract, "termination_bool_fields") {
                let (found, _) = members(bytes, 0);
                let (_, at) = found
                    .iter()
                    .find(|(key, _)| key == "termination")
                    .expect("termination");
                let (inner, _) = members(bytes, *at);
                let (_, value) = inner
                    .iter()
                    .find(|(key, _)| key == field)
                    .unwrap_or_else(|| panic!("termination has no {field}"));
                let text: String = bytes[*value..]
                    .iter()
                    .take_while(|b| b.is_ascii_alphabetic())
                    .map(|b| char::from(*b))
                    .collect();
                assert!(
                    listed(&contract, "bool_literals").contains(&text.as_str()),
                    "{field} emitted {text:?}, which is not a declared boolean literal"
                );
            }
            assert_eq!(contract["null_literal"], "null");
            let asserted = raw_value(bytes, "asserted_context");
            if nullable {
                assert!(
                    asserted.contains(&contract["null_literal"]),
                    "an absent asserted digest must emit the declared null literal"
                );
            }
            assert_eq!(contract["digest_encoding"], "lowercase_hex_64");
            let plan_digest = raw_value(bytes, "plan_sha256");
            let hex = plan_digest.trim_matches('"');
            assert_eq!(hex.len(), 64, "a digest is 64 characters");
            assert!(
                hex.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "a digest is lowercase hexadecimal"
            );
            assert_eq!(contract["argument_count_encoding"], "decimal_u32");
            let count = raw_value(bytes, "argument_count");
            assert!(
                count.parse::<u32>().is_ok(),
                "argument_count emitted {count:?}, which is not a decimal u32"
            );
            assert_eq!(
                raw_value(bytes, "schema").trim_matches('"'),
                contract["schema"]
            );
            assert_eq!(
                raw_value(bytes, "version").trim_matches('"'),
                contract["version"]
            );
        }

        // ---------------------------------------------- 5. the committed bytes
        //
        // The third leg of the link the contract rests on: the schema declares
        // an order, the production serializer emits it, and the **committed**
        // artifact preserves it. Read from the published file, not from a
        // freshly serialised record.
        let document: serde_json::Value =
            serde_json::from_str(VECTORS_JSON).expect("the published vector file is strict JSON");
        let published = document["vectors"].as_array().expect("vectors is an array");
        assert!(!published.is_empty(), "the published vector set is empty");
        for value in published {
            let name = value["name"].as_str().expect("vector name");
            let committed = unhex(
                value["exact_bytes_base16"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{name}: missing exact_bytes_base16")),
            );
            assert_eq!(
                keys_at(&committed, &[]),
                listed(&contract, "top_level_order"),
                "{name}: the committed bytes do not carry the declared top-level order"
            );
            assert_eq!(
                keys_at(&committed, &["asserted_context"]),
                listed(&contract, "asserted_context_order"),
                "{name}: asserted_context order"
            );
            assert_eq!(
                keys_at(&committed, &["executable"]),
                listed(&contract, "executable_order"),
                "{name}: executable order"
            );
            assert_eq!(
                keys_at(&committed, &["termination"]),
                listed(&contract, "termination_order"),
                "{name}: termination order"
            );
        }

        // ---------------------------------------------- 6. nullability
        //
        // Both states of the two nullable fields, from production output.
        let mut absent = vector_base();
        absent.asserted_context = AssertedContext {
            subject_spec_sha256: None,
            binding_report_sha256: None,
        };
        let mut present = vector_base();
        present.asserted_context = AssertedContext {
            subject_spec_sha256: Some(d(0x11)),
            binding_report_sha256: Some(d(0x22)),
        };
        let nullable = listed(&contract, "asserted_context_nullable");
        assert_eq!(nullable, listed(&contract, "asserted_context_order"));
        let absent_bytes = LaunchReceipt::from_record(absent).exact_bytes().to_vec();
        let present_bytes = LaunchReceipt::from_record(present).exact_bytes().to_vec();
        for field in &nullable {
            let (found, _) = members(&absent_bytes, 0);
            let (_, at) = found
                .iter()
                .find(|(key, _)| key == "asserted_context")
                .expect("asserted_context");
            let (inner, _) = members(&absent_bytes, *at);
            let (_, value) = inner
                .iter()
                .find(|(key, _)| key == field)
                .unwrap_or_else(|| panic!("asserted_context has no {field}"));
            let text: String = absent_bytes[*value..]
                .iter()
                .take_while(|b| b.is_ascii_alphabetic())
                .map(|b| char::from(*b))
                .collect();
            assert_eq!(
                text, contract["null_literal"],
                "{field} must be nullable and emit the declared null literal"
            );
        }
        assert_ne!(
            absent_bytes, present_bytes,
            "the two nullability states must not serialise identically"
        );
    }

    /// The document identifies the contract block as the machine-checked part,
    /// and does not present the surrounding prose as machine-verified.
    #[test]
    fn the_schema_document_separates_its_machine_block_from_its_prose() {
        for phrase in [
            "Sections 1 to 6 are written for people. This section is written for a test.",
            "remain documentation",
            "no second serializer exists",
        ] {
            assert!(
                SCHEMA_DOC.contains(phrase),
                "the schema document no longer states {phrase:?}"
            );
        }
        // The nonclaims of sections 1, 4 and 5 stay prose, and stay out of the
        // machine block. What keeps them out is that the block's key set is
        // **closed**: `contract()` rejects any key outside `CONTRACT_KEYS`, and
        // every one of those names a structural or vocabulary fact that this
        // file checks against the production model. A line asserting what a
        // receipt *means* has nowhere to go, because there is no key for it and
        // a new one is refused rather than ignored.
        let parsed = contract();
        assert_eq!(
            parsed.len(),
            CONTRACT_KEYS.len(),
            "the contract block and the declared key set disagree in size"
        );
        let declared: BTreeSet<&str> = CONTRACT_KEYS.into_iter().collect();
        let present: BTreeSet<&str> = parsed.keys().map(String::as_str).collect();
        assert_eq!(
            present, declared,
            "the contract block's key set is not exactly the declared closed set"
        );
    }
}
