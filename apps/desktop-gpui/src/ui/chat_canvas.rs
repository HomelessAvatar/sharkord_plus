//! Chat Canvas view component (Flex 1).
//! Renders the channel header with topic and member toggle, scrollable message stream
//! with sender avatars and timestamps (grouped within 5-minute windows), and docked bottom
//! composer input bar. Adheres strictly to the Pure Neutral Dark design specification.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::{InteractiveComponent, Message, PluginCommand};

use crate::theme;
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
        }
    }

    pub fn with_commands(mut self, commands: Vec<PluginCommand>) -> Self {
        self.available_commands = commands;
        self
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
        let visible_groups: &[MessageFeedGroup] = if self.is_virtualized && self.groups.len() > self.virtual_viewport_size {
            let start = self.virtual_viewport_offset.min(self.groups.len());
            let end = (start + self.virtual_viewport_size).min(self.groups.len());
            &self.groups[start..end]
        } else {
            &self.groups
        };

        div()
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
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(render_markdown_element(
                                                &msg.content.clone().unwrap_or_default(),
                                            ))
                                            .children(render_interactive_components(
                                                &msg.components,
                                            ))
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
                        .child({
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
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .gap_1()
                                        .child(
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
            )
    }
}

impl Render for ChatCanvasView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
