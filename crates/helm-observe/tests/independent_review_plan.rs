//! Independent adversarial plan-contract tests for helm-observe 0.1.
//!
//! Written by the independent reviewer of candidate
//! `e2a62081ece52391b30ede153eee139103c98e31`. Expected values come from the
//! documented contract, not from the implementation under test. Pure: no
//! filesystem, process or network access.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use helm_observe::{
    MAX_COMPONENT_BYTES, MAX_ID_BYTES, MAX_JSON_DEPTH, MAX_PATH_BYTES, MAX_PATH_COMPONENTS,
    MAX_PLAN_BYTES, MAX_ROOTS, MAX_TARGETS, PlanErrorCode as C, parse_plan,
};

const SUBJECT: &str = "b60672385dc371a79b78f90636206bc1a0a7e0f5e017c4bdec323c545b74521e";

fn doc(roots: &str, targets: &str) -> Vec<u8> {
    format!(
        "{{\"schema\":\"helm-observation-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"roots\":[{roots}],\"targets\":[{targets}]}}"
    )
    .into_bytes()
}

fn accepted(raw: &[u8]) -> bool {
    parse_plan(raw).is_ok()
}

// ------------------------------------------------------- duplicate-key control

/// The crate documents that duplicate object keys are rejected **before** a map
/// can silently keep the last one. That control must hold at every object depth,
/// including objects nested inside the `roots` and `targets` arrays.
#[test]
fn duplicate_keys_inside_array_nested_objects_are_rejected() {
    let dup_target_id = doc(
        "{\"id\":\"r\"}",
        "{\"id\":\"decoy\",\"root\":\"r\",\"observable\":\"directory_metadata\",\"id\":\"real\"}",
    );
    assert!(
        parse_plan(&dup_target_id)
            .unwrap_err()
            .contains(C::DuplicateKey),
        "a duplicate target id key must not be resolved silently to the last value"
    );

    let dup_target_path = doc(
        "{\"id\":\"r\"}",
        "{\"id\":\"t\",\"root\":\"r\",\"path\":\"harmless\",\
         \"observable\":\"regular_file_sha256\",\"path\":\"other/elsewhere\"}",
    );
    assert!(
        parse_plan(&dup_target_path)
            .unwrap_err()
            .contains(C::DuplicateKey),
        "two path keys in one target must not silently select the last spelling"
    );

    let dup_root_id = doc(
        "{\"id\":\"decoy\",\"id\":\"r\"}",
        "{\"id\":\"t\",\"root\":\"r\",\"observable\":\"directory_metadata\"}",
    );
    assert!(
        parse_plan(&dup_root_id)
            .unwrap_err()
            .contains(C::DuplicateKey),
        "a duplicate root id key must be rejected"
    );
}

/// JSON permits several spellings of the same decoded key. A document whose plain
/// and escaped spellings collide must be rejected, not silently resolved to the
/// last occurrence.
#[test]
fn duplicate_keys_written_with_json_escapes_are_rejected() {
    // "\u0074argets" decodes to "targets".
    let escaped_targets = format!(
        "{{\"schema\":\"helm-observation-plan\",\"version\":\"0.1\",\
         \"subject_spec_sha256\":\"{SUBJECT}\",\"roots\":[{{\"id\":\"r\"}}],\
         \"targets\":[{{\"id\":\"decoy\",\"root\":\"r\",\"observable\":\"directory_metadata\"}}],\
         \"\\u0074argets\":[{{\"id\":\"real\",\"root\":\"r\",\"path\":\"elsewhere\",\
         \"observable\":\"regular_file_sha256\"}}]}}"
    );
    let err = parse_plan(escaped_targets.as_bytes()).unwrap_err();
    assert!(
        err.contains(C::DuplicateKey),
        "an escaped duplicate of a schema key must be rejected, got {err}"
    );

    // "\u0073chema" decodes to "schema"; the duplicate must be caught on the key,
    // independently of whether the duplicated value happens to be invalid.
    let escaped_schema = format!(
        "{{\"schema\":\"helm-observation-plan\",\"\\u0073chema\":\"helm-observation-plan\",\
         \"version\":\"0.1\",\"subject_spec_sha256\":\"{SUBJECT}\",\
         \"roots\":[{{\"id\":\"r\"}}],\
         \"targets\":[{{\"id\":\"t\",\"root\":\"r\",\"observable\":\"directory_metadata\"}}]}}"
    );
    assert!(
        parse_plan(escaped_schema.as_bytes())
            .unwrap_err()
            .contains(C::DuplicateKey)
    );
}

/// The control must not become over-eager: identical key spellings in *sibling*
/// objects, and a value string that merely looks like a repeated key, stay valid.
#[test]
fn distinct_objects_may_reuse_the_same_key_names() {
    let raw = doc(
        "{\"id\":\"r1\"},{\"id\":\"r2\"}",
        "{\"id\":\"t1\",\"root\":\"r1\",\"observable\":\"directory_metadata\"},\
         {\"id\":\"t2\",\"root\":\"r2\",\"path\":\"a/b\",\"observable\":\"regular_file_sha256\"}",
    );
    assert!(
        accepted(&raw),
        "sibling objects legitimately reuse key names"
    );

    // Key-shaped text inside a string *value* is data, not a second key.
    let value_looks_like_a_key = doc(
        "{\"id\":\"r\"}",
        "{\"id\":\"t\",\"root\":\"r\",\"path\":\"a,\\\"id\\\":b\",\
         \"observable\":\"regular_file_sha256\"}",
    );
    let parsed = parse_plan(&value_looks_like_a_key)
        .unwrap_or_else(|e| panic!("value text must not be read as a duplicate key: {e}"));
    assert_eq!(parsed.targets()[0].path(), Some("a,\"id\":b"));
}

// ------------------------------------------------------------ exact boundaries

#[test]
fn every_documented_ceiling_is_exact_on_both_sides() {
    let target = |path: &str| {
        format!(
            "{{\"id\":\"t\",\"root\":\"r\",\"path\":\"{path}\",\"observable\":\"regular_file_sha256\"}}"
        )
    };

    // Path bytes: the limit itself is accepted, one more is refused. Components
    // stay inside the per-component ceiling so only the total length is tested.
    let path_of = |total: usize| {
        // Components of at most 100 bytes, so only the total length is at issue.
        let n = total.div_ceil(101);
        let payload = total - (n - 1);
        let base = payload / n;
        let extra = payload % n;
        let parts: Vec<String> = (0..n)
            .map(|i| "a".repeat(base + usize::from(i < extra)))
            .collect();
        parts.join("/")
    };
    let at = path_of(MAX_PATH_BYTES);
    assert_eq!(at.len(), MAX_PATH_BYTES);
    assert!(
        accepted(&doc("{\"id\":\"r\"}", &target(&at))),
        "a path of exactly {MAX_PATH_BYTES} bytes must be accepted"
    );
    let over = path_of(MAX_PATH_BYTES + 1);
    assert_eq!(over.len(), MAX_PATH_BYTES + 1);
    assert!(
        parse_plan(&doc("{\"id\":\"r\"}", &target(&over)))
            .unwrap_err()
            .contains(C::PathTooLong)
    );

    // Component bytes.
    let comp_at = "b".repeat(MAX_COMPONENT_BYTES);
    assert!(accepted(&doc("{\"id\":\"r\"}", &target(&comp_at))));
    let comp_over = "b".repeat(MAX_COMPONENT_BYTES + 1);
    assert!(
        parse_plan(&doc("{\"id\":\"r\"}", &target(&comp_over)))
            .unwrap_err()
            .contains(C::PathComponentTooLong)
    );

    // Component count.
    let at_count = vec!["c"; MAX_PATH_COMPONENTS].join("/");
    assert!(accepted(&doc("{\"id\":\"r\"}", &target(&at_count))));
    let over_count = vec!["c"; MAX_PATH_COMPONENTS + 1].join("/");
    assert!(
        parse_plan(&doc("{\"id\":\"r\"}", &target(&over_count)))
            .unwrap_err()
            .contains(C::PathComponentCount)
    );

    // Identifier bytes.
    let id_at = "d".repeat(MAX_ID_BYTES);
    let id_over = "d".repeat(MAX_ID_BYTES + 1);
    assert!(accepted(&doc(
        &format!("{{\"id\":\"{id_at}\"}}"),
        &format!("{{\"id\":\"t\",\"root\":\"{id_at}\",\"observable\":\"directory_metadata\"}}")
    )));
    assert!(
        parse_plan(&doc(
            &format!("{{\"id\":\"{id_over}\"}}"),
            &format!(
                "{{\"id\":\"t\",\"root\":\"{id_over}\",\"observable\":\"directory_metadata\"}}"
            )
        ))
        .unwrap_err()
        .contains(C::IdGrammar)
    );

    // Root and target counts.
    let roots_at: Vec<String> = (0..MAX_ROOTS)
        .map(|i| format!("{{\"id\":\"r{i}\"}}"))
        .collect();
    assert!(accepted(&doc(
        &roots_at.join(","),
        "{\"id\":\"t\",\"root\":\"r0\",\"observable\":\"directory_metadata\"}"
    )));
    let roots_over: Vec<String> = (0..=MAX_ROOTS)
        .map(|i| format!("{{\"id\":\"r{i}\"}}"))
        .collect();
    assert!(
        parse_plan(&doc(
            &roots_over.join(","),
            "{\"id\":\"t\",\"root\":\"r0\",\"observable\":\"directory_metadata\"}"
        ))
        .unwrap_err()
        .contains(C::RootCountLimit)
    );

    let targets_at: Vec<String> = (0..MAX_TARGETS)
        .map(|i| {
            format!("{{\"id\":\"t{i}\",\"root\":\"r\",\"observable\":\"directory_metadata\"}}")
        })
        .collect();
    assert!(accepted(&doc("{\"id\":\"r\"}", &targets_at.join(","))));
    let targets_over: Vec<String> = (0..=MAX_TARGETS)
        .map(|i| {
            format!("{{\"id\":\"t{i}\",\"root\":\"r\",\"observable\":\"directory_metadata\"}}")
        })
        .collect();
    assert!(
        parse_plan(&doc("{\"id\":\"r\"}", &targets_over.join(",")))
            .unwrap_err()
            .contains(C::TargetCountLimit)
    );

    // Document size.
    assert!(
        parse_plan(&vec![b' '; MAX_PLAN_BYTES + 1])
            .unwrap_err()
            .contains(C::InputTooLarge)
    );
}

/// Nesting is bounded, and the bound is applied to the raw document rather than
/// to whatever the object model happens to keep.
#[test]
fn json_nesting_bound_is_enforced_on_the_raw_document() {
    let deep = format!(
        "{}1{}",
        "[".repeat(MAX_JSON_DEPTH + 1),
        "]".repeat(MAX_JSON_DEPTH + 1)
    );
    assert!(
        parse_plan(deep.as_bytes())
            .unwrap_err()
            .contains(C::NestingTooDeep)
    );
}

// ------------------------------------------------------- independent fuzz pass

/// A new independent corpus with a reviewer-chosen seed and structure-aware
/// mutations, not the author's byte-flip seed. Any failure is reproducible from
/// the printed seed and case index.
#[test]
fn independent_seeded_corpus_never_panics_and_stays_deterministic() {
    const SEED: u64 = 0x1234_5678_9ABC_DEF0;
    let template = doc(
        "{\"id\":\"r\"},{\"id\":\"s\"}",
        "{\"id\":\"t\",\"root\":\"r\",\"path\":\"a/b/c\",\"observable\":\"regular_file_sha256\"},\
         {\"id\":\"u\",\"root\":\"s\",\"observable\":\"directory_metadata\"}",
    );
    let interesting: [&[u8]; 12] = [
        b"\"", b"\\", b"{", b"}", b"[", b"]", b",", b":", b"\\u0000", b"\xff", b"/../", b"\\ud800",
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
        let a = parse_plan(&m);
        let b = parse_plan(&m);
        assert_eq!(
            a.is_ok(),
            b.is_ok(),
            "seed {SEED:#x} case {case} is not deterministic"
        );
        match (&a, &b) {
            (Err(x), Err(y)) => assert_eq!(x, y, "seed {SEED:#x} case {case} error order differs"),
            (Ok(x), Ok(y)) => {
                assert_eq!(x.sha256(), y.sha256());
                assert_eq!(
                    x.exact_bytes(),
                    m.as_slice(),
                    "seed {SEED:#x} case {case}: identity must be over the exact input bytes"
                );
            }
            _ => unreachable!(),
        }
    }
}

/// Plan identity is over the exact accepted bytes, with no canonicalisation, and
/// a mutation anywhere in the document changes the digest.
#[test]
fn plan_identity_tracks_every_input_byte() {
    let raw = doc(
        "{\"id\":\"r\"}",
        "{\"id\":\"t\",\"root\":\"r\",\"path\":\"a/b\",\"observable\":\"regular_file_sha256\"}",
    );
    let base = parse_plan(&raw).unwrap();
    assert_eq!(base.exact_bytes(), raw.as_slice());

    // Insert one space in every legal position between tokens; each is a distinct
    // authorisation identity even though the inert model is unchanged.
    let mut seen = std::collections::BTreeSet::new();
    seen.insert(base.sha256().to_hex());
    let mut variants = 0;
    for i in 0..raw.len() {
        if raw[i] != b',' && raw[i] != b':' {
            continue;
        }
        let mut v = raw.clone();
        v.insert(i + 1, b' ');
        let p = parse_plan(&v).unwrap();
        assert_eq!(p.root_ids(), base.root_ids());
        assert_eq!(p.targets(), base.targets());
        assert!(
            seen.insert(p.sha256().to_hex()),
            "whitespace variant {i} collided with an earlier identity"
        );
        variants += 1;
    }
    assert!(variants > 5, "expected several whitespace variants");
}
