//! Chat Canvas view component (Flex 1).
//! Renders the channel header with topic and member toggle, scrollable message stream
//! with sender avatars and timestamps (grouped within 5-minute windows), and docked bottom
//! composer input bar.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use sharkord_core::types::{InteractiveComponent, Message, PluginCommand};

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
            available_commands: default_slash_commands(),
        }
    }

    pub fn with_commands(mut self, commands: Vec<PluginCommand>) -> Self {
        self.available_commands = commands;
        self
    }

    pub fn matching_slash_commands(&self) -> Vec<PluginCommand> {
        if !self.message_draft.starts_with('/') {
            return Vec::new();
        }
        let query = self.message_draft[1..].trim().to_lowercase();
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
            .child(if self.groups.is_empty() {
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
            } else {
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
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(theme::text_primary())
                                                    .child(msg.content.clone().unwrap_or_default()),
                                            )
                                            .children(render_interactive_components(
                                                &msg.components,
                                            ))
                                    })),
                            )
                    }))
            })
            // 3. Bottom Composer Input Bar with Slash Command Autocomplete Popover
            .child(
                div().p_4().bg(theme::bg_base()).child(
                    div()
                        .flex()
                        .flex_col()
                        .children(if self.message_draft.starts_with('/') {
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
                                );

                            let mut send_btn = div()
                                .id("chat_composer_send_btn")
                                .cursor_pointer()
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
                                .child("Send ↵");

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
                        }),
                ),
            )
    }
}

impl Render for ChatCanvasView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
