//! Independent expectations: ADR-0021, documented schema 0.1 and parser budgets.
//! No product-code changes or external fuzzing framework are required by these tests.
#![allow(clippy::unwrap_used, clippy::expect_used)]
use helm_app_spec::{ErrorCode as C, MAX_INPUT_BYTES, parse_spec};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

const NOTES: &str = include_str!("fixtures/synthetic-notes.json");
const A0: &str = include_str!("fixtures/a0-7zip.json");

fn edit(pointer: &str, value: Value) -> Vec<u8> {
    let mut root: Value = serde_json::from_str(NOTES).unwrap();
    *root.pointer_mut(pointer).unwrap() = value;
    serde_json::to_vec(&root).unwrap()
}

fn codes(bytes: &[u8]) -> Vec<(C, String)> {
    let result = parse_spec(bytes).unwrap_err();
    assert_eq!(parse_spec(bytes).unwrap_err(), result);
    result
        .as_slice()
        .iter()
        .map(|e| (e.code(), e.location().to_owned()))
        .collect()
}

fn one(bytes: &[u8], code: C) {
    assert_eq!(codes(bytes), [(code, "$".to_owned())]);
}

#[test]
fn exact_byte_variants_preserve_all_desired_views() {
    let original = parse_spec(NOTES.as_bytes()).unwrap();
    let variants = [
        NOTES.replace("  ", "\t"),
        NOTES.replacen(
            "\"schema\": \"helm-app-spec\",\n  \"version\": \"0.1\"",
            "\"version\": \"0.1\",\n  \"schema\": \"helm-app-spec\"",
            1,
        ),
        NOTES.replace("synthetic-notes", "synthetic-\\u006eotes"),
        NOTES.trim_end_matches('\n').to_owned(),
    ];
    let mut hashes = vec![original.spec_sha256().as_str().to_owned()];
    for bytes in variants {
        assert_ne!(bytes, NOTES);
        let spec = parse_spec(bytes.as_bytes()).unwrap();
        assert_eq!(spec.application(), original.application());
        assert_eq!(spec.runtime(), original.runtime());
        assert_eq!(spec.environment(), original.environment());
        assert_eq!(spec.entry_point(), original.entry_point());
        assert_eq!(spec.verification(), original.verification());
        assert_ne!(spec, original);
        hashes.push(spec.spec_sha256().as_str().to_owned());
    }
    assert_eq!(
        hashes
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5
    );
    // Python hashlib checks these exact transformations independently in the review tool.
    println!(
        "REVIEW_IDENTITIES={}",
        serde_json::to_string(&hashes).unwrap()
    );
}

#[test]
fn exact_parser_boundaries_and_rejection_precedence() {
    for n in [15, 16] {
        one(
            serde_json::to_string(&vec![0; n]).unwrap().as_bytes(),
            C::FieldType,
        );
        let object = format!(
            "{{{}}}",
            (0..n)
                .map(|i| format!("\"k{i}\":0"))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert_eq!(codes(object.as_bytes())[0].0, C::FieldUnknown);
        let duplicate = format!("{},\"k0\":0}}", &object[..object.len() - 1]);
        one(
            duplicate.as_bytes(),
            if n == 15 {
                C::FieldDuplicate
            } else {
                C::JsonLimit
            },
        );
    }
    one(
        serde_json::to_string(&vec![0; 17]).unwrap().as_bytes(),
        C::JsonLimit,
    );
    one(
        format!(
            "{{{}}}",
            (0..17)
                .map(|i| format!("\"k{i}\":0"))
                .collect::<Vec<_>>()
                .join(",")
        )
        .as_bytes(),
        C::JsonLimit,
    );
    // Root + 15 array nodes + 239/240 scalar nodes = 255/256 nodes.
    for last in [15, 16] {
        let mut arrays = vec![vec![0; 16]; 15];
        arrays[14] = vec![0; last];
        one(&serde_json::to_vec(&arrays).unwrap(), C::FieldType);
    }
    let mut arrays = vec![vec![0; 16]; 15];
    arrays.push(vec![]); // Node 257 is an empty container, which still counts.
    one(&serde_json::to_vec(&arrays).unwrap(), C::JsonLimit);
    one(
        format!("{}0{}", "[".repeat(8), "]".repeat(8)).as_bytes(),
        C::FieldType,
    );
    one(
        format!("{}{}", "[".repeat(9), "]".repeat(9)).as_bytes(),
        C::FieldType,
    );
    one(
        format!("{}0{}", "[".repeat(9), "]".repeat(9)).as_bytes(),
        C::JsonLimit,
    );
    one(&vec![b'['; MAX_INPUT_BYTES], C::JsonLimit);
    for (n, expected) in [(2048, C::FieldType), (2049, C::JsonLimit)] {
        one(format!("\"{}\"", "a".repeat(n)).as_bytes(), expected);
        one(format!("\"{}\"", "\\u0061".repeat(n)).as_bytes(), expected);
    }
    for (n, expected) in [(80, C::FieldUnknown), (81, C::JsonLimit)] {
        assert_eq!(
            codes(format!("{{\"{}\":0}}", "\\u0061".repeat(n)).as_bytes())[0].0,
            expected
        );
    }
    // Limits deliberately terminate parsing before an oversized subtree is parsed.
    one(format!("[{}{{", "0,".repeat(16)).as_bytes(), C::JsonLimit);
    one(b"{\"x\":0,\"x\":", C::FieldDuplicate);
}

#[test]
fn number_representation_errors_are_distinguished_from_semantics() {
    let template = String::from_utf8(edit("/application/source/size", json!(424242))).unwrap();
    for token in [
        "0",
        "-0",
        "-1",
        "-9223372036854775809",
        "18446744073709551616",
        "1.0",
        "1e0",
        "1E+1",
        "1e-999999999999",
        "null",
        "false",
        "1099511627777",
    ] {
        let input = template.replace("424242", token);
        assert_eq!(
            codes(input.as_bytes()),
            [(C::SizeInvalid, "$.application.source.size".to_owned())],
            "{token}"
        );
    }
    for token in [
        "1e309",
        "1e999999999999",
        "01",
        "+1",
        "1.",
        "NaN",
        "Infinity",
        "--1",
        &"9".repeat(50000),
    ] {
        one(template.replace("424242", token).as_bytes(), C::JsonInvalid);
    }
    for token in ["1", "1099511627776"] {
        assert!(parse_spec(template.replace("424242", token).as_bytes()).is_ok());
    }
    one(b"null", C::FieldType);
    one(b"\"\xff\"", C::JsonInvalid);
    one(br#""\ud800""#, C::JsonInvalid);
    one(br#""\udc00""#, C::JsonInvalid);
    one(br#""\ud800\udc00""#, C::FieldType);
    one(format!("{NOTES} null").as_bytes(), C::JsonInvalid);
    assert!(parse_spec(format!("{NOTES}\r\n\t ").as_bytes()).is_ok());
}

#[test]
fn decoded_set_duplicates_and_case_variants_never_validate() {
    for (pointer, value, needle, replacement, expected) in [
        (
            "/runtime/artifacts",
            json!([{"role":"same", "size":1,"sha256":"a".repeat(64)}, {"role":"escape", "size":2,"sha256":"b".repeat(64)}]),
            "escape",
            "s\\u0061me",
            C::DuplicateRuntimeArtifact,
        ),
        (
            "/verification/definitions",
            json!([{"role":"same", "size":1,"sha256":"a".repeat(64)}, {"role":"escape", "size":2,"sha256":"b".repeat(64)}]),
            "escape",
            "\\u0073ame",
            C::DuplicateVerificationDefinition,
        ),
        (
            "/environment/disabled_dlls",
            json!(["mshtml", "escape"]),
            "escape",
            "m\\u0073html",
            C::DuplicateDll,
        ),
    ] {
        let input = String::from_utf8(edit(pointer, value))
            .unwrap()
            .replace(needle, replacement);
        assert!(codes(input.as_bytes()).iter().any(|e| e.0 == expected));
    }
    for pointer in [
        "/runtime/artifacts/0/role",
        "/verification/definitions/0/role",
    ] {
        assert_eq!(
            codes(&edit(pointer, json!("SAME")))[0].0,
            C::IdentifierInvalid
        );
    }
    assert_eq!(
        codes(&edit(
            "/environment/disabled_dlls",
            json!(["mshtml", "MSHTML"])
        ))[0]
            .0,
        C::DllInvalid
    );
    for input in [
        r#"{"a/b":1,"a\/b":2}"#,
        r#"{"\ud83d\ude00":0,"😀":0}"#,
        r#"{"x":{"x":{"k":0,"\u006b":1}}}"#,
    ] {
        one(input.as_bytes(), C::FieldDuplicate);
    }
}

#[test]
fn path_grammar_edges_and_device_case_variants() {
    let bad = [
        "drive_cx/app.exe",
        "drive_c./app.exe",
        "drive_c /app.exe",
        "Drive_c/app.exe",
        "drive_c",
        " drive_c/app.exe",
        "drive_c/a/.. /app",
        "drive_c/a/ . /app",
        "drive_c/aux .txt",
        "drive_c/con..txt",
        "drive_c/COM¹.exe",
        "drive_c/LPT²",
        "drive_c/COM9.txt",
        "drive_c/LpT1.exe",
        "drive_c/a\u{7f}",
        "drive_c/a\t",
        "drive_c/a:b",
        "drive_c//server/file",
        "//server/share/file",
        "drive_c/\\server",
        "drive_c/a. ",
    ];
    for path in bad {
        assert_eq!(
            codes(&edit("/entry_point/path", json!(path))),
            [(C::PathUnsafe, "$.entry_point.path".to_owned())],
            "{path:?}"
        );
    }
    for stem in ["CON", "PrN", "aUx", "Nul", "cOm1", "LpT9"] {
        for suffix in ["", ".exe", " .txt", "..txt"] {
            assert_eq!(
                codes(&edit(
                    "/entry_point/path",
                    json!(format!("drive_c/{stem}{suffix}"))
                ))[0]
                    .0,
                C::PathUnsafe
            );
        }
    }
    for path in [
        "drive_c/com0",
        "drive_c/lpt0",
        "drive_c/com10.exe",
        "drive_c/com01.exe",
        "drive_c/.hidden/app",
        "drive_c/a..b",
        "drive_c/COMx",
        "drive_c/No Such Directory/a",
    ] {
        assert_eq!(
            parse_spec(&edit("/entry_point/path", json!(path)))
                .unwrap()
                .entry_point()
                .path()
                .as_str(),
            path
        );
    }
}

#[test]
fn sibling_errors_survive_parent_and_constructor_failures() {
    let mut root: Value = serde_json::from_str(NOTES).unwrap();
    root["application"]["id"] = json!("!");
    root["application"]["source"] = json!({"size":0,"sha256":"!","architecture":"arm64"});
    root["runtime"] = json!({"family":"proton","artifacts":null});
    root["environment"] =
        json!({"windows_architecture":"win32","prefix":false,"disabled_dlls":null});
    root["entry_point"] = json!({"path":"!","sha256":"!"});
    root["verification"]["definitions"] = json!([]);
    let expected = [
        (C::IdentifierInvalid, "$.application.id"),
        (C::SizeInvalid, "$.application.source.size"),
        (C::DigestInvalid, "$.application.source.sha256"),
        (C::ValueUnsupported, "$.application.source.architecture"),
        (C::ValueUnsupported, "$.runtime.family"),
        (C::FieldType, "$.runtime.artifacts"),
        (C::ValueUnsupported, "$.environment.windows_architecture"),
        (C::FieldType, "$.environment.prefix"),
        (C::FieldType, "$.environment.disabled_dlls"),
        (C::PathUnsafe, "$.entry_point.path"),
        (C::DigestInvalid, "$.entry_point.sha256"),
        (C::CollectionLimit, "$.verification.definitions"),
    ]
    .map(|(c, l)| (c, l.to_owned()));
    assert_eq!(codes(&serde_json::to_vec(&root).unwrap()), expected);
    root["application"] = json!(false);
    let errors = codes(&serde_json::to_vec(&root).unwrap());
    assert_eq!(errors[0], (C::FieldType, "$.application".to_owned()));
    assert_eq!(&errors[1..], &expected[4..]);
}

#[test]
fn diagnostics_never_echo_private_values_and_cap_output() {
    for n in [1, 80, 256, 2048, 50000] {
        let marker = "PRIVATE_SENTINEL".repeat(n / 16 + 1);
        for input in [
            format!("{{\"{marker}\":\"{marker}\"}}"),
            format!("{{\"schema\":\"{marker}\"}}"),
            format!("{{\"{marker}\":"),
        ] {
            let error = parse_spec(input.as_bytes()).unwrap_err();
            let output = format!("{error} {error:?}");
            assert!(!output.contains("PRIVATE_SENTINEL"));
            assert!(output.len() < 20000);
        }
    }
    let errors = codes(&edit(
        "/runtime/artifacts",
        json!(
            (0..16)
                .map(|_| json!({"role":"!","size":0,"sha256":"!","label":""}))
                .collect::<Vec<_>>()
        ),
    ));
    assert_eq!(errors.len(), 64);
    assert_eq!(errors[63], (C::ErrorLimit, "$".to_owned()));
}

fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn independent_seeded_properties_and_platform_fingerprint() {
    const SEED: u64 = 0x0021_2026_0908_51a7;
    let mut rng = SEED;
    let mut transcript = Sha256::new();
    let mut valid = 0;
    let mut rejected = 0;
    for index in 0..20000 {
        let bytes = match index % 5 {
            0 => {
                // Multiple insert/delete/replace mutations, both real and synthetic fixtures.
                let mut input = if next(&mut rng) & 1 == 0 {
                    A0.as_bytes()
                } else {
                    NOTES.as_bytes()
                }
                .to_vec();
                for _ in 0..(next(&mut rng) % 8 + 1) {
                    let pos = (next(&mut rng) % input.len() as u64) as usize;
                    match next(&mut rng) % 3 {
                        0 => {
                            input.remove(pos);
                        }
                        1 => input.insert(pos, next(&mut rng) as u8),
                        _ => input[pos] = next(&mut rng) as u8,
                    }
                }
                input
            }
            1 => {
                let length = (next(&mut rng) % 2049) as usize;
                (0..length).map(|_| next(&mut rng) as u8).collect()
            }
            2 => {
                // Independent valid model generation within unchanged Wine scope.
                edit(
                    "/application/id",
                    json!(format!("app-{:x}", next(&mut rng))),
                )
            }
            3 => {
                // Grammar-generated escaped duplicates, nested at random accepted depth.
                let depth = (next(&mut rng) % 8) as usize;
                let key = (b'a' + (next(&mut rng) % 26) as u8) as char;
                format!(
                    "{}{{\"{key}\":null,\"\\u{:04x}\":false}}{}",
                    "[".repeat(depth),
                    key as u32,
                    "]".repeat(depth)
                )
                .into_bytes()
            }
            _ => {
                let width = (next(&mut rng) % 19) as usize;
                let depth = (next(&mut rng) % 12) as usize;
                format!(
                    "{}[{}]{}",
                    "[".repeat(depth),
                    vec!["0"; width].join(","),
                    "]".repeat(depth)
                )
                .into_bytes()
            }
        };
        let outcome = std::panic::catch_unwind(|| parse_spec(&bytes));
        assert!(
            outcome.is_ok(),
            "seed={SEED:x} index={index} bytes={bytes:?}"
        );
        let outcome = outcome.unwrap();
        assert_eq!(parse_spec(&bytes), outcome, "seed={SEED:x} index={index}");
        if index % 5 == 2 {
            assert!(outcome.is_ok(), "seed={SEED:x} index={index}");
        }
        if index % 5 == 3 {
            one(&bytes, C::FieldDuplicate);
        }
        match &outcome {
            Ok(spec) => {
                valid += 1;
                assert_eq!(spec.clone(), *spec);
                assert_eq!(
                    spec.spec_sha256().as_str(),
                    format!("{:x}", Sha256::digest(&bytes))
                );
                assert!(!spec.runtime().artifacts().is_empty());
            }
            Err(errors) => {
                rejected += 1;
                assert!((1..=64).contains(&errors.as_slice().len()));
                assert!(format!("{errors} {errors:?}").len() < 20000);
            }
        }
        // Exact bytes + complete Debug of result (all typed values/errors) for platform comparison.
        transcript.update((bytes.len() as u64).to_le_bytes());
        transcript.update(&bytes);
        transcript.update(format!("{outcome:?}\n"));
    }
    for input in [A0, NOTES] {
        transcript.update(format!("{:?}\n", parse_spec(input.as_bytes()).unwrap()));
    }
    println!(
        "REVIEW_PROPERTIES={{\"seed\":\"{SEED:016x}\",\"cases\":20000,\"valid\":{valid},\"rejected\":{rejected},\"transcript_sha256\":\"{:x}\"}}",
        transcript.finalize()
    );
}
