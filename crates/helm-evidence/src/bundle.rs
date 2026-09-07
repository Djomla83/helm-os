//! The only product filesystem access. Ambient authority is used once, for the caller's root.
use cap_std::{ambient_authority, fs::Dir};
use std::{
    io::{self, Read},
    path::Path,
};

pub(crate) const MANIFEST_LIMIT: u64 = 256 * 1024;
pub(crate) const FILE_LIMIT: u64 = 8 * 1024 * 1024;
pub(crate) const TOTAL_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ReadError {
    Missing,
    Unsafe,
    Io,
    TooLarge,
}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            Self::Missing
        } else {
            Self::Io
        }
    }
}

pub(crate) struct Bundle {
    dir: Dir,
    remaining: u64,
}

impl Bundle {
    pub(crate) fn open(root: &Path) -> Result<Self, ReadError> {
        Ok(Self {
            dir: Dir::open_ambient_dir(root, ambient_authority())?,
            remaining: TOTAL_LIMIT,
        })
    }

    pub(crate) fn read(&mut self, path: &str, limit: u64) -> Result<Vec<u8>, ReadError> {
        if !safe_path(path) {
            return Err(ReadError::Unsafe);
        }
        // Reject even contained symlinks/reparse points: the contract names exact files.
        // cap-std's handle-relative open also enforces containment if a link is swapped in.
        let mut prefix = String::new();
        for part in path.split('/') {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            let meta = self.dir.symlink_metadata(&prefix)?;
            if meta.file_type().is_symlink() {
                return Err(ReadError::Unsafe);
            }
            #[cfg(windows)]
            {
                use cap_std::fs::MetadataExt;
                if meta.file_attributes() & 0x400 != 0 {
                    return Err(ReadError::Unsafe);
                }
            }
            if prefix == path {
                if !meta.is_file() {
                    return Err(ReadError::Unsafe);
                }
            } else if !meta.is_dir() {
                return Err(ReadError::Unsafe);
            }
        }
        let file = self.dir.open(path)?;
        let meta = file.metadata()?;
        if !meta.is_file() {
            return Err(ReadError::Unsafe);
        }
        if self.remaining == 0 {
            return Err(ReadError::TooLarge);
        }
        let limit = limit.min(self.remaining);
        if meta.len() > limit {
            return Err(ReadError::TooLarge);
        }
        let mut bytes = Vec::new();
        let read = file.take(limit + 1).read_to_end(&mut bytes);
        // Charge even a partial/erroring read or a file that grew after metadata inspection.
        self.remaining = self.remaining.saturating_sub(bytes.len() as u64);
        read?;
        if bytes.len() as u64 > limit {
            return Err(ReadError::TooLarge);
        }
        Ok(bytes)
    }
}

/// Portable, deliberately narrow path spelling. Also used for experiment-relative destinations.
pub(crate) fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && path.split('/').count() <= 32
        && path.split('/').all(|part| {
            if part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' ']) {
                return false;
            }
            if !part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._- ".contains(&b))
            {
                return false;
            }
            let stem = part
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.as_bytes()[3].is_ascii_digit())
        })
}

pub(crate) fn identifier(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

pub(crate) fn digest_shape(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_portable_escape_and_device_spellings() {
        for path in [
            "../canary",
            "/canary",
            "a/../b",
            "a//b",
            "./a",
            "a\\b",
            "C:canary",
            "https://example.test/a",
            "NUL",
            "com1.txt",
            "LPT9",
            "COM¹",
            "a.",
            "a ",
            "a\0b",
        ] {
            assert!(!safe_path(path), "{path:?}");
        }
        assert!(safe_path("outputs/before-restart/workflow.zip"));
        assert!(safe_path("with spaces/output.json"));
    }

    #[test]
    fn identity_syntax_is_bounded() {
        assert!(identifier("W1-V1"));
        assert!(!identifier("\nprivate/path"));
        assert!(digest_shape(&"a".repeat(64)));
        assert!(!digest_shape(&"A".repeat(64)));
    }
}
