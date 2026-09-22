//! The GUI-side orchestration adapter for the first real G2 vertical.
//!
//! **This is not a HELM orchestrator.** It is the smallest concrete adapter for
//! one flow: the paths a person selected, opened by the caller, handed to the
//! accepted `helm-launch` public API, and the resulting facts handed back to the
//! user interface. There is deliberately no trait, no service, no provider and
//! no manager here — nothing a future subsystem could be tempted to implement.
//!
//! **`helm-launch` is consumed, never extended.** Every step below calls the
//! accepted public API exactly as published. This crate adds no convenience
//! wrapper to it, duplicates none of its checks, and reaches none of its
//! internals.
//!
//! ## Where authority lives
//!
//! The GUI knows the paths because a person chose them, and it displays them.
//! That is **not** path-based authority: `helm-launch` never receives a path.
//! The caller — this adapter — opens the object the person selected and moves
//! the owned descriptor in. The accepted admission functions then inspect and
//! pin that already-open object, and the descriptor is the only execution
//! authority that exists.

use std::fs::File;
use std::io;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::thread;

use helm_launch::{
    AdmissionError, AuthorizationRefusal, AuthorizedLaunch, ExecutableCapability, LaunchError,
    LaunchOutcome, LaunchPlanErrors, ValidatedLaunchPlan, WorkingDirectoryCapability,
    admit_executable, admit_working_directory, authorize, launch, parse_launch_plan,
};

// ---------------------------------------------------------------------------
// The fixed first-vertical policy.
//
// There is no plan editor and no settings surface. This slice runs one visible
// policy, and the Authority screen states every value of it.
// ---------------------------------------------------------------------------

/// The plan's working-directory capability identifier. An identifier, never a
/// path: `helm-launch` matches it against the admitted capability's own
/// identifier and nothing else.
pub const WORKING_DIRECTORY_ID: &str = "g2-workdir";

/// Run deadline, matching the accepted prototype's stated bound.
pub const TIMEOUT_MS: u32 = 30_000;

/// Grace between the stop request and the force, matching the prototype.
pub const GRACE_MS: u32 = 5_000;

/// Bounded capture per stream. Well under the accepted maximum: this is a
/// prefix for a person to read on screen, not a transcript, and the interface
/// says so when it is truncated.
pub const CAPTURE_PREFIX_BYTES: u32 = 16_384;

/// The plan document this slice always builds, with only `argv[0]` varying.
///
/// Built through `serde_json`, never by string concatenation, so a selected
/// file name containing a quote or a backslash cannot produce a malformed
/// document. The bytes then go through the accepted parser like any other
/// untrusted input — this adapter does not bypass it and does not pre-validate.
#[must_use]
pub fn plan_bytes(argv0: &str) -> Vec<u8> {
    let document = serde_json::json!({
        "schema": "helm-launch-plan",
        "version": "0.1",
        "execution_kind": "linux_exact_executable",
        "argv": [argv0],
        "environment": { "mode": "empty" },
        "working_directory": { "capability_id": WORKING_DIRECTORY_ID },
        "stdin": { "mode": "closed_pipe_eof" },
        "stdout": { "capture_prefix_bytes": CAPTURE_PREFIX_BYTES },
        "stderr": { "capture_prefix_bytes": CAPTURE_PREFIX_BYTES },
        "timeout_ms": TIMEOUT_MS,
        "termination": { "signal": "SIGTERM", "grace_ms": GRACE_MS },
    });
    serde_json::to_vec(&document).unwrap_or_default()
}

/// A deterministic minimal `argv[0]`: the selected file's own name.
///
/// It is caller data, not an invention, and it is never a path, a command
/// string or anything a shell would see — no shell exists in this flow.
#[must_use]
pub fn argv0_for(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("program")
        .to_owned()
}

/// Parses the fixed plan through the accepted public parser.
///
/// # Errors
///
/// The parser's own bounded [`LaunchPlanErrors`].
pub fn parse_plan(argv0: &str) -> Result<ValidatedLaunchPlan, LaunchPlanErrors> {
    parse_launch_plan(&plan_bytes(argv0))
}

// ---------------------------------------------------------------------------
// Refusals the interface has to explain
// ---------------------------------------------------------------------------

/// Why a selection did not become a capability.
///
/// `NotLocal` and `Open` belong to the caller — this adapter — and happen
/// before `helm-launch` is consulted at all. `Admission` is the accepted
/// crate's own closed refusal, carried through unchanged.
#[derive(Debug)]
pub enum Refusal {
    /// The chooser returned something with no local filesystem path.
    NotLocal,
    /// The caller could not open the selected object.
    Open(io::Error),
    /// `helm-launch` refused to admit the opened object.
    Admission(AdmissionError),
}

impl Refusal {
    /// The accepted closed code, when the refusal came from `helm-launch`.
    #[must_use]
    pub const fn admission(&self) -> Option<&AdmissionError> {
        match self {
            Self::Admission(error) => Some(error),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Caller-side opening
// ---------------------------------------------------------------------------

/// Opens a local object read-only as the caller and hands back the owned
/// descriptor.
///
/// `File::open` is `O_RDONLY`, which is the one access mode the accepted
/// executable admission will take. Nothing here inspects the object: judging
/// it is `helm-launch`'s job, and this adapter does not pre-empt it by looking
/// at an extension, a name or a magic number.
fn open_read_only(path: &Path) -> Result<OwnedFd, Refusal> {
    File::open(path).map(OwnedFd::from).map_err(Refusal::Open)
}

/// Opens and admits one executable. Blocking: it measures the whole body, so
/// callers on the GTK thread must use [`spawn_admit_executable`] instead.
///
/// # Errors
///
/// [`Refusal`], which is either a caller-side open failure or the accepted
/// crate's own [`AdmissionError`].
pub fn admit_executable_at(path: &Path) -> Result<ExecutableCapability, Refusal> {
    let fd = open_read_only(path)?;
    admit_executable(fd).map_err(Refusal::Admission)
}

/// Opens and admits one working directory.
///
/// On Linux a directory opens `O_RDONLY` like any other object, so the caller
/// hands in a descriptor exactly as it does for the executable.
///
/// # Errors
///
/// [`Refusal`], as above. A refusal here is **not** a launch error: no launch
/// exists yet.
pub fn admit_working_directory_at(path: &Path) -> Result<WorkingDirectoryCapability, Refusal> {
    let fd = open_read_only(path)?;
    admit_working_directory(WORKING_DIRECTORY_ID, fd).map_err(Refusal::Admission)
}

/// Composes the one-shot authority, at the person's explicit instruction.
///
/// # Errors
///
/// [`AuthorizationRefusal`], raised before any process could exist.
pub fn authorise(
    plan: ValidatedLaunchPlan,
    executable: ExecutableCapability,
    working_directory: WorkingDirectoryCapability,
) -> Result<AuthorizedLaunch, AuthorizationRefusal> {
    authorize(plan, executable, working_directory)
}

// ---------------------------------------------------------------------------
// Worker threads
//
// `helm_launch::launch` is synchronous and stays that way. Running it on a
// worker thread is a GUI adaptation and nothing more: it creates no session
// handle, no stop, no cancel and no live child state, and the interface offers
// none of those. Only inert values cross back, never a GTK object.
// ---------------------------------------------------------------------------

/// The identity of one asynchronous operation.
///
/// It is minted before the worker starts, moves with the request, and comes
/// back attached to the result. The main thread then compares it against the
/// operation that is currently allowed to change that piece of state, and a
/// result whose operation is no longer that one is dropped.
///
/// It is **opaque and inert**: it names nothing, resolves nothing, authorises
/// nothing and outlives no process. It is not a session handle, not a process
/// handle and not a durable identity — G-1 and G-2 remain unauthorised. Its
/// only use is that comparison.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OperationId(u64);

impl OperationId {
    /// Minted by the session, which owns the only counter.
    pub(crate) const fn from_raw(value: u64) -> Self {
        Self(value)
    }
}

/// What a worker hands back to the main thread.
///
/// Every variant carries the identity of the operation that produced it. The
/// kind of result is not enough: two admissions of the same kind are told
/// apart by their operation, never by their arrival order.
pub enum Message {
    /// Executable admission finished.
    Executable {
        op: OperationId,
        result: Result<ExecutableCapability, Refusal>,
    },
    /// Working-directory admission finished.
    WorkingDirectory {
        op: OperationId,
        result: Result<WorkingDirectoryCapability, Refusal>,
    },
    /// The synchronous launch call returned.
    Launched {
        op: OperationId,
        result: Box<Result<LaunchOutcome, LaunchError>>,
    },
}

/// Admits an executable off the GTK thread.
///
/// Measuring a large executable reads and hashes the whole body, so doing this
/// on the main loop would freeze the window.
pub fn spawn_admit_executable(op: OperationId, path: PathBuf, tx: Sender<Message>) {
    thread::spawn(move || {
        let result = admit_executable_at(&path);
        // A closed receiver means the window went away; there is nothing to
        // report to and nothing to clean up but the capability, which drops
        // here and closes its descriptor. The same is true of a result the
        // main thread drops as stale.
        drop(tx.send(Message::Executable { op, result }));
    });
}

/// Admits a working directory off the GTK thread, for symmetry and because a
/// directory on a slow mount can still block.
pub fn spawn_admit_working_directory(op: OperationId, path: PathBuf, tx: Sender<Message>) {
    thread::spawn(move || {
        let result = admit_working_directory_at(&path);
        drop(tx.send(Message::WorkingDirectory { op, result }));
    });
}

/// Runs the accepted synchronous launch on a worker thread.
///
/// The authority is moved in and consumed exactly once. Nothing is returned
/// that could launch again.
pub fn spawn_launch(op: OperationId, authorized: AuthorizedLaunch, tx: Sender<Message>) {
    thread::spawn(move || {
        let result = launch(authorized);
        drop(tx.send(Message::Launched {
            op,
            result: Box::new(result),
        }));
    });
}
