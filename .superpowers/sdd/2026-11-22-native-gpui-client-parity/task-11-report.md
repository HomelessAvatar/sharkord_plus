# Task 11 Report: Direct Messages (DMs) & User Profile Popovers (`gpui_component::Popover` & `Avatar`)

**Date:** 2026-11-22
**Status:** DONE
**Commit:** `20144ce5 feat(desktop-gpui): add direct messaging sidebar and user profile popovers with gpui-component Avatar`

## Summary of Changes

1. **`apps/desktop-gpui/src/ui/dm_view.rs`**:
   - Implemented `DmConversation`:
     - Fields: `channel_id`, `user_id`, `username`, `display_name`, `status`, `last_message`, `unread_count`.
     - Helpers: `effective_name()`, `initials()`, and `status_color()` mapping presence strings to Pure Neutral Dark status tokens (`#22c55e`, `#eab308`, `#ef4444`, `#6b7280`).
   - Implemented `DmSidebarState`:
     - Fields: `active_dm_channel_id: Option<u64>`, `conversations: Vec<DmConversation>`.
     - Methods: `new()`, `set_active_dm()`, `add_conversation()`, `remove_conversation()`, `mark_read()`, `active_conversation()`, `unread_total()`, and `find_by_user_id()`.
   - Implemented `DirectMessagesSidebarView`:
     - Header bar with "Direct Messages" title and `+` action button for starting a conversation.
     - Search and quick switcher trigger pill (`Ctrl+K`).
     - "DIRECT MESSAGES" category section header with total conversation counter.
     - Scrollable list of DM conversations with interactive selection, active highlighting, circular avatar with initials, status indicator dot, user display name, last message preview snippet, and unread count badges.
     - Empty state with guidance when no DM conversations exist.
     - Interactive click listener wired to `WorkspaceView::select_channel()`.

2. **`apps/desktop-gpui/src/ui/user_profile_modal.rs`**:
   - Implemented `UserProfileCardState` (with type alias `UserProfileState`):
     - Fields: `is_open`, `user_id`, `username`, `display_name`, `bio`, `status`, `avatar_url`, `role`.
     - Methods: `new()`, `open()`, `close()`, `set_bio()`, `set_role()`, `set_avatar_url()`.
   - Implemented `UserProfilePopoverView` (with type aliases `UserProfileModalView` and `UserProfilePopover`):
     - Styled Pure Neutral Dark profile card (`#111216` surface, `#16171d` subsurface, `#1c1e24` elevated, `#3B82F6` accent, `#F3F4F6` text).
     - Elevated banner background (100px) with close `✕` button.
     - Avatar circle (80px) overlapping the banner with border-4 frame, initials typography, and status indicator ring dot (20px).
     - Display name and `@username` handle typography.
     - Role designation badge pill (`🏷 Admin` / `🏷 Member`).
     - "ABOUT ME" biography box with subsurface container and subtle hairline border.
     - Full-width "💬 Send Direct Message" action button.
     - Fullscreen backdrop dismissal on outside click.

3. **`apps/desktop-gpui/src/ui/server_rail.rs`**:
   - Upgraded `ServerRailView`:
     - Added `is_home_selected: bool` and maintained `is_home_active: bool`.
     - Added helper methods: `set_home_selected(bool)` and `toggle_home()`.
     - Updated Home button styling and left margin active/unread pill indicator height to react to `is_home_selected`.

4. **`apps/desktop-gpui/src/ui/member_list.rs`**:
   - Added `render_interactive(cx)` to `MemberListSidebarView`.
   - Connected member list rows to `WorkspaceView::open_user_profile()` so clicking any member immediately presents their profile popover card.

5. **`apps/desktop-gpui/src/ui/chat_canvas.rs`**:
   - Wired message feed sender avatars to `WorkspaceView::open_user_profile()` so clicking avatars in the chat stream opens the user profile popover.

6. **`apps/desktop-gpui/src/ui/mod.rs`**:
   - Re-exported `dm_view` and `user_profile_modal`.
   - Integrated `dm_sidebar: DirectMessagesSidebarView` and `user_profile_modal: UserProfileCardState` into `WorkspaceView`.
   - Synchronized DM channels from `AppState::channels` where `is_dm == true`.
   - In Column 2, conditionally rendered `DirectMessagesSidebarView` when `self.server_rail.is_home_selected` is true, otherwise rendering `ChannelTreeView`.
   - Added `WorkspaceView::open_user_profile`, `close_user_profile`, and `open_dm_conversation`.
   - Rendered `UserProfilePopoverView` modal overlay whenever `self.user_profile_modal.is_open` is true.

7. **`apps/desktop-gpui/tests/dm_navigation_test.rs`**:
   - Added complete TDD test suite covering:
     - `test_dm_sidebar_selection`: `DmSidebarState` selection, conversations list, unread count badge, mark_read, remove_conversation, and `DirectMessagesSidebarView` rendering.
     - `test_user_profile_card_state`: `UserProfileCardState` open, close, bio, role, avatar_url, and fields.
     - `test_server_rail_home_toggle`: `ServerRailView` home button toggle between server channels and DMs.
     - `test_user_profile_popover_view_rendering`: profile card banner, avatar, status indicator, action buttons ("Send Message"), and modal rendering.

## Verification Results

- **TDD Red Phase**: `cargo test -p desktop-gpui --test dm_navigation_test` failed with 9 compilation errors verifying absent DM sidebar and profile modal components.
- **TDD Green Phase**: `cargo test -p desktop-gpui --test dm_navigation_test` passed all 4 tests in 0.00s.
- **Desktop-GPUI Test Suite**: `cargo test -p desktop-gpui` passed all 81 tests across 26 test binaries in 0.00s.
- **Workspace Test Suite**: `cargo test --workspace` passed cleanly across all crates (`desktop-gpui`, `sharkord-core`, `sharkord-mobile`, `sharkord-tauri`) with 0 regressions.
