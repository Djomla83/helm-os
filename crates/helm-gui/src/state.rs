//! Presentation state, and the mapping from **real** `helm-launch` facts to the
//! words HELM is allowed to say about them.
//!
//! This module holds no capability, no authority and no descriptor. It takes
//! typed facts the accepted crate produced and turns them into inert display
//! data. That separation is the point: the copy rules of the accepted product
//! definition live here, in one place, where a unit test can hold them still.
//!
//! **There is no mock result anywhere in this file.** Everything rendered on
//! the Result and Evidence surfaces is derived from a real [`ReceiptRecord`] or
//! a real [`LaunchError`]. The only fixed strings are the non-claims themselves.

use helm_launch::{
    AdmissionError, AdmissionErrorCode, Backend, ChildEnd, ChildStage, Completeness,
    EnvironmentMode, ExecStatus, ExecutableMeasurement, GroupSweep, IndeterminateReason,
    LaunchError, LaunchErrorCode, PreparationStep, ReceiptRecord, Stream, ValidatedLaunchPlan,
};

// ---------------------------------------------------------------------------
// Presentation state
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Library,
    Choose,
    Program,
    Authority,
    Attempt,
    Result,
    Evidence,
}

impl Screen {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Choose => "choose",
            Self::Program => "program",
            Self::Authority => "authority",
            Self::Attempt => "attempt",
            Self::Result => "result",
            Self::Evidence => "evidence",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Disclosure {
    Normal,
    Advanced,
}

/// The non-colour treatment each state carries. No state is conveyed by colour
/// alone: every one of these pairs with its own word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mark {
    OutlineSquare,
    OutlineDiamond,
    OutlineCircle,
    FilledSquare,
}

impl Mark {
    pub const fn css_class(self) -> &'static str {
        match self {
            Self::OutlineSquare => "mark-known",
            Self::OutlineDiamond => "mark-available",
            Self::OutlineCircle => "mark-attempt",
            Self::FilledSquare => "mark-ended",
        }
    }
}

/// Where the session has got to. Each variant is a real condition, not a demo
/// step: nothing advances without the corresponding real call having returned.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Nothing chosen. HELM knows nothing about any program.
    Nothing,
    /// A selection is being opened and measured by the accepted crate.
    Inspecting,
    /// Both objects admitted and a plan parsed. No authority exists yet.
    Admitted,
    /// A single-use authorisation exists.
    Authorised,
    /// The synchronous launch call has not returned.
    Attempting,
    /// The call returned a real outcome.
    Ended,
    /// The call refused before any child existed.
    CouldNotBegin,
}

impl Phase {
    pub const fn state_word(self) -> &'static str {
        match self {
            Self::Nothing => "Nothing open",
            Self::Inspecting => "Inspecting",
            Self::Admitted => "Known",
            Self::Authorised => "Launch available",
            Self::Attempting => "Launch attempt in progress",
            Self::Ended => "Ended",
            Self::CouldNotBegin => "Could not begin",
        }
    }

    pub const fn state_note(self) -> &'static str {
        match self {
            Self::Nothing => "HELM knows nothing about any program yet.",
            Self::Inspecting => "HELM is opening and measuring the file you chose.",
            Self::Admitted => "Program and folder admitted. No attempt made.",
            Self::Authorised => "A single-use authorisation exists.",
            Self::Attempting => {
                "HELM's launch call has not returned. Not a fact about the program."
            }
            Self::Ended => "The direct child was observed to end.",
            Self::CouldNotBegin => "HELM could not begin this launch attempt.",
        }
    }

    pub const fn mark(self) -> Mark {
        match self {
            Self::Authorised => Mark::OutlineDiamond,
            Self::Attempting | Self::Inspecting => Mark::OutlineCircle,
            Self::Ended | Self::CouldNotBegin => Mark::FilledSquare,
            Self::Nothing | Self::Admitted => Mark::OutlineSquare,
        }
    }

    pub const fn can_authorise(self) -> bool {
        matches!(self, Self::Admitted | Self::Ended | Self::CouldNotBegin)
    }

    pub const fn can_attempt(self) -> bool {
        matches!(self, Self::Authorised)
    }

    pub const fn is_open(self) -> bool {
        !matches!(self, Self::Nothing)
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Digits grouped in threes, as the accepted design sets every byte count.
#[must_use]
pub fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let lead = digits.len() % 3;
    for (index, ch) in digits.chars().enumerate() {
        if index != 0 && index % 3 == lead {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

#[must_use]
pub fn bytes_label(value: u64) -> String {
    format!("{} bytes", grouped(value))
}

/// Mode bits as the octal spelling the design uses.
#[must_use]
pub fn mode_bits_label(bits: u16) -> String {
    format!("0o{bits:o}")
}

/// A digest abbreviated for a ledger row. The full value is always available
/// on the Evidence surface; this is a reading aid, never a different identity.
#[must_use]
pub fn short_digest(hex: &str) -> String {
    if hex.len() <= 12 {
        return hex.to_owned();
    }
    let head: String = hex.chars().take(4).collect();
    let tail: String = hex.chars().skip(hex.len().saturating_sub(4)).collect();
    format!("sha256:{head}…{tail}")
}

// ---------------------------------------------------------------------------
// Admitted program facts
// ---------------------------------------------------------------------------

/// What HELM measured about the pinned object, from the real capability.
#[derive(Clone, Debug)]
pub struct ProgramFacts {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub sha256_hex: String,
    pub mode_bits: u16,
    pub elf_type: &'static str,
    pub elf_type_display: String,
}

impl ProgramFacts {
    #[must_use]
    pub fn from_measurement(name: &str, path: &str, measurement: &ExecutableMeasurement) -> Self {
        Self {
            name: name.to_owned(),
            path: path.to_owned(),
            size: measurement.pre_exec_body_size(),
            sha256_hex: measurement.pre_exec_body_sha256().to_hex(),
            mode_bits: measurement.pre_exec_mode_bits(),
            // The accepted spelling is `et_exec` / `et_dyn`; the header field
            // reads better upper-cased in a normal row, and Evidence still
            // carries the exact spelling.
            elf_type: measurement.elf_type().as_str(),
            elf_type_display: measurement.elf_type().as_str().to_uppercase(),
        }
    }

    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} · ELF {} · {}",
            bytes_label(self.size),
            self.elf_type_display,
            mode_bits_label(self.mode_bits)
        )
    }
}

/// The fixed policy of this slice, read back from the parsed plan rather than
/// from the constants that produced it, so the screen states what was actually
/// validated.
#[derive(Clone, Debug)]
pub struct PlanFacts {
    pub plan_sha256_hex: String,
    pub argv0: String,
    pub argument_count: usize,
    pub environment_mode: &'static str,
    pub stdin_mode: &'static str,
    pub working_directory_id: String,
    pub timeout_ms: u32,
    pub grace_ms: u32,
    pub termination_signal: &'static str,
    pub capture_bytes: u32,
}

impl PlanFacts {
    #[must_use]
    pub fn from_plan(plan: &ValidatedLaunchPlan) -> Self {
        Self {
            plan_sha256_hex: plan.sha256().to_hex(),
            argv0: plan.argv().first().cloned().unwrap_or_default(),
            argument_count: plan.argv().len(),
            environment_mode: plan.environment_mode().as_str(),
            stdin_mode: plan.stdin_mode().as_str(),
            working_directory_id: plan.working_directory_id().to_owned(),
            timeout_ms: plan.timeout_ms(),
            grace_ms: plan.grace_ms(),
            termination_signal: plan.termination_signal().as_str(),
            capture_bytes: plan.capture_prefix_bytes(Stream::Stdout),
        }
    }

    /// "It may run for at most 30 seconds. After that HELM asks it to stop,
    /// waits 5 seconds, then forces it."
    #[must_use]
    pub fn time_sentence(&self) -> String {
        format!(
            "It may run for at most {} seconds. After that HELM asks it to stop, waits {} \
             seconds, then forces it.",
            self.timeout_ms / 1000,
            self.grace_ms / 1000
        )
    }
}

// ---------------------------------------------------------------------------
// Admission refusals
// ---------------------------------------------------------------------------

/// Plain-language title for a real admission refusal. It names the condition;
/// it never judges the file.
#[must_use]
pub const fn admission_title(code: AdmissionErrorCode) -> &'static str {
    match code {
        AdmissionErrorCode::NotElf => "Refused — the file is not an ELF executable",
        AdmissionErrorCode::ElfNotInCohort => "Refused — this ELF is not one HELM can run here",
        AdmissionErrorCode::SetIdBitsPresent => "Refused — the file has set-id bits",
        AdmissionErrorCode::NotRegularFile => "Refused — that is not a regular file",
        AdmissionErrorCode::NotDirectory => "Refused — that is not a folder",
        AdmissionErrorCode::ExecutableTooLarge => "Refused — the file is larger than HELM admits",
        AdmissionErrorCode::DescriptorModeUnsuitable => {
            "Refused — HELM could not open it for reading only"
        }
        AdmissionErrorCode::MeasurementInstabilityDetected => {
            "Refused — the file changed while HELM was measuring it"
        }
        AdmissionErrorCode::MetadataUnavailable => {
            "Refused — HELM could not read the file's properties"
        }
        AdmissionErrorCode::ReadFailed => "Refused — HELM could not finish reading the file",
        AdmissionErrorCode::WorkingDirectoryIdInvalid => {
            "Refused — the working-folder identifier is not valid"
        }
        _ => "Refused — HELM did not admit this object",
    }
}

/// The explanation. Each states the condition and stops; none of them says the
/// object is dangerous, broken or wrong.
#[must_use]
pub const fn admission_text(code: AdmissionErrorCode) -> &'static str {
    match code {
        AdmissionErrorCode::NotElf => {
            "HELM admits only ELF executables on this computer. This file is something else, so \
             there is nothing to launch."
        }
        AdmissionErrorCode::ElfNotInCohort => {
            "The file is an ELF object, but not one of the kinds HELM runs on this computer."
        }
        AdmissionErrorCode::SetIdBitsPresent => {
            "HELM does not admit a file that would change user or group identity when it runs. \
             This is a property of the file, not a judgement about it."
        }
        AdmissionErrorCode::NotRegularFile => {
            "HELM runs a regular file. The object you chose is something else."
        }
        AdmissionErrorCode::NotDirectory => {
            "A working folder has to be a directory. The object you chose is something else."
        }
        AdmissionErrorCode::ExecutableTooLarge => {
            "HELM refuses a file above its admission bound before reading any of its body."
        }
        AdmissionErrorCode::DescriptorModeUnsuitable => {
            "HELM admits an object it holds open for reading only, and this one was not."
        }
        AdmissionErrorCode::MeasurementInstabilityDetected => {
            "The properties HELM sampled changed while it was sampling them, so HELM does not \
             have a stable measurement. It does not say the file changed, and it does not say it \
             did not."
        }
        AdmissionErrorCode::MetadataUnavailable => {
            "HELM could not obtain the object's kind or mode, so it refused rather than guessing."
        }
        AdmissionErrorCode::ReadFailed => {
            "A read failed part-way through measuring the file, so there is no complete \
             measurement."
        }
        AdmissionErrorCode::WorkingDirectoryIdInvalid => {
            "The identifier HELM uses for the working folder was not accepted."
        }
        _ => "HELM did not admit this object.",
    }
}

/// The exact closed code, for the advanced layer.
#[must_use]
pub fn admission_detail(error: &AdmissionError) -> String {
    match error.errno_name() {
        Some(name) => format!("{} · {name}", error.code().as_str()),
        None => error.code().as_str().to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Pre-child launch refusal
// ---------------------------------------------------------------------------

/// What a person is told when `launch` refused before any child existed.
///
/// **No child was created and no receipt exists.** This is a fact about HELM,
/// never about the program, and it is never dressed as a result.
#[must_use]
pub const fn launch_error_normal(code: LaunchErrorCode) -> &'static str {
    match code {
        LaunchErrorCode::PreparationFailed => {
            "HELM could not begin this launch attempt: it failed while preparing, before creating \
             anything."
        }
        LaunchErrorCode::ProcessCreationFailed => {
            "HELM could not begin this launch attempt: creating the process failed."
        }
        LaunchErrorCode::ProcessCreationUnavailable => {
            "HELM could not begin this launch attempt: process creation is unavailable on this \
             computer."
        }
        LaunchErrorCode::InternalInvariant => {
            "HELM could not begin this launch attempt, and stopped rather than continue in a \
             state it does not understand."
        }
        _ => "HELM could not begin this launch attempt.",
    }
}

#[must_use]
pub const fn preparation_step_name(step: PreparationStep) -> &'static str {
    match step {
        PreparationStep::Pipe => "pipe",
        PreparationStep::Relocate => "relocate",
        PreparationStep::NonBlocking => "non_blocking",
        PreparationStep::SignalMask => "signal_mask",
        _ => "preparation",
    }
}

/// The closed detail line for the advanced layer.
#[must_use]
pub fn launch_error_detail(error: &LaunchError) -> String {
    let mut detail = error.code().as_str().to_owned();
    if let Some(step) = error.step() {
        detail.push_str(" · ");
        detail.push_str(preparation_step_name(step));
    }
    if let Some(name) = error.errno_name() {
        detail.push_str(" · ");
        detail.push_str(name);
    } else if let Some(number) = error.errno_number() {
        detail.push_str(&format!(" · errno {number}"));
    }
    detail
}

// ---------------------------------------------------------------------------
// Result facts
// ---------------------------------------------------------------------------

/// The headline for a real observed end.
///
/// Every one of these states what was observed and stops. None of them says
/// the program succeeded, failed, worked or was safe — HELM issues no verdict
/// about a program, and an exit code is the program's own report.
#[must_use]
pub fn child_end_headline(end: ChildEnd) -> String {
    match end {
        ChildEnd::Exited { code } => format!("The program ended and reported {code}."),
        ChildEnd::Signaled { signal, .. } => {
            format!("The direct child was observed to end after signal {signal}.")
        }
        ChildEnd::EndUnobservable => {
            "HELM could not observe how the direct child ended.".to_owned()
        }
        ChildEnd::EndNotObserved => {
            "HELM did not observe the direct child's end within the bounded observation period."
                .to_owned()
        }
        _ => "HELM observed an end it cannot describe further.".to_owned(),
    }
}

/// The qualifying line under "How it ended".
#[must_use]
pub fn child_end_fact(end: ChildEnd) -> String {
    match end {
        ChildEnd::Exited { code } => {
            format!("The direct child was observed to end by exit, reporting {code}.")
        }
        ChildEnd::Signaled {
            signal,
            core_dumped,
        } => {
            let dumped = if core_dumped {
                " The end was reported as core-dumping."
            } else {
                ""
            };
            format!("The direct child was observed to end after signal {signal}.{dumped}")
        }
        ChildEnd::EndUnobservable => {
            "The end could not be classified — for example because the child was reaped elsewhere."
                .to_owned()
        }
        ChildEnd::EndNotObserved => {
            "No end was observed within the bound. HELM makes no claim that the program is still \
             running at any later moment."
                .to_owned()
        }
        _ => "HELM cannot describe this end further.".to_owned(),
    }
}

/// The accompanying subtitle. It exists to refuse the interpretation a person
/// would otherwise supply for themselves.
#[must_use]
pub const fn child_end_subtitle(end: ChildEnd) -> &'static str {
    match end {
        ChildEnd::Exited { .. } => {
            "HELM does not interpret what that number means. The program is the authority on its \
             own exit codes."
        }
        _ => "HELM reports what it observed. It does not interpret it.",
    }
}

/// Normal-language exec status.
///
/// **There is no success value to report.** `ExecStatus` is either a pre-exec
/// failure record or indeterminate, and clean end-of-file on the status channel
/// is not positive proof that execution began.
#[must_use]
pub const fn exec_status_normal(status: ExecStatus) -> &'static str {
    match status {
        ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord) => {
            "HELM received no start-up failure report. It did not establish that the program \
             began running."
        }
        ExecStatus::Indeterminate(IndeterminateReason::PreExecStatusTimeout) => {
            "No start-up report arrived within HELM's bound. HELM did not establish that the \
             program began running."
        }
        ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed) => {
            "The start-up report HELM read was not well formed, so HELM can say nothing about \
             the execution attempt."
        }
        ExecStatus::Indeterminate(IndeterminateReason::StatusReadFailed { .. }) => {
            "Reading the start-up channel failed, so HELM can say nothing about the execution \
             attempt. This is not treated as an ordinary end of file."
        }
        ExecStatus::PreExecFailure { .. } => {
            "The program did not begin running: HELM received a start-up failure report from \
             before the execution attempt completed."
        }
        _ => "HELM can say nothing further about the execution attempt.",
    }
}

#[must_use]
pub const fn child_stage_name(stage: ChildStage) -> &'static str {
    stage.as_str()
}

/// The exact closed vocabulary, for the advanced layer.
#[must_use]
pub fn exec_status_detail(status: ExecStatus) -> String {
    match status {
        ExecStatus::Indeterminate(reason) => {
            format!("{} · {}", status.as_str(), reason.as_str())
        }
        ExecStatus::PreExecFailure { stage, errno } => {
            format!("{} · {} · errno {errno}", status.as_str(), stage.as_str())
        }
        _ => status.as_str().to_owned(),
    }
}

/// The group-cleanup line. `Issued` says the one call was made and **nothing
/// else**: it is best-effort cleanup, never containment.
#[must_use]
pub const fn group_sweep_fact(sweep: GroupSweep) -> &'static str {
    match sweep {
        GroupSweep::Issued => {
            "HELM issued the best-effort process-group cleanup call. That is not containment: it \
             does not establish that any descendant received it, and a descendant that left the \
             group survives it."
        }
        GroupSweep::NotIssuedGroupNotEstablished => {
            "No process-group cleanup call was made, because group authority was never positively \
             established."
        }
        GroupSweep::NotIssuedChildAlreadyReaped => {
            "No process-group cleanup call was made, because the direct child had already been \
             reaped elsewhere."
        }
        _ => "HELM made no process-group cleanup call.",
    }
}

/// The run-deadline and termination line.
///
/// Takes plain booleans rather than the receipt's [`Termination`] so the rule
/// can be held still by a test without fabricating a receipt.
#[must_use]
pub fn deadline_fact(expired: bool, sigterm_sent: bool, sigkill_sent: bool) -> String {
    if !expired && !sigterm_sent && !sigkill_sent {
        return "The deadline did not expire. HELM issued no stop and no force.".to_owned();
    }
    let deadline = if expired {
        "The run deadline expired"
    } else {
        "The run deadline did not expire"
    };
    let acted = match (sigterm_sent, sigkill_sent) {
        (true, true) => "HELM asked the program to stop and then forced it.",
        (true, false) => "HELM asked the program to stop.",
        (false, true) => "HELM forced the program to stop.",
        (false, false) => "HELM issued no stop and no force.",
    };
    format!("{deadline}. {acted}")
}

/// The same line, read straight off a real receipt's termination facts.
#[must_use]
pub fn deadline_fact_of(record: &ReceiptRecord) -> String {
    let termination = record.termination();
    deadline_fact(
        record.run_deadline_expired(),
        termination.sigterm_sent(),
        termination.sigkill_sent(),
    )
}

/// How a captured stream ended. Each states the condition without implying the
/// program misbehaved.
#[must_use]
pub const fn completeness_fact(completeness: Completeness) -> &'static str {
    match completeness {
        Completeness::CompleteAtEof => {
            "The stream reached end of file and every byte was drained. Nothing was cut short."
        }
        Completeness::WriterRetainedAfterChildExit => {
            "The direct child ended while another process still held the stream open. Nothing is \
             claimed about anything written later."
        }
        Completeness::ReadStoppedChildEndNotObserved => {
            "Reading stopped because no end of the direct child was observed within the bound."
        }
        Completeness::ReadFailed { .. } => {
            "A read reported an error. The bytes before it were counted. This is never treated as \
             end of file."
        }
        _ => "HELM cannot describe how this stream ended.",
    }
}

/// Captured output, with non-UTF-8 disclosed rather than quietly replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputText {
    /// Nothing was captured.
    Empty,
    /// Valid UTF-8, shown as text.
    Text(String),
    /// Not valid UTF-8. The interface says so and shows a bounded, explicitly
    /// lossy rendering rather than pretending the bytes were text.
    NotUtf8 { lossy: String },
}

impl OutputText {
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::Empty;
        }
        match core::str::from_utf8(bytes) {
            Ok(text) => Self::Text(text.to_owned()),
            Err(_) => Self::NotUtf8 {
                lossy: String::from_utf8_lossy(bytes).into_owned(),
            },
        }
    }

    /// The disclosure shown beside the bytes, when one is needed.
    #[must_use]
    pub const fn notice(&self) -> Option<&'static str> {
        match self {
            Self::NotUtf8 { .. } => Some(
                "Captured output contains non-UTF-8 bytes. What follows is a lossy rendering, not \
                 the captured bytes.",
            ),
            _ => None,
        }
    }
}

/// One captured stream, as the interface shows it.
#[derive(Clone, Debug)]
pub struct StreamView {
    pub bytes_drained: u64,
    pub truncated: bool,
    pub completeness: &'static str,
    pub completeness_fact: &'static str,
    pub digest_hex: String,
    pub text: OutputText,
}

impl StreamView {
    #[must_use]
    pub fn build(record: &ReceiptRecord, stream: Stream, prefix: &[u8], truncated: bool) -> Self {
        let facts = record.stream(stream);
        Self {
            bytes_drained: facts.bytes_drained(),
            truncated,
            completeness: facts.completeness().as_str(),
            completeness_fact: completeness_fact(facts.completeness()),
            digest_hex: facts.drained_sha256().to_hex(),
            text: OutputText::from_bytes(prefix),
        }
    }

    #[must_use]
    pub fn label(&self) -> String {
        bytes_label(self.bytes_drained)
    }

    /// Stated only when it is true.
    #[must_use]
    pub const fn truncation_notice(&self) -> Option<&'static str> {
        if self.truncated {
            Some(
                "HELM kept a bounded prefix of this stream. What is shown is not everything the program printed.",
            )
        } else {
            None
        }
    }
}

#[must_use]
pub const fn backend_name(backend: Backend) -> &'static str {
    backend.as_str()
}

#[must_use]
pub const fn environment_mode_name(mode: EnvironmentMode) -> &'static str {
    mode.as_str()
}

/// The eight Evidence rows, built from the real record.
#[must_use]
pub fn receipt_rows(record: &ReceiptRecord) -> Vec<(&'static str, String, String)> {
    let executable = record.executable();
    let termination = record.termination();
    let stdout = record.stream(Stream::Stdout);
    let stderr = record.stream(Stream::Stderr);
    vec![
        (
            "Exec status",
            "exec_status".to_owned(),
            exec_status_detail(record.exec_status()),
        ),
        (
            "Child end",
            "child_end".to_owned(),
            child_end_detail(record.child_end()),
        ),
        (
            "Termination",
            "sigterm_sent · sigkill_sent · group_sweep".to_owned(),
            format!(
                "{} · {} · {}",
                termination.sigterm_sent(),
                termination.sigkill_sent(),
                termination.group_sweep().as_str()
            ),
        ),
        (
            "Output stream",
            "stdout · drained · completeness".to_owned(),
            format!(
                "{} · {}",
                bytes_label(stdout.bytes_drained()),
                stdout.completeness().as_str()
            ),
        ),
        (
            "Error stream",
            "stderr · drained · completeness".to_owned(),
            format!(
                "{} · {}",
                bytes_label(stderr.bytes_drained()),
                stderr.completeness().as_str()
            ),
        ),
        (
            "Pre-execution measurement",
            "size · sha256 · mode_bits · elf_type".to_owned(),
            format!(
                "{} · {} · {} · {}",
                grouped(executable.pre_exec_body_size()),
                short_digest(&executable.pre_exec_body_sha256().to_hex()),
                mode_bits_label(executable.pre_exec_mode_bits()),
                executable.elf_type().as_str()
            ),
        ),
        (
            "Plan identity",
            "plan_sha256".to_owned(),
            short_digest(&record.plan_sha256().to_hex()),
        ),
        (
            "Backend identity",
            "backend".to_owned(),
            record.backend().as_str().to_owned(),
        ),
    ]
}

#[must_use]
pub fn child_end_detail(end: ChildEnd) -> String {
    match end {
        ChildEnd::Exited { code } => format!("{} · code {code}", end.as_str()),
        ChildEnd::Signaled {
            signal,
            core_dumped,
        } => format!(
            "{} · signal {signal} · core_dumped {core_dumped}",
            end.as_str()
        ),
        _ => end.as_str().to_owned(),
    }
}

// ---------------------------------------------------------------------------
// Tests
//
// These pin the product semantics against real typed facts. They are the guard
// that stops backend wiring from quietly acquiring a verdict vocabulary.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Words HELM must never apply to anything, because each of them would be
    /// a judgement HELM has not made and cannot make.
    const NEVER: [&str; 9] = [
        "success",
        "succeeded",
        "compatible",
        "trusted",
        "verified",
        "secure",
        "sandbox",
        "contained",
        "works correctly",
    ];

    /// Verdict words. These are prohibited in copy about the subject program.
    ///
    /// `failure` and `failed` are deliberately **not** here. The accepted
    /// vocabulary uses them for things HELM genuinely observed or did not
    /// observe — a *start-up failure report*, a read that failed — and those
    /// are named records and operations, not judgements about a program. The
    /// behavioural tests below hold the actual rule: an exit code is reported
    /// and never interpreted.
    const NEVER_ABOUT_A_PROGRAM: [&str; 3] = ["pass", "safe", "works"];

    fn every_child_end() -> Vec<ChildEnd> {
        vec![
            ChildEnd::Exited { code: 0 },
            ChildEnd::Exited { code: 3 },
            ChildEnd::Signaled {
                signal: 15,
                core_dumped: false,
            },
            ChildEnd::Signaled {
                signal: 11,
                core_dumped: true,
            },
            ChildEnd::EndUnobservable,
            ChildEnd::EndNotObserved,
        ]
    }

    fn every_exec_status() -> Vec<ExecStatus> {
        vec![
            ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord),
            ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed),
            ExecStatus::Indeterminate(IndeterminateReason::PreExecStatusTimeout),
            ExecStatus::Indeterminate(IndeterminateReason::StatusReadFailed { errno: 5 }),
            ExecStatus::PreExecFailure {
                stage: ChildStage::Exec,
                errno: 13,
            },
        ]
    }

    /// Copy that describes the **subject program** or its outcome. This is the
    /// copy a verdict could creep into.
    fn program_copy() -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for end in every_child_end() {
            out.push(child_end_headline(end));
            out.push(child_end_fact(end));
            out.push(child_end_subtitle(end).to_owned());
        }
        for status in every_exec_status() {
            out.push(exec_status_normal(status).to_owned());
        }
        for sweep in [
            GroupSweep::Issued,
            GroupSweep::NotIssuedGroupNotEstablished,
            GroupSweep::NotIssuedChildAlreadyReaped,
        ] {
            out.push(group_sweep_fact(sweep).to_owned());
        }
        out
    }

    /// Every user-visible string this module can produce.
    fn all_copy() -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for phase in [
            Phase::Nothing,
            Phase::Inspecting,
            Phase::Admitted,
            Phase::Authorised,
            Phase::Attempting,
            Phase::Ended,
            Phase::CouldNotBegin,
        ] {
            out.push(phase.state_word().to_owned());
            out.push(phase.state_note().to_owned());
        }
        for end in every_child_end() {
            out.push(child_end_headline(end));
            out.push(child_end_fact(end));
            out.push(child_end_subtitle(end).to_owned());
        }
        for status in every_exec_status() {
            out.push(exec_status_normal(status).to_owned());
        }
        for sweep in [
            GroupSweep::Issued,
            GroupSweep::NotIssuedGroupNotEstablished,
            GroupSweep::NotIssuedChildAlreadyReaped,
        ] {
            out.push(group_sweep_fact(sweep).to_owned());
        }
        for completeness in [
            Completeness::CompleteAtEof,
            Completeness::WriterRetainedAfterChildExit,
            Completeness::ReadStoppedChildEndNotObserved,
            Completeness::ReadFailed { errno: 5 },
        ] {
            out.push(completeness_fact(completeness).to_owned());
        }
        for code in [
            LaunchErrorCode::PreparationFailed,
            LaunchErrorCode::ProcessCreationFailed,
            LaunchErrorCode::ProcessCreationUnavailable,
            LaunchErrorCode::InternalInvariant,
        ] {
            out.push(launch_error_normal(code).to_owned());
        }
        for code in [
            AdmissionErrorCode::NotElf,
            AdmissionErrorCode::ElfNotInCohort,
            AdmissionErrorCode::SetIdBitsPresent,
            AdmissionErrorCode::NotRegularFile,
            AdmissionErrorCode::NotDirectory,
            AdmissionErrorCode::ExecutableTooLarge,
            AdmissionErrorCode::DescriptorModeUnsuitable,
            AdmissionErrorCode::MeasurementInstabilityDetected,
            AdmissionErrorCode::MetadataUnavailable,
            AdmissionErrorCode::ReadFailed,
            AdmissionErrorCode::WorkingDirectoryIdInvalid,
        ] {
            out.push(admission_title(code).to_owned());
            out.push(admission_text(code).to_owned());
        }
        out
    }

    #[test]
    fn nothing_anywhere_claims_a_judgement_helm_has_not_made() {
        for text in all_copy() {
            let lowered = text.to_lowercase();
            for word in NEVER {
                assert!(
                    !lowered.contains(word),
                    "forbidden word {word:?} in {text:?}"
                );
            }
        }
    }

    #[test]
    fn no_copy_about_a_program_issues_a_verdict_on_it() {
        // `failed` is legitimate about HELM's own operation — a read failed, a
        // preparation step failed — and is deliberately not swept out of that
        // copy. It must never appear in copy about the subject.
        for text in program_copy() {
            let lowered = text.to_lowercase();
            for word in NEVER_ABOUT_A_PROGRAM {
                assert!(
                    !lowered.contains(word),
                    "verdict {word:?} applied to a program in {text:?}"
                );
            }
        }
    }

    #[test]
    fn no_copy_claims_the_program_ran_or_started() {
        for text in all_copy() {
            let lowered = text.to_lowercase();
            for claim in [
                "is running",
                "started successfully",
                "launched successfully",
                "execution confirmed",
                "program started",
            ] {
                assert!(
                    !lowered.contains(claim),
                    "forbidden running claim {claim:?} in {text:?}"
                );
            }
        }
    }

    #[test]
    fn where_running_is_mentioned_at_all_it_is_denied() {
        // Execution beginning is nameable only inside a denial. This is the
        // real rule: not that the words never appear, but that they never
        // appear as an assertion.
        for text in all_copy() {
            if text.contains("began running") || text.contains("begin running") {
                assert!(
                    text.contains("did not establish")
                        || text.contains("did not begin running")
                        || text.contains("has not established"),
                    "execution is mentioned without being denied: {text}"
                );
            }
            if text.contains("running") {
                assert!(
                    !text.contains("is running") && !text.contains("was running"),
                    "a program is asserted to be running: {text}"
                );
            }
        }
    }

    #[test]
    fn no_copy_claims_an_installation() {
        for text in all_copy() {
            let lowered = text.to_lowercase();
            for claim in ["installed", "added to helm", "in your library"] {
                assert!(!lowered.contains(claim), "install claim in {text:?}");
            }
        }
    }

    #[test]
    fn a_clean_status_eof_is_never_turned_into_execution_proof() {
        let status = ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord);
        let normal = exec_status_normal(status);
        assert!(normal.contains("did not establish"));
        assert_eq!(
            exec_status_detail(status),
            "indeterminate · status_eof_without_record"
        );
    }

    #[test]
    fn exit_code_zero_is_reported_and_not_interpreted() {
        let end = ChildEnd::Exited { code: 0 };
        assert_eq!(child_end_headline(end), "The program ended and reported 0.");
        assert!(child_end_subtitle(end).contains("does not interpret"));
    }

    #[test]
    fn a_non_zero_exit_code_is_reported_the_same_way() {
        assert_eq!(
            child_end_headline(ChildEnd::Exited { code: 3 }),
            "The program ended and reported 3."
        );
    }

    #[test]
    fn an_issued_group_sweep_is_never_containment() {
        let fact = group_sweep_fact(GroupSweep::Issued);
        assert!(fact.contains("best-effort"));
        assert!(fact.contains("not containment"));
        assert!(fact.contains("survives it"));
    }

    #[test]
    fn a_pre_child_refusal_is_about_helm_and_not_about_the_program() {
        for code in [
            LaunchErrorCode::PreparationFailed,
            LaunchErrorCode::ProcessCreationFailed,
            LaunchErrorCode::ProcessCreationUnavailable,
            LaunchErrorCode::InternalInvariant,
        ] {
            assert!(launch_error_normal(code).starts_with("HELM could not begin"));
        }
    }

    #[test]
    fn non_utf8_output_is_disclosed_rather_than_silently_replaced() {
        let view = OutputText::from_bytes(&[0x68, 0x69, 0xff, 0xfe]);
        assert!(matches!(view, OutputText::NotUtf8 { .. }));
        let notice = view.notice();
        assert!(notice.is_some_and(|text| text.contains("non-UTF-8")));

        let clean = OutputText::from_bytes(b"hello");
        assert_eq!(clean, OutputText::Text("hello".to_owned()));
        assert!(clean.notice().is_none());
        assert_eq!(OutputText::from_bytes(&[]), OutputText::Empty);
    }

    #[test]
    fn the_deadline_line_states_only_what_was_done() {
        assert_eq!(
            deadline_fact(false, false, false),
            "The deadline did not expire. HELM issued no stop and no force."
        );
        assert_eq!(
            deadline_fact(true, true, true),
            "The run deadline expired. HELM asked the program to stop and then forced it."
        );
        assert_eq!(
            deadline_fact(true, true, false),
            "The run deadline expired. HELM asked the program to stop."
        );
        // A stop without an expiry is still stated exactly as observed.
        assert_eq!(
            deadline_fact(false, false, true),
            "The run deadline did not expire. HELM forced the program to stop."
        );
    }

    #[test]
    fn byte_counts_are_grouped_as_the_design_sets_them() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1_284), "1 284");
        assert_eq!(grouped(41_984), "41 984");
        assert_eq!(bytes_label(41_984), "41 984 bytes");
    }

    #[test]
    fn mode_bits_render_octal() {
        assert_eq!(mode_bits_label(0o755), "0o755");
    }

    #[test]
    fn every_phase_carries_a_distinct_word() {
        let words: Vec<&str> = [
            Phase::Nothing,
            Phase::Inspecting,
            Phase::Admitted,
            Phase::Authorised,
            Phase::Attempting,
            Phase::Ended,
        ]
        .into_iter()
        .map(Phase::state_word)
        .collect();
        let mut unique = words.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), words.len());
    }

    #[test]
    fn only_an_authorisation_permits_an_attempt() {
        assert!(Phase::Authorised.can_attempt());
        for phase in [
            Phase::Nothing,
            Phase::Inspecting,
            Phase::Admitted,
            Phase::Attempting,
            Phase::Ended,
            Phase::CouldNotBegin,
        ] {
            assert!(!phase.can_attempt(), "{phase:?} must not permit an attempt");
        }
    }
}
