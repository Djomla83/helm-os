//! Linux x86_64 capability admission, authorisation composition and their
//! refusals (**P2**).
//!
//! These are ordinary product tests. They are **not** LAUNCH-EXEC-01 cases, not
//! a formal trial and not D-7 activity. No test here executes an admitted
//! object, invokes `launcher_spike` or any helper ELF, or creates a process:
//! the crate has no function that could.
//!
//! The test is the trusted caller. It may use pathnames to build fixtures and
//! to open descriptors in each mode the accepted contract names; the product
//! API never can, and receives authority only as an owned descriptor.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::os::fd::{AsFd as _, OwnedFd};

use helm_launch::{
    AdmissionErrorCode as A, AuthorizationRefusalCode, Digest, ElfType, MAX_EXECUTABLE_BYTES,
    admit_executable, admit_working_directory, authorize, parse_launch_plan,
};
use rustix::fs::{
    Mode, OFlags, SeekFrom, chmod, fstat, ftruncate, mkdir, open, rmdir, seek, unlink,
};
use rustix::io::{Errno, FdFlags, fcntl_getfd, fcntl_setfd, write};

// ---------------------------------------------------------------- fixtures

const SCRATCH: &str = "/tmp/helm-launch-p2-admission";

/// The 64-byte ELF64 header the accepted cohort fixes, built here independently
/// of the crate's own constants.
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const EM_X86_64: u16 = 62;
const ET_EXEC: u16 = 2;
const ET_DYN: u16 = 3;

fn elf_header(e_type: u16) -> [u8; 64] {
    let mut header = [0u8; 64];
    header[..4].copy_from_slice(&[0x7f, b'E', b'L', b'F']);
    header[4] = ELFCLASS64;
    header[5] = ELFDATA2LSB;
    header[6] = 1; // EI_VERSION: deliberately outside the cohort check.
    header[16..18].copy_from_slice(&e_type.to_le_bytes());
    header[18..20].copy_from_slice(&EM_X86_64.to_le_bytes());
    header
}

/// The 200 KiB deterministic payload of the digest vector below.
fn pattern_payload() -> Vec<u8> {
    (0..204_800usize)
        .map(|i| u8::try_from(i % 251).unwrap())
        .collect()
}

fn scratch_path(name: &str) -> String {
    match mkdir(SCRATCH, Mode::from_bits_truncate(0o700)) {
        Ok(()) | Err(Errno::EXIST) => {}
        Err(other) => panic!("scratch directory: {other:?}"),
    }
    format!("{SCRATCH}/{name}")
}

/// Writes `bytes` to a fresh fixture, then sets `mode` explicitly so no umask
/// can change it, and asserts the mode actually took effect.
fn fixture(name: &str, bytes: &[u8], mode: u32) -> String {
    let path = scratch_path(name);
    let _ = unlink(path.as_str());
    let fd = open(
        path.as_str(),
        OFlags::CREATE | OFlags::WRONLY | OFlags::TRUNC,
        Mode::from_bits_truncate(0o600),
    )
    .unwrap();
    let mut written = 0usize;
    while written < bytes.len() {
        written += write(fd.as_fd(), &bytes[written..]).unwrap();
    }
    drop(fd);
    chmod(path.as_str(), Mode::from_bits_truncate(mode)).unwrap();
    let observed = fstat(read_only(path.as_str())).unwrap();
    assert_eq!(
        observed.st_mode & 0o7777,
        mode,
        "the host did not keep fixture mode {mode:o}"
    );
    path
}

fn read_only(path: &str) -> OwnedFd {
    open(path, OFlags::RDONLY, Mode::empty()).unwrap()
}

fn directory_fixture(name: &str) -> String {
    let path = scratch_path(name);
    match mkdir(path.as_str(), Mode::from_bits_truncate(0o700)) {
        Ok(()) | Err(Errno::EXIST) => {}
        Err(other) => panic!("fixture directory: {other:?}"),
    }
    path
}

/// A valid 0.1 plan naming `id` as its working-directory capability.
fn plan_bytes(id: &str, asserted_context: &str) -> Vec<u8> {
    format!(
        concat!(
            r#"{{"schema":"helm-launch-plan","version":"0.1","#,
            r#""execution_kind":"linux_exact_executable","#,
            r#""argv":["tool","--flag"],"#,
            r#""environment":{{"mode":"empty"}},"#,
            r#""working_directory":{{"capability_id":"{id}"}},"#,
            r#""stdin":{{"mode":"closed_pipe_eof"}},"#,
            r#""stdout":{{"capture_prefix_bytes":0}},"#,
            r#""stderr":{{"capture_prefix_bytes":4096}},"#,
            r#""timeout_ms":30000,"#,
            r#""termination":{{"signal":"SIGTERM","grace_ms":5000}}{ctx}}}"#
        ),
        id = id,
        ctx = asserted_context
    )
    .into_bytes()
}

// ------------------------------------------------------ A, B, J, S: admitted

#[test]
fn a_readable_regular_in_cohort_et_exec_object_is_admitted() {
    let path = fixture("a-et-exec", &elf_header(ET_EXEC), 0o755);
    let capability = admit_executable(read_only(path.as_str())).unwrap();
    assert_eq!(capability.elf_type(), ElfType::EtExec);
    assert_eq!(capability.pre_exec_body_size(), 64);
    assert_eq!(capability.pre_exec_mode_bits(), 0o755);
    unlink(path.as_str()).unwrap();
}

#[test]
fn a_readable_regular_in_cohort_et_dyn_object_is_admitted() {
    let path = fixture("b-et-dyn", &elf_header(ET_DYN), 0o755);
    let capability = admit_executable(read_only(path.as_str())).unwrap();
    assert_eq!(capability.elf_type(), ElfType::EtDyn);
    // The independently computed digest of these exact 64 bytes.
    assert_eq!(
        capability.pre_exec_body_sha256().to_hex(),
        "d58548872dbb5a72d6a518237a170b8173e6a91212356f7d9b249cbf591dc713"
    );
    unlink(path.as_str()).unwrap();
}

#[test]
fn an_unrelated_status_flag_does_not_make_the_mode_unsuitable() {
    // Only the access mode and `O_PATH` are inspected. `O_NONBLOCK` is a status
    // flag `F_GETFL` reports and admission deliberately ignores.
    let path = fixture("j-nonblock", &elf_header(ET_EXEC), 0o644);
    let fd = open(
        path.as_str(),
        OFlags::RDONLY | OFlags::NONBLOCK,
        Mode::empty(),
    )
    .unwrap();
    assert!(admit_executable(fd).is_ok());
    unlink(path.as_str()).unwrap();
}

#[test]
fn no_execute_bit_is_required_or_checked_at_admission() {
    // Execute permission is recorded, never enforced: the kernel decides at
    // execution time, which this crate cannot reach. Admitting this object
    // therefore says **nothing** about whether it could ever be executed.
    for mode in [0o400u32, 0o444, 0o600] {
        let path = fixture("s-no-execute", &elf_header(ET_EXEC), mode);
        let capability = admit_executable(read_only(path.as_str())).unwrap();
        assert_eq!(
            capability.pre_exec_mode_bits(),
            u16::try_from(mode).unwrap()
        );
        unlink(path.as_str()).unwrap();
    }
}

// --------------------------------------------------- C, D, E, O: not in cohort

#[test]
fn a_script_object_is_refused_as_not_elf() {
    // One script long enough to pass the 64-byte length rule, so its refusal
    // really comes from the absent ELF magic, and one short enough that the
    // length rule stops it first. Both are `NOT_ELF`: scripts are unsupported,
    // not partially supported.
    let mut long = b"#!/bin/sh\nexit 0\n".to_vec();
    long.resize(128, b'#');
    let short = b"#!/bin/sh\nexit 0\n".to_vec();
    assert!(long.len() > 64 && short.len() < 64);
    for script in [long, short] {
        let path = fixture("c-script", &script, 0o755);
        assert_eq!(
            admit_executable(read_only(path.as_str()))
                .unwrap_err()
                .code(),
            A::NotElf,
            "{} bytes",
            script.len()
        );
        unlink(path.as_str()).unwrap();
    }
}

#[test]
fn a_non_elf_regular_file_is_refused_as_not_elf() {
    let path = fixture("d-non-elf", &[0x5au8; 4_096], 0o644);
    assert_eq!(
        admit_executable(read_only(path.as_str()))
            .unwrap_err()
            .code(),
        A::NotElf
    );
    unlink(path.as_str()).unwrap();
}

#[test]
fn an_object_shorter_than_the_header_is_refused_as_not_elf() {
    for length in [0usize, 1, 4, 20, 63] {
        let mut bytes = elf_header(ET_EXEC).to_vec();
        bytes.truncate(length);
        let path = fixture("o-short", &bytes, 0o644);
        assert_eq!(
            admit_executable(read_only(path.as_str()))
                .unwrap_err()
                .code(),
            A::NotElf,
            "length {length}"
        );
        unlink(path.as_str()).unwrap();
    }
}

#[test]
fn an_elf_object_outside_the_cohort_is_refused() {
    let mut wrong_class = elf_header(ET_EXEC);
    wrong_class[4] = 1; // ELFCLASS32
    let mut wrong_endian = elf_header(ET_EXEC);
    wrong_endian[5] = 2; // ELFDATA2MSB
    let mut wrong_machine = elf_header(ET_EXEC);
    wrong_machine[18..20].copy_from_slice(&183u16.to_le_bytes()); // EM_AARCH64
    let unsupported_type = elf_header(1); // ET_REL
    let core_type = elf_header(4); // ET_CORE

    for (label, bytes) in [
        ("class", wrong_class),
        ("endian", wrong_endian),
        ("machine", wrong_machine),
        ("e_type ET_REL", unsupported_type),
        ("e_type ET_CORE", core_type),
    ] {
        let path = fixture("e-out-of-cohort", &bytes, 0o644);
        assert_eq!(
            admit_executable(read_only(path.as_str()))
                .unwrap_err()
                .code(),
            A::ElfNotInCohort,
            "{label}"
        );
        unlink(path.as_str()).unwrap();
    }
}

// -------------------------------------------------- F: not a regular file

#[test]
fn a_non_regular_descriptor_is_refused() {
    let directory = directory_fixture("f-directory");
    assert_eq!(
        admit_executable(read_only(directory.as_str()))
            .unwrap_err()
            .code(),
        A::NotRegularFile
    );
    rmdir(directory.as_str()).unwrap();

    // A character device is readable and not a regular file either.
    assert_eq!(
        admit_executable(read_only("/dev/null")).unwrap_err().code(),
        A::NotRegularFile
    );
}

// ----------------------------------------- G, H, I: unsuitable descriptor mode

#[test]
fn an_o_path_descriptor_is_refused() {
    let path = fixture("g-o-path", &elf_header(ET_EXEC), 0o755);
    let fd = open(path.as_str(), OFlags::PATH, Mode::empty()).unwrap();
    assert_eq!(
        admit_executable(fd).unwrap_err().code(),
        A::DescriptorModeUnsuitable
    );
    unlink(path.as_str()).unwrap();
}

#[test]
fn a_writable_descriptor_is_refused_in_either_writable_mode() {
    let path = fixture("hi-writable", &elf_header(ET_EXEC), 0o644);
    for (label, flags) in [
        ("O_WRONLY", OFlags::WRONLY),
        ("O_RDWR", OFlags::RDWR),
        ("O_RDWR|O_APPEND", OFlags::RDWR.union(OFlags::APPEND)),
    ] {
        let fd = open(path.as_str(), flags, Mode::empty()).unwrap();
        assert_eq!(
            admit_executable(fd).unwrap_err().code(),
            A::DescriptorModeUnsuitable,
            "{label}"
        );
    }
    unlink(path.as_str()).unwrap();
}

// ------------------------------------------------ K: close-on-exec is ignored

#[test]
fn close_on_exec_is_irrelevant_to_admission_in_either_state() {
    // Admission never inspects `FD_CLOEXEC`: that is a descriptor flag read
    // with `F_GETFD`, and `F_GETFL` does not report it. A future launch would
    // duplicate the descriptor with `F_DUPFD_CLOEXEC` anyway — and that future
    // launch does not exist here. All three states are therefore admissible.
    let path = fixture("k-cloexec", &elf_header(ET_EXEC), 0o755);

    // `rustix::fs::open` passes exactly the flags given, so this one has no
    // `FD_CLOEXEC`; a caller opening through `std` would get one that has it.
    let without = read_only(path.as_str());
    assert!(
        !fcntl_getfd(without.as_fd())
            .unwrap()
            .contains(FdFlags::CLOEXEC)
    );
    assert!(admit_executable(without).is_ok());

    let with = open(
        path.as_str(),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    assert!(
        fcntl_getfd(with.as_fd())
            .unwrap()
            .contains(FdFlags::CLOEXEC)
    );
    assert!(admit_executable(with).is_ok());

    // And one whose flag the caller clears after opening.
    let cleared = open(
        path.as_str(),
        OFlags::RDONLY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    fcntl_setfd(cleared.as_fd(), FdFlags::empty()).unwrap();
    assert!(
        !fcntl_getfd(cleared.as_fd())
            .unwrap()
            .contains(FdFlags::CLOEXEC)
    );
    assert!(admit_executable(cleared).is_ok());

    unlink(path.as_str()).unwrap();
}

// --------------------------------------------------- L, M: set-ID refusal

#[test]
fn a_set_id_object_is_refused_without_any_privilege_transition() {
    for (label, mode) in [("S_ISUID", 0o4755u32), ("S_ISGID", 0o2755)] {
        let path = fixture("lm-set-id", &elf_header(ET_EXEC), mode);
        assert_eq!(
            admit_executable(read_only(path.as_str()))
                .unwrap_err()
                .code(),
            A::SetIdBitsPresent,
            "{label}"
        );
        unlink(path.as_str()).unwrap();
    }
}

// ------------------------------------------------------- N: the size bound

#[test]
fn an_object_above_the_size_bound_is_refused_before_any_body_read() {
    let path = scratch_path("n-too-large");
    let _ = unlink(path.as_str());
    let fd = open(
        path.as_str(),
        OFlags::CREATE | OFlags::WRONLY | OFlags::TRUNC,
        Mode::from_bits_truncate(0o600),
    )
    .unwrap();
    // Sparse: the extent is above the bound while no data block is allocated,
    // so a refusal that read the body would be visible as a long read.
    ftruncate(fd.as_fd(), MAX_EXECUTABLE_BYTES + 1).unwrap();
    drop(fd);
    assert_eq!(MAX_EXECUTABLE_BYTES, 536_870_912);
    assert_eq!(
        admit_executable(read_only(path.as_str()))
            .unwrap_err()
            .code(),
        A::ExecutableTooLarge
    );
    unlink(path.as_str()).unwrap();
}

// ---------------------------------------- P, Q, R: measurement and its facts

#[test]
fn the_measurement_matches_an_independently_computed_digest() {
    // These digests were computed outside Rust over exactly these bytes, by two
    // implementations, so the comparison is not the crate checking itself.
    let mut body = elf_header(ET_EXEC).to_vec();
    body.extend_from_slice(&pattern_payload());
    assert_eq!(body.len(), 204_864);

    let path = fixture("p-measured", &body, 0o644);
    let capability = admit_executable(read_only(path.as_str())).unwrap();
    assert_eq!(capability.pre_exec_body_size(), 204_864);
    assert_eq!(
        capability.pre_exec_body_sha256().to_hex(),
        "5889a02db0414b3b91c1fc2b681a3001d223afbbd3f01b80152c26ae9e288347"
    );
    // A body longer than the fixed 64 KiB read buffer takes several rounds, and
    // the digest above proves every round contributed exactly once.
    assert!(capability.pre_exec_body_size() > 65_536 * 3);
    unlink(path.as_str()).unwrap();

    let header_only = fixture("p-header-only", &elf_header(ET_EXEC), 0o644);
    let capability = admit_executable(read_only(header_only.as_str())).unwrap();
    assert_eq!(
        capability.pre_exec_body_sha256().to_hex(),
        "fac1ecd4990500ce3d68a41947d21e042f592eb9007dddd2d636e8f514605ca0"
    );
    // The same bytes always measure the same, and the measurement is a value
    // that identifies bytes, nothing more.
    let again = admit_executable(read_only(header_only.as_str())).unwrap();
    assert_eq!(
        again.pre_exec_body_sha256(),
        capability.pre_exec_body_sha256()
    );
    unlink(header_only.as_str()).unwrap();
}

#[test]
fn the_shared_file_offset_is_never_used_or_moved() {
    let mut body = elf_header(ET_EXEC).to_vec();
    body.extend_from_slice(&[9u8; 70_000]);
    let path = fixture("q-offset", &body, 0o644);

    let fd = read_only(path.as_str());
    // `dup` shares the one open file description, so it observes the same
    // shared offset the admitted descriptor would use.
    let watcher = rustix::io::dup(fd.as_fd()).unwrap();
    seek(fd.as_fd(), SeekFrom::Start(4_097)).unwrap();
    let capability = admit_executable(fd).unwrap();
    assert_eq!(capability.pre_exec_body_size(), 70_064);
    assert_eq!(
        seek(watcher.as_fd(), SeekFrom::Current(0)).unwrap(),
        4_097,
        "the header read or the measurement moved the shared file offset"
    );
    unlink(path.as_str()).unwrap();
}

#[test]
fn recorded_mode_bits_come_from_the_first_accepted_metadata_sample() {
    let path = fixture("r-mode-bits", &elf_header(ET_EXEC), 0o750);
    let fd = read_only(path.as_str());
    let capability = admit_executable(fd).unwrap();
    assert_eq!(capability.pre_exec_mode_bits(), 0o750);

    // Changing the mode afterwards does not rewrite the admitted fact: it is a
    // pre-execution sample, not a live view of the object.
    chmod(path.as_str(), Mode::from_bits_truncate(0o600)).unwrap();
    assert_eq!(capability.pre_exec_mode_bits(), 0o750);
    assert_eq!(capability.measurement().pre_exec_mode_bits(), 0o750);
    unlink(path.as_str()).unwrap();
}

// ------------------------------------------------ privacy of refusals and facts

#[test]
fn no_refusal_or_capability_ever_prints_a_path_or_a_descriptor_number() {
    let path = fixture("privacy", &[0u8; 128], 0o644);
    let refusal = admit_executable(read_only(path.as_str())).unwrap_err();
    let rendered = format!("{refusal} {refusal:?}");
    assert!(!rendered.contains("/tmp"), "{rendered}");
    assert!(!rendered.contains("privacy"), "{rendered}");
    assert!(!rendered.contains("helm-launch-p2"), "{rendered}");

    let ok_path = fixture("privacy-ok", &elf_header(ET_EXEC), 0o644);
    let capability = admit_executable(read_only(ok_path.as_str())).unwrap();
    let rendered = format!("{capability:?}");
    assert!(rendered.contains("pre_exec_body_sha256"), "{rendered}");
    assert!(!rendered.contains("/tmp"), "{rendered}");
    assert!(!rendered.contains("privacy"), "{rendered}");
    assert!(!rendered.contains("fd"), "{rendered}");

    let directory = directory_fixture("privacy-dir");
    let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
    let rendered = format!("{cwd:?}");
    assert!(rendered.contains("workdir"), "{rendered}");
    assert!(!rendered.contains("/tmp"), "{rendered}");

    unlink(path.as_str()).unwrap();
    unlink(ok_path.as_str()).unwrap();
    drop(cwd);
    rmdir(directory.as_str()).unwrap();
}

// ----------------------------------------------------- working directory

#[test]
fn a_read_only_directory_with_a_valid_identifier_is_admitted() {
    let directory = directory_fixture("wd-valid");
    for id in ["workdir", "a", "0", "a.b_c-d", &"z".repeat(80)] {
        let cwd = admit_working_directory(id, read_only(directory.as_str())).unwrap();
        assert_eq!(cwd.id(), id);
    }
    rmdir(directory.as_str()).unwrap();
}

#[test]
fn an_invalid_identifier_is_refused_by_the_shared_grammar() {
    let directory = directory_fixture("wd-invalid-id");
    for id in [
        "",
        "-leading",
        ".leading",
        "_leading",
        "Upper",
        "with space",
        "with/slash",
        "with:colon",
        "tab\there",
        "nul\0byte",
        "ünicode",
        &"z".repeat(81),
    ] {
        assert_eq!(
            admit_working_directory(id, read_only(directory.as_str()))
                .unwrap_err()
                .code(),
            A::WorkingDirectoryIdInvalid,
            "identifier {id:?}"
        );
    }
    rmdir(directory.as_str()).unwrap();
}

#[test]
fn a_regular_file_is_not_a_working_directory() {
    let path = fixture("wd-regular", &elf_header(ET_EXEC), 0o644);
    assert_eq!(
        admit_working_directory("workdir", read_only(path.as_str()))
            .unwrap_err()
            .code(),
        A::NotDirectory
    );
    unlink(path.as_str()).unwrap();
}

#[test]
fn an_o_path_directory_is_refused() {
    let directory = directory_fixture("wd-o-path");
    let fd = open(directory.as_str(), OFlags::PATH, Mode::empty()).unwrap();
    assert_eq!(
        admit_working_directory("workdir", fd).unwrap_err().code(),
        A::DescriptorModeUnsuitable
    );
    rmdir(directory.as_str()).unwrap();
}

#[test]
fn working_directory_admission_neither_enumerates_nor_resolves_anything() {
    // The capability exposes exactly the caller's logical identifier: no path,
    // no entry list, no descriptor number. Admission also checks no search
    // permission — that kernel decision belongs to an execution slice which
    // does not exist here — so a directory the caller cannot search is still
    // admissible, and admission says nothing about using it later.
    let directory = directory_fixture("wd-opaque");
    let child = format!("{directory}/entry");
    let fd = open(
        child.as_str(),
        OFlags::CREATE | OFlags::WRONLY | OFlags::TRUNC,
        Mode::from_bits_truncate(0o600),
    )
    .unwrap();
    drop(fd);

    let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
    assert_eq!(cwd.id(), "workdir");
    let rendered = format!("{cwd:?}");
    assert!(!rendered.contains("entry"), "{rendered}");
    assert!(!rendered.contains(directory.as_str()), "{rendered}");

    // Remove the entry while the directory is still writable, then make it
    // readable but **not searchable**: `open(O_RDONLY)` still succeeds, while a
    // future `fchdir` would be denied by the kernel. Admission must not
    // anticipate that decision.
    unlink(child.as_str()).unwrap();
    chmod(directory.as_str(), Mode::from_bits_truncate(0o400)).unwrap();
    let unsearchable = admit_working_directory("workdir", read_only(directory.as_str()));
    assert!(
        unsearchable.is_ok(),
        "admission must not pre-check search permission"
    );
    drop(unsearchable);
    drop(cwd);

    chmod(directory.as_str(), Mode::from_bits_truncate(0o700)).unwrap();
    rmdir(directory.as_str()).unwrap();
}

// -------------------------------------------------------------- authorize

#[test]
fn authorisation_succeeds_exactly_when_the_identifiers_match() {
    let path = fixture("auth-object", &elf_header(ET_EXEC), 0o755);
    let directory = directory_fixture("auth-dir");

    let plan = parse_launch_plan(&plan_bytes("workdir", "")).unwrap();
    let plan_sha256 = plan.sha256();
    let executable = admit_executable(read_only(path.as_str())).unwrap();
    let measured = executable.measurement();
    let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();

    let authorized = authorize(plan, executable, cwd).unwrap();
    assert_eq!(authorized.working_directory_id(), "workdir");
    assert_eq!(authorized.plan_sha256(), plan_sha256);
    assert_eq!(
        authorized.executable_measurement(),
        measured,
        "authorize must move the measurement in unchanged"
    );
    // The object can change under the authorisation without the recorded facts
    // changing: `authorize` does not re-measure, reopen or re-resolve anything,
    // and the value is therefore not a claim about the object's current state.
    chmod(path.as_str(), Mode::from_bits_truncate(0o600)).unwrap();
    assert_eq!(authorized.executable_measurement(), measured);

    drop(authorized);
    unlink(path.as_str()).unwrap();
    rmdir(directory.as_str()).unwrap();
}

#[test]
fn a_mismatching_working_directory_identifier_is_refused() {
    let path = fixture("auth-mismatch", &elf_header(ET_EXEC), 0o755);
    let directory = directory_fixture("auth-mismatch-dir");

    for (plan_id, capability_id) in [
        ("workdir", "other"),
        ("other", "workdir"),
        ("workdir", "workdir2"),
        ("workdir.a", "workdir-a"),
    ] {
        let plan = parse_launch_plan(&plan_bytes(plan_id, "")).unwrap();
        let executable = admit_executable(read_only(path.as_str())).unwrap();
        let cwd = admit_working_directory(capability_id, read_only(directory.as_str())).unwrap();
        let refusal = authorize(plan, executable, cwd).unwrap_err();
        assert_eq!(
            refusal.code(),
            AuthorizationRefusalCode::WorkingDirectoryIdMismatch,
            "{plan_id} vs {capability_id}"
        );
        // The refusal carries a code and nothing else: no capability comes back.
        assert_eq!(refusal.to_string(), "WORKING_DIRECTORY_ID_MISMATCH");
    }

    unlink(path.as_str()).unwrap();
    rmdir(directory.as_str()).unwrap();
}

#[test]
fn asserted_context_digests_never_affect_authorisation() {
    // The digests are inert caller assertions. `authorize` neither reads nor
    // compares them, so their presence, absence and value change nothing.
    // No binding result, observation result or specification verdict can be
    // supplied at all: no such type is nameable in this crate.
    let path = fixture("auth-context", &elf_header(ET_EXEC), 0o755);
    let directory = directory_fixture("auth-context-dir");

    let a = "aa".repeat(32);
    let b = "bb".repeat(32);
    let contexts = [
        String::new(),
        format!(r#","asserted_context":{{"subject_spec_sha256":"{a}"}}"#),
        format!(r#","asserted_context":{{"binding_report_sha256":"{b}"}}"#),
        format!(
            r#","asserted_context":{{"subject_spec_sha256":"{a}","binding_report_sha256":"{b}"}}"#
        ),
        format!(
            r#","asserted_context":{{"subject_spec_sha256":"{b}","binding_report_sha256":"{a}"}}"#
        ),
    ];

    let mut identities = Vec::new();
    for context in &contexts {
        let plan = parse_launch_plan(&plan_bytes("workdir", context)).unwrap();
        identities.push(plan.sha256());
        let executable = admit_executable(read_only(path.as_str())).unwrap();
        let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
        let authorized = authorize(plan, executable, cwd).unwrap();
        assert_eq!(authorized.working_directory_id(), "workdir");
    }
    // Each document is a different plan, so authorisation really did run five
    // times over different asserted context, with the same outcome.
    let mut unique: Vec<String> = identities.iter().map(|d| d.to_hex()).collect();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), contexts.len());

    // A mismatching identifier is still refused whatever the asserted context.
    for context in &contexts {
        let plan = parse_launch_plan(&plan_bytes("workdir", context)).unwrap();
        let executable = admit_executable(read_only(path.as_str())).unwrap();
        let cwd = admit_working_directory("other", read_only(directory.as_str())).unwrap();
        assert_eq!(
            authorize(plan, executable, cwd).unwrap_err().code(),
            AuthorizationRefusalCode::WorkingDirectoryIdMismatch
        );
    }

    unlink(path.as_str()).unwrap();
    rmdir(directory.as_str()).unwrap();
}

#[test]
fn authorisation_produces_no_receipt_and_no_process() {
    // There is nothing to assert about a receipt or a child, because neither can
    // exist: `authorize` returns an `AuthorizedLaunch` and this crate has no
    // function that consumes one. The `compile_fail` doctests on
    // `AuthorizedLaunch` pin the absence of `launch` and `LaunchOutcome`, and
    // `tests/p2_boundary.rs` pins the absence of every process-creation token.
    let path = fixture("auth-no-receipt", &elf_header(ET_EXEC), 0o755);
    let directory = directory_fixture("auth-no-receipt-dir");
    let plan = parse_launch_plan(&plan_bytes("workdir", "")).unwrap();
    let executable = admit_executable(read_only(path.as_str())).unwrap();
    let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
    let authorized = authorize(plan, executable, cwd).unwrap();

    // The only things the authorisation exposes are inert facts.
    let _: Digest = authorized.plan_sha256();
    let _: &str = authorized.working_directory_id();
    let measurement = authorized.executable_measurement();
    assert_eq!(measurement.elf_type(), ElfType::EtExec);

    drop(authorized);
    unlink(path.as_str()).unwrap();
    rmdir(directory.as_str()).unwrap();
}

// ------------------------------------------------------ descriptor discipline

fn open_descriptor_count() -> usize {
    std::fs::read_dir("/proc/self/fd")
        .map(Iterator::count)
        .unwrap_or(0)
}

#[test]
fn refusals_and_drops_leak_no_descriptor() {
    let good = fixture("leak-good", &elf_header(ET_EXEC), 0o755);
    let bad = fixture("leak-bad", &[0u8; 128], 0o644);
    let set_id = fixture("leak-set-id", &elf_header(ET_EXEC), 0o4755);
    let directory = directory_fixture("leak-dir");

    // Warm up, so a first-call allocation cannot be mistaken for a leak.
    let baseline_warmup = admit_executable(read_only(good.as_str()));
    drop(baseline_warmup);
    let before = open_descriptor_count();
    assert!(before > 0, "/proc/self/fd must be readable for this test");

    // `/proc/self/fd` is process-wide and the harness runs tests in parallel,
    // so the assertion below tolerates a few descriptors owned by other tests.
    // A real leak here would be at least eight per round.
    const ROUNDS: usize = 64;
    const PARALLEL_ALLOWANCE: usize = 16;

    for _ in 0..ROUNDS {
        // Every refusal consumes the moved-in descriptor.
        assert!(admit_executable(read_only(bad.as_str())).is_err());
        assert!(admit_executable(read_only(set_id.as_str())).is_err());
        assert!(admit_working_directory("bad id", read_only(directory.as_str())).is_err());
        assert!(admit_working_directory("workdir", read_only(good.as_str())).is_err());

        // A refused authorisation drops both capabilities, closing both.
        let plan = parse_launch_plan(&plan_bytes("workdir", "")).unwrap();
        let executable = admit_executable(read_only(good.as_str())).unwrap();
        let cwd = admit_working_directory("other", read_only(directory.as_str())).unwrap();
        assert!(authorize(plan, executable, cwd).is_err());

        // A successful admission and authorisation closes on drop too.
        let plan = parse_launch_plan(&plan_bytes("workdir", "")).unwrap();
        let executable = admit_executable(read_only(good.as_str())).unwrap();
        let cwd = admit_working_directory("workdir", read_only(directory.as_str())).unwrap();
        drop(authorize(plan, executable, cwd).unwrap());
    }

    let after = open_descriptor_count();
    assert!(
        after <= before + PARALLEL_ALLOWANCE,
        "admission, refusal or authorisation leaked a descriptor: \
         {before} open before {ROUNDS} rounds, {after} after"
    );

    unlink(good.as_str()).unwrap();
    unlink(bad.as_str()).unwrap();
    unlink(set_id.as_str()).unwrap();
    rmdir(directory.as_str()).unwrap();
}
