//! File scanning utilities for C2L Antivirus.

#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
}

/// Calculates the SHA-256 fingerprint and obtains file metadata.
///
/// A fingerprint is not a malware verdict.
pub fn scan_file(path: impl AsRef<Path>) -> io::Result<ScanResult> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)?;

    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The specified path is not a regular file",
        ));
    }

    let sha256 = c2l_hashing::sha256_file(path)?;

    Ok(ScanResult {
        path: path.to_path_buf(),
        size_bytes: metadata.len(),
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_a_real_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let result = scan_file(&path).expect("a source file should be readable");

        assert_eq!(result.path, path);
        assert!(result.size_bytes > 0);
        assert_eq!(result.sha256.len(), 64);
        assert!(result.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn rejects_a_directory() {
        let error = scan_file(env!("CARGO_MANIFEST_DIR"))
            .err()
            .expect("a directory must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn reports_missing_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("nonexistent_c2l_scanner_test_file_1ac0e9a7");
        let error = scan_file(path)
            .err()
            .expect("a missing file must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }
}
