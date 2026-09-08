#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{path::Path, process::Command};

#[test]
fn hostile_objects_and_deterministic_open_races() -> Result<(), Box<dyn std::error::Error>> {
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/helm_evidence_linux_probe.py");
    let result = Command::new("python3")
        .arg(script)
        .args([
            "--binary",
            env!("CARGO_BIN_EXE_helm-evidence"),
            "--assert-safe",
        ])
        .output()?;
    assert!(
        result.status.success(),
        "Linux filesystem regression failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
