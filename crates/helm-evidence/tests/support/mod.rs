use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct Fixture {
    pub root: PathBuf,
}

impl Fixture {
    pub fn new() -> TestResult<Self> {
        let root = std::env::temp_dir().join(format!(
            "helm-evidence-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        Ok(Self { root })
    }

    pub fn synthetic() -> TestResult<Self> {
        let fixture = Self::new()?;
        copy_files(&fixture_path("synthetic"), &fixture.root)?;
        Ok(fixture)
    }

    pub fn json(&self, name: &str) -> TestResult<Value> {
        Ok(serde_json::from_slice(&fs::read(self.root.join(name))?)?)
    }

    pub fn write_json(&self, name: &str, value: &Value) -> TestResult {
        fs::write(self.root.join(name), serde_json::to_vec_pretty(value)?)?;
        Ok(())
    }

    pub fn edit(&self, name: &str, action: impl FnOnce(&mut Value)) -> TestResult {
        let mut value = self.json(name)?;
        action(&mut value);
        self.write_json(name, &value)?;
        if name != "bundle.json" {
            self.repin(name)?;
        }
        Ok(())
    }

    pub fn repin(&self, name: &str) -> TestResult {
        let hash = sha256(&fs::read(self.root.join(name))?);
        self.edit("bundle.json", |bundle| {
            if let Some(artifacts) = bundle["artifacts"].as_array_mut() {
                for artifact in artifacts {
                    if artifact["path"] == name {
                        artifact["sha256"] = hash.clone().into();
                    }
                }
            }
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Only the unique directory this test created, within the OS temp directory.
        if self.root.is_absolute()
            && self.root.parent() == Some(std::env::temp_dir().as_path())
            && self
                .root
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("helm-evidence-test-"))
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

pub fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn copy_files(from: &Path, to: &Path) -> TestResult {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let child = to.join(entry.file_name());
            fs::create_dir(&child)?;
            copy_files(&entry.path(), &child)?;
        } else if entry.file_type()?.is_file() {
            fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
