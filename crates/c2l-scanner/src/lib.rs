
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
