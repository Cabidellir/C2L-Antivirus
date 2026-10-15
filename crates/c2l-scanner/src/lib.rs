//! File scanning utilities for C2L Antivirus.
#![forbid(unsafe_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct ScanResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug)]
pub enum FileOutcome {
    Scanned(ScanResult),
    Failed { path: PathBuf, error: io::Error },
}

#[derive(Debug)]
pub struct DirectoryScan {
    pub outcomes: Vec<FileOutcome>,
    /// True when the configured file limit was reached.
    pub truncated: bool,
    pub cancelled: bool,
}

/// Calculates the SHA-256 fingerprint and obtains file metadata.
///
/// A fingerprint is not a malware verdict.
pub fn scan_file(path: impl AsRef<Path>) -> io::Result<ScanResult> {
    scan_file_cancellable(path, &AtomicBool::new(false))
}

pub fn scan_file_cancellable(
    path: impl AsRef<Path>,
    cancel: &AtomicBool,
) -> io::Result<ScanResult> {
    let path = path.as_ref();
    if cancel.load(Ordering::Relaxed) {
        return Err(io::Error::new(io::ErrorKind::Interrupted, "Scan cancelled"));
    }
    if !fs::metadata(path)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The specified path is not a regular file",
        ));
    }
    let file = fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The specified path is not a regular file",
        ));
    }
    let sha256 = c2l_hashing::sha256_reader_cancellable(&file, || cancel.load(Ordering::Relaxed))?;
    verify_snapshot(&metadata, &file.metadata()?)?;
    Ok(ScanResult {
        path: path.to_path_buf(),
        size_bytes: metadata.len(),
        sha256,
    })
}

/// Best-effort mutation detection on the same opened file, not an atomic snapshot.
fn verify_snapshot(before: &fs::Metadata, after: &fs::Metadata) -> io::Result<()> {
    if before.len() != after.len() || before.modified()? != after.modified()? {
        return Err(io::Error::other(FileChanged));
    }
    Ok(())
}

#[derive(Debug)]
pub struct FileChanged;
impl std::fmt::Display for FileChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("File changed during scanning; fingerprint discarded")
    }
}
impl std::error::Error for FileChanged {}

/// Compatibility API for CLI callers. Individual failures remain in the report.
pub fn scan_directory(path: impl AsRef<Path>, max_files: usize) -> io::Result<DirectoryScan> {
    scan_directory_controlled(path, max_files, &AtomicBool::new(false), |_| {})
}

/// Depth-first traversal with cooperative cancellation and completed-entry progress.
/// Keeps at most 256 directory iterators instead of accumulating every subdirectory.
/// The limit counts successful files and failures, never skipped symbolic links.
pub fn scan_directory_controlled(
    path: impl AsRef<Path>,
    max_files: usize,
    cancel: &AtomicBool,
    mut progress: impl FnMut(usize),
) -> io::Result<DirectoryScan> {
    let mut outcomes = Vec::new();
    let summary = scan_directory_streaming(path, max_files, cancel, |outcome| {
        outcomes.push(outcome);
        progress(outcomes.len());
        Ok(())
    })?;
    Ok(DirectoryScan {
        outcomes,
        truncated: summary.truncated,
        cancelled: summary.cancelled,
    })
}

#[derive(Debug, Default)]
pub struct DirectorySummary {
    pub processed: usize,
    pub truncated: bool,
    pub cancelled: bool,
}

/// Emits one outcome at a time without retaining results. A sink error stops traversal
/// immediately and is propagated; callers must preserve already written partial results.
/// The caller owns storage, progress reporting and any resource budget.
pub fn scan_directory_streaming(
    path: impl AsRef<Path>,
    max_files: usize,
    cancel: &AtomicBool,
    mut emit: impl FnMut(FileOutcome) -> io::Result<()>,
) -> io::Result<DirectorySummary> {
    if max_files == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "max_files must be greater than zero",
        ));
    }
    let mut report = DirectorySummary::default();
    if cancel.load(Ordering::Relaxed) {
        report.cancelled = true;
        return Ok(report);
    }
    let root = path.as_ref();
    let mut stack = vec![(root.to_path_buf(), fs::read_dir(root)?)];
    while !stack.is_empty() {
        if cancel.load(Ordering::Relaxed) {
            report.cancelled = true;
            break;
        }
        let (directory, entries) = stack.last_mut().unwrap();
        let Some(entry) = entries.next() else {
            stack.pop();
            continue;
        };
        let parent = directory.clone();
        // Only claim truncation if there is another entry requiring processing.
        let outcome = match entry {
            Ok(entry) => {
                let path = entry.path();
                match entry.file_type() {
                    Ok(kind) if kind.is_symlink() => continue,
                    Ok(kind) if kind.is_dir() => {
                        if report.processed >= max_files {
                            report.truncated = true;
                            break;
                        }
                        if stack.len() >= 256 {
                            FileOutcome::Failed {
                                path,
                                error: io::Error::other("Directory depth limit (256) reached"),
                            }
                        } else {
                            match fs::read_dir(&path) {
                                Ok(entries) => {
                                    stack.push((path, entries));
                                    continue;
                                }
                                Err(error) => FileOutcome::Failed { path, error },
                            }
                        }
                    }
                    Ok(kind) if kind.is_file() => {
                        if report.processed >= max_files {
                            report.truncated = true;
                            break;
                        }
                        match scan_file_cancellable(&path, cancel) {
                            Ok(result) => FileOutcome::Scanned(result),
                            Err(_) if cancel.load(Ordering::Relaxed) => {
                                report.cancelled = true;
                                break;
                            }
                            Err(error) => FileOutcome::Failed { path, error },
                        }
                    }
                    Ok(_) => continue,
                    Err(error) => FileOutcome::Failed { path, error },
                }
            }
            Err(error) => FileOutcome::Failed {
                path: parent,
                error,
            },
        };
        if report.processed >= max_files {
            report.truncated = true;
            break;
        }
        emit(outcome)?;
        report.processed += 1;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_more_than_one_thousand_and_preserves_progress_limits_and_cancellation() {
        let temp = fs::canonicalize(std::env::temp_dir()).unwrap();
        let root = temp.join(format!(
            "c2l-large-scan-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        for i in 0..1_505 {
            fs::write(root.join(format!("file-{i}.txt")), b"abc").unwrap();
        }
        let cancel = AtomicBool::new(false);
        let mut progress = 0;
        let report = scan_directory_controlled(&root, 10_000, &cancel, |count| {
            assert_eq!(count, progress + 1);
            progress = count;
        })
        .unwrap();
        assert_eq!(report.outcomes.len(), 1_505);
        assert_eq!(progress, 1_505);
        assert!(!report.truncated && !report.cancelled);
        assert!(
            report
                .outcomes
                .iter()
                .all(|outcome| matches!(outcome, FileOutcome::Scanned(_)))
        );
        let limited = scan_directory(&root, 1_000).unwrap();
        assert_eq!(limited.outcomes.len(), 1_000);
        assert!(limited.truncated);
        let exact = scan_directory(&root, 1_505).unwrap();
        assert!(!exact.truncated);
        let stopped = scan_directory_controlled(&root, 10_000, &cancel, |count| {
            if count == 3 {
                cancel.store(true, Ordering::Relaxed);
            }
        })
        .unwrap();
        assert_eq!(stopped.outcomes.len(), 3);
        assert!(stopped.cancelled);
        assert!(!stopped.truncated);
        let resolved = fs::canonicalize(&root).unwrap();
        assert_eq!(resolved.parent(), Some(temp.as_path()));
        fs::remove_dir_all(resolved).unwrap();
    }

    #[test]
    fn streaming_propagates_storage_failure() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut emitted = 0;
        let error = scan_directory_streaming(root, 100, &AtomicBool::new(false), |_| {
            emitted += 1;
            Err(io::Error::other("storage failed"))
        })
        .unwrap_err();
        assert_eq!(emitted, 1);
        assert_eq!(error.to_string(), "storage failed");
    }

    #[test]
    fn detects_file_changes_and_preserves_windows_sharing_error() {
        let path = std::env::temp_dir().join(format!(
            "c2l-mutation-{}-{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let before = file.metadata().unwrap();
        file.set_len(10).unwrap();
        let error = verify_snapshot(&before, &file.metadata().unwrap()).unwrap_err();
        assert!(error.get_ref().unwrap().is::<FileChanged>());
        assert!(verify_snapshot(&file.metadata().unwrap(), &file.metadata().unwrap()).is_ok());
        drop(file);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let locked = fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(&path)
                .unwrap();
            assert_eq!(scan_file(&path).unwrap_err().raw_os_error(), Some(32));
            drop(locked);
        }
        fs::remove_file(path).unwrap();
    }

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
