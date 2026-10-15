//! Cryptographic hashing utilities for C2L Antivirus.

#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Incremental SHA-256 state, reusable by desktop scanners and Android streams.
pub struct Sha256State {
    hasher: Sha256,
}

impl Default for Sha256State {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256State {
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    pub fn finish(self) -> String {
        format!("{:x}", self.hasher.finalize())
    }
}

/// Calculates the SHA-256 hash of a file, reading in 64 KiB blocks.
pub fn sha256_file(path: impl AsRef<Path>) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256State::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn incremental_matches_known_sha256() {
        let mut state = Sha256State::new();
        state.update(b"a");
        state.update(b"bc");
        assert_eq!(
            state.finish(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_known_value() {
        let mut reader = Cursor::new(b"abc");
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 1024];

        loop {
            let count = reader.read(&mut buffer).unwrap();

            if count == 0 {
                break;
            }

            hasher.update(&buffer[..count]);
        }

        assert_eq!(
            format!("{:x}", hasher.finalize()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
