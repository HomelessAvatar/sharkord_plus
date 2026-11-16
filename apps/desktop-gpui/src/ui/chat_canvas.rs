//! Chat Canvas view component (Flex 1).
//! Renders the channel header with topic and member toggle, scrollable message stream
//! with sender avatars and timestamps (grouped within 5-minute windows), and docked bottom
//! composer input bar.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use sharkord_core::types::Message;

use crate::theme;
use crate::ui::server_rail::server_initials;

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

/// Chat Canvas view state.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatCanvasView {
    pub channel_name: String,
    pub channel_topic: Option<String>,
    pub groups: Vec<MessageFeedGroup>,
    pub message_draft: String,
    pub is_users_panel_open: bool,
}

impl Default for ChatCanvasView {
    fn default() -> Self {
        Self {
            channel_name: "general".to_string(),
            channel_topic: Some("General discussion".to_string()),
            groups: Vec::new(),
            message_draft: String::new(),
            is_users_panel_open: true,
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
        let groups = group_messages_for_feed(messages, users);
        Self {
            channel_name: channel_name.into(),
            channel_topic,
            groups,
            message_draft: draft.into(),
            is_users_panel_open: true,
        }
    }

    /// Renders the Chat Canvas element tree.
    pub fn render_element(&self) -> impl IntoElement {
        let placeholder = format!("Message #{}...", self.channel_name);

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
                    // Right: Users list toggle button
                    .child(
                        div()
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
                            .text_color(if self.is_users_panel_open {
                                theme::text_primary()
                            } else {
                                theme::text_muted()
                            })
                            .child("👥"),
                    ),
            )
            // 2. Message History Stream
            .child(
                div()
                    .id("chat_message_history")
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.groups.iter().map(|group| {
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
                                    // Body of grouped messages
                                    .children(group.messages.iter().map(|msg| {
                                        div()
                                            .text_sm()
                                            .text_color(theme::text_primary())
                                            .child(msg.content.clone().unwrap_or_default())
                                    })),
                            )
                    })),
            )
            // 3. Bottom Composer Input Bar
            .child(
                div().p_4().bg(theme::bg_base()).child(
                    div()
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
                                .text_sm()
                                .text_color(if self.message_draft.is_empty() {
                                    theme::text_muted()
                                } else {
                                    theme::text_primary()
                                })
                                .child(if self.message_draft.is_empty() {
                                    placeholder
                                } else {
                                    self.message_draft.clone()
                                }),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .bg(theme::accent())
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme::text_primary())
                                .child("Send ↵"),
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
