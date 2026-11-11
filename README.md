# Windhawk Windows 10 Notes & Tweaks Exporter

A modern, fancy, and standalone Rust desktop application tailored specifically for **Windows 10** (builds 19041 through 21999). It integrates an interactive **Live Windows 10 Tweak Engine** with a full-featured, RFC 4180-compliant **Notes Exporter**.

> [!NOTE]
> **Reverse Engineering Analysis Context**:
> During comprehensive inspection of Windhawk v1.7.3 and 2.0.0-alpha.8, no "Export notes" or `notes.csv` feature was found upstream. This implementation realizes the minimal, assumed design specification (`id`, `title`, `body`, `created_at`, `updated_at` exported as RFC 4180 CSV with UTF-8 BOM) and extends it with live, practical Windows 10 system customizations.

---

## ✨ Features

### 1. 🎨 Modern Fancy UI
* **Sleek Dark Theme**: Styled with a modern charcoal/zinc palette, rounded card borders, clean typography, and interactive status badges.
* **Instant Search & Filter**: Real-time filtering across titles, descriptions, and categories as you type.
* **Category Navigation**:
  * 📋 **All Notes**: Unified view of all system tweaks and user notes.
  * ⚙ **Windows 10 Tweaks**: Curated list of live system tweaks.
  * 🗔 **Taskbar & Shell**: Taskbar button labeling, volume scroll, and shell tweaks.
  * 📁 **File Explorer**: File extensions, hidden folders, and view customizations.
  * 🎨 **Visual Themes**: Windows 10 dark/light theme options for apps and taskbar.
  * ➕ **New Custom Note**: Create and archive your own tweaks or mod notes.
* **Quick Action Controls**:
  * **Export Notes (CSV)**: 1-click export adhering strictly to RFC 4180 with UTF-8 BOM.
  * **Open Export Folder**: Directly opens `<Documents>\Windhawk\` in Windows File Explorer.
  * **Copy CSV to Clipboard**: Instantly copies the full CSV representation to the Windows clipboard.

### 2. ⚡ Live Windows 10 System Tweak Engine
Directly inspects and toggles real Windows 10 system registry keys, with live status indicators and instant shell notification (`SHChangeNotify` & `WM_SETTINGCHANGE`):
* **Never Combine Taskbar Buttons (Show Labels)**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarGlomLevel`
  * Values: `2` (Never combine / show labels) vs `0` (Always combine).
* **Show File Name Extensions in Explorer**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\HideFileExt`
  * Values: `0` (Show extensions like `.exe`, `.rs`, `.csv`) vs `1` (Hide extensions).
* **Show Hidden Files and Folders**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\Hidden`
  * Values: `1` (Show hidden) vs `2` (Hide hidden).
* **Windows 10 Apps Dark Theme**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme`
  * Values: `0` (Dark theme) vs `1` (Light theme).
* **Windows 10 Taskbar & Shell Dark Theme**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\SystemUsesLightTheme`
  * Values: `0` (Dark theme) vs `1` (Light theme).
* **Disable Bing Web Search in Start Menu**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Search\BingSearchEnabled`
  * Values: `0` (Fast local search only) vs `1` (Cloud web suggestions enabled).
* **Show Seconds in Taskbar Clock**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\ShowSecondsInSystemClock`
  * Values: `1` (Show real-time seconds `HH:MM:SS`) vs `0` (Hide seconds).
* **Use Small Taskbar Buttons**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\TaskbarSmallIcons`
  * Values: `1` (Compact taskbar height) vs `0` (Standard size).
* **Show 'This PC' Desktop Icon**:
  * Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel\{20D04FE0-3AEA-1069-A2D8-08002B30309D}`
  * Values: `0` (Show icon on desktop) vs `1` (Hide icon).

### 🎨 Windows 10 Taskbar & Accent Color Studio
* **Any Custom 24-bit TrueColor RGB on Taskbar & Shell**:
  * Bypasses the restricted Windows 10 Settings preset color grid to unlock full 16.7-million color RGB freedom.
  * Direct synchronization across `HKCU\Software\Microsoft\Windows\DWM\AccentColor` (ABGR), `ColorizationColor` (ARGB), `Personalize\ColorPrevalence`, and `Explorer\Accent\AccentPalette` (32-byte 8-shade palette).
  * Interactive native Color Picker, RGB integer sliders (0–255), Hex code copy, and 10 curated 1-click presets (Cyberpunk Neon, Electric Violet, Matrix Emerald, Windows Cyan, Classic Blue, Midnight Navy, Sunset Crimson, Solar Gold, Stealth Slate, Pastel Lilac).
  * Instant live broadcast via `WM_SETTINGCHANGE` (`ImmersiveColorSet`, `TraySettings`, `ShellState`, `Environment`, `Policy`) and `SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST)`.

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

The production GUI binary in `crates/gui` is built on **`egui` via `eframe`** with native Windows Shell integration (`windows-sys`).

**Why egui?**
1. **Rich, Modern Aesthetic**: Provides sleek dark styling, responsive live search filtering, animated cards, and smooth user interactions.
2. **Interactive Tweak Dashboard**: Allows users to inspect real Windows 10 registry values and apply them on-the-fly with immediate UI feedback.
3. **High Portability & Reliability**: Runs on hardware-accelerated OpenGL / Glow or software rasterization with zero runtime external DLL dependencies.
4. **Instant Startup & Low Resource Usage**: Clean native binary with lightning-fast launch time and minimal memory consumption.

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
    │   │   ├── tweaks.rs            # Windows 10 Live Tweak Registry Engine
    │   │   ├── exporter.rs          # RFC 4180 CSV serializer with UTF-8 BOM
    │   │   ├── paths.rs             # Known Folders and default paths
    │   │   ├── os_guard.rs          # RtlGetVersion FFI and mockable OS validator
    │   │   └── error.rs             # Typed error hierarchy (thiserror)
    │   └── tests/
    │       ├── golden_csv_tests.rs  # Golden output comparison & empty export
    │       ├── quoting_escaping_tests.rs # RFC 4180 quoting, newlines, UTF-8
    │       ├── path_tests.rs        # Directory creation and path resolution
    │       ├── os_guard_tests.rs    # Mockable build validator tests
    │       └── tweaks_tests.rs      # Windows 10 tweak registry & export tests
    └── gui/                         # Modern Fancy GUI executable
        ├── Cargo.toml
        ├── build.rs                 # Build-time target guard
        └── src/
            ├── main.rs              # App entrypoint, logger, OS guard
            └── ui.rs                # Modern egui view, cards, live actions
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
  | `id` | Integer (u64) | Unique note / tweak identifier |
  | `title` | Text | Short title of the tweak / note |
  | `body` | Text | Detailed content of the note (including live registry status) |
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

# Execute all unit and integration tests (17 tests)
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
