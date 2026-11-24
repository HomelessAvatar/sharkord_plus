//! Three-column workspace layout and UI components for Sharkord GPUI desktop client.
//! Composes Server Rail, Channel Tree (with docked Voice Bar), and Chat Canvas
//! according to the Pure Neutral Dark design specification.

pub mod audio_settings;
pub mod auth_modal;
pub mod channel_admin_modal;
pub mod channel_tree;
pub mod chat_canvas;
pub mod dm_view;
pub mod emoji_picker;
pub mod lightbox_modal;
pub mod markdown;
pub mod member_list;
pub mod pip_player;
pub mod quick_switcher;
pub mod server_rail;
pub mod server_settings_modal;
pub mod source_picker;
pub mod updater_modal;
pub mod user_profile_modal;
pub mod user_settings_modal;
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
pub use channel_admin_modal::{
    CreateChannelModalState, CreateChannelModalView, EditChannelModalState, EditChannelModalView,
};
pub use channel_tree::{
    channel_icon, compose_channel_tree, CategorizedChannelGroup, ChannelDisplayItem,
    ChannelTreeView,
};
pub use chat_canvas::{
    format_message_timestamp, format_reactors_tooltip, group_messages_for_feed,
    render_channel_welcome_card, render_copy_icon, render_edit_icon, render_pin_icon,
    render_react_icon, render_reply_icon, render_trash_icon, ChannelWelcomeCard, ChatCanvasView,
    ComposerAction, ComposerTextarea, MessageActionToolbar, MessageFeedGroup, ReactionBadge,
    ReplyTarget,
};
pub use dm_view::{DirectMessagesSidebarView, DmConversation, DmSidebarState};
pub use emoji_picker::{EmojiCategory, EmojiItem, EmojiPickerState, EmojiPickerView};
pub use lightbox_modal::{LightboxModalView, LightboxState};
pub use markdown::{
    parse_markdown_tokens, render_code_block, render_markdown_element, MarkdownToken, SpoilerState,
};
pub use member_list::{
    compose_member_groups, status_indicator_color, CategorizedMemberGroup, MemberListSidebarView,
};
pub use pip_player::{PipPlayerState, PipPlayerView};
pub use quick_switcher::{
    default_command_palette_items, QuickSwitcherItem, QuickSwitcherItemType, QuickSwitcherState,
    QuickSwitcherView,
};
pub use server_rail::{server_initials, server_pill_height, ServerItem, ServerRailView};
pub use server_settings_modal::{
    ServerSettingsModalState, ServerSettingsModalView, ServerSettingsTab,
};
pub use source_picker::{SourcePickerState, SourcePickerTab, SourcePickerView};
pub use updater_modal::{UpdaterBadgeView, UpdaterModalView};
pub use user_profile_modal::{
    UserProfileCardState, UserProfileModalView, UserProfilePopover, UserProfilePopoverView,
    UserProfileState,
};
pub use user_settings_modal::{
    InputMode, SettingsTab, UserSettingsModalView, UserSettingsState,
};
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
pub use crate::media::{
    CaptureSource, CaptureSourceType, LiveVoiceClient, LiveVoiceState, ScreenCaptureEngine,
    VideoDecoderPipeline, VideoFrame, VoiceInputMode,
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
    pub live_voice: LiveVoiceState,
    pub message_draft: String,
    pub server_rail: ServerRailView,
    pub channel_tree: ChannelTreeView,
    pub dm_sidebar: DirectMessagesSidebarView,
    pub user_profile_modal: UserProfileCardState,
    pub chat_canvas: ChatCanvasView,
    pub voice_stage: VoiceStageView,
    pub member_list: MemberListSidebarView,
    pub quick_switcher: QuickSwitcherState,
    pub auth_modal: AuthModalState,
    pub create_channel_modal: CreateChannelModalState,
    pub edit_channel_modal: EditChannelModalState,
    pub server_settings_modal: ServerSettingsModalState,
    pub user_settings_modal: UserSettingsState,
    pub active_center_view: ActiveContentView,
    pub focus_handle: Option<FocusHandle>,
}

impl WorkspaceView {
    /// Creates a new `WorkspaceView` initialized from an `AppState`.
    pub fn new(state: AppState) -> Self {
        let voice_bar = VoiceBarState::new();
        let mut live_voice = LiveVoiceState::new();
        if state.voice_bar_state.connection != VoiceConnectionState::Disconnected {
            live_voice.is_connected = true;
            if let VoiceConnectionState::Connected {
                channel_id,
                ref channel_name,
                ping_ms,
            } = state.voice_bar_state.connection
            {
                live_voice.channel_id = Some(channel_id);
                live_voice.channel_name = Some(channel_name.clone());
                live_voice.ping_ms = ping_ms;
            }
            live_voice.is_muted = state.voice_bar_state.is_muted;
            live_voice.is_deafened = state.voice_bar_state.is_deafened;
        }
        let message_draft = String::new();
        let mut quick_switcher = QuickSwitcherState::new();
        quick_switcher.set_channels(&state.channels);
        quick_switcher.set_members(&state.users);
        let auth_modal = AuthModalState::new();
        let dm_sidebar = DirectMessagesSidebarView::new(DmSidebarState::new());
        let user_profile_modal = UserProfileCardState::new();
        let create_channel_modal = CreateChannelModalState::new();
        let edit_channel_modal = EditChannelModalState::new();
        let mut server_settings_modal = ServerSettingsModalState::new();
        if let Some(ref settings) = state.server_settings {
            server_settings_modal.set_server_name(&settings.name);
        }
        server_settings_modal.set_members(state.users.clone());

        let mut user_settings_modal = UserSettingsState::new();
        if let Some(ref u) = state.current_user {
            user_settings_modal.set_username(&u.name);
            user_settings_modal.set_display_name(&u.name);
            if let Some(ref bio) = u.bio {
                user_settings_modal.set_bio(bio);
            }
        }

        let active_center_view = match state.active_channel() {
            Some(ch) if ch.channel_type == sharkord_core::types::ChannelType::Voice => {
                ActiveContentView::VoiceStage
            }
            _ => state.active_center_view,
        };

        let mut workspace = Self {
            state,
            voice_bar,
            live_voice,
            message_draft,
            server_rail: ServerRailView::default(),
            channel_tree: ChannelTreeView::default(),
            dm_sidebar,
            user_profile_modal,
            chat_canvas: ChatCanvasView::default(),
            voice_stage: VoiceStageView::default(),
            member_list: MemberListSidebarView::default(),
            quick_switcher,
            auth_modal,
            create_channel_modal,
            edit_channel_modal,
            server_settings_modal,
            user_settings_modal,
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

        let is_home = self.state.active_server_id.is_none();
        self.server_rail = ServerRailView {
            servers,
            is_home_active: is_home,
            is_home_selected: is_home,
            current_user_name,
            current_user_status,
        };

        // Populate DMs from state.channels where is_dm is true
        for ch in &self.state.channels {
            if ch.is_dm {
                let user = self.state.users.iter().find(|u| u.name == ch.name);
                let (user_id, display_name, status) = if let Some(u) = user {
                    (
                        u.id,
                        u.name.clone(),
                        u.status.clone().unwrap_or_else(|| "offline".to_string()),
                    )
                } else {
                    (ch.id, ch.name.clone(), "offline".to_string())
                };
                self.dm_sidebar.state.add_conversation(DmConversation {
                    channel_id: ch.id,
                    user_id,
                    username: ch.name.clone(),
                    display_name: Some(display_name),
                    status,
                    last_message: None,
                    unread_count: 0,
                });
            }
        }
        self.dm_sidebar.state.active_dm_channel_id = self.state.active_channel_id;

        // 2. Sync Channel Tree
        let server_name = self
            .state
            .server_settings
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "VibeShark".to_string());

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
        for u in &self.state.users {
            users_map.insert(u.id, u.name.clone());
        }
        if let Some(ref u) = self.state.current_user {
            users_map.insert(u.id, u.name.clone());
        }

        let channel_messages: Vec<Message> = self
            .state
            .messages_for_active_channel()
            .into_iter()
            .cloned()
            .collect();

        let existing_pending = std::mem::take(&mut self.chat_canvas.pending_attachments);
        let existing_lightbox = self.chat_canvas.lightbox.clone();
        let existing_reply = self.chat_canvas.reply_target.clone();
        let existing_editing_id = self.chat_canvas.editing_message_id;
        let existing_editing_text = self.chat_canvas.editing_text.clone();
        let existing_picker = self.chat_canvas.emoji_picker.clone();

        self.chat_canvas = ChatCanvasView::new(
            ch_name,
            ch_topic,
            &channel_messages,
            &users_map,
            &self.message_draft,
        );
        self.chat_canvas.pending_attachments = existing_pending;
        self.chat_canvas.lightbox = existing_lightbox;
        self.chat_canvas.reply_target = existing_reply;
        self.chat_canvas.editing_message_id = existing_editing_id;
        self.chat_canvas.editing_text = existing_editing_text;
        self.chat_canvas.emoji_picker = existing_picker;
        self.chat_canvas.current_user_id = self.state.current_user.as_ref().map(|u| u.id);

        // 4. Sync Voice Stage
        let (voice_ch_id, voice_ch_name) = match self.state.active_channel() {
            Some(ch) if ch.channel_type == sharkord_core::types::ChannelType::Voice => {
                (ch.id, ch.name.clone())
            }
            _ => (0, String::new()),
        };
        let mut participants = Vec::new();
        if voice_ch_id > 0 {
            for user in &self.state.users {
                participants.push(VoiceStageCard {
                    user_id: user.id,
                    username: user.name.clone(),
                    avatar_url: None,
                    is_speaking: self.live_voice.is_user_speaking(user.id),
                    is_muted: self.state.local_muted_users.contains(&user.id),
                    is_deafened: false,
                    volume: self.state.user_volumes.get(&user.id).copied().unwrap_or(1.0),
                });
            }
        }
        self.voice_stage = VoiceStageView::new(voice_ch_id, voice_ch_name, participants);

        // 5. Sync Member List
        self.member_list = MemberListSidebarView::new(&self.state.users);

        // 6. Sync Quick Switcher
        self.quick_switcher.set_channels(&self.state.channels);
        self.quick_switcher.set_members(&self.state.users);

        // 7. Sync Server Settings
        if let Some(ref settings) = self.state.server_settings {
            self.server_settings_modal.set_server_name(&settings.name);
        }
        self.server_settings_modal.set_members(self.state.users.clone());

        // 8. Sync User Settings
        if let Some(ref u) = self.state.current_user {
            self.user_settings_modal.set_username(&u.name);
            self.user_settings_modal.set_display_name(&u.name);
            if let Some(ref bio) = u.bio {
                self.user_settings_modal.set_bio(bio);
            }
        }
    }

    /// Selects a channel by ID, auto-switches center view mode based on channel type,
    /// and refreshes child views.
    pub fn select_channel(&mut self, channel_id: u64) {
        self.state.select_channel(channel_id);
        if let Some(ch) = self.state.active_channel() {
            if ch.channel_type == sharkord_core::types::ChannelType::Voice {
                self.active_center_view = ActiveContentView::VoiceStage;
                self.live_voice.connect_named(ch.id, ch.name.clone());
                self.voice_bar.connection = VoiceConnectionState::Connected {
                    channel_id: ch.id,
                    channel_name: ch.name.clone(),
                    ping_ms: self.live_voice.ping_ms,
                };
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
        self.chat_canvas.set_message_draft(&self.message_draft);
    }

    /// Submits the current message draft, appending it to the state messages list
    /// and clearing the draft. Returns the created `Message` if a channel was active.
    pub fn send_message(&mut self) -> Option<Message> {
        let channel_id = self.state.active_channel_id?;
        if self.message_draft.trim().is_empty() && self.chat_canvas.pending_attachments.is_empty() {
            return None;
        }

        let next_id = (self.state.messages.len() as u64) + 1;
        let user_id = self.state.current_user.as_ref().map(|u| u.id);

        let files: Vec<sharkord_core::types::MessageFile> = self
            .chat_canvas
            .pending_attachments
            .iter()
            .enumerate()
            .map(|(idx, att)| sharkord_core::types::MessageFile {
                id: next_id * 100 + idx as u64,
                name: att.filename.clone(),
                original_name: att.filename.clone(),
                size: att.size_bytes as i64,
                mime_type: if att.is_image {
                    "image/png".to_string()
                } else {
                    "application/octet-stream".to_string()
                },
                extension: att
                    .path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string(),
                url: att.uploaded_url.clone(),
            })
            .collect();

        let reply_to_message_id = self.chat_canvas.reply_target.as_ref().map(|r| r.message_id);

        let msg = Message {
            id: next_id,
            channel_id,
            user_id,
            content: Some(self.message_draft.clone()),
            reply_to_message_id,
            created_at: Some(1000), // Placeholder or current timestamp
            files,
            ..Default::default()
        };

        self.state.add_message(msg.clone());
        self.message_draft.clear();
        self.chat_canvas.clear_pending_attachments();
        self.chat_canvas.clear_reply_target();
        self.sync_views();

        Some(msg)
    }

    /// Commits active in-place edit from the chat canvas to the state.
    pub fn save_inline_edit(&mut self) -> Option<(u64, String)> {
        let (msg_id, text) = self.chat_canvas.save_inline_edit()?;
        self.state.edit_message_content(msg_id, &text);
        self.sync_views();
        Some((msg_id, text))
    }

    /// Toggles an emoji reaction on a message for the current user.
    pub fn toggle_reaction(&mut self, message_id: u64, emoji: &str) {
        if let Some(ref u) = self.state.current_user {
            self.state.toggle_message_reaction(message_id, u.id, emoji);
            self.sync_views();
        }
    }

    /// Toggles the pinned status of a message.
    pub fn toggle_message_pin(&mut self, message_id: u64) {
        self.state.toggle_message_pin(message_id);
        self.sync_views();
    }

    /// Deletes a message from state and refreshes child views.
    pub fn delete_message(&mut self, message_id: u64) {
        self.state.delete_message(message_id);
        self.sync_views();
    }

    /// Toggles microphone mute on the voice bar and live voice engine.
    pub fn toggle_voice_mute(&mut self) {
        self.voice_bar.toggle_mute();
        self.live_voice.toggle_mute();
        self.state.toggle_voice_mute();
    }

    /// Toggles headphone deafen on the voice bar and live voice engine.
    pub fn toggle_voice_deafen(&mut self) {
        self.voice_bar.toggle_deafen();
        self.live_voice.toggle_deafen();
        self.state.toggle_voice_deafen();
    }

    /// Disconnects from the current voice channel.
    pub fn disconnect_voice(&mut self) {
        self.voice_bar.disconnect();
        self.live_voice.disconnect();
        self.state.disconnect_voice();
    }

    /// Toggles the visibility of the right-hand member list sidebar drawer.
    pub fn toggle_right_sidebar(&mut self) {
        self.state.toggle_right_sidebar();
    }

    /// Toggles the quick switcher command palette modal dialog.
    pub fn toggle_quick_switcher(&mut self) {
        self.state.toggle_quick_switcher();
        if self.state.is_quick_switcher_open {
            self.quick_switcher.open();
        } else {
            self.quick_switcher.close();
        }
    }

    /// Interactive handler for toggling a channel category collapsible state.
    pub fn toggle_channel_category(&mut self, category_name: &str, cx: &mut Context<Self>) {
        self.channel_tree.toggle_category(category_name);
        cx.notify();
    }

    /// Interactive handler for selecting an item in the quick switcher command palette.
    pub fn select_quick_switcher_item(&mut self, item: QuickSwitcherItem, cx: &mut Context<Self>) {
        match item.item_type {
            QuickSwitcherItemType::Channel | QuickSwitcherItemType::VoiceRoom => {
                self.select_channel(item.id);
            }
            QuickSwitcherItemType::Command => match item.title.as_str() {
                "/mute" => self.toggle_voice_mute(),
                "/deafen" => self.toggle_voice_deafen(),
                "/settings" => self.open_user_settings(),
                "/status" => {}
                _ => {}
            },
            QuickSwitcherItemType::Member => {}
        }
        self.state.is_quick_switcher_open = false;
        self.quick_switcher.close();
        self.sync_views();
        cx.notify();
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
        self.state
            .bootstrap_from_server(settings, channels, messages);
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
        self.state.active_server_id = server_id.clone();
        let is_home = server_id.is_none();
        self.server_rail.set_home_selected(is_home);
        self.sync_views();
        cx.notify();
    }

    /// Opens the user profile modal card for the specified user.
    pub fn open_user_profile(
        &mut self,
        user_id: u64,
        username: &str,
        display_name: &str,
        status: &str,
    ) {
        self.user_profile_modal
            .open(user_id, username, display_name, status);
        if let Some(user) = self.state.users.iter().find(|u| u.id == user_id) {
            if let Some(ref bio) = user.bio {
                self.user_profile_modal.set_bio(bio.clone());
            }
            if !user.role_ids.is_empty() {
                self.user_profile_modal.set_role("Member");
            }
        }
    }

    /// Closes the user profile modal card.
    pub fn close_user_profile(&mut self) {
        self.user_profile_modal.close();
    }

    /// Opens or selects a direct message conversation with a user.
    pub fn open_dm_conversation(&mut self, user_id: u64, username: &str) {
        if let Some(conv) = self.dm_sidebar.state.find_by_user_id(user_id) {
            let ch_id = conv.channel_id;
            self.select_channel(ch_id);
        } else {
            let next_ch_id = (self.state.channels.len() as u64) + 1000;
            let dm_ch = sharkord_core::types::Channel {
                id: next_ch_id,
                name: username.to_string(),
                channel_type: sharkord_core::types::ChannelType::Text,
                private: true,
                is_dm: true,
                ..Default::default()
            };
            self.state.channels.push(dm_ch);
            self.dm_sidebar.state.add_conversation(DmConversation {
                channel_id: next_ch_id,
                user_id,
                username: username.to_string(),
                display_name: Some(username.to_string()),
                status: "online".to_string(),
                last_message: None,
                unread_count: 0,
            });
            self.select_channel(next_ch_id);
        }
        self.server_rail.set_home_selected(true);
        self.state.active_server_id = None;
        self.sync_views();
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
        let is_shift = event.keystroke.modifiers.shift;
        if key == "enter" && !is_shift {
            self.send_message();
        } else if key == "enter" && is_shift {
            self.message_draft.push('\n');
            self.chat_canvas.set_message_draft(&self.message_draft);
        } else if key == "backspace" {
            self.message_draft.pop();
            self.chat_canvas.set_message_draft(&self.message_draft);
        } else if let Some(ref char_str) = event.keystroke.key_char {
            self.message_draft.push_str(char_str);
            self.chat_canvas.set_message_draft(&self.message_draft);
        } else if key.len() == 1 {
            self.message_draft.push_str(key);
            self.chat_canvas.set_message_draft(&self.message_draft);
        }
        cx.notify();
    }

    /// Opens the Create Channel modal for a category.
    pub fn open_create_channel(&mut self, category: impl Into<String>) {
        self.create_channel_modal.open(category);
    }

    /// Closes the Create Channel modal.
    pub fn close_create_channel(&mut self) {
        self.create_channel_modal.close();
    }

    /// Interactive handler to open the Create Channel modal.
    pub fn open_create_channel_modal(&mut self, category: &str, cx: &mut Context<Self>) {
        self.open_create_channel(category);
        cx.notify();
    }

    /// Interactive handler to close the Create Channel modal.
    pub fn close_create_channel_modal(&mut self, cx: &mut Context<Self>) {
        self.close_create_channel();
        cx.notify();
    }

    /// Submits channel creation, appending the new channel to state and closing modal.
    pub fn submit_create_channel(&mut self) -> Option<sharkord_core::types::Channel> {
        let name = self.create_channel_modal.name.trim().to_string();
        if name.is_empty() {
            return None;
        }

        let next_id = (self.state.channels.len() as u64) + 1;
        let ch = sharkord_core::types::Channel {
            id: next_id,
            name,
            channel_type: self.create_channel_modal.channel_type,
            topic: None,
            private: false,
            is_dm: false,
            position: self.state.channels.len() as i32,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        };

        self.state.channels.push(ch.clone());
        self.create_channel_modal.reset();
        self.select_channel(next_id);
        self.sync_views();
        Some(ch)
    }

    /// Interactive handler to submit channel creation.
    pub fn submit_create_channel_interactive(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Option<sharkord_core::types::Channel> {
        let res = self.submit_create_channel();
        cx.notify();
        res
    }

    /// Opens the Edit Channel modal for a specific channel.
    pub fn open_edit_channel(&mut self, channel_id: u64) {
        if let Some(ch) = self.state.channels.iter().find(|c| c.id == channel_id) {
            self.edit_channel_modal.open(
                ch.id,
                &ch.name,
                ch.topic.as_deref().unwrap_or(""),
            );
        }
    }

    /// Closes the Edit Channel modal.
    pub fn close_edit_channel(&mut self) {
        self.edit_channel_modal.close();
    }

    /// Interactive handler to open the Edit Channel modal.
    pub fn open_edit_channel_modal(&mut self, channel_id: u64, cx: &mut Context<Self>) {
        self.open_edit_channel(channel_id);
        cx.notify();
    }

    /// Interactive handler to close the Edit Channel modal.
    pub fn close_edit_channel_modal(&mut self, cx: &mut Context<Self>) {
        self.close_edit_channel();
        cx.notify();
    }

    /// Submits channel rename and topic update to state.
    pub fn submit_edit_channel(&mut self) -> Option<u64> {
        let ch_id = self.edit_channel_modal.channel_id;
        let new_name = self.edit_channel_modal.name.trim().to_string();
        let new_topic = self.edit_channel_modal.topic.trim().to_string();

        if let Some(ch) = self.state.channels.iter_mut().find(|c| c.id == ch_id) {
            if !new_name.is_empty() {
                ch.name = new_name;
            }
            ch.topic = if new_topic.is_empty() {
                None
            } else {
                Some(new_topic)
            };
        }

        self.edit_channel_modal.close();
        self.sync_views();
        Some(ch_id)
    }

    /// Submits channel deletion and removes it from state.
    pub fn submit_delete_channel(&mut self) -> Option<u64> {
        if !self.edit_channel_modal.confirm_delete {
            return None;
        }

        let ch_id = self.edit_channel_modal.channel_id;
        self.state.channels.retain(|c| c.id != ch_id);
        if self.state.active_channel_id == Some(ch_id) {
            self.state.active_channel_id = self.state.channels.first().map(|c| c.id);
        }

        self.edit_channel_modal.reset();
        self.sync_views();
        Some(ch_id)
    }

    /// Opens the Server Settings modal.
    pub fn open_server_settings(&mut self) {
        self.server_settings_modal.open();
        if let Some(ref s) = self.state.server_settings {
            self.server_settings_modal.set_server_name(&s.name);
        }
        self.server_settings_modal.set_members(self.state.users.clone());
    }

    /// Closes the Server Settings modal.
    pub fn close_server_settings(&mut self) {
        self.server_settings_modal.close();
    }

    /// Interactive handler to open the Server Settings modal.
    pub fn open_server_settings_modal(&mut self, cx: &mut Context<Self>) {
        self.open_server_settings();
        cx.notify();
    }

    /// Interactive handler to close the Server Settings modal.
    pub fn close_server_settings_modal(&mut self, cx: &mut Context<Self>) {
        self.close_server_settings();
        cx.notify();
    }

    /// Opens the User Settings modal dialog.
    pub fn open_user_settings(&mut self) {
        self.user_settings_modal.open();
        if let Some(ref u) = self.state.current_user {
            self.user_settings_modal.set_username(&u.name);
            self.user_settings_modal.set_display_name(&u.name);
            if let Some(ref bio) = u.bio {
                self.user_settings_modal.set_bio(bio);
            }
        }
    }

    /// Closes the User Settings modal dialog.
    pub fn close_user_settings(&mut self) {
        self.user_settings_modal.close();
    }

    /// Interactive handler to open the User Settings modal.
    pub fn open_user_settings_modal(&mut self, cx: &mut Context<Self>) {
        self.open_user_settings();
        cx.notify();
    }

    /// Returns whether the client is currently authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.state.is_authenticated()
    }

    /// Returns the list of loaded channels in the workspace.
    pub fn channels(&self) -> &[sharkord_core::types::Channel] {
        &self.state.channels
    }

    /// Interactive handler to close the User Settings modal.
    pub fn close_user_settings_modal(&mut self, cx: &mut Context<Self>) {
        self.close_user_settings();
        cx.notify();
    }

    /// Renders the bottom user status bar docked under the channel tree / DM sidebar.
    pub fn render_bottom_user_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
        let initials = current_user_name
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase();

        let dot_color = match current_user_status.as_str() {
            "online" => theme::status_online(),
            "idle" => theme::status_idle(),
            "dnd" => theme::status_dnd(),
            _ => theme::status_offline(),
        };

        let mut user_card = div()
            .id("user_status_bar_profile")
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .p_1()
            .rounded_md()
            .hover(|s| s.bg(theme::bg_elevated()))
            // Avatar with status indicator
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
                    .child(if initials.is_empty() { "U".to_string() } else { initials })
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .right_0()
                            .w(px(10.0))
                            .h(px(10.0))
                            .rounded_full()
                            .border_1()
                            .border_color(theme::bg_surface())
                            .bg(dot_color),
                    ),
            )
            // Name and tag
            .child(
                div()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(current_user_name.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!("@{}", current_user_name)),
                    ),
            );

        let user_id = self.state.current_user.as_ref().map(|u| u.id).unwrap_or(0);
        let user_name_clone = current_user_name.clone();
        let user_status_clone = current_user_status.clone();
        user_card = user_card.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |view, _, _window, cx| {
                view.open_user_profile(user_id, &user_name_clone, &user_name_clone, &user_status_clone);
                cx.notify();
            }),
        );

        let mute_btn = div()
            .id("user_bar_mute_btn")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(26.0))
            .h(px(26.0))
            .rounded_md()
            .hover(|s| s.bg(theme::bg_elevated()))
            .child(render_mic_icon(self.voice_bar.is_muted))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.on_mute_click(cx);
                }),
            );

        let deafen_btn = div()
            .id("user_bar_deafen_btn")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(26.0))
            .h(px(26.0))
            .rounded_md()
            .hover(|s| s.bg(theme::bg_elevated()))
            .child(render_headphones_icon(self.voice_bar.is_deafened))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.on_deafen_click(cx);
                }),
            );

        let gear_btn = div()
            .id("user_settings_btn")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(26.0))
            .h(px(26.0))
            .rounded_md()
            .hover(|s| s.bg(theme::bg_elevated()))
            .child(render_gear_icon())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.open_user_settings_modal(cx);
                }),
            );

        div()
            .id("bottom_user_status_bar")
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_2()
            .py_1p5()
            .bg(theme::bg_surface())
            .border_t_1()
            .border_color(theme::border_hairline())
            .child(user_card)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(mute_btn)
                    .child(deafen_btn)
                    .child(gear_btn),
            )
    }
}

impl Default for WorkspaceView {
    fn default() -> Self {
        Self::new(AppState::default())
    }
}

/// Renders a clean vector chat bubble icon for Home navigation and direct messaging.
pub fn render_chat_bubble_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(18.0))
        .h(px(18.0))
        .child(
            div()
                .relative()
                .w(px(14.0))
                .h(px(11.0))
                .rounded_md()
                .border_2()
                .border_color(theme::text_primary())
                .child(
                    div()
                        .absolute()
                        .bottom(px(-3.0))
                        .left(px(2.0))
                        .w(px(4.0))
                        .h(px(4.0))
                        .bg(theme::text_primary())
                        .rounded_xs(),
                ),
        )
}

/// Renders a clean vector microphone icon with optional mute slash indicator.
pub fn render_mic_icon(is_muted: bool) -> impl IntoElement {
    let color = if is_muted {
        theme::status_dnd()
    } else {
        theme::text_secondary()
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(16.0))
        .h(px(16.0))
        .relative()
        .child(
            div()
                .w(px(6.0))
                .h(px(10.0))
                .rounded_full()
                .bg(color),
        )
        .children(if is_muted {
            Some(
                div()
                    .absolute()
                    .w(px(14.0))
                    .h(px(1.5))
                    .bg(theme::status_dnd())
                    .rounded_xs(),
            )
        } else {
            None
        })
}

/// Renders a clean vector headphones icon with optional deafen slash indicator.
pub fn render_headphones_icon(is_deafened: bool) -> impl IntoElement {
    let color = if is_deafened {
        theme::status_dnd()
    } else {
        theme::text_secondary()
    };
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(16.0))
        .h(px(16.0))
        .relative()
        .child(
            div()
                .w(px(12.0))
                .h(px(10.0))
                .rounded_t_full()
                .border_t_2()
                .border_x_2()
                .border_color(color),
        )
        .children(if is_deafened {
            Some(
                div()
                    .absolute()
                    .w(px(14.0))
                    .h(px(1.5))
                    .bg(theme::status_dnd())
                    .rounded_xs(),
            )
        } else {
            None
        })
}

/// Renders a clean vector settings gear icon.
pub fn render_gear_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(16.0))
        .h(px(16.0))
        .child(
            div()
                .w(px(10.0))
                .h(px(10.0))
                .rounded_full()
                .border_2()
                .border_color(theme::text_secondary()),
        )
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
            // Column 2: Channel Tree Sidebar or DM Sidebar (240px fixed) with docked Voice Bar
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(240.0))
                    .h_full()
                    .bg(theme::bg_subsurface())
                    .border_r_1()
                    .border_color(theme::border_hairline())
                    .child(if self.server_rail.is_home_selected {
                        self.dm_sidebar.render_interactive(cx).into_any_element()
                    } else {
                        self.channel_tree.render_interactive(cx).into_any_element()
                    })
                    .child(voice_bar_view.render_interactive(cx))
                    .child(self.render_bottom_user_bar(cx)),
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
            root = root.child(self.member_list.render_interactive(cx));
        }

        if let Some(ref pip) = self.state.active_pip_player {
            if pip.is_visible {
                root = root.child(PipPlayerView::new(pip.clone()).render_element());
            }
        }

        if self.state.is_quick_switcher_open {
            root = root.child(QuickSwitcherView::new(self.quick_switcher.clone()).render_interactive(cx));
        }

        if self.state.is_update_modal_open {
            root = root.child(UpdaterModalView::new(&self.state.update_status).render_element());
        }

        if self.state.is_auth_modal_open {
            root = root.child(AuthModalView::new(self.auth_modal.clone()).render_element());
        }

        if self.user_profile_modal.is_open {
            root = root.child(
                UserProfilePopoverView::new(self.user_profile_modal.clone()).render_interactive(cx),
            );
        }

        if self.create_channel_modal.is_open {
            root = root.child(
                CreateChannelModalView::new(self.create_channel_modal.clone())
                    .render_interactive(cx),
            );
        }

        if self.edit_channel_modal.is_open {
            root = root.child(
                EditChannelModalView::new(self.edit_channel_modal.clone()).render_interactive(cx),
            );
        }

        if self.server_settings_modal.is_open {
            root = root.child(
                ServerSettingsModalView::new(self.server_settings_modal.clone())
                    .render_interactive(cx),
            );
        }

        if self.user_settings_modal.is_open {
            root = root.child(
                UserSettingsModalView::new(self.user_settings_modal.clone())
                    .render_interactive(cx),
            );
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
