//! File scanning utilities for C2L Antivirus.
#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug)]
pub enum FileOutcome {
    Scanned(ScanResult),
    Failed { path: PathBuf, message: String },
}

#[derive(Debug)]
pub struct DirectoryScan {
    pub outcomes: Vec<FileOutcome>,
    /// True when the configured file limit was reached.
    pub truncated: bool,
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

/// Recursively scans a directory, skipping symbolic links to avoid traversal loops.
///
/// Individual unreadable entries become failures instead of aborting the whole scan.
/// No file is modified or quarantined.
pub fn scan_directory(path: impl AsRef<Path>, max_files: usize) -> io::Result<DirectoryScan> {
    let root = path.as_ref();

    if max_files == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "max_files must be greater than zero",
        ));
    }

    if !fs::metadata(root)?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The specified path is not a directory",
        ));
    }

    let mut directories = vec![root.to_path_buf()];
    let mut outcomes = Vec::new();

    while let Some(directory) = directories.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if directory == root => return Err(error),
            Err(error) => {
                outcomes.push(FileOutcome::Failed {
                    path: directory,
                    message: error.to_string(),
                });
                continue;
            }
        };

        for entry in entries {
            if outcomes.len() >= max_files {
                return Ok(DirectoryScan {
                    outcomes,
                    truncated: true,
                });
            }

            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    match entry.file_type() {
                        Ok(file_type) if file_type.is_symlink() => {}
                        Ok(file_type) if file_type.is_dir() => directories.push(path),
                        Ok(file_type) if file_type.is_file() => match scan_file(&path) {
                            Ok(result) => outcomes.push(FileOutcome::Scanned(result)),
                            Err(error) => outcomes.push(FileOutcome::Failed {
                                path,
                                message: error.to_string(),
                            }),
                        },
                        Ok(_) => {}
                        Err(error) => outcomes.push(FileOutcome::Failed {
                            path,
                            message: error.to_string(),
                        }),
                    }
                }
                Err(error) => outcomes.push(FileOutcome::Failed {
                    path: directory.clone(),
                    message: error.to_string(),
                }),
            }
        }
    }

    Ok(DirectoryScan {
        outcomes,
        truncated: false,
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
        let error =
            scan_file(env!("CARGO_MANIFEST_DIR")).expect_err("a directory must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn reports_missing_file() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("nonexistent_c2l_scanner_test_file_1ac0e9a7");
        let error = scan_file(path).expect_err("a missing file must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn scans_directory_with_limit() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let report = scan_directory(root, 1).unwrap();

        assert_eq!(report.outcomes.len(), 1);
        assert!(report.truncated);
    }

    #[test]
    fn rejects_zero_limit() {
        let error = scan_directory(env!("CARGO_MANIFEST_DIR"), 0).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
