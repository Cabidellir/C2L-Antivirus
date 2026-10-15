//! Command-line interface for C2L Antivirus.
#![forbid(unsafe_code)]

use c2l_scanner::scan_file;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Scan(PathBuf),
}

fn parse_command(args: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let command = args.next();

    match command.as_ref().and_then(|value| value.to_str()) {
        None if command.is_none() => Ok(Command::Help),
        Some("help" | "--help" | "-h") => {
            if args.next().is_some() {
                return Err("help does not accept additional arguments".to_string());
            }
            Ok(Command::Help)
        }
        Some("scan") => {
            let path = args
                .next()
                .ok_or_else(|| "missing file path for scan".to_string())?;

            if args.next().is_some() {
                return Err("scan accepts exactly one file path".to_string());
            }

            Ok(Command::Scan(PathBuf::from(path)))
        }
        _ => Err("unknown command".to_string()),
    }
}

fn print_usage() {
    println!("C2L Antivirus v{}", env!("CARGO_PKG_VERSION"));
    println!("Usage: c2l scan <FILE>");
    println!("       c2l --help");
    println!();
    println!("The scan command calculates a SHA-256 fingerprint only.");
    println!("Malware detection is not implemented yet.");
}

fn run() -> Result<(), String> {
    match parse_command(env::args_os().skip(1))? {
        Command::Help => {
            print_usage();
            Ok(())
        }
        Command::Scan(path) => {
            let result = scan_file(&path)
                .map_err(|error| format!("cannot scan '{}': {error}", path.display()))?;

            println!("C2L Antivirus v{}", env!("CARGO_PKG_VERSION"));
            println!("File: {}", result.path.display());
            println!("Size: {} bytes", result.size_bytes);
            println!("SHA-256: {}", result.sha256);
            println!("Status: fingerprint calculated; no malware detection performed");
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            eprintln!("Usage: c2l scan <FILE>");
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
            Ok(Command::Scan(PathBuf::from("sample.exe")))
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
}
