//! Chat Canvas view component (Flex 1).
//! Renders the channel header with topic and member toggle, scrollable message stream
//! with sender avatars and timestamps (grouped within 5-minute windows), and docked bottom
//! composer input bar. Adheres strictly to the Pure Neutral Dark design specification.

use std::collections::HashMap;
use std::path::PathBuf;

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::{InteractiveComponent, Message, MessageFile, PluginCommand};

use crate::theme;
use crate::ui::emoji_picker::{EmojiPickerState, EmojiPickerView};
use crate::ui::lightbox_modal::{LightboxModalView, LightboxState};
use crate::ui::markdown::render_markdown_element;
use crate::ui::server_rail::server_initials;

/// Action produced by keystroke evaluation in the composer textarea.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComposerAction {
    None,
    Send,
    Newline,
    InputUpdated,
}

/// Rich multiline text composer state with cursor navigation, selection, and clipboard support.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComposerTextarea {
    pub text: String,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    pub is_focused: bool,
}

impl ComposerTextarea {
    /// Creates an empty composer textarea.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a composer textarea initialized with the given text.
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self {
            text,
            cursor,
            selection: None,
            is_focused: false,
        }
    }

    /// Inserts text at the current cursor position, replacing any active selection.
    pub fn insert_text(&mut self, input: &str) {
        if let Some((start, end)) = self.selection {
            let (min_idx, max_idx) = (start.min(end), start.max(end));
            if min_idx <= self.text.len() && max_idx <= self.text.len() {
                self.text.replace_range(min_idx..max_idx, input);
                self.cursor = min_idx + input.len();
                self.selection = None;
                return;
            }
        }
        let pos = self.cursor.min(self.text.len());
        self.text.insert_str(pos, input);
        self.cursor = pos + input.len();
        self.selection = None;
    }

    /// Inserts a newline character at the current cursor position.
    pub fn insert_newline(&mut self) {
        self.insert_text("\n");
    }

    /// Removes the character immediately preceding the cursor or removes active selection.
    pub fn backspace(&mut self) {
        if let Some((start, end)) = self.selection {
            let (min_idx, max_idx) = (start.min(end), start.max(end));
            if min_idx <= self.text.len() && max_idx <= self.text.len() {
                self.text.replace_range(min_idx..max_idx, "");
                self.cursor = min_idx;
                self.selection = None;
                return;
            }
        }
        if self.cursor > 0 && self.cursor <= self.text.len() {
            let mut prev = self.cursor - 1;
            while prev > 0 && !self.text.is_char_boundary(prev) {
                prev -= 1;
            }
            self.text.remove(prev);
            self.cursor = prev;
        }
    }

    /// Removes the character immediately following the cursor or removes active selection.
    pub fn delete(&mut self) {
        if let Some((start, end)) = self.selection {
            let (min_idx, max_idx) = (start.min(end), start.max(end));
            if min_idx <= self.text.len() && max_idx <= self.text.len() {
                self.text.replace_range(min_idx..max_idx, "");
                self.cursor = min_idx;
                self.selection = None;
                return;
            }
        }
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    /// Moves the cursor one character to the left.
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            let mut prev = self.cursor - 1;
            while prev > 0 && !self.text.is_char_boundary(prev) {
                prev -= 1;
            }
            self.cursor = prev;
        }
        self.selection = None;
    }

    /// Moves the cursor one character to the right.
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            let mut next = self.cursor + 1;
            while next < self.text.len() && !self.text.is_char_boundary(next) {
                next += 1;
            }
            self.cursor = next;
        }
        self.selection = None;
    }

    /// Moves the cursor to the beginning of the text buffer.
    pub fn move_cursor_to_start(&mut self) {
        self.cursor = 0;
        self.selection = None;
    }

    /// Moves the cursor to the end of the text buffer.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor = self.text.len();
        self.selection = None;
    }

    /// Selects all text in the buffer.
    pub fn select_all(&mut self) {
        self.selection = Some((0, self.text.len()));
    }

    /// Clears the text buffer and resets cursor position.
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection = None;
    }

    /// Returns the currently selected text substring if a valid selection exists.
    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection?;
        let (min_idx, max_idx) = (start.min(end), start.max(end));
        if min_idx <= self.text.len() && max_idx <= self.text.len() {
            Some(self.text[min_idx..max_idx].to_string())
        } else {
            None
        }
    }

    /// Returns true if the text buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the total line count in the text buffer.
    pub fn line_count(&self) -> usize {
        self.text.split('\n').count().max(1)
    }
}

/// Formats a file size in bytes to a human-readable string (B, KB, MB, GB).
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Returns true if the file extension is a recognized image format.
pub fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico"
    )
}

/// Pending attachment queued in the composer before sending.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingAttachment {
    pub id: String,
    pub path: PathBuf,
    pub filename: String,
    pub size_bytes: u64,
    pub is_image: bool,
    pub is_uploading: bool,
    pub progress: f32,
    pub uploaded_url: Option<String>,
}

impl PendingAttachment {
    /// Creates a new `PendingAttachment` from a file path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let is_image = is_image_extension(&ext);
        let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let id = format!("att_{}_{}", filename, size_bytes);

        Self {
            id,
            path,
            filename,
            size_bytes,
            is_image,
            is_uploading: false,
            progress: 0.0,
            uploaded_url: None,
        }
    }

    /// Sets the file size in bytes builder-style.
    pub fn with_size(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }

    /// Sets upload progress (0.0 to 1.0) and updates uploading status.
    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self.is_uploading = progress < 1.0;
        self
    }

    /// Formats the file size in human-readable notation.
    pub fn formatted_size(&self) -> String {
        format_file_size(self.size_bytes)
    }
}

/// Renders a clean vector upload (+) icon.
pub fn render_upload_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(16.0))
        .h(px(16.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                .child(
                    // Horizontal bar
                    div()
                        .absolute()
                        .top(px(5.0))
                        .left(px(1.0))
                        .w(px(10.0))
                        .h(px(2.0))
                        .rounded_xs()
                        .bg(theme::text_secondary()),
                )
                .child(
                    // Vertical bar
                    div()
                        .absolute()
                        .top(px(1.0))
                        .left(px(5.0))
                        .w(px(2.0))
                        .h(px(10.0))
                        .rounded_xs()
                        .bg(theme::text_secondary()),
                ),
        )
}

/// Renders a clean vector reaction icon.
pub fn render_react_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.0))
        .h(px(14.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                .rounded_full()
                .border_1()
                .border_color(theme::text_secondary())
                // Left Eye
                .child(
                    div()
                        .absolute()
                        .top(px(3.0))
                        .left(px(2.5))
                        .w(px(1.5))
                        .h(px(1.5))
                        .rounded_full()
                        .bg(theme::text_secondary()),
                )
                // Right Eye
                .child(
                    div()
                        .absolute()
                        .top(px(3.0))
                        .right(px(2.5))
                        .w(px(1.5))
                        .h(px(1.5))
                        .rounded_full()
                        .bg(theme::text_secondary()),
                )
                // Smile Arc
                .child(
                    div()
                        .absolute()
                        .bottom(px(2.0))
                        .left(px(3.0))
                        .w(px(6.0))
                        .h(px(2.5))
                        .border_b_1()
                        .border_color(theme::text_secondary())
                        .rounded_b_sm(),
                ),
        )
}

/// Renders a clean vector reply arrow icon.
pub fn render_reply_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.0))
        .h(px(14.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                // Left arrowhead
                .child(
                    div()
                        .absolute()
                        .top(px(2.5))
                        .left(px(1.0))
                        .w(px(4.0))
                        .h(px(4.0))
                        .border_t_1()
                        .border_l_1()
                        .border_color(theme::text_secondary()),
                )
                // Curved body
                .child(
                    div()
                        .absolute()
                        .top(px(4.0))
                        .left(px(3.5))
                        .w(px(7.0))
                        .h(px(5.0))
                        .border_t_1()
                        .border_r_1()
                        .border_color(theme::text_secondary())
                        .rounded_tr_md(),
                ),
        )
}

/// Renders a clean vector pencil edit icon.
pub fn render_edit_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.0))
        .h(px(14.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                // Pencil body
                .child(
                    div()
                        .absolute()
                        .top(px(1.5))
                        .right(px(1.5))
                        .w(px(7.0))
                        .h(px(3.5))
                        .rounded_xs()
                        .bg(theme::text_secondary()),
                )
                // Pencil tip
                .child(
                    div()
                        .absolute()
                        .bottom(px(1.5))
                        .left(px(1.5))
                        .w(px(3.5))
                        .h(px(3.5))
                        .border_b_1()
                        .border_l_1()
                        .border_color(theme::text_secondary()),
                ),
        )
}

/// Renders a clean vector pin/unpin icon.
pub fn render_pin_icon(is_pinned: bool) -> impl IntoElement {
    let col = if is_pinned {
        theme::accent()
    } else {
        theme::text_secondary()
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.0))
        .h(px(14.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                // Pin head
                .child(
                    div()
                        .absolute()
                        .top(px(1.0))
                        .left(px(3.5))
                        .w(px(5.0))
                        .h(px(3.5))
                        .rounded_xs()
                        .bg(col),
                )
                // Pin body
                .child(
                    div()
                        .absolute()
                        .top(px(4.5))
                        .left(px(4.5))
                        .w(px(3.0))
                        .h(px(3.5))
                        .bg(col),
                )
                // Pin needle
                .child(
                    div()
                        .absolute()
                        .bottom(px(0.5))
                        .left(px(5.5))
                        .w(px(1.0))
                        .h(px(4.0))
                        .bg(col),
                ),
        )
}

/// Renders a clean vector trashcan delete icon.
pub fn render_trash_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.0))
        .h(px(14.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                // Trash lid
                .child(
                    div()
                        .absolute()
                        .top(px(1.5))
                        .left(px(1.5))
                        .w(px(9.0))
                        .h(px(1.5))
                        .rounded_xs()
                        .bg(theme::status_dnd()),
                )
                // Trash body
                .child(
                    div()
                        .absolute()
                        .top(px(4.0))
                        .left(px(2.5))
                        .w(px(7.0))
                        .h(px(7.0))
                        .border_1()
                        .border_color(theme::status_dnd())
                        .rounded_b_sm(),
                ),
        )
}

/// Renders a clean vector copy icon.
pub fn render_copy_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(14.0))
        .h(px(14.0))
        .child(
            div()
                .relative()
                .w(px(12.0))
                .h(px(12.0))
                // Back card
                .child(
                    div()
                        .absolute()
                        .top(px(1.0))
                        .right(px(1.0))
                        .w(px(7.5))
                        .h(px(8.0))
                        .border_1()
                        .border_color(theme::text_muted())
                        .rounded_xs(),
                )
                // Front card
                .child(
                    div()
                        .absolute()
                        .bottom(px(1.0))
                        .left(px(1.0))
                        .w(px(7.5))
                        .h(px(8.0))
                        .border_1()
                        .border_color(theme::text_secondary())
                        .bg(theme::bg_elevated())
                        .rounded_xs(),
                ),
        )
}

/// Target message being referenced in a reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplyTarget {
    pub message_id: u64,
    pub author_name: String,
    pub sender_name: String,
    pub snippet: String,
}

impl ReplyTarget {
    pub fn new(
        message_id: u64,
        author_name: impl Into<String>,
        snippet: impl Into<String>,
    ) -> Self {
        let name = author_name.into();
        Self {
            message_id,
            author_name: name.clone(),
            sender_name: name,
            snippet: snippet.into(),
        }
    }
}

impl From<(u64, &str, &str)> for ReplyTarget {
    fn from((id, author, snippet): (u64, &str, &str)) -> Self {
        ReplyTarget::new(id, author, snippet)
    }
}

impl From<(u64, String, String)> for ReplyTarget {
    fn from((id, author, snippet): (u64, String, String)) -> Self {
        ReplyTarget::new(id, author, snippet)
    }
}

/// Formats reactor names for tooltip display (e.g. "Alice", "Alice and Bob", "Alice, Bob, and 2 others").
pub fn format_reactors_tooltip(reactor_names: &[String]) -> String {
    match reactor_names.len() {
        0 => String::new(),
        1 => reactor_names[0].clone(),
        2 => format!("{} and {}", reactor_names[0], reactor_names[1]),
        3 => format!(
            "{}, {}, and {}",
            reactor_names[0], reactor_names[1], reactor_names[2]
        ),
        n => {
            let others = n - 2;
            let other_str = if others == 1 { "other" } else { "others" };
            format!(
                "{}, {}, and {} {}",
                reactor_names[0], reactor_names[1], others, other_str
            )
        }
    }
}

/// Reaction badge displaying emoji, counter pill, active user reaction state, and reactor list tooltip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionBadge {
    pub emoji: String,
    pub count: usize,
    pub has_reacted: bool,
    pub reactor_names: Vec<String>,
}

impl ReactionBadge {
    pub fn new(
        emoji: impl Into<String>,
        count: usize,
        has_reacted: bool,
        reactor_names: Vec<String>,
    ) -> Self {
        Self {
            emoji: emoji.into(),
            count,
            has_reacted,
            reactor_names,
        }
    }

    /// Returns the tooltip text representing the reactor names.
    pub fn tooltip_text(&self) -> String {
        format_reactors_tooltip(&self.reactor_names)
    }

    /// Toggles the reacted state for the active user, adjusting the count accordingly.
    pub fn toggle_reacted(&mut self) {
        if self.has_reacted {
            self.has_reacted = false;
            self.count = self.count.saturating_sub(1);
        } else {
            self.has_reacted = true;
            self.count += 1;
        }
    }

    /// Aggregates a message's reactions list into grouped `ReactionBadge` entries.
    pub fn from_reactions(
        reactions: &[sharkord_core::types::MessageReaction],
        current_user_id: Option<u64>,
        users: &HashMap<u64, String>,
    ) -> Vec<Self> {
        let mut grouped: Vec<(String, usize, bool, Vec<String>)> = Vec::new();

        for r in reactions {
            let user_name = users
                .get(&r.user_id)
                .cloned()
                .unwrap_or_else(|| format!("User {}", r.user_id));
            let is_current = current_user_id.map_or(false, |uid| uid == r.user_id);

            if let Some(entry) = grouped
                .iter_mut()
                .find(|(emoji, _, _, _)| emoji == &r.emoji)
            {
                entry.1 += 1;
                if is_current {
                    entry.2 = true;
                }
                if !entry.3.contains(&user_name) {
                    entry.3.push(user_name);
                }
            } else {
                grouped.push((r.emoji.clone(), 1, is_current, vec![user_name]));
            }
        }

        grouped
            .into_iter()
            .map(|(emoji, count, has_reacted, reactor_names)| {
                ReactionBadge::new(emoji, count, has_reacted, reactor_names)
            })
            .collect()
    }

    /// Renders the reaction pill element.
    pub fn render_element(&self) -> impl IntoElement {
        let border_col = if self.has_reacted {
            theme::accent()
        } else {
            theme::border_hairline()
        };
        let bg_col = if self.has_reacted {
            theme::bg_elevated()
        } else {
            theme::bg_surface_4()
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_full()
            .bg(bg_col)
            .border_1()
            .border_color(border_col)
            .cursor_pointer()
            .child(div().text_xs().child(self.emoji.clone()))
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(if self.has_reacted {
                        theme::accent()
                    } else {
                        theme::text_primary()
                    })
                    .child(self.count.to_string()),
            )
    }
}

/// Floating hover message action toolbar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageActionToolbar {
    pub message_id: u64,
    pub can_edit: bool,
    pub is_pinned: bool,
}

impl MessageActionToolbar {
    pub fn new(message_id: u64, can_edit: bool, is_pinned: bool) -> Self {
        Self {
            message_id,
            can_edit,
            is_pinned,
        }
    }

    pub fn render_element(&self) -> impl IntoElement {
        let mut bar = div()
            .id(ElementId::Name(
                format!("msg_actions_toolbar_{}", self.message_id).into(),
            ))
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .px_1p5()
            .py_1()
            .rounded_lg()
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .shadow_md();

        // 1. React button
        bar = bar.child(
            div()
                .id(ElementId::Name(
                    format!("action_react_{}", self.message_id).into(),
                ))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_md()
                .hover(|s| s.bg(theme::bg_surface_4()))
                .child(render_react_icon()),
        );

        // 2. Reply button
        bar = bar.child(
            div()
                .id(ElementId::Name(
                    format!("action_reply_{}", self.message_id).into(),
                ))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_md()
                .hover(|s| s.bg(theme::bg_surface_4()))
                .child(render_reply_icon()),
        );

        // 3. Edit button (if can_edit)
        if self.can_edit {
            bar = bar.child(
                div()
                    .id(ElementId::Name(
                        format!("action_edit_{}", self.message_id).into(),
                    ))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(26.0))
                    .h(px(26.0))
                    .rounded_md()
                    .hover(|s| s.bg(theme::bg_surface_4()))
                    .child(render_edit_icon()),
            );
        }

        // 4. Pin / Unpin button
        bar = bar.child(
            div()
                .id(ElementId::Name(
                    format!("action_pin_{}", self.message_id).into(),
                ))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_md()
                .hover(|s| s.bg(theme::bg_surface_4()))
                .child(render_pin_icon(self.is_pinned)),
        );

        // 5. Copy text button
        bar = bar.child(
            div()
                .id(ElementId::Name(
                    format!("action_copy_{}", self.message_id).into(),
                ))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_md()
                .hover(|s| s.bg(theme::bg_surface_4()))
                .child(render_copy_icon()),
        );

        // 6. Delete button (if can_edit)
        if self.can_edit {
            bar = bar.child(
                div()
                    .id(ElementId::Name(
                        format!("action_delete_{}", self.message_id).into(),
                    ))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(26.0))
                    .h(px(26.0))
                    .rounded_md()
                    .hover(|s| s.bg(theme::bg_surface_4()))
                    .child(render_trash_icon()),
            );
        }

        bar
    }
}

/// Renders file attachments for a message natively (image preview cards or file download pills).
pub fn render_message_attachments(files: &[MessageFile]) -> Vec<AnyElement> {
    if files.is_empty() {
        return Vec::new();
    }

    files
        .iter()
        .map(|file| {
            let is_img = is_image_extension(&file.extension);
            let formatted_size = format_file_size(file.size.max(0) as u64);
            let file_name = if !file.original_name.is_empty() {
                file.original_name.clone()
            } else {
                file.name.clone()
            };

            if is_img {
                div()
                    .id(ElementId::Name(
                        format!("msg_img_attachment_{}", file.id).into(),
                    ))
                    .cursor_pointer()
                    .flex()
                    .flex_col()
                    .max_w(px(380.0))
                    .my_1()
                    .rounded_lg()
                    .bg(theme::bg_surface_4())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .h(px(140.0))
                            .bg(theme::bg_base())
                            .text_2xl()
                            .child("🖼"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_primary())
                                    .child(file_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(formatted_size),
                            ),
                    )
                    .into_any_element()
            } else {
                div()
                    .id(ElementId::Name(
                        format!("msg_file_attachment_{}", file.id).into(),
                    ))
                    .cursor_pointer()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .max_w(px(340.0))
                    .my_1()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(theme::bg_surface_4())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2p5()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(28.0))
                                    .h(px(28.0))
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .text_xs()
                                    .text_color(theme::accent())
                                    .child("📄"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_primary())
                                            .child(file_name),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_muted())
                                            .child(formatted_size),
                                    ),
                            ),
                    )
                    .child(div().text_xs().text_color(theme::text_muted()).child("⬇"))
                    .into_any_element()
            }
        })
        .collect()
}

/// Formats a unix timestamp into an HH:MM string.
pub fn format_message_timestamp(timestamp: i64) -> String {
    let seconds = if timestamp > 1_000_000_000_000 {
        timestamp / 1000
    } else {
        timestamp
    };
    let day_seconds = (seconds % 86400).max(0);
    let hours = day_seconds / 3600;
    let minutes = (day_seconds % 3600) / 60;
    format!("{:02}:{:02}", hours, minutes)
}

/// A contiguous group of messages sent by the same user within a 5-minute window.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageFeedGroup {
    pub user_id: Option<u64>,
    pub sender_name: String,
    pub avatar_id: Option<u64>,
    pub timestamp: i64,
    pub formatted_time: String,
    pub messages: Vec<Message>,
}

/// Groups messages from the same user within 5-minute (300s) windows to eliminate repetitive headers.
pub fn group_messages_for_feed(
    messages: &[Message],
    users: &HashMap<u64, String>,
) -> Vec<MessageFeedGroup> {
    if messages.is_empty() {
        return Vec::new();
    }

    let mut groups: Vec<MessageFeedGroup> = Vec::new();

    for msg in messages {
        let msg_time = msg.created_at.unwrap_or(0);
        let sender_name = msg
            .user_id
            .and_then(|id| users.get(&id).cloned())
            .unwrap_or_else(|| "Unknown".to_string());

        let can_group = match groups.last() {
            Some(last_group) => {
                last_group.user_id == msg.user_id
                    && (msg_time.saturating_sub(last_group.timestamp) <= 300)
            }
            None => false,
        };

        if can_group {
            if let Some(last_group) = groups.last_mut() {
                last_group.messages.push(msg.clone());
            }
        } else {
            groups.push(MessageFeedGroup {
                user_id: msg.user_id,
                sender_name,
                avatar_id: None,
                timestamp: msg_time,
                formatted_time: format_message_timestamp(msg_time),
                messages: vec![msg.clone()],
            });
        }
    }

    groups
}

/// Discord-style Welcome Card displayed when a channel has no messages yet or at the beginning of the history.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelWelcomeCard {
    pub channel_name: String,
    pub topic: Option<String>,
}

impl ChannelWelcomeCard {
    pub fn new(channel_name: impl Into<String>, topic: Option<String>) -> Self {
        Self {
            channel_name: channel_name.into(),
            topic,
        }
    }

    /// Returns the headline title for the welcome card.
    pub fn title_text(&self) -> String {
        format!("Welcome to #{}!", self.channel_name)
    }

    /// Returns helper explanatory text for the welcome card.
    pub fn helper_description(&self) -> String {
        match &self.topic {
            Some(topic) if !topic.trim().is_empty() => {
                format!(
                    "This is the start of the #{} channel. Topic: {}",
                    self.channel_name,
                    topic.trim()
                )
            }
            _ => format!(
                "This is the start of the #{} channel. Send a message to start the conversation!",
                self.channel_name
            ),
        }
    }

    /// Renders the Discord-style Welcome Card with a large `#` circle, bold headline, and helper text.
    pub fn render_element(&self) -> impl IntoElement {
        div()
            .id("channel_welcome_card")
            .flex()
            .flex_col()
            .gap_2()
            .py_6()
            .child(
                // Large # icon circle
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(64.0))
                    .h(px(64.0))
                    .rounded_full()
                    .bg(theme::bg_surface_4())
                    .text_3xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child("#"),
            )
            .child(
                // Bold channel welcome headline
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child(self.title_text()),
            )
            .child(
                // Explanatory helper text
                div()
                    .text_sm()
                    .text_color(theme::text_muted())
                    .child(self.helper_description()),
            )
    }
}

/// Helper function to create a channel welcome card.
pub fn render_channel_welcome_card(channel_name: &str, topic: Option<&str>) -> ChannelWelcomeCard {
    ChannelWelcomeCard {
        channel_name: channel_name.to_string(),
        topic: topic.map(|t| t.to_string()),
    }
}

/// Default slash commands provided for native autocomplete.
pub fn default_slash_commands() -> Vec<PluginCommand> {
    vec![
        PluginCommand {
            plugin_id: "poll".to_string(),
            name: "poll".to_string(),
            description: "Create an interactive poll in the channel".to_string(),
        },
        PluginCommand {
            plugin_id: "remind".to_string(),
            name: "remind".to_string(),
            description: "Set a reminder notification".to_string(),
        },
        PluginCommand {
            plugin_id: "soundboard".to_string(),
            name: "soundboard".to_string(),
            description: "Play a sound clip in the voice channel".to_string(),
        },
    ]
}

/// Renders a keyboard shortcut badge (Kbd).
pub fn render_kbd_badge(key: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .bg(theme::bg_surface_4())
        .border_1()
        .border_color(theme::border_hairline())
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme::text_muted())
        .child(key.to_string())
}

/// Renders skeleton loader message cards while channel history is fetching.
pub fn render_skeleton_feed() -> impl IntoElement {
    div()
        .id("skeleton_message_feed")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        .flex_1()
        .overflow_y_scroll()
        .children((0..5).map(|idx| {
            let width_mod = match idx % 3 {
                0 => px(260.0),
                1 => px(340.0),
                _ => px(190.0),
            };

            div()
                .flex()
                .flex_row()
                .gap_3()
                .child(
                    // Skeleton avatar circle
                    div()
                        .w(px(36.0))
                        .h(px(36.0))
                        .rounded_full()
                        .bg(theme::bg_surface_4()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .flex_1()
                        // Skeleton header: name + time
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .w(px(90.0))
                                        .h(px(14.0))
                                        .rounded_sm()
                                        .bg(theme::bg_surface_4()),
                                )
                                .child(
                                    div()
                                        .w(px(40.0))
                                        .h(px(10.0))
                                        .rounded_sm()
                                        .bg(theme::bg_elevated()),
                                ),
                        )
                        // Skeleton message body lines
                        .child(
                            div()
                                .w(width_mod)
                                .h(px(14.0))
                                .rounded_sm()
                                .bg(theme::bg_surface_4()),
                        )
                        .child(
                            div()
                                .w(px(180.0))
                                .h(px(14.0))
                                .rounded_sm()
                                .bg(theme::bg_elevated()),
                        ),
                )
        }))
}

/// Renders clean vector users sidebar toggle icon.
pub fn render_users_toggle_icon(is_active: bool) -> impl IntoElement {
    let icon_color = if is_active {
        theme::accent()
    } else {
        theme::text_muted()
    };

    div()
        .flex()
        .flex_row()
        .items_end()
        .justify_center()
        .gap_0p5()
        // Person 1 (Primary)
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .child(div().w(px(7.0)).h(px(7.0)).rounded_full().bg(icon_color))
                .child(div().w(px(11.0)).h(px(6.0)).rounded_t_md().bg(icon_color)),
        )
        // Person 2 (Secondary)
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .child(div().w(px(5.5)).h(px(5.5)).rounded_full().bg(icon_color))
                .child(div().w(px(8.5)).h(px(5.0)).rounded_t_md().bg(icon_color)),
        )
}

/// Renders interactive message components (buttons, polls, form cards) natively.
pub fn render_interactive_components(components: &[InteractiveComponent]) -> Vec<AnyElement> {
    components
        .iter()
        .map(|comp| match comp {
            InteractiveComponent::Button(btn) => {
                let is_danger = btn.style.as_deref() == Some("danger");
                let is_primary = btn.style.as_deref() == Some("primary");
                let bg_color = if is_danger {
                    theme::status_dnd()
                } else if is_primary {
                    theme::accent()
                } else {
                    theme::bg_elevated()
                };

                div()
                    .id(ElementId::Name(format!("plugin_button_{}", btn.id).into()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .my_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(bg_color)
                            .border_1()
                            .border_color(theme::border_hairline())
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_primary())
                            .child(btn.label.clone()),
                    )
                    .into_any_element()
            }
            InteractiveComponent::Poll(poll) => div()
                .id(ElementId::Name(format!("plugin_poll_{}", poll.id).into()))
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .my_2()
                .rounded_lg()
                .bg(theme::bg_surface())
                .border_1()
                .border_color(theme::border_hairline())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::text_primary())
                        .child(poll.question.clone()),
                )
                .children(poll.options.iter().map(|opt| {
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(if opt.voted {
                            theme::bg_elevated()
                        } else {
                            theme::bg_surface_4()
                        })
                        .border_1()
                        .border_color(if opt.voted {
                            theme::accent()
                        } else {
                            theme::border_hairline()
                        })
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_primary())
                                .child(opt.text.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(format!("{} votes", opt.votes)),
                        )
                }))
                .into_any_element(),
            InteractiveComponent::Card {
                title,
                description,
                buttons,
            } => div()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .my_2()
                .rounded_lg()
                .bg(theme::bg_surface())
                .border_1()
                .border_color(theme::border_hairline())
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::text_primary())
                        .child(title.clone()),
                )
                .children(description.as_ref().map(|desc| {
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(desc.clone())
                }))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .children(buttons.iter().map(|btn| {
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(theme::bg_elevated())
                                .border_1()
                                .border_color(theme::border_hairline())
                                .text_xs()
                                .text_color(theme::text_primary())
                                .child(btn.label.clone())
                        })),
                )
                .into_any_element(),
            InteractiveComponent::Unknown => div().into_any_element(),
        })
        .collect()
}

/// Chat Canvas view state.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatCanvasView {
    pub channel_name: String,
    pub channel_topic: Option<String>,
    pub groups: Vec<MessageFeedGroup>,
    pub message_draft: String,
    pub is_users_panel_open: bool,
    pub available_commands: Vec<PluginCommand>,
    pub is_loading: bool,
    pub is_virtualized: bool,
    pub textarea: ComposerTextarea,
    pub virtual_viewport_offset: usize,
    pub virtual_viewport_size: usize,
    pub pending_attachments: Vec<PendingAttachment>,
    pub lightbox: LightboxState,
    pub reply_target: Option<ReplyTarget>,
    pub editing_message_id: Option<u64>,
    pub editing_text: String,
    pub emoji_picker: EmojiPickerState,
    pub current_user_id: Option<u64>,
    pub users: HashMap<u64, String>,
}

impl Default for ChatCanvasView {
    fn default() -> Self {
        Self {
            channel_name: "general".to_string(),
            channel_topic: Some("General discussion".to_string()),
            groups: Vec::new(),
            message_draft: String::new(),
            is_users_panel_open: true,
            available_commands: default_slash_commands(),
            is_loading: false,
            is_virtualized: true,
            textarea: ComposerTextarea::default(),
            virtual_viewport_offset: 0,
            virtual_viewport_size: 50,
            pending_attachments: Vec::new(),
            lightbox: LightboxState::new(),
            reply_target: None,
            editing_message_id: None,
            editing_text: String::new(),
            emoji_picker: EmojiPickerState::new(),
            current_user_id: None,
            users: HashMap::new(),
        }
    }
}

impl ChatCanvasView {
    pub fn new(
        channel_name: impl Into<String>,
        channel_topic: Option<String>,
        messages: &[Message],
        users: &HashMap<u64, String>,
        draft: impl Into<String>,
    ) -> Self {
        let draft_str = draft.into();
        let groups = group_messages_for_feed(messages, users);
        Self {
            channel_name: channel_name.into(),
            channel_topic,
            groups,
            message_draft: draft_str.clone(),
            is_users_panel_open: true,
            available_commands: default_slash_commands(),
            is_loading: false,
            is_virtualized: true,
            textarea: ComposerTextarea::with_text(draft_str),
            virtual_viewport_offset: 0,
            virtual_viewport_size: 50,
            pending_attachments: Vec::new(),
            lightbox: LightboxState::new(),
            reply_target: None,
            editing_message_id: None,
            editing_text: String::new(),
            emoji_picker: EmojiPickerState::new(),
            current_user_id: None,
            users: users.clone(),
        }
    }

    /// Sets the reply target message.
    pub fn set_reply_target(&mut self, target: Option<ReplyTarget>) {
        self.reply_target = target;
    }

    /// Clears the active reply target.
    pub fn clear_reply_target(&mut self) {
        self.reply_target = None;
    }

    /// Initiates in-place editing for the specified message ID.
    pub fn start_editing(&mut self, message_id: u64, current_text: &str) {
        self.editing_message_id = Some(message_id);
        self.editing_text = current_text.to_string();
    }

    /// Commits and clears the active in-place edit, returning the message ID and updated text.
    pub fn save_inline_edit(&mut self) -> Option<(u64, String)> {
        let msg_id = self.editing_message_id.take()?;
        let text = std::mem::take(&mut self.editing_text);
        Some((msg_id, text))
    }

    /// Cancels active in-place editing and clears editing buffer.
    pub fn cancel_editing(&mut self) {
        self.editing_message_id = None;
        self.editing_text.clear();
    }

    /// Opens the emoji picker popover for the specified message ID.
    pub fn open_emoji_picker(&mut self, message_id: u64) {
        self.emoji_picker.open_for_message(message_id);
    }

    /// Closes the emoji picker popover.
    pub fn close_emoji_picker(&mut self) {
        self.emoji_picker.close();
    }

    /// Selects an emoji reaction for the active target message, closing the picker.
    pub fn select_reaction_emoji(&mut self, emoji: &str) -> Option<(u64, String)> {
        let msg_id = self.emoji_picker.target_message_id?;
        let selected = self.emoji_picker.select_emoji(emoji)?;
        Some((msg_id, selected))
    }

    pub fn with_commands(mut self, commands: Vec<PluginCommand>) -> Self {
        self.available_commands = commands;
        self
    }

    /// Adds a pending file attachment to the composer queue.
    pub fn add_pending_attachment(&mut self, attachment: PendingAttachment) {
        self.pending_attachments.push(attachment);
    }

    /// Removes a pending attachment from the composer queue by its ID.
    pub fn remove_pending_attachment(&mut self, id: &str) {
        self.pending_attachments.retain(|a| a.id != id);
    }

    /// Clears all pending attachments from the composer queue.
    pub fn clear_pending_attachments(&mut self) {
        self.pending_attachments.clear();
    }

    /// Handles file drop events by wrapping each dropped path into a `PendingAttachment`.
    pub fn handle_file_drop(&mut self, paths: &[PathBuf]) {
        for path in paths {
            self.add_pending_attachment(PendingAttachment::new(path));
        }
    }

    /// Opens the native OS file picker dialog via `rfd` and queues selected files.
    pub fn pick_files_dialog(&mut self) {
        if let Some(files) = rfd::FileDialog::new().pick_files() {
            for path in files {
                self.add_pending_attachment(PendingAttachment::new(path));
            }
        }
    }

    /// Opens the image lightbox modal dialog for the specified URL and optional name.
    pub fn open_lightbox(&mut self, url: impl Into<String>, name: Option<String>) {
        if let Some(n) = name {
            self.lightbox.open_with_name(url, n);
        } else {
            self.lightbox.open(url);
        }
    }

    /// Closes the image lightbox modal dialog.
    pub fn close_lightbox(&mut self) {
        self.lightbox.close();
    }

    /// Sets the initial loading skeleton state.
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Returns the total message count across all grouped feed items.
    pub fn total_messages_count(&self) -> usize {
        self.groups.iter().map(|g| g.messages.len()).sum()
    }

    /// Updates the composer message draft and synchronizes the textarea buffer.
    pub fn set_message_draft(&mut self, draft: impl Into<String>) {
        let text = draft.into();
        self.message_draft = text.clone();
        self.textarea.text = text.clone();
        self.textarea.cursor = text.len();
    }

    /// Handles keyboard events for the rich multiline composer.
    /// Shift+Enter inserts a newline, while Enter triggers message sending.
    pub fn handle_composer_key(&mut self, key: &str, is_shift: bool) -> ComposerAction {
        match key.to_lowercase().as_str() {
            "enter" => {
                if is_shift {
                    self.textarea.insert_newline();
                    self.message_draft = self.textarea.text.clone();
                    ComposerAction::Newline
                } else {
                    ComposerAction::Send
                }
            }
            "backspace" => {
                self.textarea.backspace();
                self.message_draft = self.textarea.text.clone();
                ComposerAction::InputUpdated
            }
            ch => {
                if ch.len() == 1 {
                    self.textarea.insert_text(ch);
                    self.message_draft = self.textarea.text.clone();
                    ComposerAction::InputUpdated
                } else {
                    ComposerAction::None
                }
            }
        }
    }

    /// Returns slash commands matching the current composer input prefix.
    pub fn matching_slash_commands(&self) -> Vec<PluginCommand> {
        let draft = if !self.textarea.text.is_empty() {
            &self.textarea.text
        } else {
            &self.message_draft
        };

        if !draft.starts_with('/') {
            return Vec::new();
        }
        let query = draft[1..].trim().to_lowercase();
        let commands = if self.available_commands.is_empty() {
            default_slash_commands()
        } else {
            self.available_commands.clone()
        };
        if query.is_empty() {
            commands
        } else {
            commands
                .into_iter()
                .filter(|c| c.name.to_lowercase().starts_with(&query))
                .collect()
        }
    }

    /// Renders the Chat Canvas element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None, None)
    }

    /// Renders the Chat Canvas element tree with interactive composer input.
    pub fn render_interactive(
        &self,
        focus_handle: Option<&FocusHandle>,
        cx: &mut Context<crate::ui::WorkspaceView>,
    ) -> impl IntoElement {
        self.render_internal(focus_handle, Some(cx))
    }

    fn render_internal(
        &self,
        focus_handle: Option<&FocusHandle>,
        mut cx: Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> impl IntoElement {
        let placeholder = format!("Message #{}...", self.channel_name);
        let active_text = if !self.textarea.text.is_empty() {
            &self.textarea.text
        } else {
            &self.message_draft
        };

        // Determine visible groups slice for virtualized feed
        let visible_groups: &[MessageFeedGroup] =
            if self.is_virtualized && self.groups.len() > self.virtual_viewport_size {
                let start = self.virtual_viewport_offset.min(self.groups.len());
                let end = (start + self.virtual_viewport_size).min(self.groups.len());
                &self.groups[start..end]
            } else {
                &self.groups
            };

        let mut root = div()
            .relative()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(theme::bg_base())
            // 1. Channel Header Bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(48.0))
                    .px_4()
                    .bg(theme::bg_surface())
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    // Left: Channel name & topic
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .text_color(theme::text_primary())
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("#")
                                    .child(self.channel_name.clone()),
                            )
                            .child(div().w(px(1.0)).h(px(16.0)).bg(theme::border_hairline()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(self.channel_topic.clone().unwrap_or_default()),
                            ),
                    )
                    // Right: Users list toggle button with clean vector icon
                    .child(
                        div()
                            .id("users_toggle_btn")
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(32.0))
                            .h(px(32.0))
                            .rounded_md()
                            .bg(if self.is_users_panel_open {
                                theme::bg_elevated()
                            } else {
                                theme::bg_surface()
                            })
                            .border_1()
                            .border_color(theme::border_hairline())
                            .child(render_users_toggle_icon(self.is_users_panel_open)),
                    ),
            )
            // 2. Message History Stream with Virtualized Feed / Skeleton Loader
            .child(if self.is_loading {
                render_skeleton_feed().into_any_element()
            } else if self.groups.is_empty() {
                div()
                    .id("chat_message_history")
                    .flex()
                    .flex_col()
                    .p_4()
                    .flex_1()
                    .overflow_y_scroll()
                    .child(
                        render_channel_welcome_card(
                            &self.channel_name,
                            self.channel_topic.as_deref(),
                        )
                        .render_element(),
                    )
                    .into_any_element()
            } else {
                div()
                    .id("chat_message_history")
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(visible_groups.iter().map(|group| {
                        let initials = server_initials(&group.sender_name);

                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(
                                // Avatar circle
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(36.0))
                                    .h(px(36.0))
                                    .rounded_full()
                                    .bg(theme::bg_surface_4())
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child(initials),
                            )
                            .child(
                                // Messages container
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .flex_1()
                                    // Author & timestamp header
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_baseline()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child(group.sender_name.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child(group.formatted_time.clone()),
                                            ),
                                    )
                                    // Body of grouped messages rendered with Markdown parser
                                    .children(group.messages.iter().map(|msg| {
                                        let is_editing = self.editing_message_id == Some(msg.id);
                                        let can_edit = self
                                            .current_user_id
                                            .map_or(true, |uid| msg.user_id == Some(uid));
                                        let is_pinned = msg.pinned;
                                        let action_toolbar =
                                            MessageActionToolbar::new(msg.id, can_edit, is_pinned);
                                        let badges = ReactionBadge::from_reactions(
                                            &msg.reactions,
                                            self.current_user_id,
                                            &self.users,
                                        );

                                        let mut msg_row = div()
                                            .id(ElementId::Name(
                                                format!("msg_row_{}", msg.id).into(),
                                            ))
                                            .relative()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .py_1();

                                        if is_editing {
                                            msg_row = msg_row.child(
                                                div()
                                                    .id(ElementId::Name(
                                                        format!("inline_edit_box_{}", msg.id)
                                                            .into(),
                                                    ))
                                                    .flex()
                                                    .flex_col()
                                                    .gap_2()
                                                    .p_2()
                                                    .rounded_lg()
                                                    .bg(theme::bg_elevated())
                                                    .border_1()
                                                    .border_color(theme::accent())
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .text_color(theme::text_primary())
                                                            .child(self.editing_text.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .justify_end()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .id("cancel_inline_edit_btn")
                                                                    .cursor_pointer()
                                                                    .px_2()
                                                                    .py_0p5()
                                                                    .rounded_sm()
                                                                    .bg(theme::bg_surface_4())
                                                                    .text_xs()
                                                                    .text_color(theme::text_muted())
                                                                    .child("Cancel (Esc)"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("save_inline_edit_btn")
                                                                    .cursor_pointer()
                                                                    .px_2()
                                                                    .py_0p5()
                                                                    .rounded_sm()
                                                                    .bg(theme::accent())
                                                                    .text_xs()
                                                                    .font_weight(
                                                                        gpui::FontWeight::MEDIUM,
                                                                    )
                                                                    .text_color(
                                                                        theme::text_primary(),
                                                                    )
                                                                    .child("Save (Enter)"),
                                                            ),
                                                    ),
                                            );
                                        } else {
                                            msg_row = msg_row
                                                .child(render_markdown_element(
                                                    &msg.content.clone().unwrap_or_default(),
                                                ))
                                                .children(render_interactive_components(
                                                    &msg.components,
                                                ))
                                                .children(render_message_attachments(&msg.files));

                                            if !badges.is_empty() {
                                                msg_row = msg_row.child(
                                                    div()
                                                        .id(ElementId::Name(
                                                            format!("msg_reactions_{}", msg.id)
                                                                .into(),
                                                        ))
                                                        .flex()
                                                        .flex_row()
                                                        .flex_wrap()
                                                        .gap_1p5()
                                                        .pt_1()
                                                        .children(
                                                            badges
                                                                .iter()
                                                                .map(|b| b.render_element()),
                                                        ),
                                                );
                                            }

                                            // Action toolbar hovering
                                            msg_row = msg_row.child(
                                                div()
                                                    .absolute()
                                                    .top_neg_2()
                                                    .right_2()
                                                    .child(action_toolbar.render_element()),
                                            );
                                        }

                                        msg_row
                                    })),
                            )
                    }))
                    .into_any_element()
            })
            // 3. Bottom Rich Multiline Composer Input Bar with Slash Command Popover & Kbd Badges
            .child(
                div().p_4().bg(theme::bg_base()).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(if active_text.starts_with('/') {
                            let matching = self.matching_slash_commands();
                            if matching.is_empty() {
                                None
                            } else {
                                Some(
                                    div()
                                        .id("slash_command_popover")
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .p_2()
                                        .mb_2()
                                        .rounded_lg()
                                        .bg(theme::bg_elevated())
                                        .border_1()
                                        .border_color(theme::border_hairline())
                                        .children(matching.into_iter().map(|cmd| {
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .px_3()
                                                .py_1p5()
                                                .rounded_md()
                                                .hover(|s| s.bg(theme::bg_surface_4()))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .font_weight(gpui::FontWeight::BOLD)
                                                                .text_color(theme::accent())
                                                                .child(format!("/{}", cmd.name)),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme::text_muted())
                                                                .child(cmd.description),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::text_muted())
                                                        .child(cmd.plugin_id),
                                                )
                                        })),
                                )
                            }
                        } else {
                            None
                        })
                        .children(if let Some(ref target) = self.reply_target {
                            Some(
                                div()
                                    .id("composer_reply_preview_banner")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .px_3()
                                    .py_1p5()
                                    .mb_2()
                                    .rounded_lg()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::accent())
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme::accent())
                                                    .child(format!(
                                                        "Replying to @{}:",
                                                        target.sender_name
                                                    )),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child(target.snippet.clone()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id("cancel_reply_btn")
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme::text_muted())
                                            .hover(|s| s.text_color(theme::text_primary()))
                                            .child("×"),
                                    ),
                            )
                        } else {
                            None
                        })
                        .children(if !self.pending_attachments.is_empty() {
                            Some(
                                div()
                                    .id("composer_pending_attachments_strip")
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .p_2()
                                    .mb_2()
                                    .rounded_lg()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .overflow_x_scroll()
                                    .children(self.pending_attachments.iter().map(|att| {
                                        let icon_char = if att.is_image { "🖼" } else { "📄" };
                                        let size_str = att.formatted_size();
                                        let att_id = att.id.clone();

                                        div()
                                            .id(ElementId::Name(
                                                format!("pending_att_{}", att_id).into(),
                                            ))
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(theme::bg_elevated())
                                            .border_1()
                                            .border_color(theme::border_hairline())
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .w(px(24.0))
                                                    .h(px(24.0))
                                                    .rounded_sm()
                                                    .bg(theme::bg_surface_4())
                                                    .text_xs()
                                                    .child(icon_char),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap_0p5()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .text_color(theme::text_primary())
                                                            .child(att.filename.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme::text_muted())
                                                            .child(size_str),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .id(ElementId::Name(
                                                        format!("remove_pending_{}", att_id).into(),
                                                    ))
                                                    .cursor_pointer()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .w(px(18.0))
                                                    .h(px(18.0))
                                                    .rounded_full()
                                                    .bg(theme::bg_surface_4())
                                                    .hover(|s| s.bg(theme::bg_surface()))
                                                    .text_xs()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(theme::text_muted())
                                                    .child("×"),
                                            )
                                    })),
                            )
                        } else {
                            None
                        })
                        .child({
                            let upload_btn = div()
                                .id("composer_upload_btn")
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(28.0))
                                .h(px(28.0))
                                .mr_2()
                                .rounded_md()
                                .bg(theme::bg_surface_4())
                                .hover(|s| s.bg(theme::bg_surface()))
                                .child(render_upload_icon());

                            let mut composer_container = div()
                                .id("chat_composer_input")
                                .cursor_text()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .px_4()
                                .py_3()
                                .rounded_lg()
                                .bg(theme::bg_elevated())
                                .border_1()
                                .border_color(theme::border_hairline())
                                .child(upload_btn)
                                .child(
                                    div().flex().flex_col().flex_1().gap_1().child(
                                        div()
                                            .text_sm()
                                            .text_color(if active_text.is_empty() {
                                                theme::text_muted()
                                            } else {
                                                theme::text_primary()
                                            })
                                            .child(if active_text.is_empty() {
                                                placeholder
                                            } else {
                                                active_text.to_string()
                                            }),
                                    ),
                                );

                            let mut send_btn = div()
                                .id("chat_composer_send_btn")
                                .cursor_pointer()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1p5()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(theme::accent())
                                .hover(|s| s.bg(theme::accent_hover()))
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme::text_primary())
                                .child("Send")
                                .child(render_kbd_badge("↵"));

                            if let Some(handle) = focus_handle {
                                composer_container = composer_container.track_focus(handle);
                            }

                            if let Some(ref mut cx) = cx {
                                composer_container = composer_container
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|view, _, window, _| {
                                            if let Some(ref handle) = view.focus_handle {
                                                window.focus(handle);
                                            }
                                        }),
                                    )
                                    .on_key_down(cx.listener(
                                        |view, event: &KeyDownEvent, _, cx| {
                                            view.on_composer_key_down(event, cx);
                                        },
                                    ));

                                send_btn = send_btn.on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|view, _, _window, cx| {
                                        view.send_message();
                                        cx.notify();
                                    }),
                                );
                            }

                            composer_container.child(send_btn)
                        })
                        // Kbd Shortcut Badges Footer
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_end()
                                .gap_3()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .child(render_kbd_badge("Enter"))
                                        .child(div().child("to send")),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_1()
                                        .child(render_kbd_badge("Shift"))
                                        .child(div().child("+"))
                                        .child(render_kbd_badge("Enter"))
                                        .child(div().child("for new line")),
                                ),
                        ),
                ),
            );

        if self.emoji_picker.is_open {
            root = root.child(
                div()
                    .absolute()
                    .bottom(px(80.0))
                    .right(px(24.0))
                    .child(EmojiPickerView::new(self.emoji_picker.clone()).render_element()),
            );
        }

        if self.lightbox.is_open {
            root = root.child(LightboxModalView::new(self.lightbox.clone()).render_element());
        }

        root
    }
}

impl Render for ChatCanvasView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
