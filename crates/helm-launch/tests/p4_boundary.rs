//! P4 boundary inspection: the public launch surface, the guarded group sweep,
//! the `Err`-versus-receipt boundary, and the absence of every P5 claim.
//!
//! # Why this file reads the tree
//!
//! P4 is the first slice with a **public** way to create a process, so the
//! claims that matter are now about what that surface does **not** offer and
//! about where the one group signal may appear. Like `tests/p3_boundary.rs`
//! this file walks the sources at run time rather than embedding a list, so a
//! new file cannot slip past a claim, and it runs on **every** platform,
//! because the tree is the same everywhere. It reads files and executes
//! nothing.
//!
//! # What it claims
//!
//! * `src/launch.rs` contains **no `unsafe`**, no `libc` call, no raw syscall
//!   shim and no `std::process::Command` outside its own test fixtures.
//! * There is exactly **one** process-group signal call site in the whole
//!   crate, it is in `src/launch.rs`, and it is `SIGKILL`.
//! * The sweep is reached only through the model's guard: the crate names the
//!   non-consuming `WNOWAIT` probe, and no source infers group authority from
//!   an observed process group.
//! * The direct child is never signalled or waited for by numeric pid.
//! * `launch` returns `Err` only from the pre-child path.
//! * No exec-success, containment, sandbox or receipt-authenticity vocabulary
//!   exists anywhere in the crate.
//! * The public surface exposes no pid, pidfd, descriptor or backend type.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: a wrong assumption must fail loudly"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const LAUNCH_FILE: &str = "src/launch.rs";
const CRATE_MANIFEST: &str = include_str!("../Cargo.toml");

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn rust_files(directory: &str) -> BTreeMap<String, String> {
    let root = crate_root().join(directory);
    let mut found = BTreeMap::new();
    let mut stack = vec![root];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).expect("read the source tree") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|ext| ext == "rs") {
                let relative = path
                    .strip_prefix(crate_root())
                    .expect("inside the crate")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect("read a source file");
                found.insert(relative, text);
            }
        }
    }
    assert!(!found.is_empty(), "the scan found no source at all");
    found
}

/// Strip line comments, block comments and string literals, so a claim is about
/// code and never about prose or a fixture's source text.
fn strip(source: &str) -> String {
    let bytes: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        let rest_two: String = bytes[i..bytes.len().min(i + 2)].iter().collect();
        if rest_two == "//" {
            while i < bytes.len() && bytes[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if rest_two == "/*" {
            i += 2;
            let mut depth = 1_usize;
            while i < bytes.len() && depth > 0 {
                let two: String = bytes[i..bytes.len().min(i + 2)].iter().collect();
                if two == "/*" {
                    depth += 1;
                    i += 2;
                } else if two == "*/" {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Raw strings of any hash count, then ordinary strings and chars.
        if bytes[i] == 'r' && i + 1 < bytes.len() && (bytes[i + 1] == '#' || bytes[i + 1] == '"') {
            let mut hashes = 0_usize;
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == '#' {
                hashes += 1;
                j += 1;
            }
            if j < bytes.len() && bytes[j] == '"' {
                let closing = format!("\"{}", "#".repeat(hashes));
                j += 1;
                while j < bytes.len() {
                    let candidate: String = bytes[j..bytes.len().min(j + closing.len())]
                        .iter()
                        .collect();
                    if candidate == closing {
                        j += closing.len();
                        break;
                    }
                    j += 1;
                }
                i = j;
                out.push(' ');
                continue;
            }
        }
        if bytes[i] == '"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == '\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push(' ');
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn tokens(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    for character in strip(source).chars() {
        if character.is_alphanumeric() || character == '_' {
            current.push(character);
        } else if !current.is_empty() {
            found.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        found.push(current);
    }
    found
}

fn compact(source: &str) -> String {
    strip(source).split_whitespace().collect()
}

/// The braced body of one function in whitespace-compacted source.
///
/// Compaction removes every newline, so a body cannot be delimited by one: the
/// scan matches braces instead. Getting this wrong would silently widen a claim
/// to the whole remainder of the file.
fn function_body<'a>(code: &'a str, signature: &str) -> &'a str {
    let after = code
        .split_once(signature)
        .unwrap_or_else(|| panic!("`{signature}` is missing"))
        .1;
    let open = after.find('{').expect("the function has a body");
    let mut depth = 0_usize;
    for (offset, character) in after.char_indices().skip(open) {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after[open..=offset];
                }
            }
            _ => {}
        }
    }
    panic!("`{signature}` has an unbalanced body");
}

// ===========================================================================
// The unsafe boundary does not move
// ===========================================================================

#[test]
fn the_launch_slice_contains_no_unsafe_and_no_foreign_interface() {
    let code = tokens(&rust_files("src")[LAUNCH_FILE]);
    for forbidden in ["unsafe", "asm", "libc", "extern", "transmute"] {
        assert!(
            !code.iter().any(|token| token == forbidden),
            "{LAUNCH_FILE} names `{forbidden}`: the P4 lifecycle must stay outside the unsafe boundary"
        );
    }
    // The scoped `allow(unsafe_code)` still exists exactly once, and not here.
    let sources = rust_files("src");
    let with_allow: Vec<&String> = sources
        .iter()
        .filter(|(_, text)| compact(text).contains("allow(unsafe_code)"))
        .map(|(name, _)| name)
        .collect();
    assert_eq!(
        with_allow,
        vec![&"src/backend/mod.rs".to_owned()],
        "the scoped unsafe allow moved or multiplied"
    );
}

#[test]
fn the_launch_slice_creates_no_process_of_its_own() {
    // The one process this slice creates comes from the P3 backend. Nothing
    // here shells out, and `std::process::Command` appears only in the test
    // module's fixture builders, which `strip` keeps but which must not be in
    // the product half of the file.
    let sources = rust_files("src");
    let code = strip(&sources[LAUNCH_FILE]);
    let product = code
        .split("mod tests")
        .next()
        .expect("the file has a product half");
    for forbidden in ["Command", "fork", "posix_spawn", "system"] {
        assert!(
            !product.contains(forbidden),
            "the product half of {LAUNCH_FILE} names `{forbidden}`"
        );
    }
}

// ===========================================================================
// The one guarded process-group signal
// ===========================================================================

#[test]
fn exactly_one_process_group_signal_site_exists_and_it_is_in_the_launch_slice() {
    let sources = rust_files("src");
    let mut sites = Vec::new();
    for (name, text) in &sources {
        let count = compact(text).matches("kill_process_group(").count();
        if count > 0 {
            sites.push((name.clone(), count));
        }
    }
    assert_eq!(
        sites,
        vec![(LAUNCH_FILE.to_owned(), 1_usize)],
        "the process-group signal must have exactly one call site, in the launch slice"
    );
    let code = compact(&sources[LAUNCH_FILE]);
    assert!(
        code.contains("kill_process_group(group,Signal::KILL)"),
        "the one group signal is not SIGKILL to the established group"
    );
    // No other spelling of a group signal exists anywhere.
    for (name, text) in &sources {
        for forbidden in ["killpg", "tgkill", "tkill"] {
            assert!(
                !tokens(text).iter().any(|token| token == forbidden),
                "{name} names `{forbidden}`"
            );
        }
    }
}

#[test]
fn the_sweep_is_guarded_by_a_non_consuming_probe() {
    let code = compact(&rust_files("src")[LAUNCH_FILE]);
    assert!(
        code.contains("WaitIdOptions::EXITED|WaitIdOptions::NOHANG|WaitIdOptions::NOWAIT"),
        "the probe before the sweep is not the accepted non-consuming WNOWAIT probe"
    );
    // The sweep is performed only from the model's own action, never from a
    // condition this file invents.
    assert!(
        code.contains("Action::SweepGroup=>self.sweep_group()"),
        "the sweep is reached from something other than the model's action"
    );
    assert_eq!(
        code.matches("fnsweep_group(").count(),
        1,
        "the sweep has more than one implementation"
    );
}

#[test]
fn no_source_infers_group_authority_from_an_observed_process_group() {
    let sources = rust_files("src");
    for (name, text) in &sources {
        if name == "src/backend/tests.rs" || name == LAUNCH_FILE {
            // Both contain the accepted *separation* prose, which `strip`
            // removes, and the fixture that reads `/proc/self/stat`.
            continue;
        }
        for forbidden in ["getpgid", "getpgrp", "tcgetpgrp"] {
            assert!(
                !tokens(text).iter().any(|token| token == forbidden),
                "{name} asks the system for a process group, which is never authority"
            );
        }
    }
    // The product half of the launch slice asks for no group either: the only
    // group it ever names is the one the parent's own `setpgid` created.
    let product = strip(&sources[LAUNCH_FILE])
        .split("mod tests")
        .next()
        .expect("the file has a product half")
        .to_owned();
    for forbidden in ["getpgid", "getpgrp", "tcgetpgrp"] {
        assert!(
            !product.contains(forbidden),
            "the product half of {LAUNCH_FILE} names `{forbidden}`"
        );
    }
}

// ===========================================================================
// Direct-child identity stays descriptor-based
// ===========================================================================

#[test]
fn the_direct_child_is_never_signalled_or_waited_for_by_numeric_pid() {
    let sources = rust_files("src");
    for (name, text) in &sources {
        for forbidden in ["waitpid", "wait4", "kill_process"] {
            let named = tokens(text).iter().any(|token| token == forbidden);
            assert!(
                !named,
                "{name} names `{forbidden}`: the direct child's lifecycle is pidfd-based"
            );
        }
    }
    let code = compact(&sources[LAUNCH_FILE]);
    // Every signal this slice sends goes through the process descriptor.
    assert_eq!(
        code.matches("pidfd_send_signal(").count(),
        1,
        "the launch slice must signal the direct child through exactly one site"
    );
    assert!(code.contains("pidfd_send_signal(self.child.pidfd(),signal)"));
    // Both the waits it performs are descriptor-addressed.
    assert_eq!(
        code.matches("WaitId::PidFd(self.child.pidfd())").count(),
        1,
        "the probe is not addressed by pidfd"
    );
}

// ===========================================================================
// The `Err` boundary and the absent claims
// ===========================================================================

#[test]
fn launch_returns_err_only_before_a_child_exists() {
    let code = compact(&rust_files("src")[LAUNCH_FILE]);
    // Exactly one fallible call, and it is the pre-child one.
    assert!(
        code.contains("backend::spawn_for_lifecycle(authorized).map_err(to_launch_error)?"),
        "the only `?` in launch is not the pre-child spawn"
    );
    let body = function_body(&code, "pubfnlaunch(authorized:AuthorizedLaunch)");
    assert_eq!(
        body.matches('?').count(),
        1,
        "launch has more than one fallible step, so a child could be lost behind an error: {body}"
    );
    assert!(
        body.contains("Ok(Observer::new(spawned).run(started))"),
        "launch does not end in an unconditional Ok once a child exists"
    );
}

#[test]
fn no_exec_success_containment_or_authenticity_vocabulary_exists() {
    let sources = rust_files("src");
    for (name, text) in &sources {
        let words = tokens(text);
        for forbidden in [
            "ExecSucceeded",
            "exec_succeeded",
            "is_success",
            "contained",
            "containment",
            "sandbox",
            "sandboxed",
            "signature",
            "signed",
            "verify_receipt",
            "authenticate",
        ] {
            assert!(
                !words.iter().any(|token| token == forbidden),
                "{name} names `{forbidden}`, which is a claim this crate does not make"
            );
        }
    }
}

#[test]
fn the_receipt_record_carries_no_time_pid_or_descriptor_field() {
    let record = compact(&rust_files("src")["src/model.rs"]);
    let start = record
        .find("pubstructReceiptRecord{")
        .expect("the record exists");
    let body = &record[start..];
    let end = body.find('}').expect("the record ends");
    let fields = &body[..end];
    for forbidden in [
        "timestamp",
        "time",
        "elapsed",
        "duration",
        "pid",
        "fd",
        "path",
        "host",
        "signature",
    ] {
        assert!(
            !fields.contains(forbidden),
            "the receipt record has a `{forbidden}`-shaped field"
        );
    }
}

// ===========================================================================
// The public surface
// ===========================================================================

#[test]
fn the_public_outcome_exposes_no_descriptor_pid_or_backend_type() {
    let code = compact(&rust_files("src")[LAUNCH_FILE]);
    let start = code
        .find("pubstructLaunchOutcome{")
        .expect("the outcome exists");
    let body = &code[start..];
    let end = body.find('}').expect("the outcome ends");
    let fields = &body[..end];
    for forbidden in [
        "OwnedFd",
        "BorrowedFd",
        "RawFd",
        "ChildHandle",
        "pidfd",
        "pid",
        "Pid",
        "AuthorizedLaunch",
    ] {
        assert!(
            !fields.contains(forbidden),
            "LaunchOutcome holds a `{forbidden}`, so a launch would outlive its own descriptors"
        );
    }
    // And it is not constructible, copyable or serialisable from outside.
    for forbidden in ["derive(Clone", "Serialize", "Deserialize", "Default"] {
        assert!(
            !fields.contains(forbidden),
            "LaunchOutcome derives `{forbidden}`"
        );
    }
    assert!(
        code.contains("implcore::fmt::DebugforLaunchOutcome"),
        "LaunchOutcome must print lengths rather than derive Debug over its bytes"
    );
}

#[test]
fn the_public_launch_is_gated_on_the_linux_x86_64_cohort() {
    // Whitespace-compacted **raw** source: `strip` removes string literals, and
    // the cohort gate is spelled with them.
    let sources = rust_files("src");
    let lib: String = sources["src/lib.rs"]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("")
        .split_whitespace()
        .collect();
    let gate = r#"#[cfg(all(target_os="linux",target_arch="x86_64"))]"#;
    for gated in ["modlaunch;", "pubuselaunch::{LaunchOutcome,launch};"] {
        let at = lib.find(gated).expect("the launch module is declared");
        let before = &lib[..at];
        assert!(
            before.ends_with(gate),
            "`{gated}` is not immediately preceded by the cohort gate"
        );
    }
}

// ===========================================================================
// Bounded work per turn, and one bounded direct-child lifecycle
// ===========================================================================

#[test]
fn a_ready_stream_is_read_at_most_once_per_observation_turn() {
    // `F-P4-01`. A drain that looped to `EAGAIN` would let a child that
    // produces output faster than the parent can count and hash it hold the
    // loop away from its deadlines, from the other stream and from the process
    // descriptor — and `launch` would not return until the child chose to stop.
    // The read primitive therefore performs **one** read and returns.
    let sources = rust_files("src");
    let code = compact(&sources[LAUNCH_FILE]);
    let body = function_body(
        &code,
        "fnread_once(fd:BorrowedFd<'_>,buffer:&mut[u8],mutabsorb:implFnMut(&[u8]))->Read",
    );
    assert!(
        body.contains("returnRead::Progressed;"),
        "a successful read does not end the turn: {body}"
    );
    // The only loop it contains is the bounded `EINTR` retry.
    assert_eq!(
        body.matches("for_in0..MAX_READ_EINTR_RETRIES").count(),
        1,
        "the read primitive does not use the bounded retry form: {body}"
    );
    assert!(
        !body.contains("loop{"),
        "the read primitive still contains an unbounded loop: {body}"
    );
    // And nothing else in the slice reads a descriptor.
    assert_eq!(
        code.matches("rustix::io::read(").count(),
        1,
        "the launch slice reads descriptors from more than one place"
    );
    assert_eq!(
        code.matches("read_once(").count(),
        3,
        "expected exactly one definition and two callers of the bounded read"
    );
    // The per-turn budget is the accepted plan constant, not an ad-hoc size.
    assert!(
        code.contains("constREAD_BUFFER_BYTES:usize=65_536;"),
        "the per-turn read budget is not the accepted READ_BUFFER_BYTES"
    );
    // The status channel gets its own fixed, tiny accumulator.
    assert!(
        code.contains("constSTATUS_BUFFER_BYTES:usize=STATUS_RECORD_BYTES+1;"),
        "the status accumulator is not the fixed record-plus-detector size"
    );
}

#[test]
fn the_total_bound_counts_one_post_kill_wait_and_the_drop_guard_is_handed_back() {
    // `F-P4-02`. Two halves of one claim: the implementation's bound is the
    // accepted formula with `POST_KILL_REAP_MS` contributing exactly once, and
    // the P3 drop guard is disarmed once the accepted lifecycle has finished,
    // so no second unrecorded `SIGKILL` and no second bounded wait can happen
    // inside `launch`.
    let sources = rust_files("src");
    let launch = compact(&sources[LAUNCH_FILE]);
    let body = function_body(
        &launch,
        "fntotal_bound_ms(timeout_ms:u32,grace_ms:u32)->u64",
    );
    assert_eq!(
        body.matches("POST_KILL_REAP_MS").count(),
        1,
        "the implementation bound counts the post-kill wait more than once: {body}"
    );
    assert_eq!(body.matches("POST_EXIT_DRAIN_MS").count(), 1);
    assert!(body.contains("SPAWN_CONFIRM_TIMEOUT_MS"));

    // Exactly one hand-off, and it is after the model's own completion.
    assert_eq!(
        launch
            .matches("self.child.disarm_drop_cleanup_after_lifecycle();")
            .count(),
        1,
        "the drop guard is handed back from more or less than one place"
    );
    let run = function_body(&launch, "fnrun(mutself,started:Instant)->LaunchOutcome");
    let observed = run
        .find("letfacts=self.observe();")
        .expect("run observes first");
    let disarmed = run
        .find("self.child.disarm_drop_cleanup_after_lifecycle();")
        .expect("run hands the guard back");
    assert!(
        observed < disarmed,
        "the guard is handed back before the lifecycle finished: {run}"
    );

    // The guard itself: armed on construction, and the only early return in
    // `Drop` is the already-reaped or handed-back one.
    let spawn = compact(&sources["src/backend/spawn.rs"]);
    assert!(
        spawn.contains("cleanup_armed:Cell::new(true),"),
        "a fresh child handle does not own the cleanup responsibility"
    );
    assert!(
        spawn.contains("ifself.reaped.get()||!self.cleanup_armed.get(){return;}"),
        "the drop guard does not honour the hand-off"
    );
    assert_eq!(
        spawn
            .matches("fndisarm_drop_cleanup_after_lifecycle(&self)")
            .count(),
        1
    );
    // Nothing leaks the child instead of closing it.
    for forbidden in ["mem::forget", "ManuallyDrop", "into_raw_fd"] {
        assert!(
            !spawn.contains(forbidden),
            "src/backend/spawn.rs names `{forbidden}`"
        );
    }
}

#[test]
fn the_test_only_coordination_seam_is_not_reachable_from_the_public_launch() {
    // `F-P4-05` and `F-P4-06` need deterministic parent group authority, and
    // they get it from the **existing** pre-exec stall injection. That seam is
    // compiled only in this crate's own test build with the non-default feature
    // and debug assertions, it is not public, and `launch` does not name it.
    let sources = rust_files("src");
    let raw = &sources[LAUNCH_FILE];
    let compacted: String = raw
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("")
        .split_whitespace()
        .collect();
    let gate = r#"#[cfg(all(test,all(feature="test-fault-injection",debug_assertions)))]"#;
    let at = compacted
        .find("pub(crate)fnlaunch_with_stall_coordination(")
        .expect("the coordinated seam exists");
    assert!(
        compacted[..at].ends_with(gate),
        "the coordinated seam is not gated on cfg(test) and the two-condition feature gate"
    );
    assert!(
        !compacted.contains("pubfnlaunch_with_stall_coordination"),
        "the coordinated seam is public"
    );

    let code = compact(raw);
    let body = function_body(&code, "pubfnlaunch(authorized:AuthorizedLaunch)");
    for forbidden in ["stall", "Fault", "fault", "coordination"] {
        assert!(
            !body.contains(forbidden),
            "the public launch names `{forbidden}`: {body}"
        );
    }
    // The product path still asks the backend for no fault at all.
    let backend = compact(&sources["src/backend/mod.rs"]);
    let spawn_body = function_body(
        &backend,
        "pub(crate)fnspawn_for_lifecycle(authorized:AuthorizedLaunch,)->Result<SpawnedForLifecycle,BackendError>",
    );
    assert!(
        spawn_body.contains("spawn::spawn(&prepared,Fault::default())?"),
        "the product spawn no longer requests the default (absent) fault: {spawn_body}"
    );
}

// ===========================================================================
// The manifest
// ===========================================================================

#[test]
fn the_manifest_adds_only_the_event_feature_and_no_dependency() {
    let manifest: String = CRATE_MANIFEST.split_whitespace().collect();
    assert!(
        manifest.contains(r#"features=["std","fs","process","pipe","event"]"#),
        "the rustix feature set is not the accepted P4 one"
    );
    assert!(
        !manifest.contains(r#""time""#),
        "the `time` feature was enabled; monotonic deadlines use std::time::Instant"
    );
    for forbidden in ["linux-raw-sys", "signal-hook", "nix", "tokio", "async"] {
        assert!(
            !manifest.contains(forbidden),
            "the manifest gained a `{forbidden}` dependency"
        );
    }
    assert_eq!(
        manifest.matches(r#"rustix={version="=1.1.4""#).count(),
        2,
        "the pinned rustix version or its two cohort-gated tables changed"
    );
}
