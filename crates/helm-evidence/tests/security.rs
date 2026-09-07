mod support;
use helm_evidence::{
    model::{CheckStatus, Verdict},
    verify,
};
use std::{fs, io::Write, path::Path, process::Command};
use support::*;

#[cfg(unix)]
use std::os::unix::fs::{symlink as symlink_dir, symlink as symlink_file};
#[cfg(windows)]
use std::os::windows::fs::{symlink_dir, symlink_file};

fn rejected(root: &Path) {
    let r = verify(root);
    assert_eq!(r.verdict, Verdict::Invalid, "{r:?}");
    assert!(r.checks.iter().any(|c| c.status == CheckStatus::Invalid));
}

#[test]
fn traversal_absolute_url_and_windows_device_paths_are_rejected() -> TestResult {
    let outside = Fixture::new()?;
    fs::write(outside.root.join("canary.txt"), b"synthetic outside canary")?;
    for path in [
        "../canary.txt",
        "/canary.txt",
        "a/../../canary.txt",
        "a\\..\\canary.txt",
        "C:\\synthetic\\canary.txt",
        "//server/share/canary",
        "https://example.test/canary",
        "file://canary",
        "data:canary",
        "NUL",
        "COM1.txt",
        "a.txt:stream",
        "a/./b",
    ] {
        let f = Fixture::synthetic()?;
        f.edit("bundle.json", |b| b["artifacts"][0]["path"] = path.into())?;
        rejected(&f.root);
    }
    assert_eq!(
        fs::read(outside.root.join("canary.txt"))?,
        b"synthetic outside canary"
    );
    Ok(())
}

#[test]
fn file_symlink_outside_cannot_supply_artifact() -> TestResult {
    let f = Fixture::synthetic()?;
    let outside = Fixture::new()?;
    // Correct bytes/hash outside the capability; existence/hash alone must not admit it.
    fs::rename(f.root.join("good.json"), outside.root.join("canary.json"))?;
    symlink_file(outside.root.join("canary.json"), f.root.join("good.json"))?;
    rejected(&f.root);
    Ok(())
}

#[test]
fn directory_symlink_outside_cannot_supply_artifact() -> TestResult {
    let f = Fixture::synthetic()?;
    let outside = Fixture::new()?;
    fs::copy(f.root.join("good.json"), outside.root.join("canary.json"))?;
    symlink_dir(&outside.root, f.root.join("escape"))?;
    f.edit("bundle.json", |b| {
        if let Some(list) = b["artifacts"].as_array_mut() {
            for a in list {
                if a["id"] == "good" {
                    a["path"] = "escape/canary.json".into();
                }
            }
        }
    })?;
    rejected(&f.root);
    Ok(())
}

#[test]
fn contained_and_dangling_symlinks_are_rejected() -> TestResult {
    for target in ["copy.json", "absent.json"] {
        let f = Fixture::synthetic()?;
        fs::rename(f.root.join("good.json"), f.root.join("copy.json"))?;
        symlink_file(target, f.root.join("good.json"))?;
        rejected(&f.root);
    }
    Ok(())
}

#[test]
fn manifest_symlink_outside_is_rejected() -> TestResult {
    let f = Fixture::synthetic()?;
    let outside = Fixture::new()?;
    fs::rename(f.root.join("bundle.json"), outside.root.join("bundle.json"))?;
    symlink_file(outside.root.join("bundle.json"), f.root.join("bundle.json"))?;
    rejected(&f.root);
    Ok(())
}

#[cfg(windows)]
#[test]
fn windows_junction_outside_is_rejected() -> TestResult {
    let f = Fixture::synthetic()?;
    let outside = Fixture::new()?;
    fs::copy(f.root.join("good.json"), outside.root.join("canary.json"))?;
    let result = Command::new("powershell.exe").args(["-NoProfile", "-NonInteractive", "-Command",
        "New-Item -ItemType Junction -Path $env:HELM_TEST_LINK -Target $env:HELM_TEST_TARGET -ErrorAction Stop | Out-Null"])
        .env("HELM_TEST_LINK", f.root.join("escape")).env("HELM_TEST_TARGET", &outside.root).output()?;
    assert!(result.status.success(), "Junction test setup failed");
    f.edit("bundle.json", |b| {
        if let Some(list) = b["artifacts"].as_array_mut() {
            for a in list {
                if a["id"] == "good" {
                    a["path"] = "escape/canary.json".into();
                }
            }
        }
    })?;
    rejected(&f.root);
    // Remove only the junction entry. This is not recursive deletion of its target.
    fs::remove_dir(f.root.join("escape"))?;
    assert!(outside.root.join("canary.json").exists());
    Ok(())
}

#[test]
fn directories_cannot_masquerade_as_files() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::remove_file(f.root.join("good.json"))?;
    fs::create_dir(f.root.join("good.json"))?;
    rejected(&f.root);
    Ok(())
}

#[test]
fn manifest_artifact_and_total_size_limits() -> TestResult {
    let f = Fixture::synthetic()?;
    fs::write(f.root.join("bundle.json"), vec![b' '; 256 * 1024 + 1])?;
    rejected(&f.root);
    let f = Fixture::synthetic()?;
    fs::OpenOptions::new()
        .write(true)
        .open(f.root.join("good.json"))?
        .set_len(8 * 1024 * 1024 + 1)?;
    rejected(&f.root);
    let f = Fixture::synthetic()?;
    let bytes = vec![0_u8; 8 * 1024 * 1024];
    let hash = sha256(&bytes);
    for i in 0..9 {
        fs::write(f.root.join(format!("large-{i}")), &bytes)?;
    }
    f.edit("bundle.json", |b| {
        if let Some(list) = b["artifacts"].as_array_mut() {
            for i in 0..9 { list.push(serde_json::json!({"id": format!("large-{i}"), "path": format!("large-{i}"), "sha256": hash})); }
        }
    })?;
    let r = verify(&f.root);
    assert_eq!(r.verdict, Verdict::Invalid);
    assert!(r.checks.iter().any(|c| c.code == "RESOURCE_LIMIT"));
    Ok(())
}

#[test]
fn diagnostics_do_not_echo_host_paths_or_bundle_text() -> TestResult {
    let f = Fixture::synthetic()?;
    let marker = "C:\\Users\\SYNTHETIC_PRIVATE_CANARY\\secret";
    f.edit("bundle.json", |b| {
        b["artifacts"][0]["id"] = marker.into();
        b["artifacts"][0]["path"] = marker.into();
    })?;
    let r = verify(&f.root);
    let mut human = Vec::new();
    helm_evidence::write_human(&mut human, &r)?;
    let mut json = serde_json::to_vec(&r)?;
    json.write_all(&human)?;
    assert!(!String::from_utf8(json)?.contains("SYNTHETIC_PRIVATE_CANARY"));
    let result = Command::new(env!("CARGO_BIN_EXE_helm-evidence"))
        .arg("verify")
        .arg(f.root.join("missing-private-name"))
        .arg("--json")
        .output()?;
    assert!(!String::from_utf8(result.stdout)?.contains("missing-private-name"));
    Ok(())
}

#[test]
fn opaque_shell_text_is_never_executed() -> TestResult {
    let f = Fixture::synthetic()?;
    // Auxiliary evidence can carry command text; only identity is inspected, never execution.
    fs::write(
        f.root.join("command.txt"),
        b"echo synthetic > SHOULD_NOT_EXIST",
    )?;
    let hash = sha256(&fs::read(f.root.join("command.txt"))?);
    f.edit("bundle.json", |b| {
        if let Some(list) = b["artifacts"].as_array_mut() {
            list.push(serde_json::json!({"id": "command", "path": "command.txt", "sha256": hash}));
        }
    })?;
    assert_eq!(verify(&f.root).verdict, Verdict::Complete);
    assert!(!f.root.join("SHOULD_NOT_EXIST").exists());
    Ok(())
}
