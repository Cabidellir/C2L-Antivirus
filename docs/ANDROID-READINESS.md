# C2L Antivirus — Windows tests and Android readiness

## What the current prototype can do

- Read a file in blocks and calculate a SHA-256 fingerprint.
- Optionally compare that fingerprint with a **local** list of SHA-256 indicators.
- Recursively inspect a selected directory, skipping symbolic links and stopping after
  1,000 entries (including failed entries).
- Continue a directory scan after individual file read failures and display an error count.

**Important:** the scanner is read-only and does not quarantine, delete, repair or
automatically block anything. An indicator match is not conclusive evidence of
malware, and an absent match is not evidence of safety.

## Test on Windows

From the repository root in PowerShell:

```powershell
git pull
.\tools\windows-smoke.cmd
```

The smoke test runs workspace unit tests, checks a harmless demonstration file
against a demonstration-only indicator database, scans the actual Windows Notepad
executable, and recursively scans the repository's demonstration fixtures.

You can also run these individually:

```powershell
cargo run -p c2l-cli -- scan "C:\Windows\notepad.exe"
cargo run -p c2l-cli -- scan tests\fixtures\c2l-demo.txt --indicators tests\fixtures\demo-indicators.csv
cargo run -p c2l-cli -- scan-dir tests\fixtures --indicators tests\fixtures\demo-indicators.csv
```

`tests/fixtures/c2l-demo.txt` contains the harmless bytes `abc`. The database
entry `C2L_DEMO_ONLY` is **synthetic**, and must never be presented as a
malware finding. No actual malware sample is needed for these tests.

## Local indicators format

A UTF-8 text file with one entry per line:

```text
# comment
<64 lowercase or uppercase hexadecimal SHA-256 characters>,INDICATOR_NAME
```

Names must contain only ASCII letters, digits, underscore, dash, or dot (1–64
bytes). Duplicate hashes, invalid hashes, and malformed lines fail closed with
an error rather than silently ignoring entries. There is **no network feed**,
automatic signature update, or verification of indicator provenance yet.

## Android migration plan

### Portable Rust components

The following crates contain no Windows-specific API calls:

- `c2l-hashing`: streaming SHA-256
- `c2l-scanner`: path-based scanning of files the process can access
- `c2l-indicators`: local database parsing and lookup
- `c2l-detection`: matches an observed hash against indicators

CI checks that the four crates compile for the Android ARM64 target
(`aarch64-linux-android`). This is only a **compile check**. It is not an
Android APK, emulator test, or on-device scan.

### Android integration, in order

1. Expose a stable Rust library interface suitable for Android/JNI rather than
   attempting to run the Windows CLI as an Android app.
2. Create an Android app (Kotlin UI) with a file picker using the Storage Access
   Framework and explicit user consent. Android document `content://` URIs
   cannot be treated as ordinary filesystem paths: the Kotlin side must open
   the document descriptor or stream and pass its bytes/descriptor safely to
   the Rust engine.
3. Package the Rust native library for Android ABIs with the Android NDK,
   test in an emulator and on a physical device, and verify cancellation,
   permission revocation, battery use, and large-file behavior.
4. Add a curated and integrity-protected update channel for signatures and
   privacy-preserving reports, with clear provenance and false-positive
   handling.
5. Only then assess app scanning and background operation within Android's
   application sandbox, permission model, Play policies and device limits.

Android does **not** grant ordinary apps general access to every installed
application's private data or unrestricted background scanning. No claim of
system-wide device protection should be made until a supported, permitted
implementation and device testing establish that capability.

## Security and release gates

Before describing the product as an antivirus with protective coverage:

- Trustworthy and authenticated indicator updates.
- Robust malicious-input fuzz tests and realistic, authorized detection tests.
- Signed releases, update integrity, telemetry/privacy design, and a false-positive policy.
- Threat-model review and independent evaluation.
