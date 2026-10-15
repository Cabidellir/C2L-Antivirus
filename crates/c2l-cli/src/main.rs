//! Command-line interface for C2L Antivirus.
#![forbid(unsafe_code)]

use c2l_detection::{Verdict, classify};
use c2l_indicators::IndicatorDb;
use c2l_scanner::{FileOutcome, ScanResult, scan_directory, scan_file};
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

const MAX_DIRECTORY_FILES: usize = 1000;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Scan {
        path: PathBuf,
        indicators: Option<PathBuf>,
    },
    ScanDir {
        path: PathBuf,
        indicators: Option<PathBuf>,
    },
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(Command::Help);
    };

    let action = command
        .to_str()
        .ok_or_else(|| "command must be valid Unicode".to_string())?;

    if matches!(action, "help" | "--help" | "-h") {
        return if args.next().is_none() {
            Ok(Command::Help)
        } else {
            Err("help does not accept additional arguments".to_string())
        };
    }

    if !matches!(action, "scan" | "scan-dir") {
        return Err(format!("unknown command: {action}"));
    }

    let path = args
        .next()
        .ok_or_else(|| "missing file or directory path".to_string())?;
    let mut indicators = None;

    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--indicators") if indicators.is_none() => {
                indicators = Some(
                    args.next()
                        .ok_or_else(|| "missing path after --indicators".to_string())?
                        .into(),
                );
            }
            _ => return Err("unexpected argument (expected --indicators <FILE>)".to_string()),
        }
    }

    let path = PathBuf::from(path);

    if action == "scan" {
        Ok(Command::Scan { path, indicators })
    } else {
        Ok(Command::ScanDir { path, indicators })
    }
}

fn print_usage() {
    println!("C2L Antivirus v{}", env!("CARGO_PKG_VERSION"));
    println!("Usage: c2l scan <FILE> [--indicators <FILE>]");
    println!("       c2l scan-dir <DIRECTORY> [--indicators <FILE>]");
    println!("       c2l --help");
    println!();
    println!(
        "Scanning is read-only. Directory scans are limited to {MAX_DIRECTORY_FILES} entries."
    );
    println!("Indicator matches are not definitive malware diagnoses.");
    println!("A missing match is not evidence that a file is safe.");
}

fn load_database(path: Option<PathBuf>) -> Result<Option<IndicatorDb>, String> {
    path.map(|path| {
        IndicatorDb::load(&path)
            .map_err(|error| format!("cannot load indicators '{}': {error}", path.display()))
    })
    .transpose()
}

fn show_result(result: &ScanResult, database: Option<&IndicatorDb>) -> bool {
    println!("File: {}", result.path.display());
    println!("Size: {} bytes", result.size_bytes);
    println!("SHA-256: {}", result.sha256);

    match classify(&result.sha256, database) {
        Verdict::NotChecked => {
            println!("Indicator: NOT CHECKED (no database supplied)");
            false
        }
        Verdict::NotListed => {
            println!("Indicator: NOT LISTED (not a safety guarantee)");
            false
        }
        Verdict::Listed(name) => {
            println!("Indicator: MATCH ({name})");
            true
        }
    }
}

fn run() -> Result<(), String> {
    match parse_command(env::args_os().skip(1))? {
        Command::Help => {
            print_usage();
            Ok(())
        }
        Command::Scan { path, indicators } => {
            let database = load_database(indicators)?;
            let result = scan_file(&path)
                .map_err(|error| format!("cannot scan '{}': {error}", path.display()))?;

            println!("C2L Antivirus v{}", env!("CARGO_PKG_VERSION"));
            show_result(&result, database.as_ref());
            println!("Notice: results are based solely on SHA-256 fingerprint matching.");
            Ok(())
        }
        Command::ScanDir { path, indicators } => {
            let database = load_database(indicators)?;
            let report = scan_directory(&path, MAX_DIRECTORY_FILES)
                .map_err(|error| format!("cannot scan directory '{}': {error}", path.display()))?;

            println!("C2L Antivirus v{}", env!("CARGO_PKG_VERSION"));
            println!("Directory: {}", path.display());

            let mut scanned = 0usize;
            let mut matches = 0usize;
            let mut failures = 0usize;

            for outcome in report.outcomes {
                match outcome {
                    FileOutcome::Scanned(result) => {
                        scanned += 1;
                        if show_result(&result, database.as_ref()) {
                            matches += 1;
                        }
                    }
                    FileOutcome::Failed { path, message } => {
                        failures += 1;
                        eprintln!("Could not scan '{}': {message}", path.display());
                    }
                }
            }

            println!("Summary: {scanned} scanned, {matches} indicator matches, {failures} errors");

            if report.truncated {
                println!("Limit reached: scan stopped after {MAX_DIRECTORY_FILES} entries.");
            }

            println!("Notice: unmatched files are not verified as safe.");

            if failures > 0 {
                Err(format!("{failures} entries could not be scanned"))
            } else {
                Ok(())
            }
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(|value| OsString::from(*value)).collect()
    }

    #[test]
    fn no_arguments_shows_help() {
        assert_eq!(parse_command(args(&[])), Ok(Command::Help));
    }

    #[test]
    fn scan_accepts_one_path() {
        assert_eq!(
            parse_command(args(&["scan", "sample.exe"])),
            Ok(Command::Scan {
                path: PathBuf::from("sample.exe"),
                indicators: None,
            })
        );
    }

    #[test]
    fn scan_accepts_database() {
        assert_eq!(
            parse_command(args(&["scan", "sample.exe", "--indicators", "db.csv"])),
            Ok(Command::Scan {
                path: PathBuf::from("sample.exe"),
                indicators: Some(PathBuf::from("db.csv")),
            })
        );
    }

    #[test]
    fn scans_directory_command() {
        assert_eq!(
            parse_command(args(&["scan-dir", "C:\\temp"])),
            Ok(Command::ScanDir {
                path: PathBuf::from("C:\\temp"),
                indicators: None,
            })
        );
    }

    #[test]
    fn scan_rejects_missing_path() {
        assert!(parse_command(args(&["scan"])).is_err());
    }

    #[test]
    fn scan_rejects_extra_arguments() {
        assert!(parse_command(args(&["scan", "first", "second"])).is_err());
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(parse_command(args(&["delete"])).is_err());
    }

    #[test]
    fn rejects_missing_database_path() {
        assert!(parse_command(args(&["scan", "file", "--indicators"])).is_err());
    }
}
