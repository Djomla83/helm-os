//! Pure lifecycle state machine (plan section 8.5; ADR-0024 sections I and J).
//!
//! Inputs are synthetic events and a caller-supplied monotonic time in
//! milliseconds. Outputs are the actions a later backend would have to perform,
//! and finally the facts a receipt would record. This module reads no clock,
//! polls nothing, signals nothing, waits for nothing, reaps nothing and touches
//! no descriptor. It models policy so the ordering rules can be tested on every
//! platform before any backend exists; in P1 nothing but tests drives it.
//!
//! Rules encoded here, each with a test below:
//!
//! * clean exec-status EOF is never positive exec evidence (S5);
//! * the run deadline starts at exec-status EOF, and an end observed before a
//!   deadline is never a timeout (T4, T5, T33);
//! * deadline, `SIGTERM`, grace, `SIGKILL` (T1–T3, T29);
//! * pre-exec bound with immediate `SIGKILL` (S6, T34);
//! * bounded post-exit drain (O6, T32);
//! * bounded wait after every `SIGKILL`, ending in `end_not_observed` (T40);
//! * a read error is its own fact and never EOF (T41);
//! * no group cleanup without established authority; with it, exactly one,
//!   always before the reap, on every completion path, and none once the child
//!   is observed already reaped (T30, T31);
//! * readiness with hangup asks for a read; only a zero-byte read is EOF (O8).

#![cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "P1 has no backend; the model is exercised by tests only"
    )
)]

use crate::model::{
    ChildEnd, ChildStage, Completeness, ExecStatus, GroupSweep, IndeterminateReason, Stream,
};

/// Pre-exec status bound (frozen; S6).
pub(crate) const SPAWN_CONFIRM_TIMEOUT_MS: u64 = 5_000;
/// Post-exit drain bound (frozen; O6, O7, P4, T5).
pub(crate) const POST_EXIT_DRAIN_MS: u64 = 2_000;
/// Bound on the wait after every `SIGKILL` (T40; owner-approved initial bound).
pub(crate) const POST_KILL_REAP_MS: u64 = 5_000;

/// Whether the launcher's own group placement succeeded. Only that event
/// establishes authority; nothing is ever inferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroupAuthority {
    Established,
    NotEstablished,
}

/// Result of the non-consuming probe run just before a group cleanup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Probe {
    /// The direct child is still unreaped (a zombie or still running).
    Unreaped,
    /// The direct child was reaped elsewhere.
    AlreadyReaped,
}

/// Result of the final non-blocking reap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reap {
    Exited {
        code: i32,
    },
    Killed {
        signal: i32,
    },
    Dumped {
        signal: i32,
    },
    /// Nothing to collect without blocking.
    NothingAvailable,
    /// No classifiable result.
    Unclassifiable,
}

/// Synthetic observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    /// The status channel is readable; `hangup` is informational only.
    StatusReady {
        hangup: bool,
    },
    /// One complete structured record was read.
    StatusRecord {
        stage: ChildStage,
        errno: i32,
    },
    /// A short record followed by EOF, or bytes beyond one record.
    StatusMalformed,
    /// A zero-byte read on the status channel.
    StatusEof,
    StatusReadFailed {
        errno: i32,
    },
    /// A stream is readable; `hangup` is informational only.
    StreamReady {
        stream: Stream,
        hangup: bool,
    },
    /// A zero-byte read on a stream.
    StreamEof(Stream),
    StreamReadFailed {
        stream: Stream,
        errno: i32,
    },
    /// The process descriptor reports that the direct child ended.
    ChildEndReadable,
    /// `now` reached the time returned by [`Lifecycle::next_deadline`].
    DeadlineReached,
    Probed(Probe),
    Reaped(Reap),
}

/// Actions a backend would perform, in the order given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    ReadStatus,
    ReadStream(Stream),
    SendSigterm,
    SendSigkill,
    ProbeReaped,
    SweepGroup,
    Reap,
}

/// Everything a receipt would take from the lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Facts {
    pub(crate) exec_status: ExecStatus,
    pub(crate) child_end: ChildEnd,
    pub(crate) run_deadline_expired: bool,
    pub(crate) sigterm_sent: bool,
    pub(crate) sigkill_sent: bool,
    pub(crate) group_sweep: GroupSweep,
    pub(crate) stdout: Completeness,
    pub(crate) stderr: Completeness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Awaiting {
        record: Option<(ChildStage, i32)>,
        malformed: bool,
    },
    Classified(ExecStatus),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Observing,
    AwaitingProbe,
    AwaitingReap,
    Finished,
}

#[derive(Clone, Debug)]
pub(crate) struct Lifecycle {
    timeout_ms: u64,
    grace_ms: u64,
    authority: GroupAuthority,
    phase: Phase,
    status: Status,
    status_deadline: u64,
    run_deadline: Option<u64>,
    grace_deadline: Option<u64>,
    kill_deadline: Option<u64>,
    drain_deadline: Option<u64>,
    end_observed: bool,
    end_not_observed: bool,
    run_deadline_expired: bool,
    sigterm_sent: bool,
    sigkill_sent: bool,
    group_sweep: Option<GroupSweep>,
    child_end: Option<ChildEnd>,
    stdout: Option<Completeness>,
    stderr: Option<Completeness>,
}

impl Lifecycle {
    /// Start observing a child created at `spawned_at`.
    pub(crate) fn new(
        spawned_at: u64,
        timeout_ms: u32,
        grace_ms: u32,
        authority: GroupAuthority,
    ) -> Self {
        Self {
            timeout_ms: u64::from(timeout_ms),
            grace_ms: u64::from(grace_ms),
            authority,
            phase: Phase::Observing,
            status: Status::Awaiting {
                record: None,
                malformed: false,
            },
            status_deadline: spawned_at.saturating_add(SPAWN_CONFIRM_TIMEOUT_MS),
            run_deadline: None,
            grace_deadline: None,
            kill_deadline: None,
            drain_deadline: None,
            end_observed: false,
            end_not_observed: false,
            run_deadline_expired: false,
            sigterm_sent: false,
            sigkill_sent: false,
            group_sweep: None,
            child_end: None,
            stdout: None,
            stderr: None,
        }
    }

    fn stream_slot(&mut self, stream: Stream) -> &mut Option<Completeness> {
        match stream {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        }
    }

    const fn streams_closed(&self) -> bool {
        self.stdout.is_some() && self.stderr.is_some()
    }

    /// A classified status other than clean EOF: the launcher waits for the end
    /// only until the pre-exec bound. A read failure is not EOF (T41), so it
    /// cannot start the run deadline (T33).
    const fn waits_for_end_within_status_bound(&self) -> bool {
        matches!(
            self.status,
            Status::Classified(
                ExecStatus::PreExecFailure { .. }
                    | ExecStatus::Indeterminate(
                        IndeterminateReason::StatusRecordMalformed
                            | IndeterminateReason::StatusReadFailed { .. }
                    )
            )
        )
    }

    /// When the observer must next be woken with [`Event::DeadlineReached`].
    pub(crate) fn next_deadline(&self) -> Option<u64> {
        if self.phase != Phase::Observing {
            return None;
        }
        let ended = self.end_observed;
        [
            matches!(self.status, Status::Awaiting { .. }).then_some(self.status_deadline),
            (self.waits_for_end_within_status_bound() && !ended && !self.sigkill_sent)
                .then_some(self.status_deadline),
            self.run_deadline
                .filter(|_| !ended && !self.run_deadline_expired),
            self.grace_deadline.filter(|_| !ended && !self.sigkill_sent),
            self.kill_deadline
                .filter(|_| !ended && !self.end_not_observed),
            self.drain_deadline
                .filter(|_| ended && !self.streams_closed()),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    /// The facts, once the lifecycle has finished.
    pub(crate) fn facts(&self) -> Option<Facts> {
        if self.phase != Phase::Finished {
            return None;
        }
        let Status::Classified(exec_status) = self.status else {
            return None;
        };
        Some(Facts {
            exec_status,
            child_end: self.child_end?,
            run_deadline_expired: self.run_deadline_expired,
            sigterm_sent: self.sigterm_sent,
            sigkill_sent: self.sigkill_sent,
            group_sweep: self.group_sweep?,
            stdout: self.stdout?,
            stderr: self.stderr?,
        })
    }

    /// Apply one event observed at monotonic time `now`.
    pub(crate) fn step(&mut self, now: u64, event: Event) -> Vec<Action> {
        let mut actions = Vec::new();
        match self.phase {
            Phase::Observing => {
                self.observe(now, event, &mut actions);
                self.enter_cleanup_if_done(&mut actions);
            }
            Phase::AwaitingProbe => {
                if let Event::Probed(probe) = event {
                    self.after_probe(probe, &mut actions);
                }
            }
            Phase::AwaitingReap => {
                if let Event::Reaped(reap) = event {
                    self.after_reap(reap);
                }
            }
            Phase::Finished => {}
        }
        actions
    }

    fn observe(&mut self, now: u64, event: Event, actions: &mut Vec<Action>) {
        match event {
            Event::StatusReady { hangup: _ } => {
                if matches!(self.status, Status::Awaiting { .. }) {
                    actions.push(Action::ReadStatus);
                }
            }
            Event::StatusRecord { stage, errno } => {
                if let Status::Awaiting { record, malformed } = &mut self.status {
                    if record.is_some() {
                        *malformed = true;
                    } else {
                        *record = Some((stage, errno));
                    }
                }
            }
            Event::StatusMalformed => {
                if let Status::Awaiting { malformed, .. } = &mut self.status {
                    *malformed = true;
                }
            }
            Event::StatusEof => {
                if let Status::Awaiting { record, malformed } = self.status {
                    let status = if malformed {
                        ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed)
                    } else if let Some((stage, errno)) = record {
                        ExecStatus::PreExecFailure { stage, errno }
                    } else {
                        if !self.end_observed {
                            self.run_deadline = Some(now.saturating_add(self.timeout_ms));
                        }
                        // Also what a child that died before exec shows (S5).
                        ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord)
                    };
                    self.status = Status::Classified(status);
                }
            }
            Event::StatusReadFailed { errno } => {
                if matches!(self.status, Status::Awaiting { .. }) {
                    self.status = Status::Classified(ExecStatus::Indeterminate(
                        IndeterminateReason::StatusReadFailed { errno },
                    ));
                }
            }
            Event::StreamReady { stream, hangup: _ } => {
                // Read before hangup: readiness, with or without hangup, only
                // ever asks for a read. EOF is a zero-byte read, nothing else.
                if self.stream_slot(stream).is_none() {
                    actions.push(Action::ReadStream(stream));
                }
            }
            Event::StreamEof(stream) => {
                let slot = self.stream_slot(stream);
                if slot.is_none() {
                    *slot = Some(Completeness::CompleteAtEof);
                }
            }
            Event::StreamReadFailed { stream, errno } => {
                let slot = self.stream_slot(stream);
                if slot.is_none() {
                    *slot = Some(Completeness::ReadFailed { errno });
                }
            }
            Event::ChildEndReadable => {
                if !self.end_observed && !self.end_not_observed {
                    self.end_observed = true;
                    self.drain_deadline = Some(now.saturating_add(POST_EXIT_DRAIN_MS));
                }
            }
            Event::DeadlineReached => self.on_deadline(now, actions),
            Event::Probed(_) | Event::Reaped(_) => {}
        }
    }

    fn send_sigkill(&mut self, now: u64, actions: &mut Vec<Action>) {
        if !self.sigkill_sent {
            self.sigkill_sent = true;
            actions.push(Action::SendSigkill);
            self.kill_deadline = Some(now.saturating_add(POST_KILL_REAP_MS));
        }
    }

    /// Apply the earliest due deadline only, so that an end observation can
    /// always be interleaved before the next one.
    fn on_deadline(&mut self, now: u64, actions: &mut Vec<Action>) {
        let ended = self.end_observed;
        if matches!(self.status, Status::Awaiting { .. }) && now >= self.status_deadline {
            self.status = Status::Classified(ExecStatus::Indeterminate(
                IndeterminateReason::PreExecStatusTimeout,
            ));
            if !ended {
                self.send_sigkill(now, actions);
            }
            return;
        }
        if self.waits_for_end_within_status_bound()
            && !ended
            && !self.sigkill_sent
            && now >= self.status_deadline
        {
            self.send_sigkill(now, actions);
            return;
        }
        if let Some(deadline) = self.run_deadline
            && !ended
            && !self.run_deadline_expired
            && now >= deadline
        {
            self.run_deadline_expired = true;
            self.sigterm_sent = true;
            actions.push(Action::SendSigterm);
            self.grace_deadline = Some(now.saturating_add(self.grace_ms));
            return;
        }
        if let Some(deadline) = self.grace_deadline
            && !ended
            && !self.sigkill_sent
            && now >= deadline
        {
            self.send_sigkill(now, actions);
            return;
        }
        if let Some(deadline) = self.kill_deadline
            && !ended
            && !self.end_not_observed
            && now >= deadline
        {
            self.end_not_observed = true;
            for stream in [Stream::Stdout, Stream::Stderr] {
                let slot = self.stream_slot(stream);
                if slot.is_none() {
                    *slot = Some(Completeness::ReadStoppedChildEndNotObserved);
                }
            }
            return;
        }
        if let Some(deadline) = self.drain_deadline
            && ended
            && now >= deadline
        {
            for stream in [Stream::Stdout, Stream::Stderr] {
                let slot = self.stream_slot(stream);
                if slot.is_none() {
                    *slot = Some(Completeness::WriterRetainedAfterChildExit);
                }
            }
        }
    }

    fn enter_cleanup_if_done(&mut self, actions: &mut Vec<Action>) {
        let status_done = matches!(self.status, Status::Classified(_));
        let observation_done =
            (self.end_observed && self.streams_closed()) || self.end_not_observed;
        if self.phase != Phase::Observing || !status_done || !observation_done {
            return;
        }
        match self.authority {
            GroupAuthority::NotEstablished => {
                self.group_sweep = Some(GroupSweep::NotIssuedGroupNotEstablished);
                actions.push(Action::Reap);
                self.phase = Phase::AwaitingReap;
            }
            GroupAuthority::Established => {
                actions.push(Action::ProbeReaped);
                self.phase = Phase::AwaitingProbe;
            }
        }
    }

    fn after_probe(&mut self, probe: Probe, actions: &mut Vec<Action>) {
        match probe {
            Probe::Unreaped => {
                // The one cleanup call, strictly before the reap.
                self.group_sweep = Some(GroupSweep::Issued);
                actions.push(Action::SweepGroup);
            }
            Probe::AlreadyReaped => {
                self.group_sweep = Some(GroupSweep::NotIssuedChildAlreadyReaped);
                self.child_end = Some(ChildEnd::EndUnobservable);
            }
        }
        actions.push(Action::Reap);
        self.phase = Phase::AwaitingReap;
    }

    fn after_reap(&mut self, reap: Reap) {
        let observed = match reap {
            Reap::Exited { code } => Some(ChildEnd::Exited { code }),
            Reap::Killed { signal } => Some(ChildEnd::Signaled {
                signal,
                core_dumped: false,
            }),
            Reap::Dumped { signal } => Some(ChildEnd::Signaled {
                signal,
                core_dumped: true,
            }),
            Reap::NothingAvailable | Reap::Unclassifiable => None,
        };
        let end = match (self.child_end, observed) {
            // Reaped elsewhere: nothing collected here can be this child's end.
            (Some(ChildEnd::EndUnobservable), _) => ChildEnd::EndUnobservable,
            (_, Some(end)) => end,
            (_, None) if self.end_not_observed => ChildEnd::EndNotObserved,
            (_, None) => ChildEnd::EndUnobservable,
        };
        self.child_end = Some(end);
        self.phase = Phase::Finished;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::{
        Action, Event, Facts, GroupAuthority, Lifecycle, POST_EXIT_DRAIN_MS, POST_KILL_REAP_MS,
        Probe, Reap, SPAWN_CONFIRM_TIMEOUT_MS,
    };
    use crate::model::{
        ChildEnd, ChildStage, Completeness, ExecStatus, GroupSweep, IndeterminateReason, Stream,
    };

    const TIMEOUT: u32 = 30_000;
    const GRACE: u32 = 5_000;
    const EOF_STATUS: ExecStatus =
        ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord);

    /// Drives a lifecycle and keeps every action, for ordering assertions.
    struct Run {
        lc: Lifecycle,
        now: u64,
        actions: Vec<Action>,
    }

    impl Run {
        fn new(authority: GroupAuthority) -> Self {
            Self {
                lc: Lifecycle::new(0, TIMEOUT, GRACE, authority),
                now: 0,
                actions: Vec::new(),
            }
        }
        fn at(&mut self, now: u64, event: Event) -> Vec<Action> {
            assert!(now >= self.now, "time went backwards");
            self.now = now;
            let out = self.lc.step(now, event);
            self.actions.extend(out.iter().copied());
            out
        }
        /// Advance to the next deadline and deliver it.
        fn deadline(&mut self) -> Vec<Action> {
            let at = self.lc.next_deadline().unwrap();
            self.at(at, Event::DeadlineReached)
        }
        fn streams_eof(&mut self, now: u64) {
            self.at(now, Event::StreamEof(Stream::Stdout));
            self.at(now, Event::StreamEof(Stream::Stderr));
        }
        /// Answer cleanup requests the way a correct backend would.
        fn cleanup(&mut self, probe: Probe, reap: Reap) -> Facts {
            let now = self.now;
            if self.actions.contains(&Action::ProbeReaped) {
                self.at(now, Event::Probed(probe));
            }
            assert!(
                self.actions.contains(&Action::Reap),
                "no reap requested: {:?}",
                self.actions
            );
            self.at(now, Event::Reaped(reap));
            self.lc.facts().unwrap()
        }
        fn count(&self, action: Action) -> usize {
            self.actions.iter().filter(|a| **a == action).count()
        }
        fn assert_sweep_discipline(&self, authority: GroupAuthority) {
            let sweeps = self.count(Action::SweepGroup);
            match authority {
                GroupAuthority::NotEstablished => {
                    assert_eq!(sweeps, 0);
                    assert_eq!(self.count(Action::ProbeReaped), 0);
                }
                GroupAuthority::Established => assert!(sweeps <= 1),
            }
            if let Some(sweep) = self.actions.iter().position(|a| *a == Action::SweepGroup) {
                let reap = self
                    .actions
                    .iter()
                    .position(|a| *a == Action::Reap)
                    .unwrap();
                let probe = self
                    .actions
                    .iter()
                    .position(|a| *a == Action::ProbeReaped)
                    .unwrap();
                assert!(probe < sweep && sweep < reap, "order: {:?}", self.actions);
            }
        }
    }

    /// The completion paths of plan section 8.5 Phase C, each driven to cleanup.
    fn drive_path(path: usize, authority: GroupAuthority) -> Run {
        let mut r = Run::new(authority);
        match path {
            // Normal exit.
            0 => {
                r.at(10, Event::StatusEof);
                r.at(20, Event::ChildEndReadable);
                r.streams_eof(20);
            }
            // Pre-exec failure record, then the child's own exit.
            1 => {
                r.at(
                    5,
                    Event::StatusRecord {
                        stage: ChildStage::Exec,
                        errno: 13,
                    },
                );
                r.at(5, Event::StatusEof);
                r.at(6, Event::ChildEndReadable);
                r.streams_eof(6);
            }
            // Indeterminate: malformed record.
            2 => {
                r.at(5, Event::StatusMalformed);
                r.at(5, Event::StatusEof);
                r.at(6, Event::ChildEndReadable);
                r.streams_eof(6);
            }
            // Pre-exec timeout.
            3 => {
                assert_eq!(r.deadline(), vec![Action::SendSigkill]);
                r.at(r.now + 1, Event::ChildEndReadable);
                r.streams_eof(r.now);
            }
            // Run timeout with SIGTERM and SIGKILL.
            4 => {
                r.at(10, Event::StatusEof);
                assert_eq!(r.deadline(), vec![Action::SendSigterm]);
                assert_eq!(r.deadline(), vec![Action::SendSigkill]);
                r.at(r.now + 1, Event::ChildEndReadable);
                r.streams_eof(r.now);
            }
            // End not observed after SIGKILL.
            5 => {
                r.at(10, Event::StatusEof);
                assert_eq!(r.deadline(), vec![Action::SendSigterm]);
                assert_eq!(r.deadline(), vec![Action::SendSigkill]);
                // Kill bound expiry ends observation and requests cleanup.
                let cleanup = r.deadline();
                assert!(cleanup == vec![Action::ProbeReaped] || cleanup == vec![Action::Reap]);
            }
            _ => unreachable!(),
        }
        r
    }

    const PATHS: usize = 6;

    #[test]
    fn s5_status_eof_then_death_is_indeterminate_and_identical_to_a_normal_run() {
        // Death before exec: EOF without record, then a SIGKILL end.
        let mut died = Run::new(GroupAuthority::Established);
        died.at(3, Event::StatusEof);
        died.at(4, Event::ChildEndReadable);
        died.streams_eof(4);
        let died = died.cleanup(Probe::Unreaped, Reap::Killed { signal: 9 });

        let mut normal = Run::new(GroupAuthority::Established);
        normal.at(3, Event::StatusEof);
        normal.at(4, Event::ChildEndReadable);
        normal.streams_eof(4);
        let normal = normal.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });

        assert_eq!(died.exec_status, EOF_STATUS);
        assert_eq!(
            died.exec_status, normal.exec_status,
            "status view must not differ"
        );
        assert_eq!(
            died.child_end,
            ChildEnd::Signaled {
                signal: 9,
                core_dumped: false
            }
        );
        assert!(!died.sigkill_sent, "the launcher sent nothing");
    }

    #[test]
    fn t1_deadline_sends_sigterm_and_the_end_follows() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(100, Event::StatusEof);
        assert_eq!(r.lc.next_deadline(), Some(100 + u64::from(TIMEOUT)));
        assert_eq!(r.deadline(), vec![Action::SendSigterm]);
        assert_eq!(
            r.lc.next_deadline(),
            Some(100 + u64::from(TIMEOUT) + u64::from(GRACE))
        );
        r.at(r.now + 10, Event::ChildEndReadable);
        r.streams_eof(r.now);
        let f = r.cleanup(Probe::Unreaped, Reap::Killed { signal: 15 });
        assert!(f.run_deadline_expired && f.sigterm_sent && !f.sigkill_sent);
        assert_eq!(
            f.child_end,
            ChildEnd::Signaled {
                signal: 15,
                core_dumped: false
            }
        );
    }

    #[test]
    fn t2_grace_expiry_sends_sigkill() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(0, Event::StatusEof);
        r.deadline();
        let term_at = r.now;
        assert_eq!(r.deadline(), vec![Action::SendSigkill]);
        assert_eq!(r.now, term_at + u64::from(GRACE));
        r.at(r.now + 1, Event::ChildEndReadable);
        r.streams_eof(r.now);
        let f = r.cleanup(Probe::Unreaped, Reap::Killed { signal: 9 });
        assert!(f.run_deadline_expired && f.sigterm_sent && f.sigkill_sent);
        // Facts, not causes: a SIGKILL end beside a sent SIGKILL.
        assert_eq!(
            f.child_end,
            ChildEnd::Signaled {
                signal: 9,
                core_dumped: false
            }
        );
    }

    #[test]
    fn t3_exit_during_grace_sends_no_sigkill() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(0, Event::StatusEof);
        r.deadline();
        r.at(r.now + 1, Event::ChildEndReadable);
        assert!(r.lc.next_deadline().unwrap() <= r.now + POST_EXIT_DRAIN_MS);
        r.streams_eof(r.now);
        let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
        assert!(f.run_deadline_expired && f.sigterm_sent && !f.sigkill_sent);
        assert_eq!(r.count(Action::SendSigkill), 0);
    }

    #[test]
    fn t4_t5_an_end_observed_before_the_deadline_is_never_a_timeout() {
        for grace in [0, GRACE] {
            let mut r = Run {
                lc: Lifecycle::new(0, TIMEOUT, grace, GroupAuthority::NotEstablished),
                now: 0,
                actions: Vec::new(),
            };
            r.at(0, Event::StatusEof);
            let deadline = u64::from(TIMEOUT);
            r.at(deadline - 1, Event::ChildEndReadable);
            // The backend wakes at the stale run deadline anyway, with a writer
            // retained (T5), so the drain is still open.
            assert_eq!(r.at(deadline, Event::DeadlineReached), vec![]);
            assert_eq!(r.at(deadline + 1, Event::DeadlineReached), vec![]);
            r.deadline();
            let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
            assert!(!f.run_deadline_expired && !f.sigterm_sent && !f.sigkill_sent);
            assert_eq!(f.stdout, Completeness::WriterRetainedAfterChildExit);
            assert_eq!(r.count(Action::SendSigterm), 0);
        }
    }

    #[test]
    fn an_end_observed_before_status_eof_arms_no_run_deadline() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(1, Event::ChildEndReadable);
        r.at(2, Event::StatusEof);
        r.streams_eof(2);
        let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 3 });
        assert_eq!(f.exec_status, EOF_STATUS);
        assert!(!f.run_deadline_expired);
        assert_eq!(f.child_end, ChildEnd::Exited { code: 3 });
    }

    #[test]
    fn s6_pre_exec_status_timeout_kills_at_once_and_waits_boundedly() {
        let mut r = Run::new(GroupAuthority::Established);
        r.at(10, Event::StatusReady { hangup: false });
        assert_eq!(r.lc.next_deadline(), Some(SPAWN_CONFIRM_TIMEOUT_MS));
        assert_eq!(r.deadline(), vec![Action::SendSigkill]);
        assert_eq!(r.count(Action::SendSigterm), 0, "no grace before exec");
        assert_eq!(
            r.lc.next_deadline(),
            Some(SPAWN_CONFIRM_TIMEOUT_MS + POST_KILL_REAP_MS)
        );
        r.at(r.now + 2, Event::ChildEndReadable);
        r.streams_eof(r.now);
        let f = r.cleanup(Probe::Unreaped, Reap::Killed { signal: 9 });
        assert_eq!(
            f.exec_status,
            ExecStatus::Indeterminate(IndeterminateReason::PreExecStatusTimeout)
        );
        assert!(f.sigkill_sent && !f.run_deadline_expired);
        assert_eq!(f.group_sweep, GroupSweep::Issued);
    }

    #[test]
    fn a_record_without_an_observed_end_is_killed_at_the_pre_exec_bound() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(
            1,
            Event::StatusRecord {
                stage: ChildStage::Chdir,
                errno: 13,
            },
        );
        r.at(1, Event::StatusEof);
        assert_eq!(r.lc.next_deadline(), Some(SPAWN_CONFIRM_TIMEOUT_MS));
        assert_eq!(r.deadline(), vec![Action::SendSigkill]);
        r.at(r.now + 1, Event::ChildEndReadable);
        r.streams_eof(r.now);
        let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 127 });
        // Two facts side by side; no inference between them.
        assert_eq!(
            f.exec_status,
            ExecStatus::PreExecFailure {
                stage: ChildStage::Chdir,
                errno: 13
            }
        );
        assert_eq!(f.child_end, ChildEnd::Exited { code: 127 });
        assert!(!f.run_deadline_expired);
    }

    #[test]
    fn two_records_or_a_short_record_are_malformed_never_a_failure_record() {
        for events in [
            vec![
                Event::StatusRecord {
                    stage: ChildStage::Exec,
                    errno: 2,
                },
                Event::StatusRecord {
                    stage: ChildStage::Exec,
                    errno: 2,
                },
            ],
            vec![Event::StatusMalformed],
            vec![
                Event::StatusRecord {
                    stage: ChildStage::Exec,
                    errno: 2,
                },
                Event::StatusMalformed,
            ],
        ] {
            let mut r = Run::new(GroupAuthority::NotEstablished);
            for e in events {
                r.at(1, e);
            }
            r.at(1, Event::StatusEof);
            r.at(2, Event::ChildEndReadable);
            r.streams_eof(2);
            let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 127 });
            assert_eq!(
                f.exec_status,
                ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed)
            );
        }
    }

    #[test]
    fn o6_drain_expiry_records_a_retained_writer_only_for_open_streams() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(0, Event::StatusEof);
        r.at(50, Event::ChildEndReadable);
        r.at(60, Event::StreamEof(Stream::Stdout));
        assert_eq!(r.lc.next_deadline(), Some(50 + POST_EXIT_DRAIN_MS));
        assert_eq!(r.deadline(), vec![Action::Reap]);
        let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
        assert_eq!(f.stdout, Completeness::CompleteAtEof);
        assert_eq!(f.stderr, Completeness::WriterRetainedAfterChildExit);
        assert!(!f.run_deadline_expired);
    }

    #[test]
    fn t40_no_end_within_the_kill_bound_is_end_not_observed_on_both_kill_paths() {
        // Run path.
        let mut r = Run::new(GroupAuthority::Established);
        r.at(0, Event::StatusEof);
        r.deadline();
        r.deadline();
        let kill_at = r.now;
        assert_eq!(r.lc.next_deadline(), Some(kill_at + POST_KILL_REAP_MS));
        assert_eq!(r.deadline(), vec![Action::ProbeReaped]);
        let f = r.cleanup(Probe::Unreaped, Reap::NothingAvailable);
        assert_eq!(f.child_end, ChildEnd::EndNotObserved);
        assert_eq!(f.stdout, Completeness::ReadStoppedChildEndNotObserved);
        assert_eq!(f.stderr, Completeness::ReadStoppedChildEndNotObserved);
        assert_eq!(
            f.group_sweep,
            GroupSweep::Issued,
            "an unended child still leads its group"
        );

        // Pre-exec path, with one stream already at EOF.
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(1, Event::StreamEof(Stream::Stdout));
        r.deadline();
        assert_eq!(r.deadline(), vec![Action::Reap]);
        let f = r.cleanup(Probe::Unreaped, Reap::NothingAvailable);
        assert_eq!(f.child_end, ChildEnd::EndNotObserved);
        assert_eq!(f.stdout, Completeness::CompleteAtEof);
        assert_eq!(f.stderr, Completeness::ReadStoppedChildEndNotObserved);
        assert_eq!(
            f.exec_status,
            ExecStatus::Indeterminate(IndeterminateReason::PreExecStatusTimeout)
        );
        // No further deadline and no blocking wait exists.
        assert_eq!(r.lc.next_deadline(), None);
    }

    #[test]
    fn t41_read_errors_are_their_own_facts_and_never_eof() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(0, Event::StatusEof);
        r.at(
            1,
            Event::StreamReadFailed {
                stream: Stream::Stdout,
                errno: 5,
            },
        );
        // A later EOF cannot overwrite a recorded read failure.
        r.at(2, Event::StreamEof(Stream::Stdout));
        r.at(3, Event::ChildEndReadable);
        r.at(3, Event::StreamEof(Stream::Stderr));
        let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
        assert_eq!(f.stdout, Completeness::ReadFailed { errno: 5 });
        assert_eq!(f.stderr, Completeness::CompleteAtEof);

        let mut r = Run::new(GroupAuthority::NotEstablished);
        r.at(1, Event::StatusReadFailed { errno: 5 });
        r.at(2, Event::StatusEof);
        // A read failure starts no run deadline; the pre-exec bound applies.
        assert_eq!(r.lc.next_deadline(), Some(SPAWN_CONFIRM_TIMEOUT_MS));
        r.at(3, Event::ChildEndReadable);
        r.streams_eof(3);
        let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
        assert_eq!(
            f.exec_status,
            ExecStatus::Indeterminate(IndeterminateReason::StatusReadFailed { errno: 5 })
        );
    }

    #[test]
    fn pollin_with_pollhup_reads_first_and_hangup_is_never_eof() {
        let mut r = Run::new(GroupAuthority::NotEstablished);
        assert_eq!(
            r.at(
                1,
                Event::StreamReady {
                    stream: Stream::Stderr,
                    hangup: true
                }
            ),
            vec![Action::ReadStream(Stream::Stderr)]
        );
        assert_eq!(
            r.at(1, Event::StatusReady { hangup: true }),
            vec![Action::ReadStatus]
        );
        // Hangup changed nothing: the status is still awaited and the stream is
        // still open, so it is read again.
        assert_eq!(r.lc.next_deadline(), Some(SPAWN_CONFIRM_TIMEOUT_MS));
        assert_eq!(
            r.at(
                2,
                Event::StreamReady {
                    stream: Stream::Stderr,
                    hangup: true
                }
            ),
            vec![Action::ReadStream(Stream::Stderr)]
        );
        r.at(3, Event::StreamEof(Stream::Stderr));
        assert_eq!(
            r.at(
                4,
                Event::StreamReady {
                    stream: Stream::Stderr,
                    hangup: true
                }
            ),
            vec![]
        );
    }

    #[test]
    fn t31_without_group_authority_no_path_issues_a_sweep() {
        for path in 0..PATHS {
            let mut r = drive_path(path, GroupAuthority::NotEstablished);
            let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
            assert_eq!(
                f.group_sweep,
                GroupSweep::NotIssuedGroupNotEstablished,
                "path {path}"
            );
            r.assert_sweep_discipline(GroupAuthority::NotEstablished);
        }
    }

    #[test]
    fn t30_with_group_authority_every_path_issues_exactly_one_sweep_before_the_reap() {
        for path in 0..PATHS {
            let mut r = drive_path(path, GroupAuthority::Established);
            let f = r.cleanup(Probe::Unreaped, Reap::Exited { code: 0 });
            assert_eq!(f.group_sweep, GroupSweep::Issued, "path {path}");
            assert_eq!(r.count(Action::SweepGroup), 1, "path {path}");
            r.assert_sweep_discipline(GroupAuthority::Established);
            // Nothing after completion produces a second sweep.
            assert_eq!(r.at(r.now, Event::Probed(Probe::Unreaped)), vec![]);
            assert_eq!(r.at(r.now + 1, Event::DeadlineReached), vec![]);
            assert_eq!(r.count(Action::SweepGroup), 1);
        }
    }

    #[test]
    fn t31_a_child_already_reaped_elsewhere_gets_no_sweep() {
        for path in 0..PATHS {
            let mut r = drive_path(path, GroupAuthority::Established);
            let f = r.cleanup(Probe::AlreadyReaped, Reap::Exited { code: 0 });
            assert_eq!(
                f.group_sweep,
                GroupSweep::NotIssuedChildAlreadyReaped,
                "path {path}"
            );
            assert_eq!(f.child_end, ChildEnd::EndUnobservable, "path {path}");
            assert_eq!(r.count(Action::SweepGroup), 0);
        }
    }

    #[test]
    fn reap_classification_keeps_exit_and_signal_distinct() {
        for (reap, end) in [
            (Reap::Exited { code: 0 }, ChildEnd::Exited { code: 0 }),
            (Reap::Exited { code: 42 }, ChildEnd::Exited { code: 42 }),
            (
                Reap::Killed { signal: 11 },
                ChildEnd::Signaled {
                    signal: 11,
                    core_dumped: false,
                },
            ),
            (
                Reap::Dumped { signal: 11 },
                ChildEnd::Signaled {
                    signal: 11,
                    core_dumped: true,
                },
            ),
            (Reap::Unclassifiable, ChildEnd::EndUnobservable),
            (Reap::NothingAvailable, ChildEnd::EndUnobservable),
        ] {
            let mut r = drive_path(0, GroupAuthority::NotEstablished);
            assert_eq!(r.cleanup(Probe::Unreaped, reap).child_end, end);
        }
    }

    // --------------------------------------------------- generated sequences

    struct SplitMix(u64);
    impl SplitMix {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    fn random_event(rng: &mut SplitMix) -> Event {
        let stream = if rng.below(2) == 0 {
            Stream::Stdout
        } else {
            Stream::Stderr
        };
        match rng.below(12) {
            0 => Event::StatusReady {
                hangup: rng.below(2) == 0,
            },
            1 => Event::StatusRecord {
                stage: ChildStage::Exec,
                errno: 8,
            },
            2 => Event::StatusMalformed,
            3 | 4 => Event::StatusEof,
            5 => Event::StatusReadFailed { errno: 5 },
            6 => Event::StreamReady {
                stream,
                hangup: rng.below(2) == 0,
            },
            7 | 8 => Event::StreamEof(stream),
            9 => Event::StreamReadFailed { stream, errno: 5 },
            10 => Event::ChildEndReadable,
            _ => Event::Probed(Probe::Unreaped),
        }
    }

    /// Arbitrary event orders from a backend that answers cleanup requests
    /// correctly always finish, within the total bound, with at most one of each
    /// signal and at most one sweep, never before the probe and never after the
    /// reap, and never with an exec-success reading of clean EOF.
    #[test]
    fn generated_event_orders_keep_every_invariant_and_the_total_bound() {
        let mut rng = SplitMix(0x11fe_c7c1_e000_0003);
        for _ in 0..20_000 {
            let timeout = 1 + u32::try_from(rng.below(40_000)).unwrap();
            let grace = u32::try_from(rng.below(6_000)).unwrap();
            let authority = if rng.below(2) == 0 {
                GroupAuthority::Established
            } else {
                GroupAuthority::NotEstablished
            };
            let mut r = Run {
                lc: Lifecycle::new(0, timeout, grace, authority),
                now: 0,
                actions: Vec::new(),
            };
            let bound = SPAWN_CONFIRM_TIMEOUT_MS
                + u64::from(timeout)
                + u64::from(grace)
                + POST_KILL_REAP_MS
                + POST_EXIT_DRAIN_MS;
            let mut steps = 0;
            while r.lc.facts().is_none() {
                steps += 1;
                assert!(steps < 10_000, "lifecycle did not finish");
                let pending = r.actions.last().copied();
                if pending == Some(Action::ProbeReaped)
                    && r.lc.next_deadline().is_none()
                    && r.count(Action::Reap) == 0
                {
                    let probe = if rng.below(4) == 0 {
                        Probe::AlreadyReaped
                    } else {
                        Probe::Unreaped
                    };
                    r.at(r.now, Event::Probed(probe));
                    continue;
                }
                if r.actions.contains(&Action::Reap) {
                    let reap = match rng.below(4) {
                        0 => Reap::Exited { code: 1 },
                        1 => Reap::Killed { signal: 9 },
                        2 => Reap::NothingAvailable,
                        _ => Reap::Dumped { signal: 6 },
                    };
                    r.at(r.now, Event::Reaped(reap));
                    continue;
                }
                match r.lc.next_deadline() {
                    Some(deadline) if rng.below(3) == 0 || deadline <= r.now => {
                        r.at(deadline.max(r.now), Event::DeadlineReached);
                    }
                    Some(deadline) => {
                        let at = r.now + rng.below(deadline - r.now + 1);
                        let event = random_event(&mut rng);
                        r.at(at, event);
                    }
                    None => {
                        let event = random_event(&mut rng);
                        r.at(r.now, event);
                    }
                }
            }
            let f = r.lc.facts().unwrap();
            assert!(r.now <= bound, "finished at {} beyond bound {bound}", r.now);
            assert!(r.count(Action::SendSigterm) <= 1);
            assert!(r.count(Action::SendSigkill) <= 1);
            assert_eq!(r.count(Action::Reap), 1);
            r.assert_sweep_discipline(authority);
            assert_eq!(f.sigterm_sent, r.count(Action::SendSigterm) == 1);
            assert_eq!(f.sigkill_sent, r.count(Action::SendSigkill) == 1);
            assert_eq!(
                f.group_sweep == GroupSweep::Issued,
                r.count(Action::SweepGroup) == 1
            );
            if f.run_deadline_expired {
                assert!(f.sigterm_sent);
            }
            if matches!(f.child_end, ChildEnd::EndNotObserved) {
                assert!(f.sigkill_sent, "end_not_observed only follows a SIGKILL");
            }
        }
    }
}
