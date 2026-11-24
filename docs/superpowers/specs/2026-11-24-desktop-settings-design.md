# Desktop Application Settings Integration (Tauri 2.0 & Web Client Parity)

## Intent and The Why

The desktop client for Sharkord Plus (VibeShark) runs on a modern, cross-platform Tauri 2.0 architecture. While the web client focuses on in-browser messaging, voice, and account management, a true desktop client requires native operating system integration controls:
1. **Startup & Lifecycle**: Launching on system boot (`autoStart`), starting silently in background on system boot (`startToTray`), and preventing accidental closure (`closeToTray`).
2. **Hardware Acceleration**: Enabling/disabling GPU-accelerated video encoding (`gpuEncoding`) for low-latency, high-fps screen sharing.
3. **Storage & Diagnostics**: Quick access to persistent application data (`%APPDATA%/sharkord` or `~/.config/sharkord`), application crash and session logs, and system/runtime diagnostic information.
4. **Desktop Settings Entrypoint**: In the desktop client header bar (`TopBar`), clicking the desktop settings gear icon should directly open these desktop preferences rather than showing an unspecialized web settings page.

Integrating these controls into Sharkord's unified `UserSettings` shell preserves user muscle memory, follows the pure neutral dark design system, and completely isolates desktop functionality from browser and mobile web sessions.

---

## Goals, Non-Goals, and Anti-Goals

### Goals
- Provide an integrated **Desktop ("Masaüstü")** settings tab in `UserSettings` when running inside the desktop app (`isDesktopApp() === true`).
- Deep-link the top bar's desktop gear button and native tray menu "Settings" action directly to the `desktop` tab via `defaultTab: 'desktop'`.
- Persist settings reliably across application restarts using the atomic `SessionVault` in `sharkord-core` (`vault.json`).
- Provide instant native effect on toggle:
  - `closeToTray`: Intercept window close (`WindowEvent::CloseRequested`), call `prevent_close()`, and hide the window to tray.
  - `startToTray`: When launching at login (via `autoStart` / `--hidden` / `--autostart`), keep the main window hidden in tray until clicked. Normal user manual launches always display the window.
  - `autoStart`: Configure OS auto-launch on Windows/macOS/Linux via the official `tauri-plugin-autostart`.
  - `openAppFolder` & `openLogsFolder`: Safely open the respective directories in the native file manager (Windows Explorer / macOS Finder / Linux file manager).
  - `diagnostics`: Display platform runtime (Tauri 2.0, WebView2/WebKit, architecture, OS version).

### Non-Goals
- Adding desktop settings to browser or mobile apps (they must strictly remain unaffected).
- Adding complex multi-window or multi-monitor canvas tiling beyond standard Tauri window controls.

### Anti-Goals
- Breaking separation of concerns: Do not invent a separate, disconnected modal or window that duplicates the `SettingsShell` styling and navigation.
- Over-engineering: Do not add unnecessary non-standard third-party dependencies when official Tauri plugins (`tauri-plugin-autostart`), `sharkord-core`, and `@sharkord/ui` can achieve the exact result.

---

## Constraints
- **Design System**: Strict adherence to Pure Neutral Dark tokens (`#0A0A0A` background, muted monospace code blocks, Lucide icons, `@sharkord/ui` `SettingsSection`, `Group`, `Switch`, `IconButton`).
- **Platform Safety**: Desktop commands in TypeScript must be guarded by `isDesktopApp()` or `getDesktopApi()`. Missing desktop API in web browser mode must silently fall back or be hidden.
- **Internationalization**: All labels and descriptions must be localized in `settings.json` and synced across all supported locales using `bun run synci18n`.

---

## Approaches Evaluated

### Approach 1: Standalone Floating Desktop Preferences Dialog
A separate popup dialog triggered exclusively from the top bar gear icon, independent of `UserSettings`.
- *Pros*: Completely separate codebase from user settings.
- *Cons*: Disjointed UX; two different places for settings; duplicate shell layout and styling; confusing for users.

### Approach 2: Integrated Desktop Tab in `UserSettings` (Selected)
Conditionally register a `{ id: 'desktop', label: t('desktopTab'), icon: Monitor, content: <DesktopSettings /> }` tab inside `UserSettings` when `isDesktopApp()` is true. Support `defaultTab: 'desktop'` in `SettingsShell` and `openServerScreen`.
- *Pros*: Natural and cohesive UX; matches Discord, Slack, and legacy Sharkord desktop conventions; reusable UI components (`SettingsSection`, `Group`, `Switch`, `FolderSettings`); both top-bar gear button and avatar settings cog access the exact same cohesive settings menu.
- *Selected by*: User decision ("Seçenek a").

---

## Clarifications on Core Features (Builder Q&A)

### 1. `gpuEncoding` & AV1/H.264 Hardware Acceleration & 60 FPS Capture
- **AV1 Hardware Acceleration on Modern GPUs (RTX 5070 / Blackwell / Ada)**:
  Chromium/WebView2 has AV1 hardware encoding disabled in WebRTC by default, causing it to fall back to CPU `libaom-av1`. To enable NVENC AV1 hardware encoding on supported GPUs, the desktop app sets `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` on Windows:
  `--enable-features=WebRtcAllowHwAv1Encoding,ZeroCopyVideoStream,RawDraw --ignore-gpu-blocklist --enable-gpu-rasterization --enable-zero-copy`.
- **60 FPS Screen Capture & Encoding Pipeline Fix**:
  1. Constraints: Pass `frameRate: { ideal: fps, max: fps }` and `width: { ideal: w, max: w }`, `height: { ideal: h, max: h }` to `getDisplayMedia`.
  2. Content Hint: When `screenFramerate >= 60`, automatically apply `contentHint = 'motion'` (or allow explicit override), preventing Chromium's `contentHint = 'detail'` 30Hz screen-capture throttle.
  3. Producer Encodings: Pass `maxFramerate: screenSettings.screenFramerate` and `maxBitrate: maxBitrateKbps * 1000` to both single-layer and simulcast high-layer producer encodings.
  4. Degradation Preference: Configure `degradationPreference = screenSettings.screenFramerate >= 60 ? 'maintain-framerate' : 'balanced'` on `producer.rtpSender`.

### 2. `startToTray` (Start Minimized to Tray)
- **Role**: Aligns with legacy Sharkord and Discord behavior: *"When Sharkord opens at system login, keep it in the system tray without showing the window."*
- **Behavior**: When launched via OS autostart (or with `--hidden` / `--autostart` flag), if `startToTray` is enabled, the window starts hidden in the tray. If the user explicitly launches the application manually by double-clicking the app icon/shortcut, the main window always shows immediately.

### 3. `autoStart` (Launch at System Login)
- **Role**: Uses official `tauri-plugin-autostart` (version `2.0`), consistent with the existing official `tauri-plugin-notification` and `tauri-plugin-global-shortcut` already configured in `Cargo.toml`.
- **Behavior**: On Windows, registers under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. On macOS, configures LaunchAgent / SMAppService. On Linux, creates an XDG Autostart entry.

### 4. `dynamicTrayIcon`
- **Role**: Controls whether live server notifications / status badges / active voice state update on the tray icon and tooltip. Default is `true`.

---

## Decided Architecture & Component Design

### 1. Core Persistence (`crates/sharkord-core/src/vault.rs`)
Add `DesktopSettings` to `SessionVault`:
```rust
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

fn default_true() -> bool {
    true
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
`SessionVault` struct includes:
```rust
#[serde(default)]
pub desktop_settings: DesktopSettings,
```
This guarantees backwards compatibility with existing `vault.json` files and atomic disk persistence.

### 2. Tauri IPC & Window Management (`apps/tauri/src-tauri/`)
- **`BridgeState`**:
  Add `pub close_to_tray: AtomicBool` and `pub start_to_tray: AtomicBool`.
- **Window Event Handling (`lib.rs`)**:
  ```rust
  .on_window_event(|window, event| {
      if let tauri::WindowEvent::CloseRequested { api, .. } = event {
          let bridge_state = window.state::<BridgeState>();
          if bridge_state.close_to_tray.load(Ordering::SeqCst) {
              api.prevent_close();
              let _ = window.hide();
          }
      }
  })
  ```
  On launch setup: check command-line arguments. If `--hidden` or `--autostart` is passed and `start_to_tray` is enabled, keep window hidden in tray.
- **Commands**:
  - `bridge::get_desktop_settings(vault: State<VaultState>) -> Result<DesktopSettings, String>`
  - `bridge::update_desktop_settings(app: AppHandle, vault: State<VaultState>, bridge: State<BridgeState>, settings: DesktopSettings) -> Result<(), String>`
  - `bridge::get_app_folder() -> Result<String, String>`
  - `bridge::open_app_folder() -> Result<(), String>`
  - `bridge::get_logs_folder() -> Result<String, String>`
  - `bridge::open_logs_folder() -> Result<(), String>`
  - `bridge::get_desktop_diagnostics() -> Result<DesktopDiagnostics, String>`

### 3. Desktop Bridge & Shared Types (`packages/shared/src/desktop.ts` & `bridge_init.js`)
Extend `TDesktopBridge` with:
- `getSettings: () => Promise<TDesktopSettings>`
- `updateSettings: (settings: Partial<TDesktopSettings>) => Promise<void>`
- `getAppFolder: () => Promise<string>`
- `openAppFolder: () => Promise<void>`
- `getLogsFolder: () => Promise<string>`
- `openLogsFolder: () => Promise<void>`
- `getDiagnostics: () => Promise<TDesktopDiagnostics>`

### 4. Client Settings UI (`apps/client/src/components/server-screens/`)
- **`SettingsShell`**:
  Accept `defaultTab?: string`.
  Initialize `selectedId` with `defaultTab || entries[0]?.id || ''`.
- **`UserSettings` (`index.tsx`)**:
  Accept `defaultTab?: string` prop passed from `ServerScreensProvider`.
  When `isDesktopApp()` is true, insert the Desktop tab entry:
  ```tsx
  {
    id: 'desktop',
    label: t('desktopTab'),
    icon: Monitor,
    content: <DesktopSettings />
  }
  ```
- **`DesktopSettings` component (`user-settings/desktop/index.tsx`)**:
  Renders using `@sharkord/ui`:
  1. `SettingsSection` (General & Startup):
     - `Group` for `autoStart` with `Switch`
     - `Group` for `closeToTray` with `Switch`
     - `Group` for `startToTray` with `Switch` (dependent on tray / close to tray)
     - `Group` for `dynamicTrayIcon` with `Switch`
  2. `SettingsSection` (Screen Sharing & Hardware):
     - `Group` for `gpuEncoding` with `Switch`
  3. `SettingsSection` (Storage & Directories):
     - `FolderSettings` for App Data Folder with path display & Open Folder icon button
     - `FolderSettings` for Logs Folder with path display & Open Folder icon button
  4. `SettingsSection` (About & Diagnostics):
     - Diagnostics card displaying Version, Platform, Architecture, and WebView runtime.

### 5. Deep-Linking Trigger (`TopBar` & `DesktopController`)
- In `top-bar.tsx`:
  ```tsx
  const handleOpenDesktopSettings = useCallback(() => {
    openServerScreen(ServerScreen.USER_SETTINGS, { defaultTab: 'desktop' });
  }, []);
  ```
- In `desktop-controller/index.tsx`:
  ```tsx
  return desktopApi.onOpenSettings(() => {
    openServerScreen(ServerScreen.USER_SETTINGS, { defaultTab: 'desktop' });
  });
  ```

---

## Builder Responsibilities
The builder implementing this specification will:
1. Implement the Rust backend changes in `crates/sharkord-core` and `apps/tauri/src-tauri`.
2. Connect the Tauri bridge and JS init script.
3. Add the UI components in `apps/client` matching the Pure Neutral Dark design system.
4. Add all localized keys to `settings.json` and run `bun run synci18n`.
5. Run automated tests (`bun run test`, `cargo test`, `bun run magic`) to verify zero regressions.
