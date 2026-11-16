//! Three-column workspace layout and UI components for Sharkord GPUI desktop client.
//! Composes Server Rail, Channel Tree (with docked Voice Bar), and Chat Canvas
//! according to the Pure Neutral Dark design specification.

pub mod channel_tree;
pub mod chat_canvas;
pub mod server_rail;
pub mod voice_bar;

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use sharkord_core::types::Message;

pub use channel_tree::{
    channel_icon, compose_channel_tree, CategorizedChannelGroup, ChannelDisplayItem,
    ChannelTreeView,
};
pub use chat_canvas::{
    format_message_timestamp, group_messages_for_feed, ChatCanvasView, MessageFeedGroup,
};
pub use server_rail::{server_initials, server_pill_height, ServerItem, ServerRailView};
pub use voice_bar::{
    voice_action_labels, voice_status_indicator_color, voice_status_text, VoiceBarState,
    VoiceBarView, VoiceConnectionState,
};

use crate::state::AppState;
use crate::theme;

/// Root workspace view composing the 3-column layout:
/// 1. Server rail (72px fixed, bg_surface)
/// 2. Channel sidebar (240px fixed, bg_subsurface) with docked voice controls at bottom
/// 3. Primary chat canvas (flex 1, header, message stream, composer)
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceView {
    pub state: AppState,
    pub voice_bar: VoiceBarState,
    pub message_draft: String,
    pub server_rail: ServerRailView,
    pub channel_tree: ChannelTreeView,
    pub chat_canvas: ChatCanvasView,
}

impl WorkspaceView {
    /// Creates a new `WorkspaceView` initialized from an `AppState`.
    pub fn new(state: AppState) -> Self {
        let voice_bar = VoiceBarState::new();
        let message_draft = String::new();

        let mut workspace = Self {
            state,
            voice_bar,
            message_draft,
            server_rail: ServerRailView::default(),
            channel_tree: ChannelTreeView::default(),
            chat_canvas: ChatCanvasView::default(),
        };

        workspace.sync_views();
        workspace
    }

    /// Synchronizes the child view states from the current `AppState`.
    pub fn sync_views(&mut self) {
        // 1. Sync Server Rail
        let current_user_name = self
            .state
            .current_user
            .as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "User".to_string());
        let current_user_status = self
            .state
            .current_user
            .as_ref()
            .and_then(|u| u.status.clone())
            .unwrap_or_else(|| "online".to_string());

        let mut servers = Vec::new();
        if let Some(ref settings) = self.state.server_settings {
            servers.push(ServerItem {
                id: settings.server_id.clone(),
                name: settings.name.clone(),
                icon_url: None,
                is_active: self.state.active_server_id.as_deref() == Some(&settings.server_id),
                has_unread: false,
            });
        }

        self.server_rail = ServerRailView {
            servers,
            is_home_active: self.state.active_server_id.is_none(),
            current_user_name,
            current_user_status,
        };

        // 2. Sync Channel Tree
        let server_name = self
            .state
            .server_settings
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Sharkord Plus".to_string());

        self.channel_tree = ChannelTreeView::new(
            server_name,
            &self.state.channels,
            self.state.active_channel_id,
        );

        // 3. Sync Chat Canvas
        let (ch_name, ch_topic) = match self.state.active_channel() {
            Some(ch) => (ch.name.clone(), ch.topic.clone()),
            None => ("select-channel".to_string(), None),
        };

        let mut users_map = HashMap::new();
        if let Some(ref u) = self.state.current_user {
            users_map.insert(u.id, u.name.clone());
        }

        let channel_messages: Vec<Message> = self
            .state
            .messages_for_active_channel()
            .into_iter()
            .cloned()
            .collect();

        self.chat_canvas = ChatCanvasView::new(
            ch_name,
            ch_topic,
            &channel_messages,
            &users_map,
            &self.message_draft,
        );
    }

    /// Selects a channel by ID and refreshes child views.
    pub fn select_channel(&mut self, channel_id: u64) {
        self.state.select_channel(channel_id);
        self.sync_views();
    }

    /// Updates the draft text in the composer.
    pub fn set_message_draft(&mut self, draft: impl Into<String>) {
        self.message_draft = draft.into();
        self.chat_canvas.message_draft = self.message_draft.clone();
    }

    /// Submits the current message draft, appending it to the state messages list
    /// and clearing the draft. Returns the created `Message` if a channel was active.
    pub fn send_message(&mut self) -> Option<Message> {
        let channel_id = self.state.active_channel_id?;
        if self.message_draft.trim().is_empty() {
            return None;
        }

        let next_id = (self.state.messages.len() as u64) + 1;
        let user_id = self.state.current_user.as_ref().map(|u| u.id);

        let msg = Message {
            id: next_id,
            channel_id,
            user_id,
            content: Some(self.message_draft.clone()),
            created_at: Some(1000), // Placeholder or current timestamp
            ..Default::default()
        };

        self.state.add_message(msg.clone());
        self.message_draft.clear();
        self.sync_views();

        Some(msg)
    }

    /// Toggles microphone mute on the voice bar.
    pub fn toggle_voice_mute(&mut self) {
        self.voice_bar.toggle_mute();
    }

    /// Toggles headphone deafen on the voice bar.
    pub fn toggle_voice_deafen(&mut self) {
        self.voice_bar.toggle_deafen();
    }

    /// Disconnects from the current voice channel.
    pub fn disconnect_voice(&mut self) {
        self.voice_bar.disconnect();
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let voice_bar_view = VoiceBarView::new(self.voice_bar.clone());

        div()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .bg(theme::bg_base())
            // Column 1: Server Rail (72px fixed)
            .child(self.server_rail.render_element())
            // Column 2: Channel Tree Sidebar (240px fixed) with docked Voice Bar
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(240.0))
                    .h_full()
                    .bg(theme::bg_subsurface())
                    .border_r_1()
                    .border_color(theme::border_hairline())
                    .child(self.channel_tree.render_element())
                    .child(voice_bar_view.render_element()),
            )
            // Column 3: Primary Chat Canvas (Flex 1)
            .child(self.chat_canvas.render_element())
    }
}
