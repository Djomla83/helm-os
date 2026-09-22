//! The session state machine of the first real vertical.
//!
//! This is the same state machine the window has always run; it lives here
//! rather than in the binary so that a test can drive it without opening a
//! window, and in particular so that worker results can be delivered in an
//! order a person could not reliably produce by hand.
//!
//! **It is not a framework.** There is no trait, no registry and no generic
//! event loop: one concrete session, the transitions the seven accepted
//! surfaces actually offer, and the rule for applying a worker result. The
//! widgets stay in the binary and nothing here draws anything.
//!
//! **It holds no authority of its own.** Capabilities and the single-use
//! authorisation live here only because the interface holds them between the
//! moment a person chooses something and the moment they instruct a launch.
//! Everything that judges an object is `helm-launch`, reached through
//! [`crate::vertical`].

use std::path::PathBuf;
use std::time::Instant;

use helm_launch::{
    AuthorizedLaunch, ExecutableCapability, LaunchError, LaunchOutcome, ValidatedLaunchPlan,
    WorkingDirectoryCapability,
};

use crate::state::{Disclosure, Phase, PlanFacts, ProgramFacts, Screen};
use crate::vertical::{self, Message, Refusal};

/// Everything the session knows. It lives for one process: there is no store,
/// no file, no database and no config, so closing the window ends it and
/// starting again starts from nothing. That is intentional, and G-1 remains
/// unauthorised.
pub struct Session {
    pub screen: Screen,
    pub disclosure: Disclosure,
    pub output_open: bool,

    /// The paths a person selected. The interface knows them and shows them;
    /// `helm-launch` never receives them.
    pub exec_path: Option<PathBuf>,
    pub workdir_path: Option<PathBuf>,
    pub opened_at: Option<String>,

    exec_pending: bool,
    workdir_pending: bool,

    executable: Option<ExecutableCapability>,
    pub exec_refusal: Option<Refusal>,
    pub program: Option<ProgramFacts>,

    working_directory: Option<WorkingDirectoryCapability>,
    pub workdir_refusal: Option<Refusal>,

    plan: Option<ValidatedLaunchPlan>,
    pub plan_facts: Option<PlanFacts>,
    pub plan_refusal: Option<String>,

    /// The one-shot authority. Composed only at the person's explicit
    /// instruction, and consumed exactly once.
    authorized: Option<AuthorizedLaunch>,

    attempting: bool,
    attempt_started: Option<Instant>,
    outcome: Option<LaunchOutcome>,
    launch_error: Option<LaunchError>,

    pub copy_label: String,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self {
            screen: Screen::Library,
            disclosure: Disclosure::Advanced,
            output_open: false,
            exec_path: None,
            workdir_path: None,
            opened_at: None,
            exec_pending: false,
            workdir_pending: false,
            executable: None,
            exec_refusal: None,
            program: None,
            working_directory: None,
            workdir_refusal: None,
            plan: None,
            plan_facts: None,
            plan_refusal: None,
            authorized: None,
            attempting: false,
            attempt_started: None,
            outcome: None,
            launch_error: None,
            copy_label: "Copy the exact receipt bytes".to_owned(),
        }
    }

    // -----------------------------------------------------------------------
    // What the interface reads
    // -----------------------------------------------------------------------

    /// Derived, never stored, so it cannot drift out of step with what the
    /// session actually holds.
    #[must_use]
    pub fn phase(&self) -> Phase {
        if self.attempting {
            Phase::Attempting
        } else if self.outcome.is_some() {
            Phase::Ended
        } else if self.launch_error.is_some() {
            Phase::CouldNotBegin
        } else if self.authorized.is_some() {
            Phase::Authorised
        } else if self.exec_pending || self.workdir_pending {
            Phase::Inspecting
        } else if self.admitted() {
            Phase::Admitted
        } else {
            Phase::Nothing
        }
    }

    #[must_use]
    pub fn admitted(&self) -> bool {
        self.executable.is_some() && self.working_directory.is_some() && self.plan.is_some()
    }

    #[must_use]
    pub const fn advanced(&self) -> bool {
        matches!(self.disclosure, Disclosure::Advanced)
    }

    #[must_use]
    pub fn program_name(&self) -> String {
        self.program
            .as_ref()
            .map_or_else(|| "No program open".to_owned(), |facts| facts.name.clone())
    }

    #[must_use]
    pub const fn exec_pending(&self) -> bool {
        self.exec_pending
    }

    #[must_use]
    pub const fn workdir_pending(&self) -> bool {
        self.workdir_pending
    }

    #[must_use]
    pub const fn has_executable(&self) -> bool {
        self.executable.is_some()
    }

    #[must_use]
    pub const fn has_working_directory(&self) -> bool {
        self.working_directory.is_some()
    }

    #[must_use]
    pub const fn has_authority(&self) -> bool {
        self.authorized.is_some()
    }

    #[must_use]
    pub const fn attempting(&self) -> bool {
        self.attempting
    }

    #[must_use]
    pub const fn attempt_started(&self) -> Option<Instant> {
        self.attempt_started
    }

    #[must_use]
    pub const fn outcome(&self) -> Option<&LaunchOutcome> {
        self.outcome.as_ref()
    }

    #[must_use]
    pub const fn launch_error(&self) -> Option<&LaunchError> {
        self.launch_error.as_ref()
    }

    // -----------------------------------------------------------------------
    // Transitions a person can cause
    // -----------------------------------------------------------------------

    /// Records a new executable selection and returns the path the caller must
    /// hand to [`vertical::spawn_admit_executable`].
    pub fn begin_executable(&mut self, path: PathBuf, opened_at: Option<String>) -> PathBuf {
        // A new selection invalidates any authority and any previous plan.
        self.authorized = None;
        self.plan = None;
        self.plan_facts = None;
        self.plan_refusal = None;
        self.exec_path = Some(path.clone());
        self.executable = None;
        self.exec_refusal = None;
        self.program = None;
        self.exec_pending = true;
        self.opened_at = opened_at;
        path
    }

    /// Records a new working-folder selection and returns the path the caller
    /// must hand to [`vertical::spawn_admit_working_directory`].
    pub fn begin_working_directory(&mut self, path: PathBuf) -> PathBuf {
        self.authorized = None;
        self.plan = None;
        self.plan_facts = None;
        self.plan_refusal = None;
        self.workdir_path = Some(path.clone());
        self.working_directory = None;
        self.workdir_refusal = None;
        self.workdir_pending = true;
        path
    }

    /// The chooser returned something with no local filesystem path.
    pub fn refuse_non_local(&mut self, folder: bool) {
        if folder {
            self.workdir_refusal = Some(Refusal::NotLocal);
            self.working_directory = None;
        } else {
            self.exec_refusal = Some(Refusal::NotLocal);
            self.executable = None;
            self.program = None;
        }
    }

    /// Composes the one-shot authority, at the person's explicit instruction.
    /// This is the only place it happens.
    pub fn authorise(&mut self) {
        if !self.admitted() {
            return;
        }
        let (Some(plan), Some(executable), Some(working_directory)) = (
            self.plan.take(),
            self.executable.take(),
            self.working_directory.take(),
        ) else {
            return;
        };
        match vertical::authorise(plan, executable, working_directory) {
            Ok(authorized) => {
                self.authorized = Some(authorized);
                self.screen = Screen::Program;
            }
            Err(refusal) => {
                // The capabilities were consumed by the refusal, so the session
                // genuinely has to re-admit before it can try again.
                self.plan_refusal = Some(format!(
                    "authorisation refused · {}",
                    refusal.code().as_str()
                ));
                self.screen = Screen::Choose;
            }
        }
    }

    /// Takes the authority so the caller can consume it exactly once, on a
    /// worker thread. Returns `None` when there is none to consume.
    pub fn start_attempt(&mut self) -> Option<AuthorizedLaunch> {
        let authorized = self.authorized.take()?;
        self.attempting = true;
        self.attempt_started = Some(Instant::now());
        self.outcome = None;
        self.launch_error = None;
        self.output_open = false;
        self.screen = Screen::Attempt;
        Some(authorized)
    }

    /// "Attempt launch again" returns through fresh authority preparation: the
    /// previous authority was consumed and nothing about it is replayed. The
    /// paths are re-opened and re-admitted from scratch, so the caller gets
    /// back the two paths to hand to fresh workers.
    pub fn attempt_again(&mut self) -> (Option<PathBuf>, Option<PathBuf>) {
        self.clear_admission();
        self.screen = Screen::Authority;
        let exec_path = self.exec_path.clone();
        let workdir_path = self.workdir_path.clone();
        if exec_path.is_some() {
            self.exec_pending = true;
        }
        if workdir_path.is_some() {
            self.workdir_pending = true;
        }
        (exec_path, workdir_path)
    }

    /// Going back to the chooser drops every admission, so the next attempt
    /// genuinely re-opens and re-admits.
    pub fn go_choose(&mut self) {
        self.clear_admission();
        self.screen = Screen::Choose;
    }

    /// Everything that depends on a particular admission is dropped, so a new
    /// attempt genuinely re-opens and re-admits. Capabilities drop here, which
    /// closes their descriptors.
    pub fn clear_admission(&mut self) {
        self.executable = None;
        self.working_directory = None;
        self.exec_refusal = None;
        self.workdir_refusal = None;
        self.program = None;
        self.plan = None;
        self.plan_facts = None;
        self.plan_refusal = None;
        self.authorized = None;
    }

    pub fn close(&mut self) {
        self.clear_admission();
        self.exec_path = None;
        self.workdir_path = None;
        self.opened_at = None;
        self.outcome = None;
        self.launch_error = None;
        self.attempt_started = None;
        self.attempting = false;
        self.output_open = false;
        self.screen = Screen::Library;
    }

    // -----------------------------------------------------------------------
    // Worker results
    // -----------------------------------------------------------------------

    /// Applies one finished worker result.
    pub fn apply(&mut self, message: Message) {
        match message {
            Message::Executable(result) => {
                self.exec_pending = false;
                match result {
                    Ok(capability) => {
                        let name = self
                            .exec_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("program")
                            .to_owned();
                        let path = self
                            .exec_path
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default();
                        self.program = Some(ProgramFacts::from_measurement(
                            &name,
                            &path,
                            &capability.measurement(),
                        ));
                        self.executable = Some(capability);
                        self.exec_refusal = None;
                    }
                    Err(refusal) => {
                        self.exec_refusal = Some(refusal);
                        self.executable = None;
                        self.program = None;
                    }
                }
            }
            Message::WorkingDirectory(result) => {
                self.workdir_pending = false;
                match result {
                    Ok(capability) => {
                        self.working_directory = Some(capability);
                        self.workdir_refusal = None;
                    }
                    Err(refusal) => {
                        self.workdir_refusal = Some(refusal);
                        self.working_directory = None;
                    }
                }
            }
            Message::Launched(result) => {
                self.attempting = false;
                match *result {
                    Ok(outcome) => {
                        self.outcome = Some(outcome);
                        self.launch_error = None;
                    }
                    Err(error) => {
                        self.launch_error = Some(error);
                        self.outcome = None;
                    }
                }
                self.screen = Screen::Result;
            }
        }
        self.parse_plan_if_ready();
    }

    /// Parses the fixed plan once both objects are admitted. The parse is fast
    /// and pure, so it stays on the main thread.
    fn parse_plan_if_ready(&mut self) {
        if self.plan.is_some() || self.executable.is_none() || self.working_directory.is_none() {
            return;
        }
        let Some(path) = self.exec_path.clone() else {
            return;
        };
        let argv0 = vertical::argv0_for(&path);
        match vertical::parse_plan(&argv0) {
            Ok(plan) => {
                self.plan_facts = Some(PlanFacts::from_plan(&plan));
                self.plan = Some(plan);
                self.plan_refusal = None;
            }
            Err(errors) => {
                let codes: Vec<String> = errors
                    .as_slice()
                    .iter()
                    .map(|e| format!("{} at {}", e.code().as_str(), e.locator()))
                    .collect();
                self.plan_refusal = Some(codes.join(" · "));
            }
        }
    }
}
