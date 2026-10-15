# C2L Antivirus Android — prototype

This is an experimental Android application, **not a full antivirus**. It is
read-only, requests no broad storage permissions, does not upload selected
files, does not inspect installed apps, and does not run in the background.

## How it works

1. The user taps **Choose a file**.
2. Android's system file picker grants access to that selected document.
3. Kotlin reads it in 64 KiB chunks and passes each chunk through JNI to
   `c2l-hashing` (the same SHA-256 library used by the Windows scanner).
4. Rust produces a SHA-256 digest, then optionally compares it against the
   demonstration-only indicator file packaged into the APK.
5. The screen shows byte count, hash, demonstration indicator status, and an
   explicit disclaimer that absence of a hash match does not establish safety.

`C2L_DEMO_ONLY` is a synthetic indicator for the harmless, three-byte file
`abc`. It is not malware detection. There is no active protection, scanning
of installed apps, signature feed, or security verdict.

## Build with GitHub Actions (recommended)

Push a branch with the Android source or create a pull request. Open
[Actions](https://github.com/Cabidellir/C2L-Antivirus/actions) and find the
**Android APK** workflow. If it passes, open the run, scroll to **Artifacts**,
and download **c2l-android-debug-apk**. Its contents include
`app-debug.apk` (a **debug-signed** test package).

Debug APKs are not Play Store releases. Only install packages from a workflow
run you trust. For security, compare the run with the code in the repository.

## Run on an Android device

- Android 8.0 (API 26) or newer; APK contains `arm64-v8a` and `x86_64`
  native code. Most modern physical devices are ARM64.
- Transfer `app-debug.apk` to the phone and install it, allowing installation
  from your selected trusted source if prompted.
- Launch **C2L Antivirus (Prototype)** and choose a local file.
- To check the demo match, transfer `tests/fixtures/c2l-demo.txt` from the
  repository to the phone, select that file, and confirm the
  `MATCH (C2L_DEMO_ONLY)` result. The text file is harmless.

## Local development

Open the `android/` directory in Android Studio. The Android NDK and Rust
must be installed to produce native libraries. GitHub Actions provisions the
dependencies and produces the debug APK automatically.

The native Rust project lives under `android/native/`, intentionally outside
the desktop Cargo workspace. This isolates JNI dependencies from the desktop
lockfile. Before producing reproducible releases, commit a separately generated
native `Cargo.lock`, audit dependencies, and pin toolchain versions.

## Important limitations

- `content://` URIs are read by Android; Rust is not given unrestricted
  paths or private app storage access.
- A selected document might not remain readable after permissions are revoked.
- The current prototype has no cancellation button or progress display and
  the indicator database is purely synthetic.
- JNI handles are bounded and read-only; native scanner state lasts for one
  chosen file and is discarded afterward.
- Unmatched hashes are **not** safe/clean verdicts.
- Validate device performance, error handling, privacy, tests and app signing
  before any wider distribution.
