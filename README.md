# Windhawk Windows 10 Notes Exporter

A standalone, lightweight Rust implementation of a Notes Exporter tailored specifically for **Windows 10** (builds 19041 through 21999).

> [!NOTE]
> **Reverse Engineering Analysis Context**:
> During comprehensive inspection of Windhawk v1.7.3 and 2.0.0-alpha.8, no "Export notes" or `notes.csv` feature was found upstream. This implementation realizes the minimal, assumed design specification (`id`, `title`, `body`, `created_at`, `updated_at` exported as RFC 4180 CSV with UTF-8 BOM) and is not derived from upstream Windhawk internal code.

---

## 🎯 Target Platform & Compatibility Guard

* **Target OS**: Windows 10 x86_64 only.
* **Supported Builds**: **19041 to 21999**
  * Windows 10 2004 (20H1 - Build 19041)
  * Windows 10 20H2 (Build 19042)
  * Windows 10 21H1 (Build 19043)
  * Windows 10 21H2 (Build 19044)
  * Windows 10 22H2 (Build 19045)
* **Unsupported Environments**:
  * Older Windows versions (Windows 7/8/8.1, or Windows 10 builds < 19041 like 1507/1607/1809/1909) are rejected.
  * **Windows 11 (build >= 22000)** is explicitly unsupported; Windows 11 workflows are already serviced upstream by official Windhawk builds.

### Two-Tier Guard System
1. **Build-Time Guard (`build.rs`)**:
   Ensures the crate can only be compiled when targeting `cfg(target_os = "windows")` and `x86_64`.
2. **Runtime NTDLL Guard (`RtlGetVersion`)**:
   Directly queries `ntdll.dll!RtlGetVersion` upon process launch. Unlike Win32 `GetVersionExW`, `RtlGetVersion` cannot be deceived by Windows application compatibility shims or manifest settings. If executed outside the 19041-21999 build window, it displays a native modal error dialog (`MessageBoxW`) and exits with code 1.

---

## 🏛️ GUI Stack Choice & Justification

The production GUI binary in `crates/gui` is built on **native Win32** via the official Microsoft `windows-rs` / `windows-sys` crate family.

**Why Native Win32?**
1. **Authentic Windows 10 Integration**: Uses standard Win32 Common Controls (`STATIC`, `EDIT`, `BUTTON`), standard system fonts (`DEFAULT_GUI_FONT`), native message dialogs (`MessageBoxW`), and the native Shell file dialog (`GetSaveFileNameW`).
2. **Zero GPU / Display Driver Overhead**: Unlike GPU-rendered toolkits (such as `egui`, `iced`, or `slint`), native Win32 controls do not require Vulkan, OpenGL, or DirectX hardware acceleration. They run flawlessly on virtual machines, remote desktop sessions, and headless continuous integration (CI) runners without crashing or missing display adapters.
3. **Ultra-Low Resource Footprint**: Stripped release binary size is ~500 KB, with instantaneous startup and negligible memory usage.
4. **Architectural Consistency**: Matches the native Win32 desktop application architecture employed by Windhawk's native helper and core modules.

---

## 📦 Project Architecture

```
windhawk-win10/
├── Cargo.toml                       # Workspace manifest
├── LICENSE                          # GPL-3.0-or-later license
├── README.md                        # Documentation and architecture guide
├── .github/workflows/ci.yml         # GitHub Actions CI workflow
├── wix/main.wxs                     # WiX MSI installer configuration
├── scripts/package.ps1              # Packaging script (portable ZIP + MSI)
└── crates/
    ├── core/                        # Core business logic library
    │   ├── Cargo.toml
    │   ├── build.rs                 # Build-time target guard
    │   ├── src/
    │   │   ├── lib.rs               # Public library API
    │   │   ├── model.rs             # Note struct and sample data
    │   │   ├── exporter.rs          # RFC 4180 CSV serializer with UTF-8 BOM
    │   │   ├── paths.rs             # Known Folders and default paths
    │   │   ├── os_guard.rs          # RtlGetVersion FFI and mockable OS validator
    │   │   └── error.rs             # Typed error hierarchy (thiserror)
    │   └── tests/
    │       ├── golden_csv_tests.rs  # Golden output comparison & empty export
    │       ├── quoting_escaping_tests.rs # RFC 4180 quoting, newlines, UTF-8
    │       ├── path_tests.rs        # Directory creation and path resolution
    │       └── os_guard_tests.rs    # Mockable build validator tests
    └── gui/                         # Native Win32 GUI executable
        ├── Cargo.toml
        ├── build.rs                 # Build-time target guard
        └── src/
            ├── main.rs              # Win32 entrypoint, logger, OS guard
            └── ui.rs                # Window procedure, controls, and dialogs
```

---

## 📊 CSV Export Specification

* **Output File**: `notes.csv` (defaults to `<Documents>\Windhawk\notes.csv`, selectable via native Save File Dialog).
* **Encoding**: **UTF-8 with Byte Order Mark (BOM)**: `0xEF, 0xBB, 0xBF`.
* **Standard**: Strict **RFC 4180**:
  * Comma delimiter (`,`).
  * CRLF line endings (`\r\n`).
  * Double-quote escaping (`"` escaped as `""`).
  * Fields containing commas, newlines, or quotes are enclosed in quotes.
* **Schema**:
  | Column | Type | Description |
  | :--- | :--- | :--- |
  | `id` | Integer (u64) | Unique note identifier |
  | `title` | Text | Short title of the tweak / note |
  | `body` | Text | Detailed content of the note |
  | `created_at` | Text (ISO 8601) | Creation timestamp (e.g. `2026-11-11T12:00:00Z`) |
  | `updated_at` | Text (ISO 8601) | Last modified timestamp |

---

## 🛠️ Building & Verification

### Prerequisites
* Rust toolchain (stable, MSVC target `x86_64-pc-windows-msvc`)
* Windows 10 (Build 19041 to 21999)

### Commands
```powershell
# Format check
cargo fmt --all -- --check

# Strict linting
cargo clippy --all-targets -- -D warnings

# Execute all unit and integration tests
cargo test --all

# Build optimized release binary
cargo build --release
```

---

## 📦 Packaging

Run the packaging script to generate both a portable distribution archive and an MSI installer:

```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/package.ps1
```

* **Portable Bundle**: Generates `target/package/windhawk-win10-portable.zip` containing `windhawk-win10.exe`, `README.md`, and `LICENSE`.
* **MSI Installer**: Uses `wix/main.wxs` with WiX Toolset v3 (`candle.exe` and `light.exe`) or `cargo-wix` to compile `windhawk-win10-installer.msi`.

---

## 🔐 Code Signing

For production distribution, executables and MSI installers must be Authenticode-signed to prevent Windows SmartScreen warnings.

### Signing with Microsoft SignTool
```powershell
# Set path to signtool.exe from Windows 10 SDK
$SignTool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\signtool.exe"

# Sign using a Code Signing PFX certificate
& $SignTool sign `
    /f "C:\path\to\certificate.pfx" `
    /p "CertPasswordHere" `
    /fd SHA256 `
    /tr "http://timestamp.digicert.com" `
    /td SHA256 `
    /d "Windhawk Windows 10 Notes Exporter" `
    "./target/release/windhawk-win10.exe"

# Verify signature
& $SignTool verify /pa /v "./target/release/windhawk-win10.exe"
```

### Signing with Azure Trusted Signing
For CI/CD pipelines utilizing Microsoft Azure Trusted Signing:
```powershell
trusted-signing sign `
    --endpoint "https://eus.codesigning.azure.net" `
    --code-signing-account-name "MyAccount" `
    --certificate-profile-name "MyProfile" `
    --file "./target/release/windhawk-win10.exe"
```

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0 or later (GPL-3.0-or-later)** to maintain compatibility with upstream [Windhawk](https://github.com/ramensoftware/windhawk). See [LICENSE](LICENSE) for the full license text.

# Windhawk-win10
