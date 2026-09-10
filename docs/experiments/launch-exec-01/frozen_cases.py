"""LAUNCH-EXEC-01 frozen case manifest: membership, classes, schedules, expectations.

Machine source of truth. The prose table in
``docs/experiments/LAUNCH-EXEC-01-DEFINITION.md`` is derived from this file and
must be checked against it; where the two disagree, this file is authoritative
for membership and counts and the prose is the defect.

This file declares expectations ONLY. It never inspects spike output, and the
spike never reads it. Expected digests are derived from the byte recipes here by
``oracles.py`` using Python ``hashlib``, independently of the spike.

Authority: proposed ADR-0024, the corrected
``docs/research/HELM-LAUNCH-ARCHITECTURE.md``, and the owner decisions recorded
in ``DECISIONS`` below. NOT_RUN: no trial has been executed.
"""

# --------------------------------------------------------------- owner inputs
# Frozen before the first trial. None of these may be selected after any result
# is known; D-1 in particular is a frozen input, not a scoring choice.
DECISIONS = {
    "D-1": "arm_i_scoped_unsafe_backend",     # exact FD isolation is claimed and gated
    "D-2": "one_linux_substrate_before_wine",
    "D-3": "zero_helm_crate_dependencies",
    "D-4": "direct_child_lifecycle_only",
    "D-5": "mandatory_working_directory_capability",
    "D-6": "utf8_only_argv",
    "D-8": "no_timestamp_no_duration_in_receipt",
    "D-9": "refuse_set_id_objects_at_admission",
    "D-10": "empty_environment_only",
    "D-11": "no_new_privs_required_before_exec",
}

REVIEWED_ARCHITECTURE_TIP = "a5f09f8df8c3868b7ec2f3f6f9d973058c164a44"
REVIEW_TIP = "55f0f78c18e7bc9b980e1b99169018a7ea5566b4"
AUTHORITATIVE_MAIN = "5cc56384257a2ec1f2a2a9c64f7f4328da6cef2e"

# ------------------------------------------------------------------- constants
SPAWN_CONFIRM_TIMEOUT_MS = 5000
POST_EXIT_DRAIN_MS = 2000
MAX_CAPTURE_BYTES = 64 * 1024
MAX_ARG_BYTES = 4 * 1024
EXEC_RACE_DELAY_MS = 200          # S4 forced schedule
T3_GRACE_EXIT_DELAY_MS = 500
T4_DELTA_MS = 500
P_DESCENDANT_LIFETIME_MS = 20000
REPEAT_TRIALS = 200               # O8 and S7

REPORT_SENTINEL = b"HELM-LAUNCH-EXEC-01-REPORT-BEGIN\n"

# Frozen child-setup stage vocabulary. "The correct stage" is unverifiable
# without a closed set, and every post-fork operation must map to one of these.
STAGES = [
    "RELOCATE",
    "DUP2",
    "CLEAR_CLOEXEC",
    "CHDIR",
    "CLOSE_RANGE",
    "SETPGID",
    "SIGMASK",
    "SIGACTION",
    "NO_NEW_PRIVS",
    "EXEC",
]

# Exactly the syscalls the post-fork child may issue, enforced by case M1.
# Anything else in that window is a FAIL, which is what makes the architecture's
# "no allocation, no locking, no formatting, no panic path" falsifiable.
CHILD_PERMITTED_SYSCALLS = [
    "dup2", "dup3", "fcntl", "fchdir", "close_range", "setpgid",
    "rt_sigprocmask", "rt_sigaction", "prctl", "write", "execveat", "exit_group",
]
CHILD_FORBIDDEN_SYSCALLS = [
    "brk", "mmap", "munmap", "mprotect", "futex", "openat", "open",
    "set_robust_list", "getrandom", "rt_sigreturn", "clone", "clone3",
]

# Syscalls that appear in the child ONLY under a declared test-only fault
# injection, never in the candidate mechanism sequence. Separating these is what
# stops M1's minimality claim from being tautological: without the split, the
# claim would hold only because the injecting cases happen not to be traced,
# which is an accident of classification rather than a rule.
CHILD_TEST_INJECTION_SYSCALLS = {
    "clock_nanosleep": "--stall-pre-exec-ms, case S6 only",
    "nanosleep": "--stall-pre-exec-ms, case S6 only",
    "kill": "--die-before-exec, case S5 only",
    "getpid": "--die-before-exec, case S5 only",
}

# The spike modes that enable those injections. M1, M2 and M4 trace the
# PRODUCTION configuration, in which none of these flags is passed; a trace
# containing an injection syscall in a case that declared no injection is a FAIL.
CHILD_INJECTION_MODES = ["--stall-pre-exec-ms", "--die-before-exec",
                         "--skip-no-new-privs", "--bypass-admission",
                         "--exec-fd-no-cloexec", "--exec-fd-o-path",
                         "--post-fork-delay-ms"]

# PARENT-side test/control arms. Deliberately a SEPARATE list from the child
# injection modes above, because they are a different kind of thing: they change
# the shape of the LAUNCHER, not the syscall sequence of the child. That is what
# lets M2 carry --extra-threads and still trace a production child window; if it
# were listed above, M1's minimality rule would refuse the trace and the case
# could never be posed. Neither mode is part of the candidate mechanism, and
# neither runs unless its flag is passed.
PARENT_CONTROL_MODES = {
    "--extra-threads": "M2 only: >=3 extra live threads in the launcher, one "
                       "allocating continuously and one with a pthread_atfork "
                       "handler registered",
    "--rejected-acquisition-arm": "M5 only: the REJECTED fork+pidfd_open "
                                  "acquisition, run before the mechanism with "
                                  "its own fork, child and reap. Not a "
                                  "candidate mechanism",
}

# --------------------------------------------------- M3-T: the bounded claim
# Owner amendment M3-T, decided BEFORE the first valid trial. M3 becomes a
# traced case so its expectation rests on the external syscall record rather
# than on the launcher describing its own behaviour.
#
# The claim is deliberately narrow. This experiment cannot and does not test the
# kernel, so M3 says nothing about pidfds in general.
M3_BOUNDED_CLAIM = (
    "For the direct child in this candidate execution, the pidfd used by the "
    "launcher was returned through the CLONE_PIDFD facility of the SAME clone3 "
    "syscall that created that direct child, rather than being acquired later "
    "by pidfd_open() from a numeric PID."
)

# What M3 must NOT be read as establishing. Recorded so the boundary is part of
# the frozen manifest and not merely of the prose around it.
M3_CLAIM_EXCLUSIONS = (
    "Linux pidfds are universally race-free",
    "the kernel implementation has been proved atomic",
    "any general claim about pidfd behaviour beyond this one execution",
    "any timing-sensitive normal-launch behaviour: M3 runs under a tracer, and "
    "its evidence is restricted to the acquisition facts below",
)

# The facts the normalised trace must establish. All of them, or the case is
# INVALID -- partial evidence never yields the success token.
#
# Amended after the bounded review's findings R-2 and R-3. The CLOSURE form is
# gone: inferring the descriptor's origin from the absence of pidfd_open rested
# on frozen-source invariants the trace never observes, so E now requires the
# tracer to render the descriptor the kernel wrote back. G now requires the
# reaped process to BE the direct child, not merely some process.
M3_EVIDENCE_FACTS = {
    "A": "exactly one relevant candidate clone3 direct-child creation",
    "B": "CLONE_PIDFD present in its flags",
    "C": "clone_args.pidfd output location present",
    "D": "clone3 succeeds and returns the DIRECT_CHILD pid",
    "E": "the tracer directly renders the pidfd produced by that clone3 call",
    "F": "that pidfd equals the pidfd used by the launcher's direct-child "
         "lifecycle observation",
    "G": "waitid(P_PIDFD, that_pidfd, ...) reports si_pid == the clone3 return",
    "H": "no pidfd_open(DIRECT_CHILD_PID) acquisition path is observed",
}

# ------------------------------------------------- R-4: the tracer requirement
# A usable strace is a MANDATORY PREFLIGHT requirement, not a per-case
# condition. The definition previously promised a parent-side ptrace fallback
# that was never built, and preflight only raised no_tracer when ptrace_scope
# was restrictive -- so a host with no strace and a permissive scope left every
# traced case INVALID with no honest path. For 0.1 the strace-based tracer is
# the SOLE supported external syscall-record mechanism for the eight traced
# cases, and its absence stops the trial before the first case rather than
# degrading eight results.
#
# This is NOT clone3_unavailable and must never become M3's conditional cause:
# it is an environment failure that prevents a valid trial from starting at all.
STRACE_MIN_VERSION = (5, 4)
TRACER_REQUIREMENT = (
    "A usable strace >= %d.%d is a mandatory pretrial environment requirement. "
    "If strace is absent, older than the floor, or cannot render the trace "
    "contract the traced cases depend on, the runner HALTS before the first "
    "preregistered case. No ptrace, eBPF, helper-binary or root-requiring "
    "fallback exists or may be added for 0.1."
    % STRACE_MIN_VERSION)

# ------------------------------------------------------------- output recipe
# byte[i] = (i * 251 + tag) mod 256. A volume is not a recipe; oracles.py
# computes every expected count and digest from this and the declared volume
# alone, and never reads spike output.
STREAM_TAG = {"stdout": 1, "stderr": 2}


def stream_byte(index, tag):
    return (index * 251 + tag) % 256


# ------------------------------------------------------------------- classes
MANDATORY = "mandatory"
CONDITIONAL = "conditional"
RECORDED = "recorded"

# Reasons a conditional case may be BLOCKED. A conditional case BLOCKED with a
# recorded cause is expected and does not make the run inconclusive; a mandatory
# case BLOCKED does.
BLOCK_REASONS = {
    "euid_zero": "geteuid() == 0 makes an unprivileged DAC refusal unposable",
    "no_tracer": "no strace and no permitted ptrace tracer on the runner",
    "no_noexec_mount": "no unprivileged-writable noexec mount exists",
    "unprivileged_runner": "no privileged identity exists to attempt a real "
                           "privilege transition, and none is manufactured",
    "parent_no_new_privs_set": "the launcher's own parent process already has "
                               "no_new_privs set. The bit is inherited across "
                               "fork and execve and CANNOT be cleared, so the "
                               "N2 control arm could never observe 0. This is a "
                               "host property, never a mechanism defect",
    "clone3_unavailable": "clone3 is rejected by the environment (ENOSYS, or "
                          "EPERM/EACCES from a seccomp policy) even though the "
                          "kernel version floor is met. Syscall availability is "
                          "not established by a version number",
}


def case(name, series, cls, traced=False, predict=None, safe=None, gates=(),
         blocked_if=None, instant_reject=False, note=""):
    if (predict is None) == (safe is None):
        raise ValueError(f"{name}: exactly one of predict/safe is required")
    if cls == CONDITIONAL and blocked_if is None:
        raise ValueError(f"{name}: a conditional case must name its block reason")
    if cls != CONDITIONAL and blocked_if is not None:
        raise ValueError(f"{name}: only a conditional case may name a block reason")
    if cls == RECORDED and not gates:
        raise ValueError(f"{name}: a recorded case must carry a gated sub-assertion")
    if blocked_if is not None and blocked_if not in BLOCK_REASONS:
        raise ValueError(f"{name}: unknown block reason {blocked_if!r}")
    return dict(case=name, series=series, cls=cls, traced=traced,
                predict=predict, safe=safe, gates=list(gates),
                blocked_if=blocked_if, instant_reject=instant_reject, note=note)


# =============================================================== the case table
CASES = [
    # ---- E: executable identity and TOCTOU --------------------------------
    case("E1", "E", MANDATORY, traced=True, predict="Exited:0",
         note="normal pinned executable; marker matches; measured digest equals "
              "the independent hashlib digest taken before the run"),
    case("E2", "E", MANDATORY, predict="Exited:0", instant_reject=True,
         note="rename(2) of helper_alt OVER the pathname after the pin; the "
              "pinned body runs and the marker is helper_report"),
    case("E3", "E", MANDATORY, predict="Exited:0",
         note="original pathname renamed then unlinked after the pin"),
    case("E4", "E", MANDATORY, predict="Exited:0", instant_reject=True,
         note="the pinned leaf was a symlink whose target is retargeted after "
              "the pin; the substituted body never executes"),
    case("E5", "E", MANDATORY, predict="ExecFailed:ETXTBSY",
         note="the HARNESS process holds a second O_WRONLY descriptor on the "
              "same inode across launch; mandatory (st_dev, st_ino) oracle "
              "proves it is the same inode, never pathname equality"),
    case("E5b", "E", MANDATORY, predict="Exited:0",
         note="writer opens O_WRONLY, writes and CLOSES before launch; no "
              "ETXTBSY, because the write-deny reference is taken at exec time"),
    case("E6", "E", MANDATORY, predict="Exited:0", instant_reject=True,
         note="length-preserving ELF-valid mutation, writer closed before exec: "
              "the MUTATED marker runs and the recorded digest is pre-mutation. "
              "Documentation gate: PASSes only if the field is named "
              "pre_exec_body_sha256 and no text claims the measured bytes ran"),
    case("E6b", "E", MANDATORY, instant_reject=True,
         safe=["Exited:mutated", "ExecFailed:ENOEXEC", "Signaled:SIGBUS",
               "Signaled:SIGKILL"],
         note="ftruncate + rewrite to a different length. Every member is "
              "honest: a torn image fails the loader's header checks or faults "
              "beyond i_size. Any outcome PRESENTED as 'the measured body ran' "
              "is a FAIL. Documentation gate"),
    case("E6c", "E", RECORDED,
         safe=["ExecFailed:ETXTBSY", "Exited:mutated"],
         gates=["no_claim_that_measured_bytes_ran"],
         note="shared writable mapping survives the fd close. The one genuinely "
              "unresolved kernel question: whether a surviving i_mmap_writable "
              "mapping leaves i_writecount at zero could not be settled from "
              "primary sources, so this is recorded and carved out of the gate"),
    case("E6d", "E", CONDITIONAL, blocked_if="euid_zero",
         predict="ExecFailed:EACCES",
         note="fchmod to 0 between admission and exec; establishes that "
              "pre_exec_mode_bits is likewise a pre-execution measurement"),
    case("E7", "E", MANDATORY, traced=True, predict="Exited:0",
         note="the pinned object is DYNAMICALLY linked; the trace shows the "
              "kernel resolving PT_INTERP and ld.so resolving DT_NEEDED BY "
              "NAME. Without this no case executes a dynamic object at all. "
              "Also establishes that an empty environment does not pin the "
              "loaded-code closure"),
    case("E8", "E", MANDATORY, predict="refused:ElfNotInCohort",
         instant_reject=True,
         note="foreign-architecture ELF (EM_AARCH64); execveat must never be "
              "reached. The runner's binfmt_misc registrations are recorded "
              "before the first trial"),

    # ---- A: argv ----------------------------------------------------------
    case("A1", "A", MANDATORY, predict="argv_exact",
         note="argument containing spaces arrives as ONE byte-identical element"),
    case("A2", "A", MANDATORY, predict="argv_exact",
         note="shell metacharacters and a newline arrive as one literal "
              "element; no shell, no expansion"),
    case("A3", "A", MANDATORY, predict="argv_exact",
         note="empty argument arrives as one element of length 0"),
    case("A4", "A", MANDATORY, predict="argv_exact",
         note="argument of exactly MAX_ARG_BYTES arrives byte-identical"),
    case("A6", "A", MANDATORY, predict="argv_exact",
         note="argv[0] differs from any pathname and is observed exactly"),

    # ---- V: environment (D-10: empty only) --------------------------------
    case("V1", "V", MANDATORY, predict="environ_empty",
         note="host sets HELM_LEAK_CANARY, LD_LIBRARY_PATH, PATH and HOME "
              "before launch; the child's environ is EXACTLY empty. Under D-10 "
              "this is the whole of the environment contract the mechanism can "
              "establish, and any inherited name is a FAIL"),

    # ---- F: descriptor inheritance ----------------------------------------
    case("F1", "F", MANDATORY, predict="fds_exactly_012",
         note="baseline; the set is verified IN THE EXECUTED IMAGE, and the "
              "pre-exec set {0,1,2,exec_fd,status_w} is not a violation"),
    case("F2", "F", MANDATORY, predict="fds_exactly_012", instant_reject=True,
         note="an unrelated NON-CLOEXEC descriptor is open in the parent at "
              "exec time. Under frozen D-1 arm (i) the child not seeing it is "
              "the ONLY pass; there is no documented-failure acceptance path"),
    case("F3", "F", MANDATORY, predict="fds_exactly_012",
         note="a CLOEXEC descriptor is open before launch"),
    case("F4", "F", MANDATORY, traced=True, predict="fds_exactly_012",
         note="neither the exec descriptor nor the exec-status pipe survives"),
    case("F5", "F", MANDATORY, predict="signals_reset",
         note="harness blocks {SIGTERM, SIGUSR1} and sets SIGPIPE and SIGUSR2 "
              "to SIG_IGN. The child's SigBlk must be empty and SigIgn must "
              "contain nothing the launcher did not set. The helper reports "
              "SigBlk/SigIgn/SigCgt separately so blocked, ignored and "
              "caught/default are distinguishable"),
    case("F6", "F", MANDATORY, predict="fds_exactly_012",
         note="harness closes 0, 1 and 2 before launch, so the launcher's own "
              "pipes and exec fd can land on 0-2. A child stdio descriptor with "
              "FD_CLOEXEC set (the dup2(fd,fd) no-op) is a FAIL"),
    case("F7", "F", MANDATORY, traced=True, predict="fds_exactly_012",
         note="exec fd and status write end at ADJACENT numbers; no close_range "
              "call may return EINVAL from an inverted gap"),

    # ---- X: exec failure and admission ------------------------------------
    case("X1", "X", CONDITIONAL, blocked_if="euid_zero",
         predict="ExecFailed:EACCES",
         note="execute bits cleared by fchmod AFTER admission recorded the "
              "mode; the recorded bits are the pre-fchmod value"),
    case("X2", "X", MANDATORY, predict="refused:ElfNotInCohort",
         note="#! script refused at admission; execveat never reached"),
    case("X2b", "X", MANDATORY, predict="ExecFailed:ENOENT",
         note="same script, admission bypassed in a frozen spike mode, exec fd "
              "O_CLOEXEC as mandated. ENOENT per execveat(2) BUGS, NOT ENOEXEC: "
              "the interpreter is never invoked and no procfs dependency is hit"),
    case("X2c", "X", MANDATORY, predict="interpreter_ran_with_devfd",
         note="same script without O_CLOEXEC: the interpreter runs and receives "
              "/dev/fd/N. Recording X2b without X2c would attribute a CLOEXEC "
              "artefact to scripts as a class"),
    case("X3", "X", MANDATORY, predict="ExecFailed:EACCES",
         note="a file that has never had an execute bit; distinct from X1 only "
              "in that no post-admission change occurred"),
    case("X4", "X", MANDATORY, predict="ExecFailed:ENOEXEC",
         note="`unloadable_in_cohort.elf`: a 64-byte ELF64 header that PASSES "
              "the cohort rule -- ELFCLASS64, ELFDATA2LSB, EM_X86_64, ET_EXEC "
              "-- but has e_phnum = 0, so it is admitted and reaches execveat "
              "with nothing for the loader to map. The earlier 4-byte "
              "magic-only fixture could not pose this case at all: once "
              "admission became a 64-byte HEADER check rather than a four-byte "
              "magic check, a 4-byte file was refused as ElfNotInCohort and "
              "could never reach execveat to produce ENOEXEC"),
    case("X5", "X", MANDATORY, predict="refused:NotRegularFile",
         note="capability on a directory descriptor"),
    case("X6", "X", MANDATORY, predict="refused:DescriptorModeUnsuitable",
         note="capability on an O_PATH descriptor, obtained with the frozen "
              "spike mode --exec-fd-o-path. A HELM admission rule, NOT a kernel "
              "limitation: execveat(2) accepts O_PATH descriptors, and the "
              "refusal exists because the body cannot be measured through one. "
              "The mode and the DescriptorModeUnsuitable refusal both have to "
              "exist in the spike for this case to be posable at all"),
    case("X7", "X", MANDATORY, predict="refused:SetIdBitsPresent",
         instant_reject=True,
         note="D-9: a set-user-ID object is refused at admission, "
              "deterministically. No privileged fixture and no cross-UID "
              "elevation is manufactured; the fixture is chmod u+s on a file "
              "the running user already owns"),
    case("X8", "X", CONDITIONAL, blocked_if="no_noexec_mount",
         predict="ExecFailed:EACCES",
         note="executable on a noexec mount; no mount is created and no sudo "
              "is used"),

    # ---- O: output --------------------------------------------------------
    case("O1", "O", MANDATORY, predict="stream_exact",
         note="stdout only, 4 KiB, mode measure; count and digest match the "
              "frozen-recipe oracle; completeness CompleteAtEof"),
    case("O2", "O", MANDATORY, predict="stream_exact",
         note="stderr only, 4 KiB; correct stream; streams never merged"),
    case("O3", "O", MANDATORY, predict="stream_exact", instant_reject=True,
         note="both streams emit 8 MiB concurrently, far past pipe capacity; "
              "both CompleteAtEof; completes under 10000 ms with "
              "timeout_ms=60000. A TimedOut outcome is a FAIL"),
    case("O4", "O", MANDATORY, predict="stream_exact",
         note="output exceeds MAX_CAPTURE_BYTES; digest over ALL drained bytes, "
              "retained prefix exactly at the bound, and the in-memory "
              "truncated flag must NOT appear in the receipt"),
    case("O5", "O", MANDATORY, predict="stream_exact",
         note="child closes stdout early and keeps writing stderr. The poll() "
              "return count must be at most 10000 and CPU under 200 ms, "
              "because a spinning launcher also satisfies 'no hang'"),
    case("O6", "O", MANDATORY, predict="Exited:0", instant_reject=True,
         note="descendant inherits fds 1 and 2 and sleeps 30 s holding them "
              "while the direct child exits 0 immediately. launch() must return "
              "within the total bound and measurably before the sleep ends, "
              "with completeness WriterRetainedAfterChildExit. A non-return, a "
              "bare TimedOut, or CompleteAtEof is a FAIL"),
    case("O7", "O", MANDATORY, predict="Exited:42",
         note="exit status AND a stderr capture failure are simultaneously "
              "true; the receipt must carry both. A receipt that can report "
              "only one of the two facts is a FAIL"),
    case("O8", "O", MANDATORY, predict="stream_exact",
         note=f"POLLIN and POLLHUP in the same poll() return, {REPEAT_TRIALS} "
              "trials; any trial short of 512 bytes is a FAIL"),

    # ---- R: exit and signal -----------------------------------------------
    case("R1", "R", MANDATORY, predict="Exited:0"),
    case("R2", "R", MANDATORY, predict="Exited:42"),
    case("R3", "R", MANDATORY, predict="Signaled:SIGSEGV",
         note="self-raised; the helper sets RLIMIT_CORE=0 first so no core dump "
              "is produced (systemd-coredump is installed on the runner)"),
    case("R4", "R", RECORDED,
         safe=["ExitStatusUnobservable", "Exited:42"],
         gates=["never_reports_exited_zero", "never_hangs"],
         note="host sets SIGCHLD to SIG_IGN. Recorded because the outcome "
              "depends on a caller precondition the crate cannot enforce; "
              "gated because reporting a status it did not observe is never "
              "acceptable"),

    # ---- T: timeout -------------------------------------------------------
    case("T1", "T", MANDATORY, predict="TimedOut"),
    case("T2", "T", MANDATORY, predict="TimedOut:KilledByLauncher:SIGKILL",
         note="helper ignores SIGTERM and the grace window elapses"),
    case("T3", "T", MANDATORY, predict="TimedOut:ExitedDuringGrace:9",
         note=f"frozen schedule: handler sleeps {T3_GRACE_EXIT_DELAY_MS} ms "
              "then _exits 9, timeout_ms=2000, grace_ms=5000"),
    case("T4", "T", MANDATORY, predict="Exited:9",
         note=f"frozen schedule: sleeps timeout_ms-{T4_DELTA_MS} then _exits 9. "
              "With the pidfd polled the exit is OBSERVED against the deadline, "
              "so TimedOut{ExitedDuringGrace} is now a FAIL, not a safe set"),
    case("T5", "T", MANDATORY, predict="Exited:7",
         note="child exits 7 just before the deadline while a descendant "
              "retains fds 1 and 2, so the exit is invisible without polling "
              "the pidfd. Any TimedOut disposition is a FAIL"),
    case("T6", "T", MANDATORY, predict="TimedOut:KilledByLauncher:SIGKILL",
         note="harness parent has SIGTERM BLOCKED and SIGPIPE ignored; the "
              "wall-clock shape must match T2 from a default-signal parent, "
              "which it can only do if the child's signal state was reset"),

    # ---- N: no_new_privs (D-11) -------------------------------------------
    case("N1", "N", MANDATORY, predict="no_new_privs_1", instant_reject=True,
         note="D-11: the executed helper observes NoNewPrivs: 1 in "
              "/proc/self/status. This is the directly observed state"),
    case("N2", "N", CONDITIONAL, blocked_if="parent_no_new_privs_set",
         predict="no_new_privs_0", instant_reject=True,
         note="CONTROL: a frozen spike mode skips the prctl, and the helper "
              "must then observe NoNewPrivs: 0. Without this arm a positive N1 "
              "would be uninformative, because the runner could already have "
              "set no_new_privs process-wide. CONDITIONAL because that same "
              "possibility makes the control unposable: no_new_privs is "
              "inherited and CANNOT be cleared, so a parent that already has it "
              "set forces the child to observe 1. Preflight records the "
              "parent's own NoNewPrivs before any case; if it is 1, N2 is "
              "BLOCKED as a host property and N1 is then recorded as "
              "UNCONTROLLED in the report"),
    case("N3", "N", CONDITIONAL, blocked_if="unprivileged_runner",
         predict="privilege_transition_suppressed",
         note="the actual suppression of a set-user-ID or file-capability "
              "privilege TRANSITION. Expected BLOCKED on an unprivileged "
              "runner, and deliberately so: no privileged fixture is created "
              "to manufacture cross-UID elevation. The report must record this "
              "as an UNTESTED privileged transition resting on primary-source "
              "kernel semantics, never as a demonstrated one"),

    # ---- P: process-tree negative controls --------------------------------
    case("P1", "P", RECORDED, safe=["descendant_survived", "descendant_died"],
         gates=["launch_returns_within_total_bound", "completeness_reported"],
         note="descendant outlives its parent. Its first three actions after "
              "fork are to close 0/1/2 and reopen them on /dev/null, so a "
              "lifecycle control can never hang a mandatory case; O6 and P4 are "
              "the deliberate exceptions that retain the pipes"),
    case("P2", "P", RECORDED, safe=["descendant_survived", "descendant_died"],
         gates=["launch_returns_within_total_bound", "completeness_reported"],
         note="same, with setsid; establishes that a group sweep is best-effort"),
    case("P3", "P", RECORDED, safe=["sweep_issued", "sweep_not_issued"],
         gates=["sweep_strictly_before_reap"],
         note="the sweep syscall, if issued, must appear strictly BEFORE "
              "waitid. After the reap the pgid may have been reused and the "
              "launcher would be signalling processes it never created"),
    case("P4", "P", RECORDED, safe=["descendant_survived", "descendant_died"],
         gates=["launch_returns_within_total_bound",
                "writer_retained_after_child_exit_reported"],
         note="setsid descendant RETAINING fds 1 and 2; establishes that a "
              "group sweep does not release a pipe and that the bounded drain, "
              "not the sweep, is what makes the launcher terminate"),

    # ---- S: spawn/exec confirmation ---------------------------------------
    case("S1", "S", MANDATORY, predict="Exited:0",
         note="clean EOF with no record AND the helper report received on fd 1 "
              "AND the direct child exited normally. The report is the exec "
              "evidence; clean EOF alone is not a PASS"),
    case("S2", "S", CONDITIONAL, blocked_if="euid_zero",
         predict="ExecFailed:CHDIR:EACCES",
         note="the directory capability's own descriptor is fchmod'ed to 0000 "
              "after admission, so the child's fchdir fails. No helper report "
              "may be received: its absence is the independent proof that the "
              "image never ran"),
    case("S3", "S", MANDATORY, predict="Exited:127",
         note="the helper itself exits 127 after a SUCCESSFUL exec and must not "
              "be confused with exec failure"),
    case("S4", "S", MANDATORY, predict="Exited:7", instant_reject=True,
         note=f"rapid exec-and-exit: the parent sleeps {EXEC_RACE_DELAY_MS} ms "
              "after clone3 returns and before it polls, so the child has "
              "certainly execed and exited first. A forced schedule, not a "
              "timing hope"),
    case("S5", "S", MANDATORY, predict="ExecStatusIndeterminate",
         instant_reject=True,
         note="child killed between the last setup stage and execveat. The "
              "parent observes clean EOF with no record, BYTE-IDENTICAL to S1, "
              "and no helper report. Reporting exec success is a FAIL and a "
              "FAIL of falsifier 8. The discriminating case for the whole "
              "confirmation design"),
    case("S6", "S", MANDATORY, predict="ExecStatusIndeterminate:PreExecTimeout",
         note="pre-exec stall past SPAWN_CONFIRM_TIMEOUT_MS. The launcher must "
              "terminate and reap: a leaked child or zombie is a FAIL"),
    case("S7", "S", CONDITIONAL, blocked_if="euid_zero",
         predict="ExecFailed:CHDIR:EACCES", instant_reject=True,
         note=f"S2's construction {REPEAT_TRIALS} times, so the record and the "
              "hangup arrive in the same poll() return. Any trial reporting "
              "exec success, Exited:127 or ExecStatusIndeterminate is a FAIL"),

    # ---- M: mechanism minimality and parent shape -------------------------
    case("M1", "M", CONDITIONAL, blocked_if="no_tracer", traced=True,
         predict="child_syscalls_within_frozen_set",
         note="the traced child window from the clone3 return to execveat. This "
              "is the evidence that REPLACES the architecture's assertion of "
              "'no allocation, no locking, no formatting, no panic path'"),
    case("M2", "M", CONDITIONAL, blocked_if="no_tracer", traced=True,
         predict="identical_to_single_threaded_arm",
         note="parent has >=3 extra live threads, one with an allocation in "
              "flight and one with a registered pthread_atfork handler. Without "
              "this the mechanism is evidenced only for a single-threaded "
              "parent, which no real HELM caller is"),
    case("M3", "M", CONDITIONAL, blocked_if="clone3_unavailable", traced=True,
         predict="pidfd_acquired_atomically",
         note="clone3(CLONE_PIDFD) is the PRIMARY acquisition: it removes the "
              "three pidfd_open caller preconditions a library cannot "
              "establish, and avoids the pthread_atfork surface glibc's fork() "
              "opens. CONDITIONAL because a kernel version floor establishes "
              "that the syscall EXISTS, not that it is PERMITTED: a seccomp "
              "policy can reject clone3 on a supporting kernel. Preflight "
              "probes availability without creating a child; an unavailable "
              "clone3 disables the whole mechanism and is a preflight gate, so "
              "every case is then BLOCKED rather than this one case FAILing. "
              "TRACED under owner amendment M3-T, decided before the first "
              "valid trial: the acquisition is established from the EXTERNAL "
              "syscall record, never from a launcher receipt field naming its "
              "own acquisition mode, because a self-assertion is exactly what "
              "this case exists to avoid. See M3_BOUNDED_CLAIM and "
              "M3_EVIDENCE_FACTS"),
    case("M4", "M", CONDITIONAL, blocked_if="no_tracer", traced=True,
         predict="sequence_matches_frozen_stages",
         note="the implemented child sequence is matched syscall-for-syscall "
              "against the frozen STAGES order, including that CHDIR precedes "
              "CLOSE_RANGE and NO_NEW_PRIVS precedes EXEC. A spike whose "
              "sequence differs is INVALID, not PASS"),
    case("M5", "M", RECORDED,
         safe=["pidfd_open_esrch", "waitid_echild", "pidfd_open_succeeded"],
         gates=["never_reports_unobserved_exit_status"],
         note="the REJECTED fork+pidfd_open acquisition under SIGCHLD=SIG_IGN, "
              "recorded to evidence why clone3(CLONE_PIDFD) is primary rather "
              "than asserting it. Not a candidate mechanism"),
]

# ============================================================ derived membership
MEMBERSHIP = [c["case"] for c in CASES]
BY_NAME = {c["case"]: c for c in CASES}

MANDATORY_CASES = [c["case"] for c in CASES if c["cls"] == MANDATORY]
CONDITIONAL_CASES = [c["case"] for c in CASES if c["cls"] == CONDITIONAL]
RECORDED_CASES = [c["case"] for c in CASES if c["cls"] == RECORDED]

INSTANT_REJECT = [c["case"] for c in CASES if c["instant_reject"]]
TRACED_CASES = [c["case"] for c in CASES if c["traced"]]

# Cases whose PASS additionally requires that no document presents the
# pre-execution measurement as the identity of the executed body.
DOCUMENTATION_GATES = ["E6", "E6b", "E6d"]

# Plan-parse properties this experiment deliberately does NOT pose, because they
# are properties of a crate that does not exist and must not be created before
# the trial. Recorded so the deletion cannot later read as coverage.
OUT_OF_SCOPE = {
    "argv_nul_rejection": "was A5; a NUL byte cannot be delivered through a C "
                          "spike's argv, and the rejection is a plan-parse "
                          "property of helm-launch",
    "ld_preload_refusal": "was V3; a plan-parse property, and under D-10 there "
                          "is no explicit environment to refuse a name in",
    "explicit_env_entries": "was V2; D-10 removes explicit environment from the "
                            "0.1 contract entirely",
    "path_accepted_as_value": "was V4; no environment values exist under D-10",
    "gconv_path_accepted": "was V5; no environment values exist under D-10. The "
                           "LD_ denylist's incompleteness is now moot for 0.1 "
                           "and is recorded as an in-crate obligation only if "
                           "explicit environment is ever reintroduced",
}


def validate():
    """Structural invariants of the manifest itself. Raises on any violation."""
    errors = []
    if len(MEMBERSHIP) != len(set(MEMBERSHIP)):
        seen, dupes = set(), set()
        for n in MEMBERSHIP:
            if n in seen:
                dupes.add(n)
            seen.add(n)
        errors.append(f"duplicate case ids: {sorted(dupes)}")

    classes = {MANDATORY: MANDATORY_CASES, CONDITIONAL: CONDITIONAL_CASES,
               RECORDED: RECORDED_CASES}
    union = []
    for names in classes.values():
        union.extend(names)
    if sorted(union) != sorted(MEMBERSHIP):
        errors.append("the three classes are not a partition of the membership")
    for a in classes:
        for b in classes:
            if a < b and set(classes[a]) & set(classes[b]):
                errors.append(f"classes {a} and {b} overlap")

    for c in CASES:
        if c["cls"] == RECORDED and c["predict"] is not None:
            errors.append(f"{c['case']}: a recorded case must not have a single "
                          "prediction; use a safe set")
        if c["cls"] != RECORDED and c["gates"]:
            errors.append(f"{c['case']}: only a recorded case carries gates")
        if c["safe"] is not None and len(c["safe"]) < 2:
            errors.append(f"{c['case']}: a safe set needs at least two members")
        if c["safe"] is not None and len(c["safe"]) != len(set(c["safe"])):
            errors.append(f"{c['case']}: duplicate member in the safe set")

    for name in INSTANT_REJECT + TRACED_CASES + DOCUMENTATION_GATES:
        if name not in BY_NAME:
            errors.append(f"unknown case referenced: {name}")

    # Tracing authority: ptrace delivers a tracee's exit and signal
    # notifications to the tracer before the real parent, and the launcher IS
    # the real parent, so no case measuring waitid classification may be traced.
    untraceable_series = {"R", "T", "O", "P", "S"}
    for c in CASES:
        if c["traced"] and c["series"] in untraceable_series:
            errors.append(f"{c['case']}: series {c['series']} must not be traced")

    if errors:
        raise ValueError("frozen_cases.py is inconsistent:\n  " +
                         "\n  ".join(errors))
    return True


def summary():
    validate()
    series = {}
    for c in CASES:
        series[c["series"]] = series.get(c["series"], 0) + 1
    return {
        "total": len(MEMBERSHIP),
        "mandatory": len(MANDATORY_CASES),
        "conditional": len(CONDITIONAL_CASES),
        "recorded": len(RECORDED_CASES),
        "instant_reject": sorted(INSTANT_REJECT),
        "traced": sorted(TRACED_CASES),
        "series": dict(sorted(series.items())),
        "out_of_scope": len(OUT_OF_SCOPE),
        "decisions": DECISIONS,
    }


if __name__ == "__main__":
    # P-16. This module has no imports at all, so it cannot reach a host fact
    # and summary() is pure frozen data. It still leaves through the one
    # boundary, so the invariant is "no module emits JSON of its own" with no
    # exceptions to remember.
    import evidence

    evidence.publish(summary())
