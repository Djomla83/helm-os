//! P3 backend tests: the ABI records, the closed child window under a tracer,
//! and Linux integration against purpose-built fixtures.
//!
//! These are **unit** tests rather than files in `tests/`, because the P3
//! surface is crate-private on purpose: `launch_minimal`, `ChildPlan`,
//! `ChildHandle` and the backend errors are not public and must not become
//! public. `tests/p3_boundary.rs` inspects source and manifests instead, and
//! proves the public absence from outside.
//!
//! # These tests create and execute processes
//!
//! That is the newly authorised P3 capability, and validating it is ordinary
//! product testing. It is **not** LAUNCH-EXEC-01, Trial #3, Trial #4 or D-7
//! activity: no frozen asset is used, no `launcher_spike` is built or run, no
//! trial journal exists, and nothing here produces a verdict about any
//! application. Every fixture is newly written here and compiled by `rustc` at
//! test time.
//!
//! # The X2c rule
//!
//! A fixture that reports its own state is **producer self-tested first**: the
//! same binary is run directly through `std::process::Command`, its report is
//! parsed, and its marker is checked to name the fixture the test intends.
//! Only then is the same object admitted and launched through the backend.
//! That is the Trial #3 X2c lesson, and [`report_fixture_reports_without_the_backend`]
//! is the test that keeps it.
//!
//! # Host requirements
//!
//! `rustc` for the fixtures, `strace` for the trace tests, and `cc` for the one
//! `pthread_atfork` host-condition helper. A missing tool is a **test
//! environment failure** with an explicit message, never a silent skip.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "test code: a wrong assumption must fail loudly, and the product lints that forbid these belong to the child window"
)]

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use sha2::{Digest as _, Sha256};

use super::{
    BackendError, ChildPlan, CloseSpan, Fault, INJECTION_COMPILED, MinimalLaunch,
    POST_KILL_REAP_MS, SPAWN_CONFIRM_TIMEOUT_MS, STAGE_ORDER, STATUS_RECORD_BYTES, classify_record,
    launch_minimal, stage, stage_of, syscall,
};
use crate::authority::{admit_executable, admit_working_directory, authorize};
use crate::model::{ChildEnd, ChildStage, ExecStatus, IndeterminateReason};
use crate::plan::parse_launch_plan;

// ===========================================================================
// 1. ABI and record structure
// ===========================================================================

#[test]
fn the_child_plan_is_plain_old_data_the_child_can_copy() {
    // The compile-time proofs live beside the type; this states them as a test
    // so a reader of the suite sees them too.
    assert!(!core::mem::needs_drop::<ChildPlan>());
    assert!(!core::mem::needs_drop::<CloseSpan>());
    assert!(!core::mem::needs_drop::<Fault>());
    fn copyable<T: Copy>() {}
    copyable::<ChildPlan>();
    copyable::<CloseSpan>();
    copyable::<Fault>();
}

#[test]
fn the_clone_args_record_matches_the_kernel_uapi() {
    use syscall::CloneArgs;
    assert_eq!(core::mem::size_of::<CloneArgs>(), 88);
    assert_eq!(
        core::mem::size_of::<CloneArgs>(),
        syscall::CLONE_ARGS_SIZE_VER2
    );
    assert_eq!(core::mem::align_of::<CloneArgs>(), 8);
    assert_eq!(syscall::CLONE_ARGS_SIZE_VER0, 64);
    let offsets = [
        core::mem::offset_of!(CloneArgs, flags),
        core::mem::offset_of!(CloneArgs, pidfd),
        core::mem::offset_of!(CloneArgs, child_tid),
        core::mem::offset_of!(CloneArgs, parent_tid),
        core::mem::offset_of!(CloneArgs, exit_signal),
        core::mem::offset_of!(CloneArgs, stack),
        core::mem::offset_of!(CloneArgs, stack_size),
        core::mem::offset_of!(CloneArgs, tls),
        core::mem::offset_of!(CloneArgs, set_tid),
        core::mem::offset_of!(CloneArgs, set_tid_size),
        core::mem::offset_of!(CloneArgs, cgroup),
    ];
    assert_eq!(offsets, [0, 8, 16, 24, 32, 40, 48, 56, 64, 72, 80]);

    // The only record this crate builds: `CLONE_PIDFD` alone, `SIGCHLD` as the
    // exit signal, the pidfd output address, everything else zero.
    let args = CloneArgs::for_direct_child(0x1234);
    assert_eq!(args.flags, syscall::CLONE_PIDFD);
    assert_eq!(args.flags & syscall::CLONE_VM, 0);
    assert_eq!(args.flags & syscall::CLONE_FILES, 0);
    assert_eq!(args.flags & syscall::CLONE_VFORK, 0);
    assert_eq!(args.flags & syscall::CLONE_THREAD, 0);
    assert_eq!(args.flags.count_ones(), 1);
    assert_eq!(args.exit_signal, syscall::SIGCHLD);
    assert_eq!(args.pidfd, 0x1234);
    assert_eq!(
        (
            args.child_tid,
            args.parent_tid,
            args.stack,
            args.stack_size,
            args.tls,
            args.set_tid,
            args.set_tid_size,
            args.cgroup
        ),
        (0, 0, 0, 0, 0, 0, 0, 0)
    );
}

#[test]
fn the_kernel_sigaction_record_is_the_raw_layout_not_the_glibc_one() {
    use syscall::KernelSigaction;
    // Handler, flags, restorer, mask — the mask **last**. glibc's userspace
    // `struct sigaction` puts the mask second, which is why the raw syscall
    // must never be handed a glibc value.
    assert_eq!(core::mem::size_of::<KernelSigaction>(), 32);
    assert_eq!(
        core::mem::size_of::<KernelSigaction>(),
        syscall::KERNEL_SIGACTION_BYTES
    );
    assert_eq!(core::mem::offset_of!(KernelSigaction, handler), 0);
    assert_eq!(core::mem::offset_of!(KernelSigaction, flags), 8);
    assert_eq!(core::mem::offset_of!(KernelSigaction, restorer), 16);
    assert_eq!(core::mem::offset_of!(KernelSigaction, mask), 24);

    let action = KernelSigaction::DEFAULT_DISPOSITION;
    assert_eq!(action.handler, 0, "SIG_DFL is the null handler");
    assert_eq!(action.flags, 0);
    assert_eq!(action.restorer, 0);
    assert_eq!(action.mask, 0);
    assert_eq!(syscall::KERNEL_SIGSET_BYTES, 8);
}

#[test]
fn the_blocked_mask_covers_every_signal_including_the_two_glibc_keeps_to_itself() {
    // The structural half of the signal-blocking claim (T21). The trace half is
    // in the tracer tests; neither is sufficient alone, because strace prints
    // symbolic names and glibc's internal signals have none.
    let full = syscall::FULL_KERNEL_SIGSET;
    for signal in 1..=64_u32 {
        let bit = 1_u64 << (signal - 1);
        assert_ne!(full & bit, 0, "signal {signal} is not in the blocked set");
    }
    // `SIGCANCEL` is `__SIGRTMIN` (32) and `SIGSETXID` is `__SIGRTMIN + 1`
    // (33); `sigfillset` omits them and `pthread_sigmask` strips them.
    assert_ne!(
        full & (1_u64 << 31),
        0,
        "SIGCANCEL is not in the blocked set"
    );
    assert_ne!(
        full & (1_u64 << 32),
        0,
        "SIGSETXID is not in the blocked set"
    );
    assert_eq!(full, u64::MAX);
    assert_eq!(syscall::EMPTY_KERNEL_SIGSET, 0);

    // `SIGKILL` and `SIGSTOP` are never claimed blockable: the child's
    // disposition reset skips them, and the kernel removes them from any mask
    // it installs.
    assert_eq!(syscall::SIGKILL, 9);
    assert_eq!(syscall::SIGSTOP, 19);
}

#[test]
fn the_stage_vocabulary_is_the_accepted_sequence() {
    assert_eq!(STAGE_ORDER, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    let names: Vec<&str> = STAGE_ORDER
        .iter()
        .map(|code| stage_of(*code).unwrap().as_str())
        .collect();
    assert_eq!(
        names,
        [
            "dup2",
            "clear_cloexec",
            "chdir",
            "close_range",
            "setpgid",
            "sigaction",
            "sigmask",
            "no_new_privs",
            "exec"
        ]
    );
    assert_eq!(stage_of(stage::NONE), None);
    assert_eq!(stage_of(10), None);
    assert_eq!(stage_of(255), None);
}

#[test]
fn the_failure_record_is_eight_bytes_and_a_clean_end_of_file_is_never_success() {
    assert_eq!(STATUS_RECORD_BYTES, 8);

    // Nothing written, then end-of-file: every ordinary run, and S5.
    assert_eq!(
        classify_record([0; 9], 0),
        ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord)
    );

    // A well-formed record for each stage.
    for code in STAGE_ORDER {
        let mut buffer = [0_u8; 9];
        buffer[0] = code;
        buffer[4..8].copy_from_slice(&13_i32.to_le_bytes());
        assert_eq!(
            classify_record(buffer, 8),
            ExecStatus::PreExecFailure {
                stage: stage_of(code).unwrap(),
                errno: 13,
            }
        );
    }

    // Short, over-long, an unknown stage byte and a non-zero reserved pad are
    // all malformed rather than guessed at.
    let malformed = ExecStatus::Indeterminate(IndeterminateReason::StatusRecordMalformed);
    for filled in [1_usize, 2, 3, 4, 5, 6, 7, 9] {
        let mut buffer = [0_u8; 9];
        buffer[0] = stage::EXEC;
        assert_eq!(
            classify_record(buffer, filled),
            malformed,
            "filled {filled}"
        );
    }
    let mut unknown_stage = [0_u8; 9];
    unknown_stage[0] = 200;
    assert_eq!(classify_record(unknown_stage, 8), malformed);
    let mut dirty_pad = [0_u8; 9];
    dirty_pad[0] = stage::EXEC;
    dirty_pad[2] = 1;
    assert_eq!(classify_record(dirty_pad, 8), malformed);

    // No input at all produces an exec-success reading, because none exists.
    for filled in 0..=9_usize {
        for first in [0_u8, 1, 9, 200] {
            let mut buffer = [0_u8; 9];
            buffer[0] = first;
            let status = classify_record(buffer, filled);
            assert!(matches!(
                status,
                ExecStatus::PreExecFailure { .. } | ExecStatus::Indeterminate(_)
            ));
        }
    }
}

#[test]
fn the_fixed_internal_bounds_are_the_authorised_two() {
    assert_eq!(SPAWN_CONFIRM_TIMEOUT_MS, 5_000);
    assert_eq!(POST_KILL_REAP_MS, 5_000);
}

#[test]
fn no_release_build_compiles_the_fault_injection() {
    // A compile-time proof: in a release build the injection is not compiled,
    // whatever the feature flags say.
    const {
        assert!(
            cfg!(debug_assertions) || !INJECTION_COMPILED,
            "a release build compiled the test-only fault injection"
        );
    }
    assert_eq!(
        INJECTION_COMPILED,
        cfg!(all(feature = "test-fault-injection", debug_assertions))
    );
    // Outside an injection build the request record carries nothing at all.
    if !INJECTION_COMPILED {
        assert_eq!(core::mem::size_of::<Fault>(), 0);
    }
}

// ===========================================================================
// 2. Test support: fixtures, plans, draining
// ===========================================================================

/// A host tool the suite genuinely needs. A missing one fails loudly.
fn require_tool(tool: &str, why: &str) {
    let found = Command::new(tool).arg("--version").output();
    match found {
        Ok(output) if output.status.success() => {}
        _ => panic!(
            "TEST ENVIRONMENT FAILURE: `{tool}` is required for the helm-launch P3 suite ({why}), \
             and it is not usable here. This is not a launcher result; install the tool and rerun."
        ),
    }
}

fn fixture_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join("helm-launch-p3-fixtures");
        fs::create_dir_all(&root).expect("fixture root");
        root
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

fn unhex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0, "odd hex run: {text}");
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("hex"))
        .collect()
}

/// Compile one fixture source with `rustc`, cached by the digest of its text.
///
/// The fixture is an ordinary dynamically linked `ET_DYN` object — the same
/// shape a real caller would admit, which also exercises the point that an
/// empty environment still resolves an interpreter and libraries by name from
/// host state (E7).
fn fixture_binary(name: &str, source: &str) -> PathBuf {
    let digest = hex(&Sha256::digest(source.as_bytes()));
    let stem = format!("{name}-{}", &digest[..16]);
    let binary = fixture_root().join(&stem);
    if binary.exists() {
        // Content-addressed, so a warm cache needs no compiler at all. That is
        // also what keeps `rustc` out of a traced run.
        return binary;
    }
    require_tool("rustc", "fixtures are compiled at test time, never shipped");
    let source_path = fixture_root().join(format!("{stem}.rs"));
    fs::write(&source_path, source).expect("write fixture source");
    let staged = fixture_root().join(format!("{stem}.{}.staging", std::process::id()));
    let status = Command::new("rustc")
        .args(["--edition", "2021", "-C", "debuginfo=0", "-o"])
        .arg(&staged)
        .arg(&source_path)
        .status()
        .expect("run rustc");
    assert!(
        status.success(),
        "TEST ENVIRONMENT FAILURE: rustc could not build fixture {name}"
    );
    fs::rename(&staged, &binary).expect("stage fixture");
    let mode = fs::metadata(&binary)
        .expect("fixture metadata")
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o111,
        0o111,
        "fixture {name} is not executable: {mode:o}"
    );
    binary
}

/// The source of the reporting fixture, with its marker substituted in.
///
/// Everything it reports is read through `std` only. It writes one versioned,
/// line-oriented report to stdout and one marker line to stderr.
const REPORT_FIXTURE_SOURCE: &str = r##"
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;

const NAME: &str = "__FIXTURE_NAME__";

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        text.push_str(&format!("{:02x}", byte));
    }
    text
}

fn status_field(field: &str) -> String {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix(field) {
            if let Some(rest) = rest.strip_prefix(':') {
                return rest.trim().to_string();
            }
        }
    }
    String::new()
}

fn main() {
    let mut out = String::new();
    out.push_str("HELM-LAUNCH-P3-FIXTURE/1\n");
    out.push_str(&format!("name={}\n", NAME));

    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    out.push_str(&format!("argv_count={}\n", args.len()));
    for (index, argument) in args.iter().enumerate() {
        out.push_str(&format!("argv={}:{}\n", index, hex(argument.as_bytes())));
    }

    let mut environment: Vec<(std::ffi::OsString, std::ffi::OsString)> =
        std::env::vars_os().collect();
    environment.sort();
    out.push_str(&format!("env_count={}\n", environment.len()));
    for (key, value) in &environment {
        out.push_str(&format!("env={}:{}\n", hex(key.as_bytes()), hex(value.as_bytes())));
    }

    match std::fs::metadata(".") {
        Ok(metadata) => {
            out.push_str(&format!("cwd_dev={}\n", metadata.dev()));
            out.push_str(&format!("cwd_ino={}\n", metadata.ino()));
        }
        Err(error) => out.push_str(&format!("cwd_error={}\n", error)),
    }

    // Open descriptors, excluding the handle the scan itself holds.
    let mut descriptors: Vec<(u32, String)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let number: u32 = match name.to_string_lossy().parse() {
                Ok(number) => number,
                Err(_) => continue,
            };
            let target = std::fs::read_link(entry.path())
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default();
            if target.starts_with("/proc/") && target.ends_with("/fd") {
                continue;
            }
            descriptors.push((number, target));
        }
    }
    descriptors.sort();
    out.push_str(&format!("fd_count={}\n", descriptors.len()));
    for (number, target) in &descriptors {
        out.push_str(&format!("fd={}:{}\n", number, hex(target.as_bytes())));
    }

    out.push_str(&format!("no_new_privs={}\n", status_field("NoNewPrivs")));
    out.push_str(&format!("sig_blk={}\n", status_field("SigBlk")));
    out.push_str(&format!("sig_ign={}\n", status_field("SigIgn")));
    out.push_str(&format!("sig_cgt={}\n", status_field("SigCgt")));
    out.push_str(&format!("pgid_is_self={}\n", status_field("Tgid")));
    out.push_str("end\n");

    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(out.as_bytes());
    let _ = stdout.flush();
    let mut stderr = std::io::stderr();
    let _ = stderr.write_all(format!("HELM-LAUNCH-P3-FIXTURE-STDERR/1 name={}\n", NAME).as_bytes());
    let _ = stderr.flush();
}
"##;

const REPORT_FIXTURE_NAME: &str = "helm-launch-p3-report";

fn report_fixture() -> PathBuf {
    fixture_binary(
        "report",
        &REPORT_FIXTURE_SOURCE.replace("__FIXTURE_NAME__", REPORT_FIXTURE_NAME),
    )
}

/// A parsed fixture report.
#[derive(Debug, Default)]
struct Report {
    name: String,
    argv: Vec<Vec<u8>>,
    environment: Vec<(Vec<u8>, Vec<u8>)>,
    cwd_dev: u64,
    cwd_ino: u64,
    descriptors: BTreeMap<u32, String>,
    no_new_privs: String,
    tgid: u64,
    complete: bool,
}

fn parse_report(bytes: &[u8]) -> Report {
    let text = String::from_utf8(bytes.to_vec()).expect("the report is ASCII by construction");
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("HELM-LAUNCH-P3-FIXTURE/1"),
        "the report does not carry the expected versioned marker: {text}"
    );
    let mut report = Report::default();
    let mut argv: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    for line in lines {
        if line == "end" {
            report.complete = true;
            continue;
        }
        let (key, value) = line.split_once('=').unwrap_or((line, ""));
        match key {
            "name" => report.name = value.to_owned(),
            "argv" => {
                let (index, encoded) = value.split_once(':').expect("argv entry");
                argv.insert(index.parse().expect("argv index"), unhex(encoded));
            }
            "env" => {
                let (key, value) = value.split_once(':').expect("env entry");
                report.environment.push((unhex(key), unhex(value)));
            }
            "cwd_dev" => report.cwd_dev = value.parse().expect("cwd dev"),
            "cwd_ino" => report.cwd_ino = value.parse().expect("cwd ino"),
            "fd" => {
                let (number, target) = value.split_once(':').expect("fd entry");
                report.descriptors.insert(
                    number.parse().expect("fd number"),
                    String::from_utf8(unhex(target)).unwrap_or_default(),
                );
            }
            "no_new_privs" => report.no_new_privs = value.to_owned(),
            "tgid" => report.tgid = value.parse().expect("tgid"),
            _ => {}
        }
    }
    report.argv = argv.into_values().collect();
    assert!(report.complete, "the report was truncated: {text}");
    report
}

/// A plan for the report fixture: the argv the test wants, the empty
/// environment the contract fixes, and the one working-directory identifier.
fn plan_bytes(argv: &[&[u8]]) -> Vec<u8> {
    let arguments: Vec<String> = argv
        .iter()
        .map(|argument| {
            let text = std::str::from_utf8(argument).expect("argv is validated UTF-8");
            let escaped = text
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .replace('\r', "\\r");
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
            r#""stdout":{{"capture_prefix_bytes":0}},"#,
            r#""stderr":{{"capture_prefix_bytes":0}},"#,
            r#""timeout_ms":30000,"#,
            r#""termination":{{"signal":"SIGTERM","grace_ms":5000}}}}"#
        ),
        arguments.join(",")
    )
    .into_bytes()
}

fn open_read_only(path: &Path) -> OwnedFd {
    OwnedFd::from(fs::File::open(path).unwrap_or_else(|error| panic!("open {path:?}: {error}")))
}

/// Admit and authorise a fixture as the trusted caller would.
fn authorize_fixture(
    executable: &Path,
    working_directory: &Path,
    argv: &[&[u8]],
) -> crate::authority::AuthorizedLaunch {
    let plan = parse_launch_plan(&plan_bytes(argv)).expect("the fixture plan is valid");
    let executable = admit_executable(open_read_only(executable)).expect("admit the fixture");
    let working_directory = admit_working_directory("workdir", open_read_only(working_directory))
        .expect("admit the working directory");
    authorize(plan, executable, working_directory).expect("authorise the fixture")
}

/// A bounded test watchdog: drain one non-blocking pipe end until end-of-file.
///
/// This is **test infrastructure**, not product stream semantics. P3 implements
/// no drain policy; P4 owns that.
fn drain_bounded(fd: &OwnedFd, bound: Duration) -> Vec<u8> {
    let deadline = Instant::now() + bound;
    let mut collected = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match rustix::io::read(fd.as_fd(), &mut buffer[..]) {
            Ok(0) => return collected,
            Ok(read) => collected.extend_from_slice(&buffer[..read]),
            Err(rustix::io::Errno::INTR) => {}
            Err(rustix::io::Errno::AGAIN) => {
                assert!(
                    Instant::now() < deadline,
                    "TEST WATCHDOG: the fixture did not close its stream within {bound:?}"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("reading a fixture stream failed: {error}"),
        }
    }
}

/// Launch, drain both streams, reap within the test watchdog.
struct Completed {
    launch: MinimalLaunch,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    end: Option<ChildEnd>,
}

fn complete(launch: MinimalLaunch) -> Completed {
    let stdout = drain_bounded(&launch.stdout_read, Duration::from_secs(20));
    let stderr = drain_bounded(&launch.stderr_read, Duration::from_secs(20));
    let end = launch.child.reap_within(Duration::from_secs(20));
    assert_eq!(
        launch.child.observed_end(),
        end,
        "the handle did not latch the end it observed, so a later drop could signal a reaped child"
    );
    Completed {
        launch,
        stdout,
        stderr,
        end,
    }
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = fixture_root().join(format!("scratch-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn identity(path: &Path) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt as _;
    let metadata = fs::metadata(path).expect("metadata");
    (metadata.dev(), metadata.ino())
}

// ===========================================================================
// 3. The X2c producer self-test
// ===========================================================================

#[test]
fn report_fixture_reports_without_the_backend() {
    // The X2c rule: prove the fixture can produce its report on its own, that
    // the report parses, and that its marker names the fixture this suite
    // intends — **before** any launcher test consumes one.
    let fixture = report_fixture();
    let output = Command::new(&fixture)
        .arg("self-test")
        .output()
        .expect("run the fixture directly");
    assert!(
        output.status.success(),
        "the fixture failed its own run: {output:?}"
    );
    let report = parse_report(&output.stdout);
    assert_eq!(
        report.name, REPORT_FIXTURE_NAME,
        "the marker names another fixture"
    );
    assert_eq!(report.argv.len(), 2);
    assert_eq!(report.argv[1], b"self-test");
    assert!(
        !report.environment.is_empty(),
        "run directly, the fixture inherits this process's environment; if that were empty the \
         later empty-environment assertion would prove nothing"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(REPORT_FIXTURE_NAME),
        "the fixture's stderr marker is missing"
    );
}

// ===========================================================================
// 4. Linux integration through the P3 backend
// ===========================================================================

#[test]
fn an_authorised_object_executes_with_exactly_the_intended_descriptors() {
    let fixture = report_fixture();
    let workdir = scratch_dir("fds");

    // Unrelated descriptors the launching process holds: one close-on-exec, one
    // not. Neither may reach the executed image (F1-F4). The claim rests on the
    // child's range close, not on HELM opening its own descriptors carefully.
    let cloexec = open_read_only(&fixture);
    let plain = {
        let file = fs::File::open(&fixture).expect("open");
        let fd = OwnedFd::from(file);
        rustix::io::fcntl_setfd(fd.as_fd(), rustix::io::FdFlags::empty()).expect("clear cloexec");
        fd
    };
    assert!(
        rustix::io::fcntl_getfd(plain.as_fd())
            .expect("flags")
            .is_empty(),
        "the control descriptor must really be without close-on-exec"
    );

    let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &[b"argv-zero"]))
        .expect("the launch prepares and spawns");
    let completed = complete(launch);
    drop(cloexec);
    drop(plain);

    let report = parse_report(&completed.stdout);
    assert_eq!(report.name, REPORT_FIXTURE_NAME);
    let numbers: Vec<u32> = report.descriptors.keys().copied().collect();
    assert_eq!(
        numbers,
        vec![0, 1, 2],
        "the executed image held descriptors {:?}",
        report.descriptors
    );
    for number in [0_u32, 1, 2] {
        assert!(
            report.descriptors[&number].starts_with("pipe:"),
            "descriptor {number} is {}",
            report.descriptors[&number]
        );
    }
    assert_eq!(completed.end, Some(ChildEnd::Exited { code: 0 }));
    assert_eq!(
        completed.launch.exec_status,
        ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord),
        "a clean status end-of-file is indeterminate, never exec success"
    );

    // The image that reported is the direct child the backend created, not a
    // descendant of one, and descriptor 2 really is the launcher's pipe.
    assert_eq!(
        i64::try_from(report.tgid).unwrap(),
        completed.launch.child.pid(),
        "the reporting image is not the direct child"
    );
    assert!(
        String::from_utf8_lossy(&completed.stderr).contains(REPORT_FIXTURE_NAME),
        "the image's stderr did not reach the launcher: {:?}",
        String::from_utf8_lossy(&completed.stderr)
    );

    // The stdin write end closed in the parent right after the clone, so the
    // image reads immediate end-of-file on 0 (T22).
    assert!(
        completed.launch.stdin_write.is_none(),
        "the parent kept the stdin write end open outside the injected stall"
    );

    // The P2 facts travel through the backend unchanged: nothing re-measures,
    // reopens or re-resolves the admitted object.
    let expected = {
        let plan = parse_launch_plan(&plan_bytes(&[b"argv-zero"])).unwrap();
        let capability = admit_executable(open_read_only(&fixture)).unwrap();
        (plan.sha256(), capability.measurement())
    };
    assert_eq!(completed.launch.plan_sha256, expected.0);
    assert_eq!(
        completed.launch.measurement.pre_exec_body_sha256(),
        expected.1.pre_exec_body_sha256()
    );
    assert_eq!(
        completed.launch.measurement.pre_exec_body_size(),
        expected.1.pre_exec_body_size()
    );
    assert_eq!(
        completed.launch.measurement.elf_type(),
        expected.1.elf_type()
    );
}

#[test]
fn argv_reaches_the_image_byte_for_byte_with_no_quoting_or_expansion() {
    let fixture = report_fixture();
    let workdir = scratch_dir("argv");
    let arguments: [&[u8]; 6] = [
        b"an arbitrary argv zero",
        b"with spaces",
        b"meta $HOME `id` * ? ; | & > <",
        b"line\nbreak",
        b"",
        b"tail",
    ];
    let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &arguments)).expect("launch");
    let completed = complete(launch);
    let report = parse_report(&completed.stdout);
    assert_eq!(report.name, REPORT_FIXTURE_NAME);
    let observed: Vec<&[u8]> = report.argv.iter().map(Vec::as_slice).collect();
    assert_eq!(
        observed, arguments,
        "argv was altered on the way to the image"
    );
}

#[test]
fn the_executed_image_observes_an_empty_environment() {
    let fixture = report_fixture();
    let workdir = scratch_dir("env");
    // Canaries in the launching process. `std::env::set_var` is unsafe in the
    // 2024 edition and this crate denies unsafe outside the backend, so the
    // canaries are the ones cargo already sets plus whatever the host has: the
    // producer self-test above already proved the fixture reports a non-empty
    // environment when it inherits one.
    assert!(
        std::env::vars_os().count() > 0,
        "TEST ENVIRONMENT FAILURE: the launching process has no environment, so an empty \
         child environment would prove nothing"
    );
    assert!(
        std::env::var_os("PATH").is_some(),
        "TEST ENVIRONMENT FAILURE: PATH is unset in the launching process"
    );

    let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &[b"env"])).expect("launch");
    let completed = complete(launch);
    let report = parse_report(&completed.stdout);
    assert_eq!(report.name, REPORT_FIXTURE_NAME);
    assert!(
        report.environment.is_empty(),
        "the image saw {} environment entries: {:?}",
        report.environment.len(),
        report.environment
    );
}

#[test]
fn the_executed_image_starts_in_the_admitted_directory() {
    let fixture = report_fixture();
    let workdir = scratch_dir("cwd");
    let expected = identity(&workdir);
    let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &[b"cwd"])).expect("launch");
    let completed = complete(launch);
    let report = parse_report(&completed.stdout);
    assert_eq!(
        (report.cwd_dev, report.cwd_ino),
        expected,
        "the image's working directory is not the admitted one"
    );
}

#[test]
fn the_executed_image_has_no_new_privs_set() {
    // The control: the launching process must not already have it, or the
    // assertion below would be vacuous.
    let status = fs::read_to_string("/proc/self/status").expect("read our own status");
    let parent_value = status
        .lines()
        .find_map(|line| line.strip_prefix("NoNewPrivs:"))
        .map(str::trim)
        .expect("TEST ENVIRONMENT FAILURE: NoNewPrivs is not reported for this process");
    assert_eq!(
        parent_value, "0",
        "TEST ENVIRONMENT FAILURE: the launching process already has no_new_privs set, so the \
         child's value would not show that the child set it"
    );

    let fixture = report_fixture();
    let workdir = scratch_dir("nnp");
    let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &[b"nnp"])).expect("launch");
    let completed = complete(launch);
    let report = parse_report(&completed.stdout);
    assert_eq!(report.no_new_privs, "1");
    // N3 stays a non-claim: nothing here attempts a privilege transition, and
    // no test asserts that one was prevented.
}

#[test]
fn the_parent_establishes_group_authority_and_issues_no_group_signal() {
    let fixture = report_fixture();
    let workdir = scratch_dir("group");
    let launch =
        launch_minimal(authorize_fixture(&fixture, &workdir, &[b"group"])).expect("launch");
    assert!(
        launch.group_authority_established,
        "the parent's own setpgid(child, child) did not succeed, so no later slice could sweep"
    );
    assert!(!launch.sigkill_sent, "a normal run needs no signal at all");
    let completed = complete(launch);
    assert_eq!(completed.end, Some(ChildEnd::Exited { code: 0 }));
}

#[test]
fn an_object_without_execute_permission_reports_exec_eacces() {
    let fixture = report_fixture();
    let workdir = scratch_dir("eacces");
    let copy = workdir.join("not-executable");
    fs::copy(&fixture, &copy).expect("copy the fixture");
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o600)).expect("drop execute bits");
    assert_eq!(
        fs::metadata(&copy).unwrap().permissions().mode() & 0o777,
        0o600,
        "the generated fixture mode is not what the test intends"
    );

    let launch = launch_minimal(authorize_fixture(&copy, &workdir, &[b"x"])).expect("launch");
    assert_eq!(
        launch.exec_status,
        ExecStatus::PreExecFailure {
            stage: ChildStage::Exec,
            errno: 13,
        }
    );
    let completed = complete(launch);
    assert_eq!(
        completed.end,
        Some(ChildEnd::Exited { code: 127 }),
        "the child's own failure path exits 127"
    );
    assert!(
        completed.stdout.is_empty(),
        "nothing executed, so nothing reported"
    );
}

#[test]
fn an_unloadable_in_cohort_object_reports_exec_enoexec() {
    let fixture = report_fixture();
    let workdir = scratch_dir("enoexec");
    let copy = workdir.join("broken-header");
    let mut bytes = fs::read(&fixture).expect("read the fixture");
    assert!(bytes.len() > 4096);
    // `e_phentsize`, the two bytes at offset 54, is the **first** thing the
    // kernel's ELF loader validates after the header fields admission itself
    // checks, and `load_elf_phdrs` refuses anything but `sizeof(Elf64_Phdr)`.
    // Zeroing it therefore gives a deterministic `ENOEXEC` while leaving the
    // magic, class, data encoding, machine and type — the only fields the
    // cohort check reads — exactly as they were.
    assert_eq!(
        u16::from_le_bytes([bytes[54], bytes[55]]),
        56,
        "the fixture's e_phentsize is not the ELF64 program-header size"
    );
    bytes[54] = 0;
    bytes[55] = 0;
    fs::write(&copy, &bytes).expect("write the broken object");
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o755)).expect("mode");
    assert_eq!(
        fs::metadata(&copy).unwrap().permissions().mode() & 0o777,
        0o755
    );

    let launch = launch_minimal(authorize_fixture(&copy, &workdir, &[b"x"])).expect("launch");
    assert_eq!(
        launch.exec_status,
        ExecStatus::PreExecFailure {
            stage: ChildStage::Exec,
            errno: 8,
        },
        "an in-cohort object the loader refuses must be a structured EXEC/ENOEXEC record"
    );
    let completed = complete(launch);
    assert_eq!(completed.end, Some(ChildEnd::Exited { code: 127 }));
    assert!(
        completed.stdout.is_empty(),
        "nothing executed, so nothing reported"
    );
}

#[test]
fn an_object_held_open_for_writing_reports_exec_etxtbsy() {
    let fixture = report_fixture();
    let workdir = scratch_dir("etxtbsy");
    let copy = workdir.join("busy");
    fs::copy(&fixture, &copy).expect("copy");
    fs::set_permissions(&copy, fs::Permissions::from_mode(0o755)).expect("mode");
    // A writer held by the test, never by the product.
    let writer = fs::OpenOptions::new()
        .write(true)
        .open(&copy)
        .expect("hold a writer");

    let launch = launch_minimal(authorize_fixture(&copy, &workdir, &[b"x"])).expect("launch");
    assert_eq!(
        launch.exec_status,
        ExecStatus::PreExecFailure {
            stage: ChildStage::Exec,
            errno: 26,
        }
    );
    let completed = complete(launch);
    assert_eq!(completed.end, Some(ChildEnd::Exited { code: 127 }));
    drop(writer);
}

#[test]
fn a_working_directory_that_became_unsearchable_reports_chdir_eacces() {
    let fixture = report_fixture();
    let outer = scratch_dir("chdir");
    let workdir = outer.join("inner");
    fs::create_dir_all(&workdir).expect("inner directory");
    let authorized = authorize_fixture(&fixture, &workdir, &[b"x"]);
    // After admission, before the launch: the capability pins the directory,
    // not its mode.
    fs::set_permissions(&workdir, fs::Permissions::from_mode(0o000)).expect("mode");
    assert_eq!(
        fs::metadata(&workdir).unwrap().permissions().mode() & 0o777,
        0o000
    );

    let launch = launch_minimal(authorized).expect("launch");
    let status = launch.exec_status;
    let completed = complete(launch);
    fs::set_permissions(&workdir, fs::Permissions::from_mode(0o755)).expect("restore mode");

    // A privileged host (a container running as root) can still search a
    // mode-0000 directory, in which case the child gets past CHDIR and executes.
    match status {
        ExecStatus::PreExecFailure {
            stage: ChildStage::Chdir,
            errno: 13,
        } => assert_eq!(completed.end, Some(ChildEnd::Exited { code: 127 })),
        ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord) => {
            assert!(
                rustix::process::geteuid().is_root(),
                "an unprivileged process must not be able to search a mode-0000 directory"
            );
        }
        other => panic!("unexpected exec status {other:?}"),
    }
}

#[test]
fn the_admitted_descriptor_is_executed_after_its_pathname_is_replaced() {
    let fixture = report_fixture();
    let workdir = scratch_dir("exact");
    let original = workdir.join("target");
    fs::copy(&fixture, &original).expect("copy");
    fs::set_permissions(&original, fs::Permissions::from_mode(0o755)).expect("mode");

    let authorized = authorize_fixture(&original, &workdir, &[b"exact"]);

    // After admission the pathname is unlinked and a different object takes its
    // place. The capability pins the inode, so the admitted body still runs.
    let replacement = workdir.join("replacement");
    fs::write(&replacement, b"#!/bin/sh\nexit 3\n").expect("write the replacement");
    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o755)).expect("mode");
    fs::rename(&replacement, &original).expect("replace the pathname");

    let launch = launch_minimal(authorized).expect("launch");
    let completed = complete(launch);
    let report = parse_report(&completed.stdout);
    assert_eq!(
        report.name, REPORT_FIXTURE_NAME,
        "the object behind the old pathname ran instead of the admitted one"
    );
    assert_eq!(completed.end, Some(ChildEnd::Exited { code: 0 }));
}

// ===========================================================================
// 5. S5 and S6, through the test-only injection
// ===========================================================================

#[cfg(all(feature = "test-fault-injection", debug_assertions))]
mod injected {
    use super::{
        ChildEnd, Duration, ExecStatus, Fault, IndeterminateReason, Path, authorize_fixture,
        complete, report_fixture, scratch_dir, stage,
    };
    use crate::backend::{injection, launch_minimal_with_fault};
    use crate::model::ChildStage;

    fn launch(executable: &Path, workdir: &Path, fault: Fault) -> crate::backend::MinimalLaunch {
        launch_minimal_with_fault(
            authorize_fixture(executable, workdir, &[b"injected"]),
            fault,
        )
        .expect("launch")
    }

    #[test]
    fn a_child_that_dies_before_exec_without_a_record_is_indeterminate_never_success() {
        // S5. The child exits 0 without ever executing anything, so the parent
        // sees exactly what a successful exec shows: a clean status
        // end-of-file beside a normal exit status.
        let fixture = report_fixture();
        let workdir = scratch_dir("s5");
        let result = launch(
            &fixture,
            &workdir,
            Fault {
                stage: stage::NONE,
                errno: 0,
                mode: injection::MODE_EXIT_BEFORE_EXEC,
            },
        );
        assert_eq!(
            result.exec_status,
            ExecStatus::Indeterminate(IndeterminateReason::StatusEofWithoutRecord)
        );
        assert!(!result.sigkill_sent);
        let completed = complete(result);
        assert_eq!(completed.end, Some(ChildEnd::Exited { code: 0 }));
        assert!(
            completed.stdout.is_empty(),
            "nothing was executed, so the fixture cannot have reported"
        );
    }

    #[test]
    fn a_child_that_stalls_before_exec_is_bounded_killed_and_reaped() {
        // S6. The fixed pre-exec bound passes, one SIGKILL goes through the
        // pidfd, and the bounded non-blocking reap collects the child.
        let fixture = report_fixture();
        let workdir = scratch_dir("s6");
        let started = std::time::Instant::now();
        let result = launch(
            &fixture,
            &workdir,
            Fault {
                stage: stage::NONE,
                errno: 0,
                mode: injection::MODE_STALL_BEFORE_EXEC,
            },
        );
        assert_eq!(
            result.exec_status,
            ExecStatus::Indeterminate(IndeterminateReason::PreExecStatusTimeout)
        );
        assert!(
            result.sigkill_sent,
            "the pre-exec bound must send one SIGKILL"
        );
        assert_eq!(
            result.child.observed_end(),
            Some(ChildEnd::Signaled {
                signal: 9,
                core_dumped: false,
            }),
            "the bounded reap did not collect the killed child, which would leave a zombie"
        );
        let elapsed = started.elapsed();
        assert!(
            elapsed
                < Duration::from_millis(
                    super::SPAWN_CONFIRM_TIMEOUT_MS + super::POST_KILL_REAP_MS + 5_000
                ),
            "the bounded path took {elapsed:?}"
        );
        assert!(
            elapsed >= Duration::from_millis(super::SPAWN_CONFIRM_TIMEOUT_MS),
            "the pre-exec bound was not actually waited out: {elapsed:?}"
        );
        let _ = complete(result);
    }

    #[test]
    fn every_child_stage_can_report_its_own_structured_failure() {
        // Stages whose real failure cannot be provoked through host state still
        // have a tested failure path.
        let fixture = report_fixture();
        let workdir = scratch_dir("stages");
        for code in super::STAGE_ORDER {
            let result = launch(
                &fixture,
                &workdir,
                Fault {
                    stage: code,
                    errno: 42,
                    mode: injection::MODE_NONE,
                },
            );
            assert_eq!(
                result.exec_status,
                ExecStatus::PreExecFailure {
                    stage: super::stage_of(code).unwrap(),
                    errno: 42,
                },
                "stage {code}"
            );
            let completed = complete(result);
            assert_eq!(
                completed.end,
                Some(ChildEnd::Exited { code: 127 }),
                "stage {code}"
            );
        }
        // And the vocabulary really is the public one.
        let _: ChildStage = ChildStage::Exec;
    }
}

// ===========================================================================
// 6. The traced child window
// ===========================================================================

/// Which shape the traced inner test should run.
const TRACE_MODE_VARIABLE: &str = "HELM_LAUNCH_P3_TRACE_MODE";
const TRACE_MODE_SINGLE: &str = "single";
const TRACE_MODE_THREADED: &str = "threaded";

/// The inner half of the tracer tests. It is ignored by default and is run only
/// by [`run_traced`], under `strace -f`.
#[test]
#[ignore = "run under strace by the tracer tests, never on its own"]
fn traced_launch_inner() {
    let mode = std::env::var(TRACE_MODE_VARIABLE).unwrap_or_default();
    assert!(
        mode == TRACE_MODE_SINGLE || mode == TRACE_MODE_THREADED,
        "the traced inner test must be started by its outer test"
    );

    let fixture = report_fixture();
    let workdir = scratch_dir(&format!("trace-{mode}"));

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut helpers = Vec::new();
    if mode == TRACE_MODE_THREADED {
        // A parent whose other threads are actively allocating while the clone
        // happens (M1, M2). The child must still run only its own closed
        // sequence: no allocator, no lock, no atfork handler.
        for _ in 0..3 {
            let stop = std::sync::Arc::clone(&stop);
            helpers.push(std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut churn: Vec<Vec<u8>> = Vec::new();
                    for size in 1..64_usize {
                        churn.push(vec![7_u8; size * 97]);
                    }
                    std::hint::black_box(&churn);
                }
            }));
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let launch = launch_minimal(authorize_fixture(&fixture, &workdir, &[b"traced"]))
        .expect("the traced launch spawns");
    let completed = complete(launch);
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    for helper in helpers {
        let _ = helper.join();
    }
    let report = parse_report(&completed.stdout);
    assert_eq!(report.name, REPORT_FIXTURE_NAME);
    assert_eq!(completed.end, Some(ChildEnd::Exited { code: 0 }));
}

/// Run the inner test under `strace -f` and return the trace text.
fn run_traced(label: &str, mode: &str, extra_env: &[(&str, &OsStr)]) -> String {
    require_tool(
        "strace",
        "the closed child syscall window is a traced property",
    );
    // Build the fixture **here**, outside the tracer, so the traced process
    // finds a warm cache and no compiler runs under `strace -f`.
    let _ = report_fixture();
    let output_path = fixture_root().join(format!("trace-{label}-{}.txt", std::process::id()));
    let _ = fs::remove_file(&output_path);
    let mut command = Command::new("strace");
    command.arg("-f").arg("-qq").arg("-o").arg(&output_path);

    // Per-run variables are applied by `env(1)` **inside** the tracer rather
    // than by inheritance, because one of them is `LD_PRELOAD` and `strace`
    // itself starts its tracee with `fork(2)`. Inherited, the preload would run
    // in `strace`, whose own fork would fire the very `pthread_atfork` handlers
    // the test is proving do not run. `env` execs without forking, so the
    // preload first exists in the launcher process itself.
    if extra_env.is_empty() {
        command.arg(std::env::current_exe().expect("the test binary"));
    } else {
        require_tool("env", "per-run variables are applied inside the tracer");
        command.arg("env");
        for (key, value) in extra_env {
            let mut assignment = std::ffi::OsString::from(key);
            assignment.push("=");
            assignment.push(value);
            command.arg(assignment);
        }
        command.arg(std::env::current_exe().expect("the test binary"));
    }

    command
        .args([
            "--exact",
            "backend::tests::traced_launch_inner",
            "--ignored",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(TRACE_MODE_VARIABLE, mode);
    let output = command.output().expect("run strace");
    let trace = fs::read_to_string(&output_path).unwrap_or_default();
    assert!(
        output.status.success(),
        "the traced inner test failed.\nstdout:\n{}\nstderr:\n{}\ntrace tail:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        trace
            .lines()
            .rev()
            .take(40)
            .collect::<Vec<&str>>()
            .join("\n")
    );
    assert!(
        !trace.is_empty(),
        "TEST ENVIRONMENT FAILURE: strace produced no output"
    );
    trace
}

#[derive(Debug, Clone)]
struct TraceLine {
    pid: u64,
    name: String,
    arguments: String,
    result: String,
}

/// Parse `strace -f -o file` output into per-task syscall records.
///
/// Interrupted calls are spliced at the point they resumed, which is when the
/// call actually completed, so per-task ordering stays true.
fn parse_trace(text: &str) -> Vec<TraceLine> {
    let mut pending: BTreeMap<u64, String> = BTreeMap::new();
    let mut lines = Vec::new();
    for raw in text.lines() {
        let raw = raw.trim_end();
        let Some((pid_text, rest)) = raw.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(pid) = pid_text.parse::<u64>() else {
            continue;
        };
        let rest = rest.trim_start();
        if rest.starts_with("---") || rest.starts_with("+++") {
            continue;
        }
        let joined = if let Some(head) = rest.strip_suffix("<unfinished ...>") {
            pending.insert(pid, head.trim_end().to_owned());
            continue;
        } else if rest.starts_with("<...") {
            let Some((_, tail)) = rest.split_once("resumed>") else {
                continue;
            };
            match pending.remove(&pid) {
                Some(head) => format!("{head}{tail}"),
                None => continue,
            }
        } else {
            rest.to_owned()
        };

        let Some(open) = joined.find('(') else {
            continue;
        };
        let name = joined[..open].trim().to_owned();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let (arguments, result) = match joined.rfind(" = ") {
            Some(at) => (
                joined[open + 1..at].trim_end().to_owned(),
                joined[at + 3..].trim().to_owned(),
            ),
            None => (joined[open + 1..].to_owned(), String::new()),
        };
        let arguments = arguments.strip_suffix(')').unwrap_or(&arguments).to_owned();
        lines.push(TraceLine {
            pid,
            name,
            arguments,
            result,
        });
    }
    lines
}

/// Select the **process** clone, never a test-harness thread clone.
fn process_clone(lines: &[TraceLine]) -> &TraceLine {
    let candidates: Vec<&TraceLine> = lines
        .iter()
        .filter(|line| {
            line.name == "clone3"
                && line.arguments.contains("CLONE_PIDFD")
                && !line.arguments.contains("CLONE_THREAD")
        })
        .collect();
    assert_eq!(
        candidates.len(),
        1,
        "expected exactly one CLONE_PIDFD process clone, found {}: {:?}",
        candidates.len(),
        candidates
    );
    // The Rust test harness clones threads with `clone3` too, which is exactly
    // the Trial #2 M2 confusion this filter exists to avoid.
    let thread_clones = lines
        .iter()
        .filter(|line| line.name == "clone3" && line.arguments.contains("CLONE_THREAD"))
        .count();
    let _ = thread_clones;
    candidates[0]
}

fn assert_closed_child_window(trace: &str) {
    let lines = parse_trace(trace);
    assert!(!lines.is_empty(), "the trace parsed to nothing");

    // --- the process clone ------------------------------------------------
    let clone = process_clone(&lines);
    assert!(
        clone.arguments.contains("pidfd="),
        "clone3 did not carry a pidfd output pointer: {clone:?}"
    );
    assert!(
        clone.arguments.contains("exit_signal=SIGCHLD"),
        "clone3 did not set exit_signal=SIGCHLD: {clone:?}"
    );
    for forbidden in ["CLONE_VM", "CLONE_VFORK", "CLONE_FILES", "CLONE_THREAD"] {
        assert!(
            !clone.arguments.contains(forbidden),
            "clone3 set {forbidden}: {clone:?}"
        );
    }
    assert!(
        !lines.iter().any(|line| line.name == "pidfd_open"),
        "a pidfd was opened separately instead of coming from clone3"
    );
    let child_pid: u64 = clone
        .result
        .parse()
        .expect("the clone3 result is the child pid");

    // --- the parent's own window -----------------------------------------
    let parent: Vec<&TraceLine> = lines.iter().filter(|line| line.pid == clone.pid).collect();
    let at = parent
        .iter()
        .position(|line| std::ptr::eq(*line, clone))
        .expect("the clone is in its own task's stream");
    assert!(
        at > 0,
        "the clone is the parent's first traced call, so nothing precedes it"
    );
    assert!(
        parent.len() > at + 2,
        "the parent's traced stream ends within two calls of the clone"
    );
    let before = parent[at - 1];
    assert_eq!(
        before.name, "rt_sigprocmask",
        "clone3 is not immediately preceded by a mask change"
    );
    assert!(
        before.arguments.starts_with("SIG_SETMASK"),
        "the mask before clone3 is not a replacement: {before:?}"
    );
    assert!(
        before.arguments.contains("~[]"),
        "the mask before clone3 is not the full set: {before:?}"
    );
    let after = parent[at + 1];
    assert_eq!(
        after.name, "setpgid",
        "setpgid is not the first syscall after clone3: {after:?}"
    );
    assert_eq!(
        after.arguments.replace(' ', ""),
        format!("{child_pid},{child_pid}"),
        "setpgid did not name the child twice"
    );
    let restore = parent[at + 2];
    assert_eq!(
        restore.name, "rt_sigprocmask",
        "the mask is not restored after setpgid"
    );
    assert!(
        restore.arguments.starts_with("SIG_SETMASK"),
        "the restore is not a replacement: {restore:?}"
    );

    // --- no group signal anywhere ----------------------------------------
    for line in &lines {
        if line.name == "kill" || line.name == "tgkill" || line.name == "tkill" {
            assert!(
                !line.arguments.trim_start().starts_with('-'),
                "P3 issued a process-group signal: {line:?}"
            );
        }
    }
    assert!(
        !lines.iter().any(|line| line.name == "kill"),
        "P3 issued a pid-valued kill at all"
    );

    // --- waiting is by pidfd ---------------------------------------------
    for line in lines.iter().filter(|line| line.name == "waitid") {
        assert!(
            line.arguments.starts_with("P_PIDFD"),
            "a wait did not use P_PIDFD: {line:?}"
        );
    }
    assert!(
        !lines
            .iter()
            .any(|line| line.name == "waitpid" || line.name == "wait4"),
        "the direct child was waited for by pid"
    );

    // --- the child window -------------------------------------------------
    let child: Vec<&TraceLine> = lines.iter().filter(|line| line.pid == child_pid).collect();
    assert!(!child.is_empty(), "the child produced no traced syscall");
    let exec_at = child
        .iter()
        .position(|line| line.name == "execveat")
        .expect("the child never reached execveat");
    let window = &child[..=exec_at];

    const PERMITTED: [&str; 11] = [
        "dup2",
        "fcntl",
        "fchdir",
        "close_range",
        "setpgid",
        "rt_sigaction",
        "rt_sigprocmask",
        "prctl",
        "execveat",
        "write",
        "exit_group",
    ];
    for line in window {
        assert!(
            PERMITTED.contains(&line.name.as_str()),
            "the child window contains `{}`, which is not in the permitted set: {line:?}",
            line.name
        );
    }

    // --- the stage order --------------------------------------------------
    let observed: Vec<&str> = window.iter().map(|line| line.name.as_str()).collect();
    let mut at = 0_usize;
    let mut take = |name: &str, minimum: usize, maximum: usize| {
        let start = at;
        while at < observed.len() && observed[at] == name && at - start < maximum {
            at += 1;
        }
        let seen = at - start;
        assert!(
            seen >= minimum,
            "expected at least {minimum} `{name}` at position {start}, saw {seen}: {observed:?}"
        );
    };
    take("dup2", 3, 3);
    take("fcntl", 3, 3);
    take("fchdir", 1, 1);
    take("close_range", 1, 3);
    take("setpgid", 1, 1);
    take("rt_sigaction", 62, 62);
    take("rt_sigprocmask", 1, 1);
    take("prctl", 1, 1);
    take("execveat", 1, 1);
    assert_eq!(
        at,
        observed.len(),
        "unexpected trailing calls: {observed:?}"
    );

    let fchdir_at = observed.iter().position(|name| *name == "fchdir").unwrap();
    let close_at = observed
        .iter()
        .position(|name| *name == "close_range")
        .unwrap();
    assert!(
        fchdir_at < close_at,
        "fchdir must precede the first close_range"
    );

    // --- the execution itself --------------------------------------------
    let exec = window[exec_at];
    assert!(
        exec.arguments.contains("AT_EMPTY_PATH"),
        "the exec did not use the descriptor itself: {exec:?}"
    );
    assert!(
        exec.arguments.contains("\"\""),
        "the exec passed a pathname: {exec:?}"
    );
    assert!(
        !exec.arguments.contains("/proc/self/fd"),
        "the exec went through procfs: {exec:?}"
    );
    assert_eq!(
        exec.result, "0",
        "the traced exec did not succeed: {exec:?}"
    );
}

#[test]
fn the_child_window_is_closed_from_a_single_threaded_parent() {
    assert_closed_child_window(&run_traced("single", TRACE_MODE_SINGLE, &[]));
}

#[test]
fn the_child_window_is_closed_from_a_multithreaded_allocating_parent() {
    assert_closed_child_window(&run_traced("threaded", TRACE_MODE_THREADED, &[]));
}

#[test]
fn a_registered_pthread_atfork_handler_does_not_run_in_the_child() {
    // Registering `pthread_atfork` from Rust would need an `extern` declaration,
    // which would put unsafe outside `src/backend/`. A tiny, newly written,
    // test-only C helper is preloaded into the traced process instead. It is a
    // host-condition helper, never a launcher.
    require_tool(
        "cc",
        "the pthread_atfork host condition needs a tiny C helper",
    );
    let helper = atfork_helper();
    let registered = fixture_root().join(format!("atfork-registered-{}.txt", std::process::id()));
    let ran = fixture_root().join(format!("atfork-ran-{}.txt", std::process::id()));
    let _ = fs::remove_file(&registered);
    let _ = fs::remove_file(&ran);

    let trace = run_traced(
        "atfork",
        TRACE_MODE_SINGLE,
        &[
            ("LD_PRELOAD", helper.as_os_str()),
            ("HELM_LAUNCH_P3_ATFORK_REGISTERED", registered.as_os_str()),
            ("HELM_LAUNCH_P3_ATFORK_RAN", ran.as_os_str()),
        ],
    );

    // The control: without this the test would pass vacuously, because a
    // helper that never loaded also never runs.
    assert!(
        registered.exists(),
        "TEST ENVIRONMENT FAILURE: the atfork helper was not preloaded into the launcher process"
    );
    // The claim: `clone3` is a raw system call, so glibc runs nothing across
    // it — not the prepare handler in the parent, not the parent handler, and
    // above all not the child handler inside the closed child window.
    assert!(
        !ran.exists(),
        "a pthread_atfork handler ran across the clone: {:?}",
        fs::read_to_string(&ran)
    );
    assert_closed_child_window(&trace);
    let _ = fs::remove_file(&registered);
    let _ = fs::remove_file(&ran);
}

/// The one test-only C host-condition helper, compiled at test time.
const ATFORK_HELPER_SOURCE: &str = r#"
/* Test-only helper for helm-launch P3. It registers pthread_atfork handlers in
 * the launching process so that a test can prove they do NOT run across the
 * backend's raw clone3. It is newly written for this suite, it is not derived
 * from any LAUNCH-EXEC asset, it never enters product code, and it does not
 * launch anything. */
#define _GNU_SOURCE
#include <fcntl.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static void note(const char *variable, const char *text) {
    const char *path = getenv(variable);
    if (path == NULL) {
        return;
    }
    int fd = open(path, O_WRONLY | O_CREAT | O_APPEND, 0600);
    if (fd < 0) {
        return;
    }
    ssize_t written = write(fd, text, strlen(text));
    (void) written;
    close(fd);
}

static void before_fork(void) { note("HELM_LAUNCH_P3_ATFORK_RAN", "prepare\n"); }
static void in_parent(void)   { note("HELM_LAUNCH_P3_ATFORK_RAN", "parent\n"); }
static void in_child(void)    { note("HELM_LAUNCH_P3_ATFORK_RAN", "child\n"); }

__attribute__((constructor))
static void install(void) {
    if (pthread_atfork(before_fork, in_parent, in_child) == 0) {
        note("HELM_LAUNCH_P3_ATFORK_REGISTERED", "registered\n");
    }
}
"#;

fn atfork_helper() -> PathBuf {
    let digest = hex(&Sha256::digest(ATFORK_HELPER_SOURCE.as_bytes()));
    let stem = format!("atfork-{}", &digest[..16]);
    let library = fixture_root().join(format!("lib{stem}.so"));
    if library.exists() {
        return library;
    }
    let source = fixture_root().join(format!("{stem}.c"));
    fs::write(&source, ATFORK_HELPER_SOURCE).expect("write the helper source");
    let staged = fixture_root().join(format!("{stem}.{}.staging", std::process::id()));
    let status = Command::new("cc")
        .args(["-shared", "-fPIC", "-O1", "-o"])
        .arg(&staged)
        .arg(&source)
        .arg("-lpthread")
        .status()
        .expect("run cc");
    assert!(
        status.success(),
        "TEST ENVIRONMENT FAILURE: cc could not build the atfork host-condition helper"
    );
    fs::rename(&staged, &library).expect("stage the helper");
    library
}

// ===========================================================================
// 7. Errors that create no child
// ===========================================================================

#[test]
fn a_backend_error_is_crate_private_data_with_no_host_text() {
    // P3 adds no public error vocabulary. These values exist only inside the
    // crate, carry a raw `errno` and a fixed step, and never a host string,
    // path, pid or descriptor number.
    let error = BackendError::Preparation {
        step: super::PrepareStep::Pipe,
        errno: 24,
    };
    let rendered = format!("{error:?}");
    assert!(rendered.contains("Pipe"));
    assert!(rendered.contains("24"));
    assert!(
        !rendered.contains('/'),
        "a host path reached an error: {rendered}"
    );
    assert_eq!(
        error,
        BackendError::Preparation {
            step: super::PrepareStep::Pipe,
            errno: 24
        }
    );
    assert_ne!(error, BackendError::ProcessCreationFailed { errno: 24 });
}
