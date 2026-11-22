# Full Native GPUI Desktop Client Feature Parity Design (Powered by `gpui-kit` & /impeccable Craft)

## 1. Intent and Why

Sharkord Plus aims to provide the fastest, most resource-efficient Discord alternative in existence.
Currently, the Web client (`apps/client`) and Tauri desktop client (`apps/tauri`) are feature-complete with real tRPC networking, WebSocket real-time event subscriptions, WebRTC Mediasoup voice and video, 4K 120/144 FPS screen sharing, rich chat interactions (markdown, code syntax highlighting, file uploads, emoji reactions, replies, pins), direct messaging (DMs), server/channel administration, and user settings.

In contrast, the native Rust GPUI desktop client (`apps/desktop-gpui`) currently functions as a prototype:
- It bootstraps from an in-memory mock (`AppState::new_mock()`).
- It has no active network connection to the server (no tRPC HTTP queries/mutations and no WebSocket subscriptions).
- Message input is a rudimentary string buffer (`push_str`) without caret navigation, text selection, clipboard operations, or multiline editing.
- Messages are rendered as unformatted plain text without markdown, syntax highlighting, or attachment previews.
- Voice and video views (`VoiceBar`, `VoiceStage`, `VideoTile`, `SourcePicker`) are static layouts without live Mediasoup RTP transports or real video decoders.
- Direct messaging, server settings, channel creation/editing modals, and comprehensive user settings are missing.

By leveraging **`gpui-kit` / `gpui-component`** (Longbridge's 75+ desktop UI components for GPUI) and modern, idiomatic **Rust media crates (`webrtc-rs`, `xcap`, `openh264`, `cpal`)** alongside **`crates/sharkord-core`**, this project delivers **100% native feature parity** in `apps/desktop-gpui`. This transforms it into a production-ready, GPU-accelerated desktop client consuming <50MB RAM at idle, achieving 120 FPS rendering, with zero webview overhead.

---

## 2. /impeccable Craft Standards & Design Floor

Following `/impeccable` Operate mode and craft floor requirements:

1. **Operate Mode Principles:**
   - **Earned Familiarity:** The tool disappears into the task. Classic Discord spatial layout (Server Rail -> Channel Tree -> Center Canvas -> Member List) preserved with precision hairline borders (`1px rgba(255,255,255,0.08)`).
   - **Complete State Vocabulary:** Every interactive component must define `default`, `hover`, `focus`, `active`, `disabled`, and `loading` states.
   - **Skeleton Over Spinners:** Skeletons (`gpui_component::Skeleton` / `Shimmer`) for message stream loading and member roster loading, never empty spinners.
   - **Virtual Scrolling:** Virtualized message stream (`gpui_component::VirtualList` / `MessageScroller`) ensuring 120 FPS silky-smooth scrolling even in channels with 10,000+ messages (`#counting`).
   - **Icon System Over Unicode:** Refuse raw unicode emoji characters (`👥`, `🔊`, `Send ↵`) for UI chrome; use consistent stroke-weighted vector icons (`gpui_component::Icon` / Lucide icons).
   - **Keyboard Polish:** Shortcut hints rendered with `gpui_component::Kbd` badges (`Ctrl+K`, `Enter`, `Esc`, `Shift+Enter`).

2. **Color Palette (Pure Neutral Dark):**
   - Background Base: `#0A0B0E`
   - Background Surface: `#111216`
   - Background Subsurface: `#16171d`
   - Background Elevated: `#1c1e24`
   - Hairline Borders: `rgba(255, 255, 255, 0.08)`
   - Text Primary: `#F3F4F6` (WCAG AAA contrast > 12:1)
   - Text Secondary: `#9CA3AF` (WCAG AA contrast > 6:1)
   - Text Muted: `#6B7280` (WCAG AA contrast > 4.5:1)
   - Accent: `#3B82F6` (Electric Blue), Accent Hover: `#60A5FA`
   - Status: Online `#22C55E`, Idle `#EAB308`, DND `#EF4444`, Offline `#6B7280`

---

## 3. Comprehensive `gpui-kit` Component Mapping

Every subsystem in `apps/desktop-gpui` is mapped to specialized `gpui-component` primitives:

| Application Subsystem | `gpui-component` Primitives Used |
|---|---|
| **Chat Composer** | `Textarea` (multiline, caret, selection, clipboard, auto-grow), `Button`, `Kbd` |
| **Chat Message Feed** | `VirtualList` / `MessageScroller`, `Bubble`, `Attachment`, `HoverCard`, `Badge` |
| **Message Actions** | `ContextMenu` (Edit, Delete, Pin, Reply, Copy), `Popover` (Emoji Reaction Picker) |
| **Markdown & Code** | `TextView`, `Editor` (syntax-highlighted code blocks with copy action) |
| **Channel Tree** | `Collapsible` / `Accordion` (categories), `Badge` (unreads), `Tooltip` |
| **Quick Switcher** | `Command` (command palette search with fuzzy filter and `Kbd` badges) |
| **Modals & Overlays** | `Dialog` / `Sheet` / `AlertDialog` (Connect, Auth, Server Settings, Channel Admin, Lightbox) |
| **User Settings** | `Tabs`, `Slider` (mic gain, VU meter, noise gate), `Select` / `Combobox` (audio devices), `Switch` |
| **User Profiles** | `Popover` / `HoverCard`, `Avatar` with online status indicator dot |
| **Empty States** | `Empty` with custom illustrations, welcome hashtags, and action shortcuts |
| **Loading Feedback** | `Skeleton` / `Shimmer` for message list and channel roster initial load |

---

## 4. Goals, Non-Goals, Anti-Goals

### Goals
- **100% Feature Parity with Web/Tauri:** Every core feature available on the web and desktop clients (networking, authentication, messaging, voice, video & screen share, settings, DMs) must be available natively in GPUI.
- **Pure Rust WebRTC & Media Stack:**
  - `webrtc` (`webrtc-rs`): Native WebRTC DTLS, ICE, SRTP, and RTP transport for Mediasoup SFU voice and video tracks.
  - `xcap`: Cross-platform hardware-accelerated screen and window capture (DXGI on Windows, ScreenCaptureKit on macOS, PipeWire on Linux).
  - `openh264` / `rusty_h264`: High-performance H.264 video decoding and encoding without complex C++ build toolchains.
  - `cpal`: Native microphone capture and playback with Noise Gate DSP and VAD speaking detection.
  - GPUI texture uploads: Decoded video frames rendered directly to GPU textures in `VideoTileView` up to 4K @ 144 FPS.
- **Shared Rust Core (`crates/sharkord-core`):** Systematically expand `sharkord-core` with typed methods for all tRPC procedures, file streaming uploads, and WebSocket events.
- **Single Active Server Architecture:** Match Web and Tauri client behavior: connect to one active server at a time, with instant switching between saved server profiles in `SessionVault`.

### Non-Goals
- Modifying backend schemas: DMs will strictly adhere to Sharkord's existing 1-on-1 channel model (`is_dm: true`) rather than introducing an ad-hoc "friend request" backend table.
- Embedding webview runtimes inside GPUI (all UI must be 100% native GPUI elements).
- In-game transparent HUD overlay (deferred to a future release).

### Anti-Goals
- **Reinventing Basic UI Primitives:** Avoid handwriting custom text layout hit-testing, cursor blink timers, or modal focus trapping from scratch when `gpui-kit` provides them out of the box.
- **Mock-Only States in Release:** No placeholders, fake data, or non-functional buttons.
- **UI Freezes / Main Thread Blocking:** Network requests, WebSocket handling, audio DSP, and video decoding must run off the GPUI main thread on dedicated Tokio background tasks.

---

## 5. Technical Specifications (Layer-by-Layer Execution)

### Phase 1: Shared Core SDK Expansion & Tokio Async Bridge

#### 1.1 `sharkord-core` Expansion
- Expand `crates/sharkord-core/src/client.rs` and `protocol/` with typed procedures matching server tRPC:
  - Reactions: `toggle_reaction(message_id, emoji)`
  - Messages: `edit_message(message_id, content)`, `delete_message(message_id)`, `pin_message(message_id, pinned)`
  - Channels: `create_channel(name, channel_type, category_id)`, `update_channel(id, ...)`, `delete_channel(id)`
  - Server & Roles: `get_roles()`, `get_members()`, `kick_user(id)`, `ban_user(id)`
  - Direct Messages: `get_dms()`, `open_dm(target_user_id)`
  - File Upload: Raw HTTP streaming upload with `UploadHeaders`
  - Typed WebSocket Event variants: `ReactionAdded`, `ReactionRemoved`, `ChannelCreated`, `UserPresenceChanged`, `UserTyping`.

#### 1.2 Async Runtime Bridge & SessionVault
- In `apps/desktop-gpui`:
  - Run background Tokio runtime communicating with GPUI via `cx.update(|model, cx| ...)` and event channels.
  - On startup: Load `SessionVault::load()`. If valid token and server URL exist, connect immediately; otherwise show `ServerConnectModal`.

#### 1.3 Server Connect, Login & Register Views (`gpui_component::Dialog`)
- `ServerConnectModal`: Server URL input, connection test, capability negotiation.
- `AuthModal`: Tabbed Login and Register views with validation, error display, and remember me.

---

### Phase 2: Real-time WebSocket Event Subscription Engine

#### 2.1 WebSocket Event Loop
- Persistent WebSocket connection with automatic exponential backoff reconnection.
- Reaktif state dispatch:
  - `MessageCreated`, `MessageUpdated`, `MessageDeleted`
  - `ReactionAdded`, `ReactionRemoved`
  - `ChannelCreated`, `ChannelUpdated`, `ChannelDeleted`
  - `UserPresenceChanged`
  - `VoiceStateUpdated`
  - `UserTyping` (5s transient indicator)

---

### Phase 3: Rich Chat Canvas & Messaging UX (Powered by `gpui-kit` & /impeccable)

#### 3.1 Virtualized Message Feed (`gpui_component::VirtualList` & `Bubble`)
- High-performance virtualized feed scrolling smoothly at 120 FPS with 10k+ messages.
- Skeleton loader (`gpui_component::Skeleton`) while fetching channel message pages.
- Avatars with fallback text and status rings (`gpui_component::Avatar`).

#### 3.2 Interactive Composer (`gpui_component::Textarea`)
- Multi-line input with auto-growing height up to 200px.
- Caret positioning, text selection, clipboard operations (`Ctrl+C`/`Ctrl+V`).
- `Enter` to send, `Shift+Enter` for multiline newline, shortcut badge (`gpui_component::Kbd`).

#### 3.3 Markdown & Syntax-Highlighted Code Blocks (`gpui_component::Editor`)
- Render bold, italic, strikethrough, inline code, and blockquotes.
- Fenced code blocks rendered with `gpui_component::Editor` supporting syntax coloring, line numbers, and one-click copy button.
- Spoiler tags (`||hidden||`) with click-to-reveal.

#### 3.4 Attachments, Drag & Drop, and Image Lightbox (`gpui_component::Attachment` & `Dialog`)
- Attachment button with vector icon opening native file dialog (`rfd`).
- Sürükle-bırak (drag-and-drop) dosya yükleme.
- Inline media thumbnail cards with file size, name, and click-to-expand `Dialog` lightbox.

#### 3.5 Message Actions & Reactions (`gpui_component::ContextMenu` & `Popover`)
- Right-click / hover context menu on messages: React, Reply, Edit, Pin, Delete, Copy.
- In-place editing replacing message bubble with `Textarea`.
- Reply preview banner above composer with jump-to-message anchor.
- Reaction pill badges under messages with user list tooltip (`gpui_component::Tooltip`).
- Emoji picker popover (`gpui_component::Popover`).

---

### Phase 4: Direct Messages (DMs), Channel Tree & Command Palette

#### 4.1 Collapsible Channel Tree (`gpui_component::Collapsible`)
- Collapsible categories with smooth animation and persistent open/close state.
- Unread pill badges (`gpui_component::Badge`).
- Vector icons (`#`, speaker) from icon system.

#### 4.2 Quick Switcher Command Palette (`gpui_component::Command`)
- Raycast/Linear-grade search palette on `Ctrl+K`.
- Fuzzy search across text channels, voice channels, and DMs.
- `Kbd` shortcut badges for keyboard navigation (Up, Down, Enter, Esc).

#### 4.3 Direct Messages & User Profile Cards (`gpui_component::Popover`)
- Home button on server rail switches to DM conversation list.
- User directory modal to search and initiate DMs.
- Hover/click user profile popover card with avatar, bio, status, and action buttons.

---

### Phase 5: Administration & User Settings Modals (`gpui_component::Dialog` & `Tabs`)

#### 5.1 Channel & Server Administration
- Create/Edit Channel modals (Text vs Voice).
- Server Settings modal: Overview, Roles & Permissions, Members list with kick/ban actions.

#### 5.2 Comprehensive User Settings Modal (`Tabs`, `Slider`, `Select`, `Switch`)
- **My Account:** Edit username, email, avatar upload.
- **Voice & Video:** Audio input/output device `Select`, volume `Slider` (0% - 200%), mic test live VU meter, Noise Gate `Slider`, VAD threshold `Slider`, PTT keybind recorder.
- **Keybinds:** Global hotkey configuration.
- **Notifications & Switch Server:** `Switch` toggles for desktop alerts and `SessionVault` profile switcher.

#### 5.3 System Tray & Global Hotkeys
- System tray icon via `tray-icon` with Mute/Deafen/Quit menu.
- Global Push-to-Talk hotkey via `global-hotkey` crate.

---

### Phase 6: Mediasoup WebRTC Live Voice & Video Engine (Pure Rust Media Stack)

#### 6.1 Live Voice Audio Streaming (`webrtc-rs` + `cpal`)
- Connect `sharkord_core::voice::VoiceSession` to active voice channels via Mediasoup SFU signaling:
  - Audio capture via `cpal` with Noise Gate DSP and VAD speaking detection.
  - Push-to-Talk audio transmission gating.
  - Transmit and receive Opus audio packets over `webrtc` RTP transports.
  - Remote audio playback for voice participants.

#### 6.2 Screen Sharing & Video Rendering (`xcap` + `openh264`)
- Screen and Window Capture:
  - Use `xcap` for cross-platform desktop/window capture (DXGI, ScreenCaptureKit, PipeWire).
  - Encode frames using `openh264` (or `rusty_h264`) into H.264 bitstreams matching `VideoTrackConfig` (720p - 4K, 15 - 144 FPS).
- Remote Video Decoding & GPU Texture Rendering:
  - Decode incoming H.264 video NAL units from Mediasoup WebRTC transports via `openh264::decoder`.
  - Convert decoded YUV/RGBA frames directly into GPUI GPU textures in `VideoTileView`.
  - Smooth 60/120/144 FPS display with zero-copy texture transfers.

---

## 6. Review and Verification Plan

1. **Compilation & Unit Tests:** `cargo test -p desktop-gpui` and `cargo test --workspace` pass with zero warnings.
2. **Integration Verification:**
   - Run against local dev server (`apps/server`).
   - Verify auto-login from vault, live WebSocket messaging, reactions, editing, attachments, DMs, voice streaming, and screen sharing.
3. **Impeccable Design Quality:** Verify Pure Neutral Dark compliance, WCAG AA contrast, vector icon consistency, skeleton loading states, and smooth 120 FPS rendering.
