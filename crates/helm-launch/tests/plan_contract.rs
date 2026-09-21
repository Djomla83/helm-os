//! Adversarial launch-plan contract tests. Pure: no filesystem, process or
//! network access, so they run identically on Linux, Windows and macOS.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use helm_launch::{
    EnvironmentMode, ExecutionKind, LaunchPlanErrorCode as C, LaunchPlanErrors, MAX_ARG_BYTES,
    MAX_ARGS, MAX_CAPTURE_BYTES, MAX_GRACE_MS, MAX_PLAN_BYTES, MAX_TIMEOUT_MS, StdinMode, Stream,
    TerminationSignal, ValidatedLaunchPlan, parse_launch_plan,
};
use sha2::{Digest as _, Sha256};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";
const BINDING: &str = "d63d04e2a55a61fa149729f48bc355fb18f28e309f8aa2c973589391bf80e923";

/// Field values of a plan, each replaceable by raw JSON text.
#[derive(Clone)]
struct Doc {
    fields: Vec<(&'static str, String)>,
}

impl Doc {
    fn valid() -> Self {
        Self {
            fields: vec![
                ("schema", r#""helm-launch-plan""#.into()),
                ("version", r#""0.1""#.into()),
                ("execution_kind", r#""linux_exact_executable""#.into()),
                ("argv", r#"["tool","--flag","a b; c"]"#.into()),
                ("environment", r#"{"mode":"empty"}"#.into()),
                ("working_directory", r#"{"capability_id":"workdir"}"#.into()),
                ("stdin", r#"{"mode":"closed_pipe_eof"}"#.into()),
                ("stdout", r#"{"capture_prefix_bytes":0}"#.into()),
                ("stderr", r#"{"capture_prefix_bytes":4096}"#.into()),
                ("timeout_ms", "30000".into()),
                (
                    "termination",
                    r#"{"signal":"SIGTERM","grace_ms":5000}"#.into(),
                ),
            ],
        }
    }

    fn set(mut self, key: &'static str, raw: &str) -> Self {
        match self.fields.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = raw.to_owned(),
            None => self.fields.push((key, raw.to_owned())),
        }
        self
    }

    fn without(mut self, key: &str) -> Self {
        self.fields.retain(|(k, _)| *k != key);
        self
    }

    fn bytes(&self) -> Vec<u8> {
        self.bytes_with_extra("")
    }

    fn bytes_with_extra(&self, extra: &str) -> Vec<u8> {
        let body: Vec<String> = self
            .fields
            .iter()
            .map(|(k, v)| format!("\"{k}\":{v}"))
            .collect();
        format!("{{{}{extra}}}", body.join(",")).into_bytes()
    }
}

fn ok(bytes: &[u8]) -> ValidatedLaunchPlan {
    match parse_launch_plan(bytes) {
        Ok(plan) => plan,
        Err(e) => panic!("rejected {}: {e}", String::from_utf8_lossy(bytes)),
    }
}

fn err(bytes: &[u8]) -> LaunchPlanErrors {
    match parse_launch_plan(bytes) {
        Ok(_) => panic!("accepted {}", String::from_utf8_lossy(bytes)),
        Err(e) => e,
    }
}

fn codes(bytes: &[u8]) -> Vec<(C, &'static str, Option<usize>)> {
    err(bytes)
        .as_slice()
        .iter()
        .map(|e| (e.code(), e.locator(), e.index()))
        .collect()
}

fn only(bytes: &[u8], code: C, locator: &'static str) {
    assert_eq!(
        codes(bytes),
        vec![(code, locator, None)],
        "for {}",
        String::from_utf8_lossy(bytes)
    );
}

// ------------------------------------------------------------------ acceptance

#[test]
fn the_accepted_example_parses_into_inert_intent() {
    let raw = Doc::valid()
        .set(
            "asserted_context",
            &format!(
                r#"{{"subject_spec_sha256":"{SUBJECT}","binding_report_sha256":"{BINDING}"}}"#
            ),
        )
        .bytes();
    let plan = ok(&raw);
    assert_eq!(plan.exact_bytes(), raw.as_slice());
    assert_eq!(plan.execution_kind(), ExecutionKind::LinuxExactExecutable);
    assert_eq!(plan.argv(), ["tool", "--flag", "a b; c"]);
    assert_eq!(plan.environment_mode(), EnvironmentMode::Empty);
    assert_eq!(plan.working_directory_id(), "workdir");
    assert_eq!(plan.stdin_mode(), StdinMode::ClosedPipeEof);
    assert_eq!(plan.capture_prefix_bytes(Stream::Stdout), 0);
    assert_eq!(plan.capture_prefix_bytes(Stream::Stderr), 4096);
    assert_eq!(plan.timeout_ms(), 30_000);
    assert_eq!(plan.termination_signal(), TerminationSignal::Sigterm);
    assert_eq!(plan.grace_ms(), 5_000);
    assert_eq!(
        plan.asserted_subject_spec_sha256().unwrap().to_hex(),
        SUBJECT
    );
    assert_eq!(
        plan.asserted_binding_report_sha256().unwrap().to_hex(),
        BINDING
    );
}

#[test]
fn asserted_context_and_each_member_are_optional() {
    let plan = ok(&Doc::valid().bytes());
    assert_eq!(plan.asserted_subject_spec_sha256(), None);
    assert_eq!(plan.asserted_binding_report_sha256(), None);
    let plan = ok(&Doc::valid().set("asserted_context", "{}").bytes());
    assert_eq!(plan.asserted_subject_spec_sha256(), None);
    let plan = ok(&Doc::valid()
        .set(
            "asserted_context",
            &format!(r#"{{"binding_report_sha256":"{BINDING}"}}"#),
        )
        .bytes());
    assert_eq!(plan.asserted_subject_spec_sha256(), None);
    assert_eq!(
        plan.asserted_binding_report_sha256().unwrap().to_hex(),
        BINDING
    );
}

#[test]
fn parsing_returns_intent_only() {
    // The only entry point maps bytes to an inert value or to errors.
    let _: fn(&[u8]) -> Result<ValidatedLaunchPlan, LaunchPlanErrors> = parse_launch_plan;
    fn assert_plain_data<T: Clone + Send + Sync + core::fmt::Debug + PartialEq>() {}
    assert_plain_data::<ValidatedLaunchPlan>();
}

// ------------------------------------------------------------------- identity

/// Fixed plan identity vector. The expected digest was computed outside Rust
/// (Python `hashlib`) over exactly these bytes.
#[test]
fn a_fixed_plan_has_one_exact_byte_identity() {
    const PLAN: &[u8] = br#"{"schema":"helm-launch-plan","version":"0.1","execution_kind":"linux_exact_executable","argv":["tool","--flag","a b; c"],"environment":{"mode":"empty"},"working_directory":{"capability_id":"workdir"},"stdin":{"mode":"closed_pipe_eof"},"stdout":{"capture_prefix_bytes":0},"stderr":{"capture_prefix_bytes":4096},"timeout_ms":30000,"termination":{"signal":"SIGTERM","grace_ms":5000}}"#;
    let plan = ok(PLAN);
    println!(
        "HELM-LAUNCH-FIXTURE-PLAN: bytes={} sha256={}",
        PLAN.len(),
        plan.sha256()
    );
    assert_eq!(
        plan.sha256().to_hex(),
        "3597cffad48d156d87af28ccb956c4253fc74f8ad56709e5fdaf1fb12a074032"
    );
    assert_eq!(Doc::valid().bytes(), PLAN);
}

#[test]
fn identity_is_over_exact_bytes_without_canonicalisation() {
    let compact = Doc::valid().bytes();
    let spaced = String::from_utf8(compact.clone())
        .unwrap()
        .replace(",\"", ", \"")
        .into_bytes();
    let reordered = {
        let mut d = Doc::valid();
        d.fields.reverse();
        d.bytes()
    };
    let escaped = String::from_utf8(compact.clone())
        .unwrap()
        .replace("\"tool\"", "\"t\\u006fol\"")
        .into_bytes();
    let a = ok(&compact);
    let mut digests = vec![a.sha256()];
    for other in [&spaced, &reordered, &escaped] {
        let p = ok(other);
        assert_eq!(p.argv(), a.argv(), "same meaning");
        assert_eq!(p.exact_bytes(), other.as_slice());
        let independent: [u8; 32] = Sha256::digest(other).into();
        assert_eq!(p.sha256().as_bytes(), &independent);
        assert!(
            !digests.contains(&p.sha256()),
            "a different spelling shared an identity"
        );
        digests.push(p.sha256());
    }
    assert_eq!(ok(&compact), a, "parsing is deterministic");
}

// ----------------------------------------------------------- structural refusals

#[test]
fn size_bound_is_checked_first_and_is_exact() {
    let base = Doc::valid().bytes();
    let pad = MAX_PLAN_BYTES - base.len();
    let mut at_bound = base.clone();
    at_bound.splice(1..1, std::iter::repeat_n(b' ', pad));
    assert_eq!(at_bound.len(), MAX_PLAN_BYTES);
    ok(&at_bound);
    let mut over = at_bound.clone();
    over.insert(1, b' ');
    only(&over, C::InputTooLarge, "plan");
    // Checked before any parsing: garbage over the bound reports only size.
    only(&vec![0xff; MAX_PLAN_BYTES + 1], C::InputTooLarge, "plan");
}

#[test]
fn malformed_documents_return_one_document_level_finding() {
    let valid = Doc::valid().bytes();
    let mut trailing = valid.clone();
    trailing.extend_from_slice(b" {}");
    let mut bom = b"\xEF\xBB\xBF".to_vec();
    bom.extend_from_slice(&valid);
    let invalid_utf8 = String::from_utf8(valid.clone())
        .unwrap()
        .replace("\"tool\"", "\"to\u{0}ol\"")
        .into_bytes()
        .into_iter()
        .map(|b| if b == 0 { 0xff } else { b })
        .collect::<Vec<u8>>();
    let lone_surrogate = Doc::valid().set("argv", r#"["\ud800"]"#).bytes();
    for bytes in [
        b"".to_vec(),
        b"{".to_vec(),
        b"{\"schema\":}".to_vec(),
        trailing,
        bom,
        invalid_utf8,
        lone_surrogate,
        b"{\"schema\":\"helm-launch-plan\",}".to_vec(),
        b"nul".to_vec(),
    ] {
        only(&bytes, C::MalformedJson, "plan");
    }
    for bytes in [&b"[]"[..], b"\"x\"", b"1", b"null", b"true"] {
        only(bytes, C::NotJsonObject, "plan");
    }
}

#[test]
fn duplicate_decoded_keys_are_refused_at_every_depth() {
    let valid = String::from_utf8(Doc::valid().bytes()).unwrap();
    let cases = [
        // Top level, literal and escaped spellings.
        valid.replacen('{', r#"{"schema":"helm-launch-plan","#, 1),
        valid.replacen('{', r#"{"schema":"helm-launch-plan","#, 1),
        valid.replacen('{', r#"{"schema":"x","#, 1),
        // Nested objects.
        valid.replace(r#"{"mode":"empty"}"#, r#"{"mode":"empty","mode":"empty"}"#),
        valid.replace(
            r#"{"mode":"empty"}"#,
            r#"{"mode":"empty","mode":"inherit"}"#,
        ),
        valid.replace(
            r#"{"signal":"SIGTERM","grace_ms":5000}"#,
            r#"{"signal":"SIGTERM","grace_ms":5000,"grace_ms":0}"#,
        ),
        valid.replace(
            r#"{"capture_prefix_bytes":0}"#,
            r#"{"capture_prefix_bytes":0,"capture_prefix_bytes":65536}"#,
        ),
        // Objects nested inside arrays and inside unknown fields.
        valid.replace(r#"["tool","--flag","a b; c"]"#, r#"[{"a":1,"a":2}]"#),
        valid.replacen('{', r#"{"x":{"y":[{"k":1,"k":2}]},"#, 1),
        Doc::valid()
            .set(
                "asserted_context",
                &format!(
                    r#"{{"subject_spec_sha256":"{SUBJECT}","subject_spec_sha256":"{SUBJECT}"}}"#
                ),
            )
            .bytes()
            .into_iter()
            .map(char::from)
            .collect(),
    ];
    for case in cases {
        only(case.as_bytes(), C::DuplicateKey, "plan");
    }
}

#[test]
fn nesting_is_bounded() {
    let deep = |levels: usize| {
        let open = "[".repeat(levels);
        let close = "]".repeat(levels);
        Doc::valid().bytes_with_extra(&format!(",\"x\":{open}1{close}"))
    };
    // The root object is depth 0, so seven arrays still fit under the bound and
    // are merely an unknown field.
    only(&deep(7), C::UnknownField, "plan");
    only(&deep(8), C::NestingTooDeep, "plan");
    only(&deep(10_000), C::NestingTooDeep, "plan");
    let objects = format!("{}1{}", "{\"a\":".repeat(8), "}".repeat(8));
    only(
        &Doc::valid().bytes_with_extra(&format!(",\"x\":{objects}")),
        C::NestingTooDeep,
        "plan",
    );
}

// ------------------------------------------------------------- closed schema

#[test]
fn every_required_field_is_required() {
    for (key, locator) in [
        ("schema", "schema"),
        ("version", "version"),
        ("execution_kind", "execution_kind"),
        ("argv", "argv"),
        ("environment", "environment"),
        ("working_directory", "working_directory"),
        ("stdin", "stdin"),
        ("stdout", "stdout"),
        ("stderr", "stderr"),
        ("timeout_ms", "timeout_ms"),
        ("termination", "termination"),
    ] {
        only(&Doc::valid().without(key).bytes(), C::MissingField, locator);
    }
    for (key, raw, locator) in [
        ("environment", "{}", "environment.mode"),
        ("working_directory", "{}", "working_directory.capability_id"),
        ("stdin", "{}", "stdin.mode"),
        ("stdout", "{}", "stdout.capture_prefix_bytes"),
        ("stderr", "{}", "stderr.capture_prefix_bytes"),
        (
            "termination",
            r#"{"signal":"SIGTERM"}"#,
            "termination.grace_ms",
        ),
        ("termination", r#"{"grace_ms":1}"#, "termination.signal"),
    ] {
        only(
            &Doc::valid().set(key, raw).bytes(),
            C::MissingField,
            locator,
        );
    }
}

#[test]
fn fields_absent_by_design_are_unknown_fields() {
    for key in [
        "executable",
        "executable_path",
        "path",
        "program",
        "name",
        "command",
        "shell",
        "PATH",
        "env",
        "environment_variables",
        "fds",
        "stdin_fd",
        "expected_exit_code",
        "success_condition",
        "wine_prefix",
        "WINEPREFIX",
        "runtime",
        "cwd",
    ] {
        let raw = Doc::valid().bytes_with_extra(&format!(",\"{key}\":\"/bin/sh\""));
        only(&raw, C::UnknownField, "plan");
    }
    for (key, raw, locator) in [
        (
            "environment",
            r#"{"mode":"empty","variables":{"PATH":"/bin"}}"#,
            "environment",
        ),
        (
            "working_directory",
            r#"{"capability_id":"w","path":"/tmp"}"#,
            "working_directory",
        ),
        ("stdin", r#"{"mode":"closed_pipe_eof","fd":0}"#, "stdin"),
        (
            "stdout",
            r#"{"capture_prefix_bytes":0,"mode":"discard"}"#,
            "stdout",
        ),
        ("stderr", r#"{"capture_prefix_bytes":0,"fd":2}"#, "stderr"),
        (
            "termination",
            r#"{"signal":"SIGTERM","grace_ms":0,"then":"SIGKILL"}"#,
            "termination",
        ),
        (
            "asserted_context",
            r#"{"observation_artifact_sha256":"00"}"#,
            "asserted_context",
        ),
    ] {
        only(
            &Doc::valid().set(key, raw).bytes(),
            C::UnknownField,
            locator,
        );
    }
}

#[test]
fn constants_and_closed_modes_are_exact() {
    for (key, raw, code) in [
        ("schema", r#""helm-launch-plan ""#, C::UnknownSchema),
        ("schema", r#""HELM-LAUNCH-PLAN""#, C::UnknownSchema),
        ("version", r#""0.2""#, C::UnknownVersion),
        ("version", r#""0.10""#, C::UnknownVersion),
        (
            "execution_kind",
            r#""linux_path""#,
            C::ExecutionKindUnsupported,
        ),
        ("execution_kind", r#""wine""#, C::ExecutionKindUnsupported),
    ] {
        only(&Doc::valid().set(key, raw).bytes(), code, key);
    }
    for (key, raw, code, locator) in [
        (
            "environment",
            r#"{"mode":"inherit"}"#,
            C::EnvironmentModeUnsupported,
            "environment.mode",
        ),
        (
            "environment",
            r#"{"mode":"explicit"}"#,
            C::EnvironmentModeUnsupported,
            "environment.mode",
        ),
        (
            "stdin",
            r#"{"mode":"inherit"}"#,
            C::StdinModeUnsupported,
            "stdin.mode",
        ),
        (
            "termination",
            r#"{"signal":"SIGKILL","grace_ms":0}"#,
            C::TerminationSignalUnsupported,
            "termination.signal",
        ),
        (
            "termination",
            r#"{"signal":"sigterm","grace_ms":0}"#,
            C::TerminationSignalUnsupported,
            "termination.signal",
        ),
        (
            "termination",
            r#"{"signal":"15","grace_ms":0}"#,
            C::TerminationSignalUnsupported,
            "termination.signal",
        ),
    ] {
        only(&Doc::valid().set(key, raw).bytes(), code, locator);
    }
}

#[test]
fn wrong_json_types_are_type_mismatches() {
    for (key, raw, locator) in [
        ("schema", "1", "schema"),
        ("argv", r#""tool --flag""#, "argv"),
        ("argv", "{}", "argv"),
        ("environment", r#""empty""#, "environment"),
        ("environment", r#"{"mode":null}"#, "environment.mode"),
        (
            "working_directory",
            r#"{"capability_id":7}"#,
            "working_directory.capability_id",
        ),
        (
            "stdout",
            r#"{"capture_prefix_bytes":"10"}"#,
            "stdout.capture_prefix_bytes",
        ),
        (
            "stdout",
            r#"{"capture_prefix_bytes":1.5}"#,
            "stdout.capture_prefix_bytes",
        ),
        (
            "stderr",
            r#"{"capture_prefix_bytes":1e3}"#,
            "stderr.capture_prefix_bytes",
        ),
        (
            "stderr",
            r#"{"capture_prefix_bytes":1.0}"#,
            "stderr.capture_prefix_bytes",
        ),
        ("timeout_ms", "true", "timeout_ms"),
        ("timeout_ms", "18446744073709551616", "timeout_ms"),
        ("timeout_ms", "1e30", "timeout_ms"),
        ("termination", "[]", "termination"),
        ("asserted_context", "null", "asserted_context"),
        (
            "asserted_context",
            r#"{"subject_spec_sha256":null}"#,
            "asserted_context.subject_spec_sha256",
        ),
    ] {
        only(
            &Doc::valid().set(key, raw).bytes(),
            C::TypeMismatch,
            locator,
        );
    }
}

// ------------------------------------------------------------------ bounds

#[test]
fn numeric_bounds_are_inclusive_and_exact() {
    let cap = |v: &str| {
        Doc::valid()
            .set("stdout", &format!(r#"{{"capture_prefix_bytes":{v}}}"#))
            .bytes()
    };
    ok(&cap("0"));
    assert_eq!(
        ok(&cap(&MAX_CAPTURE_BYTES.to_string())).capture_prefix_bytes(Stream::Stdout),
        MAX_CAPTURE_BYTES
    );
    for bad in [
        (MAX_CAPTURE_BYTES + 1).to_string(),
        "-1".into(),
        "4294967296".into(),
        "-9223372036854775808".into(),
    ] {
        only(
            &cap(&bad),
            C::CaptureBoundOutOfRange,
            "stdout.capture_prefix_bytes",
        );
    }

    let timeout = |v: &str| Doc::valid().set("timeout_ms", v).bytes();
    assert_eq!(ok(&timeout("1")).timeout_ms(), 1);
    assert_eq!(
        ok(&timeout(&MAX_TIMEOUT_MS.to_string())).timeout_ms(),
        MAX_TIMEOUT_MS
    );
    for bad in [
        "0".to_string(),
        (MAX_TIMEOUT_MS + 1).to_string(),
        "-1".into(),
    ] {
        only(&timeout(&bad), C::TimeoutOutOfRange, "timeout_ms");
    }

    let grace = |v: &str| {
        Doc::valid()
            .set(
                "termination",
                &format!(r#"{{"signal":"SIGTERM","grace_ms":{v}}}"#),
            )
            .bytes()
    };
    assert_eq!(ok(&grace("0")).grace_ms(), 0);
    assert_eq!(
        ok(&grace(&MAX_GRACE_MS.to_string())).grace_ms(),
        MAX_GRACE_MS
    );
    for bad in [(MAX_GRACE_MS + 1).to_string(), "-1".into()] {
        only(&grace(&bad), C::GraceOutOfRange, "termination.grace_ms");
    }
}

#[test]
fn argv_rules_hold_at_their_edges() {
    let argv = |items: &[String]| {
        let json: Vec<String> = items.iter().map(|s| format!("\"{s}\"")).collect();
        Doc::valid()
            .set("argv", &format!("[{}]", json.join(",")))
            .bytes()
    };
    // argv[0] is caller data, never invented or rewritten.
    let plan = ok(&argv(&["not-the-file-name".into()]));
    assert_eq!(plan.argv(), ["not-the-file-name"]);
    only(
        &Doc::valid().set("argv", "[]").bytes(),
        C::ArgvEmpty,
        "argv",
    );

    let many: Vec<String> = (0..MAX_ARGS).map(|i| i.to_string()).collect();
    assert_eq!(ok(&argv(&many)).argv().len(), MAX_ARGS);
    let mut too_many = many.clone();
    too_many.push("x".into());
    only(&argv(&too_many), C::ArgvTooMany, "argv");

    let longest = "a".repeat(MAX_ARG_BYTES);
    assert_eq!(
        ok(&argv(std::slice::from_ref(&longest))).argv()[0].len(),
        MAX_ARG_BYTES
    );
    assert_eq!(
        codes(&argv(&["ok".into(), format!("{longest}b")])),
        vec![(C::ArgTooLong, "argv", Some(1))]
    );
    // The byte bound applies after decoding: 2-byte UTF-8 characters count twice.
    let wide = "\u{e9}".repeat(MAX_ARG_BYTES / 2 + 1);
    assert_eq!(
        codes(&argv(&[wide])),
        vec![(C::ArgTooLong, "argv", Some(0))]
    );

    // NUL, including the JSON escape, is refused per element.
    for nul in [r"\u0000", r"a\u0000", r"\u0000b"] {
        assert_eq!(
            codes(&argv(&["tool".into(), nul.into()])),
            vec![(C::ArgContainsNul, "argv", Some(1))]
        );
    }
    // Literal data survives unchanged: spaces, metacharacters, newline, empty.
    let literal = ok(&Doc::valid()
        .set(
            "argv",
            r#"["","$(id)","a;b|c&&d","line\nbreak","*","\"quoted\"","é"]"#,
        )
        .bytes());
    assert_eq!(
        literal.argv(),
        [
            "",
            "$(id)",
            "a;b|c&&d",
            "line\nbreak",
            "*",
            "\"quoted\"",
            "\u{e9}"
        ]
    );
    // Non-string elements are positional type mismatches.
    assert_eq!(
        codes(&Doc::valid().set("argv", r#"["tool",1,null,["x"]]"#).bytes()),
        vec![
            (C::TypeMismatch, "argv", Some(1)),
            (C::TypeMismatch, "argv", Some(2)),
            (C::TypeMismatch, "argv", Some(3)),
        ]
    );
}

#[test]
fn identifier_and_digest_grammars_are_exact() {
    let id = |v: &str| {
        Doc::valid()
            .set(
                "working_directory",
                &format!(r#"{{"capability_id":"{v}"}}"#),
            )
            .bytes()
    };
    let longest = format!("w{}", "0".repeat(79));
    assert_eq!(ok(&id(&longest)).working_directory_id(), longest);
    for good in ["0", "a.b_c-d", "workdir"] {
        ok(&id(good));
    }
    for bad in [
        "",
        "Workdir",
        ".hidden",
        "-x",
        "a b",
        "../up",
        "/abs",
        r"a\\b",
        "\u{e9}",
        &format!("{longest}0"),
    ] {
        only(&id(bad), C::IdGrammar, "working_directory.capability_id");
    }

    let digest = |v: &str| {
        Doc::valid()
            .set(
                "asserted_context",
                &format!(r#"{{"subject_spec_sha256":"{v}"}}"#),
            )
            .bytes()
    };
    ok(&digest(SUBJECT));
    for bad in [
        SUBJECT.to_uppercase(),
        SUBJECT[..63].to_owned(),
        format!("{SUBJECT}0"),
        format!("sha256:{}", &SUBJECT[7..]),
        String::new(),
    ] {
        only(
            &digest(&bad),
            C::DigestGrammar,
            "asserted_context.subject_spec_sha256",
        );
    }
}

// ------------------------------------------------------------------ determinism

#[test]
fn many_findings_are_all_reported_in_a_stable_order() {
    let raw = Doc::valid()
        .set("schema", r#""other""#)
        .set("argv", r#"["\u0000",2]"#)
        .set("environment", r#"{"mode":"inherit","extra":1}"#)
        .set("timeout_ms", "0")
        .set("termination", r#"{"signal":"SIGINT","grace_ms":999999}"#)
        .set("asserted_context", r#"{"binding_report_sha256":"XYZ"}"#)
        .without("stdin")
        .bytes_with_extra(r#","bogus":true"#);
    let expected = vec![
        (C::UnknownField, "environment", None),
        (C::UnknownField, "plan", None),
        (C::MissingField, "stdin", None),
        (C::TypeMismatch, "argv", Some(1)),
        (C::UnknownSchema, "schema", None),
        (C::ArgContainsNul, "argv", Some(0)),
        (C::EnvironmentModeUnsupported, "environment.mode", None),
        (C::TimeoutOutOfRange, "timeout_ms", None),
        (C::GraceOutOfRange, "termination.grace_ms", None),
        (C::TerminationSignalUnsupported, "termination.signal", None),
        (
            C::DigestGrammar,
            "asserted_context.binding_report_sha256",
            None,
        ),
    ];
    assert_eq!(codes(&raw), expected);
    for _ in 0..3 {
        assert_eq!(err(&raw), err(&raw));
    }
}

#[test]
fn errors_never_echo_document_text() {
    let canary = "CANARY-8d1f";
    let raw = Doc::valid()
        .set("schema", &format!("\"{canary}\""))
        .set("argv", &format!("[\"{canary}\\u0000\"]"))
        .set(
            "working_directory",
            &format!(r#"{{"capability_id":"{canary}"}}"#),
        )
        .bytes_with_extra(&format!(",\"{canary}\":\"{canary}\""));
    let e = err(&raw);
    let shown = format!("{e} {e:?}");
    assert!(!shown.contains(canary), "{shown}");
    assert!(shown.len() < 2_000);
}

// --------------------------------------------------------- arbitrary input

struct SplitMix(u64);
impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % (n as u64)).unwrap()
    }
}

/// Bounded adversarial property test: arbitrary bytes and mutations of a valid
/// plan never panic, always yield at least one finding or a plan whose identity
/// is SHA-256 of exactly the input, and give the same answer twice.
#[test]
fn arbitrary_and_mutated_inputs_never_panic_and_stay_consistent() {
    let mut rng = SplitMix(0x9a25_e1a7_0000_0004);
    let seed = Doc::valid()
        .set(
            "asserted_context",
            &format!(r#"{{"subject_spec_sha256":"{SUBJECT}"}}"#),
        )
        .bytes();
    const ALPHABET: &[u8] = b"{}[]\":,\\u0123456789abcdefnrtl -.eE\x00\xff\xc3\xa9";
    let mut accepted = 0usize;
    for round in 0..40_000 {
        let input: Vec<u8> = if round % 4 == 0 {
            let len = rng.below(256);
            (0..len)
                .map(|_| ALPHABET[rng.below(ALPHABET.len())])
                .collect()
        } else {
            let mut m = seed.clone();
            for _ in 0..=rng.below(4) {
                match rng.below(4) {
                    0 if !m.is_empty() => {
                        let i = rng.below(m.len());
                        m[i] = ALPHABET[rng.below(ALPHABET.len())];
                    }
                    1 => {
                        let i = rng.below(m.len() + 1);
                        m.insert(i, ALPHABET[rng.below(ALPHABET.len())]);
                    }
                    2 if !m.is_empty() => {
                        let i = rng.below(m.len());
                        m.remove(i);
                    }
                    _ => {
                        let i = rng.below(m.len() + 1);
                        m.truncate(i);
                    }
                }
            }
            m
        };
        let first = parse_launch_plan(&input);
        match &first {
            Ok(plan) => {
                accepted += 1;
                assert_eq!(plan.exact_bytes(), input.as_slice());
                let independent: [u8; 32] = Sha256::digest(&input).into();
                assert_eq!(plan.sha256().as_bytes(), &independent);
                assert!(!plan.argv().is_empty());
                assert!(plan.argv().iter().all(|a| !a.contains('\0')));
            }
            Err(e) => {
                assert!(!e.as_slice().is_empty());
                assert!(e.as_slice().len() <= helm_launch::MAX_PLAN_ERRORS);
            }
        }
        assert_eq!(first, parse_launch_plan(&input));
    }
    assert!(
        accepted > 0,
        "mutations never produced a valid plan; the test is too weak"
    );
}
