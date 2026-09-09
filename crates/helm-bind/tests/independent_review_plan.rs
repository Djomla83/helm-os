//! Independent adversarial review of the binding-plan contract.
//!
//! Written by the independent reviewer of candidate
//! `832a112decaa333ee3b618d52e966a20c443513e`, not by the implementation author.
//! Expected values come from the accepted contract and from independent oracles,
//! never from the code under test. Pure, so it runs on every platform.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use helm_bind::{
    BindingPlanErrorCode as C, MAX_CLAIMS, MAX_ID_BYTES, MAX_JSON_DEPTH, MAX_PLAN_BYTES,
    parse_binding_plan,
};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";
const SOURCE: &str = "{\"claim\":\"source_body\",\"target\":\"src\"}";

fn doc(extra: &str, claims: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\"{extra},\"claims\":[{claims}]}}"
    )
    .into_bytes()
}

fn codes(raw: &[u8]) -> Vec<C> {
    parse_binding_plan(raw).unwrap_err().codes()
}

fn sha256_hex(raw: &[u8]) -> String {
    use sha2::Digest as _;
    let d: [u8; 32] = sha2::Sha256::digest(raw).into();
    d.iter().map(|b| format!("{b:02x}")).collect()
}

// ------------------------------------------------------------ exact identity

/// The mapping identity must be SHA-256 of the exact accepted bytes, checked
/// against an oracle the crate under test does not supply, and every accepted
/// one-byte change must move it.
#[test]
fn mapping_identity_is_the_independent_digest_of_the_exact_bytes() {
    let raw = doc("", SOURCE);
    let plan = parse_binding_plan(&raw).unwrap();
    assert_eq!(plan.exact_bytes(), raw.as_slice());
    let expected = sha256_hex(&raw);
    assert_eq!(plan.sha256().to_hex(), expected);
    assert_eq!(plan.sha256().to_hex().len(), 64);
    // Every one of the 32 bytes is rendered in place: no truncation, no prefix.
    for (i, chunk) in expected.as_bytes().chunks(2).enumerate() {
        assert_eq!(
            &plan.sha256().to_hex()[2 * i..2 * i + 2],
            std::str::from_utf8(chunk).unwrap()
        );
    }

    // Every one-byte change that still parses must move the identity. The subject
    // digest is 64 hex characters, so each of its positions gives an accepted
    // one-byte variant; whitespace insertions give more.
    let mut identities = std::collections::BTreeSet::new();
    identities.insert(plan.sha256().to_hex());
    let digest_at = raw
        .windows(SUBJECT.len())
        .position(|w| w == SUBJECT.as_bytes())
        .expect("the subject digest appears in the document");
    for i in 0..SUBJECT.len() {
        let mut v = raw.clone();
        v[digest_at + i] = if v[digest_at + i] == b'a' { b'b' } else { b'a' };
        let other = parse_binding_plan(&v)
            .unwrap_or_else(|e| panic!("digest position {i} must stay valid: {e}"));
        assert_eq!(other.exact_bytes(), v.as_slice());
        assert!(
            identities.insert(other.sha256().to_hex()),
            "digest position {i} did not move the mapping identity"
        );
        assert_ne!(
            other.subject_spec_sha256(),
            plan.subject_spec_sha256(),
            "digest position {i} must also move the declared subject"
        );
    }
    assert_eq!(
        identities.len(),
        SUBJECT.len() + 1,
        "each digest position is a distinct identity"
    );

    // Outside the digest the closed schema has essentially no slack: flipping bit
    // five of any byte turns a keyword, an identifier or a structural character
    // into something the schema refuses, which is itself worth recording. One-byte
    // *insertions* of whitespace are accepted, and each must move the identity.
    let mut refused_flips = 0usize;
    for i in 0..raw.len() {
        if (digest_at..digest_at + SUBJECT.len()).contains(&i) {
            continue;
        }
        let mut v = raw.clone();
        v[i] ^= 0x20;
        assert!(
            parse_binding_plan(&v).is_err(),
            "byte {i} outside the digest unexpectedly stayed valid"
        );
        refused_flips += 1;
    }
    assert!(
        refused_flips > 100,
        "the sweep must actually cover the document"
    );

    let mut inserted = 0usize;
    for i in 0..raw.len() {
        if !matches!(raw[i], b',' | b':') {
            continue;
        }
        let mut v = raw.clone();
        v.insert(i + 1, b' ');
        let other = parse_binding_plan(&v)
            .unwrap_or_else(|e| panic!("a space after byte {i} must stay valid: {e}"));
        inserted += 1;
        assert!(
            identities.insert(other.sha256().to_hex()),
            "a space after byte {i} did not move the mapping identity"
        );
        assert_eq!(
            other.claims(),
            plan.claims(),
            "the inert model is unchanged"
        );
    }
    assert!(inserted > 5, "expected several whitespace variants");
}

/// Whitespace, field order and a legal escape spelling are all different
/// documents, and none of them is canonicalised away.
#[test]
fn no_canonicalisation_of_any_kind() {
    let base = parse_binding_plan(&doc("", SOURCE)).unwrap();
    let variants = [
        format!(
            "{{ \"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
             \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
        ),
        format!(
            "{{\"claims\":[{SOURCE}],\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
             \"subject_spec_sha256\":\"{SUBJECT}\"}}"
        ),
        format!(
            "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
             \"subject_spec_sha256\":\"{SUBJECT}\",\
             \"claims\":[{{\"claim\":\"source_body\",\"target\":\"\\u0073rc\"}}]}}"
        ),
    ];
    let mut seen = std::collections::BTreeSet::new();
    seen.insert(base.sha256().to_hex());
    for raw in &variants {
        let plan = parse_binding_plan(raw.as_bytes()).unwrap();
        assert_eq!(plan.claims(), base.claims(), "the inert model is the same");
        assert!(
            seen.insert(plan.sha256().to_hex()),
            "a formatting variant collided with another identity"
        );
        assert_eq!(plan.sha256().to_hex(), sha256_hex(raw.as_bytes()));
    }
}

// --------------------------------------------------------- duplicate detection

#[test]
fn decoded_duplicate_keys_are_rejected_wherever_they_appear() {
    // Root, plain and escaped.
    for second in ["\"version\"", "\"\\u0076ersion\""] {
        let raw = format!(
            "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",{second}:\"0.1\",\
             \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]}}"
        );
        assert!(
            parse_binding_plan(raw.as_bytes())
                .unwrap_err()
                .contains(C::DuplicateKey),
            "root duplicate spelled {second} must be rejected"
        );
    }
    // Inside a claims entry, plain and escaped, for each field.
    for field in ["claim", "role", "target"] {
        for escaped in [false, true] {
            let second = if escaped {
                let mut s = String::from("\\u00");
                s.push_str(&format!("{:02x}", field.as_bytes()[0]));
                format!("{s}{}", &field[1..])
            } else {
                field.to_owned()
            };
            let raw = doc(
                "",
                &format!(
                    "{{\"claim\":\"runtime_artifact_body\",\"role\":\"a\",\"target\":\"t\",\
                     \"{second}\":\"x\"}}"
                ),
            );
            assert!(
                parse_binding_plan(&raw)
                    .unwrap_err()
                    .contains(C::DuplicateKey),
                "nested duplicate of {field} (escaped={escaped}) must be rejected"
            );
        }
    }
}

/// The control must not over-fire: key-shaped text inside a value is data.
#[test]
fn key_shaped_text_inside_a_value_is_not_a_duplicate_key() {
    // A role whose spelling contains no JSON metacharacters but reads like a key.
    let raw = doc(
        "",
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"claim-target-role\",\"target\":\"t\"}",
    );
    let plan = parse_binding_plan(&raw).unwrap();
    assert_eq!(plan.claims()[0].role(), Some("claim-target-role"));

    // Sibling objects legitimately reuse the same field names.
    let siblings = doc(
        "",
        "{\"claim\":\"runtime_artifact_body\",\"role\":\"a\",\"target\":\"t1\"},\
         {\"claim\":\"runtime_artifact_body\",\"role\":\"b\",\"target\":\"t2\"}",
    );
    assert!(parse_binding_plan(&siblings).is_ok());
}

// ------------------------------------------------------------ exact boundaries

#[test]
fn every_ceiling_is_exact_on_both_sides() {
    // Document size. A plan of exactly the ceiling must still parse, so it is
    // padded with whitespace inside the object rather than truncated.
    let head = format!(
        "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{SOURCE}]"
    );
    let tail = "}";
    let pad = MAX_PLAN_BYTES - head.len() - tail.len();
    let at = format!("{head}{}{tail}", " ".repeat(pad));
    assert_eq!(at.len(), MAX_PLAN_BYTES);
    assert!(
        parse_binding_plan(at.as_bytes()).is_ok(),
        "a document of exactly {MAX_PLAN_BYTES} bytes must be accepted"
    );
    let over = format!("{head}{}{tail}", " ".repeat(pad + 1));
    assert_eq!(over.len(), MAX_PLAN_BYTES + 1);
    assert!(
        parse_binding_plan(over.as_bytes())
            .unwrap_err()
            .contains(C::InputTooLarge)
    );

    // Mapping count.
    let entry = |i: usize| {
        format!("{{\"claim\":\"runtime_artifact_body\",\"role\":\"r{i}\",\"target\":\"t{i}\"}}")
    };
    let at_count: Vec<String> = (0..MAX_CLAIMS).map(entry).collect();
    assert!(parse_binding_plan(&doc("", &at_count.join(","))).is_ok());
    let over_count: Vec<String> = (0..=MAX_CLAIMS).map(entry).collect();
    assert!(codes(&doc("", &over_count.join(","))).contains(&C::ClaimCountLimit));

    // Identifier length, for target, role and the asserted root.
    let at_id = "d".repeat(MAX_ID_BYTES);
    let over_id = "d".repeat(MAX_ID_BYTES + 1);
    assert!(
        parse_binding_plan(&doc(
            "",
            &format!("{{\"claim\":\"source_body\",\"target\":\"{at_id}\"}}")
        ))
        .is_ok()
    );
    assert!(
        codes(&doc(
            "",
            &format!("{{\"claim\":\"source_body\",\"target\":\"{over_id}\"}}")
        ))
        .contains(&C::IdGrammar)
    );
    assert!(
        codes(&doc(
            "",
            &format!(
                "{{\"claim\":\"runtime_artifact_body\",\"role\":\"{over_id}\",\"target\":\"t\"}}"
            )
        ))
        .contains(&C::IdGrammar)
    );
    assert!(
        codes(&doc(
            &format!(",\"asserted_prefix_root_id\":\"{over_id}\""),
            "{\"claim\":\"entry_point_presence\",\"target\":\"e\"}"
        ))
        .contains(&C::IdGrammar)
    );

    // Nesting. The document itself is three deep, so the boundary is probed with
    // bare arrays: MAX_JSON_DEPTH nested containers pass the depth guard and then
    // fail the schema, and one more is refused by the depth guard itself.
    let at_depth = format!(
        "{}1{}",
        "[".repeat(MAX_JSON_DEPTH),
        "]".repeat(MAX_JSON_DEPTH)
    );
    assert!(
        !codes(at_depth.as_bytes()).contains(&C::NestingTooDeep),
        "exactly {MAX_JSON_DEPTH} containers must clear the depth guard"
    );
    let over_depth = format!(
        "{}1{}",
        "[".repeat(MAX_JSON_DEPTH + 1),
        "]".repeat(MAX_JSON_DEPTH + 1)
    );
    assert!(codes(over_depth.as_bytes()).contains(&C::NestingTooDeep));
}

// ------------------------------------------------------------ role and shape

#[test]
fn role_rules_hold_for_every_claim_kind() {
    let cases: [(&str, bool); 5] = [
        ("source_body", false),
        ("runtime_artifact_body", true),
        ("verification_definition_body", true),
        ("entry_point_presence", false),
        ("entry_point_body", false),
    ];
    for (kind, takes_role) in cases {
        let extra = if kind.starts_with("entry_point") {
            ",\"asserted_prefix_root_id\":\"p\""
        } else {
            ""
        };
        let with_role = doc(
            extra,
            &format!("{{\"claim\":\"{kind}\",\"role\":\"r\",\"target\":\"t\"}}"),
        );
        let without_role = doc(extra, &format!("{{\"claim\":\"{kind}\",\"target\":\"t\"}}"));
        if takes_role {
            assert!(
                parse_binding_plan(&with_role).is_ok(),
                "{kind} takes a role"
            );
            assert!(codes(&without_role).contains(&C::RoleRequired), "{kind}");
        } else {
            assert!(
                parse_binding_plan(&without_role).is_ok(),
                "{kind} takes no role"
            );
            assert!(codes(&with_role).contains(&C::RoleForbidden), "{kind}");
        }
    }
}

#[test]
fn the_asserted_prefix_root_matrix_is_exact() {
    let entry = "{\"claim\":\"entry_point_presence\",\"target\":\"e\"}";
    let body = "{\"claim\":\"entry_point_body\",\"target\":\"e\"}";
    let root = ",\"asserted_prefix_root_id\":\"p\"";
    // present iff an entry claim is mapped
    assert!(parse_binding_plan(&doc("", SOURCE)).is_ok());
    assert!(parse_binding_plan(&doc(root, entry)).is_ok());
    assert!(parse_binding_plan(&doc(root, body)).is_ok());
    assert!(parse_binding_plan(&doc(root, &format!("{entry},{body}"))).is_ok());
    assert!(codes(&doc("", entry)).contains(&C::AssertedPrefixRootRequired));
    assert!(codes(&doc("", body)).contains(&C::AssertedPrefixRootRequired));
    assert!(codes(&doc(root, SOURCE)).contains(&C::AssertedPrefixRootForbidden));
    assert!(
        parse_binding_plan(&doc(root, &format!("{SOURCE},{entry}"))).is_ok(),
        "a mixed mapping containing an entry claim keeps the assertion"
    );
}

// ---------------------------------------------------------------- robustness

#[test]
fn adversarial_documents_never_panic_and_stay_deterministic() {
    let big = "9".repeat(400);
    let owned: Vec<String> = vec![
        format!("{{\"schema\":\"helm-binding-plan\",\"version\":{big},\"claims\":[]}}"),
        format!("{{\"subject_spec_sha256\":{big}}}"),
        format!(
            "{{\"schema\":\"helm-binding-plan\",\"version\":\"0.1\",\
             \"subject_spec_sha256\":\"{SUBJECT}\",\"claims\":[{{\"claim\":\"source_body\",\
             \"target\":\"t\",\"role\":{big}}}]}}"
        ),
        "{\"claims\":null}".to_owned(),
        "{\"claims\":[null]}".to_owned(),
        "{\"claims\":[[]]}".to_owned(),
        "\u{feff}{}".to_owned(),
        "{}{}".to_owned(),
        "{\"a\":\"\\ud800\"}".to_owned(),
    ];
    let mut corpus: Vec<&[u8]> = owned.iter().map(String::as_bytes).collect();
    corpus.extend_from_slice(&[b"", b"{", b"[", b"null", b"true", b"0", b"\xff\xfe"]);
    for raw in corpus {
        let a = parse_binding_plan(raw);
        let b = parse_binding_plan(raw);
        assert_eq!(a.is_ok(), b.is_ok());
        if let (Err(x), Err(y)) = (&a, &b) {
            assert_eq!(x, y, "diagnostics must be deterministic");
            assert!(x.as_slice().len() <= helm_bind::MAX_PLAN_ERRORS);
        }
    }
}

/// A new corpus with a reviewer-chosen seed, different from the author's, and
/// with structure-aware mutations. Any failure reproduces from seed and index.
#[test]
fn independent_seeded_corpus() {
    const SEED: u64 = 0x0117_2026_0909_0001;
    let template = doc(
        ",\"asserted_prefix_root_id\":\"prefix\"",
        "{\"claim\":\"source_body\",\"target\":\"src\"},\
         {\"claim\":\"runtime_artifact_body\",\"role\":\"rt\",\"target\":\"a1\"},\
         {\"claim\":\"verification_definition_body\",\"role\":\"proto\",\"target\":\"d1\"},\
         {\"claim\":\"entry_point_presence\",\"target\":\"entry\"},\
         {\"claim\":\"entry_point_body\",\"target\":\"entry\"}",
    );
    let interesting: [&[u8]; 12] = [
        b"\"",
        b"\\",
        b"{",
        b"}",
        b"[",
        b"]",
        b",",
        b":",
        b"\\u0000",
        b"\\ud800",
        b"\xff",
        b"source_body",
    ];
    let mut state = SEED;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut accepted = 0usize;
    for case in 0..2048u32 {
        let mut m = template.clone();
        for _ in 0..(1 + next() % 6) {
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
                    let end = (idx + 1 + (next() as usize % 24)).min(m.len());
                    m.drain(idx..end);
                }
            }
        }
        let a = parse_binding_plan(&m);
        let b = parse_binding_plan(&m);
        assert_eq!(a.is_ok(), b.is_ok(), "seed {SEED:#x} case {case}");
        match (&a, &b) {
            (Err(x), Err(y)) => assert_eq!(x, y, "seed {SEED:#x} case {case}"),
            (Ok(x), Ok(y)) => {
                accepted += 1;
                assert_eq!(x.sha256(), y.sha256());
                assert_eq!(
                    x.exact_bytes(),
                    m.as_slice(),
                    "seed {SEED:#x} case {case}: exact bytes must be retained"
                );
                assert_eq!(x.sha256().to_hex(), sha256_hex(&m));
                // Nothing a mutation can do may create an unmapped claim kind or a
                // duplicate semantic claim key.
                let mut keys: Vec<(helm_bind::ClaimKind, Option<&str>)> =
                    x.claims().iter().map(|c| (c.kind(), c.role())).collect();
                let before = keys.len();
                keys.sort();
                keys.dedup();
                assert_eq!(
                    before,
                    keys.len(),
                    "seed {SEED:#x} case {case}: duplicate claim key"
                );
            }
            _ => unreachable!(),
        }
    }
    assert!(
        accepted > 0,
        "seed {SEED:#x} produced no accepted mutant; the corpus would be vacuous"
    );
}
