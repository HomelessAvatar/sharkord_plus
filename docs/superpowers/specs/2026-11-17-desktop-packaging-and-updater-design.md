# Architecture & Design: Desktop Packaging, Multi-Platform Installers & In-App Auto-Updater

## 1. Intent & Problem Statement

Users of Sharkord Plus currently run the client by compiling from source via `cargo run -p desktop-gpui`. For general adoption, testing, and production distribution, the desktop client requires:
1. Native, standardized installation packages for the three major operating systems:
   - **Windows:** `.msi` installer and portable `.zip`.
   - **macOS:** `.dmg` disk image and compressed `.app.zip` (supporting both Apple Silicon `aarch64` and Intel `x86_64`).
   - **Linux:** `.AppImage` (distro-agnostic standalone executable) and `.deb` (Debian/Ubuntu package).
2. A built-in, non-intrusive **In-App Auto-Updater** that periodically checks GitHub Releases for new published versions, alerts the user with release notes, and allows seamless background downloading and user-confirmed graceful restart & installation.
3. Upgraded **CI/CD Release Workflow** in `.github/workflows/rust-ci.yml` that compiles, packages with `cargo-packager`, and attaches all platform installer artifacts to GitHub Releases on version tag creation (`v*.*.*`).

---

## 2. Goals, Non-Goals & Anti-Goals

### Goals
- **Single Packaging Source of Truth:** Use `cargo-packager` with a centralized `apps/desktop-gpui/packager.toml` to configure metadata, icons, bundle identifiers, and OS-specific artifact formats across Windows, macOS, and Linux.
- **Cross-Platform Release Artifacts:**
  - Windows: MSI installer with desktop shortcut and uninstaller, plus portable ZIP.
  - macOS: Drag-to-Applications `.dmg` and compressed `Sharkord Plus.app.zip`.
  - Linux: Executable `.AppImage` with embedded desktop file and icon, plus `.deb` for apt-based distributions.
- **Robust Updater Engine in `crates/sharkord-core`:**
  - SemVer version comparison (`is_newer_version(current, latest)`).
  - GitHub Releases API parser (`ReleaseInfo`, `ReleaseAsset`).
  - Target-aware asset matcher (`find_platform_asset`) with deterministic format precedence:
    - Windows: `.msi` > `.zip`
    - macOS: `.dmg` > `.zip` > `.tar.gz`
    - Linux: `.AppImage` > `.deb` > `.tar.gz`
  - Explicit State Machine: `Idle` -> `Checking` -> `UpdateAvailable(ReleaseInfo)` | `UpToDate` -> `Downloading { progress }` -> `ReadyToInstall { installer_path }` -> `Error(String)`.
- **Pure Neutral Dark GPUI Update Dialog (`apps/desktop-gpui`):**
  - Subtle update badge indicator when a new release is available.
  - Modal dialog with release version, changelog markdown preview, progress bar, and "Download & Install" button.
  - Safe transition to `ReadyToInstall`: button becomes "Restart & Install" which prompts user, launches installer detached, and gracefully shuts down the application (`cx.quit()` / `exit(0)`) to avoid executable file locking.
- **CI/CD Integration in `.github/workflows/rust-ci.yml`:**
  - Single consolidated workflow (no split/conflicting release workflows).
  - Builds Windows, Linux, and macOS packages via `cargo-packager` and attaches `.msi`, `.dmg`, `.AppImage`, `.deb`, and `.zip` files to the GitHub Release.

### Non-Goals
- Paid enterprise EV certificates / Apple Developer ID notarization at this stage (packages will be built standard/unsigned, with standard gatekeeper bypass instructions provided in release notes).
- Centralized proprietary update servers (all update metadata is pulled directly from GitHub Releases API).

### Anti-Goals
- Background daemon services running continuously while the application is closed.
- Forced auto-restarts or disruption of ongoing voice or screen share sessions.

---

## 3. Global Constraints
- **Language Policy:** Zero Turkish text across all code comments, test names, identifiers, commit messages, and documentation files.
- **Quality Gates:** 100% test pass rate across the workspace with zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
- **Design Tokens:** Strict adherence to Pure Neutral Dark palette (`#0a0b0e`, `#111216`, `#16171d`, `#1c1e24`, `#3b82f6`, `#22c55e`).
- **Monorepo Compatibility:** No regressions or breaking changes to existing web client, server, or plugin packages.

---

## 4. Evaluated Approaches & Selection

### Approach A: Unified `cargo-packager` + Custom Core Updater Engine (Selected)
- **Mechanism:** `cargo-packager` handles all OS target packaging from one configuration file. A dedicated, lightweight `updater` module in `crates/sharkord-core` manages GitHub API interaction, semver comparison, and platform asset matching, while `desktop-gpui` renders the UI dialog.
- **Pros:**
  - Single configuration file (`packager.toml`) eliminates multi-tool drift.
  - First-class support for Windows MSI/ZIP, macOS DMG/APP, and Linux AppImage/DEB.
  - Consolidates cleanly with existing `.github/workflows/rust-ci.yml`.
  - Lightweight, fully testable Rust unit tests.
- **Cons:** Requires `cargo-packager` CLI in CI build environments.

### Approach B: Disjoint Native Tools (`cargo-wix` + `cargo-bundle` + `cargo-deb`)
- **Mechanism:** Maintain individual config files (`wix/main.wxs`, `Cargo.toml` deb sections, etc.).
- **Pros:** Independent toolchains.
- **Cons:** High maintenance overhead, conflicting metadata formats, lack of unified CI scripting.

**Decision:** Approach A is selected for cohesion, maintainability, and standard Rust desktop ecosystem alignment.

---

## 5. Detailed Architecture & Components

### 5.1 Packaging Configuration (`apps/desktop-gpui/packager.toml`)
- **Package Info:**
  - Name: `Sharkord Plus`
  - Identifier: `com.homelessavatar.sharkord`
  - Version: Read dynamically from `Cargo.toml`
  - Authors: HomelessAvatar
  - Description: High-performance Discord-alternative chat, voice, and video client in Rust & GPUI.
- **Assets:**
  - Desktop Icons: `assets/icons/icon.ico` (Windows), `assets/icons/icon.icns` (macOS), `assets/icons/128x128.png` (Linux).
- **Target Formats:**
  - Windows: `["msi", "zip"]`
  - macOS: `["dmg", "app"]`
  - Linux: `["appimage", "deb"]`

### 5.2 Core Updater Engine (`crates/sharkord-core/src/updater.rs`)
- **Types & Enums:**
  - `ReleaseAsset`: name, size, browser_download_url, content_type.
  - `ReleaseInfo`: tag_name, version, name, body, published_at, assets: `Vec<ReleaseAsset>`.
  - `UpdateStatus`:
    - `Idle`
    - `Checking`
    - `UpdateAvailable(ReleaseInfo)`
    - `UpToDate`
    - `Downloading { percent: f32, downloaded_bytes: u64, total_bytes: u64 }`
    - `ReadyToInstall { installer_path: PathBuf, version: String }`
    - `Error(String)`
- **Core Functions & Precedence:**
  - `pub fn parse_semver(version_str: &str) -> Option<(u64, u64, u64)>`: strips leading `v`, handles standard `X.Y.Z`.
  - `pub fn is_newer_version(current: &str, remote: &str) -> bool`: returns true if remote > current.
  - `pub fn find_platform_asset<'a>(release: &'a ReleaseInfo, os: &str, arch: &str) -> Option<&'a ReleaseAsset>`:
    - Precedence:
      - Windows: `.msi` > `.zip` matching `x86_64` or `windows`.
      - macOS: `.dmg` > `.zip` > `.tar.gz` matching `aarch64` / `arm64` or `x86_64`.
      - Linux: `.AppImage` > `.deb` > `.tar.gz` matching `x86_64` or `amd64`.
  - `pub struct UpdaterClient`: manages GitHub release querying (`https://api.github.com/repos/HomelessAvatar/sharkord_plus/releases/latest`).

### 5.3 Desktop GPUI Updater UI (`apps/desktop-gpui/src/ui/updater_modal.rs`)
- **Components:**
  - `UpdaterBadge`: A subtle pill in the workspace title/settings bar showing `✨ Update Available (vX.Y.Z)`.
  - `UpdaterModalView`:
    - 520px wide centered overlay modal (`bg_elevated`, `border_hairline`).
    - Header: "Update Available" with current vs new version pills.
    - Scrollable changelog markdown preview (`bg_subsurface`, `rounded_lg`).
    - Download progress bar with live percentage and size when downloading.
    - Action buttons:
      - When `UpdateAvailable`: "Download & Install" (`bg_accent`), "Remind Me Later" (`bg_surface_4`), "Dismiss".
      - When `ReadyToInstall`: "Restart & Install" (`bg_accent`) which launches installer detached and exits process gracefully, avoiding Windows file locking issues.
- **State Integration:**
  - Integrated into `AppState` (`update_status: UpdateStatus`, `is_update_modal_open: bool`).
  - Wire modal into `WorkspaceView::render` overlay stack.

### 5.4 Consolidated CI/CD Workflow (`.github/workflows/rust-ci.yml`)
- Trigger: Tag push `v*` or `workflow_dispatch`.
- Steps:
  - Windows: builds with `cargo-packager` -> outputs `.msi` and `.zip`.
  - Linux: builds with `cargo-packager` -> outputs `.AppImage` and `.deb`.
  - macOS: builds with `cargo-packager` -> outputs `.dmg` and zips `.app`.
  - `publish-release`: gathers all generated packages and publishes them cleanly to GitHub Releases via `softprops/action-gh-release@v3`.

---

## 6. What's Left to the Builder
- Generating placeholder icon files (`icon.ico`, `icon.icns`, `128x128.png`) in `assets/icons/`.
- Platform-specific detached process spawning (`cmd.exe /c start` on Windows, `open` on macOS, `chmod +x && execute` on Linux).
