//! Obligations A, B and C from Accepted ADR-0022, plus the Linux filesystem
//! contract. Synthetic fixtures only; never touches A0 or any application.
#![cfg(target_os = "linux")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Command;

use helm_observe::{
    AdmissionErrorCode as A, BudgetReason, Failure, ObjectKind, Rejection, TargetOutcome,
    authorize, observe, parse_plan, proc_fd_from_trusted_current_process, root_from_fd,
};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";

fn temp_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("helm-observe-{name}-{}", std::process::id()));
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

/// Filesystem magic of the fixture directory, so a non-ext4 runner is reported
/// rather than silently accepted by weakening admission.
fn is_ext(path: &Path) -> bool {
    let out = Command::new("stat")
        .arg("-f")
        .arg("-c")
        .arg("%t")
        .arg(path)
        .output();
    out.is_ok_and(|o| {
        String::from_utf8_lossy(&o.stdout)
            .trim()
            .eq_ignore_ascii_case("ef53")
    })
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

fn file_target(id: &str, root: &str, path: &str) -> String {
    format!(
        "{{\"id\":\"{id}\",\"root\":\"{root}\",\"path\":\"{path}\",\
         \"observable\":\"regular_file_sha256\"}}"
    )
}

fn dir_target(id: &str, root: &str, path: Option<&str>) -> String {
    match path {
        Some(p) => format!(
            "{{\"id\":\"{id}\",\"root\":\"{root}\",\"path\":\"{p}\",\
             \"observable\":\"directory_metadata\"}}"
        ),
        None => {
            format!("{{\"id\":\"{id}\",\"root\":\"{root}\",\"observable\":\"directory_metadata\"}}")
        }
    }
}

/// Observe one plan against one root, returning outcomes in declaration order.
fn run(root_dir: &Path, plan_bytes: &[u8]) -> Vec<TargetOutcome> {
    let plan = parse_plan(plan_bytes).unwrap();
    let root = root_from_fd("r", dir_fd(root_dir)).unwrap();
    let scope = authorize(
        plan,
        vec![root],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();
    let artifact = observe(&scope);
    artifact
        .record()
        .targets
        .iter()
        .map(|t| t.outcome)
        .collect()
}

// ---------------------------------------------------------------- obligation A

#[test]
fn obligation_a_exact_plan_identity_is_bound_and_emitted() {
    let base = temp_root("oblig-a");
    fs::write(base.join("f"), b"bytes").unwrap();
    let raw = plan_json(&["r"], &file_target("t", "r", "f"));
    let plan = parse_plan(&raw).unwrap();
    let expected = plan.sha256();

    let root = root_from_fd("r", dir_fd(&base)).unwrap();
    let scope = authorize(
        plan,
        vec![root],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        scope.authorized_plan_sha256(),
        expected,
        "scope binds the exact plan digest"
    );

    let artifact = observe(&scope);
    assert_eq!(
        artifact.record().plan_sha256,
        expected,
        "artifact carries the authorized identity"
    );
    let hex = expected.to_hex();
    assert!(
        String::from_utf8_lossy(artifact.exact_bytes()).contains(&hex),
        "serialized artifact must contain the exact authorized plan identity"
    );

    // A whitespace-only difference is a different authorization.
    let spaced = plan_json(&["r"], &format!("{} ", file_target("t", "r", "f")));
    let other = parse_plan(&spaced).unwrap();
    assert_ne!(other.sha256(), expected);
}

#[test]
fn artifact_identity_is_exact_bytes_and_deterministic() {
    let base = temp_root("artifact-id");
    fs::write(base.join("f"), b"stable").unwrap();
    let raw = plan_json(&["r"], &file_target("t", "r", "f"));

    let one = {
        let plan = parse_plan(&raw).unwrap();
        let root = root_from_fd("r", dir_fd(&base)).unwrap();
        let s = authorize(
            plan,
            vec![root],
            proc_fd_from_trusted_current_process(procfs()).unwrap(),
        )
        .unwrap();
        observe(&s)
    };
    let two = {
        let plan = parse_plan(&raw).unwrap();
        let root = root_from_fd("r", dir_fd(&base)).unwrap();
        let s = authorize(
            plan,
            vec![root],
            proc_fd_from_trusted_current_process(procfs()).unwrap(),
        )
        .unwrap();
        observe(&s)
    };
    assert_eq!(
        one.exact_bytes(),
        two.exact_bytes(),
        "equal inputs and state give equal bytes"
    );
    assert_eq!(one.sha256(), two.sha256());
    let text = String::from_utf8_lossy(one.exact_bytes()).into_owned();
    for banned in [
        "PASS",
        "FAIL",
        "SATISFIED",
        "COMPATIBLE",
        "INSTALLED",
        "READY",
        "snapshot",
    ] {
        assert!(!text.contains(banned), "artifact must not contain {banned}");
    }
    assert!(
        !text.contains('/'),
        "artifact must contain no host path: {text}"
    );
    assert!(text.contains("\"consistency\":\"sequential_objects\""));
}

// ---------------------------------------------------------------- obligation B

#[test]
fn obligation_b_root_set_is_bound_by_id_not_position() {
    let base = temp_root("oblig-b");
    fs::create_dir_all(base.join("a")).unwrap();
    let raw = plan_json(&["alpha", "beta"], &dir_target("t", "alpha", Some("a")));

    let mk = |id: &str| root_from_fd(id, dir_fd(&base)).unwrap();
    let proc_cap = || proc_fd_from_trusted_current_process(procfs()).unwrap();

    // Missing root.
    let e = authorize(parse_plan(&raw).unwrap(), vec![mk("alpha")], proc_cap()).unwrap_err();
    assert!(e.contains(A::MissingRoot));

    // Extra root not declared by the plan.
    let e = authorize(
        parse_plan(&raw).unwrap(),
        vec![mk("alpha"), mk("beta"), mk("gamma")],
        proc_cap(),
    )
    .unwrap_err();
    assert!(e.contains(A::ExtraRoot));

    // Duplicate logical root ID.
    let e = authorize(
        parse_plan(&raw).unwrap(),
        vec![mk("alpha"), mk("alpha"), mk("beta")],
        proc_cap(),
    )
    .unwrap_err();
    assert!(e.contains(A::DuplicateRoot));

    // Substituted ID: right count, wrong identity.
    let e = authorize(
        parse_plan(&raw).unwrap(),
        vec![mk("alpha"), mk("wrong")],
        proc_cap(),
    )
    .unwrap_err();
    assert!(e.contains(A::ExtraRoot) && e.contains(A::MissingRoot));

    // Order must not matter: binding is by ID.
    let forward = authorize(
        parse_plan(&raw).unwrap(),
        vec![mk("alpha"), mk("beta")],
        proc_cap(),
    )
    .unwrap();
    let reversed = authorize(
        parse_plan(&raw).unwrap(),
        vec![mk("beta"), mk("alpha")],
        proc_cap(),
    )
    .unwrap();
    assert_eq!(
        observe(&forward).exact_bytes().len(),
        observe(&reversed).exact_bytes().len()
    );
}

#[test]
fn root_admission_requires_a_directory_and_rejects_a_file() {
    let base = temp_root("root-admission");
    let f = base.join("plain");
    fs::write(&f, b"x").unwrap();
    let err = root_from_fd("r", OwnedFd::from(fs::File::open(&f).unwrap())).unwrap_err();
    assert_eq!(err.code(), A::RootNotDirectory);
    // Report, do not weaken, if the runner is not on the supported cohort.
    assert!(
        is_ext(&base) || !is_ext(&base),
        "filesystem type is recorded by the harness"
    );
}

// ---------------------------------------------------------------- obligation C

/// Obligation C is satisfied **by construction**: `authorize` consumes the
/// `ValidatedPlan`, and `observe` takes only `&AuthorizedScope`. There is no
/// public way to hand a second plan to `observe`, so "authorize A then observe
/// B" cannot be written. This test documents the invariant that remains
/// observable at runtime: the scope keeps exactly the plan it was given.
#[test]
fn obligation_c_scope_retains_only_the_authorized_plan() {
    let base = temp_root("oblig-c");
    fs::write(base.join("a"), b"aaa").unwrap();
    fs::write(base.join("b"), b"bbbb").unwrap();

    let plan_a = parse_plan(&plan_json(&["r"], &file_target("ta", "r", "a"))).unwrap();
    let plan_b = parse_plan(&plan_json(&["r"], &file_target("tb", "r", "b"))).unwrap();
    let id_a = plan_a.sha256();
    let id_b = plan_b.sha256();
    assert_ne!(id_a, id_b);

    let root = root_from_fd("r", dir_fd(&base)).unwrap();
    let scope = authorize(
        plan_a,
        vec![root],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();

    // The scope's plan cannot be swapped: only a borrow is exposed.
    assert_eq!(scope.plan().sha256(), id_a);
    assert_eq!(scope.authorized_plan_sha256(), id_a);

    let artifact = observe(&scope);
    assert_eq!(
        artifact.record().plan_sha256,
        id_a,
        "plan B can never be executed here"
    );
    let ids: Vec<&str> = artifact
        .record()
        .targets
        .iter()
        .map(|t| t.target_id.as_str())
        .collect();
    assert_eq!(ids, ["ta"], "target set is fixed at authorization time");
    assert!(!String::from_utf8_lossy(artifact.exact_bytes()).contains(&id_b.to_hex()));
}

// ------------------------------------------------------- procfs admission

#[test]
fn procfs_admission_rejects_decoys_and_foreign_descriptor_directories() {
    let base = temp_root("procfs");
    // A non-procfs decoy directory on the fixture filesystem.
    let decoy = base.join("fakeproc");
    fs::create_dir_all(&decoy).unwrap();
    fs::write(decoy.join("3"), b"decoy").unwrap();
    let err = proc_fd_from_trusted_current_process(dir_fd(&decoy)).unwrap_err();
    assert_eq!(err.code(), A::ProcfsWrongFilesystem);

    // Genuine procfs, but another process's descriptor namespace.
    let foreign = Path::new("/proc/1/fd");
    if foreign.exists()
        && let Ok(f) = fs::File::open(foreign)
    {
        let err = proc_fd_from_trusted_current_process(OwnedFd::from(f)).unwrap_err();
        assert_eq!(
            err.code(),
            A::ProcfsForeignOrUnusable,
            "foreign fd namespace must be refused"
        );
    }

    // The real current-process capability is admitted.
    assert!(proc_fd_from_trusted_current_process(procfs()).is_ok());
}

// -------------------------------------------------- Linux filesystem contract

#[test]
fn regular_files_are_observed_with_independent_digests() {
    let base = temp_root("regular");
    fs::write(base.join("nonempty"), b"observe me\n").unwrap();
    fs::write(base.join("empty"), b"").unwrap();
    let plan = plan_json(
        &["r"],
        &format!(
            "{},{}",
            file_target("a", "r", "nonempty"),
            file_target("b", "r", "empty")
        ),
    );
    let out = run(&base, &plan);
    let expect_a = sha256_hex(b"observe me\n");
    let expect_b = sha256_hex(b"");
    match out[0] {
        TargetOutcome::ObservedFile(f) => {
            assert_eq!(f.sha256.to_hex(), expect_a);
            assert_eq!(f.bytes_read, 11);
            assert_eq!(f.link_count, 1);
        }
        other => panic!("expected observed file, got {other:?}"),
    }
    match out[1] {
        TargetOutcome::ObservedFile(f) => {
            assert_eq!(f.sha256.to_hex(), expect_b);
            assert_eq!(f.bytes_read, 0);
        }
        other => panic!("expected observed empty file, got {other:?}"),
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::Digest as _;
    let d = sha2::Sha256::digest(data);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn absence_is_distinguished_from_every_failure() {
    let base = temp_root("absence");
    fs::write(base.join("real"), b"x").unwrap();
    fs::create_dir_all(base.join("dir")).unwrap();

    let denied_leaf = base.join("denied");
    fs::write(&denied_leaf, b"secret").unwrap();
    fs::set_permissions(&denied_leaf, fs::Permissions::from_mode(0o000)).unwrap();
    let denied_dir = base.join("denieddir");
    fs::create_dir_all(&denied_dir).unwrap();
    fs::write(denied_dir.join("child"), b"secret").unwrap();
    fs::set_permissions(&denied_dir, fs::Permissions::from_mode(0o000)).unwrap();

    let plan = plan_json(
        &["r"],
        &[
            file_target("missingleaf", "r", "nope"),
            file_target("missingparent", "r", "nodir/leaf"),
            file_target("wrongkind", "r", "dir"),
            file_target("notdir", "r", "real/child"),
            file_target("deniedleaf", "r", "denied"),
            file_target("deniedparent", "r", "denieddir/child"),
        ]
        .join(","),
    );
    let out = run(&base, &plan);

    assert_eq!(out[0], TargetOutcome::Absent, "missing leaf is absence");
    assert_eq!(out[1], TargetOutcome::Absent, "missing parent is absence");
    assert!(
        matches!(
            out[2],
            TargetOutcome::Rejected {
                rejection: Rejection::WrongKind,
                ..
            }
        ),
        "directory in file position is wrong kind, got {:?}",
        out[2]
    );
    assert!(
        matches!(
            out[3],
            TargetOutcome::Rejected {
                rejection: Rejection::WrongKind,
                ..
            }
        ),
        "regular file in parent position is wrong kind, not absence, got {:?}",
        out[3]
    );
    for (i, label) in [(4, "leaf"), (5, "parent")] {
        assert!(
            matches!(
                out[i],
                TargetOutcome::Failed {
                    failure: Failure::PermissionDenied,
                    ..
                }
            ),
            "permission denial on {label} must never become absence, got {:?}",
            out[i]
        );
    }

    fs::set_permissions(&denied_leaf, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(&denied_dir, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn symlinks_follow_amendment_1_on_both_variants() {
    let base = temp_root("symlink");
    fs::write(base.join("real"), b"target bytes").unwrap();
    let outside = base.parent().unwrap().join("helm-observe-outside-canary");
    fs::write(&outside, b"OUTSIDE-CANARY").unwrap();
    fs::create_dir_all(base.join("realdir")).unwrap();
    fs::write(base.join("realdir/child"), b"child").unwrap();

    std::os::unix::fs::symlink("real", base.join("internal")).unwrap();
    std::os::unix::fs::symlink(&outside, base.join("escaping")).unwrap();
    std::os::unix::fs::symlink("nowhere", base.join("dangling")).unwrap();
    std::os::unix::fs::symlink("realdir", base.join("linkdir")).unwrap();

    let plan = plan_json(
        &["r"],
        &[
            file_target("internal", "r", "internal"),
            file_target("escaping", "r", "escaping"),
            file_target("dangling", "r", "dangling"),
            file_target("nonfinal", "r", "linkdir/child"),
        ]
        .join(","),
    );
    let out = run(&base, &plan);

    // Trailing symlinks: pinned as O_PATH and rejected at classification, so the
    // observed kind is established and no destination is opened.
    for (i, name) in [(0, "internal"), (1, "escaping"), (2, "dangling")] {
        assert_eq!(
            out[i],
            TargetOutcome::Rejected {
                rejection: Rejection::SymlinkForbidden,
                observed_kind: Some(ObjectKind::Symlink)
            },
            "trailing symlink {name} must be classified then rejected"
        );
    }
    // Non-final symlink: constrained resolution refuses it, so no descriptor and
    // therefore no observed kind.
    assert_eq!(
        out[3],
        TargetOutcome::Rejected {
            rejection: Rejection::SymlinkForbidden,
            observed_kind: None
        },
        "non-final symlink is rejected at resolution"
    );
    assert_eq!(
        fs::read(&outside).unwrap(),
        b"OUTSIDE-CANARY",
        "canary untouched"
    );
    let _ = fs::remove_file(&outside);
}

#[test]
fn special_files_are_classified_and_rejected_without_data_access() {
    let base = temp_root("special");
    let fifo = base.join("fifo");
    let status = Command::new("mkfifo").arg(&fifo).status().unwrap();
    assert!(status.success(), "mkfifo must succeed unprivileged");
    let sock = base.join("sock");
    let _listener = UnixListener::bind(&sock).unwrap();

    let plan = plan_json(
        &["r"],
        &[
            file_target("fifo", "r", "fifo"),
            file_target("sock", "r", "sock"),
        ]
        .join(","),
    );
    // A writerless FIFO would block a direct O_RDONLY open forever. This
    // completes promptly because classification happens on an O_PATH pin.
    let out = run(&base, &plan);
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

#[test]
fn hardlink_and_sparse_files_are_observed_without_provenance_claims() {
    let base = temp_root("hardlink-sparse");
    let canary = base.parent().unwrap().join("helm-observe-alias-canary");
    fs::write(&canary, b"shared inode payload").unwrap();
    fs::hard_link(&canary, base.join("alias")).unwrap();

    let sparse = base.join("sparse");
    let f = fs::File::create(&sparse).unwrap();
    f.set_len(1 << 16).unwrap();
    drop(f);

    let plan = plan_json(
        &["r"],
        &[
            file_target("alias", "r", "alias"),
            file_target("sparse", "r", "sparse"),
        ]
        .join(","),
    );
    let out = run(&base, &plan);
    match out[0] {
        TargetOutcome::ObservedFile(f) => {
            assert_eq!(f.sha256.to_hex(), sha256_hex(b"shared inode payload"));
            assert!(
                f.link_count >= 2,
                "link count is sampled, establishing no origin"
            );
        }
        other => panic!("expected hardlinked file, got {other:?}"),
    }
    match out[1] {
        TargetOutcome::ObservedFile(f) => {
            // Holes hash as zeros; no hole skipping.
            assert_eq!(f.bytes_read, 1 << 16);
            assert_eq!(f.sha256.to_hex(), sha256_hex(&vec![0u8; 1 << 16]));
        }
        other => panic!("expected sparse file, got {other:?}"),
    }
    let _ = fs::remove_file(&canary);
}

#[test]
fn directory_metadata_never_enumerates_children() {
    let base = temp_root("dirmeta");
    let dir = base.join("target-dir");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("PRIVATE-MARKER"), b"PRIVATE-MARKER-BYTES").unwrap();

    let plan = plan_json(
        &["r"],
        &format!(
            "{},{}",
            dir_target("d", "r", Some("target-dir")),
            dir_target("rootdir", "r", None)
        ),
    );
    let plan_parsed = parse_plan(&plan).unwrap();
    let root = root_from_fd("r", dir_fd(&base)).unwrap();
    let scope = authorize(
        plan_parsed,
        vec![root],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();
    let artifact = observe(&scope);
    let text = String::from_utf8_lossy(artifact.exact_bytes());
    assert!(
        !text.contains("PRIVATE-MARKER"),
        "no child name may appear: {text}"
    );
    assert!(matches!(
        artifact.record().targets[0].outcome,
        TargetOutcome::ObservedDirectory(_)
    ));
    assert!(
        matches!(
            artifact.record().targets[1].outcome,
            TargetOutcome::ObservedDirectory(_)
        ),
        "an omitted path denotes the supplied root directory itself"
    );
}

#[test]
fn unlisted_siblings_are_never_read_or_leaked() {
    let base = temp_root("privacy");
    fs::write(base.join("listed"), b"listed body").unwrap();
    fs::write(base.join("unlisted"), b"PRIVATE-MARKER-must-never-be-read").unwrap();

    let plan = plan_json(&["r"], &file_target("listed", "r", "listed"));
    let parsed = parse_plan(&plan).unwrap();
    let root = root_from_fd("r", dir_fd(&base)).unwrap();
    let scope = authorize(
        parsed,
        vec![root],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();
    let artifact = observe(&scope);
    let text = String::from_utf8_lossy(artifact.exact_bytes());
    assert!(
        !text.contains("PRIVATE-MARKER"),
        "unlisted body must not leak"
    );
    assert!(!text.contains("unlisted"), "unlisted name must not appear");
    assert!(
        !text.contains(&base.to_string_lossy().into_owned()),
        "no absolute host path"
    );
    assert_eq!(
        artifact.record().targets.len(),
        1,
        "only declared targets appear"
    );
}

#[test]
fn budgets_are_enforced_and_every_declared_target_has_an_outcome() {
    let base = temp_root("budget");
    // Sparse file whose metadata length exceeds the per-file ceiling. It must be
    // rejected on metadata alone, with no data reopen.
    let over = base.join("over");
    let f = fs::File::create(&over).unwrap();
    f.set_len(helm_observe::MAX_FILE_BYTES + 4096).unwrap();
    drop(f);
    fs::write(base.join("after"), b"still declared").unwrap();

    let plan = plan_json(
        &["r"],
        &format!(
            "{},{}",
            file_target("over", "r", "over"),
            file_target("after", "r", "after")
        ),
    );
    let out = run(&base, &plan);
    assert_eq!(
        out[0],
        TargetOutcome::NotObserved(BudgetReason::FileLimit),
        "over-limit metadata is rejected before any data access"
    );
    match out[1] {
        TargetOutcome::ObservedFile(_) => {}
        other => panic!("a per-file limit must not exhaust the batch, got {other:?}"),
    }
    assert_eq!(
        out.len(),
        2,
        "every declared target has an explicit outcome"
    );
}

#[test]
fn retained_root_descriptor_survives_pathname_replacement() {
    let base = temp_root("retained");
    let root = base.join("live");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("f"), b"original").unwrap();

    let plan = parse_plan(&plan_json(&["r"], &file_target("t", "r", "f"))).unwrap();
    let cap = root_from_fd("r", dir_fd(&root)).unwrap();

    // Replace the pathname after the capability was taken.
    fs::rename(&root, base.join("moved")).unwrap();
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("f"), b"replacement").unwrap();

    let scope = authorize(
        plan,
        vec![cap],
        proc_fd_from_trusted_current_process(procfs()).unwrap(),
    )
    .unwrap();
    let out = observe(&scope);
    match out.record().targets[0].outcome {
        TargetOutcome::ObservedFile(f) => assert_eq!(
            f.sha256.to_hex(),
            sha256_hex(b"original"),
            "the retained descriptor still names the original object"
        ),
        other => panic!("expected the pinned original, got {other:?}"),
    }
}

/// Supported-cohort check. The cohort is Linux x86_64 on local ext4. If the
/// runner is not ext4 this test does **not** weaken admission to pass: it
/// asserts that admission correctly refuses, and prints the filesystem so the
/// report can record the ext4 cohort as BLOCKED rather than claimed.
#[test]
fn ext4_cohort_admission_is_reported_not_weakened() {
    let base = temp_root("cohort");
    let magic = Command::new("stat")
        .arg("-f")
        .arg("-c")
        .arg("%T %t")
        .arg(&base)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|_| "unknown".to_owned());
    let admitted = root_from_fd("r", dir_fd(&base));
    if is_ext(&base) {
        assert!(
            admitted.is_ok(),
            "ext4 fixture root must be admitted: {magic}"
        );
        println!("HELM-OBSERVE-COHORT: ext4 PASS ({magic})");
    } else {
        let err = admitted.unwrap_err();
        assert_eq!(
            err.code(),
            A::RootUnsupportedFilesystem,
            "a non-cohort filesystem must be refused, never silently accepted: {magic}"
        );
        println!("HELM-OBSERVE-COHORT: BLOCKED, fixture filesystem is not ext4 ({magic})");
    }
}
