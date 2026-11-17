# Desktop Packaging, Multi-Platform Installers & In-App Auto-Updater Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide native installation packages (.msi, .dmg, .AppImage, .deb, .zip) across Windows, macOS, and Linux via `cargo-packager`, build a built-in in-app auto-updater querying GitHub Releases with graceful restart, and upgrade the CI/CD release pipeline.

**Architecture:** Extend `crates/sharkord-core` with a pure Rust SemVer and GitHub Releases updater engine (`updater.rs`). Add Pure Neutral Dark update notification and modal dialogs to `apps/desktop-gpui` (`updater_modal.rs`). Add unified packaging configuration (`packager.toml`) with desktop icons and integrate installer artifact generation into `.github/workflows/rust-ci.yml`.

**Tech Stack:** Rust 2021, GPUI, `cargo-packager`, SemVer, GitHub Releases REST API, Pure Neutral Dark design tokens (`#0a0b0e`, `#111216`, `#16171d`, `#1c1e24`, `#3b82f6`, `#22c55e`).

**Spec:** `docs/superpowers/specs/2026-11-17-desktop-packaging-and-updater-design.md`

## Global Constraints
- Zero Turkish text in code comments, test names, identifiers, commit messages, and documentation files. Direct communication with the user is in Turkish.
- Maintain 100% test pass rate across the workspace with zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
- Pure Neutral Dark palette: background base `#0a0b0e`, surface `#111216`, subsurface `#16171d`, elevated `#1c1e24`, accent `#3b82f6`, online `#22c55e`.
- Maintain backwards compatibility with all existing tests across `desktop-gpui`, `sharkord-core`, and monorepo packages.

## Review Focus
- SemVer comparison correctly handles `v` prefix, prerelease tags (e.g., `0.1.0-alpha.1` vs `0.1.0`), and patch version increments.
- Platform asset matching enforces deterministic precedence (`.msi` > `.zip` on Windows, `.dmg` > `.zip` on macOS, `.AppImage` > `.deb` on Linux).
- UI modal provides distinct states for `UpdateAvailable` ("Download & Install") and `ReadyToInstall` ("Restart & Install").
- Application does not perform abrupt or unprompted restarts during active voice or screen share sessions.
- `packager.toml` validates correctly and references valid icon and asset paths.

---

### Task 1: Core Updater Engine in `sharkord-core`

**Files:**
- Create: `crates/sharkord-core/src/updater.rs`
- Modify: `crates/sharkord-core/src/lib.rs`
- Test: `crates/sharkord-core/tests/updater_test.rs`

**Interfaces:**
- Consumes: `serde`, `semver` or custom parsing
- Produces: `ReleaseAsset`, `ReleaseInfo`, `UpdateStatus`, `parse_semver`, `is_newer_version`, `find_platform_asset`, `UpdaterClient`

- [ ] **Step 1: Write failing unit test in `crates/sharkord-core/tests/updater_test.rs`**

```rust
use sharkord_core::updater::{
    find_platform_asset, is_newer_version, parse_semver, ReleaseAsset, ReleaseInfo, UpdateStatus,
};

#[test]
fn test_semver_parsing_and_version_comparison() {
    assert_eq!(parse_semver("0.1.0"), Some((0, 1, 0)));
    assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_semver("v2.0.0-beta.1"), Some((2, 0, 0)));
    assert_eq!(parse_semver("invalid"), None);

    assert!(is_newer_version("0.1.0", "0.1.1"));
    assert!(is_newer_version("0.1.0", "v0.2.0"));
    assert!(is_newer_version("1.0.0", "2.0.0"));
    assert!(!is_newer_version("0.2.0", "0.1.9"));
    assert!(!is_newer_version("1.0.0", "1.0.0"));
}

#[test]
fn test_platform_asset_precedence_matching() {
    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "### Changelog\n- Added voice grid".to_string(),
        published_at: "2026-11-17T00:00:00Z".to_string(),
        assets: vec![
            ReleaseAsset {
                name: "sharkord-plus-windows-x64.zip".to_string(),
                size: 15_000_000,
                browser_download_url: "https://example.com/win.zip".to_string(),
                content_type: "application/zip".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-windows-x64.msi".to_string(),
                size: 16_000_000,
                browser_download_url: "https://example.com/win.msi".to_string(),
                content_type: "application/x-msi".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-macos-aarch64.dmg".to_string(),
                size: 18_000_000,
                browser_download_url: "https://example.com/mac.dmg".to_string(),
                content_type: "application/x-apple-diskimage".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-linux-x86_64.AppImage".to_string(),
                size: 20_000_000,
                browser_download_url: "https://example.com/linux.AppImage".to_string(),
                content_type: "application/x-executable".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-linux-amd64.deb".to_string(),
                size: 14_000_000,
                browser_download_url: "https://example.com/linux.deb".to_string(),
                content_type: "application/vnd.debian.binary-package".to_string(),
            },
        ],
    };

    // Windows preference: .msi over .zip
    let win_asset = find_platform_asset(&release, "windows", "x86_64").unwrap();
    assert_eq!(win_asset.name, "sharkord-plus-windows-x64.msi");

    // macOS preference: .dmg
    let mac_asset = find_platform_asset(&release, "macos", "aarch64").unwrap();
    assert_eq!(mac_asset.name, "sharkord-plus-macos-aarch64.dmg");

    // Linux preference: .AppImage over .deb
    let linux_asset = find_platform_asset(&release, "linux", "x86_64").unwrap();
    assert_eq!(linux_asset.name, "sharkord-plus-linux-x86_64.AppImage");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test updater_test`
Expected: FAIL with unresolved `updater` module in `sharkord_core`

- [ ] **Step 3: Implement `updater.rs` in `crates/sharkord-core/src/`**

Define `ReleaseAsset`, `ReleaseInfo`, `UpdateStatus` enums/structs with serde serialization. Implement `parse_semver(version_str: &str) -> Option<(u64, u64, u64)>`, `is_newer_version(current: &str, remote: &str) -> bool`, and `find_platform_asset(release: &ReleaseInfo, os: &str, arch: &str) -> Option<&ReleaseAsset>`. Export `pub mod updater;` in `crates/sharkord-core/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test updater_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add crates/sharkord-core
git commit -m "feat(core): implement core semver parsing, github release matching, and update state machine"
```

---

### Task 2: Desktop In-App Updater Modal & State in GPUI

**Files:**
- Create: `apps/desktop-gpui/src/ui/updater_modal.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/updater_modal_test.rs`

**Interfaces:**
- Consumes: `ReleaseInfo`, `UpdateStatus` from `sharkord_core::updater`
- Produces: `UpdaterBadgeView`, `UpdaterModalView`, `AppState::update_status`, `AppState::toggle_update_modal`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/updater_modal_test.rs`**

```rust
use desktop_gpui::state::AppState;
use desktop_gpui::ui::updater_modal::{UpdaterBadgeView, UpdaterModalView};
use sharkord_core::updater::{ReleaseAsset, ReleaseInfo, UpdateStatus};

#[test]
fn test_updater_state_and_modal_view_transitions() {
    let mut state = AppState::new();
    assert_eq!(state.update_status, UpdateStatus::Idle);
    assert!(!state.is_update_modal_open);

    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "### What's Changed\n- Screen sharing\n- Discord parity polish".to_string(),
        published_at: "2026-11-17T00:00:00Z".to_string(),
        assets: vec![],
    };

    state.update_status = UpdateStatus::UpdateAvailable(release.clone());
    state.toggle_update_modal();
    assert!(state.is_update_modal_open);

    let modal = UpdaterModalView::new(&state.update_status);
    assert_eq!(modal.action_button_label(), "Download & Install");

    let downloading_status = UpdateStatus::Downloading {
        percent: 0.65,
        downloaded_bytes: 13_000_000,
        total_bytes: 20_000_000,
    };
    let downloading_modal = UpdaterModalView::new(&downloading_status);
    assert_eq!(downloading_modal.progress_percent(), Some(0.65));

    let ready_status = UpdateStatus::ReadyToInstall {
        installer_path: std::path::PathBuf::from("/tmp/installer.msi"),
        version: "0.2.0".to_string(),
    };
    let ready_modal = UpdaterModalView::new(&ready_status);
    assert_eq!(ready_modal.action_button_label(), "Restart & Install");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test updater_modal_test`
Expected: FAIL with unresolved module `updater_modal`

- [ ] **Step 3: Implement `updater_modal.rs` and wire into `AppState` & `WorkspaceView`**

Implement `UpdaterBadgeView` (title bar indicator pill) and `UpdaterModalView` (centered 520px Pure Neutral Dark modal overlay with header, markdown preview, progress bar, and dynamic action buttons). Add `pub update_status: UpdateStatus` and `pub is_update_modal_open: bool` to `AppState`. Export `updater_modal` in `apps/desktop-gpui/src/ui/mod.rs` and render modal overlay conditionally in `WorkspaceView::render`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test updater_modal_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement in-app update notification badge and updater modal dialog"
```

---

### Task 3: Packaging Configuration & Desktop Assets (`packager.toml`)

**Files:**
- Create: `apps/desktop-gpui/packager.toml`
- Create: `apps/desktop-gpui/assets/icons/icon.png` (high-res base icon)
- Create: `apps/desktop-gpui/assets/icons/icon.ico`
- Create: `apps/desktop-gpui/assets/icons/icon.icns`
- Create: `apps/desktop-gpui/assets/icons/128x128.png`
- Test: `apps/desktop-gpui/tests/packager_config_test.rs`

**Interfaces:**
- Produces: `apps/desktop-gpui/packager.toml`, multi-format icon assets
- Consumes: `cargo-packager` format schema

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/packager_config_test.rs`**

```rust
use std::fs;
use std::path::Path;

#[test]
fn test_packager_configuration_file_validity() {
    let config_path = Path::new("packager.toml");
    let full_path = if config_path.exists() {
        config_path.to_path_buf()
    } else {
        Path::new("apps/desktop-gpui/packager.toml").to_path_buf()
    };

    assert!(full_path.exists(), "packager.toml must exist");
    let content = fs::read_to_string(&full_path).expect("Failed to read packager.toml");

    // Must define product name and bundle identifier
    assert!(content.contains("product_name = \"Sharkord Plus\""));
    assert!(content.contains("identifier = \"com.homelessavatar.sharkord\""));

    // Must define icons
    assert!(content.contains("icons = ["));

    // Verify icon assets exist on disk
    let icon_dir = full_path.parent().unwrap().join("assets").join("icons");
    assert!(icon_dir.join("icon.ico").exists(), "icon.ico must exist");
    assert!(icon_dir.join("icon.icns").exists(), "icon.icns must exist");
    assert!(icon_dir.join("128x128.png").exists(), "128x128.png must exist");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test packager_config_test`
Expected: FAIL with missing `packager.toml`

- [ ] **Step 3: Create `packager.toml` and generate desktop icon assets**

Create `apps/desktop-gpui/packager.toml` specifying package metadata, Windows formats (`msi`, `zip`), macOS formats (`dmg`, `app`), and Linux formats (`appimage`, `deb`). Generate valid application icons in `apps/desktop-gpui/assets/icons/` (`icon.ico`, `icon.icns`, `128x128.png`).

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test packager_config_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui/packager.toml apps/desktop-gpui/assets/icons apps/desktop-gpui/tests/packager_config_test.rs
git commit -m "feat(desktop): add cargo-packager configuration and cross-platform desktop icons"
```

---

### Task 4: Upgrade Consolidated Release Pipeline in `.github/workflows/rust-ci.yml`

**Files:**
- Modify: `.github/workflows/rust-ci.yml`

**Interfaces:**
- Produces: automated package builds on `windows-latest`, `macos-latest`, `ubuntu-24.04` and GitHub Release asset publishing.

- [ ] **Step 1: Check existing `rust-ci.yml` syntax and job structure**

Read `.github/workflows/rust-ci.yml` and review `check-and-test`, `build-macos`, and `publish-release` jobs.

- [ ] **Step 2: Upgrade jobs in `.github/workflows/rust-ci.yml` with `cargo-packager`**

In `check-and-test` (Windows & Linux) and `build-macos` (macOS), install `cargo-packager`, run `cargo packager --release` to generate `.msi`, `.zip`, `.AppImage`, `.deb`, and `.dmg` packages, and upload them as release artifacts. In `publish-release`, collect all installer packages and publish them via `softprops/action-gh-release@v3`.

- [ ] **Step 3: Verify workflow YAML syntax and consistency**

Run validation check on `.github/workflows/rust-ci.yml` ensuring valid YAML parsing and action version consistency.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/rust-ci.yml
git commit -m "ci: integrate multi-platform cargo-packager installers into release pipeline"
```

---

### Task 5: End-to-End Test Suite, Verification & Roadmap Update

**Files:**
- Modify: `ROADMAP.md`

- [ ] **Step 1: Run complete cargo test suite across all workspace crates**
Run: `& "$HOME\.cargo\bin\cargo.exe" test --all`
Expected: 100% pass rate.

- [ ] **Step 2: Run formatting and clippy checks**
Run: `& "$HOME\.cargo\bin\cargo.exe" fmt --check`
Run: `& "$HOME\.cargo\bin\cargo.exe" clippy --all-targets -- -D warnings`
Expected: 0 warnings.

- [ ] **Step 3: Run monorepo type checks and linting (`bun run magic`)**
Run: `& "$HOME\.bun\bin\bun.exe" run magic`
Expected: Clean pass across all 7 packages.

- [ ] **Step 4: Update `ROADMAP.md` marking Phase 2 complete and commit**
Mark Phase 2: Desktop Packaging, Installers & Auto-Updater ✅ (Completed) with detailed items.
Commit message: `docs: mark Phase 2 desktop packaging and auto-updater complete in roadmap`
