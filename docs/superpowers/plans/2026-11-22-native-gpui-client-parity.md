# Full Native GPUI Desktop Client Feature Parity Implementation Plan (with `gpui-kit` & /impeccable Craft)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver 100% native feature parity in `apps/desktop-gpui` matching the Sharkord Web and Tauri desktop clients using `gpui-kit` (Longbridge's 75+ shadcn-style desktop components for GPUI), `crates/sharkord-core`, and pure Rust media libraries (`webrtc-rs`, `xcap`, `openh264`, `cpal`), achieving sub-50MB RAM idle performance, 120 FPS rendering, and zero webview overhead while adhering to `/impeccable` Operate mode craft standards.

**Architecture:** 
- Shared Rust core (`crates/sharkord-core`) is expanded to support all server tRPC queries/mutations, binary upload streaming, and WebSocket subscription events.
- Desktop GPUI (`apps/desktop-gpui`) integrates a multi-threaded Tokio background runtime communicating with GPUI's thread context via `cx.update(...)`.
- Visual UI is built on `gpui-kit` components (`VirtualList`, `Textarea`, `Dialog`, `ContextMenu`, `Popover`, `Command`, `Collapsible`, `Tabs`, `Slider`, `Select`, `Switch`, `Avatar`, `Badge`, `Tooltip`, `Skeleton`, `Empty`) styled strictly to the `/impeccable` Pure Neutral Dark design system.
- Live WebRTC audio and video streaming connects to Mediasoup SFU via `webrtc-rs`, `cpal`, `xcap` (screen capture), and `openh264` (frame decoding) with zero-copy GPU texture rendering in `VideoTileView`.

**Tech Stack:** Rust 2021 (GPUI 0.2, `gpui-kit 0.10`, `tokio`, `webrtc 0.12`, `xcap 0.5`, `openh264 0.6`, `cpal 0.15`, `rfd 0.15`, `serde`, `serde_json`), Bun + tRPC backend.

**Spec:** [docs/superpowers/specs/2026-11-22-native-gpui-client-parity-design.md](file:///c:/Users/ersan/sharkord_plus/docs/superpowers/specs/2026-11-22-native-gpui-client-parity-design.md)

## Global Constraints & /impeccable Floor

- Zero Turkish on GitHub surfaces (commits, code comments, test names, identifiers, documentation). English only. Direct conversation with user in Turkish.
- Pure Neutral Dark palette (`#0A0B0E` base, `#111216` surface, `#16171d` subsurface, `#1c1e24` elevated, `#3B82F6` accent, `#F3F4F6` text primary).
- Contrast floor: Body and placeholder text ≥4.5:1, large text ≥3:1.
- No unicode emoji characters for UI chrome (replace `👥`, `🔊`, `Send ↵` with consistent vector icons from `gpui_component::Icon`).
- Skeletons over spinners: use `gpui_component::Skeleton` / `Shimmer` for loading states.
- Virtual scrolling: `gpui_component::VirtualList` for message feed ensuring 120 FPS rendering on 10k+ messages.
- Full state vocabulary: every interactive component must define `default`, `hover`, `focus`, `active`, `disabled`, and `loading`.
- Full backwards compatibility with standard Sharkord and Sharkord Plus server tRPC routers.
- No regressions in existing test suites: `cargo test --workspace` must pass with zero warnings at every task gate.

## Review Focus

1. **Session Vault Auto-Login on Launch:** Opening the app with a previously saved valid token must directly enter the workspace without displaying the login form.
2. **Dynamic WebSocket Event Dispatch:** Real-time messages, edits, deletions, reactions, and typing indicators must update GPUI state reactively without requiring manual refresh.
3. **Virtualized Feed & Multiline Composer:** Message stream must use `VirtualList` for 120 FPS performance; composer must use `Textarea` with caret navigation, selection, copy/paste, Shift+Enter newlines, and render syntax-highlighted code blocks with copy button.
4. **Interactive Overlays & Menus:** Context menus on messages and popovers for emoji reactions must escape parent overflow containers cleanly.
5. **Hardware-Accelerated WebRTC Audio/Video:** Connecting to a voice channel must stream live mic input with active VAD speaking detection and decode remote video frames into GPU textures up to 4K @ 144 FPS.

---

### Task 1: Add `gpui-kit` & Desktop Dependencies to `apps/desktop-gpui`

**Files:**
- Modify: `apps/desktop-gpui/Cargo.toml`
- Test: `apps/desktop-gpui/tests/gpui_kit_dependency_test.rs`

**Interfaces:**
- Produces: `gpui_kit::*`, `rfd::FileDialog`, `webrtc::*`, `xcap::*`, `openh264::*`

- [ ] **Step 1: Write the failing test for `gpui-kit` and media dependencies availability**

```rust
// apps/desktop-gpui/tests/gpui_kit_dependency_test.rs
#[test]
fn test_gpui_kit_and_media_crates_importable() {
    assert!(true);
}
```

- [ ] **Step 2: Run test to verify dependencies build**

Run: `cargo test -p desktop-gpui --test gpui_kit_dependency_test`

- [ ] **Step 3: Update `apps/desktop-gpui/Cargo.toml` with required dependencies**

Add `gpui-kit = "0.10"`, `rfd = "0.15"`, `webrtc = "0.12"`, `xcap = "0.5"`, `openh264 = "0.6"`.

- [ ] **Step 4: Run workspace tests to verify clean compilation**

Run: `cargo test -p desktop-gpui`
Expected: PASS (all existing tests pass).

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/Cargo.toml apps/desktop-gpui/tests/gpui_kit_dependency_test.rs
git commit -m "chore(desktop-gpui): add gpui-kit, rfd, and media dependencies"
```

---

### Task 2: Expand `crates/sharkord-core` with Complete tRPC API Procedures & Models

**Files:**
- Modify: `crates/sharkord-core/src/client.rs`
- Modify: `crates/sharkord-core/src/types.rs`
- Test: `crates/sharkord-core/tests/client_trpc_expansion_test.rs`

**Interfaces:**
- Consumes: `reqwest::Client`, `UploadHeaders`
- Produces: 
  - `SharkordClient::toggle_reaction(message_id: u64, emoji: &str) -> Result<(), SharkordError>`
  - `SharkordClient::edit_message(message_id: u64, content: &str) -> Result<Message, SharkordError>`
  - `SharkordClient::delete_message(message_id: u64) -> Result<(), SharkordError>`
  - `SharkordClient::pin_message(message_id: u64, pinned: bool) -> Result<(), SharkordError>`
  - `SharkordClient::create_channel(name: &str, channel_type: ChannelType, category_id: Option<u64>) -> Result<Channel, SharkordError>`
  - `SharkordClient::update_channel(id: u64, name: Option<&str>, topic: Option<&str>) -> Result<Channel, SharkordError>`
  - `SharkordClient::delete_channel(id: u64) -> Result<(), SharkordError>`
  - `SharkordClient::get_dms() -> Result<Vec<Channel>, SharkordError>`
  - `SharkordClient::open_dm(target_user_id: u64) -> Result<Channel, SharkordError>`
  - `SharkordClient::get_members() -> Result<Vec<User>, SharkordError>`
  - `SharkordClient::upload_file(filename: &str, data: Vec<u8>) -> Result<u64, SharkordError>`

- [ ] **Step 1: Write the failing tests for new tRPC procedures**

```rust
// crates/sharkord-core/tests/client_trpc_expansion_test.rs
use sharkord_core::client::SharkordClient;

#[test]
fn test_client_has_expansion_methods() {
    let client = SharkordClient::new("http://localhost:4991").unwrap();
    assert_eq!(client.base_url(), "http://localhost:4991");
}
```

- [ ] **Step 2: Run test to verify failures**

Run: `cargo test -p sharkord-core --test client_trpc_expansion_test`

- [ ] **Step 3: Implement new tRPC methods and models in `crates/sharkord-core/src/client.rs`**

Add procedures for reactions, message mutations, channel CRUD, DMs, members, and raw HTTP binary upload streaming matching server endpoints in `apps/server/src/routers/`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sharkord-core`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sharkord-core/src/client.rs crates/sharkord-core/src/types.rs crates/sharkord-core/tests/client_trpc_expansion_test.rs
git commit -m "feat(core): expand tRPC client with reactions, channels, DMs, and upload procedures"
```

---

### Task 3: Expand `crates/sharkord-core` WebSocket Event Stream & Protocol Decoders

**Files:**
- Modify: `crates/sharkord-core/src/protocol/events.rs`
- Test: `crates/sharkord-core/tests/events_expansion_test.rs`

**Interfaces:**
- Produces: `EventType::{ReactionAdded, ReactionRemoved, ChannelCreated, ChannelUpdated, ChannelDeleted, UserPresenceChanged, UserTyping}`

- [ ] **Step 1: Write failing tests for expanded WebSocket events**

```rust
// crates/sharkord-core/tests/events_expansion_test.rs
use sharkord_core::protocol::events::{EventType, decode_websocket_frame};

#[test]
fn test_decode_reaction_and_channel_events() {
    let raw = r#"{"type":"reactionAdded","data":{"messageId":1,"emoji":"👍","userId":2}}"#;
    let evt = decode_websocket_frame(raw).unwrap();
    assert_eq!(evt.event_type, EventType::ReactionAdded);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sharkord-core --test events_expansion_test`

- [ ] **Step 3: Implement new variants in `crates/sharkord-core/src/protocol/events.rs`**

Add `ReactionAdded`, `ReactionRemoved`, `ChannelCreated`, `ChannelUpdated`, `ChannelDeleted`, `UserPresenceChanged`, and `UserTyping` variants with serde mappings and string conversions.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sharkord-core --test events_expansion_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sharkord-core/src/protocol/events.rs crates/sharkord-core/tests/events_expansion_test.rs
git commit -m "feat(core): add reaction, channel, and typing WebSocket event decoders"
```

---

### Task 4: Tokio Background Runtime Bridge & SessionVault Auto-Login in `desktop-gpui`

**Files:**
- Modify: `apps/desktop-gpui/src/main.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Create: `apps/desktop-gpui/src/network/mod.rs`
- Create: `apps/desktop-gpui/src/network/bridge.rs`
- Test: `apps/desktop-gpui/tests/network_bridge_test.rs`

**Interfaces:**
- Consumes: `sharkord_core::client::SharkordClient`, `sharkord_core::vault::SessionVault`
- Produces: `NetworkBridge`, `AppState::bootstrap_from_server`, `AppState::is_connecting`

- [ ] **Step 1: Write failing test for network bridge and vault auto-login**

```rust
// apps/desktop-gpui/tests/network_bridge_test.rs
use desktop_gpui::state::AppState;
use sharkord_core::vault::SavedServerSession;

#[test]
fn test_app_state_auto_login_from_vault() {
    let mut state = AppState::new();
    let session = SavedServerSession {
        server_id: "test".to_string(),
        server_name: "Test Server".to_string(),
        server_url: "http://localhost:4991".to_string(),
        token: "mock_token".to_string(),
        user_id: 1,
        username: "admin".to_string(),
        display_name: "Admin".to_string(),
        avatar_url: None,
        last_accessed_at: 1000,
    };
    state.apply_saved_session(&session);
    assert!(state.current_user.is_some());
    assert_eq!(state.current_user.unwrap().name, "admin");
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test network_bridge_test`

- [ ] **Step 3: Implement `NetworkBridge` and `AppState::apply_saved_session`**

Initialize multi-threaded Tokio runtime alongside GPUI; implement `SessionVault::load()` check on startup; if session exists, auto-bootstrap `AppState`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test network_bridge_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/network/ apps/desktop-gpui/src/state.rs apps/desktop-gpui/src/main.rs apps/desktop-gpui/tests/network_bridge_test.rs
git commit -m "feat(desktop-gpui): implement Tokio async bridge and SessionVault auto-login"
```

---

### Task 5: Server Connect, Login & Register Views (`gpui_component::Dialog`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/auth_modal.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/auth_modal_test.rs`

**Interfaces:**
- Consumes: `gpui_component::Dialog`, `gpui_component::Input`, `gpui_component::Button`, `NetworkBridge::login`
- Produces: `AuthModalView`, `ServerConnectView`, `AppState::is_auth_modal_open`

- [ ] **Step 1: Write failing test for auth modal state transitions**

```rust
// apps/desktop-gpui/tests/auth_modal_test.rs
use desktop_gpui::ui::auth_modal::{AuthModalState, AuthTab};

#[test]
fn test_auth_modal_tabs_and_error_handling() {
    let mut state = AuthModalState::new();
    assert_eq!(state.active_tab, AuthTab::Login);
    state.set_tab(AuthTab::Register);
    assert_eq!(state.active_tab, AuthTab::Register);
    state.set_error("Invalid credentials");
    assert_eq!(state.error_message.as_deref(), Some("Invalid credentials"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test auth_modal_test`

- [ ] **Step 3: Implement `AuthModalView` using `gpui_component::Dialog`**

Render server URL connection input, Login tab, Register tab, error banner, and remember session toggle adhering strictly to Pure Neutral Dark.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test auth_modal_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/auth_modal.rs apps/desktop-gpui/src/ui/mod.rs apps/desktop-gpui/tests/auth_modal_test.rs
git commit -m "feat(desktop-gpui): add server connect and authentication modal views with gpui-component Dialog"
```

---

### Task 6: Real-time WebSocket Event Subscription Dispatcher in `desktop-gpui`

**Files:**
- Modify: `apps/desktop-gpui/src/network/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/websocket_dispatch_test.rs`

**Interfaces:**
- Consumes: `sharkord_core::protocol::events::SubscriptionEvent`
- Produces: `AppState::handle_subscription_event`

- [ ] **Step 1: Write failing test for WebSocket event dispatching**

```rust
// apps/desktop-gpui/tests/websocket_dispatch_test.rs
use desktop_gpui::state::AppState;
use sharkord_core::protocol::events::{EventType, SubscriptionEvent};

#[test]
fn test_dispatch_message_created_event() {
    let mut state = AppState::new();
    state.active_channel_id = Some(1);
    let event = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageCreated,
        data: serde_json::json!({
            "id": 42,
            "channelId": 1,
            "content": "Live WebSocket message",
            "userId": 2,
            "createdAt": 1000
        }),
    };
    state.handle_subscription_event(event);
    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].id, 42);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test websocket_dispatch_test`

- [ ] **Step 3: Implement `AppState::handle_subscription_event`**

Handle `MessageCreated`, `MessageUpdated`, `MessageDeleted`, `ReactionAdded`, `ReactionRemoved`, `ChannelCreated`, `UserPresenceChanged`, and `UserTyping`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test websocket_dispatch_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/network/ apps/desktop-gpui/src/state.rs apps/desktop-gpui/tests/websocket_dispatch_test.rs
git commit -m "feat(desktop-gpui): implement reactive WebSocket event subscription dispatcher"
```

---

### Task 7: Virtualized Feed & Rich Interactive Composer (`VirtualList`, `Textarea`, `Editor`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/markdown.rs`
- Modify: `apps/desktop-gpui/src/ui/chat_canvas.rs`
- Test: `apps/desktop-gpui/tests/markdown_render_test.rs`

**Interfaces:**
- Consumes: `gpui_component::VirtualList`, `gpui_component::Textarea`, `gpui_component::Editor`, `gpui_component::Skeleton`, `gpui_component::Kbd`
- Produces: `render_markdown_element(text: &str) -> AnyElement`, `ChatCanvasView`

- [ ] **Step 1: Write failing tests for virtualized feed and markdown code blocks**

```rust
// apps/desktop-gpui/tests/markdown_render_test.rs
use desktop_gpui::ui::markdown::{parse_markdown_tokens, MarkdownToken};

#[test]
fn test_parse_markdown_bold_codeblock_spoiler() {
    let input = "**bold text** and `inline code` and ||spoiler||";
    let tokens = parse_markdown_tokens(input);
    assert!(tokens.contains(&MarkdownToken::Bold("bold text".to_string())));
    assert!(tokens.contains(&MarkdownToken::InlineCode("inline code".to_string())));
    assert!(tokens.contains(&MarkdownToken::Spoiler("spoiler".to_string())));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test markdown_render_test`

- [ ] **Step 3: Implement `markdown.rs` and integrate `gpui_component::VirtualList` & `Textarea` in `chat_canvas.rs`**

- Use `gpui_component::VirtualList` for 120 FPS message streaming.
- Use `gpui_component::Skeleton` for initial loading state.
- Wire `gpui_component::Textarea` with caret navigation, selection, clipboard, Shift+Enter newline, and `Kbd` badges.
- Render code blocks via `gpui_component::Editor` with syntax coloring and one-click copy button. Replace unicode icons with vector `Icon`s.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test markdown_render_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/markdown.rs apps/desktop-gpui/src/ui/chat_canvas.rs apps/desktop-gpui/tests/markdown_render_test.rs
git commit -m "feat(desktop-gpui): implement VirtualList feed, gpui-component Textarea, and Editor code blocks"
```

---

### Task 8: File Uploads, Drag-and-Drop, and Image Lightbox Modal (`gpui_component::Attachment` & `Dialog`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/lightbox_modal.rs`
- Modify: `apps/desktop-gpui/src/ui/chat_canvas.rs`
- Test: `apps/desktop-gpui/tests/attachment_upload_test.rs`

**Interfaces:**
- Consumes: `rfd::FileDialog`, `SharkordClient::upload_file`, `gpui_component::Attachment`, `gpui_component::Dialog`
- Produces: `LightboxModalView`, `ChatCanvasView::handle_file_drop`

- [x] **Step 1: Write failing test for attachment and lightbox state**

```rust
// apps/desktop-gpui/tests/attachment_upload_test.rs
use desktop_gpui::ui::lightbox_modal::LightboxState;

#[test]
fn test_lightbox_open_and_close() {
    let mut lightbox = LightboxState::new();
    assert!(!lightbox.is_open);
    lightbox.open("http://localhost:4991/uploads/test.png");
    assert!(lightbox.is_open);
    assert_eq!(lightbox.image_url.as_deref(), Some("http://localhost:4991/uploads/test.png"));
}
```

- [x] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test attachment_upload_test`

- [x] **Step 3: Implement native file dialog upload and image lightbox**

Add `+` vector button opening `rfd::FileDialog`, upload file via `SharkordClient::upload_file`, render `gpui_component::Attachment` surface, and display `gpui_component::Dialog` lightbox on click.

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test attachment_upload_test`
Expected: PASS

- [x] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/lightbox_modal.rs apps/desktop-gpui/src/ui/chat_canvas.rs apps/desktop-gpui/tests/attachment_upload_test.rs
git commit -m "feat(desktop-gpui): add file upload attachment surface and image lightbox Dialog"
```

---

### Task 9: Message Actions, In-Place Editing, Replies, and Emoji Reactions (`ContextMenu` & `Popover`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/emoji_picker.rs`
- Modify: `apps/desktop-gpui/src/ui/chat_canvas.rs`
- Test: `apps/desktop-gpui/tests/message_actions_test.rs`

**Interfaces:**
- Consumes: `gpui_component::ContextMenu`, `gpui_component::Popover`, `gpui_component::Tooltip`, `gpui_component::Badge`
- Produces: `EmojiPickerView`, `MessageActionToolbar`

- [ ] **Step 1: Write failing test for emoji reaction toggling and reply preview**

```rust
// apps/desktop-gpui/tests/message_actions_test.rs
use desktop_gpui::ui::chat_canvas::ChatCanvasView;

#[test]
fn test_message_reply_preview_and_editing() {
    let mut view = ChatCanvasView::default();
    view.set_reply_target(Some(10, "Hello world", "Alice"));
    assert!(view.reply_target.is_some());
    assert_eq!(view.reply_target.as_ref().unwrap().sender_name, "Alice");
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test message_actions_test`

- [ ] **Step 3: Implement message action bar, in-place editing, replies, and emoji picker popover**

Wire `gpui_component::ContextMenu` (Edit, Reply, Pin, Delete, React), inline message editing, reaction pills with `gpui_component::Tooltip` showing reactors list, and emoji reaction picker popover (`gpui_component::Popover`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test message_actions_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/emoji_picker.rs apps/desktop-gpui/src/ui/chat_canvas.rs apps/desktop-gpui/tests/message_actions_test.rs
git commit -m "feat(desktop-gpui): implement message ContextMenu, in-place editing, replies, and reaction Popover"
```

---

### Task 10: Collapsible Channel Tree & Quick Switcher (`Collapsible` & `Command`)

**Files:**
- Modify: `apps/desktop-gpui/src/ui/channel_tree.rs`
- Modify: `apps/desktop-gpui/src/ui/quick_switcher.rs`
- Test: `apps/desktop-gpui/tests/channel_tree_collapsible_test.rs`

**Interfaces:**
- Consumes: `gpui_component::Collapsible`, `gpui_component::Command`, `gpui_component::Kbd`, `gpui_component::Badge`
- Produces: `ChannelTreeView`, `QuickSwitcherView`

- [ ] **Step 1: Write failing test for category collapse and command palette**

```rust
// apps/desktop-gpui/tests/channel_tree_collapsible_test.rs
use desktop_gpui::ui::channel_tree::CategorizedChannelGroup;

#[test]
fn test_category_collapsible_state() {
    let mut group = CategorizedChannelGroup::new("TEXT CHANNELS");
    assert!(group.is_open);
    group.toggle();
    assert!(!group.is_open);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test channel_tree_collapsible_test`

- [ ] **Step 3: Upgrade Channel Tree with `Collapsible` and Quick Switcher with `Command`**

- Use `gpui_component::Collapsible` for channel categories with persistent state and unread badges.
- Upgrade `QuickSwitcher` to use `gpui_component::Command` palette with keyboard navigation, fuzzy filtering, and `Kbd` shortcut badges.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test channel_tree_collapsible_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/channel_tree.rs apps/desktop-gpui/src/ui/quick_switcher.rs apps/desktop-gpui/tests/channel_tree_collapsible_test.rs
git commit -m "feat(desktop-gpui): integrate gpui-component Collapsible categories and Command palette"
```

---

### Task 11: Direct Messages (DMs) & User Profile Popovers (`gpui_component::Popover` & `Avatar`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/dm_view.rs`
- Create: `apps/desktop-gpui/src/ui/user_profile_modal.rs`
- Modify: `apps/desktop-gpui/src/ui/server_rail.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/dm_navigation_test.rs`

**Interfaces:**
- Consumes: `gpui_component::Avatar`, `gpui_component::Popover`, `SharkordClient::get_dms`, `SharkordClient::open_dm`
- Produces: `DirectMessagesSidebarView`, `UserProfilePopover`

- [ ] **Step 1: Write failing test for DM view navigation and profile modal**

```rust
// apps/desktop-gpui/tests/dm_navigation_test.rs
use desktop_gpui::ui::dm_view::DmSidebarState;

#[test]
fn test_dm_sidebar_selection() {
    let mut state = DmSidebarState::new();
    state.set_active_dm(Some(5));
    assert_eq!(state.active_dm_channel_id, Some(5));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test dm_navigation_test`

- [ ] **Step 3: Implement DM sidebar view and user profile popover card**

Clicking Home on server rail shows DM list and user directory; clicking any avatar displays `gpui_component::Popover` card with avatar, online status indicator ring, and "Send DM" button.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test dm_navigation_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/dm_view.rs apps/desktop-gpui/src/ui/user_profile_modal.rs apps/desktop-gpui/src/ui/server_rail.rs apps/desktop-gpui/src/ui/mod.rs apps/desktop-gpui/tests/dm_navigation_test.rs
git commit -m "feat(desktop-gpui): add direct messaging sidebar and user profile popovers with gpui-component Avatar"
```

---

### Task 12: Channel & Server Administration Modals (`gpui_component::Dialog` & `Select`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/channel_admin_modal.rs`
- Create: `apps/desktop-gpui/src/ui/server_settings_modal.rs`
- Modify: `apps/desktop-gpui/src/ui/channel_tree.rs`
- Test: `apps/desktop-gpui/tests/admin_modals_test.rs`

**Interfaces:**
- Consumes: `SharkordClient::create_channel`, `SharkordClient::delete_channel`, `SharkordClient::get_roles`, `gpui_component::Dialog`, `gpui_component::Select`
- Produces: `CreateChannelModalView`, `ServerSettingsModalView`

- [ ] **Step 1: Write failing test for channel creation and server settings state**

```rust
// apps/desktop-gpui/tests/admin_modals_test.rs
use desktop_gpui::ui::channel_admin_modal::CreateChannelModalState;
use sharkord_core::types::ChannelType;

#[test]
fn test_create_channel_modal_type_selection() {
    let mut state = CreateChannelModalState::new();
    state.set_channel_type(ChannelType::Voice);
    assert_eq!(state.channel_type, ChannelType::Voice);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test admin_modals_test`

- [ ] **Step 3: Implement channel creation/edit modals and server settings modal**

Build modals using `gpui_component::Dialog` and `Select` for Text vs Voice channel creation, renaming, deletion, and server settings tabs (Overview, Roles, Members).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test admin_modals_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/channel_admin_modal.rs apps/desktop-gpui/src/ui/server_settings_modal.rs apps/desktop-gpui/src/ui/channel_tree.rs apps/desktop-gpui/tests/admin_modals_test.rs
git commit -m "feat(desktop-gpui): add channel administration and server settings Dialogs"
```

---

### Task 13: Comprehensive User Settings Modal & System Tray (`Tabs`, `Slider`, `Select`, `Switch`)

**Files:**
- Create: `apps/desktop-gpui/src/ui/user_settings_modal.rs`
- Create: `apps/desktop-gpui/src/tray.rs`
- Modify: `apps/desktop-gpui/src/main.rs`
- Test: `apps/desktop-gpui/tests/user_settings_modal_test.rs`

**Interfaces:**
- Consumes: `gpui_component::Dialog`, `gpui_component::Tabs`, `gpui_component::Slider`, `gpui_component::Select`, `gpui_component::Switch`, `tray-icon`
- Produces: `UserSettingsModalView`, `DesktopTray`

- [ ] **Step 1: Write failing test for user settings tabs and tray state**

```rust
// apps/desktop-gpui/tests/user_settings_modal_test.rs
use desktop_gpui::ui::user_settings_modal::{UserSettingsState, SettingsTab};

#[test]
fn test_user_settings_tab_navigation() {
    let mut state = UserSettingsState::new();
    assert_eq!(state.active_tab, SettingsTab::Account);
    state.set_tab(SettingsTab::VoiceAndVideo);
    assert_eq!(state.active_tab, SettingsTab::VoiceAndVideo);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test user_settings_modal_test`

- [ ] **Step 3: Implement tabbed User Settings modal and system tray**

Tabs for Account, Voice & Video (input/output device `Select`, volume `Slider`, VU meter, noise gate `Slider`, VAD threshold `Slider`, PTT keybind recorder), Keybinds, and `Switch` toggles. Add system tray icon with Mute/Deafen/Quit menu.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test user_settings_modal_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/user_settings_modal.rs apps/desktop-gpui/src/tray.rs apps/desktop-gpui/src/main.rs apps/desktop-gpui/tests/user_settings_modal_test.rs
git commit -m "feat(desktop-gpui): implement User Settings Dialog with gpui-component Tabs, Sliders, and system tray"
```

---

### Task 14: Live Mediasoup WebRTC Audio & Voice Stage (`webrtc-rs` + `cpal`)

**Files:**
- Create: `apps/desktop-gpui/src/media/mod.rs`
- Create: `apps/desktop-gpui/src/media/voice_client.rs`
- Modify: `apps/desktop-gpui/src/ui/voice_bar.rs`
- Modify: `apps/desktop-gpui/src/ui/voice_stage.rs`
- Test: `apps/desktop-gpui/tests/live_voice_test.rs`

**Interfaces:**
- Consumes: `webrtc::*`, `sharkord_core::voice::*`, `cpal`
- Produces: `LiveVoiceClient::connect`, `LiveVoiceClient::disconnect`

- [ ] **Step 1: Write failing test for live voice client state and audio levels**

```rust
// apps/desktop-gpui/tests/live_voice_test.rs
use desktop_gpui::media::voice_client::LiveVoiceState;

#[test]
fn test_live_voice_state_transitions() {
    let mut voice = LiveVoiceState::new();
    assert!(!voice.is_connected);
    voice.set_speaking(true, 0.85);
    assert!(voice.is_speaking);
    assert_eq!(voice.audio_level, 0.85);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test live_voice_test`

- [ ] **Step 3: Implement `LiveVoiceClient` bridging `cpal` audio and `webrtc` RTP transports**

Stream microphone input through noise gate and VAD; playback incoming participant audio; update green speaking ring dynamically; bind Push-to-Talk audio gating and global hotkey.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test live_voice_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/media/ apps/desktop-gpui/src/ui/voice_bar.rs apps/desktop-gpui/src/ui/voice_stage.rs apps/desktop-gpui/tests/live_voice_test.rs
git commit -m "feat(desktop-gpui): connect live Mediasoup WebRTC voice engine with VAD and PTT"
```

---

### Task 15: Screen Sharing Capture & Video Texture Rendering (`xcap` + `openh264`)

**Files:**
- Create: `apps/desktop-gpui/src/media/screen_capture.rs`
- Create: `apps/desktop-gpui/src/media/video_decoder.rs`
- Modify: `apps/desktop-gpui/src/ui/video_tile.rs`
- Modify: `apps/desktop-gpui/src/ui/source_picker.rs`
- Test: `apps/desktop-gpui/tests/video_pipeline_test.rs`

**Interfaces:**
- Consumes: `xcap::Monitor`, `xcap::Window`, `openh264::decoder::Decoder`
- Produces: `ScreenCaptureEngine`, `VideoDecoderPipeline`, `VideoTileView::render_frame`

- [ ] **Step 1: Write failing test for screen capture enumeration and frame decoding**

```rust
// apps/desktop-gpui/tests/video_pipeline_test.rs
use desktop_gpui::media::video_decoder::VideoDecoderPipeline;

#[test]
fn test_video_decoder_initialization() {
    let decoder = VideoDecoderPipeline::new();
    assert!(decoder.is_ok());
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p desktop-gpui --test video_pipeline_test`

- [ ] **Step 3: Implement `xcap` capture engine and `openh264` GPU texture upload**

Enumerate monitors/windows in `SourcePickerView`; capture frames via DXGI/ScreenCaptureKit; decode incoming H.264 video NAL units and upload into GPUI GPU textures in `VideoTileView`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui --test video_pipeline_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/media/screen_capture.rs apps/desktop-gpui/src/media/video_decoder.rs apps/desktop-gpui/src/ui/video_tile.rs apps/desktop-gpui/src/ui/source_picker.rs apps/desktop-gpui/tests/video_pipeline_test.rs
git commit -m "feat(desktop-gpui): implement screen sharing capture and GPU video texture rendering"
```

---

### Task 16: Full Integration Verification & /impeccable Craft Inspection Pass

**Files:**
- Test: `apps/desktop-gpui/tests/e2e_integration_test.rs`
- Verify: `DEVELOPMENT.md`

- [ ] **Step 1: Run complete workspace test suite**

Run: `cargo test --workspace`
Expected: All tests in `sharkord-core` and `desktop-gpui` pass cleanly with zero warnings.

- [ ] **Step 2: Run end-to-end local dev sanity test**

Launch `apps/server` (`bun dev`) and verify `cargo run -p desktop-gpui` authenticates, receives WebSocket events, sends messages, displays attachments, connects to voice, and renders smoothly at 120 FPS.

- [ ] **Step 3: Commit final integration pass**

```bash
git commit --allow-empty -m "chore(desktop-gpui): verify full client parity and impeccable craft across workspace"
```
