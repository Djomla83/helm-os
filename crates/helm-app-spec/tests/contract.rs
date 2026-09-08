//! Expectations come from the owner's negative matrix, ADR-0021 and frozen A0 pins.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use helm_app_spec::{
    ErrorCode as Code, MAX_INPUT_BYTES, PrefixRole, RuntimeFamily, SourceArchitecture, SpecErrors,
    ValidatedAppSpec, WindowsArchitecture, parse_spec,
};
use serde_json::{Value, json};

const A0: &[u8] = include_bytes!("fixtures/a0-7zip.json");
const SYNTHETIC: &[u8] = include_bytes!("fixtures/synthetic-notes.json");

fn fixture() -> Value {
    serde_json::from_slice(A0).unwrap()
}
fn encode(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn edit(pointer: &str, replacement: Value) -> Vec<u8> {
    let mut value = fixture();
    *value.pointer_mut(pointer).unwrap() = replacement;
    encode(&value)
}
fn error(bytes: &[u8], code: Code, location: &str) -> SpecErrors {
    let errors = parse_spec(bytes).unwrap_err();
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|e| e.code() == code && e.location() == location),
        "{errors:?}"
    );
    errors
}
fn same_desired(a: &ValidatedAppSpec, b: &ValidatedAppSpec) {
    assert_eq!(a.application(), b.application());
    assert_eq!(a.runtime(), b.runtime());
    assert_eq!(a.environment(), b.environment());
    assert_eq!(a.entry_point(), b.entry_point());
    assert_eq!(a.verification(), b.verification());
}

#[test]
fn a0_declares_only_frozen_desired_inputs() {
    let spec = parse_spec(A0).unwrap();
    assert_eq!(spec.application().id().as_str(), "7zip-x64");
    assert_eq!(spec.application().version(), "26.03");
    assert_eq!(
        spec.application().source().architecture(),
        SourceArchitecture::X86_64
    );
    assert_eq!(spec.application().source().size(), 1_661_239);
    assert_eq!(
        spec.application().source().sha256().as_str(),
        "0859c524b8a63551848f0c246abddcb1d0b7b656b0fbfe879f8d85e61a9e6edd"
    );
    assert_eq!(spec.runtime().family(), RuntimeFamily::Wine);
    assert_eq!(
        spec.runtime()
            .artifacts()
            .iter()
            .map(|a| a.size())
            .collect::<Vec<_>>(),
        [11586578, 150150970, 1320, 142140490]
    );
    assert_eq!(
        spec.environment().windows_architecture(),
        WindowsArchitecture::Win64
    );
    assert_eq!(spec.environment().prefix_role(), PrefixRole::Dedicated);
    assert_eq!(spec.environment().disabled_dlls(), ["mscoree", "mshtml"]);
    assert_eq!(
        spec.entry_point().path().as_str(),
        "drive_c/Program Files/7-Zip/7zFM.exe"
    );
    assert_eq!(spec.entry_point().sha256(), None);
    assert_eq!(
        spec.verification()
            .iter()
            .map(|d| d.size())
            .collect::<Vec<_>>(),
        [34585, 9093, 1290]
    );
    assert_eq!(spec.verification()[0].role().as_str(), "protocol");
    assert_eq!(
        spec.verification()[0].sha256().as_str(),
        "bd55677cc1724d078ee25dfbcbbc966350b20c129475bdcad28d2603785a2240"
    );
}

#[test]
fn second_application_and_arbitrary_valid_ids_use_same_contract() {
    let spec = parse_spec(SYNTHETIC).unwrap();
    assert_eq!(spec.application().id().as_str(), "synthetic-notes");
    assert_eq!(spec.runtime().artifacts().len(), 1);
    assert_eq!(spec.runtime().artifacts()[0].label(), None);
    assert!(spec.environment().disabled_dlls().is_empty());
    assert!(spec.entry_point().sha256().is_some());
    for id in ["a", "0", "not-7zip", "third.app_42", &"a".repeat(80)] {
        let bytes = edit("/application/id", json!(id));
        assert_eq!(parse_spec(&bytes).unwrap().application().id().as_str(), id);
    }
}

#[test]
fn byte_different_whitespace_has_distinct_exact_document_identity() {
    let mut spaced = A0.to_vec();
    spaced.extend_from_slice(b" \n\t\r\n");
    let a = parse_spec(A0).unwrap();
    let b = parse_spec(&spaced).unwrap();
    same_desired(&a, &b);
    // Independently computed with Python hashlib over the frozen fixture bytes.
    assert_eq!(
        a.spec_sha256().as_str(),
        "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e"
    );
    assert_eq!(
        b.spec_sha256().as_str(),
        "55326743fb72e8037c52ab59e1e1be078fad9079b661de04b37173121fa0a5a2"
    );
    assert_ne!(a.spec_sha256(), b.spec_sha256());
    assert_ne!(a, b);
}

#[test]
fn field_order_changes_identity_but_preserves_desired_values() {
    let original = std::str::from_utf8(A0).unwrap();
    let reordered = original.replacen(
        "\"schema\": \"helm-app-spec\",\n  \"version\": \"0.1\"",
        "\"version\": \"0.1\",\n  \"schema\": \"helm-app-spec\"",
        1,
    );
    assert_ne!(original, reordered);
    let a = parse_spec(A0).unwrap();
    let b = parse_spec(reordered.as_bytes()).unwrap();
    same_desired(&a, &b);
    assert_ne!(a.spec_sha256(), b.spec_sha256());
}

#[test]
fn same_labels_with_changed_artifact_identity_remain_distinct_requirements() {
    for pointer in ["/runtime/artifacts/0/sha256", "/application/source/sha256"] {
        let other = parse_spec(&edit(pointer, json!("a".repeat(64)))).unwrap();
        let original = parse_spec(A0).unwrap();
        assert_eq!(
            other.application().version(),
            original.application().version()
        );
        assert_eq!(
            other.runtime().artifacts()[0].label(),
            original.runtime().artifacts()[0].label()
        );
        assert_ne!(original, other);
        if pointer.starts_with("/runtime") {
            assert_ne!(original.runtime(), other.runtime());
        } else {
            assert_ne!(
                original.application().source(),
                other.application().source()
            );
        }
    }
}

#[test]
fn empty_malformed_utf8_surrogates_bom_and_trailing_input_reject() {
    for bytes in [
        b"".as_slice(),
        b" ",
        b"{",
        b"null null",
        b"{\"a\":}",
        b"{\"a\":1,}",
        b"[1,]",
        b"\xff",
        br#""\ud800""#,
        b"\xef\xbb\xbf{}",
        b"{\"x\":1e9999}",
    ] {
        error(bytes, Code::JsonInvalid, "$");
    }
    let mut extra = A0.to_vec();
    extra.extend_from_slice(b"{}");
    error(&extra, Code::JsonInvalid, "$");
}

#[test]
fn byte_limit_is_inclusive_and_checked_before_content() {
    let mut bytes = A0.to_vec();
    bytes.resize(MAX_INPUT_BYTES, b' ');
    assert!(parse_spec(&bytes).is_ok());
    bytes.push(0xff);
    let errors = error(&bytes, Code::InputTooLarge, "$");
    assert_eq!(errors.as_slice().len(), 1);
    error(&vec![0xff; MAX_INPUT_BYTES + 1], Code::InputTooLarge, "$");
}

#[test]
fn unknown_schema_version_and_typed_vocabulary_reject() {
    for (pointer, value, code) in [
        ("/schema", "other", Code::SchemaUnknown),
        ("/version", "0.2", Code::SchemaVersion),
        ("/runtime/family", "proton", Code::ValueUnsupported),
        ("/runtime/family", "Wine", Code::ValueUnsupported),
        (
            "/application/source/architecture",
            "arm64",
            Code::ValueUnsupported,
        ),
        (
            "/application/source/architecture",
            "x86",
            Code::ValueUnsupported,
        ),
        (
            "/environment/windows_architecture",
            "wow64",
            Code::ValueUnsupported,
        ),
        (
            "/environment/windows_architecture",
            "win32",
            Code::ValueUnsupported,
        ),
        ("/environment/prefix/role", "shared", Code::ValueUnsupported),
    ] {
        error(
            &edit(pointer, json!(value)),
            code,
            &format!("${}", pointer.replace('/', ".")),
        );
    }
}

const OBJECTS: &[&str] = &[
    "",
    "/application",
    "/application/source",
    "/runtime",
    "/runtime/artifacts/0",
    "/environment",
    "/environment/prefix",
    "/entry_point",
    "/verification",
    "/verification/definitions/0",
];
fn location(pointer: &str) -> String {
    format!("${}", pointer.replace("/0", "[0]").replace('/', "."))
}

#[test]
fn unknown_fields_and_wrong_object_types_reject_at_every_schema_level() {
    for pointer in OBJECTS {
        let mut value = fixture();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("private-unknown-field".into(), json!("ignored?"));
        error(&encode(&value), Code::FieldUnknown, &location(pointer));
        for wrong in [
            Value::Null,
            json!([]),
            json!("object"),
            json!(42),
            json!(false),
        ] {
            error(&edit(pointer, wrong), Code::FieldType, &location(pointer));
        }
    }
}

#[test]
fn every_required_field_is_really_required() {
    for pointer in OBJECTS {
        let object = fixture()
            .pointer(pointer)
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        for name in object.keys() {
            if *pointer == "/runtime/artifacts/0" && name == "label" {
                continue;
            }
            let mut value = fixture();
            value
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(name);
            error(
                &encode(&value),
                Code::FieldMissing,
                &format!("{}.{}", location(pointer), name),
            );
        }
    }
}

#[test]
fn digests_require_exact_lowercase_hex_in_all_identity_locations() {
    for pointer in [
        "/application/source/sha256",
        "/runtime/artifacts/0/sha256",
        "/verification/definitions/0/sha256",
    ] {
        for bad in [
            "".to_owned(),
            "a".repeat(63),
            "a".repeat(65),
            "A".repeat(64),
            "g".repeat(64),
            format!("{} ", "a".repeat(63)),
            "é".repeat(32),
        ] {
            error(
                &edit(pointer, json!(bad)),
                Code::DigestInvalid,
                &location(pointer),
            );
        }
        error(
            &edit(pointer, Value::Null),
            Code::FieldType,
            &location(pointer),
        );
    }
    let mut value = fixture();
    value["entry_point"]["sha256"] = json!("bad");
    error(&encode(&value), Code::DigestInvalid, "$.entry_point.sha256");
    value["entry_point"]["sha256"] = json!("a".repeat(64));
    assert!(parse_spec(&encode(&value)).is_ok());
}

#[test]
fn invalid_ids_and_long_identifiers_reject() {
    for pointer in [
        "/application/id",
        "/runtime/artifacts/0/role",
        "/verification/definitions/0/role",
    ] {
        for bad in [
            "",
            "UPPER",
            ".",
            "-leading",
            "a b",
            "a/b",
            "a\\b",
            "a;exit",
            "a\n",
            "é",
            &"a".repeat(81),
        ] {
            error(
                &edit(pointer, json!(bad)),
                Code::IdentifierInvalid,
                &location(pointer),
            );
        }
        error(
            &edit(pointer, json!("a".repeat(20000))),
            Code::JsonLimit,
            "$",
        );
    }
}

#[test]
fn duplicate_roles_and_dlls_reject_in_declaration_order() {
    for (pointer, code, field) in [
        (
            "/runtime/artifacts",
            Code::DuplicateRuntimeArtifact,
            ".role",
        ),
        (
            "/verification/definitions",
            Code::DuplicateVerificationDefinition,
            ".role",
        ),
        ("/environment/disabled_dlls", Code::DuplicateDll, ""),
    ] {
        let mut value = fixture();
        let list = value.pointer_mut(pointer).unwrap().as_array_mut().unwrap();
        list[1] = list[0].clone();
        error(
            &encode(&value),
            code,
            &format!("{}[1]{field}", location(pointer)),
        );
    }
}

#[test]
fn conflicting_runtime_roles_still_reject() {
    let mut value = fixture();
    value["runtime"]["artifacts"][1]["role"] = value["runtime"]["artifacts"][0]["role"].clone();
    assert_ne!(
        value["runtime"]["artifacts"][0]["sha256"],
        value["runtime"]["artifacts"][1]["sha256"]
    );
    error(
        &encode(&value),
        Code::DuplicateRuntimeArtifact,
        "$.runtime.artifacts[1].role",
    );
}

#[test]
fn artifact_source_and_definition_sizes_are_positive_bounded_integers() {
    for pointer in [
        "/application/source/size",
        "/runtime/artifacts/0/size",
        "/verification/definitions/0/size",
    ] {
        for bad in [
            json!(0),
            json!(-1),
            json!(1.0),
            json!(1.5),
            json!(1_099_511_627_777_u64),
            json!(u64::MAX),
            json!("12"),
            json!(true),
            Value::Null,
        ] {
            error(&edit(pointer, bad), Code::SizeInvalid, &location(pointer));
        }
        for good in [1, 1_099_511_627_776_u64] {
            assert!(parse_spec(&edit(pointer, json!(good))).is_ok());
        }
    }
    let text = String::from_utf8(edit("/application/source/size", json!(42))).unwrap();
    for bad in ["18446744073709551616", "1e10", "-0"] {
        error(
            text.replace("\"size\":42", &format!("\"size\":{bad}"))
                .as_bytes(),
            Code::SizeInvalid,
            "$.application.source.size",
        );
    }
}

#[test]
fn collection_minima_maxima_and_excess_reject() {
    error(
        &edit("/runtime/artifacts", json!([])),
        Code::CollectionLimit,
        "$.runtime.artifacts",
    );
    error(
        &edit("/verification/definitions", json!([])),
        Code::CollectionLimit,
        "$.verification.definitions",
    );
    assert!(parse_spec(&edit("/environment/disabled_dlls", json!([]))).is_ok());
    for (pointer, max) in [
        ("/runtime/artifacts", 16),
        ("/verification/definitions", 8),
        ("/environment/disabled_dlls", 8),
    ] {
        let mut value = fixture();
        let seed = value.pointer(pointer).unwrap()[0].clone();
        let list = value.pointer_mut(pointer).unwrap().as_array_mut().unwrap();
        *list = (0..=max)
            .map(|index| {
                if pointer.ends_with("dlls") {
                    json!(format!("dll{index}"))
                } else {
                    let mut item = seed.clone();
                    item["role"] = json!(format!("role-{index}"));
                    item
                }
            })
            .collect();
        let field = location(pointer);
        error(
            &encode(&value),
            if max == 16 {
                Code::JsonLimit
            } else {
                Code::CollectionLimit
            },
            if max == 16 { "$" } else { &field },
        );
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(parse_spec(&encode(&value)).is_ok());
    }
    let malicious = format!("[{}]", "0,".repeat(20000) + "0");
    error(malicious.as_bytes(), Code::JsonLimit, "$");
}

#[test]
fn disabled_dll_grammar_has_no_override_language_or_case_aliases() {
    for bad in [
        "",
        "MSHTML",
        "mshtml.dll",
        "mshtml=n,b",
        "a,b",
        "../x",
        "a-b",
        "1name",
        &"a".repeat(65),
    ] {
        error(
            &edit("/environment/disabled_dlls", json!([bad])),
            Code::DllInvalid,
            "$.environment.disabled_dlls[0]",
        );
    }
    error(
        &edit("/environment/disabled_dlls", json!(["mshtml", "MSHTML"])),
        Code::DllInvalid,
        "$.environment.disabled_dlls[1]",
    );
    assert!(
        parse_spec(&edit(
            "/environment/disabled_dlls",
            json!(["a", "name_1", "a".repeat(64)])
        ))
        .is_ok()
    );
}

#[test]
fn entry_point_rejects_absolute_traversal_ambiguous_and_shell_spellings() {
    for path in [
        "",
        "/tmp/app.exe",
        "C:/app.exe",
        "C:\\app.exe",
        "C:app.exe",
        "//server/share/app.exe",
        "\\\\?\\C:\\app.exe",
        "../app.exe",
        "drive_c/../app.exe",
        "drive_c/a/../../app.exe",
        "drive_c/./a.exe",
        "drive_c//a.exe",
        "drive_c/a.exe/",
        "drive_c\\a.exe",
        "drive_c/a\\b.exe",
        "drive_c/a:stream",
        "drive_c/a.exe.",
        "drive_c/a.exe ",
        "drive_c/ a.exe",
        "drive_c/con.exe",
        "drive_c/COM1.log",
        "drive_c/lpt9",
        "drive_c/CON .exe",
        "drive_c/NUL/file.exe",
        "drive_c/é.exe",
        "drive_c/hello\n.exe",
        "drive_c/a\0.exe",
        "drive_c/%TEMP%/a.exe",
        "drive_c/$HOME/a.exe",
        "drive_c/$(echo x).exe",
        "drive_c/a;exit.exe",
        "drive_c/a`echo`.exe",
        "drive_c/http://x",
        "drive_c/a~1.exe",
        "drive_c/",
        "drive_d/app.exe",
    ] {
        error(
            &edit("/entry_point/path", json!(path)),
            Code::PathUnsafe,
            "$.entry_point.path",
        );
    }
}

#[test]
fn path_depth_component_and_total_length_boundaries() {
    let depth32 = format!("drive_c/{}a.exe", "a/".repeat(30));
    assert!(parse_spec(&edit("/entry_point/path", json!(depth32))).is_ok());
    error(
        &edit(
            "/entry_point/path",
            json!(format!("drive_c/{}a.exe", "a/".repeat(31))),
        ),
        Code::PathUnsafe,
        "$.entry_point.path",
    );
    assert!(
        parse_spec(&edit(
            "/entry_point/path",
            json!(format!("drive_c/{}", "a".repeat(255)))
        ))
        .is_ok()
    );
    error(
        &edit(
            "/entry_point/path",
            json!(format!("drive_c/{}", "a".repeat(256))),
        ),
        Code::PathUnsafe,
        "$.entry_point.path",
    );
    let path1024 = format!(
        "drive_c/{}/{}/{}/{}",
        "a".repeat(255),
        "b".repeat(255),
        "c".repeat(255),
        "d".repeat(248)
    );
    assert_eq!(path1024.len(), 1024);
    assert!(parse_spec(&edit("/entry_point/path", json!(path1024))).is_ok());
    error(
        &edit("/entry_point/path", json!(path1024 + "e")),
        Code::PathUnsafe,
        "$.entry_point.path",
    );
}

#[test]
fn path_spelling_is_preserved_without_case_normalization_or_resolution() {
    for path in [
        "drive_c/No Such App/missing.EXE",
        "drive_c/Program Files/Other_App/app-1.exe",
        "drive_c/com10/app.exe",
    ] {
        let spec = parse_spec(&edit("/entry_point/path", json!(path))).unwrap();
        assert_eq!(spec.entry_point().path().as_str(), path);
    }
}

#[test]
fn optional_fields_must_be_omitted_instead_of_null() {
    let mut value = fixture();
    value["entry_point"]["sha256"] = Value::Null;
    error(&encode(&value), Code::FieldType, "$.entry_point.sha256");
    error(
        &edit("/runtime/artifacts/0/label", Value::Null),
        Code::FieldType,
        "$.runtime.artifacts[0].label",
    );
}

#[test]
fn metadata_is_bounded_but_shell_like_text_stays_inert() {
    let shell = "$(touch SHOULD_NEVER_EXIST); echo %USERPROFILE% | ignored `text`";
    let spec = parse_spec(&edit("/runtime/artifacts/0/label", json!(shell))).unwrap();
    assert_eq!(spec.runtime().artifacts()[0].label(), Some(shell));
    for pointer in ["/application/version", "/runtime/artifacts/0/label"] {
        for bad in ["", "a\n", "a\0", "a\u{7f}", "é", &"a".repeat(257)] {
            error(
                &edit(pointer, json!(bad)),
                Code::MetadataInvalid,
                &location(pointer),
            );
        }
        assert!(parse_spec(&edit(pointer, json!("a".repeat(256)))).is_ok());
    }
}

#[test]
fn observed_state_execution_and_evidence_result_fields_are_unknown() {
    for (pointer, name) in [
        ("/runtime", "installed_runtime"),
        ("/runtime", "actual_loader"),
        ("/application", "detected_version"),
        ("/environment", "existing_prefix"),
        ("/application", "installation_result"),
        ("/entry_point", "verified_entrypoint"),
        ("/verification", "evidence_bundle"),
        ("/verification", "result"),
        ("/environment", "variables"),
        ("/environment", "registry"),
        ("/environment", "winetricks"),
        ("", "install"),
        ("", "launch"),
        ("", "recovery"),
    ] {
        let mut value = fixture();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(name.to_owned(), json!({"command": "echo inert"}));
        error(&encode(&value), Code::FieldUnknown, &location(pointer));
    }
}

#[test]
fn duplicate_decoded_keys_are_rejected_before_map_insertion_everywhere() {
    // Demonstrate the upstream behavior we must NOT inherit.
    let ordinary: Value = serde_json::from_str(r#"{"x":1,"\u0078":2}"#).unwrap();
    assert_eq!(ordinary["x"], 2);
    for pointer in OBJECTS {
        let value = fixture();
        let map = value.pointer(pointer).unwrap().as_object().unwrap();
        let (key, val) = map.iter().next().unwrap();
        let fragment = serde_json::to_string(value.pointer(pointer).unwrap()).unwrap();
        for duplicate in [
            key.clone(),
            format!("\\u{:04x}{}", key.as_bytes()[0], &key[1..]),
        ] {
            let repeated = format!(
                "{{\"{duplicate}\":{},{}",
                serde_json::to_string(val).unwrap(),
                &fragment[1..]
            );
            let bytes = String::from_utf8(encode(&value))
                .unwrap()
                .replacen(&fragment, &repeated, 1);
            error(bytes.as_bytes(), Code::FieldDuplicate, "$");
        }
    }
    for bytes in [
        br#"{"unknown":{"x":1,"x":2}}"#.as_slice(),
        br#"{"optional":null,"optional":null}"#,
        br#"{"a":[{"x":1,"\u0078":1}]}"#,
    ] {
        error(bytes, Code::FieldDuplicate, "$");
    }
}

#[test]
fn structural_budgets_bound_unknown_subtrees() {
    let deep = format!("{}0{}", "[".repeat(1000), "]".repeat(1000));
    error(deep.as_bytes(), Code::JsonLimit, "$");
    error(
        format!("{{\"{}\":0}}", "a".repeat(81)).as_bytes(),
        Code::JsonLimit,
        "$",
    );
    let wide = (0..17)
        .map(|i| format!("\"k{i}\":0"))
        .collect::<Vec<_>>()
        .join(",");
    error(format!("{{{wide}}}").as_bytes(), Code::JsonLimit, "$");
    let nodes = json!(
        (0..16)
            .map(|_| (0..16).map(|_| 0).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
    error(&encode(&nodes), Code::JsonLimit, "$");
}

#[test]
fn errors_are_fixed_bounded_private_and_deterministic() {
    let mut value = fixture();
    value["schema"] = json!("private-marker");
    value["application"]["id"] = json!("private-marker/invalid");
    value["runtime"]["artifacts"][0]["sha256"] = json!("private-marker");
    value["environment"]["disabled_dlls"] = json!(["mshtml", "mshtml"]);
    value["entry_point"]["path"] = json!("/private-marker");
    let bytes = encode(&value);
    let errors = parse_spec(&bytes).unwrap_err();
    assert_eq!(
        errors
            .as_slice()
            .iter()
            .map(|e| (e.code(), e.location()))
            .collect::<Vec<_>>(),
        [
            (Code::SchemaUnknown, "$.schema"),
            (Code::IdentifierInvalid, "$.application.id"),
            (Code::DigestInvalid, "$.runtime.artifacts[0].sha256"),
            (Code::DuplicateDll, "$.environment.disabled_dlls[1]"),
            (Code::PathUnsafe, "$.entry_point.path"),
        ]
    );
    for _ in 0..100 {
        assert_eq!(parse_spec(&bytes).unwrap_err(), errors);
    }
    assert!(!format!("{errors} {errors:?}").contains("private-marker"));
    for e in errors.as_slice() {
        assert!(e.explanation().len() <= 128);
        assert!(e.location().len() <= 80);
    }
    assert_eq!(Code::DigestInvalid.as_str(), "DIGEST_INVALID");
}

#[test]
fn error_count_limit_is_explicit_and_repeatable() {
    let mut value = fixture();
    value["runtime"]["artifacts"] = json!(
        (0..16)
            .map(|_| json!({"role":"!", "size":0, "sha256":"!", "label":""}))
            .collect::<Vec<_>>()
    );
    let bytes = encode(&value);
    let errors = parse_spec(&bytes).unwrap_err();
    assert_eq!(errors.as_slice().len(), 64);
    assert_eq!(errors.as_slice()[63].code(), Code::ErrorLimit);
    assert_eq!(parse_spec(&bytes).unwrap_err(), errors);
}

#[test]
fn identical_bytes_repeat_equal_model_and_exact_digest() {
    for bytes in [A0, SYNTHETIC] {
        let first = parse_spec(bytes).unwrap();
        for _ in 0..100 {
            assert_eq!(parse_spec(bytes).unwrap(), first);
        }
        assert_eq!(first.clone(), first);
    }
}

#[test]
fn combined_collection_maxima_fit_the_parser_budget() {
    let mut value = fixture();
    let artifact = value["runtime"]["artifacts"][0].clone();
    let definition = value["verification"]["definitions"][0].clone();
    value["runtime"]["artifacts"] = json!(
        (0..16)
            .map(|i| {
                let mut a = artifact.clone();
                a["role"] = json!(format!("artifact-{i}"));
                a["label"] = json!("a".repeat(256));
                a
            })
            .collect::<Vec<_>>()
    );
    value["verification"]["definitions"] = json!(
        (0..8)
            .map(|i| {
                let mut d = definition.clone();
                d["role"] = json!(format!("definition-{i}"));
                d
            })
            .collect::<Vec<_>>()
    );
    value["environment"]["disabled_dlls"] =
        json!((0..8).map(|i| format!("dll{i}")).collect::<Vec<_>>());
    let spec = parse_spec(&encode(&value)).unwrap();
    assert_eq!(spec.runtime().artifacts().len(), 16);
    assert_eq!(spec.verification().len(), 8);
    assert_eq!(spec.environment().disabled_dlls().len(), 8);
}

#[test]
fn malformed_corpus_and_mutations_never_panic_and_repeat_identically() {
    let check = |bytes: &[u8]| {
        let result = std::panic::catch_unwind(|| parse_spec(bytes));
        assert!(result.is_ok(), "panic for {} bytes", bytes.len());
        let result = result.unwrap();
        if let Err(errors) = &result {
            assert!(!errors.as_slice().is_empty());
            assert!(errors.as_slice().len() <= 64);
        }
        assert_eq!(parse_spec(bytes), result);
    };
    for byte in 0..=255 {
        check(&[byte]);
    }
    for end in 0..A0.len() {
        check(&A0[..end]);
    }
    for index in 0..A0.len() {
        for byte in [0, b'"', b'\\', 0xff] {
            let mut bytes = A0.to_vec();
            bytes[index] = byte;
            check(&bytes);
        }
    }
    for length in [16, 64, 1024, MAX_INPUT_BYTES] {
        for byte in [0, b'[', b'{', b'9', b'"', 0xff] {
            check(&vec![byte; length]);
        }
    }
}
