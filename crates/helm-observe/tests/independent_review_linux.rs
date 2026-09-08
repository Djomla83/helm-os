//! Independent adversarial review tests for helm-observe 0.1 (Linux backend).
//!
//! Written by the independent reviewer of candidate
//! `e2a62081ece52391b30ede153eee139103c98e31`, not by the implementation author.
//! Every expected value here is derived from Accepted ADR-0022 and from the frozen
//! OBS-FS-01 definition, never from the implementation under test.
//!
//! Synthetic fixtures only. No A0 access, no Wine, no 7-Zip, no privilege change.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::io::{BufRead as _, BufReader};
use std::os::fd::{AsRawFd as _, OwnedFd};
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use helm_observe::{
    AdmissionErrorCode as A, BudgetReason, Failure, ObjectKind, Rejection, TargetOutcome,
    authorize, observe, parse_plan, proc_fd_from_trusted_current_process, root_from_fd,
};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";

fn temp_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("helm-observe-ir-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn dir_fd(path: &Path) -> OwnedFd {
    OwnedFd::from(fs::File::open(path).unwrap())
}

fn procfs() -> OwnedFd {
    dir_fd(Path::new("/proc/self/fd"))
}

fn plan_json(roots: &[&str], targets: &str) -> Vec<u8> {
    let roots: Vec<String> = roots
        .iter()
        .map(|r| format!("{{\"id\":\"{r}\"}}"))
        .collect();
    format!(
        "{{\"schema\":\"helm-observation-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"roots\":[{}],\"targets\":[{targets}]}}",
        roots.join(",")
    )
    .into_bytes()
}

fn file_target(id: &str, path: &str) -> String {
    format!(
        "{{\"id\":\"{id}\",\"root\":\"r\",\"path\":\"{path}\",\
         \"observable\":\"regular_file_sha256\"}}"
    )
}

fn run(root_dir: &Path, plan_bytes: &[u8]) -> Vec<TargetOutcome> {
    let plan = parse_plan(plan_bytes).unwrap();
    let root = root_from_fd("r", dir_fd(root_dir)).unwrap();
    let scope = authorize(
        plan,
        vec![root],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();
    observe(&scope)
        .record()
        .targets
        .iter()
        .map(|t| t.outcome)
        .collect()
}

/// Run one observation on a worker thread and fail instead of hanging forever.
/// A product that reached a data open on a writerless FIFO would block here.
fn run_with_deadline(root_dir: &Path, plan_bytes: &[u8], secs: u64) -> Vec<TargetOutcome> {
    let (tx, rx) = mpsc::channel();
    let dir = root_dir.to_owned();
    let plan = plan_bytes.to_vec();
    std::thread::spawn(move || {
        let _ = tx.send(run(&dir, &plan));
    });
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(v) => v,
        Err(_) => panic!("observation did not complete within {secs}s: it blocked on an object"),
    }
}

// ------------------------------------------------- foreign-process procfs attacks

/// Selects helper mode when this test binary is re-executed as a foreign process.
/// Shell redirection cannot express the attack portably, because POSIX shells are
/// only required to honour single-digit descriptor numbers, so the reviewer
/// re-executes this same binary instead.
const HELPER_ENV: &str = "HELM_OBSERVE_REVIEW_HELPER";
/// How many descriptors a foreign helper holds open.
const HOARD_COUNT: usize = 320;

/// Foreign-helper entry point. It does nothing at all in an ordinary test run and
/// only becomes a helper when the reviewer sets `HELM_OBSERVE_REVIEW_HELPER`.
#[test]
fn foreign_helper_process_entry_point() {
    let Ok(mode) = std::env::var(HELPER_ENV) else {
        return;
    };
    let path = match mode.strip_prefix("shared:") {
        Some(shared) => shared.to_owned(),
        None => "/proc/self/fd".to_owned(),
    };
    let mut held: Vec<fs::File> = Vec::new();
    for _ in 0..HOARD_COUNT {
        match fs::File::open(&path) {
            Ok(f) => held.push(f),
            Err(_) => break,
        }
    }
    let highest = held.iter().map(|f| f.as_raw_fd()).max().unwrap_or(0);
    println!("HELPER-READY held={} highest={highest}", held.len());
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
    std::thread::sleep(Duration::from_secs(120));
}

/// Re-execute this test binary as a foreign process holding `HOARD_COUNT`
/// descriptors on `mode`'s object, and wait until it says it is ready.
fn spawn_foreign_helper(mode: &str) -> Child {
    let exe = std::env::current_exe().unwrap();
    let mut child = Command::new(exe)
        .args([
            "--exact",
            "foreign_helper_process_entry_point",
            "--nocapture",
        ])
        .env(HELPER_ENV, mode)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let out = child.stdout.take().unwrap();
    let mut reader = BufReader::new(out);
    for _ in 0..40 {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if let Some(rest) = line.trim().strip_prefix("HELPER-READY") {
            assert!(
                rest.contains(&format!("held={HOARD_COUNT}")),
                "foreign helper could not hold enough descriptors:{rest}"
            );
            return child;
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("foreign helper never reported readiness");
}

/// A foreign process that saturates its low descriptor numbers with its **own**
/// `/proc/self/fd` directory. Every one of those numeric names then resolves,
/// through `/proc/<child>/fd/<n>`, to the very directory the caller supplied, so a
/// self-identity probe that reuses the supplied capability as its own probe object
/// sees a matching device and inode.
fn spawn_self_fd_hoarder() -> Child {
    spawn_foreign_helper("self_fd")
}

/// A foreign process holding one shared regular file at the same descriptor
/// numbers, standing in for descriptors inherited from the caller.
fn spawn_shared_object_hoarder(shared: &Path) -> Child {
    spawn_foreign_helper(&format!("shared:{}", shared.display()))
}

fn foreign_fd_dir(child: &Child) -> OwnedFd {
    let path = format!("/proc/{}/fd", child.id());
    OwnedFd::from(fs::File::open(&path).unwrap_or_else(|e| panic!("open {path}: {e}")))
}

/// ADR-0022 admits only a **trusted current-process** procfs capability. A foreign
/// process's descriptor directory must be refused however that process arranges
/// its own descriptor table.
#[test]
fn procfs_admission_refuses_a_foreign_self_fd_hoarding_process() {
    let mut child = spawn_self_fd_hoarder();
    let cap = foreign_fd_dir(&child);
    let number = cap.as_raw_fd();
    let result = proc_fd_from_trusted_current_process(cap);
    let _ = child.kill();
    let _ = child.wait();
    assert!(
        number >= 0 && (number as usize) < HOARD_COUNT,
        "capability landed at fd {number}, outside the attacked range; test is inconclusive"
    );
    match result {
        Err(e) => assert_eq!(e.code(), A::ProcfsForeignOrUnusable),
        Ok(_) => panic!(
            "a foreign process descriptor directory was admitted as the trusted \
             current-process procfs capability (probe descriptor {number})"
        ),
    }
}

/// The same attack where the foreign process instead holds an object the caller
/// also holds, standing in for an inherited descriptor.
#[test]
fn procfs_admission_refuses_a_foreign_process_sharing_the_callers_objects() {
    let base = temp_root("procfs-shared");
    let shared = base.join("shared");
    fs::write(&shared, b"shared object").unwrap();
    let _held = fs::File::open(&shared).unwrap();

    let mut child = spawn_shared_object_hoarder(&shared);
    let cap = foreign_fd_dir(&child);
    let result = proc_fd_from_trusted_current_process(cap);
    let _ = child.kill();
    let _ = child.wait();
    match result {
        Err(e) => assert_eq!(e.code(), A::ProcfsForeignOrUnusable),
        Ok(_) => panic!("a foreign descriptor directory was admitted"),
    }
}

/// A descriptor directory whose process has exited must fail admission, and must
/// never fall back to a pathname.
#[test]
fn procfs_admission_refuses_a_dead_foreign_process() {
    let mut child = spawn_self_fd_hoarder();
    let cap = foreign_fd_dir(&child);
    let _ = child.kill();
    let _ = child.wait();
    match proc_fd_from_trusted_current_process(cap) {
        Err(e) => assert_eq!(e.code(), A::ProcfsForeignOrUnusable),
        Ok(_) => panic!("an exited process descriptor directory was admitted"),
    }
}

/// A genuine tmpfs decoy, not merely an ext4 one.
#[test]
fn procfs_admission_refuses_a_tmpfs_decoy() {
    let shm = Path::new("/dev/shm");
    if !shm.is_dir() {
        return;
    }
    let decoy = shm.join(format!("helm-observe-ir-decoy-{}", std::process::id()));
    let _ = fs::remove_dir_all(&decoy);
    fs::create_dir_all(&decoy).unwrap();
    fs::write(decoy.join("3"), b"decoy").unwrap();
    let err = proc_fd_from_trusted_current_process(dir_fd(&decoy)).unwrap_err();
    let _ = fs::remove_dir_all(&decoy);
    assert_eq!(err.code(), A::ProcfsWrongFilesystem);
}

/// The genuine current-process capability still works after the attacks above.
#[test]
fn procfs_admission_accepts_the_real_current_process_capability() {
    assert!(proc_fd_from_trusted_current_process(procfs()).is_ok());
    // An unprivileged runner is normally refused `/proc/1/fd` by the kernel, in
    // which case the crate never sees it and the case proves nothing either way.
    if let Ok(init) = fs::File::open("/proc/1/fd") {
        match proc_fd_from_trusted_current_process(OwnedFd::from(init)) {
            Err(e) => assert_eq!(e.code(), A::ProcfsForeignOrUnusable),
            Ok(_) => panic!("/proc/1/fd was admitted"),
        }
    }
}

// ------------------------------------------------------------ special files

/// A FIFO whose writer is blocked in `open(2)`. If the observer ever opened the
/// FIFO for data the writer would be released and would print `opened`. It must
/// stay blocked, and the observation must not block either.
#[test]
fn fifo_with_a_blocked_writer_is_classified_without_releasing_the_writer() {
    let base = temp_root("fifo-writer");
    let fifo = base.join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success(),
        "mkfifo must succeed unprivileged"
    );

    let mut writer = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "exec 3>{}\necho opened\nexec sleep 60\n",
            fifo.display()
        ))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // Give the writer time to reach its blocking open.
    std::thread::sleep(Duration::from_millis(500));

    let plan = plan_json(&["r"], &file_target("fifo", "fifo"));
    let out = run_with_deadline(&base, &plan, 20);

    assert_eq!(
        out[0],
        TargetOutcome::Rejected {
            rejection: Rejection::SpecialFile,
            observed_kind: Some(ObjectKind::Fifo)
        }
    );
    let still_blocked = writer.try_wait().unwrap().is_none();
    // Control: release the writer ourselves and confirm it really was waiting.
    let mut released = String::new();
    {
        let reader = fs::File::open(&fifo).unwrap();
        std::thread::sleep(Duration::from_millis(500));
        if let Some(o) = writer.stdout.take() {
            let _ = BufReader::new(o).read_line(&mut released);
        }
        drop(reader);
    }
    let _ = writer.kill();
    let _ = writer.wait();
    assert!(
        still_blocked,
        "the observer released a blocked FIFO writer, so it opened the FIFO for data"
    );
    assert_eq!(
        released.trim(),
        "opened",
        "control: the writer really was waiting in open(2)"
    );
}

/// A writerless FIFO and a bound Unix socket, under a deadline.
#[test]
fn writerless_fifo_and_socket_never_block_and_never_open_data() {
    let base = temp_root("special");
    let fifo = base.join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let sock = base.join("sock");
    let _listener = std::os::unix::net::UnixListener::bind(&sock).unwrap();

    let plan = plan_json(
        &["r"],
        &format!(
            "{},{}",
            file_target("fifo", "fifo"),
            file_target("sock", "sock")
        ),
    );
    let out = run_with_deadline(&base, &plan, 20);
    assert_eq!(
        out[0],
        TargetOutcome::Rejected {
            rejection: Rejection::SpecialFile,
            observed_kind: Some(ObjectKind::Fifo)
        }
    );
    assert_eq!(
        out[1],
        TargetOutcome::Rejected {
            rejection: Rejection::SpecialFile,
            observed_kind: Some(ObjectKind::Socket)
        }
    );
}

// ---------------------------------------------------------- errno discipline

/// Absence must come only from a legitimate constrained lookup miss.
#[test]
fn only_lookup_absence_produces_absent() {
    let base = temp_root("errno");
    fs::write(base.join("real"), b"x").unwrap();
    fs::create_dir_all(base.join("dir")).unwrap();
    let denied = base.join("denied");
    fs::write(&denied, b"secret").unwrap();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
    std::os::unix::fs::symlink("real", base.join("link")).unwrap();
    fs::create_dir_all(base.join("realdir")).unwrap();
    fs::write(base.join("realdir/child"), b"c").unwrap();
    std::os::unix::fs::symlink("realdir", base.join("linkdir")).unwrap();

    let plan = plan_json(
        &["r"],
        &[
            file_target("missing", "nope"),
            file_target("missingparent", "nodir/leaf"),
            file_target("notdir", "real/child"),
            file_target("denied", "denied"),
            file_target("trailinglink", "link"),
            file_target("nonfinallink", "linkdir/child"),
            file_target("wrongkind", "dir"),
        ]
        .join(","),
    );
    let out = run(&base, &plan);
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o600)).unwrap();

    assert_eq!(out[0], TargetOutcome::Absent);
    assert_eq!(out[1], TargetOutcome::Absent);
    for (i, label) in [
        (2, "ENOTDIR"),
        (3, "EACCES"),
        (4, "trailing symlink"),
        (5, "non-final symlink"),
        (6, "wrong kind"),
    ] {
        assert_ne!(out[i], TargetOutcome::Absent, "{label} must not be absence");
    }
    assert!(matches!(
        out[2],
        TargetOutcome::Rejected {
            rejection: Rejection::WrongKind,
            ..
        }
    ));
    assert!(matches!(
        out[3],
        TargetOutcome::Failed {
            failure: Failure::PermissionDenied,
            ..
        }
    ));
    assert_eq!(
        out[4],
        TargetOutcome::Rejected {
            rejection: Rejection::SymlinkForbidden,
            observed_kind: Some(ObjectKind::Symlink)
        },
        "Amendment 1: a trailing symlink is pinned and rejected at classification"
    );
    assert_eq!(
        out[5],
        TargetOutcome::Rejected {
            rejection: Rejection::SymlinkForbidden,
            observed_kind: None
        },
        "Amendment 1: a non-final symlink is rejected at resolution, with no descriptor"
    );
}

/// A permission denial reached at the procfs reopen stage, after a successful pin
/// and classification, must be `permission_denied` with no bytes read.
#[test]
fn reopen_permission_denial_reports_no_bytes_and_is_not_absence() {
    let base = temp_root("reopen-denied");
    let f = base.join("unreadable");
    fs::write(&f, b"unreadable payload").unwrap();
    fs::set_permissions(&f, fs::Permissions::from_mode(0o000)).unwrap();
    let out = run(&base, &plan_json(&["r"], &file_target("t", "unreadable")));
    fs::set_permissions(&f, fs::Permissions::from_mode(0o600)).unwrap();
    match out[0] {
        TargetOutcome::Failed {
            failure: Failure::PermissionDenied,
            partial_bytes,
            ..
        } => assert_eq!(partial_bytes, Some(0), "no byte may be read after denial"),
        ref other => panic!("expected permission_denied, got {other:?}"),
    }
}

// ------------------------------------------------------------------ budgets

/// Metadata over the per-file ceiling is refused on metadata alone, and the
/// boundary is exact: ceiling+1 is refused, and the batch is not exhausted.
#[test]
fn per_file_ceiling_boundary_is_exact_and_does_not_exhaust_the_batch() {
    let base = temp_root("budget-boundary");
    let over = base.join("over");
    fs::File::create(&over)
        .unwrap()
        .set_len(helm_observe::MAX_FILE_BYTES + 1)
        .unwrap();
    fs::write(base.join("small"), b"small").unwrap();

    let plan = plan_json(
        &["r"],
        &format!(
            "{},{}",
            file_target("over", "over"),
            file_target("small", "small")
        ),
    );
    let out = run(&base, &plan);
    assert_eq!(out[0], TargetOutcome::NotObserved(BudgetReason::FileLimit));
    assert!(
        matches!(out[1], TargetOutcome::ObservedFile(_)),
        "a per-file refusal must not exhaust the aggregate budget, got {:?}",
        out[1]
    );
}

/// Aggregate accounting: after one exactly-ceiling file the remaining budget can
/// no longer admit another, and every later target still receives an explicit
/// outcome. Heavy: streams one 512 MiB sparse file, so it is opt-in.
#[test]
#[ignore = "streams 512 MiB; run explicitly in release mode"]
fn aggregate_ceiling_is_exact_and_later_targets_are_explicit() {
    let base = temp_root("budget-aggregate");
    for name in ["a", "b"] {
        fs::File::create(base.join(name))
            .unwrap()
            .set_len(helm_observe::MAX_FILE_BYTES)
            .unwrap();
    }
    fs::write(base.join("tail"), b"tail").unwrap();
    let plan = plan_json(
        &["r"],
        &[
            file_target("a", "a"),
            file_target("b", "b"),
            file_target("tail", "tail"),
        ]
        .join(","),
    );
    let out = run(&base, &plan);
    match out[0] {
        TargetOutcome::ObservedFile(f) => {
            assert_eq!(f.bytes_read, helm_observe::MAX_FILE_BYTES);
        }
        ref other => panic!("an exactly-ceiling file must be observed, got {other:?}"),
    }
    assert_eq!(
        out[1],
        TargetOutcome::NotObserved(BudgetReason::TotalLimit),
        "the aggregate ceiling must refuse the second ceiling-sized file"
    );
    assert_eq!(
        out[2],
        TargetOutcome::NotObserved(BudgetReason::NotAttemptedTotalLimit),
        "every later target still gets an explicit outcome"
    );
}

// --------------------------------------------------------- resource lifetime

/// Repeated observations must not leak descriptors on the success, rejection,
/// failure or absence paths.
#[test]
fn repeated_observations_leak_no_descriptors() {
    let base = temp_root("fd-leak");
    fs::write(base.join("f"), b"payload").unwrap();
    fs::create_dir_all(base.join("d")).unwrap();
    let fifo = base.join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    std::os::unix::fs::symlink("f", base.join("link")).unwrap();
    let denied = base.join("denied");
    fs::write(&denied, b"x").unwrap();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();

    let plan = plan_json(
        &["r"],
        &[
            file_target("f", "f"),
            file_target("fifo", "fifo"),
            file_target("link", "link"),
            file_target("denied", "denied"),
            file_target("missing", "nope"),
        ]
        .join(","),
    );

    let count = || {
        fs::read_dir("/proc/self/fd")
            .map(Iterator::count)
            .unwrap_or(0)
    };
    for _ in 0..3 {
        let _ = run(&base, &plan);
    }
    let before = count();
    for _ in 0..25 {
        let _ = run(&base, &plan);
    }
    let after = count();
    fs::set_permissions(&denied, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(
        after <= before,
        "descriptor count grew from {before} to {after} over 25 observations"
    );
}

// ------------------------------------------------------- cohort identification

/// Records what the runner fixture filesystem actually is, from primary kernel
/// sources, without weakening admission and without asserting an ext4 identity
/// that the admission mechanism cannot establish.
#[test]
fn fixture_filesystem_cohort_is_reported_from_primary_sources() {
    let base = temp_root("cohort");
    let magic = Command::new("stat")
        .args(["-f", "-c", "%t"])
        .arg(&base)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default();
    // /proc/self/mountinfo carries the mounted filesystem *type name*, which is
    // strictly more information than the 0xEF53 superblock magic. It still does
    // not prove an on-disk ext4 feature set.
    let mountinfo = fs::read_to_string("/proc/self/mountinfo").unwrap_or_default();
    let mut best = (String::new(), 0usize);
    for line in mountinfo.lines() {
        let Some((left, right)) = line.split_once(" - ") else {
            continue;
        };
        let Some(mount_point) = left.split_whitespace().nth(4) else {
            continue;
        };
        let Some(fstype) = right.split_whitespace().next() else {
            continue;
        };
        if base.starts_with(mount_point) && mount_point.len() >= best.1 {
            best = (fstype.to_owned(), mount_point.len());
        }
    }
    let admitted = root_from_fd("r", dir_fd(&base));
    println!(
        "HELM-OBSERVE-INDEPENDENT-COHORT: magic=0x{magic} mounted_fstype={} admitted={} arch={}",
        if best.0.is_empty() {
            "unknown"
        } else {
            &best.0
        },
        admitted.is_ok(),
        std::env::consts::ARCH
    );
    if magic.eq_ignore_ascii_case("ef53") {
        assert!(
            admitted.is_ok(),
            "an ext-family (0xEF53) root must be admitted"
        );
    } else {
        assert_eq!(
            admitted.unwrap_err().code(),
            A::RootUnsupportedFilesystem,
            "a non-ext-family filesystem must be refused, never silently accepted"
        );
    }
}
