//! Pure plan-contract tests. No filesystem, process or network access.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use helm_observe::{
    MAX_PATH_BYTES, MAX_PLAN_BYTES, MAX_ROOTS, MAX_TARGETS, Observable, PlanErrorCode as C,
    parse_plan,
};
use serde_json::{Value, json};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";

/// Neutral synthetic fixture: no 7-Zip or A0-specific naming anywhere.
fn plan_value() -> Value {
    json!({
        "schema": "helm-observation-plan",
        "version": "0.1",
        "subject_spec_sha256": SUBJECT,
        "roots": [{"id": "notes-prefix"}, {"id": "notes-runtime"}],
        "targets": [
            {"id": "prefix-dir", "root": "notes-prefix", "observable": "directory_metadata"},
            {"id": "entry", "root": "notes-prefix", "path": "drive_c/Program Files/Notes/notes.exe",
             "observable": "regular_file_sha256"},
            {"id": "loader", "root": "notes-runtime", "path": "bin/loader",
             "observable": "regular_file_sha256"}
        ]
    })
}

fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}

fn codes(v: &Value) -> Vec<C> {
    parse_plan(&bytes(v)).unwrap_err().codes()
}

#[test]
fn valid_plan_parses_and_preserves_declaration_order() {
    let raw = bytes(&plan_value());
    let plan = parse_plan(&raw).unwrap();
    assert_eq!(plan.exact_bytes(), raw.as_slice());
    assert_eq!(plan.root_ids(), ["notes-prefix", "notes-runtime"]);
    let ids: Vec<&str> = plan
        .targets()
        .iter()
        .map(helm_observe::Target::id)
        .collect();
    assert_eq!(ids, ["prefix-dir", "entry", "loader"]);
    assert_eq!(
        plan.targets()[0].observable(),
        Observable::DirectoryMetadata
    );
    assert_eq!(
        plan.targets()[0].path(),
        None,
        "omitted path denotes the root directory"
    );
    assert_eq!(
        plan.targets()[1].observable(),
        Observable::RegularFileSha256
    );
    assert_eq!(plan.subject_spec_sha256().to_hex(), SUBJECT);
}

#[test]
fn plan_identity_is_exact_bytes_not_semantics() {
    let compact = bytes(&plan_value());
    let pretty = serde_json::to_vec_pretty(&plan_value()).unwrap();
    let a = parse_plan(&compact).unwrap();
    let b = parse_plan(&pretty).unwrap();
    assert_ne!(
        a.sha256(),
        b.sha256(),
        "whitespace must change exact-byte identity"
    );
    assert_eq!(
        a.root_ids(),
        b.root_ids(),
        "but the inert model is the same"
    );

    // Field reordering is a different document too.
    let reordered = br#"{"version":"0.1","schema":"helm-observation-plan","subject_spec_sha256":"b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e","roots":[{"id":"r"}],"targets":[{"id":"t","root":"r","observable":"directory_metadata"}]}"#;
    let ordered = br#"{"schema":"helm-observation-plan","version":"0.1","subject_spec_sha256":"b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e","roots":[{"id":"r"}],"targets":[{"id":"t","root":"r","observable":"directory_metadata"}]}"#;
    let x = parse_plan(reordered).unwrap();
    let y = parse_plan(ordered).unwrap();
    assert_ne!(x.sha256(), y.sha256(), "field order changes exact identity");
}

#[test]
fn every_digest_byte_participates_in_identity() {
    let plan = parse_plan(&bytes(&plan_value())).unwrap();
    let d = plan.sha256();
    let raw = *d.as_bytes();
    assert_eq!(raw.len(), 32);
    for i in 0..32 {
        let mut flipped = raw;
        flipped[i] ^= 0x01;
        assert_ne!(
            &flipped,
            d.as_bytes(),
            "byte {i} must participate in equality"
        );
    }
    assert_eq!(d.to_hex().len(), 64);
    assert!(
        d.to_hex()
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
}

#[test]
fn duplicate_decoded_keys_are_rejected_before_a_map_hides_one() {
    let raw = br#"{"schema":"helm-observation-plan","schema":"helm-observation-plan","version":"0.1","subject_spec_sha256":"b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e","roots":[{"id":"r"}],"targets":[{"id":"t","root":"r","observable":"directory_metadata"}]}"#;
    assert!(parse_plan(raw).unwrap_err().contains(C::DuplicateKey));
}

#[test]
fn unknown_schema_version_and_fields_are_rejected() {
    let mut v = plan_value();
    v["schema"] = json!("other-schema");
    assert!(codes(&v).contains(&C::UnknownSchema));

    let mut v = plan_value();
    v["version"] = json!("0.2");
    assert!(codes(&v).contains(&C::UnknownVersion));

    let mut v = plan_value();
    v["extra"] = json!(1);
    assert!(codes(&v).contains(&C::UnknownField));

    // A descriptor-shaped field is simply not in the closed schema.
    let mut v = plan_value();
    v["root_fd"] = json!(7);
    assert!(codes(&v).contains(&C::UnknownField));

    let mut v = plan_value();
    v["targets"][0]["fd"] = json!(3);
    assert!(codes(&v).contains(&C::UnknownField));
}

#[test]
fn identifier_and_digest_grammar_is_enforced() {
    for bad in ["Upper", "-leading", "sp ace", "", "tab\t"] {
        let mut v = plan_value();
        v["roots"][0]["id"] = json!(bad);
        assert!(
            codes(&v).contains(&C::IdGrammar),
            "root id {bad:?} must be rejected"
        );
    }
    for bad in ["", "abc", &"a".repeat(63), &"A".repeat(64), &"g".repeat(64)] {
        let mut v = plan_value();
        v["subject_spec_sha256"] = json!(bad);
        assert!(
            codes(&v).contains(&C::DigestGrammar),
            "digest {bad:?} must be rejected"
        );
    }
}

#[test]
fn unsafe_path_grammar_is_rejected() {
    let cases: [(&str, C); 9] = [
        ("/etc/passwd", C::PathAbsolute),
        ("a//b", C::PathEmptyComponent),
        ("./a", C::PathDotComponent),
        ("../escape", C::PathParentComponent),
        ("a\\b", C::PathBackslash),
        (" leading/x", C::PathLeadingSpace),
        ("trailing /x", C::PathTrailingSpaceOrDot),
        ("dot./x", C::PathTrailingSpaceOrDot),
        ("caf\u{e9}/x", C::PathNonPortableByte),
    ];
    for (path, want) in cases {
        let mut v = plan_value();
        v["targets"][1]["path"] = json!(path);
        assert!(
            codes(&v).contains(&want),
            "path {path:?} should raise {want:?}"
        );
    }
    // A NUL byte cannot survive JSON string decoding into a C path.
    let mut v = plan_value();
    v["targets"][1]["path"] = json!("a\u{0}b");
    assert!(codes(&v).contains(&C::PathNulByte));
}

#[test]
fn resource_limits_are_enforced() {
    let mut v = plan_value();
    v["targets"][1]["path"] = json!("a".repeat(MAX_PATH_BYTES + 1));
    assert!(codes(&v).contains(&C::PathTooLong));

    let mut v = plan_value();
    v["targets"][1]["path"] = json!("a".repeat(256));
    assert!(codes(&v).contains(&C::PathComponentTooLong));

    let mut v = plan_value();
    v["targets"][1]["path"] = json!(vec!["a"; 40].join("/"));
    assert!(codes(&v).contains(&C::PathComponentCount));

    let mut v = plan_value();
    v["roots"] = json!(
        (0..=MAX_ROOTS)
            .map(|i| json!({"id": format!("r{i}")}))
            .collect::<Vec<_>>()
    );
    assert!(codes(&v).contains(&C::RootCountLimit));

    let mut v = plan_value();
    v["targets"] = json!(
        (0..=MAX_TARGETS)
            .map(|i| json!({"id": format!("t{i}"), "root": "notes-prefix",
                            "observable": "directory_metadata"}))
            .collect::<Vec<_>>()
    );
    assert!(codes(&v).contains(&C::TargetCountLimit));

    let oversize = vec![b' '; MAX_PLAN_BYTES + 1];
    assert!(
        parse_plan(&oversize)
            .unwrap_err()
            .contains(C::InputTooLarge)
    );
}

#[test]
fn structural_reference_and_duplicate_errors_are_reported() {
    let mut v = plan_value();
    v["targets"][1]["root"] = json!("no-such-root");
    assert!(codes(&v).contains(&C::UnknownRootReference));

    let mut v = plan_value();
    v["roots"] = json!([{"id": "dup"}, {"id": "dup"}]);
    assert!(codes(&v).contains(&C::DuplicateId));

    let mut v = plan_value();
    v["targets"][2]["id"] = json!("entry");
    assert!(codes(&v).contains(&C::DuplicateId));

    let mut v = plan_value();
    v["targets"] = json!([]);
    assert!(codes(&v).contains(&C::EmptyPlan));

    let mut v = plan_value();
    v["targets"][1]["observable"] = json!("read_whole_tree");
    assert!(codes(&v).contains(&C::UnknownObservable));

    let mut v = plan_value();
    v.as_object_mut().unwrap().remove("roots");
    assert!(codes(&v).contains(&C::MissingField));
}

#[test]
fn errors_are_deterministic_bounded_and_private() {
    let mut v = plan_value();
    v["targets"][1]["path"] = json!("/../ bad\\x");
    v["roots"][0]["id"] = json!("BAD");
    let first = parse_plan(&bytes(&v)).unwrap_err();
    let second = parse_plan(&bytes(&v)).unwrap_err();
    assert_eq!(
        first, second,
        "identical input yields identical ordered findings"
    );
    assert!(first.as_slice().len() <= helm_observe::error::MAX_PLAN_ERRORS);
    let rendered = first.to_string();
    assert!(
        !rendered.contains('/'),
        "diagnostics must not echo path text: {rendered}"
    );
    for e in first.as_slice() {
        assert!(
            [
                "plan",
                "schema",
                "version",
                "subject_spec_sha256",
                "roots",
                "targets",
                "targets.path"
            ]
            .contains(&e.locator()),
            "locator {} must come from the closed schema",
            e.locator()
        );
    }
}

#[test]
fn malformed_and_deep_json_never_panics() {
    let corpus: [&[u8]; 10] = [
        b"",
        b"{",
        b"[]",
        b"null",
        b"\xff\xfe\x00",
        b"{\"schema\":}",
        br#"{"schema":"helm-observation-plan","version":"0.1","subject_spec_sha256":1,"roots":1,"targets":1}"#,
        br#"{"roots":[{"id":[]}],"targets":[]}"#,
        b"[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]",
        br#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":{"i":1}}}}}}}}}"#,
    ];
    for raw in corpus {
        let _ = parse_plan(raw);
    }
}

#[test]
fn seeded_malformed_corpus_never_panics_and_is_deterministic() {
    // Deterministic xorshift; seeds are fixed so any failure is reproducible.
    let template = bytes(&plan_value());
    let mut state: u64 = 0x0B5F_5001 ^ 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for case in 0..512u32 {
        let mut mutated = template.clone();
        let mutations = 1 + (next() % 4) as usize;
        for _ in 0..mutations {
            if mutated.is_empty() {
                break;
            }
            let idx = (next() as usize) % mutated.len();
            match next() % 3 {
                0 => mutated[idx] = (next() % 256) as u8,
                1 => {
                    mutated.remove(idx);
                }
                _ => mutated.insert(idx, (next() % 256) as u8),
            }
        }
        let first = parse_plan(&mutated);
        let second = parse_plan(&mutated);
        assert_eq!(
            first.is_ok(),
            second.is_ok(),
            "case {case} must be deterministic"
        );
        if let (Err(a), Err(b)) = (&first, &second) {
            assert_eq!(a, b, "case {case} error ordering must be deterministic");
        }
    }
}
