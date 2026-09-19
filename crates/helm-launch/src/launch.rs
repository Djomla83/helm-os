//! The public Linux x86_64 launch, its observation loop and the real receipt
//! (**P4**, plan sections 5.2, 8.5, 8.6 and 13).
//!
//! # What this module is
//!
//! It is the **only** public way to make this crate create a process, and it is
//! compiled only under `cfg(all(target_os = "linux", target_arch = "x86_64"))`.
//! [`launch`] consumes an `AuthorizedLaunch` exactly once, asks the P3 backend
//! for **one** direct child, observes that child under fixed bounds, and
//! returns a [`LaunchOutcome`] carrying a deterministic
//! [`LaunchReceipt`](crate::LaunchReceipt).
//!
//! # Where the policy lives
//!
//! **Not here.** The accepted lifecycle policy is the pure state machine in
//! [`crate::lifecycle`], which P1 wrote and tested on every platform. This
//! module is its **input/output adapter**: it turns `poll` results and reads
//! into [`Event`]s, hands them to [`Lifecycle::step`], and performs the
//! [`Action`]s the model returns. There is deliberately **no second copy of
//! the policy**: every deadline, every classification, the `EndNotObserved`
//! latch and the whole guarded group-sweep decision are the model's, so the
//! real loop cannot drift from the model that is tested portably.
//!
//! # The `unsafe` boundary does not move
//!
//! This file contains **no `unsafe`**. Process creation stays inside
//! `src/backend/`; everything here — `poll`, the reads, `pidfd_send_signal`,
//! `waitid` and the process-group signal — is a safe `rustix` call. No `libc`
//! function is called and no raw syscall shim exists outside the backend.
//!
//! # What is deliberately absent
//!
//! No sandbox, no containment, no cgroup, no process-tree supervision, no Wine,
//! no orchestration, no async API, no receipt signature, verification
//! constructor or provenance claim, and **no positive exec-success claim**: a
//! clean exec-status end-of-file is
//! [`IndeterminateReason::StatusEofWithoutRecord`] and the run deadline starts
//! at exactly that event, never at a confirmed execution.

use std::collections::VecDeque;
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::process::{
    Pid, Signal, WaitId, WaitIdOptions, kill_process_group, pidfd_send_signal, waitid,
};
use sha2::{Digest as _, Sha256};

use crate::authority::AuthorizedLaunch;
use crate::backend::{
    self, BackendError, ChildHandle, POST_KILL_REAP_MS, PrepareStep, SPAWN_CONFIRM_TIMEOUT_MS,
    STATUS_RECORD_BYTES, stage_of,
};
use crate::error::{LaunchError, LaunchErrorCode, PreparationStep};
use crate::lifecycle::{
    Action, Event, Facts, GroupAuthority, Lifecycle, POST_EXIT_DRAIN_MS, Probe, Reap,
};
use crate::model::{
    AssertedContext, Backend, ChildEnd, Completeness, Digest, ExecutableMeasurement, ReceiptRecord,
    Stream, StreamFacts, Termination,
};
use crate::plan::ValidatedLaunchPlan;
use crate::receipt::LaunchReceipt;

/// Longest single `poll` wait.
///
/// Every wait this loop performs is the minimum of the model's next deadline
/// and this cap, so a missing deadline can never become an unbounded wait: the
/// loop still wakes, re-evaluates and re-checks the absolute guard below.
const MAX_POLL_WAIT_MS: u64 = 250;

/// Slack added to the accepted sum of the fixed bounds to form the absolute
/// guard. The guard exists so that "no unbounded wait after child creation" is
/// a **structural** property of this loop and not only an argument about the
/// model's deadlines.
const GUARD_SLACK_MS: u64 = 5_000;

/// One read buffer's size. Output is counted and hashed in full; only the
/// plan's `capture_prefix_bytes` are kept.
const READ_CHUNK_BYTES: usize = 16 * 1024;

/// The total wall-clock bound `launch` observes for one child, in milliseconds
/// (plan section 8.6).
///
/// `POST_KILL_REAP_MS` appears **twice**: once for the lifecycle's own bounded
/// post-`SIGKILL` observation, and once for the P3 `ChildHandle` drop guard,
/// which is reached only on the pathological `EndNotObserved` path where a
/// child has survived `SIGKILL` for the whole first bound.
fn total_bound_ms(timeout_ms: u32, grace_ms: u32) -> u64 {
    SPAWN_CONFIRM_TIMEOUT_MS
        .saturating_add(u64::from(timeout_ms))
        .saturating_add(u64::from(grace_ms))
        .saturating_add(POST_KILL_REAP_MS)
        .saturating_add(POST_KILL_REAP_MS)
        .saturating_add(POST_EXIT_DRAIN_MS)
}

// ------------------------------------------------------------ LaunchOutcome

/// What one attempted launch produced.
///
/// **Owns no descriptor.** Every pipe end, the process descriptor and the
/// direct-child handle are closed before `launch` returns, so no process handle
/// and no authority-bearing value survives here. The output prefixes live in
/// memory only and are **never** serialised into the receipt.
///
/// There is no public constructor, no `Default`, no `Clone`, no `Serialize` and
/// no `Deserialize`: outside this crate a `LaunchOutcome` can only be read, and
/// only one that `launch` produced can exist.
pub struct LaunchOutcome {
    receipt: LaunchReceipt,
    stdout_prefix: Vec<u8>,
    stderr_prefix: Vec<u8>,
    stdout_truncated: bool,
    stderr_truncated: bool,
    elapsed: Duration,
}

impl LaunchOutcome {
    /// The deterministic receipt for this attempt.
    #[must_use]
    pub const fn receipt(&self) -> &LaunchReceipt {
        &self.receipt
    }

    /// The first `capture_prefix_bytes` the plan asked for, from this stream.
    ///
    /// In memory only. These bytes are **never** part of the receipt and never
    /// reach any durable artifact this crate produces.
    #[must_use]
    pub fn stdout_prefix(&self) -> &[u8] {
        &self.stdout_prefix
    }

    /// The captured prefix of the child's standard error. In memory only.
    #[must_use]
    pub fn stderr_prefix(&self) -> &[u8] {
        &self.stderr_prefix
    }

    /// Whether more bytes were drained from `stream` than the prefix kept.
    ///
    /// The excess was counted and hashed before it was discarded, so
    /// [`StreamFacts::bytes_drained`] and [`StreamFacts::drained_sha256`] cover
    /// every byte, not only the prefix.
    #[must_use]
    pub const fn prefix_truncated(&self, stream: Stream) -> bool {
        match stream {
            Stream::Stdout => self.stdout_truncated,
            Stream::Stderr => self.stderr_truncated,
        }
    }

    /// Monotonic wall-clock time this launch took.
    ///
    /// In memory only: no duration and no timestamp is ever recorded in a
    /// receipt, because neither would be deterministic.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Consume the outcome and keep only the inert receipt.
    #[must_use]
    pub fn into_receipt(self) -> LaunchReceipt {
        self.receipt
    }
}

/// Prints lengths, never captured bytes.
impl core::fmt::Debug for LaunchOutcome {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LaunchOutcome")
            .field("receipt_sha256", &self.receipt.sha256())
            .field("stdout_prefix_len", &self.stdout_prefix.len())
            .field("stderr_prefix_len", &self.stderr_prefix.len())
            .field("stdout_prefix_truncated", &self.stdout_truncated)
            .field("stderr_prefix_truncated", &self.stderr_truncated)
            .finish_non_exhaustive()
    }
}

// -------------------------------------------------------------------- launch

/// Execute the one object an [`AuthorizedLaunch`] authorises, and observe it.
///
/// The authorisation is **consumed**: there is no way to launch it twice.
///
/// # The `Err` boundary
///
/// **Before a direct child exists**, every failure is `Err(LaunchError)` and no
/// receipt exists. **Once `clone3` has returned a child**, this returns
/// `Ok(LaunchOutcome)` with a receipt **whatever happened** — a child setup
/// failure, an `execveat` failure, indeterminate exec evidence, a timeout, a
/// signal this crate sent, a stream read failure, or an end that was never
/// observed. A child attempt is never lost behind an error value.
///
/// # What it never claims
///
/// A clean exec-status end-of-file establishes only
/// `Indeterminate(StatusEofWithoutRecord)`. It is not exec success, not a
/// confirmation that the image ran, and no `ExecSucceeded` value exists
/// anywhere in this crate.
///
/// # Bounds
///
/// Synchronous, and bounded by [`total_bound_ms`] plus scheduling slack. It is
/// not async, not re-entrant from a signal handler and not `Sync`-shared.
///
/// # Errors
///
/// [`LaunchError`] when preparation or process creation failed and **no child
/// was created**.
pub fn launch(authorized: AuthorizedLaunch) -> Result<LaunchOutcome, LaunchError> {
    let started = Instant::now();
    // Everything that can fail without creating a child fails here.
    let spawned = backend::spawn_for_lifecycle(authorized).map_err(to_launch_error)?;
    // A child exists from this line on: no path below returns `Err`.
    Ok(Observer::new(spawned).run(started))
}

/// Map the crate-private backend refusal onto the public vocabulary.
///
/// Every variant reachable from `spawn_for_lifecycle` is raised **before**
/// `clone3` created anything.
fn to_launch_error(error: BackendError) -> LaunchError {
    match error {
        BackendError::Preparation { step, errno } => {
            LaunchError::preparation(public_step(step), errno)
        }
        BackendError::SignalMaskFailed { errno } => {
            LaunchError::preparation(PreparationStep::SignalMask, errno)
        }
        BackendError::ProcessCreationUnavailable { errno } => {
            LaunchError::with_errno(LaunchErrorCode::ProcessCreationUnavailable, errno)
        }
        BackendError::ProcessCreationFailed { errno } => {
            LaunchError::with_errno(LaunchErrorCode::ProcessCreationFailed, errno)
        }
        // Unreachable from this entry point: the mask restore is returned as a
        // fact, and no signal is sent before the observation loop owns the
        // child. Mapped rather than asserted, because an internal invariant may
        // never panic in a library.
        BackendError::SignalMaskRestoreFailed { .. }
        | BackendError::SignalFailed { .. }
        | BackendError::PidfdNotProvided
        | BackendError::InternalInvariant(_) => {
            LaunchError::new(LaunchErrorCode::InternalInvariant)
        }
    }
}

const fn public_step(step: PrepareStep) -> PreparationStep {
    match step {
        PrepareStep::Pipe => PreparationStep::Pipe,
        PrepareStep::Relocate => PreparationStep::Relocate,
        PrepareStep::NonBlocking => PreparationStep::NonBlocking,
    }
}

// ------------------------------------------------------------ stream capture

/// One stream's in-flight observation.
///
/// Every drained byte is counted and hashed. Only the plan's prefix bound is
/// retained, and the storage for it is allocated **once**, so a child that
/// writes gigabytes cannot grow this.
struct Capture {
    fd: Option<OwnedFd>,
    hasher: Sha256,
    bytes: u64,
    prefix: Vec<u8>,
    limit: usize,
    truncated: bool,
}

impl Capture {
    fn new(fd: OwnedFd, limit: usize) -> Self {
        Self {
            fd: Some(fd),
            hasher: Sha256::new(),
            bytes: 0,
            prefix: Vec::with_capacity(limit),
            limit,
            truncated: false,
        }
    }

    /// Count, hash, then keep only what still fits in the prefix.
    fn absorb(&mut self, chunk: &[u8]) {
        self.bytes = self
            .bytes
            .saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
        self.hasher.update(chunk);
        let room = self.limit.saturating_sub(self.prefix.len());
        if room == 0 {
            self.truncated = self.truncated || !chunk.is_empty();
            return;
        }
        if chunk.len() > room {
            self.prefix.extend_from_slice(&chunk[..room]);
            self.truncated = true;
        } else {
            self.prefix.extend_from_slice(chunk);
        }
    }

    fn facts(&self, completeness: Completeness) -> StreamFacts {
        StreamFacts {
            bytes_drained: self.bytes,
            drained_sha256: Digest::from_raw(self.hasher.clone().finalize().into()),
            completeness,
        }
    }
}

/// Read a non-blocking descriptor until it would block, ends, or fails.
///
/// `EINTR` is retried without consuming any deadline. A descriptor leaves the
/// poll set only on a zero-byte read or a real read error — never because
/// `POLLHUP` was reported.
enum Drained {
    /// The descriptor would block; it stays in the poll set.
    WouldBlock,
    /// A zero-byte read: end-of-file, and nothing else, is EOF.
    Eof,
    /// A real read error. The descriptor leaves the poll set.
    Failed(i32),
}

fn drain(fd: BorrowedFd<'_>, mut absorb: impl FnMut(&[u8])) -> Drained {
    let mut buffer = [0_u8; READ_CHUNK_BYTES];
    loop {
        match rustix::io::read(fd, &mut buffer) {
            Ok(0) => return Drained::Eof,
            Ok(read) => {
                if let Some(chunk) = buffer.get(..read) {
                    absorb(chunk);
                }
            }
            Err(rustix::io::Errno::INTR) => {}
            Err(rustix::io::Errno::AGAIN) => return Drained::WouldBlock,
            Err(errno) => return Drained::Failed(errno.raw_os_error()),
        }
    }
}

// ---------------------------------------------------------------- the loop

/// The observation loop: the model's input/output adapter.
struct Observer {
    child: ChildHandle,
    life: Lifecycle,
    status: Option<OwnedFd>,
    status_buffer: [u8; 9],
    status_filled: usize,
    status_record_seen: bool,
    stdout: Capture,
    stderr: Capture,
    /// Whether the process descriptor has already reported the child's end.
    ///
    /// A pidfd stays readable once the child has ended, so leaving it in the
    /// poll set would make every later wait return at once and turn the
    /// post-exit drain into a busy loop. The model is told exactly once, and
    /// the descriptor then leaves the set; the handle itself stays owned,
    /// because the reap still needs it.
    end_observed: bool,
    plan: ValidatedLaunchPlan,
    measurement: ExecutableMeasurement,
    spawned_at: Instant,
    guard_ms: u64,
}

impl Observer {
    fn new(spawned: backend::SpawnedForLifecycle) -> Self {
        let backend::SpawnedForLifecycle {
            child,
            group_authority_established,
            status_read,
            stdout_read,
            stderr_read,
            measurement,
            plan,
        } = spawned;

        let authority = if group_authority_established {
            GroupAuthority::Established
        } else {
            GroupAuthority::NotEstablished
        };
        let life = Lifecycle::new(0, plan.timeout_ms(), plan.grace_ms(), authority);
        let guard_ms =
            total_bound_ms(plan.timeout_ms(), plan.grace_ms()).saturating_add(GUARD_SLACK_MS);
        let stdout_limit = usize::try_from(plan.capture_prefix_bytes(Stream::Stdout)).unwrap_or(0);
        let stderr_limit = usize::try_from(plan.capture_prefix_bytes(Stream::Stderr)).unwrap_or(0);

        Self {
            child,
            life,
            status: Some(status_read),
            status_buffer: [0; 9],
            status_filled: 0,
            status_record_seen: false,
            stdout: Capture::new(stdout_read, stdout_limit),
            stderr: Capture::new(stderr_read, stderr_limit),
            end_observed: false,
            plan,
            measurement,
            spawned_at: Instant::now(),
            guard_ms,
        }
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.spawned_at.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// Step the model with one event and perform everything it asks for,
    /// feeding the results of probe and reap straight back in.
    fn drive(&mut self, now: u64, event: Event) {
        let mut queue: VecDeque<Event> = VecDeque::new();
        queue.push_back(event);
        while let Some(next) = queue.pop_front() {
            for action in self.life.step(now, next) {
                match action {
                    Action::ReadStatus => queue.extend(self.read_status()),
                    Action::ReadStream(stream) => queue.extend(self.read_stream(stream)),
                    Action::SendSigterm => self.signal(Signal::TERM),
                    Action::SendSigkill => self.signal(Signal::KILL),
                    Action::ProbeReaped => queue.push_back(Event::Probed(self.probe())),
                    Action::SweepGroup => self.sweep_group(),
                    Action::Reap => queue.push_back(Event::Reaped(self.reap())),
                }
            }
        }
    }

    /// Drain the exec-status channel and say what it established.
    ///
    /// One complete eight-byte record becomes [`Event::StatusRecord`]; a byte
    /// beyond it, or a short record followed by end-of-file, becomes
    /// [`Event::StatusMalformed`]; a zero-byte read is the only end-of-file.
    fn read_status(&mut self) -> Vec<Event> {
        let Some(fd) = self.status.as_ref() else {
            return Vec::new();
        };
        let mut events = Vec::new();
        let mut buffer = self.status_buffer;
        let mut filled = self.status_filled;
        let mut record_seen = self.status_record_seen;
        let outcome = drain(fd.as_fd(), |chunk| {
            for byte in chunk {
                if let Some(slot) = buffer.get_mut(filled) {
                    *slot = *byte;
                    filled = filled.saturating_add(1);
                } else {
                    // More than one record plus its detector byte.
                    filled = filled.saturating_add(1);
                }
            }
        });
        // A complete record, exactly once.
        if !record_seen && filled >= STATUS_RECORD_BYTES {
            record_seen = true;
            match (
                stage_of(buffer[0]),
                buffer[1],
                buffer[2],
                buffer[3],
                filled == STATUS_RECORD_BYTES,
            ) {
                (Some(stage), 0, 0, 0, true) => events.push(Event::StatusRecord {
                    stage,
                    errno: i32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]),
                }),
                // An unknown stage byte, a non-zero reserved pad, or a byte
                // beyond the fixed record: malformed, never a guess.
                _ => events.push(Event::StatusMalformed),
            }
        } else if record_seen && filled > STATUS_RECORD_BYTES {
            events.push(Event::StatusMalformed);
        }
        self.status_buffer = buffer;
        self.status_filled = filled;
        self.status_record_seen = record_seen;

        match outcome {
            Drained::WouldBlock => {}
            Drained::Eof => {
                self.status = None;
                // A short record that never completed is malformed, and the
                // model must learn that before it classifies the end-of-file.
                if !record_seen && filled > 0 {
                    events.push(Event::StatusMalformed);
                }
                events.push(Event::StatusEof);
            }
            Drained::Failed(errno) => {
                self.status = None;
                events.push(Event::StatusReadFailed { errno });
            }
        }
        events
    }

    fn read_stream(&mut self, stream: Stream) -> Vec<Event> {
        let capture = match stream {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        };
        // Moving the descriptor out leaves the capture free to be borrowed for
        // the whole drain, and puts it back only if the stream is still live.
        let Some(fd) = capture.fd.take() else {
            return Vec::new();
        };
        let outcome = drain(fd.as_fd(), |chunk| capture.absorb(chunk));
        match outcome {
            // Still open, still polled. `POLLHUP` alone never removes it.
            Drained::WouldBlock => {
                capture.fd = Some(fd);
                Vec::new()
            }
            Drained::Eof => vec![Event::StreamEof(stream)],
            Drained::Failed(errno) => vec![Event::StreamReadFailed { stream, errno }],
        }
    }

    /// One signal to the **direct child, through its process descriptor**.
    ///
    /// Never by numeric pid: a pid can be reused, a process descriptor cannot.
    /// The model records that the action was taken; whether the child then ends
    /// by that signal is a separate observation it never assumes.
    fn signal(&self, signal: Signal) {
        let _ = pidfd_send_signal(self.child.pidfd(), signal);
    }

    /// The non-consuming probe that guards the sweep.
    ///
    /// `WNOWAIT` leaves the child collectable, so this can never take the end
    /// the reap must classify. `ECHILD` — or any other refusal — means this
    /// process can no longer wait for that child, so no group derived from it
    /// may be signalled.
    fn probe(&self) -> Probe {
        match waitid(
            WaitId::PidFd(self.child.pidfd()),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        ) {
            Ok(_) => Probe::Unreaped,
            Err(rustix::io::Errno::INTR) => Probe::Unreaped,
            Err(_) => Probe::AlreadyReaped,
        }
    }

    /// The one guarded process-group cleanup signal.
    ///
    /// Reached **only** when the model has established that the parent's own
    /// `setpgid(child, child)` succeeded and that the probe found the child
    /// still collectable. The target is the group that call created, whose
    /// identifier is the direct child's own pid — never a group discovered by
    /// asking the system what group anything is in.
    ///
    /// **This is best-effort cleanup, not containment.** Issuing it says only
    /// that the call was made: no descendant is claimed to have received it,
    /// died, or been contained, and a descendant that left the group by
    /// `setsid`, `setpgid` or a service handoff survives.
    fn sweep_group(&self) {
        let Ok(pid) = i32::try_from(self.child.pid()) else {
            return;
        };
        let Some(group) = Pid::from_raw(pid) else {
            return;
        };
        let _ = kill_process_group(group, Signal::KILL);
    }

    /// The final non-blocking, descriptor-based reap.
    fn reap(&self) -> Reap {
        match self.child.reap_once() {
            Some(ChildEnd::Exited { code }) => Reap::Exited { code },
            Some(ChildEnd::Signaled {
                signal,
                core_dumped: false,
            }) => Reap::Killed { signal },
            Some(ChildEnd::Signaled {
                signal,
                core_dumped: true,
            }) => Reap::Dumped { signal },
            // Nothing classifiable: the model decides what that means, and it
            // never overwrites a latched end.
            Some(_) => Reap::Unclassifiable,
            None => Reap::NothingAvailable,
        }
    }

    /// Run the loop to the model's own completion, then build the outcome.
    fn run(mut self, started: Instant) -> LaunchOutcome {
        let facts = self.observe();
        let elapsed = started.elapsed();
        let record = self.record(facts);
        LaunchOutcome {
            receipt: LaunchReceipt::from_record(record),
            stdout_prefix: core::mem::take(&mut self.stdout.prefix),
            stderr_prefix: core::mem::take(&mut self.stderr.prefix),
            stdout_truncated: self.stdout.truncated,
            stderr_truncated: self.stderr.truncated,
            elapsed,
        }
    }

    fn observe(&mut self) -> Facts {
        loop {
            if let Some(facts) = self.life.facts() {
                return facts;
            }
            let now = self.now_ms();

            // A due deadline is applied before anything is waited for. The
            // model applies the earliest due one per call, so repeating this
            // drains them in order.
            if let Some(deadline) = self.life.next_deadline()
                && now >= deadline
            {
                self.drive(now, Event::DeadlineReached);
                continue;
            }

            // The absolute guard. It cannot be reached while the model still
            // has deadlines of its own, and it makes "no unbounded wait after
            // child creation" structural rather than argued.
            if now >= self.guard_ms {
                self.drive(u64::MAX, Event::DeadlineReached);
                continue;
            }

            let wait = self
                .life
                .next_deadline()
                .map_or(MAX_POLL_WAIT_MS, |deadline| {
                    deadline.saturating_sub(now).min(MAX_POLL_WAIT_MS)
                });
            self.wait_once(wait);
        }
    }

    /// One bounded `poll` over the live descriptors, translated into events.
    fn wait_once(&mut self, wait_ms: u64) {
        // Order is fixed so the revents can be read back positionally.
        let mut slots: Vec<(Option<Stream>, bool)> = Vec::with_capacity(4);
        let mut fds: Vec<PollFd<'_>> = Vec::with_capacity(4);
        if let Some(status) = self.status.as_ref() {
            fds.push(PollFd::new(status, PollFlags::IN));
            slots.push((None, false));
        }
        if let Some(fd) = self.stdout.fd.as_ref() {
            fds.push(PollFd::new(fd, PollFlags::IN));
            slots.push((Some(Stream::Stdout), false));
        }
        if let Some(fd) = self.stderr.fd.as_ref() {
            fds.push(PollFd::new(fd, PollFlags::IN));
            slots.push((Some(Stream::Stderr), false));
        }
        if !self.end_observed {
            let pidfd = self.child.pidfd();
            fds.push(PollFd::from_borrowed_fd(pidfd, PollFlags::IN));
            slots.push((None, true));
        }
        if fds.is_empty() {
            // Nothing left to wait on: only the model's own deadlines can move
            // the lifecycle now, and `observe` applies them. Returning here
            // rather than calling `poll` with an empty set keeps the wait
            // bounded by that deadline instead of by a spin.
            std::thread::sleep(Duration::from_millis(wait_ms.min(MAX_POLL_WAIT_MS)));
            return;
        }

        let timeout = Timespec {
            tv_sec: (wait_ms / 1_000).try_into().unwrap_or(0),
            tv_nsec: ((wait_ms % 1_000).saturating_mul(1_000_000))
                .try_into()
                .unwrap_or(0),
        };
        match poll(&mut fds, Some(&timeout)) {
            // `EINTR` consumes no deadline: the loop recomputes `now` from the
            // monotonic clock and waits out whatever remains.
            Err(rustix::io::Errno::INTR) | Ok(0) => return,
            Err(_) => {
                // A `poll` that cannot report readiness is an internal
                // invariant after a child exists. It is never an error return:
                // the loop stops waiting on readiness and lets the model's own
                // deadlines finish the lifecycle.
                let now = self.now_ms();
                self.drive(now, Event::DeadlineReached);
                return;
            }
            Ok(_) => {}
        }

        let ready: Vec<(Option<Stream>, bool, PollFlags)> = fds
            .iter()
            .zip(slots.iter())
            .map(|(fd, (stream, is_pidfd))| (*stream, *is_pidfd, fd.revents()))
            .collect();
        let now = self.now_ms();
        for (stream, is_pidfd, revents) in ready {
            if revents.is_empty() {
                continue;
            }
            let hangup = revents.intersects(PollFlags::HUP | PollFlags::ERR | PollFlags::NVAL);
            // Readiness only ever asks for a read; `POLLIN` is handled before
            // any hangup is interpreted, and only a zero-byte read is EOF.
            match (stream, is_pidfd) {
                (_, true) => {
                    if revents.contains(PollFlags::IN) {
                        self.end_observed = true;
                        self.drive(now, Event::ChildEndReadable);
                    }
                }
                (Some(stream), false) => self.drive(now, Event::StreamReady { stream, hangup }),
                (None, false) => self.drive(now, Event::StatusReady { hangup }),
            }
        }
    }

    /// Turn the model's facts into the accepted receipt record.
    fn record(&self, facts: Facts) -> ReceiptRecord {
        let argument_count = u32::try_from(self.plan.argv().len()).unwrap_or(u32::MAX);
        ReceiptRecord {
            backend: Backend::LinuxX8664Clone3PidfdExecveat,
            plan_sha256: self.plan.sha256(),
            asserted_context: AssertedContext {
                subject_spec_sha256: self.plan.asserted_subject_spec_sha256(),
                binding_report_sha256: self.plan.asserted_binding_report_sha256(),
            },
            working_directory_id: self.plan.working_directory_capability_id().clone(),
            executable: self.measurement,
            argument_count,
            environment_mode: self.plan.environment_mode(),
            exec_status: facts.exec_status,
            child_end: facts.child_end,
            run_deadline_expired: facts.run_deadline_expired,
            termination: Termination {
                sigterm_sent: facts.sigterm_sent,
                sigkill_sent: facts.sigkill_sent,
                group_sweep: facts.group_sweep,
            },
            stdout: self.stdout.facts(facts.stdout),
            stderr: self.stderr.facts(facts.stderr),
        }
    }
}

// ===========================================================================
// P4 tests: the real observation loop against real children
// ===========================================================================
//
// Level 3 O/R/S/T/P, on purpose-built fixtures compiled at test time. **No
// LAUNCH-EXEC-01 asset is used**: no `launcher_spike`, no frozen runner and no
// frozen helper ELF. Every fixture here is new source written for this slice.
//
// Every test that creates a descendant cleans it up, and every real group
// sweep targets the **child's own dedicated group** — whose identifier is the
// direct child's pid, created by the `setpgid` pair P3 already performs — so no
// test can signal the cargo harness group or anything else on the host.

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "test code: a wrong assumption must fail loudly"
)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    use sha2::{Digest as _, Sha256};

    use super::{GUARD_SLACK_MS, LaunchOutcome, launch, total_bound_ms};
    use crate::authority::{admit_executable, admit_working_directory, authorize};
    use crate::backend::tests::{
        fixture_binary, fixture_root, open_read_only, require_tool, scratch_dir,
    };
    use crate::model::{
        ChildEnd, ChildStage, Completeness, Digest, ExecStatus, GroupSweep, IndeterminateReason,
        Stream,
    };
    use crate::plan::parse_launch_plan;

    // ------------------------------------------------------------- fixtures

    /// Exit immediately with the code named by `argv[1]`, writing nothing.
    const QUIET_SOURCE: &str = r##"
fn main() {
    let code: i32 = std::env::args().nth(1).unwrap_or_default().parse().unwrap_or(0);
    std::process::exit(code);
}
"##;

    /// Write `argv[1]` bytes to stdout and `argv[2]` bytes to stderr, then exit
    /// 0. The two streams are written in interleaved chunks, so a launcher that
    /// drained them one after the other would deadlock on the second.
    const WRITER_SOURCE: &str = r##"
use std::io::Write;
fn main() {
    let out_bytes: usize = std::env::args().nth(1).unwrap_or_default().parse().unwrap_or(0);
    let err_bytes: usize = std::env::args().nth(2).unwrap_or_default().parse().unwrap_or(0);
    let chunk = 4096usize;
    let mut out_written = 0usize;
    let mut err_written = 0usize;
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut out = stdout.lock();
    let mut err = stderr.lock();
    while out_written < out_bytes || err_written < err_bytes {
        if out_written < out_bytes {
            let n = chunk.min(out_bytes - out_written);
            let _ = out.write_all(&vec![b'O'; n]);
            let _ = out.flush();
            out_written += n;
        }
        if err_written < err_bytes {
            let n = chunk.min(err_bytes - err_written);
            let _ = err.write_all(&vec![b'E'; n]);
            let _ = err.flush();
            err_written += n;
        }
    }
}
"##;

    /// Sleep for `argv[1]` milliseconds. The default `SIGTERM` disposition ends
    /// it, because nothing here installs a handler.
    const SLEEPER_SOURCE: &str = r##"
fn main() {
    let ms: u64 = std::env::args().nth(1).unwrap_or_default().parse().unwrap_or(60_000);
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
"##;

    /// Start one descendant that inherits this image's stdout and outlives it,
    /// print the descendant's pid on stdout, then exit 0 at once.
    ///
    /// The launcher therefore observes the direct child's end while a writer it
    /// never created still holds the stream open.
    const RETAINER_SOURCE: &str = r##"
use std::io::Write;
fn main() {
    let helper = std::env::args().nth(1).unwrap_or_default();
    let ms = std::env::args().nth(2).unwrap_or_default();
    let child = std::process::Command::new(&helper).arg(&ms).spawn().expect("spawn descendant");
    let mut out = std::io::stdout();
    let _ = out.write_all(format!("descendant={}\n", child.id()).as_bytes());
    let _ = out.flush();
    std::process::exit(0);
}
"##;

    /// Start one descendant **in this image's own process group** and one that
    /// leaves it with `setsid`-equivalent placement, print both pids, exit 0.
    const GROUPER_SOURCE: &str = r##"
use std::io::Write;
use std::os::unix::process::CommandExt;
fn main() {
    let helper = std::env::args().nth(1).unwrap_or_default();
    let ms = std::env::args().nth(2).unwrap_or_default();
    let same = std::process::Command::new(&helper).arg(&ms).spawn().expect("same-group");
    let mut escaped = std::process::Command::new(&helper);
    escaped.arg(&ms).process_group(0);
    let escaped = escaped.spawn().expect("escaped");
    let mut out = std::io::stdout();
    let _ = out.write_all(format!("same={}\nescaped={}\n", same.id(), escaped.id()).as_bytes());
    let _ = out.flush();
    std::process::exit(0);
}
"##;

    /// Ignore `SIGTERM` and never end on its own.
    ///
    /// Written in C because ignoring a signal needs `signal(2)`, which the Rust
    /// standard library does not expose and which this crate will not reach for:
    /// the fixture is a separate program, not part of the crate.
    const STUBBORN_SOURCE: &str = r##"
#include <signal.h>
#include <unistd.h>
int main(void) {
    signal(SIGTERM, SIG_IGN);
    for (;;) {
        pause();
    }
    return 0;
}
"##;

    static C_BUILD_NONCE: AtomicU64 = AtomicU64::new(0);

    /// Compile one C fixture executable, with the same concurrency protocol
    /// `fixture_binary` uses (owner finding `P3R-20`): a content-addressed
    /// name, a unique source and staging pathname per builder, and publication
    /// by `hard_link`, which refuses atomically rather than replacing a winner.
    fn c_fixture_binary(name: &str, source: &str) -> PathBuf {
        let digest = hex(&Sha256::digest(source.as_bytes()));
        let stem = format!("{name}-{}", &digest[..16]);
        let binary = fixture_root().join(&stem);
        if binary.exists() {
            return binary;
        }
        require_tool("cc", "the SIGTERM-resistant fixture needs a C compiler");
        let nonce = C_BUILD_NONCE.fetch_add(1, Ordering::Relaxed);
        let unique = format!("{stem}-{}-{nonce}", std::process::id());
        let source_path = fixture_root().join(format!("{unique}.c"));
        let staged = fixture_root().join(format!("{unique}.staging"));
        fs::write(&source_path, source).expect("write the C fixture source");
        let status = Command::new("cc")
            .args(["-O1", "-o"])
            .arg(&staged)
            .arg(&source_path)
            .status()
            .expect("run cc");
        let _ = fs::remove_file(&source_path);
        assert!(
            status.success(),
            "FIXTURE BUILD FAILURE: cc ran and exited {status} building {name}"
        );
        match fs::hard_link(&staged, &binary) {
            Ok(()) => {
                let _ = fs::remove_file(&staged);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&staged);
            }
            Err(error) => panic!("publishing the C fixture failed: {error}"),
        }
        binary
    }

    fn hex(bytes: &[u8]) -> String {
        let mut text = String::new();
        for byte in bytes {
            text.push_str(&format!("{byte:02x}"));
        }
        text
    }

    fn quiet() -> PathBuf {
        fixture_binary("p4-quiet", QUIET_SOURCE)
    }
    fn writer() -> PathBuf {
        fixture_binary("p4-writer", WRITER_SOURCE)
    }
    fn sleeper() -> PathBuf {
        fixture_binary("p4-sleeper", SLEEPER_SOURCE)
    }
    fn retainer() -> PathBuf {
        fixture_binary("p4-retainer", RETAINER_SOURCE)
    }
    fn grouper() -> PathBuf {
        fixture_binary("p4-grouper", GROUPER_SOURCE)
    }
    fn stubborn() -> PathBuf {
        c_fixture_binary("p4-stubborn", STUBBORN_SOURCE)
    }

    // ----------------------------------------------------------- the harness

    /// One plan, with the bounds and capture sizes a test needs.
    struct Spec<'a> {
        argv: &'a [&'a str],
        timeout_ms: u32,
        grace_ms: u32,
        stdout_capture: u32,
        stderr_capture: u32,
    }

    impl<'a> Spec<'a> {
        fn new(argv: &'a [&'a str]) -> Self {
            Self {
                argv,
                timeout_ms: 30_000,
                grace_ms: 5_000,
                stdout_capture: 4_096,
                stderr_capture: 4_096,
            }
        }
        const fn bounds(mut self, timeout_ms: u32, grace_ms: u32) -> Self {
            self.timeout_ms = timeout_ms;
            self.grace_ms = grace_ms;
            self
        }
        const fn capture(mut self, stdout: u32, stderr: u32) -> Self {
            self.stdout_capture = stdout;
            self.stderr_capture = stderr;
            self
        }
    }

    fn plan_bytes(spec: &Spec<'_>) -> Vec<u8> {
        let arguments: Vec<String> = spec
            .argv
            .iter()
            .map(|argument| {
                let escaped = argument
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n");
                format!("\"{escaped}\"")
            })
            .collect();
        format!(
            concat!(
                r#"{{"schema":"helm-launch-plan","version":"0.1","#,
                r#""execution_kind":"linux_exact_executable","#,
                r#""argv":[{}],"#,
                r#""environment":{{"mode":"empty"}},"#,
                r#""working_directory":{{"capability_id":"workdir"}},"#,
                r#""stdin":{{"mode":"closed_pipe_eof"}},"#,
                r#""stdout":{{"capture_prefix_bytes":{}}},"#,
                r#""stderr":{{"capture_prefix_bytes":{}}},"#,
                r#""timeout_ms":{},"#,
                r#""termination":{{"signal":"SIGTERM","grace_ms":{}}}}}"#
            ),
            arguments.join(","),
            spec.stdout_capture,
            spec.stderr_capture,
            spec.timeout_ms,
            spec.grace_ms
        )
        .into_bytes()
    }

    /// Admit, authorise and launch one fixture as a trusted caller would.
    fn run(executable: &Path, workdir: &Path, spec: &Spec<'_>) -> LaunchOutcome {
        let plan = parse_launch_plan(&plan_bytes(spec)).expect("the fixture plan is valid");
        let exe = admit_executable(open_read_only(executable)).expect("admit the fixture");
        let cwd =
            admit_working_directory("workdir", open_read_only(workdir)).expect("admit the workdir");
        let authorized = authorize(plan, exe, cwd).expect("authorise the fixture");
        launch(authorized).expect("a child was created, so this is Ok")
    }

    /// Is this pid still alive? Used only to clean up test descendants and to
    /// observe whether a sweep reached one; never to make a product claim.
    fn alive(pid: u32) -> bool {
        Path::new(&format!("/proc/{pid}")).exists()
    }

    /// Terminate a test descendant the harness created, whatever happened.
    fn cleanup(pid: u32) {
        if pid == 0 {
            return;
        }
        let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
        for _ in 0..200 {
            if !alive(pid) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn field(text: &str, key: &str) -> u32 {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix(&format!("{key}=")) {
                return rest.trim().parse().unwrap_or(0);
            }
        }
        0
    }

    // ------------------------------------------------- S: the status channel

    #[test]
    fn a_clean_status_eof_is_indeterminate_and_never_exec_success() {
        // S-series. The whole point of the accepted contract: a successful
        // `execveat` closes the close-on-exec status channel and the parent
        // sees an ordinary end-of-file, which is exactly what a child that died
        // before exec would also show. No positive exec claim exists.
        let workdir = scratch_dir("p4-eof");
        let outcome = run(&quiet(), &workdir, &Spec::new(&["quiet", "0"]));
        assert_eq!(
            outcome.receipt().record().exec_status(),
            ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord)
        );
        assert_eq!(
            outcome.receipt().record().child_end(),
            ChildEnd::Exited { code: 0 }
        );
    }

    #[test]
    fn an_object_without_execute_permission_reports_the_exec_stage_failure() {
        // S-series. A structured pre-exec record arrives, is classified, and
        // still produces a receipt rather than an error.
        let workdir = scratch_dir("p4-eacces");
        let copy = workdir.join("not-executable");
        fs::copy(quiet(), &copy).expect("copy the fixture");
        fs::set_permissions(&copy, fs::Permissions::from_mode(0o600)).expect("drop execute bits");
        let outcome = run(&copy, &workdir, &Spec::new(&["quiet", "0"]));
        assert_eq!(
            outcome.receipt().record().exec_status(),
            ExecStatus::PreExecFailure {
                stage: ChildStage::Exec,
                errno: 13,
            }
        );
        assert_eq!(
            outcome.receipt().record().child_end(),
            ChildEnd::Exited { code: 127 },
            "the child's own failure path exits 127"
        );
    }

    // ------------------------------------------------------ R: the child end

    #[test]
    fn a_nonzero_exit_code_is_recorded_as_a_fact_not_a_verdict() {
        // R-series.
        let workdir = scratch_dir("p4-exit");
        let outcome = run(&quiet(), &workdir, &Spec::new(&["quiet", "7"]));
        assert_eq!(
            outcome.receipt().record().child_end(),
            ChildEnd::Exited { code: 7 }
        );
        let termination = outcome.receipt().record().termination();
        assert!(!termination.sigterm_sent());
        assert!(!termination.sigkill_sent());
        assert!(!outcome.receipt().record().run_deadline_expired());
    }

    // ---------------------------------------------------------- O: the streams

    #[test]
    fn both_streams_are_drained_concurrently_and_counted_in_full() {
        // O-series. The fixture interleaves the two streams and flushes after
        // every chunk, and it writes far more than one pipe buffer to each, so
        // a launcher that drained stdout to end-of-file before touching stderr
        // would deadlock here rather than fail an assertion.
        let workdir = scratch_dir("p4-dual");
        let out_bytes = 512 * 1024;
        let err_bytes = 384 * 1024;
        let outcome = run(
            &writer(),
            &workdir,
            &Spec::new(&["writer", "524288", "393216"]).capture(64, 32),
        );
        let record = outcome.receipt().record();
        assert_eq!(record.stream(Stream::Stdout).bytes_drained(), out_bytes);
        assert_eq!(record.stream(Stream::Stderr).bytes_drained(), err_bytes);
        assert_eq!(
            record.stream(Stream::Stdout).completeness(),
            Completeness::CompleteAtEof
        );
        assert_eq!(
            record.stream(Stream::Stderr).completeness(),
            Completeness::CompleteAtEof
        );
        assert_eq!(record.child_end(), ChildEnd::Exited { code: 0 });

        // Every drained byte is hashed, including the bytes the prefix bound
        // discarded: the digest is over the whole stream, not over the capture.
        let expected_out = Digest::of(&vec![b'O'; out_bytes as usize]);
        let expected_err = Digest::of(&vec![b'E'; err_bytes as usize]);
        assert_eq!(record.stream(Stream::Stdout).drained_sha256(), expected_out);
        assert_eq!(record.stream(Stream::Stderr).drained_sha256(), expected_err);

        // The prefixes are bounded, and truncation is reported.
        assert_eq!(outcome.stdout_prefix().len(), 64);
        assert_eq!(outcome.stderr_prefix().len(), 32);
        assert!(outcome.prefix_truncated(Stream::Stdout));
        assert!(outcome.prefix_truncated(Stream::Stderr));
        assert!(outcome.stdout_prefix().iter().all(|byte| *byte == b'O'));
    }

    #[test]
    fn a_stream_shorter_than_its_bound_is_kept_whole_and_not_marked_truncated() {
        // O-series.
        let workdir = scratch_dir("p4-short");
        let outcome = run(
            &writer(),
            &workdir,
            &Spec::new(&["writer", "10", "0"]).capture(4_096, 4_096),
        );
        assert_eq!(outcome.stdout_prefix(), b"OOOOOOOOOO");
        assert!(!outcome.prefix_truncated(Stream::Stdout));
        assert!(!outcome.prefix_truncated(Stream::Stderr));
        assert_eq!(
            outcome
                .receipt()
                .record()
                .stream(Stream::Stdout)
                .bytes_drained(),
            10
        );
        assert_eq!(outcome.stderr_prefix(), b"");
    }

    #[test]
    fn a_zero_capture_bound_still_counts_and_hashes_every_byte() {
        // O-series. The receipt's stream facts are independent of the in-memory
        // capture the caller asked for.
        let workdir = scratch_dir("p4-nocap");
        let outcome = run(
            &writer(),
            &workdir,
            &Spec::new(&["writer", "1000", "0"]).capture(0, 0),
        );
        assert!(outcome.stdout_prefix().is_empty());
        assert!(outcome.prefix_truncated(Stream::Stdout));
        let record = outcome.receipt().record();
        assert_eq!(record.stream(Stream::Stdout).bytes_drained(), 1_000);
        assert_eq!(
            record.stream(Stream::Stdout).drained_sha256(),
            Digest::of(&vec![b'O'; 1_000])
        );
    }

    #[test]
    fn a_writer_that_outlives_the_child_is_bounded_and_named_as_such() {
        // O-series, T5. A descendant the launcher never created keeps the
        // stdout write end open after the direct child exits. The post-exit
        // drain bound must expire, the stream must be named
        // `writer_retained_after_child_exit`, and `launch` must return.
        let workdir = scratch_dir("p4-retained");
        let helper = sleeper();
        let argv = ["retainer", helper.to_str().unwrap(), "20000"];
        let spec = Spec::new(&argv);
        let started = std::time::Instant::now();
        let outcome = run(&retainer(), &workdir, &spec);
        let elapsed = started.elapsed();
        let descendant = field(
            &String::from_utf8_lossy(outcome.stdout_prefix()),
            "descendant",
        );
        cleanup(descendant);

        let record = outcome.receipt().record();
        assert_eq!(record.child_end(), ChildEnd::Exited { code: 0 });
        assert_eq!(
            record.stream(Stream::Stdout).completeness(),
            Completeness::WriterRetainedAfterChildExit,
            "a retained writer must be named, never waited out"
        );
        assert!(
            elapsed < std::time::Duration::from_millis(15_000),
            "the post-exit drain did not bound the wait: {elapsed:?}"
        );
    }

    // ------------------------------------------ T: deadline, grace and kill

    #[test]
    fn a_child_that_ends_before_the_deadline_expires_nothing() {
        // T-series.
        let workdir = scratch_dir("p4-quick");
        let outcome = run(
            &sleeper(),
            &workdir,
            &Spec::new(&["sleeper", "50"]).bounds(10_000, 1_000),
        );
        let record = outcome.receipt().record();
        assert!(!record.run_deadline_expired());
        assert!(!record.termination().sigterm_sent());
        assert!(!record.termination().sigkill_sent());
        assert_eq!(record.child_end(), ChildEnd::Exited { code: 0 });
    }

    #[test]
    fn a_child_that_outlives_the_run_deadline_is_sigtermed_and_then_ends() {
        // T1/T2. The deadline expires, one `SIGTERM` goes through the pidfd,
        // the image has the default disposition, so it ends inside the grace
        // period and no `SIGKILL` is needed.
        let workdir = scratch_dir("p4-deadline");
        let outcome = run(
            &sleeper(),
            &workdir,
            &Spec::new(&["sleeper", "60000"]).bounds(300, 5_000),
        );
        let record = outcome.receipt().record();
        assert!(
            record.run_deadline_expired(),
            "the deadline must be recorded"
        );
        assert!(record.termination().sigterm_sent());
        assert!(
            !record.termination().sigkill_sent(),
            "a child that ends during the grace period needs no SIGKILL"
        );
        assert_eq!(
            record.child_end(),
            ChildEnd::Signaled {
                signal: 15,
                core_dumped: false,
            },
            "the observed end is a fact; it is never a claim about who sent it"
        );
    }

    #[test]
    fn a_child_that_ignores_sigterm_is_sigkilled_after_the_grace_period() {
        // T3. The image ignores `SIGTERM`, so the grace deadline expires and
        // exactly one `SIGKILL` follows through the same pidfd.
        let workdir = scratch_dir("p4-grace");
        let started = std::time::Instant::now();
        let outcome = run(
            &stubborn(),
            &workdir,
            &Spec::new(&["stubborn"]).bounds(300, 400),
        );
        let elapsed = started.elapsed();
        let record = outcome.receipt().record();
        assert!(record.run_deadline_expired());
        assert!(record.termination().sigterm_sent());
        assert!(record.termination().sigkill_sent());
        assert_eq!(
            record.child_end(),
            ChildEnd::Signaled {
                signal: 9,
                core_dumped: false,
            }
        );
        assert!(
            elapsed >= std::time::Duration::from_millis(700),
            "the deadline and the grace period were not actually waited out: {elapsed:?}"
        );
        assert!(
            elapsed < std::time::Duration::from_millis(20_000),
            "the bounded path took {elapsed:?}"
        );
    }

    #[test]
    fn every_launch_returns_inside_the_accepted_total_bound() {
        // T-series, plan section 8.6. The worst case this suite can produce is
        // the ignored-`SIGTERM` path, which pays the deadline, the grace period
        // and the kill wait.
        let workdir = scratch_dir("p4-bound");
        let spec = Spec::new(&["stubborn"]).bounds(200, 200);
        let bound = total_bound_ms(spec.timeout_ms, spec.grace_ms) + GUARD_SLACK_MS;
        let started = std::time::Instant::now();
        let outcome = run(&stubborn(), &workdir, &spec);
        let elapsed = started.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(bound),
            "launch took {elapsed:?}, outside the accepted bound of {bound} ms"
        );
        assert!(outcome.elapsed() <= elapsed);
    }

    // ------------------------------------------------ P: the group sweep

    #[test]
    fn a_launch_records_exactly_one_guarded_group_sweep_disposition() {
        // P-series. Two facts are deterministic and are the ones asserted:
        // the disposition is a member of the accepted vocabulary, and a sweep
        // is never recorded as issued without established authority.
        //
        // **`P3R-21` applies here.** Whether the parent's own
        // `setpgid(child, child)` wins the race against the child's `execveat`
        // is scheduler-dependent on an ordinary launch, so `group_sweep` is
        // legitimately either `issued` or `not_issued_group_not_established`.
        // Requiring one of them would repeat exactly the defect that failed the
        // second P3 publication.
        let workdir = scratch_dir("p4-sweep");
        let outcome = run(&quiet(), &workdir, &Spec::new(&["quiet", "0"]));
        let sweep = outcome.receipt().record().termination().group_sweep();
        assert!(
            matches!(
                sweep,
                GroupSweep::Issued | GroupSweep::NotIssuedGroupNotEstablished
            ),
            "an ordinary completed launch cannot report {sweep:?}"
        );
    }

    #[test]
    fn the_sweep_reaches_a_same_group_descendant_and_never_an_escaped_one() {
        // P-series. Whichever way the authority race went, this test asserts a
        // real fact:
        //
        // * `issued` — the same-group descendant is gone, which is evidence
        //   that the one group signal reached the child's own group. It is
        //   **not** a containment claim.
        // * `not_issued_group_not_established` — no group signal was issued, so
        //   the same-group descendant survives and the harness cleans it up.
        //
        // The escaped descendant must survive in **both** cases, and the
        // harness kills it itself. That is the accepted non-claim: a process
        // that left the group is not reached, and P4 is not containment.
        let workdir = scratch_dir("p4-group");
        let helper = sleeper();
        let argv = ["grouper", helper.to_str().unwrap(), "20000"];
        let outcome = run(&grouper(), &workdir, &Spec::new(&argv).capture(4_096, 0));
        let text = String::from_utf8_lossy(outcome.stdout_prefix()).into_owned();
        let same = field(&text, "same");
        let escaped = field(&text, "escaped");
        assert!(
            same != 0 && escaped != 0,
            "the fixture reported no pids: {text:?}"
        );

        let sweep = outcome.receipt().record().termination().group_sweep();
        // Give the kernel a moment to reap what the sweep signalled.
        let mut same_alive = true;
        for _ in 0..200 {
            same_alive = alive(same);
            if !same_alive || sweep != GroupSweep::Issued {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let escaped_alive = alive(escaped);
        cleanup(same);
        cleanup(escaped);

        match sweep {
            GroupSweep::Issued => assert!(
                !same_alive,
                "the one group signal was issued but the same-group descendant survived"
            ),
            GroupSweep::NotIssuedGroupNotEstablished => assert!(
                same_alive,
                "no group signal was issued, so nothing should have ended the descendant"
            ),
            other => panic!("an ordinary completed launch cannot report {other:?}"),
        }
        assert!(
            escaped_alive,
            "a descendant that left the group must survive: the sweep is cleanup, not containment"
        );
    }

    // ----------------------------------------------------------- the receipt

    #[test]
    fn the_receipt_is_deterministic_bounded_and_recomputable() {
        let workdir = scratch_dir("p4-receipt");
        let outcome = run(&quiet(), &workdir, &Spec::new(&["quiet", "0"]));
        let receipt = outcome.receipt();
        assert_eq!(
            receipt.sha256(),
            Digest::of(receipt.exact_bytes()),
            "the digest must be over exactly the published bytes"
        );
        assert!(receipt.exact_bytes().len() <= crate::MAX_RECEIPT_BYTES);

        // Two launches of the same fixture with the same plan produce byte
        // identical receipts: nothing in the record is a timestamp, a duration,
        // a pid, a descriptor number or a host path.
        let second = run(&quiet(), &workdir, &Spec::new(&["quiet", "0"]));
        assert_eq!(
            receipt.exact_bytes(),
            second.receipt().exact_bytes(),
            "two identical launches produced different receipt bytes"
        );
    }

    #[test]
    fn no_captured_byte_and_no_host_detail_reaches_the_receipt() {
        // The privacy canary. The child prints an identifiable secret; it may
        // appear in the in-memory prefix and must appear nowhere else.
        const CANARY: &str = "HELM-P4-CANARY-b7f3a1d29c";
        let workdir = scratch_dir("p4-canary");
        let source = format!("fn main() {{ println!(\"{CANARY}\"); eprintln!(\"{CANARY}\"); }}\n");
        let canary_fixture = fixture_binary("p4-canary", &source);
        let outcome = run(&canary_fixture, &workdir, &Spec::new(&["canary"]));

        assert!(
            String::from_utf8_lossy(outcome.stdout_prefix()).contains(CANARY),
            "the in-memory prefix is where captured output is allowed to be"
        );
        let bytes = outcome.receipt().exact_bytes();
        let text = String::from_utf8_lossy(bytes).into_owned();
        assert!(
            !text.contains(CANARY),
            "the receipt carries captured output"
        );
        assert!(!text.contains("HELM-P4-CANARY"), "a fragment leaked");
        // Neither debug rendering may print captured bytes.
        assert!(!format!("{outcome:?}").contains("CANARY"));
        assert!(!format!("{:?}", outcome.receipt()).contains("CANARY"));
        // No host path, pid or descriptor number.
        assert!(!text.contains('/'), "a path-like value reached the receipt");
        assert!(
            !text.contains("timestamp") && !text.contains("elapsed") && !text.contains("duration"),
            "a time-valued field reached the receipt"
        );
        for forbidden in ["pid", "pidfd", "signature", "verified", "authentic"] {
            assert!(!text.contains(forbidden), "the receipt names `{forbidden}`");
        }
    }

    #[test]
    fn the_outcome_owns_no_descriptor_and_consumes_its_authorisation() {
        // Every descriptor the launch opened is closed before it returns, so
        // the count of this process's open descriptors is unchanged by a
        // completed launch.
        let workdir = scratch_dir("p4-fds");
        let before = open_descriptor_count();
        let outcome = run(&quiet(), &workdir, &Spec::new(&["quiet", "0"]));
        let after = open_descriptor_count();
        assert_eq!(
            before, after,
            "a launch leaked a descriptor: {before} open before, {after} after"
        );
        let _ = outcome.into_receipt();
        assert_eq!(open_descriptor_count(), before);
    }

    fn open_descriptor_count() -> usize {
        fs::read_dir("/proc/self/fd")
            .map(|entries| entries.count())
            .unwrap_or(0)
    }

    use std::os::unix::fs::PermissionsExt as _;
}
