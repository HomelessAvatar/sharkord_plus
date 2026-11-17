# Mobile Client (Android & iOS) & Unified Multi-Platform Packaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the desktop release asset upload pipeline so compiled binaries (`.exe`, `.zip`, `.msi`, `.AppImage`, `.deb`, `.dmg`) appear under GitHub Releases assets; build the cross-platform Flutter mobile client (`apps/mobile/`) with pure dark Discord parity UX, background voice, and in-app auto-updater for Android & iOS; and wire Android APK releases into the automated GitHub Releases pipeline.

**Architecture:**
- **Desktop CI:** Correct `packager.toml` (`[deb]` section), stage release artifacts into clean flat directories (`dist-windows/`, `dist-linux/`, `dist-macos/`), and recursively collect all built binaries into `dist-release/` for `action-gh-release`.
- **Mobile Client:** Flutter 3.x (Dart 3+) inside `apps/mobile/` with Riverpod reactive state, Pure Neutral Dark design system (`#0a0b0e`, `#111216`, `#16171d`, `#1c1e24`, `#3b82f6`, `#22c55e`), `flutter_webrtc` for media, and `crates/sharkord-mobile` for core business logic.
- **Mobile Updater:** SemVer release checker querying GitHub Releases API. On Android, downloads `.apk` with progress bar and launches package installer via platform Intent (`OpenFilex`). On iOS, displays update modal and redirects to download/TestFlight via `url_launcher`.
- **Packaging Pipeline:** GitHub Actions workflow compiles Android `.apk` via Flutter and packages all desktop and mobile deliverables into release assets.

**Language Policy:** Zero Turkish text across all code comments, test names, identifiers, commit messages, and documentation files on GitHub. Direct user conversation in Turkish.

---

## File Structure

```
.github/workflows/
└── rust-ci.yml                               # Updated desktop & mobile release pipeline

apps/desktop-gpui/
└── packager.toml                             # Fixed Debian table and packager settings

apps/mobile/
├── android/                                  # Android native host with permissions & FileProvider
│   └── app/src/main/AndroidManifest.xml
├── ios/                                      # iOS native host with background modes & audio
├── lib/
│   ├── main.dart                             # Flutter app entry point & Riverpod Scope
│   ├── core/
│   │   ├── theme/
│   │   │   ├── colors.dart                   # Pure Neutral Dark tokens (#0a0b0e, #111216, etc.)
│   │   │   └── theme.dart                    # ThemeData configuration
│   │   └── update/
│   │       ├── update_model.dart             # ReleaseInfo, SemVer parser, AssetMatcher
│   │       ├── update_service.dart           # GitHub Releases query & version checker
│   │       ├── android_updater.dart          # Streaming APK download & package installer
│   │       └── ios_updater.dart              # iOS redirect & TestFlight updater
│   ├── models/
│   │   ├── user.dart
│   │   ├── channel.dart
│   │   └── message.dart
│   ├── providers/
│   │   ├── auth_provider.dart
│   │   ├── channel_provider.dart
│   │   ├── message_provider.dart
│   │   ├── voice_provider.dart
│   │   └── update_provider.dart
│   ├── screens/
│   │   ├── login_screen.dart                 # Server URL & login credentials
│   │   └── workspace_screen.dart             # Three-pane drawer workspace
│   └── widgets/
│       ├── channel_drawer.dart               # Server rail + channel category tree
│       ├── member_drawer.dart                # Online/offline roster with status dots
│       ├── chat_canvas.dart                  # Virtualized message list + composer
│       ├── voice_bar.dart                    # Docked mini voice bar
│       ├── voice_stage_sheet.dart            # Modal voice stage & participant cards
│       └── update_dialog.dart                # Discord-grade update modal with progress bar
├── test/
│   ├── update_service_test.dart              # SemVer comparison and asset lookup tests
│   ├── theme_test.dart                       # Color token and typography tests
│   └── widget_test.dart                      # Drawer, chat, and voice bar widget tests
└── pubspec.yaml                              # Flutter dependencies & metadata

crates/sharkord-mobile/
├── Cargo.toml                                # Bridge crate linking sharkord-core
└── src/
    └── lib.rs                                # Mobile FFI API signatures
```

---

## Tasks

### Task 1: Fix Desktop CI/CD Release Packaging Pipeline & Verify Asset Staging

**Files:**
- Modify: `apps/desktop-gpui/packager.toml`
- Modify: `.github/workflows/rust-ci.yml`

- [x] **Step 1.1: Verify `apps/desktop-gpui/packager.toml` tables**
  Ensure `[deb]` table is used for `section = "net"` and invalid `[linux]` section is corrected.

- [x] **Step 1.2: Refactor artifact staging in `.github/workflows/rust-ci.yml`**
  In `check-and-test` (Windows), stage `desktop-gpui.exe` and zipped `sharkord-gpui-windows-x64.zip` directly into `dist-windows/`, and upload `dist-windows/*`.
  In `check-and-test` (Linux), stage `desktop-gpui` and `sharkord-gpui-linux-x64.tar.gz` directly into `dist-linux/`, and upload `dist-linux/*`.
  In `build-macos`, stage `desktop-gpui` and `sharkord-gpui-macos-arm64.zip` directly into `dist-macos/`, and upload `dist-macos/*`.

- [x] **Step 1.3: Update `Prepare release assets` in `publish-release`**
  Use recursive `find release-artifacts ... -exec cp {} dist-release/ \;` to guarantee all `.exe`, `.msi`, `.AppImage`, `.deb`, `.dmg`, `.zip`, `.tar.gz`, `.apk` are gathered into `dist-release/`.

- [x] **Step 1.4: Run local verification**
  Run `cargo check` and `cargo test` across the workspace.

- [x] **Step 1.5: Commit changes**
  Commit with message `fix(ci): stage desktop artifacts into dedicated directories for release assets`.

---

### Task 2: Scaffold Flutter Mobile Client (`apps/mobile/`) with Pure Neutral Dark Theme

**Files:**
- Create: `apps/mobile/pubspec.yaml`
- Create: `apps/mobile/lib/main.dart`
- Create: `apps/mobile/lib/core/theme/colors.dart`
- Create: `apps/mobile/lib/core/theme/theme.dart`
- Create: `apps/mobile/test/theme_test.dart`

- [x] **Step 2.1: Write failing test for Pure Neutral Dark Theme**
  Write `apps/mobile/test/theme_test.dart` asserting exact hex codes (`background: #0a0b0e`, `sidebar: #111216`, `surface: #16171d`, `card: #1c1e24`, `primary: #3b82f6`, `speaking: #22c55e`).

- [x] **Step 2.2: Create `apps/mobile/pubspec.yaml`**
  Define dependencies: `flutter_riverpod`, `http`, `open_filex`, `package_info_plus`, `url_launcher`, `path_provider`, `flutter_webrtc`.

- [x] **Step 2.3: Implement `colors.dart` and `theme.dart`**
  Implement constant tokens and `SharkordTheme.darkTheme`.

- [x] **Step 2.4: Implement `main.dart`**
  Initialize `ProviderScope` and base MaterialApp.

- [x] **Step 2.5: Run tests and verify**
  Run `flutter test test/theme_test.dart`.

- [x] **Step 2.6: Commit changes**
  Commit with message `feat(mobile): scaffold Flutter client with Pure Neutral Dark theme`.

---

### Task 3: Mobile In-App Update Engine & Auto-Updater (Android & iOS)

**Files:**
- Create: `apps/mobile/lib/core/update/update_model.dart`
- Create: `apps/mobile/lib/core/update/update_service.dart`
- Create: `apps/mobile/lib/core/update/android_updater.dart`
- Create: `apps/mobile/lib/core/update/ios_updater.dart`
- Create: `apps/mobile/lib/widgets/update_dialog.dart`
- Create: `apps/mobile/test/update_service_test.dart`

- [ ] **Step 3.1: Write failing tests for update models and SemVer parser**
  Write tests in `apps/mobile/test/update_service_test.dart` testing `SemVer.isNewer("0.2.0", "0.2.1")`, asset matching for Android `.apk`, and iOS release redirect.

- [ ] **Step 3.2: Implement `update_model.dart`**
  Implement `ReleaseInfo`, `ReleaseAsset`, and `SemVer` comparator.

- [ ] **Step 3.3: Implement `update_service.dart`**
  Implement GitHub Releases API client with HTTP client mocking support.

- [ ] **Step 3.4: Implement `android_updater.dart`**
  Implement streaming APK download with progress callback and `OpenFilex.open()` package installation trigger.

- [ ] **Step 3.5: Implement `ios_updater.dart`**
  Implement launch URL handler opening TestFlight / GitHub release page.

- [ ] **Step 3.6: Implement `update_dialog.dart`**
  Discord-grade dialog showing release notes, current vs target version, progress bar during download, and "Install" button.

- [ ] **Step 3.7: Run tests and verify**
  Run `flutter test test/update_service_test.dart`.

- [ ] **Step 3.8: Commit changes**
  Commit with message `feat(mobile): implement in-app update checker and auto-updater for Android and iOS`.

---

### Task 4: Mobile Rust FFI Bridge (`crates/sharkord-mobile/`) & Riverpod State Layer

**Files:**
- Create: `crates/sharkord-mobile/Cargo.toml`
- Create: `crates/sharkord-mobile/src/lib.rs`
- Create: `crates/sharkord-mobile/tests/bridge_test.rs`
- Create: `apps/mobile/lib/models/user.dart`
- Create: `apps/mobile/lib/models/channel.dart`
- Create: `apps/mobile/lib/models/message.dart`
- Create: `apps/mobile/lib/providers/auth_provider.dart`
- Create: `apps/mobile/lib/providers/channel_provider.dart`
- Create: `apps/mobile/lib/providers/message_provider.dart`
- Create: `apps/mobile/lib/providers/voice_provider.dart`

- [ ] **Step 4.1: Write failing test for `crates/sharkord-mobile`**
  Create `crates/sharkord-mobile/tests/bridge_test.rs` verifying client initialization and type mappings.

- [ ] **Step 4.2: Implement `crates/sharkord-mobile`**
  Wrap `sharkord-core` types, client configuration, and export clean FFI bridge methods. Add crate to root `Cargo.toml`.

- [ ] **Step 4.3: Implement Dart data models and Riverpod providers**
  Create `User`, `Channel`, `Message`, and providers with state management and optimistic updates.

- [ ] **Step 4.4: Run Rust and Dart tests**
  Run `cargo test -p sharkord-mobile` and `flutter test`.

- [ ] **Step 4.5: Commit changes**
  Commit with message `feat(mobile): implement Rust mobile bridge crate and Riverpod state providers`.

---

### Task 5: Mobile Navigation, Chat Feed & Discord-Grade Voice UI

**Files:**
- Create: `apps/mobile/lib/screens/login_screen.dart`
- Create: `apps/mobile/lib/screens/workspace_screen.dart`
- Create: `apps/mobile/lib/widgets/channel_drawer.dart`
- Create: `apps/mobile/lib/widgets/member_drawer.dart`
- Create: `apps/mobile/lib/widgets/chat_canvas.dart`
- Create: `apps/mobile/lib/widgets/voice_bar.dart`
- Create: `apps/mobile/lib/widgets/voice_stage_sheet.dart`
- Create: `apps/mobile/test/widget_test.dart`

- [ ] **Step 5.1: Write failing widget test for workspace screen**
  Test drawer opening, channel selection, voice bar appearance when connected, and voice stage bottom sheet.

- [ ] **Step 5.2: Implement `workspace_screen.dart` and drawers**
  Build three-pane layout: Left Drawer (`channel_drawer.dart`), Center Canvas (`chat_canvas.dart`), Right Drawer (`member_drawer.dart`).

- [ ] **Step 5.3: Implement `voice_bar.dart` and `voice_stage_sheet.dart`**
  Docked mini voice bar pinned to bottom; voice stage sheet showing participant cards and green speaking indicators.

- [ ] **Step 5.4: Implement `login_screen.dart`**
  Clean login form with server URL, email/password, and validation.

- [ ] **Step 5.5: Run widget tests**
  Run `flutter test test/widget_test.dart`.

- [ ] **Step 5.6: Commit changes**
  Commit with message `feat(mobile): implement Discord-grade three-pane navigation and voice stage UI`.

---

### Task 6: Android & iOS CI Packaging Pipeline & Release Asset Verification

**Files:**
- Modify: `.github/workflows/rust-ci.yml`

- [ ] **Step 6.1: Add `build-mobile-android` job to `.github/workflows/rust-ci.yml`**
  Set up Java 17 and Flutter stable, run `flutter pub get` and `flutter build apk --release`, copy output to `dist-mobile/sharkord-mobile-android.apk`, and upload artifact `sharkord-mobile-android`.

- [ ] **Step 6.2: Include mobile artifacts in `publish-release`**
  Add `build-mobile-android` to `needs: [check-and-test, build-macos, build-mobile-android]` and ensure `dist-release` includes the Android APK.

- [ ] **Step 6.3: Verify entire workspace build and formatting**
  Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --all`.

- [ ] **Step 6.4: Commit all changes**
  Commit with message `ci: integrate Android APK packaging into release pipeline`.

- [ ] **Step 6.5: Push to remote and trigger release**
  Push branch, re-tag or push tag `v0.2.0` (or `v0.2.1` if user requests version bump), monitor GitHub Actions run, and verify all desktop and mobile assets appear on the GitHub release page.
