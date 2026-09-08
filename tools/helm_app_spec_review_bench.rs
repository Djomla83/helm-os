// Development-only measurement of the exact sha2 build shared with helm-evidence.
use sha2::{Digest, Sha256};
use std::{hint::black_box, path::Path, time::Instant};

fn samples(mut f: impl FnMut(), iterations: usize) -> Vec<f64> {
    for _ in 0..10 {
        f();
    }
    (0..5)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..iterations {
                f();
            }
            start.elapsed().as_nanos() as f64 / iterations as f64
        })
        .collect()
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    for folder in &args {
        let report = helm_evidence::verify(Path::new(folder));
        let json = serde_json::to_vec(&report).unwrap();
        let times = samples(
            || {
                black_box(helm_evidence::verify(black_box(Path::new(folder))));
            },
            50,
        );
        println!(
            "{}",
            serde_json::json!({"kind":"verify", "fixture":Path::new(folder).file_name().unwrap().to_str().unwrap(), "report_sha256":format!("{:x}",Sha256::digest(&json)), "report":report, "iterations":50, "ns_per_call":times})
        );
    }
    for size in [65536, 4 * 1024 * 1024] {
        let bytes: Vec<_> = (0..size).map(|i| (i % 251) as u8).collect();
        let times = samples(
            || {
                black_box(Sha256::digest(black_box(&bytes)));
            },
            16,
        );
        println!(
            "{}",
            serde_json::json!({"kind":"hash", "bytes":size,"sha256":format!("{:x}",Sha256::digest(&bytes)), "iterations":16, "ns_per_call":times})
        );
    }
}
