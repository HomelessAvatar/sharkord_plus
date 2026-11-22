//! Three-column workspace layout and UI components for Sharkord GPUI desktop client.
//! Composes Server Rail, Channel Tree (with docked Voice Bar), and Chat Canvas
//! according to the Pure Neutral Dark design specification.

pub mod audio_settings;
pub mod auth_modal;
pub mod channel_tree;
pub mod chat_canvas;
pub mod member_list;
pub mod pip_player;
pub mod quick_switcher;
pub mod server_rail;
pub mod source_picker;
pub mod updater_modal;
pub mod video_tile;
pub mod voice_bar;
pub mod voice_context_menu;
pub mod voice_stage;

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::Message;

pub use audio_settings::{
    AudioSettingsState, AudioSettingsView, DEFAULT_INPUT_VOLUME, DEFAULT_OUTPUT_VOLUME,
    DEFAULT_VAD_THRESHOLD,
};
pub use auth_modal::{AuthModalState, AuthModalView, AuthTab, ServerConnectView};
pub use channel_tree::{
    channel_icon, compose_channel_tree, CategorizedChannelGroup, ChannelDisplayItem,
    ChannelTreeView,
};
pub use chat_canvas::{
    format_message_timestamp, group_messages_for_feed, render_channel_welcome_card,
    ChannelWelcomeCard, ChatCanvasView, MessageFeedGroup,
};
pub use member_list::{
    compose_member_groups, status_indicator_color, CategorizedMemberGroup, MemberListSidebarView,
};
pub use pip_player::{PipPlayerState, PipPlayerView};
pub use quick_switcher::{QuickSwitcherItem, QuickSwitcherState, QuickSwitcherView};
pub use server_rail::{server_initials, server_pill_height, ServerItem, ServerRailView};
pub use source_picker::{SourcePickerState, SourcePickerTab, SourcePickerView};
pub use updater_modal::{UpdaterBadgeView, UpdaterModalView};
pub use video_tile::{aspect_ratio_box, VideoDisplayMode, VideoTileState, VideoTileView};
pub use voice_bar::{
    voice_action_labels, voice_status_indicator_color, voice_status_text, VoiceBarState,
    VoiceBarView, VoiceConnectionState,
};
pub use voice_context_menu::{VoiceParticipantMenuState, VoiceParticipantMenuView};
pub use voice_stage::{
    calculate_grid_columns, render_empty_stage_placeholder, EmptyStagePlaceholder, VoiceStageCard,
    VoiceStageView,
};

use crate::state::AppState;
use crate::theme;

/// Active center content canvas view mode in `WorkspaceView`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ActiveContentView {
    #[default]
    Chat,
    VoiceStage,
    ScreenShare,
}

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
    pub voice_stage: VoiceStageView,
    pub member_list: MemberListSidebarView,
    pub quick_switcher: QuickSwitcherState,
    pub auth_modal: AuthModalState,
    pub active_center_view: ActiveContentView,
    pub focus_handle: Option<FocusHandle>,
}

impl WorkspaceView {
    /// Creates a new `WorkspaceView` initialized from an `AppState`.
    pub fn new(state: AppState) -> Self {
        let voice_bar = VoiceBarState::new();
        let message_draft = String::new();
        let quick_switcher = QuickSwitcherState::new(&state.channels);
        let auth_modal = AuthModalState::new();

        let active_center_view = match state.active_channel() {
            Some(ch) if ch.channel_type == sharkord_core::types::ChannelType::Voice => {
                ActiveContentView::VoiceStage
            }
            _ => state.active_center_view,
        };

        let mut workspace = Self {
            state,
            voice_bar,
            message_draft,
            server_rail: ServerRailView::default(),
            channel_tree: ChannelTreeView::default(),
            chat_canvas: ChatCanvasView::default(),
            voice_stage: VoiceStageView::default(),
            member_list: MemberListSidebarView::default(),
            quick_switcher,
            auth_modal,
            active_center_view,
            focus_handle: None,
        };

        workspace.sync_views();
        workspace
    }

    /// Creates a new `WorkspaceView` in a GPUI context with an active `FocusHandle`.
    pub fn new_in_context(state: AppState, cx: &mut Context<Self>) -> Self {
        let mut workspace = Self::new(state);
        workspace.focus_handle = Some(cx.focus_handle());
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

        // 4. Sync Voice Stage
        let (voice_ch_id, voice_ch_name) = match self.state.active_channel() {
            Some(ch) if ch.channel_type == sharkord_core::types::ChannelType::Voice => {
                (ch.id, ch.name.clone())
            }
            _ => (0, String::new()),
        };
        self.voice_stage = VoiceStageView::new(voice_ch_id, voice_ch_name, Vec::new());

        // 5. Sync Member List
        self.member_list = MemberListSidebarView::new(&self.state.users);

        // 6. Sync Quick Switcher
        self.quick_switcher.set_channels(&self.state.channels);
    }

    /// Selects a channel by ID, auto-switches center view mode based on channel type,
    /// and refreshes child views.
    pub fn select_channel(&mut self, channel_id: u64) {
        self.state.select_channel(channel_id);
        if let Some(ch) = self.state.active_channel() {
            if ch.channel_type == sharkord_core::types::ChannelType::Voice {
                self.active_center_view = ActiveContentView::VoiceStage;
            } else {
                self.active_center_view = ActiveContentView::Chat;
            }
        } else {
            self.active_center_view = ActiveContentView::Chat;
        }
        self.state.set_center_view(self.active_center_view);
        self.sync_views();
    }

    /// Sets the active center content view mode.
    pub fn set_center_view(&mut self, view: ActiveContentView) {
        self.active_center_view = view;
        self.state.set_center_view(view);
    }

    /// Returns the currently active center content view mode.
    pub fn active_center_view(&self) -> ActiveContentView {
        self.active_center_view
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

    /// Toggles the visibility of the right-hand member list sidebar drawer.
    pub fn toggle_right_sidebar(&mut self) {
        self.state.toggle_right_sidebar();
    }

    /// Toggles the quick switcher command palette modal dialog.
    pub fn toggle_quick_switcher(&mut self) {
        self.state.toggle_quick_switcher();
    }

    /// Toggles the in-app update modal dialog open state.
    pub fn toggle_update_modal(&mut self) {
        self.state.toggle_update_modal();
    }

    /// Sets connection progress state.
    pub fn set_connecting(&mut self, connecting: bool) {
        self.state.set_connecting(connecting);
        self.auth_modal.set_loading(connecting);
    }

    /// Sets authentication modal visibility.
    pub fn set_auth_modal_open(&mut self, open: bool) {
        self.state.set_auth_modal_open(open);
    }

    /// Toggles the authentication modal open state.
    pub fn toggle_auth_modal(&mut self) {
        self.state.toggle_auth_modal();
    }

    /// Bootstraps server settings, channels, and messages from the server,
    /// transitioning connection state and updating child views.
    pub fn bootstrap_from_server(
        &mut self,
        settings: impl Into<Option<sharkord_core::types::ServerSettings>>,
        channels: Vec<sharkord_core::types::Channel>,
        messages: Vec<sharkord_core::types::Message>,
    ) {
        self.state.bootstrap_from_server(settings, channels, messages);
        self.auth_modal.set_loading(false);
        self.auth_modal.clear_error();
        self.sync_views();
    }

    /// Bootstraps server state using a `BootstrapData` payload.
    pub fn bootstrap_from_data(&mut self, data: crate::network::BootstrapData) {
        self.state.bootstrap_from_data(data);
        self.auth_modal.set_loading(false);
        self.auth_modal.clear_error();
        self.sync_views();
    }

    /// Interactive handler for channel selection in the channel tree.
    pub fn on_channel_click(&mut self, channel_id: u64, cx: &mut Context<Self>) {
        self.select_channel(channel_id);
        cx.notify();
    }

    /// Interactive handler for selecting a channel by its name or string ID.
    pub fn on_channel_click_str(&mut self, channel_identifier: &str, cx: &mut Context<Self>) {
        if let Ok(id) = channel_identifier.parse::<u64>() {
            self.select_channel(id);
        } else if let Some(ch) = self
            .state
            .channels
            .iter()
            .find(|c| c.name == channel_identifier)
        {
            self.select_channel(ch.id);
        }
        cx.notify();
    }

    /// Interactive handler for server rail item selection.
    pub fn on_server_click(&mut self, server_id: Option<String>, cx: &mut Context<Self>) {
        self.state.active_server_id = server_id;
        self.sync_views();
        cx.notify();
    }

    /// Interactive handler for voice mute toggle button.
    pub fn on_mute_click(&mut self, cx: &mut Context<Self>) {
        self.toggle_voice_mute();
        cx.notify();
    }

    /// Interactive handler for voice deafen toggle button.
    pub fn on_deafen_click(&mut self, cx: &mut Context<Self>) {
        self.toggle_voice_deafen();
        cx.notify();
    }

    /// Interactive handler for voice disconnect button.
    pub fn on_disconnect_click(&mut self, cx: &mut Context<Self>) {
        self.disconnect_voice();
        cx.notify();
    }

    /// Interactive handler for keystrokes in the chat composer input field.
    pub fn on_composer_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if key == "enter" {
            self.send_message();
        } else if key == "backspace" {
            self.message_draft.pop();
            self.chat_canvas.message_draft = self.message_draft.clone();
        } else if let Some(ref char_str) = event.keystroke.key_char {
            self.message_draft.push_str(char_str);
            self.chat_canvas.message_draft = self.message_draft.clone();
        } else if key.len() == 1 {
            self.message_draft.push_str(key);
            self.chat_canvas.message_draft = self.message_draft.clone();
        }
        cx.notify();
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_handle.is_none() {
            self.focus_handle = Some(cx.focus_handle());
        }

        let voice_bar_view = VoiceBarView::new(self.voice_bar.clone());

        let center_element: AnyElement = match self.active_center_view {
            ActiveContentView::Chat => self
                .chat_canvas
                .render_interactive(self.focus_handle.as_ref(), cx)
                .into_any_element(),
            ActiveContentView::VoiceStage => self.voice_stage.render_element().into_any_element(),
            ActiveContentView::ScreenShare => {
                let tile_state = VideoTileState::new(0, "Screen Share".to_string());
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .items_center()
                    .justify_center()
                    .bg(theme::bg_base())
                    .child(VideoTileView::new(tile_state).render_element())
                    .into_any_element()
            }
        };

        let mut root = div()
            .relative()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .bg(theme::bg_base())
            // Column 1: Server Rail (72px fixed)
            .child(self.server_rail.render_interactive(cx))
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
                    .child(self.channel_tree.render_interactive(cx))
                    .child(voice_bar_view.render_interactive(cx)),
            )
            // Column 3: Dynamic Center Content Canvas (Chat, VoiceStage, or ScreenShare)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(center_element),
            );

        // Column 4: Right-hand Member List Sidebar Drawer (240px fixed, conditionally rendered)
        if self.state.right_sidebar_open {
            root = root.child(self.member_list.render_element());
        }

        if let Some(ref pip) = self.state.active_pip_player {
            if pip.is_visible {
                root = root.child(PipPlayerView::new(pip.clone()).render_element());
            }
        }

        if self.state.is_quick_switcher_open {
            root = root.child(QuickSwitcherView::new(self.quick_switcher.clone()).render_element());
        }

        if self.state.is_update_modal_open {
            root = root.child(UpdaterModalView::new(&self.state.update_status).render_element());
        }

        if self.state.is_auth_modal_open {
            root = root.child(AuthModalView::new(self.auth_modal.clone()).render_element());
        }

        if self.state.is_connecting && !self.state.is_auth_modal_open {
            root = root.child(
                div()
                    .absolute()
                    .top_4()
                    .right_4()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1p5()
                    .rounded_full()
                    .bg(theme::bg_elevated())
                    .border_1()
                    .border_color(theme::border_subtle())
                    .shadow_lg()
                    .child(
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(theme::accent()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_primary())
                            .child("Connecting to server..."),
                    ),
            );
        }

        root
    }
}
