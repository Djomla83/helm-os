//! P3 boundary inspection: unsafe confinement, the closed child vocabulary,
//! the backend lint policy, the release absence of fault injection, and the
//! public absence of every backend surface.
//!
//! # Why this file reads the tree
//!
//! `tests/p2_boundary.rs` embeds a fixed list of files with `include_str!`,
//! which is exact but cannot notice a **new** file. A confinement claim has to:
//! "unsafe appears only under `src/backend/`" is worth nothing if a file that
//! was never on the list may contain it. This file therefore walks
//! `crates/helm-launch/src` and `crates/helm-launch/tests` at run time, fails on
//! any file the expected inventory does not name, and then makes its claims over
//! everything it found.
//!
//! It runs on **every** platform, because the tree is the same everywhere: the
//! Linux x86_64 gate decides what is compiled, not what exists. It reads files
//! and executes nothing.
//!
//! # What it claims
//!
//! * `unsafe` appears **as code** only in the four backend product files, and
//!   the one scoped `#![allow(unsafe_code)]` exists only at the backend module
//!   boundary. No test source, and no other module, relaxes the lint.
//! * There is exactly **one** raw syscall implementation: one `asm!`.
//! * The child window's file names no `std`, `alloc`, `rustix`, allocation,
//!   formatting, printing or panicking vocabulary, and carries
//!   `#![no_implicit_prelude]`.
//! * The backend contains no `extern` block, no `libc` **call**, no
//!   `std::process::Command`, no forbidden clone flag, no `pidfd_open`, no
//!   `fork`, no `/proc/self/fd` execution and **no process-group signal**.
//! * The fault injection is gated on the feature **and** `debug_assertions`, at
//!   every site.
//! * No backend item is public, and an external caller cannot consume an
//!   `AuthorizedLaunch` into process creation.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code: a wrong assumption must fail loudly"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const CRATE_MANIFEST: &str = include_str!("../Cargo.toml");

/// The complete expected inventory of `src/`. A file that is not here fails the
/// inventory test, so nothing can be added outside the scans below.
const EXPECTED_SOURCES: [&str; 15] = [
    "src/authority.rs",
    "src/backend/child.rs",
    "src/backend/injection.rs",
    "src/backend/mod.rs",
    "src/backend/spawn.rs",
    "src/backend/syscall.rs",
    "src/backend/tests.rs",
    "src/error.rs",
    "src/launch.rs",
    "src/layout.rs",
    "src/lib.rs",
    "src/lifecycle.rs",
    "src/model.rs",
    "src/plan.rs",
    "src/receipt.rs",
];

/// The complete expected inventory of `tests/`.
const EXPECTED_TESTS: [&str; 6] = [
    "tests/linux_admission.rs",
    "tests/p2_boundary.rs",
    "tests/p3_boundary.rs",
    "tests/p4_boundary.rs",
    "tests/p5_regressions.rs",
    "tests/plan_contract.rs",
];

/// The backend product files: the only ones in which the `unsafe` token may
/// appear as code at all, and the only ones the owner authorised for it.
const BACKEND_PRODUCT: [&str; 5] = [
    "src/backend/child.rs",
    "src/backend/injection.rs",
    "src/backend/mod.rs",
    "src/backend/spawn.rs",
    "src/backend/syscall.rs",
];

/// The files in which it actually appears. `src/backend/mod.rs` carries the
/// scoped `allow` and the plain-old-data contract, and performs no unsafe
/// operation of its own.
const UNSAFE_PRESENT: [&str; 4] = [
    "src/backend/child.rs",
    "src/backend/injection.rs",
    "src/backend/spawn.rs",
    "src/backend/syscall.rs",
];

/// The backend's own test module is under `src/backend/`, so the scoped `allow`
/// covers it — but it must still not contain any unsafe operation of its own.
const BACKEND_TEST_FILE: &str = "src/backend/tests.rs";

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under one directory of the crate, as `dir/relative` names.
fn rust_files(directory: &str) -> BTreeMap<String, String> {
    fn walk(base: &Path, at: &Path, prefix: &str, into: &mut BTreeMap<String, String>) {
        for entry in std::fs::read_dir(at).unwrap_or_else(|e| panic!("read {at:?}: {e}")) {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, prefix, into);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let relative = path
                    .strip_prefix(base)
                    .expect("inside the crate")
                    .to_string_lossy()
                    .replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect("read a source file");
                into.insert(format!("{prefix}{relative}"), text);
            }
        }
    }
    let base = crate_root().join(directory);
    let mut found = BTreeMap::new();
    walk(&base, &base, &format!("{directory}/"), &mut found);
    found
}

/// Rust code with comments blanked and every string, raw-string and character
/// literal blanked. Structure is kept, so documentation that describes what the
/// crate does *not* do cannot trip a scan.
fn strip(source: &str) -> String {
    let bytes: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut at = 0_usize;
    while at < bytes.len() {
        let c = bytes[at];
        let next = bytes.get(at + 1).copied().unwrap_or('\0');
        // Line comment.
        if c == '/' && next == '/' {
            while at < bytes.len() && bytes[at] != '\n' {
                out.push(' ');
                at += 1;
            }
            continue;
        }
        // Block comment, nesting as Rust does.
        if c == '/' && next == '*' {
            let mut depth = 1_usize;
            out.push_str("  ");
            at += 2;
            while at < bytes.len() && depth > 0 {
                let c = bytes[at];
                let next = bytes.get(at + 1).copied().unwrap_or('\0');
                if c == '/' && next == '*' {
                    depth += 1;
                    out.push_str("  ");
                    at += 2;
                    continue;
                }
                if c == '*' && next == '/' {
                    depth -= 1;
                    out.push_str("  ");
                    at += 2;
                    continue;
                }
                out.push(if c == '\n' { '\n' } else { ' ' });
                at += 1;
            }
            continue;
        }
        // Raw string, with any number of hashes.
        if (c == 'r' || (c == 'b' && next == 'r')) && {
            let mut probe = at + if c == 'b' { 2 } else { 1 };
            while bytes.get(probe) == Some(&'#') {
                probe += 1;
            }
            bytes.get(probe) == Some(&'"')
        } {
            let mut probe = at + if c == 'b' { 2 } else { 1 };
            let mut hashes = 0_usize;
            while bytes.get(probe) == Some(&'#') {
                hashes += 1;
                probe += 1;
            }
            out.push('"');
            at = probe + 1;
            let closing: String = std::iter::once('"')
                .chain(std::iter::repeat_n('#', hashes))
                .collect();
            let rest: String = bytes[at..].iter().collect();
            let end = rest.find(&closing).map_or(bytes.len(), |i| at + i);
            for c in &bytes[at..end] {
                out.push(if *c == '\n' { '\n' } else { ' ' });
            }
            out.push('"');
            at = end + closing.len();
            continue;
        }
        // Ordinary string or character literal.
        if c == '"'
            || (c == '\'' && bytes.get(at + 2) == Some(&'\''))
            || (c == '\'' && next == '\\')
        {
            let quote = c;
            out.push(quote);
            at += 1;
            while at < bytes.len() {
                if bytes[at] == '\\' {
                    out.push_str("  ");
                    at += 2;
                    continue;
                }
                if bytes[at] == quote {
                    break;
                }
                out.push(if bytes[at] == '\n' { '\n' } else { ' ' });
                at += 1;
            }
            out.push(quote);
            at += 1;
            continue;
        }
        out.push(c);
        at += 1;
    }
    out
}

/// Identifier-shaped tokens of stripped code.
fn code_tokens(source: &str) -> Vec<String> {
    strip(source)
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Stripped code with all whitespace removed, for substring pinning.
fn compact(source: &str) -> String {
    strip(source).split_whitespace().collect()
}

// ===========================================================================
// Inventory
// ===========================================================================

#[test]
fn the_source_and_test_inventory_is_exactly_what_the_scans_cover() {
    let sources: BTreeSet<String> = rust_files("src").into_keys().collect();
    assert_eq!(
        sources,
        EXPECTED_SOURCES.iter().map(|s| (*s).to_owned()).collect(),
        "a source file was added or removed; every scan below must be reviewed for it"
    );
    let tests: BTreeSet<String> = rust_files("tests").into_keys().collect();
    assert_eq!(
        tests,
        EXPECTED_TESTS.iter().map(|s| (*s).to_owned()).collect(),
        "a test file was added or removed"
    );
    // The backend is exactly the authorised directory and nothing else: no
    // other subdirectory of `src/` exists.
    for name in &sources {
        let inside = name
            .strip_prefix("src/")
            .expect("every source is under src/");
        assert!(
            !inside.contains('/') || inside.starts_with("backend/"),
            "{name} is in a subdirectory of src/ other than backend/"
        );
    }
    // P4 added `src/launch.rs`. It must exist, and it must be a sibling of the
    // backend rather than a part of it: the unsafe boundary is a directory, and
    // the public lifecycle is deliberately outside it.
    assert!(
        sources.contains("src/launch.rs"),
        "the P4 launch slice is missing"
    );
    assert!(
        !sources
            .iter()
            .any(|name| name.starts_with("src/backend/launch")),
        "the launch slice moved inside the unsafe boundary"
    );
}

// ===========================================================================
// Unsafe confinement
// ===========================================================================

#[test]
fn unsafe_appears_as_code_only_under_the_backend() {
    let word = ["un", "safe"].concat();
    let mut naming: BTreeSet<String> = BTreeSet::new();
    for (name, source) in rust_files("src").into_iter().chain(rust_files("tests")) {
        if code_tokens(&source).contains(&word) {
            naming.insert(name);
        }
    }
    let permitted: BTreeSet<String> = BACKEND_PRODUCT.iter().map(|s| (*s).to_owned()).collect();
    assert!(
        naming.is_subset(&permitted),
        "the {word} token appears as code outside the authorised backend files: {naming:?}"
    );
    assert_eq!(
        naming,
        UNSAFE_PRESENT.iter().map(|s| (*s).to_owned()).collect(),
        "the set of files holding an {word} operation changed"
    );
}

#[test]
fn the_backend_test_module_contains_no_unsafe_operation() {
    // It lives under `src/backend/`, so the scoped `allow` covers it. It must
    // still not use the escape hatch: the tests observe the backend, they do
    // not extend it.
    let word = ["un", "safe"].concat();
    let sources = rust_files("src");
    let tests = &sources[BACKEND_TEST_FILE];
    assert!(
        !code_tokens(tests).contains(&word),
        "{BACKEND_TEST_FILE} contains an {word} operation"
    );
}

#[test]
fn exactly_one_scoped_allow_of_the_unsafe_lint_exists_and_it_is_the_backend_boundary() {
    let sources = rust_files("src");
    let mut relaxing = Vec::new();
    for (name, source) in &sources {
        let code = compact(source);
        for spelling in [
            "allow(unsafe_code",
            "allow(unsafe_op_in_unsafe_fn",
            "expect(unsafe_code",
            "warn(unsafe_code",
            "allow(unsafe_code,",
        ] {
            if code.contains(spelling) {
                relaxing.push(name.clone());
                break;
            }
        }
    }
    assert_eq!(
        relaxing,
        vec!["src/backend/mod.rs".to_owned()],
        "the scoped allow must exist exactly at the backend module boundary"
    );
    // It is an inner attribute of that module, so it covers the backend and
    // nothing else, and the module's own lint block re-denies the discipline.
    let backend = compact(&sources["src/backend/mod.rs"]);
    assert!(backend.contains("#![allow(unsafe_code)]"));
    assert!(
        !backend.contains("#![allow(unsafe_code,"),
        "the allow list widened"
    );
    for required in [
        "clippy::indexing_slicing",
        "clippy::arithmetic_side_effects",
        "clippy::as_conversions",
        "clippy::missing_safety_doc",
        "clippy::undocumented_unsafe_blocks",
        "clippy::multiple_unsafe_ops_per_block",
        "unsafe_op_in_unsafe_fn",
    ] {
        assert!(
            backend.contains(&format!("#![deny({required}"))
                || backend.contains(&format!(",{required}")),
            "the backend does not deny {required}"
        );
    }
    // The crate root still denies, and still does not forbid.
    let root = compact(&sources["src/lib.rs"]);
    assert!(root.contains("#![deny(unsafe_code,unsafe_op_in_unsafe_fn)]"));
    assert!(
        !root.contains("forbid("),
        "the crate root forbids, leaving no room for the scoped allow"
    );
}

#[test]
fn no_test_source_relaxes_the_unsafe_lint() {
    for (name, source) in rust_files("tests") {
        let code = compact(&source);
        for spelling in [
            "allow(unsafe_code",
            "expect(unsafe_code",
            "allow(unsafe_op_in_unsafe_fn",
        ] {
            assert!(!code.contains(spelling), "{name} relaxes the unsafe lint");
        }
    }
}

#[test]
fn there_is_exactly_one_raw_syscall_implementation() {
    let sources = rust_files("src");
    let mut with_assembly: Vec<&str> = Vec::new();
    let mut blocks = 0_usize;
    for (name, source) in &sources {
        let tokens = code_tokens(source);
        let count = tokens.iter().filter(|t| *t == "asm").count();
        if count > 0 {
            with_assembly.push(name);
            blocks += count;
        }
        for forbidden in ["global_asm", "naked_asm", "no_mangle", "link_section"] {
            assert!(
                !tokens.iter().any(|t| t == forbidden),
                "{name} uses `{forbidden}`"
            );
        }
    }
    assert_eq!(
        with_assembly,
        vec!["src/backend/syscall.rs"],
        "assembly may exist only in the one reviewed syscall module"
    );
    // `use core::arch::asm;` plus exactly one `asm!` invocation.
    assert_eq!(blocks, 2, "more than one assembly block exists");
    let shim = compact(&sources["src/backend/syscall.rs"]);
    assert_eq!(
        shim.matches("asm!(").count(),
        1,
        "more than one asm! invocation"
    );
    let shim_raw: String = sources["src/backend/syscall.rs"]
        .split_whitespace()
        .collect();
    assert!(
        shim_raw.contains(r#"asm!("syscall","#),
        "the shim is not the syscall instruction"
    );
    // The two options that would be wrong for a general syscall are absent, and
    // the two clobbers the instruction really destroys are declared.
    for wrong in ["nomem", "preserves_flags", "readonly", "pure"] {
        assert!(
            !shim.contains(&format!("options({wrong}")) && !shim.contains(&format!(",{wrong}")),
            "the shim claims `{wrong}`"
        );
    }
    assert!(
        !shim.contains("options("),
        "the shim requests an asm! option"
    );
    assert!(
        shim_raw.contains(r#"lateout("rcx")_,"#),
        "rcx is not declared clobbered"
    );
    assert!(
        shim_raw.contains(r#"lateout("r11")_,"#),
        "r11 is not declared clobbered"
    );
    assert!(shim_raw.contains(r#"inlateout("rax")nr=>result,"#));
    for (register, argument) in [
        ("rdi", "a1"),
        ("rsi", "a2"),
        ("rdx", "a3"),
        ("r10", "a4"),
        ("r8", "a5"),
        ("r9", "a6"),
    ] {
        assert!(
            shim_raw.contains(&format!(r#"in("{register}"){argument},"#)),
            "the shim does not place {argument} in {register}"
        );
    }
}

#[test]
fn every_unsafe_block_in_the_backend_carries_its_own_safety_comment() {
    // `clippy::undocumented_unsafe_blocks` enforces this at compile time; this
    // fails the same way without clippy, and also pins that no single block was
    // widened to cover the whole file.
    let word = ["un", "safe"].concat();
    let sources = rust_files("src");
    for name in UNSAFE_PRESENT {
        let source = &sources[name];
        let blocks = code_tokens(source).iter().filter(|t| **t == word).count();
        let comments = source.matches("// SAFETY:").count();
        let safety_sections = source.matches("/// # Safety").count();
        assert!(
            blocks > 0,
            "{name} is listed as holding unsafe but has none"
        );
        assert!(
            comments + safety_sections >= blocks / 2,
            "{name}: {blocks} unsafe tokens but only {comments} SAFETY comments and \
             {safety_sections} Safety sections"
        );
        assert!(
            !source.contains("#[allow(clippy::undocumented_unsafe_blocks)]"),
            "{name} silences the safety-comment lint"
        );
    }
}

// ===========================================================================
// The closed child window
// ===========================================================================

#[test]
fn the_child_file_has_no_implicit_prelude_and_a_closed_vocabulary() {
    let sources = rust_files("src");
    let child = &sources["src/backend/child.rs"];
    assert!(
        compact(child).contains("#![no_implicit_prelude]"),
        "the child module must not get the standard prelude"
    );

    // Nothing that allocates, locks, formats, prints, panics or calls into a
    // runtime may be named as code on the child path.
    const FORBIDDEN_IN_THE_CHILD: &[&str] = &[
        "std",
        "alloc",
        "rustix",
        "libc",
        "Vec",
        "String",
        "Box",
        "Rc",
        "Arc",
        "Mutex",
        "RefCell",
        "format",
        "println",
        "print",
        "eprintln",
        "write",
        "panic",
        "unwrap",
        "unreachable",
        "todo",
        "vec",
        "to_string",
        "collect",
        "iter",
        "clone",
        "drop",
        "Drop",
        "thread",
        "Instant",
        "Duration",
        "sleep",
        "malloc",
        "free",
        "transmute",
        "mmap",
        "fork",
        "vfork",
        "pidfd_open",
        "kill",
        "Command",
        "static_mut",
        "extern",
    ];
    let tokens = code_tokens(child);
    for token in &tokens {
        assert!(
            !FORBIDDEN_IN_THE_CHILD.contains(&token.as_str()),
            "src/backend/child.rs names `{token}` as code on the child path"
        );
    }
    // Panicking accessors are absent as calls, whatever an attribute spells.
    let child_code = compact(child);
    for call in [
        ".unwrap(",
        ".expect(",
        "panic!(",
        "assert!(",
        "unreachable!(",
    ] {
        assert!(
            !child_code.contains(call),
            "src/backend/child.rs uses `{call}` on the child path"
        );
    }
    // The scan is real: it does name the two things it is allowed to.
    assert!(tokens.iter().any(|t| t == "syscall"));
    assert!(tokens.iter().any(|t| t == "ChildPlan"));

    // The stage sequence, in the one accepted order, inside the entry point
    // itself: the range close is issued by `close_span`, which the entry point
    // calls three times in place.
    let code = compact(child);
    let entry_at = code
        .find("unsafefnchild_main(plan:&ChildPlan)->!")
        .expect("the child entry point");
    let entry_end = code[entry_at..]
        .find("unsafefnissue(")
        .map_or(code.len(), |end| entry_at + end);
    let entry = &code[entry_at..entry_end];
    let order = [
        "stage::DUP2",
        "stage::CLEAR_CLOEXEC",
        "stage::CHDIR",
        "close_span(plan,first)",
        "stage::SETPGID",
        "stage::SIGACTION",
        "stage::SIGMASK",
        "stage::NO_NEW_PRIVS",
        "stage::EXEC",
    ];
    let mut at = 0_usize;
    for stage in order {
        let found = entry[at..]
            .find(stage)
            .unwrap_or_else(|| panic!("{stage} is missing or out of order in the child sequence"));
        at += found + stage.len();
    }
    // Each stdio mapping is issued individually, and the range close is the
    // only stage the entry point delegates.
    assert_eq!(entry.matches("stage::DUP2").count(), 3);
    assert_eq!(entry.matches("stage::CLEAR_CLOEXEC").count(), 3);
    assert_eq!(entry.matches("close_span(plan,").count(), 3);
    assert!(
        code[entry_end..].contains("stage::CLOSE_RANGE"),
        "close_span does not report the CLOSE_RANGE stage"
    );
    assert!(
        !entry.contains("stage::CLOSE_RANGE"),
        "the entry point issues a range close directly"
    );
    // Exactly the accepted syscalls, and no other.
    for required in [
        "NR_DUP2",
        "NR_FCNTL",
        "NR_FCHDIR",
        "NR_CLOSE_RANGE",
        "NR_SETPGID",
        "NR_RT_SIGACTION",
        "NR_RT_SIGPROCMASK",
        "NR_PRCTL",
        "NR_EXECVEAT",
        "NR_WRITE",
        "NR_EXIT_GROUP",
    ] {
        assert!(code.contains(required), "the child never issues {required}");
    }
    for forbidden in [
        "NR_CLONE3",
        "NR_OPENAT",
        "NR_MMAP",
        "NR_GETPID",
        "NR_KILL",
        "NR_WAITID",
        "NR_PIDFD_SEND_SIGNAL",
    ] {
        assert!(!code.contains(forbidden), "the child issues {forbidden}");
    }
    // `read` is reached only by the test-only stall injection, which lives in
    // its own gated file, never in the child's own sequence.
    assert!(!code.contains("NR_READ"), "the child's own sequence reads");
}

#[test]
fn the_child_exits_only_by_exec_or_by_one_record_and_exit_group() {
    let sources = rust_files("src");
    let child = compact(&sources["src/backend/child.rs"]);
    assert!(
        child.contains("fnchild_main(plan:&ChildPlan)->!"),
        "the child entry point must borrow the plan, never dereference a raw pointer: a raw \
         dereference emits compiler null and alignment checks that call the panic runtime"
    );
    assert!(
        child.contains("#[inline(never)]"),
        "the child entry point must stay out of line so the machine-code gate always has a root"
    );
    // No whole-plan copy: a by-value read of the record lowers to a `memcpy`
    // call into libc in an unoptimised build, which the closed child contract
    // forbids (owner disposition of P3R-02).
    assert!(
        !child.contains("letplan:ChildPlan="),
        "the child copies the whole plan, which lowers to a memcpy call"
    );
    assert!(
        !child.contains("*constChildPlan"),
        "a raw plan pointer reappeared in the child"
    );
    assert!(
        child.contains("let[first,second,third]=&plan.close_ranges;"),
        "the close ranges must be borrowed, not copied out as a 72-byte aggregate"
    );
    assert!(child.contains("fnfail(plan:&ChildPlan,stage:u8,errno:i32)->!"));
    // The record is built by destructuring, never by indexing or formatting.
    assert!(child.contains("letrecord:[u8;8]=[stage,pad1,pad2,pad3,e0,e1,e2,e3];"));
    assert!(child.contains("fnclose_span(plan:&ChildPlan,span:&CloseSpan)"));
    assert!(child.contains("errno.to_le_bytes()"));
    // The failure write retries on EINTR and on nothing else.
    assert!(child.contains("syscall::EINTR"));
}

// ===========================================================================
// The backend's forbidden operations
// ===========================================================================

#[test]
fn the_backend_uses_no_foreign_interface_and_no_libc_function() {
    let sources = rust_files("src");
    for name in BACKEND_PRODUCT {
        let tokens = code_tokens(&sources[name]);
        for forbidden in [
            "extern",
            "transmute",
            "from_raw_parts",
            "assume_init",
            "MaybeUninit",
            "static_mut",
            "fork",
            "vfork",
            "pidfd_open",
            "posix_spawn",
            "fexecve",
            "Command",
            "dlopen",
            "dlsym",
        ] {
            assert!(
                !tokens.iter().any(|t| t == forbidden),
                "{name} names `{forbidden}`"
            );
        }
        // `libc` may be named only inside a constant assertion.
        let code = compact(&sources[name]);
        for call in ["libc::syscall", "libc::fork", "libc::clone", "libc::kill"] {
            assert!(!code.contains(call), "{name} calls {call}");
        }
    }
    // `libc` is used for constants only: every mention is inside a `const _`
    // assertion in the one module that pins the ABI numbers.
    let with_libc: Vec<&str> = BACKEND_PRODUCT
        .iter()
        .filter(|name| code_tokens(&sources[**name]).iter().any(|t| t == "libc"))
        .copied()
        .collect();
    assert_eq!(with_libc, vec!["src/backend/syscall.rs"]);
    let shim = compact(&sources["src/backend/syscall.rs"]);
    let mentions = shim.matches("libc::").count();
    let pinned = shim.matches("const_:()=assert!(libc::").count();
    assert!(
        mentions > 20,
        "the ABI pinning shrank to {mentions} constants"
    );
    assert_eq!(
        mentions, pinned,
        "a `libc::` mention is not inside a const assertion: {mentions} mentions, {pinned} pinned"
    );
}

#[test]
fn the_backend_sets_no_forbidden_clone_flag_and_never_falls_back() {
    let sources = rust_files("src");
    let shim = compact(&sources["src/backend/syscall.rs"]);
    // The flag word is exactly `CLONE_PIDFD`, proven at compile time.
    assert!(shim.contains("flags:CLONE_PIDFD,"));
    assert!(shim.contains("exit_signal:SIGCHLD,"));
    assert!(shim.contains("const_:()=assert!(CLONE_PIDFD.count_ones()==1);"));
    for forbidden in ["CLONE_VM", "CLONE_FILES", "CLONE_VFORK", "CLONE_THREAD"] {
        assert!(
            shim.contains(&format!("const_:()=assert!(CLONE_PIDFD&{forbidden}==0);")),
            "the clone flags are not proven free of {forbidden}"
        );
    }
    // No pathname execution and no procfs fallback anywhere in the backend.
    // Prose may say the crate does not read procfs; a **string literal** naming
    // one would be the defect, so the scan looks for a quoted path.
    for name in BACKEND_PRODUCT {
        let source = &sources[name];
        assert!(
            !source.contains(r#""/proc"#),
            "{name} carries a procfs path as a string literal"
        );
        let code = compact(source);
        assert!(!code.contains("execve("), "{name} executes a pathname");
        assert!(!code.contains("fexecve"), "{name} uses fexecve");
    }
}

#[test]
fn the_backend_issues_no_process_group_signal() {
    // The one authorised signal is `SIGKILL` to the direct child through its
    // pidfd. No negative pid, no `kill`, no group sweep: that is P4's.
    let sources = rust_files("src");
    for name in BACKEND_PRODUCT {
        let tokens = code_tokens(&sources[name]);
        for forbidden in ["kill", "killpg", "tgkill", "tkill", "SIGTERM", "Term"] {
            assert!(
                !tokens.iter().any(|t| t == forbidden),
                "{name} names `{forbidden}`, which would be a P4 lifecycle operation"
            );
        }
    }
    let spawn = compact(&sources["src/backend/spawn.rs"]);
    assert_eq!(
        spawn.matches("pidfd_send_signal(").count(),
        1,
        "the direct-child signal must have exactly one site"
    );
    assert!(
        spawn.contains("Signal::KILL"),
        "the one signal is not SIGKILL"
    );
    assert!(!spawn.contains("Signal::TERM"), "a SIGTERM path exists");
    // Group authority is recorded and never acted on.
    assert!(spawn.contains("group_authority_established"));
    assert!(spawn.contains("setpgid(Some(pid),Some(pid))"));

    // Waiting is by pidfd, never by pid.
    assert!(spawn.contains("WaitId::PidFd("));
    assert!(spawn.contains("WaitIdOptions::EXITED|WaitIdOptions::NOHANG"));
    for forbidden in [
        "waitpid",
        "wait4",
        "WaitId::Pid(",
        "WaitId::All",
        "WaitId::Pgid",
    ] {
        assert!(!spawn.contains(forbidden), "spawn.rs uses {forbidden}");
    }
}

#[test]
fn no_backend_product_file_creates_a_process_through_std() {
    let sources = rust_files("src");
    for name in BACKEND_PRODUCT {
        let code = compact(&sources[name]);
        for forbidden in ["std::process::Command", "process::Command", "Command::new"] {
            assert!(!code.contains(forbidden), "{name} uses {forbidden}");
        }
    }
    // The scan is real: the test module does use it, as the trusted caller that
    // proves a fixture can report on its own (the X2c rule).
    assert!(
        compact(&sources["src/backend/tests.rs"]).contains("Command::new"),
        "the producer self-test no longer runs a fixture directly"
    );
}

// ===========================================================================
// Fault injection
// ===========================================================================

#[test]
fn every_fault_injection_site_is_gated_on_the_feature_and_on_debug_assertions() {
    // The two-condition gate, as it appears inside `#[cfg(...)]`, `cfg!(...)`
    // and the `cfg(not(...))` complement alike.
    const GATE: &str = r#"all(feature="test-fault-injection",debug_assertions)"#;
    let sources = rust_files("src");
    let mut gated_files: BTreeSet<String> = BTreeSet::new();
    for (name, source) in &sources {
        // The gate lives inside a string literal, so this scan reads the raw
        // text with whitespace removed rather than stripped code. Prose may
        // name the feature; every `cfg` spelling of it must be the full
        // two-condition gate.
        let code: String = source.split_whitespace().collect();
        let conditions = code.matches(r#"feature="test-fault-injection""#).count();
        if conditions == 0 {
            continue;
        }
        gated_files.insert(name.clone());
        assert_eq!(
            code.matches(GATE).count(),
            conditions,
            "{name}: a `test-fault-injection` condition is not the two-condition gate"
        );
    }
    // `src/launch.rs` joined this inventory with the P4 correction: the
    // deterministic group-sweep and foreign-reaper regressions reuse the
    // existing pre-exec stall injection as **test-only coordination**, so the
    // gate is named there too. Every occurrence there additionally carries
    // `cfg(test)`, which the assertion above still requires to contain the full
    // two-condition gate, and the P4 boundary suite pins that the coordinated
    // entry point is not reachable from the public `launch`.
    assert_eq!(
        gated_files,
        [
            "src/backend/child.rs",
            "src/backend/injection.rs",
            "src/backend/mod.rs",
            "src/backend/tests.rs",
            "src/launch.rs",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect::<BTreeSet<String>>(),
        "the injection gate appears in an unexpected file"
    );

    // The injection module itself exists only behind that gate, and the marker
    // a release build must not contain is kept in the artifact.
    let root: String = sources["src/backend/mod.rs"].split_whitespace().collect();
    assert!(
        root.contains(&format!("#[cfg({GATE})]modinjection;")),
        "the injection module is not declared behind the two-condition gate"
    );
    // The complement branch exists exactly once, so the non-injection build has
    // a definition of its own rather than a hole.
    assert_eq!(root.matches(&format!("cfg(not({GATE}))")).count(), 1);
    let injection = compact(&sources["src/backend/injection.rs"]);
    assert!(injection.contains("#[used]staticPRESENCE_MARKER_KEPT:&str=PRESENCE_MARKER;"));
    assert!(
        sources["src/backend/injection.rs"].contains("helm_launch_p3_fault_injection_present"),
        "the release-absence marker changed; the CI check must change with it"
    );
    // S6 reconfirmation (owner disposition, 2026-09-18). The pre-exec stall is
    // deterministic rather than race-dependent because the parent deliberately
    // retains the stdin write end for the duration, so the child's blocking
    // read on 0 cannot reach end-of-file. That retention must be unable to
    // affect a normal product build: outside an injection build it is a
    // compile-time `false` constant, not a runtime branch.
    assert!(
        root.contains(&format!(
            "#[cfg({GATE})]letretain_stdin_writer=fault.mode==injection::MODE_STALL_BEFORE_EXEC;"
        )),
        "the retained stdin writer is not behind the two-condition gate"
    );
    assert!(
        root.contains(&format!("#[cfg(not({GATE}))]letretain_stdin_writer=false;")),
        "a normal build must define the retained stdin writer as a constant false"
    );

    // The feature is declared, non-default and enables nothing else.
    assert!(CRATE_MANIFEST.contains("test-fault-injection = []"));
    assert!(
        !CRATE_MANIFEST.contains("default = ["),
        "the injection feature became default"
    );
}

// ===========================================================================
// Public absence
// ===========================================================================

#[test]
fn no_backend_item_is_public() {
    let sources = rust_files("src");
    let root = compact(&sources["src/lib.rs"]);
    assert_eq!(root.matches("modbackend;").count(), 1);
    assert!(!root.contains("pubmodbackend"));
    assert!(!root.contains("pubusebackend"));
    assert!(!root.contains("pub(crate)usebackend"));

    // No backend surface name is re-exported under any spelling.
    for name in [
        "launch",
        "launch_minimal",
        "spawn",
        "SpawnedChild",
        "PreparedLaunch",
        "Backend",
        "LaunchOutcome",
        "ChildPlan",
        "ChildHandle",
        "MinimalLaunch",
        "BackendError",
        "pidfd",
        "AuthorizedParts",
    ] {
        for exported in [
            format!("pub{name}"),
            format!("pubuse{name}"),
            format!(",{name},"),
        ] {
            assert!(
                !root.contains(&format!("pubusebackend::{{{name}")),
                "{name} is re-exported"
            );
            let _ = exported;
        }
    }

    // The crate-private consumer of an authorisation is `pub(crate)`, never
    // `pub`, and exists in exactly one place.
    let authority = compact(&sources["src/authority.rs"]);
    assert_eq!(authority.matches("pub(crate)fninto_parts(self)").count(), 1);
    assert!(
        !authority.contains("pubfninto_parts"),
        "into_parts became public"
    );
    assert_eq!(
        authority
            .matches("pub(crate)structAuthorizedParts{")
            .count(),
        1
    );
    assert!(!authority.contains("pubstructAuthorizedParts"));

    // Every public item of the backend's own files is at most `pub(crate)`.
    for name in BACKEND_PRODUCT {
        let code = strip(&sources[name]);
        for line in code.lines().map(str::trim) {
            if let Some(rest) = line.strip_prefix("pub ") {
                panic!("{name} declares a bare `pub` item: pub {rest}");
            }
        }
    }
}

#[test]
fn the_crate_root_proves_the_narrowness_of_the_public_execution_path() {
    // The `compile_fail` doctests themselves run under `cargo test`; this pins
    // that they are the on-point ones, not an unrelated missing import.
    //
    // **P4 changed what is being proved, not how much.** `launch` and
    // `LaunchOutcome` now exist on the Linux x86_64 cohort, so the proofs that
    // they are absent moved into the off-cohort block and two new proofs were
    // added: a `LaunchOutcome` cannot be built from its parts, and an
    // authorisation cannot be launched twice.
    let sources = rust_files("src");
    let lib = &sources["src/lib.rs"];
    for proof in [
        "helm_launch::backend::launch_minimal",
        "a.into_parts()",
        "c.descriptor()",
        "helm_launch::LaunchOutcome { receipt: r }",
        "let _ = helm_launch::launch(a);",
        "use helm_launch::launch;",
        "use helm_launch::LaunchOutcome;",
        "use helm_launch::backend;",
        "use helm_launch::SpawnedChild;",
        "use helm_launch::PreparedLaunch;",
        "use helm_launch::ChildHandle;",
        "use helm_launch::MinimalLaunch;",
        "use helm_launch::Fault;",
    ] {
        assert!(
            lib.contains(proof),
            "the crate root has no compile-fail proof containing `{proof}`"
        );
    }
    // The two that must now be off-cohort only, so that the on-cohort build
    // cannot silently lose `launch` and still pass this file.
    let cohort_gate = "not(all(target_os = \"linux\", target_arch = \"x86_64\"))";
    let (_, off_cohort) = lib
        .split_once(cohort_gate)
        .expect("the crate root has no off-cohort documentation block");
    for proof in [
        "use helm_launch::launch;",
        "use helm_launch::LaunchOutcome;",
    ] {
        assert!(
            off_cohort.contains(proof),
            "`{proof}` is not inside the off-cohort block, so it would be a false claim on Linux"
        );
    }
}

// ===========================================================================
// Fixed bounds and absent lifecycle
// ===========================================================================

#[test]
fn only_the_two_authorised_fixed_bounds_exist() {
    let sources = rust_files("src");
    let backend = compact(&sources["src/backend/mod.rs"]);
    assert!(backend.contains("constSPAWN_CONFIRM_TIMEOUT_MS:u64=5_000;"));
    assert!(backend.contains("constPOST_KILL_REAP_MS:u64=5_000;"));
    // Nothing that would be a P4 run lifecycle.
    for name in BACKEND_PRODUCT {
        let code = compact(&sources[name]);
        for forbidden in [
            "POST_EXIT_DRAIN_MS",
            "MAX_CAPTURE_BYTES",
            "timeout_ms()",
            "grace_ms()",
            "run_deadline",
            "LaunchOutcome",
            "LaunchReceipt",
            "ReceiptRecord",
            "GroupSweep",
        ] {
            assert!(
                !code.contains(forbidden),
                "{name} names `{forbidden}`, which belongs to P4 or P5"
            );
        }
    }
    // And no `poll`: the observation loop that needs the `event` feature is
    // P4's, and it lives in `src/launch.rs`, outside the unsafe boundary.
    for name in BACKEND_PRODUCT {
        let tokens = code_tokens(&sources[name]);
        for forbidden in ["poll", "PollFd", "epoll", "select"] {
            assert!(
                !tokens.iter().any(|t| t == forbidden),
                "{name} names `{forbidden}`"
            );
        }
    }
}
