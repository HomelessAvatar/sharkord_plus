# Task 9 Report: Message Actions, In-Place Editing, Replies, and Emoji Reactions (`ContextMenu` & `Popover`)

**Date:** 2026-11-22
**Status:** DONE
**Commit:** `fd4099e4 feat(desktop-gpui): implement message ContextMenu, in-place editing, replies, and reaction Popover`

## Summary of Changes

1. **`apps/desktop-gpui/src/ui/emoji_picker.rs`**:
   - Implemented `EmojiItem` with emoji glyph, shortcode name, searchable keywords list, and category assignment.
   - Implemented `EmojiCategory` holding category name, icon, and categorized emoji list.
   - Built comprehensive default categorized emojis spanning Smileys, Gestures, Symbols, and Objects.
   - Implemented `EmojiPickerState`:
     - Quick-pick top row: `["👍", "❤️", "😂", "🎉", "🔥", "🚀"]`.
     - Live search filtering matching emoji character, shortcode name, or any keyword.
     - Category switching and keyboard selection navigation (`move_selection_next`, `move_selection_prev`, `selected_emoji`).
     - Message targeting (`open_for_message`, `close`, `select_emoji`).
   - Implemented `EmojiPickerView`:
     - Pure Neutral Dark theme tokens (`#0A0B0E` base, `#111216` surface, `#16171d` subsurface, `#1c1e24` elevated, `#3B82F6` accent, `#F3F4F6` text).
     - Search header input, quick reactions bar, category tab strip, and scrollable emoji grid (`#emoji_grid_scroll`).

2. **`apps/desktop-gpui/src/ui/chat_canvas.rs`**:
   - Implemented `ReplyTarget` struct with `new(message_id, author_name, snippet)` plus `sender_name` compatibility, supporting tuple conversions.
   - Implemented `format_reactors_tooltip(reactor_names: &[String]) -> String` formatting 0, 1 ("Alice"), 2 ("Alice and Bob"), 3 ("Alice, Bob, and Charlie"), and N ("Alice, Bob, and 2 others").
   - Implemented `ReactionBadge` struct:
     - Emoji, count, `has_reacted` state toggle, and reactor names list.
     - Grouping helper `from_reactions(&msg.reactions, current_user_id, &users)` to transform core `MessageReaction` data into visual reaction pills.
     - Pure Neutral Dark pill styling with accent border when the active user has reacted.
   - Implemented clean vector icons (strictly avoiding raw unicode in UI action buttons):
     - `render_react_icon()`: vector smiley face with eyes and smile arc.
     - `render_reply_icon()`: vector curved reply arrow pointing left.
     - `render_edit_icon()`: vector pencil icon with lead tip.
     - `render_pin_icon(is_pinned)`: vector pushpin icon tinted accent when pinned.
     - `render_trash_icon()`: vector trashcan with lid and bin.
     - `render_copy_icon()`: vector layered clipboard documents.
   - Implemented `MessageActionToolbar` floating hover toolbar:
     - React, Reply, Edit (if owner), Pin/Unpin, Copy, and Delete (if owner).
   - Enhanced `ChatCanvasView`:
     - Added `reply_target: Option<ReplyTarget>`, `editing_message_id: Option<u64>`, `editing_text: String`, `emoji_picker: EmojiPickerState`, `current_user_id: Option<u64>`, `users: HashMap<u64, String>`.
     - In-place editing: when `editing_message_id == Some(id)`, the message body switches to an inline textarea editor with `[Cancel (Esc)]` and `[Save (Enter)]` controls.
     - Reply preview banner: rendered above the composer when `reply_target` is `Some`, displaying `@sender_name: snippet` with a `×` cancel button.
     - Reaction pills: displayed underneath the message body with interactive counter and reactor tooltips.
     - Emoji reaction picker popover: rendered when `emoji_picker.is_open` is true.

3. **`apps/desktop-gpui/src/state.rs`**:
   - Added `edit_message_content(message_id, new_content)` updating message text and timestamp.
   - Added `toggle_message_reaction(message_id, user_id, emoji)` toggling `MessageReaction` entries in the store.
   - Added `toggle_message_pin(message_id)` toggling `pinned` status and `pinned_at`.
   - Added `delete_message(message_id)` removing deleted messages from the store.

4. **`apps/desktop-gpui/src/ui/mod.rs`**:
   - Exported `pub mod emoji_picker;`.
   - Re-exported `EmojiPickerState`, `EmojiPickerView`, `EmojiCategory`, `EmojiItem`, `ReplyTarget`, `ReactionBadge`, `MessageActionToolbar`, and vector icons.
   - Enhanced `WorkspaceView::sync_views` to preserve reply targets, inline editing state, and emoji picker state across updates, while populating `users_map` with all known members.
   - Enhanced `WorkspaceView::send_message` to link `reply_to_message_id` from `reply_target` and clear the reply target upon dispatch.
   - Added `WorkspaceView` coordination methods: `save_inline_edit`, `toggle_reaction`, `toggle_message_pin`, `delete_message`.

5. **`apps/desktop-gpui/tests/message_actions_test.rs`**:
   - Added comprehensive TDD test cases:
     - `test_message_reply_preview_and_editing`: setting and clearing `ReplyTarget`, starting inline editing, updating and committing inline edit, canceling inline editing on Escape.
     - `test_emoji_picker_categories_and_selection`: initial state, quick-reactions list, opening for message ID, search filtering by name and keywords ("fire", "heart"), category selection, selecting emoji and auto-closing.
     - `test_reaction_badges_and_tooltips`: tooltip name formatting (1, 2, 3, and 4+ users), badge counter increments and active-user toggle, aggregating reactions from `Message` data with current user state.
     - `test_message_action_toolbar_and_context_menu`: vector icon generation without errors, toolbar instantiation for owned and pinned messages, canvas element tree rendering with active reply target, inline editor, and reaction pills.

## Verification

- **TDD Red Phase**: `cargo test -p desktop-gpui --test message_actions_test` failed with 24 compilation errors verifying absent interfaces.
- **TDD Green Phase**: `cargo test -p desktop-gpui --test message_actions_test` passed all 4 tests in 0.00s.
- **Desktop-GPUI Test Suite**: `cargo test -p desktop-gpui` passed all 73 tests across 21 test binaries in 0.00s.
- **Workspace Test Suite**: `cargo test --workspace` passed all 117 tests cleanly across all crates (`desktop-gpui`, `sharkord-core`, `sharkord-mobile`, `sharkord-tauri`) with 0 regressions.
