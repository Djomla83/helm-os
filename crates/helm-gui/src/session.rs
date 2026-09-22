//! The session state machine of the first real vertical.
//!
//! This is the same state machine the window has always run; it lives here
//! rather than in the binary so that a test can drive it without opening a
//! window, and in particular so that worker results can be delivered in an
//! order a person cannot reliably produce by hand.
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
//!
//! ## The rule for worker results (`PGR-01`)
//!
//! Work is asynchronous and the session can move on while it runs, so the kind
//! of a result is not enough to say what it may change. Each operation is
//! minted with a [`vertical::OperationId`] before its worker starts, and the
//! session records which operation is currently allowed to change each piece of
//! state:
//!
//! * an executable admission may change executable state only while it is the
//!   operation in [`Session::exec_op`];
//! * a working-directory admission likewise;
//! * a launch result may be recorded only against the attempt that started it.
//!
//! A result whose operation is no longer the current one is **dropped**. A
//! dropped capability closes its descriptor as it goes; a dropped
//! [`LaunchOutcome`] is discarded as data and is never relabelled.
//!
//! Two further rules keep a fact attached to its subject:
//!
//! * an [`Attempt`] owns immutable copies of the subject and the plan it was
//!   authorised from, so a result can only ever be reported under those; and
//! * while an attempt is outstanding the open program cannot be replaced. That
//!   is refused here, in the state machine, not by which button happens to be
//!   drawn.

use std::path::{Path, PathBuf};
use std::time::Instant;

use helm_launch::{
    AuthorizedLaunch, ExecutableCapability, LaunchError, LaunchOutcome, ValidatedLaunchPlan,
    WorkingDirectoryCapability,
};

use crate::state::{Disclosure, Phase, PlanFacts, ProgramFacts, Screen};
use crate::vertical::{self, Message, OperationId, Refusal};

/// The single-use authority, together with the facts it was composed from.
///
/// They travel as one value because they are one thing: this authority
/// authorises exactly this measured subject, under exactly this plan, in
/// exactly this folder. Separating them is what let a result be reported
/// under a subject that did not produce it.
struct Authority {
    launch: AuthorizedLaunch,
    subject: ProgramFacts,
    plan: PlanFacts,
    workdir_path: Option<PathBuf>,
}

/// What the accepted synchronous launch returned.
pub enum AttemptResult {
    /// A direct child was created and observed to end.
    ///
    /// Boxed because it is much the larger of the two: an outcome carries the
    /// receipt and both captured prefixes, a refusal carries a code.
    Ended(Box<LaunchOutcome>),
    /// HELM refused before any child existed.
    CouldNotBegin(LaunchError),
}

/// One launch attempt: the facts it was authorised from, and what came back.
///
/// The subject and plan here are copies taken when the authority was composed.
/// Every surface that reports this attempt reports **these**, so the attempt's
/// result cannot be drawn under a subject that did not produce it.
pub struct Attempt {
    op: OperationId,
    subject: ProgramFacts,
    plan: PlanFacts,
    workdir_path: Option<PathBuf>,
    started: Instant,
    result: Option<AttemptResult>,
}

impl Attempt {
    /// The subject as it was measured for this attempt.
    #[must_use]
    pub const fn subject(&self) -> &ProgramFacts {
        &self.subject
    }

    /// The plan as it was validated for this attempt.
    #[must_use]
    pub const fn plan(&self) -> &PlanFacts {
        &self.plan
    }

    #[must_use]
    pub fn workdir_path(&self) -> Option<&Path> {
        self.workdir_path.as_deref()
    }

    #[must_use]
    pub const fn started(&self) -> Instant {
        self.started
    }

    #[must_use]
    pub const fn running(&self) -> bool {
        self.result.is_none()
    }

    #[must_use]
    pub fn outcome(&self) -> Option<&LaunchOutcome> {
        match &self.result {
            Some(AttemptResult::Ended(outcome)) => Some(outcome.as_ref()),
            _ => None,
        }
    }

    #[must_use]
    pub const fn error(&self) -> Option<&LaunchError> {
        match &self.result {
            Some(AttemptResult::CouldNotBegin(error)) => Some(error),
            _ => None,
        }
    }
}

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

    /// The next operation identity. Monotonic within this process and nothing
    /// more: it is minted here, compared here, and ends with the process.
    next_op: u64,

    /// The admission currently allowed to change executable state, with the
    /// path that operation is actually measuring. Display facts are built from
    /// **this** path, never from whatever `exec_path` happens to hold when the
    /// result arrives.
    exec_op: Option<(OperationId, PathBuf)>,
    workdir_op: Option<(OperationId, PathBuf)>,

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
    authorized: Option<Authority>,

    attempt: Option<Attempt>,

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
            next_op: 0,
            exec_op: None,
            workdir_op: None,
            executable: None,
            exec_refusal: None,
            program: None,
            working_directory: None,
            workdir_refusal: None,
            plan: None,
            plan_facts: None,
            plan_refusal: None,
            authorized: None,
            attempt: None,
            copy_label: "Copy the exact receipt bytes".to_owned(),
        }
    }

    /// Mints the next operation identity.
    fn mint(&mut self) -> OperationId {
        self.next_op = self.next_op.saturating_add(1);
        OperationId::from_raw(self.next_op)
    }

    // -----------------------------------------------------------------------
    // What the interface reads
    // -----------------------------------------------------------------------

    /// Derived, never stored, so it cannot drift out of step with what the
    /// session actually holds.
    #[must_use]
    pub fn phase(&self) -> Phase {
        if self.attempt_running() {
            Phase::Attempting
        // A re-admission in flight comes before a previous attempt's result:
        // "Attempt launch again" re-opens and re-admits, and while that is
        // happening the entry is being inspected, not ended.
        } else if self.exec_pending() || self.workdir_pending() {
            Phase::Inspecting
        } else if self.attempt.as_ref().and_then(Attempt::outcome).is_some() {
            Phase::Ended
        } else if self.attempt.as_ref().and_then(Attempt::error).is_some() {
            Phase::CouldNotBegin
        } else if self.authorized.is_some() {
            Phase::Authorised
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

    /// The open entry's program name — what the Library row and the status
    /// line describe.
    #[must_use]
    pub fn program_name(&self) -> String {
        self.program
            .as_ref()
            .map_or_else(|| "No program open".to_owned(), |facts| facts.name.clone())
    }

    /// The attempt HELM last authorised, if any. Surfaces that report a launch
    /// read their subject, plan and folder from here.
    #[must_use]
    pub const fn attempt(&self) -> Option<&Attempt> {
        self.attempt.as_ref()
    }

    #[must_use]
    pub fn attempt_running(&self) -> bool {
        self.attempt.as_ref().is_some_and(Attempt::running)
    }

    #[must_use]
    pub const fn exec_pending(&self) -> bool {
        self.exec_op.is_some()
    }

    #[must_use]
    pub const fn workdir_pending(&self) -> bool {
        self.workdir_op.is_some()
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
    pub fn outcome(&self) -> Option<&LaunchOutcome> {
        self.attempt.as_ref().and_then(Attempt::outcome)
    }

    #[must_use]
    pub fn launch_error(&self) -> Option<&LaunchError> {
        self.attempt.as_ref().and_then(Attempt::error)
    }

    // -----------------------------------------------------------------------
    // Transitions a person can cause
    // -----------------------------------------------------------------------

    /// Records a new executable selection and returns the operation identity
    /// and the path the caller must hand to
    /// [`vertical::spawn_admit_executable`].
    ///
    /// Returns `None` while an attempt is outstanding: the program that
    /// attempt is running cannot be replaced underneath it. The refusal is
    /// here rather than in the interface, so it does not depend on which
    /// control happens to be drawn.
    pub fn begin_executable(
        &mut self,
        path: PathBuf,
        opened_at: Option<String>,
    ) -> Option<(OperationId, PathBuf)> {
        if self.attempt_running() {
            return None;
        }
        // A new program is a new entry: the previous attempt was an attempt of
        // a different program and is not this entry's result.
        self.attempt = None;
        // A new selection invalidates any authority and any previous plan.
        self.authorized = None;
        self.plan = None;
        self.plan_facts = None;
        self.plan_refusal = None;
        self.exec_path = Some(path.clone());
        self.executable = None;
        self.exec_refusal = None;
        self.program = None;
        self.opened_at = opened_at;
        let op = self.mint();
        self.exec_op = Some((op, path.clone()));
        Some((op, path))
    }

    /// Records a new working-folder selection and returns the operation
    /// identity and the path for
    /// [`vertical::spawn_admit_working_directory`].
    ///
    /// Returns `None` while an attempt is outstanding, for the same reason.
    pub fn begin_working_directory(&mut self, path: PathBuf) -> Option<(OperationId, PathBuf)> {
        if self.attempt_running() {
            return None;
        }
        self.authorized = None;
        self.plan = None;
        self.plan_facts = None;
        self.plan_refusal = None;
        self.workdir_path = Some(path.clone());
        self.working_directory = None;
        self.workdir_refusal = None;
        let op = self.mint();
        self.workdir_op = Some((op, path.clone()));
        Some((op, path))
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
        if self.attempt_running() || !self.admitted() {
            return;
        }
        let (Some(plan), Some(executable), Some(working_directory)) = (
            self.plan.take(),
            self.executable.take(),
            self.working_directory.take(),
        ) else {
            return;
        };
        // The facts this authority is being composed from, taken before it
        // exists, so that they can only ever describe this authority.
        let measured = self.program.clone().zip(self.plan_facts.clone());
        match (
            vertical::authorise(plan, executable, working_directory),
            measured,
        ) {
            (Ok(launch), Some((subject, plan))) => {
                self.authorized = Some(Authority {
                    launch,
                    subject,
                    plan,
                    workdir_path: self.workdir_path.clone(),
                });
                self.screen = Screen::Program;
            }
            (Ok(_), None) => {
                // Unreachable while an authority can only follow a completed
                // admission, and safe if that ever changes: the authority
                // drops here, closing its descriptors, and nothing is claimed.
                self.plan_refusal =
                    Some("authorisation refused · nothing measured to authorise".to_owned());
                self.screen = Screen::Choose;
            }
            (Err(refusal), _) => {
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
    /// worker thread, and opens the attempt its result will be recorded
    /// against. Returns `None` when there is no authority to consume.
    pub fn start_attempt(&mut self) -> Option<(OperationId, AuthorizedLaunch)> {
        let authority = self.authorized.take()?;
        let op = self.mint();
        self.attempt = Some(Attempt {
            op,
            subject: authority.subject,
            plan: authority.plan,
            workdir_path: authority.workdir_path,
            started: Instant::now(),
            result: None,
        });
        self.output_open = false;
        self.screen = Screen::Attempt;
        Some((op, authority.launch))
    }

    /// "Attempt launch again" returns through fresh authority preparation: the
    /// previous authority was consumed and nothing about it is replayed. The
    /// paths are re-opened and re-admitted from scratch, so the caller gets
    /// back a fresh operation for each.
    ///
    /// The previous attempt is deliberately kept: this is another attempt of
    /// the same program, and until a new result exists the previous one is
    /// still the last thing HELM observed about it.
    #[allow(clippy::type_complexity, reason = "two optional worker requests")]
    pub fn attempt_again(
        &mut self,
    ) -> (
        Option<(OperationId, PathBuf)>,
        Option<(OperationId, PathBuf)>,
    ) {
        if self.attempt_running() {
            return (None, None);
        }
        self.clear_admission();
        self.screen = Screen::Authority;
        let exec = self.exec_path.clone().map(|path| {
            let op = self.mint();
            self.exec_op = Some((op, path.clone()));
            (op, path)
        });
        let workdir = self.workdir_path.clone().map(|path| {
            let op = self.mint();
            self.workdir_op = Some((op, path.clone()));
            (op, path)
        });
        (exec, workdir)
    }

    /// Going back to the chooser drops every admission and the previous
    /// attempt, so the next attempt genuinely re-opens, re-admits and is
    /// reported on its own.
    pub fn go_choose(&mut self) {
        if self.attempt_running() {
            return;
        }
        self.attempt = None;
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

    /// Closing the entry ends it. Every outstanding operation loses its claim
    /// on the session, so a result that arrives afterwards is dropped: a
    /// capability closes its descriptor, and a launch outcome is discarded as
    /// data rather than reported against an entry that no longer exists.
    ///
    /// The launch itself is synchronous and has no cancel — by design, since
    /// G-2 is unauthorised — so a running attempt runs to completion on its
    /// worker. What closing changes is only what HELM will say about it.
    pub fn close(&mut self) {
        self.clear_admission();
        self.exec_op = None;
        self.workdir_op = None;
        self.attempt = None;
        self.exec_path = None;
        self.workdir_path = None;
        self.opened_at = None;
        self.output_open = false;
        self.screen = Screen::Library;
    }

    // -----------------------------------------------------------------------
    // Worker results
    // -----------------------------------------------------------------------

    /// Applies one finished worker result, if the operation that produced it is
    /// still the one allowed to change that state. Otherwise the result is
    /// dropped here and changes nothing.
    pub fn apply(&mut self, message: Message) {
        match message {
            Message::Executable { op, result } => self.apply_executable(op, result),
            Message::WorkingDirectory { op, result } => self.apply_working_directory(op, result),
            Message::Launched { op, result } => self.apply_launched(op, *result),
        }
        self.parse_plan_if_ready();
    }

    fn apply_executable(&mut self, op: OperationId, result: Result<ExecutableCapability, Refusal>) {
        // The path this operation actually measured, not whatever is selected
        // now. Taking it here is what closes the operation.
        let Some(path) = self.close_exec_op(op) else {
            return;
        };
        match result {
            Ok(capability) => {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("program")
                    .to_owned();
                self.program = Some(ProgramFacts::from_measurement(
                    &name,
                    &path.display().to_string(),
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

    fn apply_working_directory(
        &mut self,
        op: OperationId,
        result: Result<WorkingDirectoryCapability, Refusal>,
    ) {
        if !self.close_workdir_op(op) {
            return;
        }
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

    fn apply_launched(&mut self, op: OperationId, result: Result<LaunchOutcome, LaunchError>) {
        let Some(attempt) = self.attempt.as_mut().filter(|a| a.op == op && a.running()) else {
            // The attempt this belongs to is gone, or this is not its result.
            // The outcome is discarded as data; it is never relabelled.
            return;
        };
        attempt.result = Some(match result {
            Ok(outcome) => AttemptResult::Ended(Box::new(outcome)),
            Err(error) => AttemptResult::CouldNotBegin(error),
        });
        self.screen = Screen::Result;
    }

    /// Closes the current executable admission if `op` is it, handing back the
    /// path that operation was measuring. Any other operation is stale, and
    /// nothing is closed.
    fn close_exec_op(&mut self, op: OperationId) -> Option<PathBuf> {
        match &self.exec_op {
            Some((current, path)) if *current == op => {
                let path = path.clone();
                self.exec_op = None;
                Some(path)
            }
            _ => None,
        }
    }

    /// The same, for the working folder. The path is not needed: a folder
    /// carries no measurement, and its capability is matched by identifier.
    fn close_workdir_op(&mut self, op: OperationId) -> bool {
        match &self.workdir_op {
            Some((current, _)) if *current == op => {
                self.workdir_op = None;
                true
            }
            _ => false,
        }
    }

    /// Parses the fixed plan once both objects are admitted. The parse is fast
    /// and pure, so it stays on the main thread.
    ///
    /// `argv[0]` comes from the path the admitted capability was measured
    /// from, which is the subject's own name and not a later selection's.
    fn parse_plan_if_ready(&mut self) {
        if self.plan.is_some() || self.executable.is_none() || self.working_directory.is_none() {
            return;
        }
        let Some(facts) = self.program.as_ref() else {
            return;
        };
        let argv0 = facts.name.clone();
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
