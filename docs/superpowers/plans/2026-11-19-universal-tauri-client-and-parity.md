# Universal Tauri 2.0 Client & Full Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver 100% full client parity across Web, Desktop (Windows, macOS, Linux), and Mobile (Android, iOS) by wrapping the production-ready React client with Tauri 2.0, while establishing the interactive foundation for the native GPUI desktop.

**Architecture:** A single universal frontend (`apps/client`) runs on Web and inside Tauri 2.0 (`apps/tauri`). The Rust backend in Tauri leverages `crates/sharkord-core` for encrypted `SessionVault` persistence, implements the `TDesktopBridge` contract (`packages/shared/src/desktop.ts`) for system tray and global push-to-talk, and provides Android WebRTC permissions with a voice foreground service. GPUI (`apps/desktop-gpui`) receives an interactive state model and text input composer on its parallel roadmap.

**Tech Stack:** Tauri 2.0, Rust (edition 2021), React 18, Vite, Redux Toolkit, Tailwind CSS, Mediasoup Client, WebRTC, Android NDK/SDK, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-19-universal-tauri-client-and-gpui-design.md`

## Global Constraints

- SIFIR TÜRKÇE (Zero Turkish) on all GitHub surfaces: code, comments, commit messages, PRs, docs, and test names must be 100% English. User chat remains in Turkish.
- Pure Neutral Dark Theme (`#0A0A0A` base, `#121212` surface, `#1E1E1E` borders, `#3B82F6` electric blue accent, `#EDEDED` text).
- Adhere to `TDesktopBridge` in `packages/shared/src/desktop.ts` with zero breaking changes to `apps/client`.
- No Electron: Binary size must stay under 20 MB; idle RAM must stay under 200 MB.
- Mobile WebRTC must automatically acquire microphone and camera permissions without webview permission rejections.

## Review Focus

1. **WebView Origin & Remote Server Mismatch:** In Tauri bundled mode (`frontendDist`), `window.location.host` is `tauri.localhost`. If `getHostFromServer()` does not properly read from `SessionVault` / connection dialog, API and WebSocket calls fail to connect to the remote Sharkord server.
2. **Global Push-To-Talk Key Up/Down Synchronization:** If `tauri-plugin-global-shortcut` fails to fire the key-up event when the user switches windows mid-press, the microphone can stay stuck in an unmuted transmitting state.
3. **Android Background Voice Suspension:** When Android screen locks or the user switches apps during an active voice call, Android's battery optimizer suspends WebView audio unless a Foreground Service with `FOREGROUND_SERVICE_MICROPHONE` notification is active.
4. **SessionVault Concurrent Multi-Server State:** Switching servers must atomically write the new active profile and trigger complete Redux store reset (`cleanup()` and `resetServerState()`) to prevent state leaking between servers.
5. **WebRTC Permission Rejection on Mobile:** Android WebView `WebChromeClient.onPermissionRequest` by default denies microphone access unless explicitly overridden to grant access to the Tauri bundle origin.

---

### Task 1: Tauri 2.0 Workspace Scaffolding & Configuration

**Files:**
- Create: `apps/tauri/package.json`
- Create: `apps/tauri/src-tauri/Cargo.toml`
- Create: `apps/tauri/src-tauri/tauri.conf.json`
- Create: `apps/tauri/src-tauri/src/main.rs`
- Create: `apps/tauri/src-tauri/src/lib.rs`
- Modify: `package.json` (root workspace scripts)
- Test: `apps/tauri/src-tauri/tests/scaffold_test.rs`

**Interfaces:**
- Consumes: `crates/sharkord-core` (path: `../../../crates/sharkord-core`), `apps/client` dist build
- Produces: Executable Tauri application running on desktop with configured window, capabilities, and dependencies

- [ ] **Step 1: Write the failing compilation/config test**

```rust
// apps/tauri/src-tauri/tests/scaffold_test.rs
#[test]
fn test_tauri_config_exists_and_valid() {
    let config_str = include_str!("../tauri.conf.json");
    let json: serde_json::Value = serde_json::from_str(config_str)
        .expect("tauri.conf.json must be valid JSON");
    assert_eq!(json["productName"], "Sharkord Plus");
    assert_eq!(json["app"]["windows"][0]["title"], "Sharkord Plus");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml`
Expected: FAIL (file or directory not found)

- [ ] **Step 3: Implement Tauri 2.0 workspace and configuration**

Create `apps/tauri/package.json`:
```json
{
  "name": "@sharkord/tauri",
  "version": "0.2.5",
  "private": true,
  "scripts": {
    "dev": "tauri dev",
    "build": "tauri build",
    "android:dev": "tauri android dev",
    "android:build": "tauri android build"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0"
  }
}
```

Create `apps/tauri/src-tauri/Cargo.toml`:
```toml
[package]
name = "sharkord-tauri"
version = "0.2.5"
description = "Sharkord Plus Universal Desktop and Mobile Client"
authors = ["Sharkord Team"]
edition = "2021"

[lib]
name = "sharkord_tauri_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["tray-icon", "image-png"] }
tauri-plugin-global-shortcut = "2.0"
tauri-plugin-notification = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sharkord-core = { path = "../../../crates/sharkord-core" }
tokio = { version = "1.38", features = ["full"] }

[dev-dependencies]
serde_json = "1.0"
```

Create `apps/tauri/src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/tooling/cli/schema.json",
  "productName": "Sharkord Plus",
  "version": "0.2.5",
  "identifier": "com.sharkord.plus",
  "build": {
    "beforeDevCommand": "bun --filter @sharkord/client dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "bun --filter @sharkord/client build",
    "frontendDist": "../../../apps/client/dist"
  },
  "app": {
    "windows": [
      {
        "title": "Sharkord Plus",
        "width": 1280,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "backgroundColor": "#0A0A0A"
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
  }
}
```

Create `apps/tauri/src-tauri/src/lib.rs` and `apps/tauri/src-tauri/src/main.rs`.
Update root `package.json` with scripts: `"tauri:dev": "bun --filter @sharkord/tauri dev"` and `"tauri:build": "bun --filter @sharkord/tauri build"`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/tauri package.json
git commit -m "feat(tauri): scaffold tauri 2.0 universal workspace and configuration"
```

---

### Task 2: Encrypted `SessionVault` IPC & Server Discovery Integration

**Files:**
- Create: `apps/tauri/src-tauri/src/vault.rs`
- Modify: `apps/tauri/src-tauri/src/lib.rs`
- Modify: `apps/client/src/helpers/get-file-url.ts`
- Modify: `apps/client/src/features/server/actions.ts`
- Test: `apps/tauri/src-tauri/tests/vault_ipc_test.rs`

**Interfaces:**
- Consumes: `sharkord_core::vault::{SessionVault, ServerProfile}`
- Produces: Tauri IPC commands: `get_saved_session`, `save_session`, `clear_session`, `list_saved_servers`
- Client: `getHostFromServer()` resolves target server host dynamically when inside Tauri runtime

- [ ] **Step 1: Write the failing vault IPC test**

```rust
// apps/tauri/src-tauri/tests/vault_ipc_test.rs
use sharkord_core::vault::ServerProfile;

#[test]
fn test_vault_save_and_retrieve_profile() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("test_vault.json");
    let mut vault = sharkord_core::vault::SessionVault::with_path(vault_path);

    let profile = ServerProfile {
        id: "srv_test".into(),
        name: "Test Server".into(),
        server_url: "http://127.0.0.1:4991".into(),
        token: "jwt_token_123".into(),
        user: None,
    };

    vault.save_profile(profile.clone()).unwrap();
    assert_eq!(vault.get_profile("srv_test").unwrap().server_url, "http://127.0.0.1:4991");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --test vault_ipc_test`
Expected: Check passes or fails on missing deps.

- [ ] **Step 3: Implement Tauri vault commands in `apps/tauri/src-tauri/src/vault.rs`**

```rust
use sharkord_core::vault::{ServerProfile, SessionVault};
use std::sync::Mutex;
use tauri::State;

pub struct VaultState(pub Mutex<SessionVault>);

#[tauri::command]
pub fn get_saved_session(state: State<'_, VaultState>) -> Result<Option<ServerProfile>, String> {
    let vault = state.0.lock().map_err(|e| e.to_string())?;
    Ok(vault.get_active_profile().cloned())
}

#[tauri::command]
pub fn save_session(profile: ServerProfile, state: State<'_, VaultState>) -> Result<(), String> {
    let mut vault = state.0.lock().map_err(|e| e.to_string())?;
    vault.save_profile(profile).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_session(server_id: String, state: State<'_, VaultState>) -> Result<(), String> {
    let mut vault = state.0.lock().map_err(|e| e.to_string())?;
    vault.remove_profile(&server_id).map_err(|e| e.to_string())?;
    Ok(())
}
```

In `apps/client/src/helpers/get-file-url.ts`:
Add detection for Tauri environment (`window.__TAURI_INTERNALS__`). If present, retrieve active server host from stored server configuration (`SessionVault` / local override).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --test vault_ipc_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/tauri/src-tauri/src/vault.rs apps/tauri/src-tauri/src/lib.rs apps/client/src/helpers/get-file-url.ts
git commit -m "feat(tauri): add encrypted session vault ipc and client server discovery"
```

---

### Task 3: Full `TDesktopBridge` Implementation in Tauri

**Files:**
- Create: `apps/tauri/src-tauri/src/bridge.rs`
- Create: `apps/tauri/src-tauri/src/tray.rs`
- Create: `apps/tauri/src-tauri/src/shortcuts.rs`
- Modify: `apps/tauri/src-tauri/src/lib.rs`
- Test: `apps/tauri/src-tauri/tests/bridge_test.rs`

**Interfaces:**
- Consumes: `packages/shared/src/desktop.ts` (`TDesktopBridge`, `TDesktopUnreadState`, `TDesktopHotkeyBinding`)
- Produces: Native system tray, window badges, native notifications, and global hotkeys matching the `TDesktopBridge` contract

- [ ] **Step 1: Write the failing bridge test**

```rust
// apps/tauri/src-tauri/tests/bridge_test.rs
#[test]
fn test_bridge_contract_types() {
    let js_bridge = include_str!("../src/bridge_init.js");
    assert!(js_bridge.contains("window.sharkordDesktop"));
    assert!(js_bridge.contains("DESKTOP_BRIDGE_VERSION"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --test bridge_test`
Expected: FAIL (missing bridge_init.js)

- [ ] **Step 3: Implement bridge adapter, system tray, and global shortcuts**

Create `apps/tauri/src-tauri/src/bridge_init.js`:
Injected on webview creation to register `window.sharkordDesktop = { version: 1, connect: (v) => createTauriBridge() }`.
Expose invoke calls for:
- `setUnreadState`: Sets window badge / taskbar flash via Tauri window API.
- `setConnectionState`: Updates system tray icon tooltip.
- `setVoiceActive`: Updates mic indicator state in tray.
- `showNotification`: Dispatches native toast via `tauri-plugin-notification`.
- `setHotkeys`: Registers global push-to-talk shortcut via `tauri-plugin-global-shortcut`.

In `apps/tauri/src-tauri/src/tray.rs`:
Setup system tray with items:
- "Show Sharkord" -> brings main window to front and focuses.
- "Mute Microphone" -> emits toggle event to frontend.
- "Deafen" -> emits deafen event to frontend.
- "Quit" -> exits app.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --test bridge_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/tauri/src-tauri/src/bridge.rs apps/tauri/src-tauri/src/tray.rs apps/tauri/src-tauri/src/shortcuts.rs
git commit -m "feat(tauri): implement TDesktopBridge contract, system tray, and global shortcuts"
```

---

### Task 4: Mobile WebRTC Permissions & Background Voice Support

**Files:**
- Create: `apps/tauri/src-tauri/src/mobile_voice.rs`
- Modify: `apps/tauri/src-tauri/tauri.conf.json`
- Create/Configure: `apps/tauri/src-tauri/gen/android/app/src/main/AndroidManifest.xml`
- Test: `apps/tauri/src-tauri/tests/android_manifest_test.rs`

**Interfaces:**
- Consumes: `tauri::mobile_entry_point`
- Produces: Android WebRTC permissions (`RECORD_AUDIO`, `CAMERA`) auto-grant in WebView and continuous background voice call notification

- [ ] **Step 1: Write the failing Android manifest and permissions test**

```rust
// apps/tauri/src-tauri/tests/android_manifest_test.rs
#[test]
fn test_android_manifest_permissions() {
    let manifest = include_str!("../gen/android/app/src/main/AndroidManifest.xml");
    assert!(manifest.contains("android.permission.RECORD_AUDIO"));
    assert!(manifest.contains("android.permission.CAMERA"));
    assert!(manifest.contains("android.permission.MODIFY_AUDIO_SETTINGS"));
    assert!(manifest.contains("android.permission.FOREGROUND_SERVICE_MICROPHONE"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --test android_manifest_test`
Expected: FAIL (file not found)

- [ ] **Step 3: Implement Android manifest, WebChromeClient permission grant, and voice notification**

Configure `AndroidManifest.xml` with required hardware features and permissions:
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.RECORD_AUDIO" />
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE_MICROPHONE" />
<uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
```

In `MainActivity.kt`:
Override `onPermissionRequest(request: PermissionRequest)` on `WebChromeClient` to automatically grant `RESOURCE_AUDIO_CAPTURE` and `RESOURCE_VIDEO_CAPTURE` when origin is `tauri://localhost` or `http://localhost`.

In `apps/tauri/src-tauri/src/mobile_voice.rs`:
Expose `set_mobile_voice_active(active: bool)` command that triggers the Android foreground service with a persistent notification while in a voice lounge.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --test android_manifest_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/tauri/src-tauri/gen/android apps/tauri/src-tauri/src/mobile_voice.rs
git commit -m "feat(mobile): add android webrtc permissions and voice foreground service"
```

---

### Task 5: CI/CD Build Pipeline Automation for Desktop & Mobile

**Files:**
- Modify: `.github/workflows/desktop.yml`
- Create: `.github/workflows/mobile-tauri.yml`
- Test: Dry-run workflow syntax validation using GitHub Actions schema check

**Interfaces:**
- Consumes: `apps/tauri` and `apps/client` source trees
- Produces: Automated release builds:
  - Windows: `.exe` / `.msi`
  - Linux: `.AppImage` / `.deb`
  - macOS: `.dmg`
  - Android: `.apk` / `.aab`

- [ ] **Step 1: Write the failing workflow verification test**

```powershell
# Verify workflow files exist and have valid YAML syntax
$desktop = Get-Content .github/workflows/desktop.yml -Raw
$mobile = Get-Content .github/workflows/mobile-tauri.yml -Raw
if (-not ($desktop -match "tauri build")) { throw "desktop.yml missing tauri build" }
if (-not ($mobile -match "tauri android build")) { throw "mobile-tauri.yml missing tauri android build" }
```

- [ ] **Step 2: Run test to verify it fails**

Run in PowerShell: Execute above check.
Expected: FAIL (missing `tauri android build` / `mobile-tauri.yml`)

- [ ] **Step 3: Update `.github/workflows/desktop.yml` and create `.github/workflows/mobile-tauri.yml`**

In `.github/workflows/desktop.yml`:
Update matrix to use `tauri-apps/tauri-action@v0` to build Windows x64, macOS, and Linux binaries.

Create `.github/workflows/mobile-tauri.yml`:
Configure runner with `actions/setup-java@v4` (Java 17), `android-actions/setup-android@v3` (NDK, SDK), `tauri-apps/tauri-action@v0` to build Android APK and upload release artifacts.

- [ ] **Step 4: Run verification check to verify it passes**

Run in PowerShell: Execute verification check.
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/desktop.yml .github/workflows/mobile-tauri.yml
git commit -m "ci: automate universal tauri desktop and android release builds"
```

---

### Task 6: GPUI Long-Term Interactive Foundation Milestone

**Files:**
- Modify: `apps/desktop-gpui/src/main.rs`
- Modify: `apps/desktop-gpui/src/ui/chat_canvas.rs`
- Modify: `apps/desktop-gpui/src/ui/channel_tree.rs`
- Modify: `apps/desktop-gpui/src/ui/voice_bar.rs`
- Test: `apps/desktop-gpui/tests/gpui_interactive_test.rs`

**Interfaces:**
- Consumes: GPUI 0.2 `ViewContext`, `Model<AppState>`
- Produces: Reactive interactive state for channel selection, voice mute/deafen toggling, and text input composer state in GPUI

- [ ] **Step 1: Write the failing GPUI interactive test**

```rust
// apps/desktop-gpui/tests/gpui_interactive_test.rs
#![recursion_limit = "256"]
use sharkord_gpui::model::AppState;

#[test]
fn test_app_state_channel_and_voice_toggle() {
    let mut state = AppState::new_mock();
    assert_eq!(state.active_channel_id, "chan_general");
    
    state.select_channel("chan_voice_lounge");
    assert_eq!(state.active_channel_id, "chan_voice_lounge");

    assert!(!state.is_muted);
    state.toggle_mute();
    assert!(state.is_muted);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path apps/desktop-gpui/Cargo.toml --test gpui_interactive_test`
Expected: FAIL (missing `AppState` interactive methods)

- [ ] **Step 3: Implement dynamic `AppState` and interactive click handlers in GPUI**

Add `select_channel`, `toggle_mute`, `toggle_deafen`, and `send_message` methods to `AppState`.
In `channel_tree.rs`: Attach `.on_mouse_down(MouseButton::Left, cx.listener(|view, _, cx| view.on_channel_click(channel_id, cx)))`.
In `voice_bar.rs`: Attach click handlers to Mute, Deafen, and Disconnect buttons.
In `chat_canvas.rs`: Replace static mock text input with an interactive input state that accepts keystrokes and appends to the message history on Enter.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path apps/desktop-gpui/Cargo.toml --test gpui_interactive_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui
git commit -m "feat(gpui): implement reactive app state and interactive click handlers"
```
