//! Mock state for the G2-D9 UI fidelity spike.
//!
//! **NOTHING HERE TOUCHES THIS COMPUTER.** No file is opened, no process is
//! created, no receipt is produced, nothing is read from disk and nothing is
//! written to it. Every path, byte count, digest, timestamp, exit code and
//! receipt value below is a constant written into this file, exactly as in the
//! canonical D8 prototype.
//!
//! This module deliberately has **no GTK dependency**. It is the whole of the
//! spike's logic, it is unit-tested, and keeping it separate is what stops mock
//! values from looking like backend integration: there is no trait, no client,
//! no abstraction that a backend could later be slotted into. Connecting HELM
//! state is a later, separately authorised piece of work that would replace
//! this file rather than implement against it.

// ---------------------------------------------------------------------------
// Mock constants. Fixed strings, not measurements.
// ---------------------------------------------------------------------------

pub const PROGRAM_NAME: &str = "helm-probe";
pub const PROGRAM_PATH: &str = "/home/mk/build/helm-probe";
pub const OPENED_AT: &str = "11:42:07";
pub const SIZE_BYTES: &str = "41 984 bytes";
pub const MODE_BITS: &str = "0o755";
pub const ELF_TYPE: &str = "EXEC";
pub const MEASURED_SUMMARY: &str = "41 984 bytes · ELF EXEC · 0o755";
pub const PRE_EXEC_DIGEST: &str = "sha256:9d41…c7e0";
pub const PLAN_DIGEST: &str = "sha256:4f1c…9ab2";
pub const BACKEND_IDENTITY: &str = "linux_x86_64_clone3_pidfd_execveat";
pub const WORKDIR_ID: &str = "wd-7";
pub const STDOUT_BYTES: &str = "1 284 bytes";
pub const STDERR_BYTES: &str = "0 bytes";

pub const PROGRAM_OUTPUT: &str = "probe: reading working folder\nprobe: 14 entries\nprobe: done";

pub const RECEIPT_DIGEST: &str = "3f8a1c9e5b47d20a6c1f8e93b7d4a05c2e6f9182d3b7c4a5e8f01926d7c3b4a5";

pub const RECEIPT_BYTES: &str = concat!(
    r#"{"receipt_version":"0.1","backend":"linux_x86_64_clone3_pidfd_execveat","#,
    r#""exec_status":{"indeterminate":"status_eof_without_record"},"#,
    r#""child_end":{"exited":{"code":0}},"sigterm_sent":false,"#,
    r#""sigkill_sent":false,"group_sweep":"issued","#,
    r#""stdout":{"drained":1284,"completeness":"complete_at_eof"},"#,
    r#""stderr":{"drained":0,"completeness":"complete_at_eof"},"#,
    r#""pre_exec":{"size":41984,"sha256":"9d41…c7e0","mode_bits":493,"#,
    r#""elf_type":"EXEC"},"plan_sha256":"4f1c…9ab2"}"#,
);

/// The run deadline the Authority and Attempt copy both state.
pub const DEADLINE_SECONDS: f64 = 30.0;

/// How long the mock attempt runs before it reaches its mock conclusion. This
/// is a presentation timing, not a measurement of anything.
pub const MOCK_ATTEMPT_MS: u64 = 2_600;

// ---------------------------------------------------------------------------
// State
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
    /// Stack child names, and the rail items that map onto them.
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

/// What HELM holds for this session. `None` means nothing is open — not that
/// something was removed from a library, because there is no library.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Entry {
    None,
    Known,
    Available,
    Attempt,
    Ended,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    SetIdBitsPresent,
    NotElf,
}

impl Refusal {
    pub const fn title(self) -> &'static str {
        match self {
            Self::SetIdBitsPresent => "Refused — the file has set-id bits",
            Self::NotElf => "Refused — the file is not an ELF executable",
        }
    }

    pub const fn text(self) -> &'static str {
        match self {
            Self::SetIdBitsPresent => {
                "HELM does not admit a file that would change user or group identity when it \
                 runs. This is a property of the file, not a judgement about it."
            }
            Self::NotElf => {
                "HELM admits only ELF executables on this computer. This file is something \
                 else, so there is nothing to launch."
            }
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::SetIdBitsPresent => "set_id_bits_present",
            Self::NotElf => "not_elf",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Program {
    None,
    Admitted,
    Refused(Refusal),
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

/// The three files the chooser stand-in offers. One is admitted; two are
/// refused, for the two admission refusals the prototype demonstrates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Candidate {
    HelmProbe,
    MountHelper,
    NotesTxt,
}

impl Candidate {
    pub const fn name(self) -> &'static str {
        match self {
            Self::HelmProbe => "helm-probe",
            Self::MountHelper => "mount-helper",
            Self::NotesTxt => "notes.txt",
        }
    }

    pub const fn size(self) -> &'static str {
        match self {
            Self::HelmProbe => "41 984 bytes",
            Self::MountHelper => "18 120 bytes",
            Self::NotesTxt => "2 044 bytes",
        }
    }

    const fn outcome(self) -> Program {
        match self {
            Self::HelmProbe => Program::Admitted,
            Self::MountHelper => Program::Refused(Refusal::SetIdBitsPresent),
            Self::NotesTxt => Program::Refused(Refusal::NotElf),
        }
    }

    pub const fn all() -> [Self; 3] {
        [Self::HelmProbe, Self::MountHelper, Self::NotesTxt]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct State {
    pub screen: Screen,
    pub entry: Entry,
    pub program: Program,
    pub folder: bool,
    pub output_open: bool,
    pub disclosure: Disclosure,
    pub elapsed_seconds: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            screen: Screen::Library,
            entry: Entry::Known,
            program: Program::Admitted,
            folder: true,
            output_open: false,
            disclosure: Disclosure::Advanced,
            elapsed_seconds: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Derived values. These mirror the canonical prototype's `renderVals()` one for
// one, so that the two can be compared line by line.
// ---------------------------------------------------------------------------

impl State {
    pub const fn advanced(&self) -> bool {
        matches!(self.disclosure, Disclosure::Advanced)
    }

    /// The state word. Never "Running", never "Started", never a verdict.
    pub const fn state_word(&self) -> &'static str {
        match self.entry {
            Entry::Available => "Launch available",
            Entry::Attempt => "Launch attempt in progress",
            Entry::Ended => "Ended",
            _ => "Known",
        }
    }

    pub const fn state_note(&self) -> &'static str {
        match self.entry {
            Entry::Available => "A single-use authorisation exists.",
            Entry::Attempt => "HELM's launch call has not returned. Not a fact about the program.",
            Entry::Ended => "The direct child was observed to end, reporting 0.",
            _ => "Program and folder admitted. No attempt made.",
        }
    }

    pub const fn mark(&self) -> Mark {
        match self.entry {
            Entry::Available => Mark::OutlineDiamond,
            Entry::Attempt => Mark::OutlineCircle,
            Entry::Ended => Mark::FilledSquare,
            _ => Mark::OutlineSquare,
        }
    }

    pub const fn rail_label(&self) -> &'static str {
        match self.entry {
            Entry::None => "Nothing open",
            _ => "Held in memory",
        }
    }

    pub const fn rail_note(&self) -> &'static str {
        match self.entry {
            Entry::None => "HELM knows nothing about any program yet.",
            _ => "Entries live until HELM closes. Nothing was written to disk.",
        }
    }

    pub fn footer_subject(&self) -> String {
        match self.entry {
            Entry::None => "No program open".to_owned(),
            _ => format!("{PROGRAM_NAME} — {}", self.state_word()),
        }
    }

    pub fn footer_bound(&self) -> String {
        format!("Run deadline {} s", DEADLINE_SECONDS as u32)
    }

    pub const fn footer_mode(&self) -> &'static str {
        match self.disclosure {
            Disclosure::Advanced => "Advanced disclosure",
            Disclosure::Normal => "Normal disclosure",
        }
    }

    pub const fn program_phase(&self) -> &'static str {
        match self.program {
            Program::Admitted => "Admitted",
            Program::Refused(_) => "Refused",
            Program::None => "Not checked yet",
        }
    }

    pub const fn chosen(&self) -> bool {
        matches!(self.program, Program::Admitted) && self.folder
    }

    pub const fn folder_phase(&self) -> &'static str {
        if self.chosen() {
            "Admitted"
        } else {
            "Not checked yet"
        }
    }

    pub const fn can_authorise(&self) -> bool {
        matches!(self.entry, Entry::Known | Entry::Ended)
    }

    pub const fn can_attempt(&self) -> bool {
        matches!(self.entry, Entry::Available)
    }

    pub const fn has_result(&self) -> bool {
        matches!(self.entry, Entry::Ended)
    }

    pub fn elapsed_label(&self) -> String {
        let shown = self.elapsed_seconds.min(DEADLINE_SECONDS);
        format!("{shown:.1} s of at most {} s", DEADLINE_SECONDS as u32)
    }

    /// 0.0 to 1.0, for the bound meter. It is a bound, not progress towards
    /// success.
    pub fn elapsed_fraction(&self) -> f64 {
        (self.elapsed_seconds / DEADLINE_SECONDS).clamp(0.0, 1.0)
    }

    pub const fn output_toggle_label(&self) -> &'static str {
        if self.output_open {
            "Hide what was printed"
        } else {
            "Show what was printed"
        }
    }
}

// ---------------------------------------------------------------------------
// Transitions
// ---------------------------------------------------------------------------

impl State {
    pub const fn go(&mut self, screen: Screen) {
        self.screen = screen;
    }

    /// Entering Choose discards the current selection, as the prototype does.
    pub const fn go_choose(&mut self) {
        self.screen = Screen::Choose;
        self.program = Program::None;
        self.folder = false;
    }

    pub const fn go_program(&mut self) {
        self.screen = Screen::Program;
        if matches!(self.entry, Entry::None) {
            self.entry = Entry::Known;
        }
    }

    pub const fn pick(&mut self, candidate: Candidate) {
        self.program = candidate.outcome();
        if matches!(self.program, Program::Admitted) {
            self.folder = true;
        }
    }

    pub const fn choose_folder(&mut self) {
        self.folder = true;
    }

    /// A single-use authorisation. It is consumed by one attempt.
    pub const fn authorise(&mut self) {
        self.screen = Screen::Program;
        self.entry = Entry::Available;
    }

    pub const fn refuse_authority(&mut self) {
        self.screen = Screen::Program;
        self.entry = Entry::Known;
    }

    /// Begins the mock attempt. There is no counterpart that cancels it:
    /// `launch` is synchronous and returns no handle, so the spike offers no
    /// control that would imply otherwise.
    pub const fn start_attempt(&mut self) {
        self.screen = Screen::Attempt;
        self.entry = Entry::Attempt;
        self.elapsed_seconds = 0.0;
    }

    pub const fn finish_attempt(&mut self) {
        self.screen = Screen::Result;
        self.entry = Entry::Ended;
    }

    pub const fn close_entry(&mut self) {
        self.screen = Screen::Library;
        self.entry = Entry::None;
        self.program = Program::None;
        self.folder = false;
    }

    pub const fn toggle_output(&mut self) {
        self.output_open = !self.output_open;
    }

    pub const fn toggle_disclosure(&mut self) {
        self.disclosure = match self.disclosure {
            Disclosure::Advanced => Disclosure::Normal,
            Disclosure::Normal => Disclosure::Advanced,
        };
    }
}

// ---------------------------------------------------------------------------
// Tests
//
// These are not UI tests. They pin the product semantics the spike must not
// drift away from, in the one place where drift would be silent.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Every string this spike can put on screen, for the prohibition sweeps.
    fn all_copy() -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for entry in [
            Entry::None,
            Entry::Known,
            Entry::Available,
            Entry::Attempt,
            Entry::Ended,
        ] {
            let mut state = State {
                entry,
                ..State::default()
            };
            out.push(state.state_word().to_owned());
            out.push(state.state_note().to_owned());
            out.push(state.rail_label().to_owned());
            out.push(state.rail_note().to_owned());
            out.push(state.footer_subject());
            out.push(state.footer_bound());
            out.push(state.elapsed_label());
            out.push(state.output_toggle_label().to_owned());
            for disclosure in [Disclosure::Normal, Disclosure::Advanced] {
                state.disclosure = disclosure;
                out.push(state.footer_mode().to_owned());
            }
            for program in [
                Program::None,
                Program::Admitted,
                Program::Refused(Refusal::NotElf),
                Program::Refused(Refusal::SetIdBitsPresent),
            ] {
                state.program = program;
                out.push(state.program_phase().to_owned());
                out.push(state.folder_phase().to_owned());
            }
        }
        for refusal in [Refusal::NotElf, Refusal::SetIdBitsPresent] {
            out.push(refusal.title().to_owned());
            out.push(refusal.text().to_owned());
            out.push(refusal.code().to_owned());
        }
        out.push(PROGRAM_OUTPUT.to_owned());
        out.push(RECEIPT_BYTES.to_owned());
        out
    }

    #[test]
    fn an_attempt_is_never_described_as_running() {
        let state = State {
            entry: Entry::Attempt,
            ..State::default()
        };
        assert_eq!(state.state_word(), "Launch attempt in progress");
        assert!(state.state_note().contains("launch call has not returned"));
        assert!(state.state_note().contains("Not a fact about the program"));
    }

    #[test]
    fn no_copy_claims_the_program_is_running_or_started() {
        for text in all_copy() {
            let lowered = text.to_lowercase();
            for forbidden in [
                "running",
                "is started",
                "started successfully",
                "launched successfully",
                "launch confirmed",
            ] {
                assert!(
                    !lowered.contains(forbidden),
                    "forbidden running-claim {forbidden:?} in {text:?}"
                );
            }
        }
    }

    #[test]
    fn no_copy_issues_a_verdict_about_the_program() {
        for text in all_copy() {
            let lowered = text.to_lowercase();
            for verdict in [
                "pass",
                "fail",
                "success",
                "works",
                "compatible",
                "safe",
                "trusted",
                "verified",
                "secure",
                "sandbox",
                "contained",
                "isolated",
            ] {
                assert!(
                    !lowered.contains(verdict),
                    "forbidden verdict {verdict:?} in {text:?}"
                );
            }
        }
    }

    #[test]
    fn no_copy_claims_an_installation_or_a_durable_library() {
        for text in all_copy() {
            let lowered = text.to_lowercase();
            for claim in ["installed", "added to helm", "in your library"] {
                assert!(
                    !lowered.contains(claim),
                    "forbidden install claim {claim:?} in {text:?}"
                );
            }
        }
    }

    #[test]
    fn the_receipt_keeps_its_indeterminate_exec_status() {
        assert!(
            RECEIPT_BYTES
                .contains(r#""exec_status":{"indeterminate":"status_eof_without_record"}"#)
        );
        assert!(!RECEIPT_BYTES.contains("ExecSucceeded"));
        assert!(!RECEIPT_BYTES.contains("succeeded"));
    }

    #[test]
    fn the_group_sweep_is_only_ever_issued() {
        assert!(RECEIPT_BYTES.contains(r#""group_sweep":"issued""#));
    }

    #[test]
    fn ending_reports_a_code_without_interpreting_it() {
        let state = State {
            entry: Entry::Ended,
            ..State::default()
        };
        assert_eq!(state.state_word(), "Ended");
        assert!(state.state_note().contains("reporting 0"));
        assert!(!state.state_note().to_lowercase().contains("success"));
    }

    #[test]
    fn the_two_demonstrated_refusals_carry_their_real_codes() {
        assert_eq!(Refusal::NotElf.code(), "not_elf");
        assert_eq!(Refusal::SetIdBitsPresent.code(), "set_id_bits_present");
    }

    #[test]
    fn the_chooser_admits_one_candidate_and_refuses_two() {
        let mut admitted = 0;
        let mut refused = 0;
        for candidate in Candidate::all() {
            let mut state = State::default();
            state.pick(candidate);
            match state.program {
                Program::Admitted => admitted += 1,
                Program::Refused(_) => refused += 1,
                Program::None => unreachable!(),
            }
        }
        assert_eq!((admitted, refused), (1, 2));
    }

    #[test]
    fn a_refusal_admits_nothing() {
        let mut state = State::default();
        state.go_choose();
        state.pick(Candidate::NotesTxt);
        assert!(!state.chosen());
        assert_eq!(state.folder_phase(), "Not checked yet");
    }

    #[test]
    fn an_authorisation_is_single_use() {
        let mut state = State::default();
        state.authorise();
        assert!(state.can_attempt());
        assert!(!state.can_authorise());
        state.start_attempt();
        state.finish_attempt();
        // The attempt consumed it: authorising has to happen again.
        assert!(!state.can_attempt());
        assert!(state.can_authorise());
    }

    #[test]
    fn the_elapsed_readout_is_a_bound_and_never_exceeds_it() {
        let state = State {
            elapsed_seconds: DEADLINE_SECONDS * 4.0,
            ..State::default()
        };
        assert_eq!(state.elapsed_label(), "30.0 s of at most 30 s");
        assert!((state.elapsed_fraction() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn closing_an_entry_returns_to_nothing_open() {
        let mut state = State::default();
        state.close_entry();
        assert_eq!(state.entry, Entry::None);
        assert_eq!(state.rail_label(), "Nothing open");
        assert_eq!(state.footer_subject(), "No program open");
    }

    #[test]
    fn entering_choose_discards_the_current_selection() {
        let mut state = State::default();
        assert!(state.chosen());
        state.go_choose();
        assert!(!state.chosen());
        assert_eq!(state.program_phase(), "Not checked yet");
    }

    #[test]
    fn disclosure_toggles_between_exactly_two_modes() {
        let mut state = State::default();
        assert!(state.advanced());
        state.toggle_disclosure();
        assert!(!state.advanced());
        assert_eq!(state.footer_mode(), "Normal disclosure");
        state.toggle_disclosure();
        assert!(state.advanced());
    }

    #[test]
    fn every_state_carries_a_distinct_non_colour_mark() {
        let marks: Vec<Mark> = [Entry::Known, Entry::Available, Entry::Attempt, Entry::Ended]
            .into_iter()
            .map(|entry| {
                State {
                    entry,
                    ..State::default()
                }
                .mark()
            })
            .collect();
        let mut seen = marks.clone();
        seen.dedup();
        assert_eq!(seen.len(), marks.len(), "two states share one mark");
    }
}
