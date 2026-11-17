# Desktop UX Polish & Discord Parity Views Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the 4 identified desktop UX polish items: dynamic center view switching (Chat vs Voice Stage vs Video Tile), right-hand member list sidebar drawer, Discord-grade empty states/welcome cards, and a `Ctrl+K` Quick Switcher modal.

**Architecture:** Extend `apps/desktop-gpui` with `ActiveContentView` in `WorkspaceView`, a new 4th column `MemberListSidebarView` (`member_list.rs`), rich welcoming empty state elements in `chat_canvas.rs` and `voice_stage.rs`, and an overlay `QuickSwitcherModal` (`quick_switcher.rs`). All elements adhere strictly to Pure Neutral Dark tokens.

**Tech Stack:** Rust 2021, GPUI, Pure Neutral Dark design tokens (`#0a0b0e`, `#111216`, `#16171d`, `#1c1e24`, `#3b82f6`, `#22c55e`).

**Spec:** `PRODUCT.md`, `DESIGN.md`, and `impeccable-voice-video-review.md`.

## Global Constraints
- Zero Turkish text in code comments, test names, identifiers, commit messages, and documentation files. Direct communication with the user is in Turkish.
- Maintain 100% test pass rate across the workspace with zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
- Pure Neutral Dark palette: background base `#0a0b0e`, surface `#111216`, subsurface `#16171d`, elevated `#1c1e24`, accent `#3b82f6`, online `#22c55e`.
- Maintain backwards compatibility with all existing tests.

## Review Focus
- Center view switching preserves chat history and does not reset message drafts.
- Right member list properly groups users by online vs offline without duplicate cards.
- Quick Switcher query filtering handles empty input, case-insensitivity, and channels of both Text and Voice types.
- Empty state rendering does not interfere with normal message feeds when messages exist.

---

### Task 1: Dynamic Center View Mode Switcher in WorkspaceView

**Files:**
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/view_switcher_test.rs`

**Interfaces:**
- Consumes: `AppState`, `ChatCanvasView`, `VoiceStageView`, `VideoTileView`
- Produces: `ActiveContentView`, `WorkspaceView::set_center_view`, `WorkspaceView::active_center_view`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/view_switcher_test.rs`**

```rust
use desktop_gpui::state::AppState;
use desktop_gpui::ui::{ActiveContentView, WorkspaceView};
use sharkord_core::types::{Channel, ChannelType};

#[test]
fn test_workspace_center_view_switching() {
    let mut state = AppState::new();
    state.set_channels(vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            topic: None,
            private: false,
            is_dm: false,
            position: 0,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
        Channel {
            id: 2,
            name: "lounge-voice".to_string(),
            channel_type: ChannelType::Voice,
            topic: None,
            private: false,
            is_dm: false,
            position: 1,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
    ]);

    let mut workspace = WorkspaceView::new(state);

    // Default view for text channel is Chat
    workspace.select_channel(1);
    assert_eq!(workspace.active_center_view, ActiveContentView::Chat);

    // Selecting a voice channel automatically switches center view to VoiceStage
    workspace.select_channel(2);
    assert_eq!(workspace.active_center_view, ActiveContentView::VoiceStage);

    // User can manually toggle center view mode
    workspace.set_center_view(ActiveContentView::Chat);
    assert_eq!(workspace.active_center_view, ActiveContentView::Chat);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test view_switcher_test`
Expected: FAIL with unresolved `ActiveContentView`

- [ ] **Step 3: Implement `ActiveContentView` and view switching in `apps/desktop-gpui/src/ui/mod.rs`**

Define `ActiveContentView` enum (`Chat`, `VoiceStage`, `ScreenShare`). Add `pub active_center_view: ActiveContentView` to `WorkspaceView`. In `select_channel`, check channel type: if `ChannelType::Voice`, switch to `ActiveContentView::VoiceStage`, else `ActiveContentView::Chat`. In `render`, render the chosen view in Column 3.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test view_switcher_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement dynamic center view switcher between chat and voice stage"
```

---

### Task 2: Right-hand Member List Sidebar Drawer

**Files:**
- Create: `apps/desktop-gpui/src/ui/member_list.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/member_list_test.rs`

**Interfaces:**
- Consumes: `User`, `AppState`
- Produces: `MemberListSidebarView`, `CategorizedMemberGroup`, `compose_member_groups`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/member_list_test.rs`**

```rust
use desktop_gpui::ui::member_list::{compose_member_groups, MemberListSidebarView};
use sharkord_core::types::User;

#[test]
fn test_member_list_grouping_and_view() {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            status: Some("online".to_string()),
            ..Default::default()
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            status: Some("offline".to_string()),
            ..Default::default()
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            status: Some("dnd".to_string()),
            ..Default::default()
        },
    ];

    let groups = compose_member_groups(&users);
    assert_eq!(groups.len(), 2); // ONLINE (Alice, Charlie) and OFFLINE (Bob)

    let online_group = groups.iter().find(|g| g.title.starts_with("ONLINE")).unwrap();
    assert_eq!(online_group.members.len(), 2);

    let offline_group = groups.iter().find(|g| g.title.starts_with("OFFLINE")).unwrap();
    assert_eq!(offline_group.members.len(), 1);

    let view = MemberListSidebarView::new(&users);
    assert_eq!(view.total_members_count(), 3);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test member_list_test`
Expected: FAIL with unresolved module `member_list`

- [ ] **Step 3: Implement `apps/desktop-gpui/src/ui/member_list.rs` and wire into `WorkspaceView`**

Implement `MemberListSidebarView` (240px fixed, `bg_subsurface`, border-l hairline). Group users into Online (`online`, `idle`, `dnd`) and Offline (`offline` / None). Render avatars, status dots (`theme::status_online`, `theme::status_dnd`, etc.), and usernames. Wire into `WorkspaceView` as Column 4 conditionally rendered when `self.state.right_sidebar_open` is true. Export in `apps/desktop-gpui/src/ui/mod.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test member_list_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement right-hand member list sidebar drawer with status groups"
```

---

### Task 3: Discord-Grade Empty States & Channel Welcoming Cards

**Files:**
- Modify: `apps/desktop-gpui/src/ui/chat_canvas.rs`
- Modify: `apps/desktop-gpui/src/ui/voice_stage.rs`
- Test: `apps/desktop-gpui/tests/empty_states_test.rs`

**Interfaces:**
- Consumes: `ChatCanvasView`, `VoiceStageView`
- Produces: `render_channel_welcome_card`, `render_empty_stage_placeholder`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/empty_states_test.rs`**

```rust
use desktop_gpui::ui::chat_canvas::render_channel_welcome_card;
use desktop_gpui::ui::voice_stage::render_empty_stage_placeholder;

#[test]
fn test_empty_states_rendering_functions() {
    let welcome = render_channel_welcome_card("general", Some("General discussion topic"));
    assert!(!welcome.channel_name.is_empty());
    assert_eq!(welcome.channel_name, "general");
    assert_eq!(welcome.title_text(), "Welcome to #general!");

    let stage_placeholder = render_empty_stage_placeholder("Voice Lounge");
    assert_eq!(stage_placeholder.channel_name, "Voice Lounge");
    assert_eq!(stage_placeholder.headline(), "It's quiet in here...");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test empty_states_test`
Expected: FAIL with unresolved functions

- [ ] **Step 3: Implement welcoming card and empty stage card**

In `chat_canvas.rs`, if `channel_messages` is empty, render the Discord-style Welcome Card at the top with large `#` circle, bold channel name, and explanatory helper text. In `voice_stage.rs`, if `participants` is empty, render a centered stage illustration card with "It's quiet in here... Share an invite link or invite friends to join!".

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test empty_states_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): add discord-grade welcome cards and empty stage states"
```

---

### Task 4: Quick Switcher (`Ctrl+K`) Modal

**Files:**
- Create: `apps/desktop-gpui/src/ui/quick_switcher.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/quick_switcher_test.rs`

**Interfaces:**
- Consumes: `Channel`, `AppState`
- Produces: `QuickSwitcherState`, `QuickSwitcherView`, `QuickSwitcherItem`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/quick_switcher_test.rs`**

```rust
use desktop_gpui::ui::quick_switcher::{QuickSwitcherItem, QuickSwitcherState};
use sharkord_core::types::{Channel, ChannelType};

#[test]
fn test_quick_switcher_filtering_and_navigation() {
    let channels = vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 2,
            name: "announcements".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 3,
            name: "voice-lounge".to_string(),
            channel_type: ChannelType::Voice,
            ..Default::default()
        },
    ];

    let mut switcher = QuickSwitcherState::new(&channels);
    assert_eq!(switcher.filtered_items().len(), 3);
    assert_eq!(switcher.selected_index, 0);

    // Search query filtering
    switcher.set_query("voice");
    let filtered = switcher.filtered_items();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "voice-lounge");

    // Index navigation
    switcher.set_query("");
    switcher.select_next();
    assert_eq!(switcher.selected_index, 1);
    switcher.select_prev();
    assert_eq!(switcher.selected_index, 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test quick_switcher_test`
Expected: FAIL with unresolved module `quick_switcher`

- [ ] **Step 3: Implement `apps/desktop-gpui/src/ui/quick_switcher.rs` and wire into `WorkspaceView`**

Implement Raycast/Linear-style `QuickSwitcherModal` (560px wide, centered modal overlay) with search input (`bg_elevated`, search icon), icon indicators (`#` for text, `🔊` for voice), keyboard selection highlight (`theme::bg_surface_4`), and Enter selection. Wire open/close state into `AppState` (`is_quick_switcher_open: bool`, `toggle_quick_switcher()`).

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test quick_switcher_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement raycast-style ctrl+k quick switcher modal"
```

---

### Task 5: End-to-End Test Suite & Verification

**Files:**
- Modify: `ROADMAP.md`

- [ ] **Step 1: Run complete cargo test suite across all crates**
- [ ] **Step 2: Run formatting and clippy checks**
- [ ] **Step 3: Run monorepo type checks and linting (`bun run magic`)**
- [ ] **Step 4: Update roadmap and commit**
