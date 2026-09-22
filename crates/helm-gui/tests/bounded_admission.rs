//! `PGR-02` — a selected object must reach a HELM refusal, not wait forever
//! before admission can look at it.
//!
//! The adapter opens what a person selected and moves the owned descriptor
//! into `helm-launch`. Admission then classifies the object and refuses
//! anything that is not a regular file or a directory — but it cannot classify
//! anything until the caller-side open has returned.
//!
//! **This is not a claim about the window.** Admission already runs on a
//! worker thread, so the GTK main loop keeps running either way and there is
//! no frozen-window claim here or anywhere else. What these tests measure is
//! narrower and is the actual concern: whether the *operation* completes, so
//! that the person is told something.
//!
//! **Nor is it a claim that filesystem I/O is bounded.** It is not. A network
//! filesystem, a failing device or a pathological mount can still hold an open
//! or a read for as long as the kernel does, and nothing here changes that.
//! These tests cover one specific, entirely local case.
//!
//! The subject is a FIFO with no writer. Opening one read-only blocks until a
//! writer appears, by POSIX design; a person can select one from the file
//! chooser like any other filename, and `helm-launch` would refuse it the
//! moment it could see it.

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
use std::process::Command;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::thread;
use std::time::{Duration, Instant};

use helm_gui::vertical::{self, Refusal};
use helm_launch::AdmissionErrorCode;

/// Generous by three orders of magnitude: admitting a FIFO is `open`, `fcntl`
/// and `fstat` on a local filesystem. It is a bound, not a measurement.
const BOUND: Duration = Duration::from_secs(5);

const FIXTURE_MODE: u32 = 0o755;

struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "helm-g2-bounded-{}-{tag}-{}",
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

    /// A harmless local FIFO with no writer, made with `mkfifo(1)` so this
    /// crate needs no `libc` and no `unsafe` to build one.
    fn fifo(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        let status = Command::new("mkfifo")
            .arg(&path)
            .status()
            .expect("run mkfifo; coreutils is required to build this fixture");
        assert!(status.success(), "mkfifo failed for {path:?}");
        let kind = fs::symlink_metadata(&path)
            .expect("stat the fixture")
            .file_type();
        assert!(
            !kind.is_file() && !kind.is_dir(),
            "the fixture must be a special file, not a regular file or a directory"
        );
        path
    }

    fn subject(&self, name: &str) -> PathBuf {
        let source = Path::new(env!("CARGO_BIN_EXE_g2-launch-fixture"));
        assert!(source.is_file(), "fixture binary was not built: {source:?}");
        let target = self.root.join(name);
        fs::copy(source, &target).expect("copy fixture");
        fs::set_permissions(&target, fs::Permissions::from_mode(FIXTURE_MODE))
            .expect("set fixture mode");
        target
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Releases a reader blocked in `open(fifo, O_RDONLY)`, by becoming the writer
/// it is waiting for.
///
/// On Linux, opening a FIFO read-write returns immediately whether or not a
/// peer exists, so this cannot itself block and needs no flag constant. It
/// runs only when the reader is known to be stuck, and it is what lets a
/// failing run still tear down cleanly — and what demonstrates *why* the run
/// was stuck.
fn release_blocked_reader(fifo: &Path) -> bool {
    fs::File::options()
        .read(true)
        .write(true)
        .open(fifo)
        .is_ok()
}

/// What the adapter said about the object, reduced to something a test can
/// compare. A refusal is not stringly-typed in the product; it is here only so
/// the capability drops on the worker thread rather than crossing back.
#[derive(Debug, PartialEq, Eq)]
enum Said {
    Admitted,
    CallerCouldNotOpen,
    NotLocal,
    Refused(AdmissionErrorCode),
}

fn said(result: &Result<impl Sized, Refusal>) -> Said {
    match result {
        Ok(_) => Said::Admitted,
        Err(Refusal::Open(_)) => Said::CallerCouldNotOpen,
        Err(Refusal::NotLocal) => Said::NotLocal,
        Err(Refusal::Admission(error)) => Said::Refused(error.code()),
    }
}

/// Runs one adapter call on a worker, exactly as the interface does, and waits
/// [`BOUND`] for it. Returns how long it took and what it said.
///
/// On a timeout the FIFO is released so the worker can finish and the test can
/// join it: a run that fails still leaves nothing behind.
fn within_bound<F>(fifo: &Path, call: F) -> (Duration, Said)
where
    F: FnOnce() -> Said + Send + 'static,
{
    let (tx, rx) = channel();
    let worker = thread::spawn(move || {
        let started = Instant::now();
        let outcome = call();
        drop(tx.send((started.elapsed(), outcome)));
    });

    match rx.recv_timeout(BOUND) {
        Ok(answer) => {
            worker.join().expect("the worker finished");
            // Visible under `-- --nocapture`, so a run reports what it measured
            // rather than only that it was under the bound.
            println!("reached {:?} in {:?}", answer.1, answer.0);
            answer
        }
        Err(RecvTimeoutError::Timeout) => {
            let released = release_blocked_reader(fifo);
            let joined = worker.join().is_ok();
            panic!(
                "the caller-side open did not reach a HELM refusal within {BOUND:?}. \
                 Opening the FIFO for writing released it: {released}; worker finished after \
                 that: {joined}. The operation was waiting in open(2) for a writer that a \
                 person selecting a filename has no reason to provide."
            );
        }
        Err(RecvTimeoutError::Disconnected) => panic!("the worker ended without answering"),
    }
}

// ---------------------------------------------------------------------------
// The finding
// ---------------------------------------------------------------------------

#[test]
fn a_fifo_with_no_writer_reaches_a_bounded_refusal_through_the_executable_adapter() {
    let scratch = Scratch::new("exec");
    let fifo = scratch.fifo("chosen.fifo");

    let probe = fifo.clone();
    let (elapsed, answer) =
        within_bound(&fifo, move || said(&vertical::admit_executable_at(&probe)));

    assert_eq!(
        answer,
        Said::Refused(AdmissionErrorCode::NotRegularFile),
        "the object must be refused by helm-launch, which is the thing that classifies objects"
    );
    assert!(
        elapsed < BOUND,
        "the operation took {elapsed:?}, which is not bounded by {BOUND:?}"
    );
}

#[test]
fn a_fifo_with_no_writer_reaches_a_bounded_refusal_through_the_folder_adapter() {
    let scratch = Scratch::new("folder");
    let fifo = scratch.fifo("chosen.fifo");

    let probe = fifo.clone();
    let (elapsed, answer) = within_bound(&fifo, move || {
        said(&vertical::admit_working_directory_at(&probe))
    });

    assert_eq!(
        answer,
        Said::Refused(AdmissionErrorCode::NotDirectory),
        "a FIFO is not a folder, and helm-launch is what says so"
    );
    assert!(elapsed < BOUND, "the operation took {elapsed:?}");
}

// ---------------------------------------------------------------------------
// What the correction must not cost
//
// The open mode the adapter uses has to stay one the accepted admission takes.
// `helm-launch` inspects the descriptor's access mode with `F_GETFL` and
// admits `O_RDONLY` only, refusing `O_PATH`, `O_WRONLY` and `O_RDWR` with
// `DescriptorModeUnsuitable`. These two tests fail if the caller-side open
// ever stops satisfying that.
// ---------------------------------------------------------------------------

#[test]
fn a_real_executable_is_still_admitted_through_the_caller_side_open() {
    let scratch = Scratch::new("elf");
    let subject = scratch.subject("g2-subject");

    match vertical::admit_executable_at(&subject) {
        Ok(capability) => {
            let measurement = capability.measurement();
            assert!(
                measurement.pre_exec_body_size() > 0,
                "the whole body is still measured through this descriptor"
            );
            assert_eq!(
                measurement.pre_exec_mode_bits(),
                u16::try_from(FIXTURE_MODE).expect("the fixture mode fits"),
            );
        }
        Err(refusal) => panic!(
            "the caller-side open no longer produces a descriptor the accepted admission \
             takes: {refusal:?}"
        ),
    }
}

#[test]
fn a_real_directory_is_still_admitted_through_the_caller_side_open() {
    let scratch = Scratch::new("dir");
    let workdir = scratch.dir("work");

    match vertical::admit_working_directory_at(&workdir) {
        Ok(capability) => assert_eq!(capability.id(), vertical::WORKING_DIRECTORY_ID),
        Err(refusal) => panic!(
            "the caller-side open no longer produces a directory descriptor the accepted \
             admission takes: {refusal:?}"
        ),
    }
}
