//! Local SHA-256 indicator database. Matching a hash is not, by itself, a malware verdict.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// A strict text database: `<64 hex SHA-256 characters>,<identifier>` per line.
/// Blank lines and lines beginning with # are ignored.
#[derive(Debug, Default)]
pub struct IndicatorDb {
    entries: HashMap<String, String>,
}

#[derive(Debug)]
pub enum IndicatorError {
    Io(io::Error),
    InvalidLine(usize),
    DuplicateHash(usize),
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::InvalidLine(line) => write!(f, "invalid indicator at line {line}"),
            Self::DuplicateHash(line) => write!(f, "duplicate SHA-256 at line {line}"),
        }
    }
}

impl std::error::Error for IndicatorError {}

impl From<io::Error> for IndicatorError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl IndicatorDb {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, IndicatorError> {
        let file = File::open(path)?;
        Self::parse(BufReader::new(file))
    }

    pub fn parse(reader: impl BufRead) -> Result<Self, IndicatorError> {
        let mut entries = HashMap::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let (hash, name) = trimmed
                .split_once(',')
                .ok_or(IndicatorError::InvalidLine(index + 1))?;
            let hash = hash.trim();
            let name = name.trim();

            if hash.len() != 64
                || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
                || name.is_empty()
                || name.len() > 64
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
            {
                return Err(IndicatorError::InvalidLine(index + 1));
            }

            if entries
                .insert(hash.to_ascii_lowercase(), name.to_owned())
                .is_some()
            {
                return Err(IndicatorError::DuplicateHash(index + 1));
            }
        }

        Ok(Self { entries })
    }

    pub fn find(&self, sha256: &str) -> Option<&str> {
        self.entries
            .get(&sha256.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn parses_and_matches_case_insensitively() {
        let db = IndicatorDb::parse(Cursor::new(format!(
            "# comment\n\n{},DEMO_ONLY\n",
            ABC.to_ascii_uppercase()
        )))
        .unwrap();

        assert_eq!(db.len(), 1);
        assert_eq!(db.find(ABC), Some("DEMO_ONLY"));
        assert_eq!(db.find(&"0".repeat(64)), None);
    }

    #[test]
    fn rejects_invalid_sha256() {
        assert!(matches!(
            IndicatorDb::parse(Cursor::new("abc,TEST\n")),
            Err(IndicatorError::InvalidLine(1))
        ));
    }

    #[test]
    fn rejects_duplicate_hash() {
        let input = format!("{ABC},FIRST\n{ABC},SECOND\n");
        assert!(matches!(
            IndicatorDb::parse(Cursor::new(input)),
            Err(IndicatorError::DuplicateHash(2))
        ));
    }

    #[test]
    fn rejects_unsafe_label() {
        let input = format!("{ABC},bad label\n");
        assert!(matches!(
            IndicatorDb::parse(Cursor::new(input)),
            Err(IndicatorError::InvalidLine(1))
        ));
    }
}
