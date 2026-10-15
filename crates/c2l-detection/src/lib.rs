//! SHA-256 indicator matching. A negative match does not imply a safe file.
#![forbid(unsafe_code)]

use c2l_indicators::IndicatorDb;

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict<'a> {
    /// No indicator database was supplied.
    NotChecked,
    /// The database was checked, but the hash was absent.
    NotListed,
    /// The hash was present in the supplied database.
    Listed(&'a str),
}

pub fn classify<'a>(sha256: &str, database: Option<&'a IndicatorDb>) -> Verdict<'a> {
    match database {
        None => Verdict::NotChecked,
        Some(db) => match db.find(sha256) {
            Some(name) => Verdict::Listed(name),
            None => Verdict::NotListed,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    #[test]
    fn distinguishes_no_database_match_and_non_match() {
        let db = IndicatorDb::parse(Cursor::new(format!("{ABC},DEMO_ONLY\n"))).unwrap();

        assert_eq!(classify(ABC, None), Verdict::NotChecked);
        assert_eq!(classify(ABC, Some(&db)), Verdict::Listed("DEMO_ONLY"));
        assert_eq!(classify(&"0".repeat(64), Some(&db)), Verdict::NotListed);
    }
}
