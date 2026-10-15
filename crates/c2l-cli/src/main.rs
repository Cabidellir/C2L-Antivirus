//! Initial command-line entry point for C2L Antivirus.
#![forbid(unsafe_code)]

fn main() {
    println!("C2L Antivirus v{}", env!("CARGO_PKG_VERSION"));
    println!("Development scaffold only; scanning is not implemented yet.");
}
