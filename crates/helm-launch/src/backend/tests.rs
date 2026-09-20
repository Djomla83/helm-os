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
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
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
pub(crate) fn require_tool(tool: &str, why: &str) {
    let found = Command::new(tool).arg("--version").output();
    match found {
        Ok(output) if output.status.success() => {}
        _ => panic!(
            "TEST ENVIRONMENT FAILURE: `{tool}` is required for the helm-launch P3 suite ({why}), \
             and it is not usable here. This is not a launcher result; install the tool and rerun."
        ),
    }
}

pub(crate) fn fixture_root() -> &'static Path {
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

/// Decode one hexadecimal run, or refuse it. Fallible, because a producer
/// report that is not well formed must be **refused** rather than abort the
/// parser mid-way (owner disposition of P3R-10).
fn try_unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len() / 2)
        .map(|i| {
            text.get(i * 2..i * 2 + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect()
}

/// One filename namespace per **actual** compiler invocation.
///
/// `std::process::id()` alone is not enough: cargo runs tests in parallel
/// threads of one process, so every thread sees the same pid. Seventeen tests
/// request the same content-addressed fixture, so several of them called
/// `rustc` with the same source pathname and the same `-o` pathname at once —
/// and `rustc` derives its intermediate object basenames from that `-o`
/// pathname, so the concurrent invocations deleted and overwrote one another's
/// objects. That was **`P3R-20`**, and it is what failed the first hosted
/// Linux run of this suite: `undefined hidden symbol` for the fixture's own
/// code generation units in one run, `cannot open …rcgu.o` in the next, with a
/// different set of tests failing each time.
///
/// The pid still separates concurrent test **processes**; this nonce separates
/// builders inside one.
static FIXTURE_BUILD_NONCE: AtomicU64 = AtomicU64::new(0);

/// Compile one fixture source with `rustc`, cached by the digest of its text.
///
/// The fixture is an ordinary dynamically linked `ET_DYN` object — the same
/// shape a real caller would admit, which also exercises the point that an
/// empty environment still resolves an interpreter and libraries by name from
/// host state (E7).
///
/// **Safe to call concurrently for the same source** (`P3R-20`). Every call
/// that really compiles owns a unique source pathname and a unique `-o`
/// pathname, and publication is atomic and **no-replace**: the winner links its
/// staged object into the content-addressed name, every loser is told
/// `AlreadyExists`, drops its own object and uses the winner's. The published
/// object is therefore never replaced once it exists.
///
/// Correctness rests on the filesystem rather than on a process-local lock, so
/// separate test processes publishing the same fixture are safe too, and no
/// lock is held across launching anything.
pub(crate) fn fixture_binary(name: &str, source: &str) -> PathBuf {
    let digest = hex(&Sha256::digest(source.as_bytes()));
    let stem = format!("{name}-{}", &digest[..16]);
    let binary = fixture_root().join(&stem);
    if binary.exists() {
        // Content-addressed, so a warm cache needs no compiler at all. That is
        // also what keeps `rustc` out of a traced run.
        return binary;
    }
    require_tool("rustc", "fixtures are compiled at test time, never shipped");

    // Two concurrent builders share no source pathname, no `-o` pathname and
    // therefore no `rustc` intermediate object basename. The separator is `-`
    // and not `.`, because `rustc` derives the crate name from the **source**
    // file stem and refuses one containing a dot.
    let nonce = FIXTURE_BUILD_NONCE.fetch_add(1, Ordering::Relaxed);
    let unique = format!("{stem}-{}-{nonce}", std::process::id());
    let source_path = fixture_root().join(format!("{unique}.rs"));
    let staged = fixture_root().join(format!("{unique}.staging"));

    // The crate name stays derived from the content-addressed stem alone, so
    // the emitted fixture is the same bytes whichever builder compiled it and
    // the content-addressed name keeps describing the content.
    let crate_name: String = stem
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();

    fs::write(&source_path, source).expect("write fixture source");
    let status = Command::new("rustc")
        .args(["--edition", "2021", "-C", "debuginfo=0", "--crate-name"])
        .arg(&crate_name)
        .arg("-o")
        .arg(&staged)
        .arg(&source_path)
        .status()
        .expect("run rustc");
    // This builder owns both temporary names, so it removes both whatever
    // happens, and never touches a name another builder owns.
    let _ = fs::remove_file(&source_path);
    if !status.success() {
        let _ = fs::remove_file(&staged);
        // A missing or unusable `rustc` is a test environment failure, raised
        // by `require_tool` above. A `rustc` that ran and refused the source is
        // a different thing and says so.
        panic!(
            "FIXTURE BUILD FAILURE: rustc ran and exited {status} building fixture {name}; \
             the compiler is present, so this is a harness or fixture-source failure"
        );
    }

    // Publish without replacing a winner. `rename` would replace an
    // already-published inode, and testing `exists()` before a `rename` would
    // only move the race; `hard_link` refuses atomically instead.
    match fs::hard_link(&staged, &binary) {
        // This builder published the fixture. The inode survives under
        // `binary`, so its staging name can go.
        Ok(()) => {}
        // Another builder published the same content first. Its object is the
        // one every caller must use, and this builder's own object is dropped
        // unused. The published fixture is never removed or replaced here.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => panic!("FIXTURE BUILD FAILURE: could not publish fixture {name}: {error}"),
    }
    let _ = fs::remove_file(&staged);

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

// The process group this image belongs to, from field 5 of `/proc/self/stat`.
// The comm field is parenthesised and may itself contain spaces and
// parentheses, so the scan starts after the LAST `)`: state, ppid, pgrp.
fn process_group() -> Option<u32> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after_comm = stat.get(stat.rfind(')')? + 1..)?;
    let mut fields = after_comm.split_whitespace();
    let _state = fields.next()?;
    let _ppid = fields.next()?;
    fields.next()?.parse().ok()
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

    // Two DIFFERENT facts, reported separately and never conflated (owner
    // disposition of P3R-10). `tgid` is this image's own process identity, which
    // on Linux is the thread-group id of a single-threaded process and is what
    // `std::process::id` returns. `pgid_is_self` is whether this image leads its
    // own process group, which is a statement about the group and not about
    // identity. An image that cannot determine its group reports a value the
    // parser refuses, rather than a plausible-looking guess.
    let identity = std::process::id();
    out.push_str(&format!("tgid={}\n", identity));
    match process_group() {
        Some(group) => out.push_str(&format!("pgid_is_self={}\n", group == identity)),
        None => out.push_str("pgid_is_self=unavailable\n"),
    }
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
///
/// Every field is **mandatory**, and the type derives no `Default`, so a
/// `Report` cannot exist carrying a value the producer never sent. That is the
/// structural fix for **P3R-10**: before it, a key the fixture emitted under one
/// spelling and the parser read under another left a default behind, and an
/// assertion against that default compared `0` to a real pid.
#[derive(Debug, PartialEq, Eq)]
struct Report {
    name: String,
    argv: Vec<Vec<u8>>,
    environment: Vec<(Vec<u8>, Vec<u8>)>,
    cwd_dev: u64,
    cwd_ino: u64,
    descriptors: BTreeMap<u32, String>,
    no_new_privs: String,
    /// The executed image's own process identity, which on Linux is the
    /// thread-group id of a single-threaded process. **Not** a group id.
    tgid: u32,
    /// Whether the executed image leads its own process group. A different fact
    /// from [`Report::tgid`], and asserted separately: pid, tgid and pgid are
    /// never conflated.
    pgid_is_self: bool,
}

/// Why a producer report is refused. A refused report is never partially used.
#[derive(Debug, PartialEq, Eq)]
enum ReportRejection {
    /// The output was not text at all.
    NotText,
    /// The versioned first line is missing or is not the expected marker.
    Marker,
    /// The terminating `end` line never arrived.
    Truncated,
    /// A key the schema does not define. A producer and a parser that disagree
    /// on a spelling therefore fail loudly instead of leaving a default behind.
    UnknownKey(String),
    /// A key the schema requires never appeared.
    Missing(&'static str),
    /// A key the schema allows once appeared more than once.
    Duplicate(&'static str),
    /// A value did not parse as the schema requires.
    Malformed(&'static str),
}

/// The versioned first line of the report-capable fixture schema.
const REPORT_MARKER: &str = "HELM-LAUNCH-P3-FIXTURE/1";

/// The scalar keys the schema defines, each required **exactly once**.
const REPORT_SCALARS: [&str; 12] = [
    "argv_count",
    "cwd_dev",
    "cwd_ino",
    "env_count",
    "fd_count",
    "name",
    "no_new_privs",
    "pgid_is_self",
    "sig_blk",
    "sig_cgt",
    "sig_ign",
    "tgid",
];

/// Keys the schema defines but does not require. `cwd_error` replaces the
/// working-directory identity, so a report carrying it is still refused — for
/// the missing `cwd_dev`, with the reason visible.
const REPORT_OPTIONAL: [&str; 1] = ["cwd_error"];

/// The keys that may repeat, each carrying one entry of a list.
const REPORT_REPEATED: [&str; 3] = ["argv", "env", "fd"];

fn scalar_key(key: &str) -> Option<&'static str> {
    REPORT_SCALARS.iter().copied().find(|known| *known == key)
}

fn repeated_key(key: &str) -> Option<&'static str> {
    REPORT_REPEATED.iter().copied().find(|known| *known == key)
}

/// Parse one producer report against the closed schema.
///
/// Every key must be one the schema defines, every scalar key must appear
/// exactly once, every required value must parse, and the declared counts must
/// agree with the entries that arrived. A report that fails any of those is
/// **refused**, never partially used (owner disposition of P3R-10).
fn try_parse_report(bytes: &[u8]) -> Result<Report, ReportRejection> {
    let text = std::str::from_utf8(bytes).map_err(|_| ReportRejection::NotText)?;
    let mut lines = text.lines();
    if lines.next() != Some(REPORT_MARKER) {
        return Err(ReportRejection::Marker);
    }

    let mut scalars: BTreeMap<&'static str, &str> = BTreeMap::new();
    let mut argv: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut descriptors: BTreeMap<u32, String> = BTreeMap::new();
    let mut complete = false;

    for line in lines {
        if line == "end" {
            complete = true;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(ReportRejection::UnknownKey(line.to_owned()));
        };
        if let Some(known) = scalar_key(key) {
            if scalars.insert(known, value).is_some() {
                return Err(ReportRejection::Duplicate(known));
            }
            continue;
        }
        let Some(known) = repeated_key(key) else {
            if REPORT_OPTIONAL.contains(&key) {
                continue;
            }
            return Err(ReportRejection::UnknownKey(key.to_owned()));
        };
        let Some((left, right)) = value.split_once(':') else {
            return Err(ReportRejection::Malformed(known));
        };
        match known {
            "argv" => {
                let index: usize = left
                    .parse()
                    .map_err(|_| ReportRejection::Malformed(known))?;
                let decoded = try_unhex(right).ok_or(ReportRejection::Malformed(known))?;
                if argv.insert(index, decoded).is_some() {
                    return Err(ReportRejection::Duplicate(known));
                }
            }
            "env" => {
                let name = try_unhex(left).ok_or(ReportRejection::Malformed(known))?;
                let value = try_unhex(right).ok_or(ReportRejection::Malformed(known))?;
                environment.push((name, value));
            }
            // "fd"
            _ => {
                let number: u32 = left
                    .parse()
                    .map_err(|_| ReportRejection::Malformed(known))?;
                let target = try_unhex(right).ok_or(ReportRejection::Malformed(known))?;
                if descriptors
                    .insert(number, String::from_utf8_lossy(&target).into_owned())
                    .is_some()
                {
                    return Err(ReportRejection::Duplicate(known));
                }
            }
        }
    }

    if !complete {
        return Err(ReportRejection::Truncated);
    }

    let scalar = |key: &'static str| -> Result<&str, ReportRejection> {
        scalars
            .get(key)
            .copied()
            .ok_or(ReportRejection::Missing(key))
    };
    let number = |key: &'static str| -> Result<u64, ReportRejection> {
        scalar(key)?
            .parse()
            .map_err(|_| ReportRejection::Malformed(key))
    };
    let count = |key: &'static str| -> Result<usize, ReportRejection> {
        scalar(key)?
            .parse()
            .map_err(|_| ReportRejection::Malformed(key))
    };

    // The declared counts must match the entries that arrived, and the argv
    // indices must be the complete ascending run, so a dropped or renumbered
    // entry cannot pass as a shorter vector.
    if count("argv_count")? != argv.len() || !argv.keys().copied().eq(0..argv.len()) {
        return Err(ReportRejection::Malformed("argv_count"));
    }
    if count("env_count")? != environment.len() {
        return Err(ReportRejection::Malformed("env_count"));
    }
    if count("fd_count")? != descriptors.len() {
        return Err(ReportRejection::Malformed("fd_count"));
    }

    let tgid: u32 = scalar("tgid")?
        .parse()
        .map_err(|_| ReportRejection::Malformed("tgid"))?;
    let pgid_is_self = match scalar("pgid_is_self")? {
        "true" => true,
        "false" => false,
        _ => return Err(ReportRejection::Malformed("pgid_is_self")),
    };

    Ok(Report {
        name: scalar("name")?.to_owned(),
        argv: argv.into_values().collect(),
        environment,
        cwd_dev: number("cwd_dev")?,
        cwd_ino: number("cwd_ino")?,
        descriptors,
        no_new_privs: scalar("no_new_privs")?.to_owned(),
        tgid,
        pgid_is_self,
    })
}

/// [`try_parse_report`], failing the test with the exact reason for a refusal.
fn parse_report(bytes: &[u8]) -> Report {
    match try_parse_report(bytes) {
        Ok(report) => report,
        Err(why) => panic!(
            "the producer report was refused as {why:?}; the fixture and this parser must agree \
             on the report schema:\n{}",
            String::from_utf8_lossy(bytes)
        ),
    }
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

pub(crate) fn open_read_only(path: &Path) -> OwnedFd {
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

pub(crate) fn scratch_dir(name: &str) -> PathBuf {
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

/// One well-formed producer report, in exactly the shape the fixture emits.
fn well_formed_report() -> Vec<String> {
    vec![
        REPORT_MARKER.to_owned(),
        format!("name={REPORT_FIXTURE_NAME}"),
        "argv_count=2".to_owned(),
        "argv=0:61".to_owned(),
        "argv=1:62".to_owned(),
        "env_count=0".to_owned(),
        "cwd_dev=7".to_owned(),
        "cwd_ino=11".to_owned(),
        "fd_count=3".to_owned(),
        "fd=0:70697065".to_owned(),
        "fd=1:70697065".to_owned(),
        "fd=2:70697065".to_owned(),
        "no_new_privs=1".to_owned(),
        "sig_blk=0000000000000000".to_owned(),
        "sig_ign=0000000000000000".to_owned(),
        "sig_cgt=0000000000000000".to_owned(),
        "tgid=4242".to_owned(),
        "pgid_is_self=true".to_owned(),
        "end".to_owned(),
    ]
}

#[test]
fn the_producer_report_schema_is_closed_and_a_malformed_report_is_refused() {
    // P3R-10. The defect was a producer key and a parser key that disagreed,
    // which left a default value behind and made an assertion compare 0 to a
    // real pid. Both spellings are schema keys now, both are mandatory, and an
    // unknown key is refused — so the same class of disagreement cannot recur
    // silently.
    let render = |lines: &[String]| lines.join("\n").into_bytes();
    let without = |key: &str| -> Vec<String> {
        let prefix = format!("{key}=");
        well_formed_report()
            .into_iter()
            .filter(|line| !line.starts_with(&prefix))
            .collect()
    };
    let replacing = |key: &str, replacement: &str| -> Vec<String> {
        let prefix = format!("{key}=");
        well_formed_report()
            .into_iter()
            .map(|line| {
                if line.starts_with(&prefix) {
                    replacement.to_owned()
                } else {
                    line
                }
            })
            .collect()
    };
    let replacing_line = |old: &str, new: &str| -> Vec<String> {
        let found = well_formed_report().iter().any(|line| line == old);
        assert!(found, "the baseline report has no line `{old}`");
        well_formed_report()
            .into_iter()
            .map(|line| if line == old { new.to_owned() } else { line })
            .collect()
    };
    let plus = |extra: &str| -> Vec<String> {
        let mut lines = well_formed_report();
        lines.insert(lines.len() - 1, extra.to_owned());
        lines
    };

    // The positive control: the baseline really parses, so every refusal below
    // is caused by the one mutation that produced it.
    let baseline = try_parse_report(&render(&well_formed_report())).expect("the baseline parses");
    assert_eq!(baseline.name, REPORT_FIXTURE_NAME);
    assert_eq!(baseline.tgid, 4242);
    assert!(baseline.pgid_is_self);
    assert_eq!(baseline.argv, vec![b"a".to_vec(), b"b".to_vec()]);
    assert_eq!(
        baseline.descriptors.keys().copied().collect::<Vec<u32>>(),
        vec![0, 1, 2]
    );
    assert_eq!(baseline.cwd_dev, 7);
    assert_eq!(baseline.cwd_ino, 11);

    // A required key that never arrived.
    for key in [
        "tgid",
        "pgid_is_self",
        "name",
        "cwd_dev",
        "cwd_ino",
        "no_new_privs",
    ] {
        assert_eq!(
            try_parse_report(&render(&without(key))),
            Err(ReportRejection::Missing(key)),
            "a report without {key} was not refused"
        );
    }

    // A required value that does not parse.
    for (key, line) in [
        ("tgid", "tgid=not-a-number"),
        ("tgid", "tgid=-1"),
        ("tgid", "tgid="),
        ("pgid_is_self", "pgid_is_self=yes"),
        ("pgid_is_self", "pgid_is_self=1"),
        ("pgid_is_self", "pgid_is_self=True"),
        // Exactly what the fixture emits when it cannot determine its group: a
        // value the parser refuses rather than a plausible-looking guess.
        ("pgid_is_self", "pgid_is_self=unavailable"),
        ("cwd_dev", "cwd_dev=x"),
    ] {
        assert_eq!(
            try_parse_report(&render(&replacing(key, line))),
            Err(ReportRejection::Malformed(key)),
            "`{line}` was not refused"
        );
    }

    // A key the schema allows once, twice.
    for (key, line) in [
        ("tgid", "tgid=99"),
        ("pgid_is_self", "pgid_is_self=false"),
        ("name", "name=another"),
        ("argv", "argv=0:63"),
        ("fd", "fd=1:70697065"),
    ] {
        assert_eq!(
            try_parse_report(&render(&plus(line))),
            Err(ReportRejection::Duplicate(key)),
            "a duplicate {key} was not refused"
        );
    }

    // A key the schema does not define at all. This is the guard that makes a
    // producer/parser spelling disagreement impossible to miss.
    assert_eq!(
        try_parse_report(&render(&plus("pgid_is_leader=true"))),
        Err(ReportRejection::UnknownKey("pgid_is_leader".to_owned()))
    );
    assert_eq!(
        try_parse_report(&render(&plus("a line with no separator"))),
        Err(ReportRejection::UnknownKey(
            "a line with no separator".to_owned()
        ))
    );

    // A declared count that disagrees with the entries that arrived, and an argv
    // run with a hole in it.
    assert_eq!(
        try_parse_report(&render(&replacing("argv_count", "argv_count=3"))),
        Err(ReportRejection::Malformed("argv_count"))
    );
    assert_eq!(
        try_parse_report(&render(&replacing("fd_count", "fd_count=9"))),
        Err(ReportRejection::Malformed("fd_count"))
    );
    assert_eq!(
        try_parse_report(&render(&replacing("env_count", "env_count=1"))),
        Err(ReportRejection::Malformed("env_count"))
    );
    assert_eq!(
        try_parse_report(&render(&replacing_line("argv=1:62", "argv=2:62"))),
        Err(ReportRejection::Malformed("argv_count")),
        "an argv run with a hole in it was not refused"
    );

    // Malformed hexadecimal, or a malformed index, in a repeated entry.
    for line in ["argv=1:6", "argv=1:zz", "argv=one:62", "argv=1"] {
        assert_eq!(
            try_parse_report(&render(&replacing_line("argv=1:62", line))),
            Err(ReportRejection::Malformed("argv")),
            "`{line}` was not refused"
        );
    }
    for line in ["fd=2:7069706", "fd=two:70697065"] {
        assert_eq!(
            try_parse_report(&render(&replacing_line("fd=2:70697065", line))),
            Err(ReportRejection::Malformed("fd")),
            "`{line}` was not refused"
        );
    }

    // Structure: the wrong marker, no marker, no terminator, and output that is
    // not text at all.
    assert_eq!(
        try_parse_report(&render(&replacing_line(
            REPORT_MARKER,
            "HELM-LAUNCH-P3-FIXTURE/2"
        ))),
        Err(ReportRejection::Marker)
    );
    assert_eq!(try_parse_report(b""), Err(ReportRejection::Marker));
    assert_eq!(
        try_parse_report(b"not a marker at all\nend"),
        Err(ReportRejection::Marker)
    );
    assert_eq!(
        try_parse_report(
            &well_formed_report()
                .into_iter()
                .filter(|line| line != "end")
                .collect::<Vec<String>>()
                .join("\n")
                .into_bytes()
        ),
        Err(ReportRejection::Truncated)
    );
    assert_eq!(
        try_parse_report(&[0xff, 0xfe, 0xfd]),
        Err(ReportRejection::NotText)
    );

    // A report that could not stat its working directory carries `cwd_error`
    // instead of the identity, and is refused for the field it is missing.
    let with_cwd_error: Vec<String> = without("cwd_dev")
        .into_iter()
        .filter(|line| !line.starts_with("cwd_ino="))
        .chain(std::iter::once("cwd_error=denied".to_owned()))
        .collect();
    assert_eq!(
        try_parse_report(&render(&with_cwd_error)),
        Err(ReportRejection::Missing("cwd_dev"))
    );
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
    // Spawned rather than run through `output()`, so the harness knows the
    // identity the fixture is expected to report. Without that, `tgid` would be
    // "some number the producer printed" and the launcher test could not rely on
    // it (owner disposition of P3R-10).
    let running = Command::new(&fixture)
        .arg("self-test")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run the fixture directly");
    let observed_identity = running.id();
    let output = running
        .wait_with_output()
        .expect("collect the fixture output");
    assert!(
        output.status.success(),
        "the fixture failed its own run: {output:?}"
    );
    // The schema itself is validated here, before any launcher test consumes a
    // report: a refusal fails this test with its exact reason.
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
    // `tgid` is the fixture's own process identity and nothing else. Proven
    // against the pid this harness observed, so the launcher test's
    // direct-child assertion rests on a field that is known to be correct.
    assert_eq!(
        report.tgid, observed_identity,
        "the fixture does not report its own process identity"
    );
    // And `pgid_is_self` really tracks the process group: run directly the
    // fixture inherits this harness's group and is therefore NOT its own group
    // leader. If this were true here, the launched assertion would prove
    // nothing.
    assert!(
        !report.pgid_is_self,
        "run directly the fixture must not lead its own process group"
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
        i64::from(report.tgid),
        completed.launch.child.pid(),
        "the reporting image is not the direct child"
    );
    // A separate fact, separately asserted: the executed image leads its own
    // process group, whichever of the parent's `setpgid(child, child)` and the
    // child's `setpgid(0, 0)` ran first. The producer self-test proves this
    // field is `false` when the same fixture runs outside the backend, so `true`
    // here is a result rather than a constant.
    assert!(
        report.pgid_is_self,
        "the executed image does not lead its own process group"
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

/// The two group facts an ordinary launch really does establish: the executed
/// image leads its own process group, and P3 issues no group signal at all.
///
/// **`P3R-21`.** This test deliberately requires **neither** value of
/// `group_authority_established`, because on an ordinary, uncoordinated launch
/// that boolean is legitimately scheduler-dependent. It is the parent's own
/// `setpgid(child, child)` result and nothing else; the child's own
/// `setpgid(0, 0)` is stage 5 of its closed sequence, so when the child reaches
/// `execveat` before the parent's first post-`clone3` system call runs, Linux
/// answers the parent `EACCES`. Requiring it here treated a permitted
/// scheduling outcome as a defect, and that is what failed the hosted run
/// `35461333887` — on the same commit and runner image on which the same test
/// passed in another workflow.
///
/// The positive authority fact is **not** lost. It is asserted deterministically
/// by `injected::a_child_that_stalls_before_exec_is_bounded_killed_and_reaped`,
/// where the child is held before `execveat` and so cannot win that race.
///
/// **Process-group state and group-sweep authority are two separate facts, and
/// stay separate here.** `pgid_is_self` is an observation the executed image
/// makes about itself; it is evidence that the image leads its group, and it is
/// **not** evidence that the parent holds sweep authority. Nothing in this test
/// promotes one into the other.
#[test]
fn the_executed_image_leads_its_group_and_p3_issues_no_group_signal() {
    let fixture = report_fixture();
    let workdir = scratch_dir("group");
    let launch =
        launch_minimal(authorize_fixture(&fixture, &workdir, &[b"group"])).expect("launch");
    assert!(!launch.sigkill_sent, "a normal run needs no signal at all");
    let completed = complete(launch);
    // The producer self-test proves this field is `false` when the same fixture
    // runs outside the backend, so `true` here is a result rather than a
    // constant — whichever of the two `setpgid` calls happened to run first.
    let report = parse_report(&completed.stdout);
    assert!(
        report.pgid_is_self,
        "the executed image does not lead its own process group"
    );
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
        // `P3R-21`. The deterministic positive case for parent-side group-sweep
        // authority, and the reason the ordinary launch test does not need one.
        //
        // The stall injection runs after the child's own stage-5
        // `setpgid(0, 0)` and strictly before `execveat`, and it blocks in
        // `read` on descriptor 0 while the parent deliberately keeps the write
        // end open for exactly this mode. The child therefore cannot execute at
        // all until the parent's fixed pre-exec bound resolves it. The parent's
        // `setpgid(child, child)` is its first system call after `clone3`, so
        // here — unlike an uncoordinated launch — it cannot lose a race against
        // `execveat` and be answered `EACCES` by an already-executed child.
        //
        // Only that call's success sets the fact. Nothing is inferred from the
        // child's own `setpgid`, and P3 still issues no group signal.
        assert!(
            result.group_authority_established,
            "the child was held before exec, so the parent's setpgid(child, child) could not \
             have raced execveat; without its success no later slice could sweep"
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

/// Build the one `pthread_atfork` host-condition helper.
///
/// Deliberately **not** given the `P3R-20` treatment, because the race that
/// finding describes is not reachable here: this function has exactly one
/// caller, [`a_registered_pthread_atfork_handler_does_not_run_in_the_child`],
/// which is a single `#[test]` that cargo runs once on one thread and which
/// spawns no thread of its own. There is therefore no concurrent construction
/// path inside a test process, and across processes the pid in the staging name
/// already separates builders. Widening `P3R-20` into a general harness
/// refactor is out of its bounded scope; if a second caller is ever added, this
/// builder needs the same unique-namespace and no-replace publication that
/// [`fixture_binary`] now has.
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

// ===========================================================================
// 8. The fixture builder itself (P3R-20)
// ===========================================================================

/// A fixture with enough shape to need several code generation units and a real
/// link, so a concurrent build is a genuine compiler invocation rather than a
/// trivial one that finishes before it can overlap with anything.
const CONCURRENCY_FIXTURE_SOURCE: &str = r##"
fn shape(count: usize) -> String {
    let mut items: Vec<String> = Vec::new();
    for index in 0..count {
        items.push(format!("{index}"));
    }
    let mut out = String::new();
    for item in items.iter().rev() {
        out.push_str(item);
        out.push(',');
    }
    out
}

fn main() {
    let shaped = shape(8);
    assert!(shaped.starts_with("7,"), "{shaped}");
    print!("__MARKER__");
}
"##;

/// `P3R-20`. Several threads build the **same, previously uncached** fixture at
/// the same moment, through the real [`fixture_binary`] and a real `rustc`.
///
/// This is the regression for the defect that failed the first hosted Linux
/// publication run. Seventeen tests request the same content-addressed fixture,
/// cargo runs them in parallel threads of one process, and the builder's
/// temporary names were unique only by `std::process::id()` — which every one
/// of those threads shares. Two concurrent `rustc` invocations then wrote the
/// same `-o` path and destroyed each other's intermediate objects.
///
/// Overlap here is a synchronisation fact, not timing luck: the builders are
/// released by a [`Barrier`], and nothing sleeps.
#[test]
fn concurrent_builders_of_one_fixture_publish_exactly_one_object() {
    use std::os::unix::fs::MetadataExt as _;
    use std::sync::{Arc, Barrier};

    require_tool(
        "rustc",
        "the fixture-builder regression compiles a real fixture concurrently",
    );

    // The cache is content-addressed and `fixture_root()` outlives the run, so
    // the source has to differ per run or a warm cache would prove nothing.
    let marker = format!(
        "helm-launch-p3r20-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("a clock after the epoch")
            .as_nanos()
    );
    let source = CONCURRENCY_FIXTURE_SOURCE.replace("__MARKER__", &marker);
    let stem = format!("p3r20-{}", &hex(&Sha256::digest(source.as_bytes()))[..16]);
    let published = fixture_root().join(&stem);
    assert!(
        !published.exists(),
        "the regression needs a cold cache, and {published:?} already exists"
    );

    const BUILDERS: usize = 6;
    let before = FIXTURE_BUILD_NONCE.load(Ordering::Relaxed);
    let barrier = Arc::new(Barrier::new(BUILDERS));
    let mut builders = Vec::new();
    for _ in 0..BUILDERS {
        let barrier = Arc::clone(&barrier);
        let source = source.clone();
        builders.push(std::thread::spawn(move || {
            barrier.wait();
            let path = fixture_binary("p3r20", &source);
            let identity = fs::metadata(&path).expect("published fixture metadata");
            (path, identity.dev(), identity.ino())
        }));
    }
    // Every builder returns, so none panicked: no link failure, no missing
    // intermediate object, no failed staging rename.
    let observed: Vec<(PathBuf, u64, u64)> = builders
        .into_iter()
        .map(|builder| builder.join().expect("a concurrent fixture builder failed"))
        .collect();
    let compiled = FIXTURE_BUILD_NONCE.load(Ordering::Relaxed) - before;

    // The regression is only a regression if more than one builder really
    // reached the compiler. The barrier releases all six within microseconds
    // and a real `rustc` run is far longer than that, so this is a fact about
    // the released threads rather than about how fast the host is.
    assert!(
        compiled >= 2,
        "only {compiled} builder(s) reached rustc, so concurrent construction was not exercised"
    );

    // One path for every caller, and it is the content-addressed name.
    for (path, _, _) in &observed {
        assert_eq!(path, &published, "a builder returned a different fixture");
    }

    // One object, never replaced. A losing builder that had published over the
    // winner — as an unconditional `rename` would — would show a different
    // inode to the builders that returned before it.
    let (_, device, inode) = observed[0];
    for (_, observed_device, observed_inode) in &observed {
        assert_eq!(
            (*observed_device, *observed_inode),
            (device, inode),
            "the published fixture was replaced while builders were still running"
        );
    }
    let settled = fs::metadata(&published).expect("published fixture metadata");
    assert_eq!(
        (settled.dev(), settled.ino()),
        (device, inode),
        "the published fixture was replaced after the builders finished"
    );

    // Exactly one name refers to the published object, which is what proves
    // both that the winner released its staging name and that no loser left a
    // second link behind.
    assert_eq!(settled.nlink(), 1, "a staging link outlived its builder");

    // Executable, complete and actually runnable: a partially linked object
    // would not produce the marker.
    assert_eq!(
        settled.permissions().mode() & 0o111,
        0o111,
        "the published fixture is not executable"
    );
    let run = Command::new(&published)
        .output()
        .expect("run the published fixture");
    assert!(
        run.status.success(),
        "the published fixture did not run: {run:?}"
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        marker,
        "the published fixture is not the one that was built"
    );

    // Every builder removed the two names it owned. Six builders wrote six
    // distinct sources and six distinct staged objects into one shared
    // directory, so a surviving name would mean either a collision or a
    // builder reaching for a name that was not its own.
    let owned = format!("{stem}-");
    let leftovers: Vec<String> = fs::read_dir(fixture_root())
        .expect("read the fixture root")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|entry| {
            entry.starts_with(&owned) && (entry.ends_with(".rs") || entry.ends_with(".staging"))
        })
        .collect();
    assert!(
        leftovers.is_empty(),
        "fixture build temporaries were left behind: {leftovers:?}"
    );

    // This test owns the object it created, unlike the shared fixtures.
    let _ = fs::remove_file(&published);
}

// ===========================================================================
// 9. The direct-child drop guard and its P4 hand-off
// ===========================================================================
//
// `F-P4-02`. `ChildHandle`'s drop guard exists so that no path which
// **abandons** a child can leave one behind: it sends one `SIGKILL` through the
// pidfd and reaps within the fixed bound. It is armed by default and P3 never
// disarms it.
//
// The P4 lifecycle is not such a path. It performs the complete accepted
// direct-child sequence itself — at most one `SIGTERM`, at most one `SIGKILL`,
// one bounded post-kill observation and one non-blocking reap — and then hands
// the responsibility back. Leaving the guard armed there would send a second,
// **unrecorded** `SIGKILL` and could pay a second `POST_KILL_REAP_MS` inside
// `launch`, which the accepted total bound of plan section 8.6 does not
// contain.
//
// These cases pin both halves against a real child.

/// Sleep for `argv[1]` milliseconds and then exit 0. Nothing here installs a
/// handler, so the default disposition ends it.
const DROP_SLEEPER_SOURCE: &str = r##"
fn main() {
    let ms: u64 = std::env::args().nth(1).unwrap_or_default().parse().unwrap_or(60_000);
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
"##;

fn drop_sleeper() -> PathBuf {
    fixture_binary("p3-drop-sleeper", DROP_SLEEPER_SOURCE)
}

/// Is this pid still present in `/proc`? Test-only observation.
///
/// It answers **presence**, not liveness and not reaping: a child that was
/// killed and never waited for stays here as a zombie. Owner classification
/// `P4PUB-02` is exactly that confusion, so nothing below uses this as a reap
/// oracle.
fn pid_present(pid: i64) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

/// The bound on the harness's own cleanup. It is a **test** bound and has
/// nothing to do with `POST_KILL_REAP_MS`, which the product owns.
const HARNESS_CLEANUP_BOUND: Duration = Duration::from_secs(5);

/// Has the process this descriptor refers to already ended?
///
/// TEST HARNESS ONLY, and **non-consuming**: `NOWAIT` leaves the end available,
/// so asking the question cannot itself reap the child and cannot change what
/// the cleanup below then observes.
///
/// A refusal is returned rather than swallowed, because the two ways of failing
/// are different facts and the caller says so in its message: `Ok(true)` is a
/// child that was signalled and not collected, and `Err(ECHILD)` is one that
/// was signalled **and** collected, which is what a still-armed drop guard
/// would leave behind.
fn child_has_ended(probe: BorrowedFd<'_>) -> Result<bool, rustix::io::Errno> {
    rustix::process::waitid(
        rustix::process::WaitId::PidFd(probe),
        rustix::process::WaitIdOptions::EXITED
            | rustix::process::WaitIdOptions::NOHANG
            | rustix::process::WaitIdOptions::NOWAIT,
    )
    .map(|status| status.is_some())
}

/// Terminate **and reap** a child this test deliberately left running.
///
/// This is test-harness disposal of the harness's own descendant, performed
/// after the product semantics under test have already been observed. It is not
/// lifecycle behaviour and it is not a product path: `ChildHandle` is
/// unchanged, and no product code gains a wait of any kind.
///
/// Both halves are needed. `kill -9` ends the child, but a direct child that
/// nobody waits for remains a zombie — still listed in `/proc`, still holding a
/// slot — until its parent collects it. The reap is therefore the oracle, and
/// it is addressed by the harness's own duplicate of the child's process
/// descriptor rather than by a numeric pid, so the descriptor-based identity
/// the crate keeps everywhere is kept here too.
fn harness_terminate_and_reap(probe: BorrowedFd<'_>, pid: i64) -> Result<(), String> {
    let killed = Command::new("kill").arg("-9").arg(pid.to_string()).status();
    let started = Instant::now();
    loop {
        match rustix::process::waitid(
            rustix::process::WaitId::PidFd(probe),
            rustix::process::WaitIdOptions::EXITED | rustix::process::WaitIdOptions::NOHANG,
        ) {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(errno) => {
                return Err(format!(
                    "waiting for the harness's own child was refused with {errno}, so something \
                     else collected it; `kill -9` reported {killed:?}"
                ));
            }
        }
        if started.elapsed() >= HARNESS_CLEANUP_BOUND {
            return Err(format!(
                "the harness did not reap its own child within {HARNESS_CLEANUP_BOUND:?}; \
                 `kill -9` reported {killed:?}"
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn the_drop_guard_is_armed_by_default_and_cleans_up_an_abandoned_child() {
    // The P3 contract, unchanged: a handle nobody finished with kills and reaps
    // its child. Every internal early-return path depends on this.
    let workdir = scratch_dir("drop-armed");
    let launch = launch_minimal(authorize_fixture(
        &drop_sleeper(),
        &workdir,
        &[b"sleeper", b"60000"],
    ))
    .expect("launch");
    assert!(
        launch.child.drop_cleanup_armed(),
        "a fresh handle must own the cleanup responsibility"
    );
    let pid = launch.child.pid();
    assert!(pid_present(pid));

    let started = Instant::now();
    drop(launch);
    let elapsed = started.elapsed();

    assert!(
        !pid_present(pid),
        "an abandoned child survived its handle's drop"
    );
    assert!(
        elapsed < Duration::from_millis(POST_KILL_REAP_MS),
        "the armed drop took {elapsed:?}, which is not the bounded cleanup"
    );
}

#[test]
fn a_disarmed_drop_guard_neither_signals_nor_waits() {
    // The P4 hand-off. After the accepted lifecycle has finished, dropping the
    // handle must close its descriptors and do nothing else: no second
    // `SIGKILL`, and no second bounded wait inside `launch`.
    //
    // A live child is the sharpest shape: if the guard still fired, the child
    // would be gone, and if it still waited, the drop would take the whole
    // post-kill bound.
    //
    // Once those two facts are in hand the **harness** owns the surviving
    // child, and disposing of it is the harness's job: it terminates it and
    // then actually reaps it. Owner classification `P4PUB-02`: the first
    // publication killed the child and polled `/proc/<pid>` without ever
    // waiting for it, and a killed-but-unreaped direct child legitimately stays
    // visible in `/proc` as a zombie, so the cleanup could never finish. The
    // product behaviour under test was already correct when that happened.
    let workdir = scratch_dir("drop-disarmed");
    let launch = launch_minimal(authorize_fixture(
        &drop_sleeper(),
        &workdir,
        &[b"sleeper", b"60000"],
    ))
    .expect("launch");
    let pid = launch.child.pid();
    // A harness-owned duplicate of the child's process descriptor, taken while
    // the handle still holds one. The drop closes the handle's copy; this copy
    // is what lets the observation and the cleanup below stay
    // descriptor-addressed. It is an ordinary duplicate: it neither keeps the
    // child alive nor prevents it being reaped, and the handle cannot tell it
    // exists.
    let probe = launch
        .child
        .pidfd()
        .try_clone_to_owned()
        .expect("duplicate the child's process descriptor for the harness");
    assert!(launch.child.drop_cleanup_armed());
    launch.child.disarm_drop_cleanup_after_lifecycle();
    assert!(!launch.child.drop_cleanup_armed());

    let started = Instant::now();
    drop(launch);
    let elapsed = started.elapsed();

    // The two load-bearing observations, taken before anything is cleaned up.
    // The probe is non-consuming, so asking does not reap; and because it asks
    // whether the child **ended** rather than whether its pid is still listed,
    // a drop that killed the child without reaping it fails this case instead
    // of passing it on the zombie that would be left in `/proc`.
    let probed = child_has_ended(probe.as_fd());
    let cleanup = harness_terminate_and_reap(probe.as_fd(), pid);

    assert!(
        matches!(probed, Ok(false)),
        "a disarmed drop still signalled the child, so an unrecorded SIGKILL exists: the \
         non-consuming probe answered {probed:?}, where Ok(true) is a child that was signalled \
         and not collected and Err(ECHILD) one that was signalled and collected"
    );
    assert!(
        elapsed < Duration::from_millis(250),
        "a disarmed drop waited {elapsed:?}, so a second post-kill wait still exists"
    );
    if let Err(reason) = cleanup {
        panic!("the harness did not dispose of the child it deliberately left running: {reason}");
    }
    assert!(
        !pid_present(pid),
        "the harness reaped its child, so no trace of it may remain"
    );
}

#[test]
fn a_reaped_child_is_never_signalled_again_by_a_drop() {
    // The ordinary P4 shape: Phase C collected the end, so the handle is
    // already latched and the drop is a no-op whether or not it was disarmed.
    let workdir = scratch_dir("drop-reaped");
    let launch = launch_minimal(authorize_fixture(
        &drop_sleeper(),
        &workdir,
        &[b"sleeper", b"10"],
    ))
    .expect("launch");
    let end = launch
        .child
        .reap_within(Duration::from_secs(20))
        .expect("the fixture ends on its own");
    assert_eq!(end, ChildEnd::Exited { code: 0 });
    // Still armed — P3 never disarms — but already reaped, so the guard has
    // nothing to do and cannot signal a pid this handle no longer owns.
    assert!(launch.child.drop_cleanup_armed());

    let started = Instant::now();
    drop(launch);
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "dropping a reaped child waited, so it tried to signal and collect again"
    );
}
