# Desktop Application Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the integrated desktop application settings tab inside `UserSettings` for Tauri 2.0 with native startup lifecycle controls, hardware video acceleration preference, file system directory shortcuts, and diagnostics.

**Architecture:** Extend `SessionVault` in `sharkord-core` with atomic `DesktopSettings` persistence, wire Tauri 2.0 native lifecycle handlers (close to tray, start to tray, autostart, directory opener) into `bridge.rs` and `lib.rs`, expose them via `TDesktopBridge` in `packages/shared`, and render an integrated `DesktopSettings` tab in `apps/client` matching Pure Neutral Dark styling, accessible directly from `TopBar` and tray via `defaultTab: 'desktop'`.

**Tech Stack:** Rust (Tauri 2.0, `sharkord-core`, serde), TypeScript, React 19, Redux Toolkit, Tailwind CSS, `@sharkord/ui`, Lucide icons.

**Spec:** `docs/superpowers/specs/2026-11-24-desktop-settings-design.md`

## Global Constraints

- **Design System:** Follow Pure Neutral Dark tokens (`#0A0A0A` background, muted monospace code blocks, Lucide icons, `@sharkord/ui` `SettingsSection`, `Group`, `Switch`, `IconButton`).
- **Platform Safety:** Desktop settings must only render when `isDesktopApp()` is true. Web and mobile environments must not break, show desktop options, or throw missing-bridge errors.
- **Persistence:** All desktop settings must persist across restarts in `SessionVault` (`vault.json`) with backwards compatibility.
- **Coding Style:** Named exports, kebab-case filenames, modern ES6+, no inline JSX functions where memoization applies, user strings in `i18n` with `bun run synci18n`.
- **Verification:** Every change must pass `bun run magic`, `cargo test -p sharkord-core`, and `cargo check --manifest-path apps/tauri/src-tauri/Cargo.toml`.

## Review Focus

1. **Backwards Compatibility with Existing `vault.json`**: An existing user config file missing the `desktopSettings` field must deserialize cleanly without data loss or crashes, defaulting to standard desktop settings (`closeToTray: true`, `gpuEncoding: true`, `dynamicTrayIcon: true`).
2. **Web / Mobile Browser Fallback**: When running in a standard web browser or mobile wrapper where `window.sharkordDesktop` is absent, `isDesktopApp()` is false; opening `UserSettings` must not error or display undefined desktop tabs.
3. **Double-Click Launch vs. System Autostart**: If `startToTray` is enabled, manually launching the app (clicking desktop/start-menu shortcut) must show the window; the window only stays hidden in tray when launched with `--hidden` or `--autostart`.
4. **Prevent Close Interception Lifecycle**: When `closeToTray` is enabled, clicking the window close button (`X`) must prevent window destruction (`api.prevent_close()`) and hide the window, while tray menu "Quit" or `app.exit(0)` must genuinely quit.
5. **Path Formatting and Directory Launch**: When opening app or logs folders, missing directory structures must be created before attempting to launch the OS file manager, preventing silent failures or OS errors.

---

### Task 1: Core Vault Persistence & DesktopSettings Schema

**Files:**
- Modify: `crates/sharkord-core/src/vault.rs`
- Test: `crates/sharkord-core/tests/vault_test.rs`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct DesktopSettings {
      pub auto_start: bool,
      pub close_to_tray: bool,
      pub start_to_tray: bool,
      pub dynamic_tray_icon: bool,
      pub gpu_encoding: bool,
  }
  ```
  `SessionVault.desktop_settings: DesktopSettings` with `get_desktop_settings()` and `update_desktop_settings()`.

- [ ] **Step 1: Write failing unit test for `DesktopSettings` serialization and backward compatibility**

In `crates/sharkord-core/tests/vault_test.rs`, add:
```rust
#[test]
fn test_desktop_settings_default_and_backward_compatibility() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("vault.json");

    // Old vault JSON without desktopSettings
    let old_json = r#"{"servers":[],"profiles":[]}"#;
    std::fs::write(&file_path, old_json).unwrap();

    let mut vault = SessionVault::load_from_path(&file_path).unwrap();
    assert_eq!(vault.desktop_settings.close_to_tray, true);
    assert_eq!(vault.desktop_settings.gpu_encoding, true);
    assert_eq!(vault.desktop_settings.auto_start, false);
    assert_eq!(vault.desktop_settings.start_to_tray, false);

    // Update desktop settings
    vault.desktop_settings.auto_start = true;
    vault.desktop_settings.close_to_tray = false;
    vault.save().unwrap();

    let reloaded = SessionVault::load_from_path(&file_path).unwrap();
    assert_eq!(reloaded.desktop_settings.auto_start, true);
    assert_eq!(reloaded.desktop_settings.close_to_tray, false);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p sharkord-core --test vault_test test_desktop_settings_default_and_backward_compatibility`
Expected: FAIL (no field `desktop_settings` on `SessionVault`).

- [ ] **Step 3: Implement `DesktopSettings` in `crates/sharkord-core/src/vault.rs`**

Add `DesktopSettings` struct with `default_true` helpers and field `pub desktop_settings: DesktopSettings` with `#[serde(default)]` in `SessionVault`:
```rust
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettings {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub start_to_tray: bool,
    #[serde(default = "default_true")]
    pub dynamic_tray_icon: bool,
    #[serde(default = "default_true")]
    pub gpu_encoding: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            close_to_tray: true,
            start_to_tray: false,
            dynamic_tray_icon: true,
            gpu_encoding: true,
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test -p sharkord-core --test vault_test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sharkord-core/src/vault.rs crates/sharkord-core/tests/vault_test.rs
git commit -m "feat(core): add DesktopSettings schema and vault persistence"
```

---

### Task 2: Tauri IPC Bridge, Hardware Acceleration & Window Event Lifecycle

**Files:**
- Modify: `apps/tauri/src-tauri/Cargo.toml`
- Modify: `apps/tauri/src-tauri/src/bridge.rs`
- Modify: `apps/tauri/src-tauri/src/lib.rs`
- Modify: `apps/tauri/src-tauri/src/bridge_init.js`

**Interfaces:**
- Consumes: `sharkord_core::vault::DesktopSettings`, `SessionVault` from Task 1.
- Produces:
  Tauri IPC commands:
  - `get_desktop_settings`
  - `update_desktop_settings`
  - `get_app_folder`
  - `open_app_folder`
  - `get_logs_folder`
  - `open_logs_folder`
  - `get_desktop_diagnostics`
  Hardware acceleration: Configures `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` with `--enable-features=WebRtcAllowHwAv1Encoding,ZeroCopyVideoStream,RawDraw --ignore-gpu-blocklist --enable-gpu-rasterization --enable-zero-copy` to unlock NVENC AV1 encoding on RTX 50-series GPUs.
  Window lifecycle: `close_to_tray` and `start_to_tray` handling.

- [ ] **Step 1: Write failing test for desktop settings bridge commands**

In `apps/tauri/src-tauri/src/bridge.rs`, add unit tests for `DesktopDiagnostics` and folder path resolution:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_structure() {
        let diag = get_diagnostics_info();
        assert!(!diag.version.is_empty());
        assert!(!diag.platform.is_empty());
        assert!(!diag.arch.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path apps/tauri/src-tauri/Cargo.toml test_diagnostics_structure`
Expected: FAIL (function `get_diagnostics_info` not found).

- [ ] **Step 3: Implement IPC commands and window event handling**

1. In `apps/tauri/src-tauri/src/bridge.rs`:
   - Add `pub close_to_tray: AtomicBool` to `BridgeState`.
   - Implement `get_desktop_settings`, `update_desktop_settings`.
   - Implement `get_app_folder`, `open_app_folder`, `get_logs_folder`, `open_logs_folder` (creating directory if missing and using `open::that` or `std::process::Command` to show in explorer).
   - Implement `DesktopDiagnostics` struct and `get_desktop_diagnostics`.
2. In `apps/tauri/src-tauri/src/lib.rs`:
   - Initialize `BridgeState.close_to_tray` from vault on launch.
   - Attach `.on_window_event` intercepting `WindowEvent::CloseRequested` when `close_to_tray` is enabled.
   - Check launch arguments (`--hidden` or `--autostart`) with `start_to_tray` to keep window hidden if launched on system boot.
   - Register new IPC commands in `tauri::generate_handler!`.
3. In `apps/tauri/src-tauri/src/bridge_init.js`:
   - Add `getSettings`, `updateSettings`, `getAppFolder`, `openAppFolder`, `getLogsFolder`, `openLogsFolder`, `getDiagnostics` to `entry.connect()`.

- [ ] **Step 4: Verify cargo check and tests pass**

Run: `& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path apps/tauri/src-tauri/Cargo.toml`
Run: `& "$env:USERPROFILE\.cargo\bin\cargo.exe" check --manifest-path apps/tauri/src-tauri/Cargo.toml`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/tauri/src-tauri/
git commit -m "feat(tauri): add desktop settings IPC commands and window close lifecycle"
```

---

### Task 3: Shared Desktop Bridge Types & Interface Parity

**Files:**
- Modify: `packages/shared/src/desktop.ts`
- Test: `packages/shared/src/__tests__/desktop.test.ts`

**Interfaces:**
- Consumes: Task 2 IPC API shape.
- Produces:
  - `TDesktopSettings` type and `desktopSettingsSchema`.
  - `TDesktopDiagnostics` type.
  - Extended `TDesktopBridge` interface with settings and folder methods.

- [ ] **Step 1: Write test for `TDesktopSettings` schema validation**

Create or extend `packages/shared/src/__tests__/desktop.test.ts`:
```ts
import { describe, expect, it } from 'bun:test';
import { desktopSettingsSchema, DEFAULT_DESKTOP_SETTINGS } from '../desktop';

describe('desktop settings schema', () => {
  it('validates default settings', () => {
    const parsed = desktopSettingsSchema.parse(DEFAULT_DESKTOP_SETTINGS);
    expect(parsed.closeToTray).toBe(true);
    expect(parsed.gpuEncoding).toBe(true);
  });

  it('validates partial updates', () => {
    const partial = desktopSettingsSchema.partial().parse({ closeToTray: false });
    expect(partial.closeToTray).toBe(false);
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run: `bun run --filter @sharkord/shared test`
Expected: FAIL (`desktopSettingsSchema` not found).

- [ ] **Step 3: Implement types and schema in `packages/shared/src/desktop.ts`**

Export `desktopSettingsSchema`, `DEFAULT_DESKTOP_SETTINGS`, `TDesktopSettings`, `TDesktopDiagnostics`, and add methods to `TDesktopBridge`:
```ts
export type TDesktopSettings = {
  autoStart: boolean;
  closeToTray: boolean;
  startToTray: boolean;
  dynamicTrayIcon: boolean;
  gpuEncoding: boolean;
};

export type TDesktopDiagnostics = {
  version: string;
  platform: string;
  arch: string;
  osVersion: string;
  webviewVersion: string;
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `bun run --filter @sharkord/shared test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/shared/src/
git commit -m "feat(shared): add desktop settings types and bridge signatures"
```

---

### Task 4: Client SettingsShell & UserSettings Tab Navigation Support

**Files:**
- Modify: `apps/client/src/components/server-screens/settings-shell/index.tsx`
- Modify: `apps/client/src/components/server-screens/user-settings/index.tsx`
- Modify: `apps/client/src/screens/server/top-bar.tsx`
- Modify: `apps/client/src/components/desktop-controller/index.tsx`

**Interfaces:**
- Consumes: `openServerScreen` from `features/server-screens/actions`.
- Produces:
  - `defaultTab?: string` support in `SettingsShell` and `UserSettings`.
  - TopBar desktop gear button and desktop controller opening `{ defaultTab: 'desktop' }`.
  - Conditional rendering of `desktop` tab in `UserSettings` if `isDesktopApp()` is true.

- [ ] **Step 1: Write test for UserSettings tab registration**

In `apps/client/src/components/server-screens/__tests__/user-settings.test.tsx`:
Add a test asserting that when `isDesktopApp()` returns true, the desktop entry `{ id: 'desktop' }` is present in the rendered settings list.

- [ ] **Step 2: Run test to verify failure**

Run: `bun run --filter client test`
Expected: FAIL (tab 'desktop' not found).

- [ ] **Step 3: Update `SettingsShell`, `UserSettings`, `TopBar`, and `DesktopController`**

1. In `SettingsShell`:
   - Accept `defaultTab?: string`.
   - Initialize `selectedId` with `defaultTab || entries[0]?.id || ''`.
2. In `UserSettings`:
   - Accept `defaultTab?: string`.
   - If `isDesktopApp()` is true, insert Desktop tab with `Monitor` icon.
3. In `TopBar`:
   - Update `handleOpenDesktopSettings`:
     ```ts
     openServerScreen(ServerScreen.USER_SETTINGS, { defaultTab: 'desktop' });
     ```
4. In `DesktopController`:
   - Update `onOpenSettings` callback:
     ```ts
     openServerScreen(ServerScreen.USER_SETTINGS, { defaultTab: 'desktop' });
     ```

- [ ] **Step 4: Run tests to verify passing**

Run: `bun run --filter client test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/client/src/components/server-screens/ apps/client/src/screens/server/ apps/client/src/components/desktop-controller/
git commit -m "feat(client): add defaultTab navigation and desktop tab registration"
```

---

### Task 5: DesktopSettings Component & Translations

**Files:**
- Create: `apps/client/src/components/server-screens/user-settings/desktop/index.tsx`
- Create: `apps/client/src/components/server-screens/user-settings/desktop/folder-item.tsx`
- Modify: `apps/client/src/i18n/locales/en/settings.json`
- Modify: `apps/client/src/i18n/locales/tr/settings.json`

**Interfaces:**
- Consumes: `getDesktopApi` from `helpers/desktop-api`, `@sharkord/ui` components (`SettingsSection`, `Group`, `Switch`, `IconButton`, `Button`, `Card`).
- Produces: The full Desktop Settings tab UI.

- [ ] **Step 1: Write test for DesktopSettings component rendering**

Create `apps/client/src/components/server-screens/user-settings/desktop/__tests__/desktop-settings.test.tsx`:
Verify that all 4 sections (General, Screen Share, Storage, Diagnostics) and switches render without error.

- [ ] **Step 2: Run test to verify failure**

Run: `bun run --filter client test desktop-settings`
Expected: FAIL (component doesn't exist).

- [ ] **Step 3: Implement `DesktopSettings` and `FolderItem` components**

1. In `folder-item.tsx`:
   - Render title, description, monospace path box, and open folder button with `FolderOpen` icon.
2. In `index.tsx`:
   - Load initial settings and folder paths using `desktopApi.getSettings()`, `getAppFolder()`, `getLogsFolder()`, `getDiagnostics()`.
   - Provide instant handlers for `autoStart`, `closeToTray`, `startToTray`, `dynamicTrayIcon`, `gpuEncoding` calling `desktopApi.updateSettings()`.
   - Provide buttons to open folders and display diagnostics.
3. Add i18n keys to `en/settings.json` and `tr/settings.json`:
   - `desktopTab`, `desktopGeneral`, `desktopAutoStart`, `desktopAutoStartDesc`, `desktopCloseToTray`, `desktopCloseToTrayDesc`, `desktopStartToTray`, `desktopStartToTrayDesc`, `desktopDynamicTrayIcon`, `desktopDynamicTrayIconDesc`, `desktopGpuEncoding`, `desktopGpuEncodingDesc`, `desktopAppFolder`, `desktopAppFolderDesc`, `desktopLogsFolder`, `desktopLogsFolderDesc`, `desktopDiagnostics`, `desktopVersion`, `desktopPlatform`, `desktopArch`.
4. Run `bun run synci18n` to propagate to all languages.

- [ ] **Step 4: Run client tests and typechecks**

Run: `bun run --filter client test`
Run: `bun run magic`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/client/
git commit -m "feat(client): implement DesktopSettings component with full i18n support"
```

---

### Task 6: Screen Share 60 FPS Capture, Motion Optimization & Encoder Parameters

**Files:**
- Modify: `apps/client/src/components/voice-provider/index.tsx`
- Modify: `apps/client/src/components/voice-provider/helpers.ts`
- Test: `apps/client/src/components/voice-provider/__tests__/helpers.test.ts`

**Interfaces:**
- Consumes: `screenSettings.screenFramerate`, `screenSettings.screenResolution`, `screenSettings.screenContentHint`.
- Produces: Full 60 FPS unthrottled WebRTC screen share stream with proper `contentHint`, `maxFramerate` on encodings, and `degradationPreference`.

- [ ] **Step 1: Write unit test for screen share encodings including target framerate**

In `apps/client/src/components/voice-provider/__tests__/helpers.test.ts`:
Add a test asserting that `getScreenShareSimulcastEncodings(maxBitrate, targetFramerate)` passes `maxFramerate: targetFramerate` to the high layer encoding and scales layers accordingly.

- [ ] **Step 2: Run test to verify failure**

Run: `bun run --filter client test helpers`
Expected: FAIL.

- [ ] **Step 3: Update `voice-provider/index.tsx` and `helpers.ts`**

1. In `voice-provider/index.tsx`:
   - Structure `displayMediaConstraints`:
     ```ts
     const { width, height } = getResWidthHeight(screenSettings.screenResolution);
     const fps = screenSettings.screenFramerate || 30;
     const displayMediaConstraints: MediaStreamConstraints = {
       video: {
         width: { ideal: width, max: width },
         height: { ideal: height, max: height },
         frameRate: { ideal: fps, max: fps },
         // @ts-expect-error - display capture only
         cursor: screenSettings.screenCursor
       },
       ...
     };
     ```
   - Automatically promote `videoTrack.contentHint` to `'motion'` when `fps >= 60` (or when user selected `ScreenContentHint.MOTION`), bypassing Chromium's 30Hz `detail` cap:
     ```ts
     const effectiveHint = fps >= 60 && screenSettings.screenContentHint === ScreenContentHint.DETAIL
       ? ScreenContentHint.MOTION
       : screenSettings.screenContentHint;
     videoTrack.contentHint = effectiveHint;
     ```
   - Add `maxFramerate: fps` and `maxBitrate: maxBitrateKbps * 1000` to `screenShareProducerOptions.encodings[0]`.
   - Update `getScreenShareSimulcastEncodings` to accept `targetFramerate: number` and set `maxFramerate: targetFramerate` on the high layer.
   - Set `degradationPreference = fps >= 60 ? 'maintain-framerate' : 'balanced'` on `producer.rtpSender`.

- [ ] **Step 4: Run client voice provider tests**

Run: `bun run --filter client test helpers`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/client/src/components/voice-provider/
git commit -m "fix(client): uncap screen sharing at 60fps with motion hint and sender parameters"
```

---

### Task 7: Full Integration Verification

**Files:**
- All touched files.

- [ ] **Step 1: Run comprehensive tests across workspaces**
  - Run `cargo test -p sharkord-core`
  - Run `cargo check --manifest-path apps/tauri/src-tauri/Cargo.toml`
  - Run `bun run test`
  - Run `bun run magic`
- [ ] **Step 2: Commit any formatting or lint fixes**
```bash
git add .
git commit -m "chore: verify desktop settings integration and clean formatting"
```
