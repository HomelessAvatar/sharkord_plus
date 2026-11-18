# Dual Desktop Strategy & Client Parity Design

## 1. Intent and Why

Sharkord needs full client parity across all supported platforms (Web, Desktop, Mobile).
Currently, the Web client (`apps/client`) is feature-complete with real WebRTC Mediasoup voice/video, 4K 120/144 FPS streaming, file/image uploads, emoji reactions, soundboard, settings, and plugin integrations.

However:
- The experimental native Rust GPUI desktop client (`apps/desktop-gpui`) is in an early prototype stage with static mock views, missing network bindings, lack of editable text inputs, missing context menus, no file picker, and no video rendering pipeline.
- The mobile Flutter client (`apps/mobile`) has persistent session vault and real WebRTC voice engine, but lacks UI affordances for file sharing, emoji reactions, message context menus, and video stream rendering (`RTCVideoRenderer`).

To deliver an immediate, fully working, production-grade desktop client to users without delay while preserving the long-term vision of a blazing-fast native Rust desktop experience, we adopt a **Dual Desktop Strategy**:
1. **Ship Tauri 2.0 Hybrid Desktop First (`apps/desktop-tauri`):** Wrap the battle-tested `apps/client` React application using Tauri 2.0 + `crates/sharkord-core`. Delivers instant 100% full parity on Day 1 (voice, video, files, emojis, plugins, settings) combined with native OS features (system tray, global push-to-talk hotkeys, OS notifications, encrypted `SessionVault`).
2. **Close Remaining Mobile Gaps (`apps/mobile`):** Wire file picker with raw binary upload, emoji reaction picker, message context menu, and `RTCVideoRenderer`.
3. **Advance GPUI to Full Parity as an Independent Track (`apps/desktop-gpui`):** Systematically build the missing interactive widgets (`TextInput`, context menus, popups, tokio tRPC client, WebRTC video texture rendering) in `apps/desktop-gpui` on a dedicated development track until it reaches complete parity with Web and Tauri.

---

## 2. Goals, Non-Goals, Anti-Goals

### Goals
- **Immediate Desktop Parity:** Deliver a desktop app that works out of the box with zero broken buttons, full WebRTC voice/video, file uploads, and session persistence.
- **Resource Efficiency:** Keep Tauri memory consumption within ~150–220 MB RAM (well below Electron's 500–800 MB).
- **Native OS Superpowers in Tauri:** Provide global hotkeys (push-to-talk outside the app window), system tray icon with mute/deafen and quit, OS badge/notifications, and native auto-update via Rust.
- **Mobile Parity:** Add file picker, emoji reactions, message edit/delete, and `RTCVideoRenderer` to Flutter mobile.
- **Existing Protocol & Contract Alignment:** Satisfy `TDesktopBridge` in `packages/shared/src/desktop.ts` via a Tauri IPC bridge adapter; adhere to server's raw binary upload protocol in `apps/server/src/http/upload.ts`.

### Non-Goals
- Replacing the Web client (`apps/client`).
- In-game overlay for desktop (deferred to a later release).
- Built-in soundboard in the Rust core (remains a plugin).
- Mobile bidirectional 4K camera broadcasting (mobile video focus is remote viewing first).

### Anti-Goals
- **Mock-Only Interfaces:** No more static mock-only screens in release builds.
- **Dual Codebase Divergence for Web:** We do not rewrite web features for desktop twice; Tauri directly uses `apps/client`.
- **Heavy Electron Bloat:** We strictly reject Electron (150+ MB installer, Chromium bundled); Tauri uses the OS WebView2 / WebKit.

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
4. **Platform Compatibility:** Tauri 2.0 app must build cleanly for Windows x64, macOS, and Linux x86_64.

---

## 4. Architectural Approaches Evaluated

### Approach 1: Single Native GPUI Desktop (Rejected for Immediate Ship)
- Build all missing widgets (rich text editor, context menu, modal system, file dialog, emoji picker) and WebRTC C++ video decoder in GPUI.
- *Trade-offs:* Requires months of complex GUI development in Rust. Delays production desktop delivery.

### Approach 2: Electron Hybrid Desktop (Rejected)
- Wrap `apps/client` with Electron.
- *Trade-offs:* Heavy RAM footprint (600+ MB), 120+ MB installer binary, contradicts Sharkord's lightweight performance philosophy.

### Approach 3: Dual Strategy (Tauri 2.0 First + GPUI Parity Track) (Selected)
- **Tauri 2.0:** Packages `apps/client` with a Rust backend leveraging `crates/sharkord-core` for `SessionVault`, global shortcuts, tray, and native notifications. Delivers 100% feature parity immediately.
- **GPUI Track:** Maintained as the high-performance native option, progressing incrementally with real widgets and networking.
- *Why Selected:* Gives users a fully working, polished desktop client today, while preserving the competitive advantage of a sub-50MB RAM pure-Rust client as it matures.

---

## 5. Technical Specifications

### Phase A: Tauri 2.0 Hybrid Desktop (`apps/desktop-tauri`)

#### 5.1 Project Structure
```
apps/desktop-tauri/
├── src-tauri/
│   ├── Cargo.toml          # Tauri 2.0, sharkord-core, tauri-plugin-global-shortcut, tray
│   ├── tauri.conf.json     # App window configs, security, deep link, bundle settings
│   ├── src/
│   │   ├── main.rs         # Tauri application bootstrap
│   │   ├── bridge.rs       # Implements TDesktopBridge IPC commands
│   │   ├── vault.rs        # SessionVault IPC handlers
│   │   ├── tray.rs         # System tray icon with status, mute, and settings
│   │   └── shortcuts.rs    # Global push-to-talk and mute hotkeys
│   └── icons/              # Sharkord desktop app icons (ico, png)
└── package.json            # Scripts to invoke vite build from apps/client and tauri dev/build
```

#### 5.2 Desktop Bridge Implementation (`TDesktopBridge`)
- Satisfies `packages/shared/src/desktop.ts`:
  - Injects `window.sharkordDesktop = { version: 1, connect: () => tauriBridge }`.
  - Implements:
    - `setUnreadState(state)`: Updates OS window badge and taskbar flash.
    - `setConnectionState(state)`: Reflects in tray icon tooltip/status.
    - `setVoiceActive(active)`: Reflects mic status in tray.
    - `showNotification(notification)`: Displays native OS toast notification with click callback.
    - `setHotkeys(bindings)` / `onHotkey(listener)`: Connects global Push-To-Talk via Tauri global shortcut plugin.
- Adapts `getHostFromServer()`: When running inside desktop wrapper, reads target host from stored server configuration (`SessionVault`), with a fallback to user-entered Server URL on initial connection.

#### 5.3 Rust Backend Capabilities in Tauri
- **Encrypted Session Vault:** Exposes `save_session`, `get_session`, `clear_session` to frontend via Tauri IPC using `crates/sharkord-core/src/vault.rs`.
- **System Tray:** Tray menu with `Show Sharkord`, `Mute Microphone`, `Deafen`, and `Quit`.
- **Global Shortcuts:** Registers global push-to-talk key to emit events to frontend even when game/other window is focused.
- **Single Instance & Deep Links:** Prevents multi-instance launch and handles `sharkord://` invite links.

---

### Phase B: Mobile Client Parity (`apps/mobile`)

#### 5.4 File / Image Sharing
- Add `file_picker` dependency to `apps/mobile/pubspec.yaml`.
- In `ChatInputBar`:
  - Add attachment button (`+` icon).
  - On tap: open system file/photo picker.
  - Upload file via HTTP raw binary stream using `UploadHeaders` (`x-sharkord-token`, `x-sharkord-original-name`, `x-sharkord-content-length`) matching `apps/server/src/http/upload.ts`.
  - Attach returned file ID to message payload via tRPC `messages.send`.

#### 5.5 Emoji Reactions
- Add emoji reaction picker bottom sheet on message long-press.
- Display reaction counters on message bubble.
- Wire to `messages.toggleReaction` tRPC procedure.

#### 5.6 Message Context Menu
- On message long-press / secondary tap:
  - If own message: Edit, Delete, Pin, Reply, Copy.
  - If other's message: Add Reaction, Reply, Pin (with permissions), Copy.
- Wire Edit to `messages.edit` and Delete to `messages.delete`.

#### 5.7 WebRTC Video Stream Rendering (`RTCVideoRenderer`)
- In `VoiceRoomScreen`:
  - Initialize `RTCVideoRenderer` instances for active remote video producers (cameras & screenshares).
  - Bind Mediasoup consumer streams to `RTCVideoRenderer.srcObject`.
  - Display video tiles in a responsive grid layout with speaking border indicators.

---

### Phase C: GPUI Desktop Parity Roadmap (`apps/desktop-gpui`)
*(Dedicated subsequent development track — does not block shipping Phase A & B)*

#### 5.8 Interactive State & Networking
- Replace static mock data with dynamic models managed via `gpui::Model<AppState>`.
- Wire `crates/sharkord-core/src/client.rs` using Tokio background runtime inside GPUI to connect to real server WebSocket and tRPC subscriptions.

#### 5.9 Interactive UI Widgets
- Implement editable `TextInput` component supporting focus, caret navigation, text selection, and Enter-to-send.
- Add click handlers (`.on_mouse_down(MouseButton::Left, ...)`) on channel list, server list, and voice bar buttons.
- Implement popup/overlay system for context menus and modals.

#### 5.10 WebRTC Video Pipeline in Rust
- Connect Rust Mediasoup consumer video RTP packets to a decoder (OpenH264 / VP8 via `ffmpeg-next` or `webrtc-rs`).
- Convert decoded RGB frames into GPUI image textures rendered in `video_tile.rs`.

---

## 6. What Is Left to the Builder (Architecture Calls)

1. **Server Discovery & Selection in Desktop (Phase A):**
   - On first launch, desktop displays a connection screen asking for `Server URL` (defaulting to `http://localhost:4991` or user's custom server), exactly like mobile does.
   - Once connected and logged in, credentials and server URL are persisted in `SessionVault`.
   - Settings menu provides a "Change Server / Disconnect" button to switch servers.
2. **Desktop Bridge Compatibility:**
   - Implements existing `TDesktopBridge` contract in `packages/shared/src/desktop.ts` to ensure 100% drop-in compatibility with `apps/client`.
3. **Upload Protocol Compliance:**
   - Mobile client strictly implements the server's existing raw binary stream with `UploadHeaders`, requiring zero breaking changes to `apps/server/src/http/upload.ts`.
4. **Mobile Video Scope:**
   - Remote screenshare/video viewing in `VoiceRoomScreen` is implemented first using `RTCVideoRenderer`.
