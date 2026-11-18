# Universal Tauri 2.0 Client (Desktop & Mobile) & Native GPUI Roadmap

## 1. Intent and Why

Sharkord aims for 100% full client parity across all platforms (Web, Desktop, and Mobile).
Currently, the Web client (`apps/client`) is feature-complete with:
- Mediasoup real WebRTC voice & video engine
- 4K & 120/144 FPS screen sharing
- File/image uploads & previews
- Emoji reactions & custom emojis
- Message management (edit, delete, reply, pin)
- Rich markdown & syntax highlighting
- Server settings & member management
- Interactive slash commands, polls, and plugins
- Mobile-responsive drawer navigation (`useMobileDrawer`)

However, client parity on native targets has lagged:
- The experimental native Rust GPUI desktop client (`apps/desktop-gpui`) is in an early prototype stage with static mock views, missing network bindings, lack of editable text inputs, missing context menus, no file picker, and no video rendering pipeline.
- The Flutter mobile client (`apps/mobile`) required maintaining a completely separate UI codebase, resulting in missing features (file picker, emoji reactions, message edit/delete, remote video rendering).

To achieve immediate, seamless, 100% parity across **both Desktop (Windows, macOS, Linux) and Mobile (Android, iOS)** while avoiding duplicate frontend maintenance, we adopt the **Universal Tauri 2.0 Architecture**:
1. **Universal Tauri 2.0 Client (`apps/tauri`):** Packages `apps/client` for Desktop and Mobile using Tauri 2.0 + `crates/sharkord-core`.
   - **Desktop Targets (Windows, macOS, Linux):** System tray, global push-to-talk hotkeys, native OS notifications, encrypted `SessionVault`, window styling.
   - **Mobile Targets (Android, iOS):** Native WebRTC camera/mic permissions, Android foreground service for background voice, touch-optimized responsive layout, encrypted `SessionVault`.
2. **Retire/Supersede Flutter (`apps/mobile`):** By unifying desktop and mobile under the battle-tested React web client, all future features instantly ship across Web, Desktop, and Mobile simultaneously.
3. **Advance GPUI on an Independent High-Performance Track (`apps/desktop-gpui`):** Maintain GPUI as an optional, ultra-lightweight (<50MB RAM) native Rust client, developing missing interactive widgets (`TextInput`, context menus, Tokio tRPC networking, WebRTC video texture rendering) systematically.

---

## 2. Goals, Non-Goals, Anti-Goals

### Goals
- **Instant Universal 100% Parity:** Desktop and Mobile clients gain full feature parity with Web on Day 1 (voice, video, files, emojis, plugins, soundboard, settings).
- **Single Frontend Source of Truth:** `apps/client` serves Web, Desktop, and Mobile.
- **Resource Efficiency:**
  - Desktop Tauri: ~120–180 MB RAM (idle), ~250–320 MB (video).
  - Mobile Tauri: Native WebView resource consumption (~80–120 MB RAM).
- **Native OS Superpowers:**
  - Desktop: Global Push-to-Talk hotkeys, System Tray menu, OS Notifications.
  - Mobile: Android Foreground Service (keeps voice calls active when app is minimized or screen is locked), WebRTC camera & microphone permissions.
  - Common: Encrypted `SessionVault` from `crates/sharkord-core` for seamless auto-login.
- **Server Discovery & Connection:** Clean Server URL connection screen on first launch (defaulting to `http://localhost:4991`), persistent session storage, and an in-app "Switch Server" button.

### Non-Goals
- Maintaining duplicate Dart/Flutter code in parallel with Tauri Mobile.
- In-game desktop overlay (deferred to a later milestone).
- Built-in soundboard in Rust core (soundboard remains a plugin).

### Anti-Goals
- **Mock-Only Screens:** Zero static, non-interactive mock views in release builds.
- **Heavy Electron Bloat:** No Electron (no 150+ MB bundle or Chromium distribution).

---

## 3. Constraints

1. **Zero Turkish on GitHub Surfaces:** All code, comments, commit messages, PRs, and documentation must be 100% English. User chat remains in Turkish.
2. **Pure Neutral Dark Design System:** Adhere strictly to the `/impeccable` palette:
   - Primary Background: `#0A0A0A`
   - Secondary / Surface: `#121212`
   - Border / Dividers: `#1E1E1E`
   - Accent / Brand: `#3B82F6` (Electric Blue)
   - Text Primary: `#EDEDED`, Text Muted: `#888888`
3. **Security Check Order & Invariants:** Adhere to `AGENTS.md` guidelines for any tRPC or backend interactions.
4. **Platform Compatibility:** Tauri 2.0 app must build cleanly for Windows x64, macOS, Linux, and Android APK / AAB.

---

## 4. Technical Architecture

### 4.1 Universal Tauri 2.0 Workspace (`apps/tauri`)
```
apps/tauri/
├── src-tauri/
│   ├── Cargo.toml                  # Tauri 2.0, sharkord-core, plugins (global-shortcut, notification)
│   ├── tauri.conf.json             # Universal config for Desktop & Mobile
│   ├── capabilities/
│   │   ├── default.json            # Core permissions (core:default, notification, global-shortcut)
│   │   └── mobile.json             # Mobile-specific capabilities
│   ├── gen/android/                # Tauri Android project generated via `tauri android init`
│   └── src/
│       ├── lib.rs                  # Universal Tauri builder entry point
│       ├── main.rs                 # Desktop entry point
│       ├── bridge.rs               # Implements TDesktopBridge contract
│       ├── vault.rs                # SessionVault IPC commands (save, get, clear, list)
│       ├── tray.rs                 # Desktop system tray (Windows, macOS, Linux)
│       └── mobile_voice.rs         # Android foreground service bridge for active voice
└── package.json                    # Monorepo workspace package scripts (dev, build, build:android)
```

### 4.2 Web & Desktop Bridge Integration
- **Frontend Target:** Tauri points `frontendDist: "../../apps/client/dist"` and `devUrl: "http://localhost:5173"`.
- **Desktop Bridge (`TDesktopBridge`):**
  - Injects `window.sharkordDesktop = { version: 1, connect: () => tauriBridge }`.
  - Implements:
    - `setUnreadState(state)`: Updates OS window badge and taskbar flash.
    - `setConnectionState(state)`: Reflects in tray icon tooltip/status.
    - `setVoiceActive(active)`: Reflects mic status in tray, triggers Android foreground notification when in a voice room.
    - `showNotification(notification)`: Displays native OS toast notification with click callback.
    - `setHotkeys(bindings)` / `onHotkey(listener)`: Connects global Push-To-Talk via Tauri global shortcut plugin.
- **Server Discovery (`getHostFromServer`):**
  - When running inside Tauri (Desktop or Mobile), checks stored active server in `SessionVault`.
  - If no server is saved, presents the Server URL input modal (default: `http://localhost:4991`), authenticates, and persists the session.
  - "Switch Server" button in user settings clears active session or switches profile in `SessionVault`.

### 4.3 Mobile-Specific Implementations
- **WebRTC Permissions:**
  - `AndroidManifest.xml` includes `RECORD_AUDIO`, `CAMERA`, `MODIFY_AUDIO_SETTINGS`, and `FOREGROUND_SERVICE_MICROPHONE`.
  - WebView `onPermissionRequest` is handled in Android MainActivity to auto-grant WebRTC permissions to the app's origin.
- **Background Voice Call Service:**
  - When in an active voice room (`voiceActive == true`), an Android Foreground Service runs with a persistent notification ("Sharkord: Connected to voice channel"), preventing Android from suspending audio input/output when the app is backgrounded.
- **Responsive Touch Layout:**
  - `apps/client` already provides `useMobileDrawer` (`MobileDrawerContext`), opening the channel rail and member sidebar via touch swipe or top-bar hamburger icon.

### 4.4 GPUI Desktop Roadmap (`apps/desktop-gpui`)
*(Maintained on a separate high-performance track)*
- **Milestone 1:** Dynamic state management with `gpui::Model<AppState>` and interactive click handlers on channels, servers, and voice controls.
- **Milestone 2:** Custom editable `TextInput` component with focus, caret navigation, text selection, and Enter-to-send.
- **Milestone 3:** Tokio background runtime bridge connecting `crates/sharkord-core/src/client.rs` to real server WebSocket and tRPC subscriptions.
- **Milestone 4:** Context menus, popups, and modal dialogs.
- **Milestone 5:** WebRTC video texture rendering (OpenH264 / VP8 decoder pushing RGB frames to GPUI image textures).

---

## 5. Migration & Workspace Strategy

1. **New Package:** Create `apps/tauri` using official Tauri 2.0 CLI.
2. **Monorepo Scripts:** Root `package.json` adds:
   - `bun run tauri:dev` (runs `apps/client` dev server and launches Tauri window)
   - `bun run tauri:build` (builds `apps/client` and bundles desktop binaries: `.exe`, `.msi`, `.dmg`, `.AppImage`)
   - `bun run tauri:android:dev` / `bun run tauri:android:build` (bundles Android `.apk` / `.aab`)
3. **CI/CD Automation:**
   - Update `.github/workflows/desktop.yml` to build Tauri Windows x64 (`.msi` / `.exe`), Linux x86_64 (`.AppImage` / `.deb`), macOS (`.dmg`).
   - Add `.github/workflows/mobile-tauri.yml` to build Android APK using Ubuntu runner + Android SDK.
