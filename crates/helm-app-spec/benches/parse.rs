//! Development measurement only: timer/output are outside the product library.
use helm_app_spec::{MAX_INPUT_BYTES, parse_spec};
use std::{hint::black_box, time::Instant};

fn main() {
    let oversized = vec![b' '; MAX_INPUT_BYTES + 1];
    let cases: [(&str, &[u8], bool); 4] = [
        ("a0", include_bytes!("../tests/fixtures/a0-7zip.json"), true),
        (
            "synthetic",
            include_bytes!("../tests/fixtures/synthetic-notes.json"),
            true,
        ),
        ("malformed", b"{\"schema\":", false),
        ("oversized", &oversized, false),
    ];
    for (name, bytes, valid) in cases {
        assert_eq!(parse_spec(bytes).is_ok(), valid);
        for _ in 0..1000 {
            let _ = black_box(parse_spec(black_box(bytes)));
        }
        let samples: Vec<_> = (0..5)
            .map(|_| {
                let start = Instant::now();
                for _ in 0..10000 {
                    let _ = black_box(parse_spec(black_box(bytes)));
                }
                start.elapsed().as_nanos() as f64 / 10000.0
            })
            .collect();
        println!(
            "{{\"case\":\"{name}\",\"bytes\":{},\"iterations_per_sample\":10000,\"ns_per_parse_samples\":{:?}}}",
            bytes.len(),
            samples
        );
    }
}
