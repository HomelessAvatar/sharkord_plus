//! Direct Messages (DMs) sidebar view and navigation component.
//! Adheres strictly to Pure Neutral Dark design specification.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// Represents an individual Direct Message conversation with a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DmConversation {
    pub channel_id: u64,
    pub user_id: u64,
    pub username: String,
    pub display_name: Option<String>,
    pub status: String,
    pub last_message: Option<String>,
    pub unread_count: u32,
}

impl DmConversation {
    /// Returns the display name if available and non-empty, otherwise fallback to the username.
    pub fn effective_name(&self) -> &str {
        if let Some(ref name) = self.display_name {
            if !name.trim().is_empty() {
                return name.as_str();
            }
        }
        &self.username
    }

    /// Generates compact initials for avatar display.
    pub fn initials(&self) -> String {
        let name = self.effective_name();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return "?".to_string();
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() > 1 {
            words
                .into_iter()
                .take(2)
                .filter_map(|w| w.chars().next())
                .flat_map(|c| c.to_uppercase())
                .collect()
        } else {
            let chars: Vec<char> = trimmed.chars().collect();
            if chars.len() == 1 {
                chars[0].to_uppercase().to_string()
            } else {
                chars
                    .into_iter()
                    .take(2)
                    .flat_map(|c| c.to_uppercase())
                    .collect()
            }
        }
    }

    /// Resolves the presence indicator color according to the user status.
    pub fn status_color(&self) -> Hsla {
        match self.status.as_str() {
            "online" => theme::status_online(),
            "idle" => theme::status_idle(),
            "dnd" => theme::status_dnd(),
            _ => theme::status_offline(),
        }
    }
}

/// State for the Direct Messages sidebar containing active selection and conversation list.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DmSidebarState {
    pub active_dm_channel_id: Option<u64>,
    pub conversations: Vec<DmConversation>,
}

impl DmSidebarState {
    /// Creates a new, empty `DmSidebarState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the currently active DM channel ID.
    pub fn set_active_dm(&mut self, channel_id: Option<u64>) {
        self.active_dm_channel_id = channel_id;
    }

    /// Appends a new conversation or updates an existing one if channel_id matches.
    pub fn add_conversation(&mut self, conv: DmConversation) {
        if let Some(existing) = self
            .conversations
            .iter_mut()
            .find(|c| c.channel_id == conv.channel_id)
        {
            *existing = conv;
        } else {
            self.conversations.push(conv);
        }
    }

    /// Removes a conversation by channel ID, returning the removed item if found.
    pub fn remove_conversation(&mut self, channel_id: u64) -> Option<DmConversation> {
        if let Some(pos) = self
            .conversations
            .iter()
            .position(|c| c.channel_id == channel_id)
        {
            if self.active_dm_channel_id == Some(channel_id) {
                self.active_dm_channel_id = None;
            }
            Some(self.conversations.remove(pos))
        } else {
            None
        }
    }

    /// Clears the unread count for the given channel ID.
    pub fn mark_read(&mut self, channel_id: u64) {
        if let Some(conv) = self
            .conversations
            .iter_mut()
            .find(|c| c.channel_id == channel_id)
        {
            conv.unread_count = 0;
        }
    }

    /// Returns a reference to the active conversation if one is selected.
    pub fn active_conversation(&self) -> Option<&DmConversation> {
        let active_id = self.active_dm_channel_id?;
        self.conversations.iter().find(|c| c.channel_id == active_id)
    }

    /// Computes the total unread messages across all DM conversations.
    pub fn unread_total(&self) -> u32 {
        self.conversations.iter().map(|c| c.unread_count).sum()
    }

    /// Looks up a conversation by target user ID.
    pub fn find_by_user_id(&self, user_id: u64) -> Option<&DmConversation> {
        self.conversations.iter().find(|c| c.user_id == user_id)
    }
}

/// View component rendering the Direct Messages navigation sidebar (Column 2).
#[derive(Debug, Clone, PartialEq)]
pub struct DirectMessagesSidebarView {
    pub state: DmSidebarState,
}

impl DirectMessagesSidebarView {
    /// Creates a new `DirectMessagesSidebarView` from the provided state.
    pub fn new(state: DmSidebarState) -> Self {
        Self { state }
    }

    /// Returns the number of DM conversations.
    pub fn conversations_count(&self) -> usize {
        self.state.conversations.len()
    }

    /// Renders the DM sidebar element tree non-interactively.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the DM sidebar element tree with interactive handlers wired to `WorkspaceView`.
    pub fn render_interactive(
        &self,
        cx: &mut Context<crate::ui::WorkspaceView>,
    ) -> impl IntoElement {
        self.render_internal(Some(cx))
    }

    fn render_internal(
        &self,
        mut cx: Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> impl IntoElement {
        let active_dm_id = self.state.active_dm_channel_id;

        div()
            .id("direct_messages_sidebar")
            .flex()
            .flex_col()
            .w(px(240.0))
            .h_full()
            .bg(theme::bg_subsurface())
            .border_r_1()
            .border_color(theme::border_hairline())
            // 1. Header Bar with "Direct Messages" and "Start DM" button
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(48.0))
                    .px_4()
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child("Direct Messages"),
                    )
                    .child(
                        div()
                            .id("start_dm_action_btn")
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded_md()
                            .bg(theme::bg_elevated())
                            .hover(|s| s.bg(theme::bg_surface_4()))
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child("+"),
                    ),
            )
            // 2. Search / Quick Switcher Trigger Pill
            .child(
                div()
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .id("dm_search_trigger")
                            .cursor_pointer()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(theme::bg_elevated())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .hover(|s| s.border_color(theme::border_subtle()))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("🔍")
                                    .child("Find or start a DM"),
                            )
                            .child(
                                div()
                                    .px_1()
                                    .rounded_sm()
                                    .bg(theme::bg_surface_4())
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Ctrl+K"),
                            ),
                    ),
            )
            // 3. Section Title
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .pt_2()
                    .pb_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_muted())
                            .child("DIRECT MESSAGES"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!("{}", self.state.conversations.len())),
                    ),
            )
            // 4. Conversation List Items
            .child(
                div()
                    .id("dm_conversations_scroll_list")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .overflow_y_scroll()
                    .children(if self.state.conversations.is_empty() {
                        vec![div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .py_8()
                            .px_4()
                            .text_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_lg()
                                    .child("💬"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("No direct messages yet."),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::accent())
                                    .child("Click + to start a chat!"),
                            )
                            .into_any_element()]
                    } else {
                        self.state
                            .conversations
                            .iter()
                            .map(|conv| {
                                let is_active = active_dm_id == Some(conv.channel_id);
                                let status_color = conv.status_color();
                                let initials = conv.initials();
                                let ch_id = conv.channel_id;
                                let u_id = conv.user_id;
                                let u_name = conv.username.clone();
                                let disp_name = conv.effective_name().to_string();
                                let u_status = conv.status.clone();

                                let mut item = div()
                                    .id(ElementId::Name(format!("dm_item_{}", conv.channel_id).into()))
                                    .cursor_pointer()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .px_2()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(if is_active {
                                        theme::bg_elevated()
                                    } else {
                                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                                    })
                                    .hover(|s| {
                                        if is_active {
                                            s
                                        } else {
                                            s.bg(theme::bg_elevated())
                                        }
                                    })
                                    // Avatar circle with presence dot
                                    .child(
                                        div()
                                            .relative()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .w(px(32.0))
                                            .h(px(32.0))
                                            .rounded_full()
                                            .bg(theme::bg_surface_4())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme::text_primary())
                                            .child(initials)
                                            .child(
                                                div()
                                                    .absolute()
                                                    .bottom_0()
                                                    .right_0()
                                                    .w(px(10.0))
                                                    .h(px(10.0))
                                                    .rounded_full()
                                                    .border_1()
                                                    .border_color(theme::bg_subsurface())
                                                    .bg(status_color),
                                            ),
                                    )
                                    // Details column
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .flex_1()
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(if is_active {
                                                        theme::text_primary()
                                                    } else {
                                                        theme::text_secondary()
                                                    })
                                                    .child(conv.effective_name().to_string()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child(
                                                        conv.last_message
                                                            .clone()
                                                            .unwrap_or_else(|| "No messages yet".to_string()),
                                                    ),
                                            ),
                                    );

                                // Unread badge
                                if conv.unread_count > 0 {
                                    item = item.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .px_1p5()
                                            .py_0p5()
                                            .rounded_full()
                                            .bg(theme::status_dnd())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(gpui::rgb(0xffffff))
                                            .child(format!("{}", conv.unread_count)),
                                    );
                                }

                                if let Some(ref mut cx) = cx {
                                    let ch_id_val = ch_id;
                                    let _u_id_val = u_id;
                                    let _u_name_val = u_name.clone();
                                    let _disp_name_val = disp_name.clone();
                                    let _u_status_val = u_status.clone();

                                    item = item.on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |view, _, _window, cx| {
                                            view.select_channel(ch_id_val);
                                            cx.notify();
                                        }),
                                    );
                                }

                                item.into_any_element()
                            })
                            .collect()
                    }),
            )
    }
}

impl Render for DirectMessagesSidebarView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
