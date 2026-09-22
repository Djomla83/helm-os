//! `PGR-01` — a worker result must only change the state of the operation that
//! produced it.
//!
//! The interface hands three kinds of work to a worker thread: executable
//! admission, working-directory admission, and the synchronous launch. Each
//! one returns later, through a channel, to a session that may have moved on.
//! This file drives the real session state machine and delivers those results
//! in orders a person cannot reliably produce by hand.
//!
//! **These are state-level tests, deliberately.** Some of the orders below are
//! reachable through the current controls and some are not; the note on each
//! test says which. The state machine is held to the invariant either way,
//! because reachability is a property of whichever buttons happen to be drawn
//! today, and the integrity of a HELM fact must not be.
//!
//! Nothing here mocks a result. Every capability is a real
//! `helm_launch::admit_*` capability over a real file, and the one launch
//! outcome is a real `helm_launch::launch` return.

// A failing assertion is how a test reports, and the fixture paths are
// constructed by this file. Panicking here is the mechanism, not a defect.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test code: a failed assertion is the reporting mechanism"
)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use helm_gui::session::Session;
use helm_gui::state::Phase;
use helm_gui::vertical::{self, Message, Refusal};
use helm_launch::{AuthorizedLaunch, ExecutableCapability, WorkingDirectoryCapability};
use sha2::{Digest as _, Sha256};

const FIXTURE_MODE: u32 = 0o755;
const ALPHA: &str = "alpha-subject";
const BETA: &str = "beta-subject";

// ---------------------------------------------------------------------------
// Tickets
//
// A worker carries something back that says which operation it was. The tests
// below never name that thing directly: they take it from the transition that
// started the work and hand it back when the result arrives. That keeps every
// test body written in terms of *this operation's result*, which is the
// property under test, rather than in terms of whatever the token happens to
// be.
// ---------------------------------------------------------------------------

struct ExecTicket;
struct WorkdirTicket;
struct LaunchTicket;

fn begin_exec(session: &mut Session, path: &Path) -> ExecTicket {
    let started = session.begin_executable(path.to_path_buf(), None);
    assert_eq!(
        started, path,
        "the worker is given the path that was chosen"
    );
    ExecTicket
}

fn begin_workdir(session: &mut Session, path: &Path) -> WorkdirTicket {
    let started = session.begin_working_directory(path.to_path_buf());
    assert_eq!(
        started, path,
        "the worker is given the path that was chosen"
    );
    WorkdirTicket
}

fn deliver_exec(
    session: &mut Session,
    _ticket: &ExecTicket,
    result: Result<ExecutableCapability, Refusal>,
) {
    session.apply(Message::Executable(result));
}

fn deliver_workdir(
    session: &mut Session,
    _ticket: &WorkdirTicket,
    result: Result<WorkingDirectoryCapability, Refusal>,
) {
    session.apply(Message::WorkingDirectory(result));
}

fn deliver_launch(
    session: &mut Session,
    _ticket: &LaunchTicket,
    result: Result<helm_launch::LaunchOutcome, helm_launch::LaunchError>,
) {
    session.apply(Message::Launched(Box::new(result)));
}

fn start_attempt(session: &mut Session) -> (LaunchTicket, AuthorizedLaunch) {
    match session.start_attempt() {
        Some(authorized) => (LaunchTicket, authorized),
        None => panic!("the session held no authority to consume"),
    }
}

fn attempt_again(session: &mut Session) -> (ExecTicket, WorkdirTicket) {
    let (exec, workdir) = session.attempt_again();
    match (exec, workdir) {
        (Some(_), Some(_)) => (ExecTicket, WorkdirTicket),
        _ => panic!("attempt again must re-open and re-admit both objects"),
    }
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "helm-g2-binding-{}-{tag}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        fs::create_dir_all(&root).expect("create scratch root");
        Self { root }
    }

    fn dir(&self, name: &str) -> PathBuf {
        let dir = self.root.join(name);
        fs::create_dir_all(&dir).expect("create scratch directory");
        dir
    }

    /// Places the purpose-built ELF fixture under `name`. `padding` trailing
    /// bytes make a second subject that is a different length and therefore a
    /// different measurement, while staying an ELF the accepted admission
    /// takes: the header it classifies is the first 64 bytes either way.
    fn subject(&self, name: &str, padding: usize) -> PathBuf {
        let source = Path::new(env!("CARGO_BIN_EXE_g2-launch-fixture"));
        assert!(source.is_file(), "fixture binary was not built: {source:?}");
        let target = self.root.join(name);
        let mut bytes = fs::read(source).expect("read fixture");
        bytes.extend(std::iter::repeat_n(0_u8, padding));
        fs::write(&target, &bytes).expect("write subject");
        fs::set_permissions(&target, fs::Permissions::from_mode(FIXTURE_MODE))
            .expect("set subject mode");
        target
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn digest_of(path: &Path) -> String {
    let bytes = fs::read(path).expect("read subject for its digest");
    let digest = Sha256::digest(&bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn admit_exec(path: &Path) -> ExecutableCapability {
    match vertical::admit_executable_at(path) {
        Ok(capability) => capability,
        Err(refusal) => panic!("admission of {path:?} refused: {refusal:?}"),
    }
}

fn admit_workdir(path: &Path) -> WorkingDirectoryCapability {
    match vertical::admit_working_directory_at(path) {
        Ok(capability) => capability,
        Err(refusal) => panic!("admission of {path:?} refused: {refusal:?}"),
    }
}

/// Everything the Choose surface does before an authority can exist.
fn admit_both(session: &mut Session, subject: &Path, workdir: &Path) {
    let exec = begin_exec(session, subject);
    deliver_exec(session, &exec, Ok(admit_exec(subject)));
    let dir = begin_workdir(session, workdir);
    deliver_workdir(session, &dir, Ok(admit_workdir(workdir)));
    assert!(
        session.admitted(),
        "both objects were admitted and the fixed plan parsed"
    );
}

// ---------------------------------------------------------------------------
// PGR01-T1 — a stale executable admission
// ---------------------------------------------------------------------------

/// Not reachable through today's Choose surface, which hides the re-choose
/// button while an admission is running. The state machine is held to the
/// invariant anyway, so that a future control cannot quietly make it reachable.
#[test]
fn t1_a_stale_executable_result_must_not_populate_a_newer_selection() {
    let scratch = Scratch::new("t1");
    let alpha = scratch.subject(ALPHA, 0);
    let beta = scratch.subject(BETA, 64);
    let alpha_capability = admit_exec(&alpha);

    let mut session = Session::new();
    // Operation A begins on alpha.
    let a = begin_exec(&mut session, &alpha);
    // The selection moves to beta before A returns. B is now the subject.
    let _b = begin_exec(&mut session, &beta);
    // A's worker returns late.
    deliver_exec(&mut session, &a, Ok(alpha_capability));

    assert!(
        session.exec_pending(),
        "beta's admission is still running, so the session must still be inspecting"
    );
    assert!(
        !session.has_executable(),
        "alpha's capability must not become beta's admitted executable"
    );
    match session.program.as_ref() {
        None => {}
        Some(facts) => panic!(
            "a stale result described the current subject: name {:?}, path {:?}, digest {:?} \
             (alpha measured {:?}, beta is {:?})",
            facts.name,
            facts.path,
            facts.sha256_hex,
            digest_of(&alpha),
            digest_of(&beta)
        ),
    }
}

// ---------------------------------------------------------------------------
// PGR01-T2 — a stale working-directory admission
// ---------------------------------------------------------------------------

/// Same shape, same current unreachability, same reason for testing it.
#[test]
fn t2_a_stale_working_directory_result_must_not_satisfy_a_newer_selection() {
    let scratch = Scratch::new("t2");
    let first = scratch.dir("first");
    let second = scratch.dir("second");
    let first_capability = admit_workdir(&first);

    let mut session = Session::new();
    let a = begin_workdir(&mut session, &first);
    let _b = begin_workdir(&mut session, &second);
    deliver_workdir(&mut session, &a, Ok(first_capability));

    assert!(
        session.workdir_pending(),
        "the second folder's admission is still running"
    );
    assert!(
        !session.has_working_directory(),
        "the first folder's capability must not satisfy the second selection"
    );
    assert_eq!(
        session.workdir_path.as_deref(),
        Some(second.as_path()),
        "the displayed folder is the one that was chosen"
    );
}

// ---------------------------------------------------------------------------
// PGR01-T3 — a reset or close, undone by a late result
// ---------------------------------------------------------------------------

/// **Reachable today.** The Library row carries "Close entry" and is visible
/// whenever the session is open, which includes while an admission is running.
#[test]
fn t3_closing_the_entry_must_not_be_undone_by_a_late_executable_result() {
    let scratch = Scratch::new("t3-exec");
    let alpha = scratch.subject(ALPHA, 0);
    let capability = admit_exec(&alpha);

    let mut session = Session::new();
    let a = begin_exec(&mut session, &alpha);
    session.close();
    deliver_exec(&mut session, &a, Ok(capability));

    assert!(
        !session.has_executable(),
        "a closed entry must not be holding an executable capability"
    );
    assert!(
        session.program.is_none(),
        "a closed entry must describe no program: {:?}",
        session.program.as_ref().map(|f| (&f.name, &f.path))
    );
    assert!(
        !session.exec_pending(),
        "the closed entry has nothing outstanding"
    );
    assert_eq!(session.phase(), Phase::Nothing);
}

/// **Reachable today**, and the one with authority consequences: a
/// working-directory capability that outlived the entry it belonged to could
/// otherwise still be part of the next authorisation.
#[test]
fn t3_closing_the_entry_must_not_be_undone_by_a_late_working_directory_result() {
    let scratch = Scratch::new("t3-dir");
    let workdir = scratch.dir("work");
    let capability = admit_workdir(&workdir);

    let mut session = Session::new();
    let a = begin_workdir(&mut session, &workdir);
    session.close();
    deliver_workdir(&mut session, &a, Ok(capability));

    assert!(
        !session.has_working_directory(),
        "a closed entry must not be holding a working-directory capability"
    );
    assert!(session.workdir_path.is_none());
    assert!(!session.workdir_pending());
    assert_eq!(session.phase(), Phase::Nothing);
}

/// **Reachable today.** "Close entry" is live while a launch attempt is
/// running, and the launch is synchronous with no cancel — by design, because
/// G-2 is unauthorised. The result must therefore be dropped as data, never
/// applied to the session that no longer has the entry.
#[test]
fn t3_closing_the_entry_must_not_be_undone_by_a_late_launch_result() {
    let scratch = Scratch::new("t3-launch");
    let alpha = scratch.subject(ALPHA, 0);
    let workdir = scratch.dir("work");

    let mut session = Session::new();
    admit_both(&mut session, &alpha, &workdir);
    session.authorise();
    let (ticket, authorized) = start_attempt(&mut session);
    let outcome = helm_launch::launch(authorized);
    assert!(outcome.is_ok(), "the fixture subject launches");

    session.close();
    deliver_launch(&mut session, &ticket, outcome);

    assert!(
        session.outcome().is_none(),
        "a closed entry must not be given a result"
    );
    assert!(session.launch_error().is_none());
    assert!(!session.attempting());
    assert_eq!(
        session.phase(),
        Phase::Nothing,
        "closing the entry ends it; a late result does not bring it back"
    );
}

// ---------------------------------------------------------------------------
// PGR01-T4 — a launch result under a later subject
// ---------------------------------------------------------------------------

/// **Reachable today**, and this is the sequence the finding names. The nav
/// rail and the Library's "Choose local program" are both live while a launch
/// attempt is running, so a person can select a different program and have it
/// admitted before the first attempt returns.
///
/// The result of alpha's attempt is a real observation of alpha. It may be
/// discarded, and it may be shown; what it must never be is reported as beta's.
#[test]
fn t4_a_launch_result_must_not_be_reported_under_a_later_subject() {
    let scratch = Scratch::new("t4");
    let alpha = scratch.subject(ALPHA, 0);
    let beta = scratch.subject(BETA, 64);
    let workdir = scratch.dir("work");
    let other = scratch.dir("other");

    let mut session = Session::new();
    admit_both(&mut session, &alpha, &workdir);
    session.authorise();
    let (ticket, authorized) = start_attempt(&mut session);
    let outcome = helm_launch::launch(authorized);
    assert!(outcome.is_ok(), "the fixture subject launches");

    // While alpha's attempt is outstanding the person goes back to the chooser
    // and opens a different program, which is admitted before alpha returns.
    session.go_choose();
    admit_both(&mut session, &beta, &other);
    assert_eq!(
        session.program.as_ref().map(|f| f.name.as_str()),
        Some(BETA),
        "beta is the open subject now"
    );

    // Alpha's launch finally returns.
    deliver_launch(&mut session, &ticket, outcome);

    assert!(
        !(session.phase() == Phase::Ended && session.program_name() == BETA),
        "alpha's launch result was reported as an ended attempt of beta: phase {:?}, subject {:?}",
        session.phase(),
        session.program_name()
    );
    assert!(
        !(session.phase() == Phase::Ended
            && session
                .program
                .as_ref()
                .is_some_and(|f| f.path == beta.display().to_string())),
        "an ended attempt was labelled with beta's path"
    );
}

// ---------------------------------------------------------------------------
// PGR01-T5 — attempt-again generations
// ---------------------------------------------------------------------------

/// **Reachable today.** "Attempt launch again" re-opens and re-admits both
/// objects, and the Result surface it came from stays reachable from the
/// Chosen program screen, so a second press starts a second generation while
/// the first is still running.
#[test]
fn t5_an_earlier_admission_must_not_satisfy_a_later_attempt_again_generation() {
    let scratch = Scratch::new("t5");
    let alpha = scratch.subject(ALPHA, 0);
    let workdir = scratch.dir("work");

    let mut session = Session::new();
    admit_both(&mut session, &alpha, &workdir);
    session.authorise();
    let (ticket, authorized) = start_attempt(&mut session);
    let outcome = helm_launch::launch(authorized);
    assert!(outcome.is_ok(), "the fixture subject launches");
    deliver_launch(&mut session, &ticket, outcome);
    assert_eq!(session.phase(), Phase::Ended);

    // Generation two, then generation three before generation two returns.
    let (exec_two, dir_two) = attempt_again(&mut session);
    let (exec_three, dir_three) = attempt_again(&mut session);

    // Generation two's results arrive late.
    deliver_exec(&mut session, &exec_two, Ok(admit_exec(&alpha)));
    deliver_workdir(&mut session, &dir_two, Ok(admit_workdir(&workdir)));

    assert!(
        session.exec_pending(),
        "generation three's executable admission is still running"
    );
    assert!(
        session.workdir_pending(),
        "generation three's folder admission is still running"
    );
    assert!(
        !session.admitted(),
        "an earlier generation must not complete a later one's admission"
    );

    // Generation three's own results do complete it.
    deliver_exec(&mut session, &exec_three, Ok(admit_exec(&alpha)));
    deliver_workdir(&mut session, &dir_three, Ok(admit_workdir(&workdir)));
    assert!(!session.exec_pending());
    assert!(!session.workdir_pending());
    assert!(
        session.admitted(),
        "the current generation's own results complete it"
    );
}

// ---------------------------------------------------------------------------
// PGR01-T6 — out-of-order mixed worker messages
// ---------------------------------------------------------------------------

/// Only the current generation may change current state, in either order and
/// for either resource.
#[test]
fn t6_out_of_order_admission_results_must_not_overwrite_the_current_generation() {
    let scratch = Scratch::new("t6");
    let alpha = scratch.subject(ALPHA, 0);
    let beta = scratch.subject(BETA, 64);
    let first = scratch.dir("first");
    let second = scratch.dir("second");
    let beta_digest = digest_of(&beta);

    let mut session = Session::new();

    let exec_n = begin_exec(&mut session, &alpha);
    let exec_next = begin_exec(&mut session, &beta);
    let dir_n = begin_workdir(&mut session, &first);
    let dir_next = begin_workdir(&mut session, &second);

    // N+1 completes first for both resources.
    deliver_exec(&mut session, &exec_next, Ok(admit_exec(&beta)));
    deliver_workdir(&mut session, &dir_next, Ok(admit_workdir(&second)));
    assert!(session.admitted(), "the current generation completed");
    let settled = session
        .program
        .as_ref()
        .expect("the current generation produced program facts")
        .clone();
    assert_eq!(settled.name, BETA);
    assert_eq!(settled.sha256_hex, beta_digest);

    // N completes afterwards, for both resources, and changes nothing.
    deliver_exec(&mut session, &exec_n, Ok(admit_exec(&alpha)));
    deliver_workdir(&mut session, &dir_n, Ok(admit_workdir(&first)));

    let after = session
        .program
        .as_ref()
        .expect("the current generation's facts survive a stale result");
    assert_eq!(
        after.name, settled.name,
        "a stale result renamed the current subject"
    );
    assert_eq!(
        after.path, settled.path,
        "a stale result re-pathed the current subject"
    );
    assert_eq!(
        after.sha256_hex, beta_digest,
        "a stale result replaced the current subject's measurement"
    );
    assert_eq!(
        session.workdir_path.as_deref(),
        Some(second.as_path()),
        "a stale folder result changed the displayed folder"
    );
    assert!(session.admitted());
}
