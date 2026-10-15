
//! Cryptographic hashing utilities for C2L Antivirus.

#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Calculates the SHA-256 hash of a file, reading in 64 KiB blocks.
pub fn sha256_file(path: impl AsRef<Path>) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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
