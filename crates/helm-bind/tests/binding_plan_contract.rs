//! Adversarial binding-plan contract tests. Pure: no filesystem, process or
//! network access, so they run identically on Linux, Windows and macOS.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use helm_bind::{
    BindingPlanErrorCode as C, ClaimKind, MAX_CLAIMS, MAX_ID_BYTES, MAX_PLAN_BYTES,
    parse_binding_plan,
};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";

fn doc(extra: &str, claims: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\"{extra},\"claims\":[{claims}]}}"
    )
    .into_bytes()
}

fn simple(claims: &str) -> Vec<u8> {
    doc("", claims)
}

const SOURCE: &str = "{\"claim\":\"source_body\",\"target\":\"src\"}";

fn codes(raw: &[u8]) -> Vec<C> {
    parse_binding_plan(raw).unwrap_err().codes()
}

// ------------------------------------------------------------------ acceptance

#[test]
fn a_minimal_mapping_parses_and_preserves_declaration_order() {
    let raw = simple(
        "{\"claim\":\"source_body\",\"target\":\"src\"},\
         {\"claim\":\"runtime_artifact_body\",\"role\":\"rt\",\"target\":\"a1\"},\
         {\"claim\":\"verification_definition_body\",\"role\":\"proto\",\"target\":\"d1\"}",
    );
    let plan = parse_binding_plan(&raw).unwrap();
    assert_eq!(plan.exact_bytes(), raw.as_slice());
    assert_eq!(plan.subject_spec_sha256().to_hex(), SUBJECT);
    assert_eq!(plan.asserted_prefix_root_id(), None);
    let kinds: Vec<ClaimKind> = plan
        .claims()
        .iter()
        .map(helm_bind::ClaimMapping::kind)
        .collect();
    assert_eq!(
        kinds,
        [
            ClaimKind::SourceBody,
            ClaimKind::RuntimeArtifactBody,
            ClaimKind::VerificationDefinitionBody
        ]
    );
    assert_eq!(plan.claims()[1].role(), Some("rt"));
    assert_eq!(plan.claims()[0].role(), None);
    assert_eq!(plan.claims()[2].target(), "d1");
}

/// Two entry-point claims legitimately share one observation target.
#[test]
fn both_entry_point_claims_may_share_one_target() {
    let raw = doc(
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"entry\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"entry\"}",
    );
    let plan = parse_binding_plan(&raw).unwrap();
    assert_eq!(plan.claims().len(), 2);
    assert_eq!(plan.claims()[0].target(), plan.claims()[1].target());
    assert_eq!(plan.asserted_prefix_root_id(), Some("prefix"));
}

// -------------------------------------------------------------- exact identity

#[test]
fn identity_is_over_exact_bytes_with_no_canonicalisation() {
    let compact = simple(SOURCE);
    let spaced = format!(
        "{{\"schema\":\"helm-binding-plan\", \"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
    )
    .into_bytes();
    let reordered = format!(
        "{{\"version\":\"0.1\",\"schema\":\"helm-binding-plan\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
    )
    .into_bytes();

    let a = parse_binding_plan(&compact).unwrap();
    let b = parse_binding_plan(&spaced).unwrap();
    let c = parse_binding_plan(&reordered).unwrap();
    assert_ne!(a.sha256(), b.sha256(), "whitespace changes identity");
    assert_ne!(a.sha256(), c.sha256(), "field order changes identity");
    assert_eq!(a.claims(), b.claims(), "the inert model is the same");

    // The digest is SHA-256 over exactly the accepted bytes, computed here
    // independently of the crate under test.
    use sha2::Digest as _;
    let expected: [u8; 32] = sha2::Sha256::digest(&compact).into();
    assert_eq!(a.sha256().as_bytes(), &expected);
    assert_eq!(a.sha256().to_hex().len(), 64);
}

/// An escaped spelling of the same key is a different document, and an escaped
/// duplicate of a schema key is a duplicate, not a second field.
#[test]
fn json_escaping_changes_identity_and_escaped_duplicates_are_rejected() {
    let plain = simple(SOURCE);
    // "\u0073chema" decodes to "schema".
    let escaped_dup = format!(
        "{{\"schema\":\"helm-binding-plan\",\"\\u0073chema\":\"helm-binding-plan\",\
         \"version\":\"0.1\",\"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
    );
    assert!(
        parse_binding_plan(escaped_dup.as_bytes())
            .unwrap_err()
            .contains(C::DuplicateKey)
    );

    // An escaped but non-duplicate spelling of a value is simply different bytes.
    let escaped_value = format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\
         \"claims\":[{{\"claim\":\"source_body\",\"target\":\"\\u0073rc\"}}]}}"
    );
    let escaped = parse_binding_plan(escaped_value.as_bytes()).unwrap();
    let direct = parse_binding_plan(&plain).unwrap();
    assert_eq!(escaped.claims()[0].target(), "src", "escapes are decoded");
    assert_ne!(
        escaped.sha256(),
        direct.sha256(),
        "but the exact bytes differ, so the identity differs"
    );
}

#[test]
fn duplicate_decoded_keys_are_rejected_at_every_depth() {
    let root_dup = format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"subject_spec_sha256\":\"{SUBJECT}\",\
         \"claims\":[{SOURCE}]}}"
    );
    assert!(
        parse_binding_plan(root_dup.as_bytes())
            .unwrap_err()
            .contains(C::DuplicateKey)
    );

    // Inside an object nested in the claims array.
    let nested_dup = simple("{\"claim\":\"source_body\",\"target\":\"decoy\",\"target\":\"real\"}");
    assert!(
        parse_binding_plan(&nested_dup)
            .unwrap_err()
            .contains(C::DuplicateKey),
        "a duplicate inside an array-nested object must not resolve to the last value"
    );

    // Escaped spelling inside an array-nested object.
    let nested_escaped =
        simple("{\"claim\":\"source_body\",\"target\":\"decoy\",\"\\u0074arget\":\"real\"}");
    assert!(
        parse_binding_plan(&nested_escaped)
            .unwrap_err()
            .contains(C::DuplicateKey)
    );
}

// -------------------------------------------------------------- closed schema

#[test]
fn unknown_schema_version_and_fields_are_rejected() {
    let bad_schema = format!(
        "{{\"schema\":\"other\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
    );
    assert!(codes(bad_schema.as_bytes()).contains(&C::UnknownSchema));

    let bad_version = format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.2\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
    );
    assert!(codes(bad_version.as_bytes()).contains(&C::UnknownVersion));

    assert!(codes(&doc(",\"extra\":1", SOURCE)).contains(&C::UnknownField));
    assert!(
        codes(&simple(
            "{\"claim\":\"source_body\",\"target\":\"src\",\"expected_sha256\":\"x\"}"
        ))
        .contains(&C::UnknownField),
        "a caller-supplied expected value is simply not in the closed schema"
    );
    assert!(
        codes(&simple(
            "{\"claim\":\"source_body\",\"target\":\"src\",\"path\":\"drive_c/a\"}"
        ))
        .contains(&C::UnknownField),
        "a filesystem path is not in the closed schema"
    );
    assert!(
        codes(&simple("{\"claim\":\"whatever_body\",\"target\":\"src\"}"))
            .contains(&C::UnknownClaimKind)
    );
}

#[test]
fn role_presence_is_fixed_by_the_claim_kind() {
    assert!(
        codes(&simple(
            "{\"claim\":\"runtime_artifact_body\",\"target\":\"a\"}"
        ))
        .contains(&C::RoleRequired)
    );
    assert!(
        codes(&simple(
            "{\"claim\":\"verification_definition_body\",\"target\":\"d\"}"
        ))
        .contains(&C::RoleRequired)
    );
    assert!(
        codes(&simple(
            "{\"claim\":\"source_body\",\"role\":\"nope\",\"target\":\"s\"}"
        ))
        .contains(&C::RoleForbidden)
    );
    assert!(
        codes(&doc(
            ",\"asserted_prefix_root_id\":\"p\"",
            "{\"claim\":\"entry_point_presence\",\"role\":\"nope\",\"target\":\"e\"}"
        ))
        .contains(&C::RoleForbidden)
    );
}

#[test]
fn a_semantic_claim_key_may_be_mapped_only_once() {
    assert!(codes(&simple(&format!("{SOURCE},{SOURCE}"))).contains(&C::DuplicateClaim));
    let same_role = "{\"claim\":\"runtime_artifact_body\",\"role\":\"r\",\"target\":\"a\"}";
    assert!(codes(&simple(&format!("{same_role},{same_role}"))).contains(&C::DuplicateClaim));
    // Different roles under the same kind are different claim keys.
    let other_role = "{\"claim\":\"runtime_artifact_body\",\"role\":\"s\",\"target\":\"b\"}";
    assert!(parse_binding_plan(&simple(&format!("{same_role},{other_role}"))).is_ok());
    let entry = doc(
        ",\"asserted_prefix_root_id\":\"p\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"e\"},\
         {\"claim\":\"entry_point_presence\",\"target\":\"f\"}",
    );
    assert!(codes(&entry).contains(&C::DuplicateClaim));
}

// ------------------------------------------- conditional prefix-root assertion

#[test]
fn the_prefix_root_assertion_is_present_exactly_when_an_entry_claim_is_mapped() {
    let entry_without = simple("{\"claim\":\"entry_point_presence\",\"target\":\"e\"}");
    assert!(codes(&entry_without).contains(&C::AssertedPrefixRootRequired));

    let body_without = simple("{\"claim\":\"entry_point_body\",\"target\":\"e\"}");
    assert!(codes(&body_without).contains(&C::AssertedPrefixRootRequired));

    let root_without_entry = doc(",\"asserted_prefix_root_id\":\"p\"", SOURCE);
    assert!(codes(&root_without_entry).contains(&C::AssertedPrefixRootForbidden));

    let both = doc(
        ",\"asserted_prefix_root_id\":\"p\"",
        "{\"claim\":\"entry_point_presence\",\"target\":\"e\"}",
    );
    assert!(parse_binding_plan(&both).is_ok());
}

// ------------------------------------------------------------------- ceilings

#[test]
fn every_documented_ceiling_is_exact_on_both_sides() {
    let at = "d".repeat(MAX_ID_BYTES);
    let over = "d".repeat(MAX_ID_BYTES + 1);
    assert!(
        parse_binding_plan(&simple(&format!(
            "{{\"claim\":\"source_body\",\"target\":\"{at}\"}}"
        )))
        .is_ok()
    );
    assert!(
        codes(&simple(&format!(
            "{{\"claim\":\"source_body\",\"target\":\"{over}\"}}"
        )))
        .contains(&C::IdGrammar)
    );

    let entries: Vec<String> = (0..MAX_CLAIMS)
        .map(|i| {
            format!("{{\"claim\":\"runtime_artifact_body\",\"role\":\"r{i}\",\"target\":\"t{i}\"}}")
        })
        .collect();
    assert!(parse_binding_plan(&simple(&entries.join(","))).is_ok());
    let over_entries: Vec<String> = (0..=MAX_CLAIMS)
        .map(|i| {
            format!("{{\"claim\":\"runtime_artifact_body\",\"role\":\"r{i}\",\"target\":\"t{i}\"}}")
        })
        .collect();
    assert!(codes(&simple(&over_entries.join(","))).contains(&C::ClaimCountLimit));

    assert!(
        parse_binding_plan(&vec![b' '; MAX_PLAN_BYTES + 1])
            .unwrap_err()
            .contains(C::InputTooLarge)
    );
    assert!(codes(&simple("")).contains(&C::EmptyMapping));
    assert!(
        parse_binding_plan(b"[[[[[1]]]]]")
            .unwrap_err()
            .contains(C::NestingTooDeep)
    );
}

#[test]
fn identifier_and_digest_grammar_is_enforced() {
    for bad in ["Upper", "-lead", "sp ace", "", "sla/sh", "tab\\t"] {
        let raw = simple(&format!(
            "{{\"claim\":\"source_body\",\"target\":\"{bad}\"}}"
        ));
        assert!(
            codes(&raw).contains(&C::IdGrammar) || codes(&raw).contains(&C::MalformedJson),
            "target {bad:?} must be refused"
        );
    }
    for bad in ["", "abc", &"a".repeat(63), &"A".repeat(64), &"g".repeat(64)] {
        let raw = format!(
            "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
             \"subject_spec_sha256\":\"{bad}\",\"claims\":[{SOURCE}]}}"
        );
        assert!(
            codes(raw.as_bytes()).contains(&C::DigestGrammar),
            "digest {bad:?} must be refused"
        );
    }
}

// ------------------------------------------------------------------ robustness

#[test]
fn malformed_input_never_panics_and_ordering_is_deterministic() {
    let corpus: [&[u8]; 11] = [
        b"",
        b"{",
        b"[]",
        b"null",
        b"\xff\xfe\x00",
        b"{\"schema\":}",
        b"{\"schema\":\"helm-binding-plan\",\"version\":1,\"claims\":1}",
        b"{\"claims\":[{\"claim\":[]}]}",
        b"{\"claims\":[1,2,3]}",
        b"[[[[[[[[[[]]]]]]]]]]",
        b"{\"a\":{\"b\":{\"c\":{\"d\":{\"e\":1}}}}}",
    ];
    for raw in corpus {
        let first = parse_binding_plan(raw);
        let second = parse_binding_plan(raw);
        assert_eq!(first.is_ok(), second.is_ok());
        if let (Err(a), Err(b)) = (&first, &second) {
            assert_eq!(a, b, "diagnostics must be deterministic");
        }
    }
}

/// An independent seeded corpus with a reviewer-chosen seed and structure-aware
/// mutations. Any failure is reproducible from the printed seed and case index.
#[test]
fn seeded_corpus_never_panics_and_preserves_exact_bytes() {
    const SEED: u64 = 0x00B1_9D00_5EED_0001;
    let template = doc(
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"source_body\",\"target\":\"src\"},\
         {\"claim\":\"runtime_artifact_body\",\"role\":\"rt\",\"target\":\"a1\"},\
         {\"claim\":\"entry_point_presence\",\"target\":\"entry\"}",
    );
    let interesting: [&[u8]; 10] = [
        b"\"", b"\\", b"{", b"}", b"[", b"]", b",", b":", b"\\u0000", b"\xff",
    ];
    let mut state = SEED;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for case in 0..1024u32 {
        let mut m = template.clone();
        for _ in 0..(1 + next() % 5) {
            if m.is_empty() {
                break;
            }
            let idx = (next() as usize) % m.len();
            match next() % 5 {
                0 => m[idx] = (next() % 256) as u8,
                1 => {
                    m.remove(idx);
                }
                2 => m.insert(idx, (next() % 256) as u8),
                3 => {
                    let chunk = interesting[(next() as usize) % interesting.len()];
                    for (o, b) in chunk.iter().enumerate() {
                        m.insert(idx + o, *b);
                    }
                }
                _ => {
                    let end = (idx + 1 + (next() as usize % 16)).min(m.len());
                    m.drain(idx..end);
                }
            }
        }
        let a = parse_binding_plan(&m);
        let b = parse_binding_plan(&m);
        assert_eq!(a.is_ok(), b.is_ok(), "seed {SEED:#x} case {case}");
        match (&a, &b) {
            (Err(x), Err(y)) => assert_eq!(x, y, "seed {SEED:#x} case {case} ordering"),
            (Ok(x), Ok(y)) => {
                assert_eq!(x.sha256(), y.sha256());
                assert_eq!(
                    x.exact_bytes(),
                    m.as_slice(),
                    "seed {SEED:#x} case {case}: identity is over the exact input bytes"
                );
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn diagnostics_are_bounded_and_carry_no_untrusted_text() {
    let raw = doc(
        ",\"extra\":1",
        "{\"claim\":\"nope\",\"target\":\"WRONG\"},{\"claim\":\"source_body\"}",
    );
    let errors = parse_binding_plan(&raw).unwrap_err();
    assert!(errors.as_slice().len() <= helm_bind::MAX_PLAN_ERRORS);
    let rendered = errors.to_string();
    assert!(!rendered.contains("WRONG"), "no untrusted text: {rendered}");
    assert!(!rendered.contains('/'), "no path: {rendered}");
    for e in errors.as_slice() {
        assert!(
            [
                "plan",
                "schema",
                "version",
                "subject_spec_sha256",
                "asserted_prefix_root_id",
                "claims",
            ]
            .contains(&e.locator()),
            "locator {} must come from the closed schema",
            e.locator()
        );
    }
}
