//! Application state management for the Sharkord GPUI desktop client.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sharkord_core::protocol::events::{EventType, SubscriptionEvent};
use sharkord_core::types::{Channel, Message, MessageReaction, ServerSettings, User};
use sharkord_core::updater::UpdateStatus;
use sharkord_core::vault::SavedServerSession;

use crate::ui::pip_player::PipPlayerState;
use crate::ui::voice_bar::{VoiceBarState, VoiceConnectionState};
use crate::ui::ActiveContentView;

/// Top-level application state for Sharkord desktop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub active_server_id: Option<String>,
    pub active_channel_id: Option<u64>,
    pub channels: Vec<Channel>,
    pub messages: Vec<Message>,
    pub current_user: Option<User>,
    pub users: Vec<User>,
    pub server_settings: Option<ServerSettings>,
    pub left_sidebar_open: bool,
    pub right_sidebar_open: bool,
    pub voice_bar_state: VoiceBarState,
    pub user_volumes: HashMap<u64, f32>,
    pub local_muted_users: HashSet<u64>,
    pub active_pip_player: Option<PipPlayerState>,
    pub active_center_view: ActiveContentView,
    pub is_quick_switcher_open: bool,
    pub update_status: UpdateStatus,
    pub is_update_modal_open: bool,
    pub is_connecting: bool,
    pub is_auth_modal_open: bool,
    pub typing_users: HashMap<u64, HashSet<u64>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_server_id: None,
            active_channel_id: None,
            channels: Vec::new(),
            messages: Vec::new(),
            current_user: None,
            users: Vec::new(),
            server_settings: None,
            left_sidebar_open: true,
            right_sidebar_open: true,
            voice_bar_state: VoiceBarState::default(),
            user_volumes: HashMap::new(),
            local_muted_users: HashSet::new(),
            active_pip_player: None,
            active_center_view: ActiveContentView::default(),
            is_quick_switcher_open: false,
            update_status: UpdateStatus::Idle,
            is_update_modal_open: false,
            is_connecting: false,
            is_auth_modal_open: false,
            typing_users: HashMap::new(),
        }
    }
}

impl AppState {
    /// Creates a new `AppState` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a mock initialized `AppState` instance with preconfigured channels.
    pub fn new_mock() -> Self {
        let mut state = Self::new();
        state.server_settings = Some(ServerSettings {
            server_id: "sharkord-main".to_string(),
            name: "Sharkord Plus HQ".to_string(),
            is_plus: Some(true),
            ..Default::default()
        });
        state.current_user = Some(User {
            id: 1,
            name: "User".to_string(),
            status: Some("online".to_string()),
            ..Default::default()
        });
        state.set_channels(vec![
            Channel {
                id: 1,
                name: "general".to_string(),
                channel_type: sharkord_core::types::ChannelType::Text,
                topic: Some("General discussion for Sharkord Plus".to_string()),
                private: false,
                is_dm: false,
                position: 0,
                category_id: None,
                created_at: Some(1000),
                updated_at: None,
            },
            Channel {
                id: 2,
                name: "voice-lounge".to_string(),
                channel_type: sharkord_core::types::ChannelType::Voice,
                topic: None,
                private: false,
                is_dm: false,
                position: 1,
                category_id: None,
                created_at: Some(1000),
                updated_at: None,
            },
        ]);
        state.select_channel(1);
        state
    }

    /// Sets the channels list.
    pub fn set_channels(&mut self, channels: Vec<Channel>) {
        self.channels = channels;
    }

    /// Selects an active channel by its ID.
    pub fn select_channel(&mut self, channel_id: u64) {
        self.active_channel_id = Some(channel_id);
    }

    /// Resolves the currently active channel from the channels list.
    pub fn active_channel(&self) -> Option<&Channel> {
        let channel_id = self.active_channel_id?;
        self.channels.iter().find(|c| c.id == channel_id)
    }

    /// Appends a message to the message store.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Retrieves references to all messages belonging to the currently active channel.
    pub fn messages_for_active_channel(&self) -> Vec<&Message> {
        match self.active_channel_id {
            Some(channel_id) => self
                .messages
                .iter()
                .filter(|m| m.channel_id == channel_id)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Toggles the left sidebar visibility.
    pub fn toggle_left_sidebar(&mut self) {
        self.left_sidebar_open = !self.left_sidebar_open;
    }

    /// Toggles the right sidebar visibility.
    pub fn toggle_right_sidebar(&mut self) {
        self.right_sidebar_open = !self.right_sidebar_open;
    }

    /// Toggles the quick switcher modal open state.
    pub fn toggle_quick_switcher(&mut self) {
        self.is_quick_switcher_open = !self.is_quick_switcher_open;
    }

    /// Adds a user or updates if already present.
    pub fn add_user(&mut self, user: User) {
        if let Some(existing) = self.users.iter_mut().find(|u| u.id == user.id) {
            *existing = user;
        } else {
            self.users.push(user);
        }
    }

    /// Updates an existing user in the users store and syncs current_user if matching.
    pub fn update_user(&mut self, user: User) {
        if let Some(existing) = self.users.iter_mut().find(|u| u.id == user.id) {
            *existing = user.clone();
        } else {
            self.users.push(user.clone());
        }

        if let Some(curr) = &mut self.current_user {
            if curr.id == user.id {
                *curr = user;
            }
        }
    }

    /// Removes a user from the users store.
    pub fn remove_user(&mut self, user_id: u64) {
        self.users.retain(|u| u.id != user_id);
    }

    /// Finds a user by ID.
    pub fn user_by_id(&self, user_id: u64) -> Option<&User> {
        self.users.iter().find(|u| u.id == user_id)
    }

    /// Connects to a voice channel and updates voice bar state.
    pub fn connect_voice(&mut self, channel_id: u64, channel_name: String) {
        self.voice_bar_state.connection = VoiceConnectionState::Connected {
            channel_id,
            channel_name,
            ping_ms: 0,
        };
    }

    /// Updates voice ping in milliseconds.
    pub fn update_voice_ping(&mut self, ping_ms: u32) {
        if let VoiceConnectionState::Connected {
            ping_ms: ref mut p, ..
        } = self.voice_bar_state.connection
        {
            *p = ping_ms;
        }
    }

    /// Toggles microphone mute. Returns the new muted state.
    pub fn toggle_voice_mute(&mut self) -> bool {
        self.voice_bar_state.toggle_mute();
        self.voice_bar_state.is_muted
    }

    /// Toggles microphone mute (alias for `toggle_voice_mute`).
    pub fn toggle_mute(&mut self) -> bool {
        self.toggle_voice_mute()
    }

    /// Toggles sound deafen. When deafened, microphone is also muted.
    /// Returns the new deafened state.
    pub fn toggle_voice_deafen(&mut self) -> bool {
        self.voice_bar_state.toggle_deafen();
        if self.voice_bar_state.is_deafened {
            self.voice_bar_state.is_muted = true;
        }
        self.voice_bar_state.is_deafened
    }

    /// Toggles sound deafen (alias for `toggle_voice_deafen`).
    pub fn toggle_deafen(&mut self) -> bool {
        self.toggle_voice_deafen()
    }

    /// Disconnects from active voice channel.
    pub fn disconnect_voice(&mut self) {
        self.voice_bar_state.disconnect();
    }

    /// Sets the volume multiplier for a specific user (clamped between 0.0 and 2.0).
    pub fn set_user_volume(&mut self, user_id: u64, volume: f32) {
        self.user_volumes.insert(user_id, volume.clamp(0.0, 2.0));
    }

    /// Gets the volume multiplier for a specific user (defaults to 1.0).
    pub fn get_user_volume(&self, user_id: u64) -> f32 {
        self.user_volumes.get(&user_id).copied().unwrap_or(1.0)
    }

    /// Toggles local mute for a specific user. Returns the new muted state.
    pub fn toggle_local_mute(&mut self, user_id: u64) -> bool {
        if self.local_muted_users.contains(&user_id) {
            self.local_muted_users.remove(&user_id);
            false
        } else {
            self.local_muted_users.insert(user_id);
            true
        }
    }

    /// Returns whether a specific user is locally muted.
    pub fn is_local_muted(&self, user_id: u64) -> bool {
        self.local_muted_users.contains(&user_id)
    }

    /// Sets or updates the active Picture-in-Picture player state.
    pub fn set_pip_player(&mut self, player: PipPlayerState) {
        self.active_pip_player = Some(player);
    }

    /// Closes the active Picture-in-Picture player.
    pub fn close_pip_player(&mut self) {
        self.active_pip_player = None;
    }

    /// Sets the active center content view.
    pub fn set_center_view(&mut self, view: ActiveContentView) {
        self.active_center_view = view;
    }

    /// Toggles the in-app update modal dialog open state.
    pub fn toggle_update_modal(&mut self) {
        self.is_update_modal_open = !self.is_update_modal_open;
    }

    /// Sets the current application update status.
    pub fn set_update_status(&mut self, status: UpdateStatus) {
        self.update_status = status;
    }

    /// Applies a saved server session to the application state.
    pub fn apply_saved_session(&mut self, session: &SavedServerSession) {
        self.active_server_id = Some(session.server_id.clone());
        self.current_user = Some(User {
            id: session.user_id,
            name: session.username.clone(),
            identity: Some(session.display_name.clone()),
            status: Some("online".to_string()),
            ..Default::default()
        });
        if let Some(ref mut settings) = self.server_settings {
            settings.server_id = session.server_id.clone();
            settings.name = session.server_name.clone();
        } else {
            self.server_settings = Some(ServerSettings {
                server_id: session.server_id.clone(),
                name: session.server_name.clone(),
                ..Default::default()
            });
        }
    }

    /// Sets connection progress state.
    pub fn set_connecting(&mut self, connecting: bool) {
        self.is_connecting = connecting;
    }

    /// Sets authentication modal visibility.
    pub fn set_auth_modal_open(&mut self, open: bool) {
        self.is_auth_modal_open = open;
    }

    /// Toggles the authentication modal open state.
    pub fn toggle_auth_modal(&mut self) {
        self.is_auth_modal_open = !self.is_auth_modal_open;
    }

    /// Bootstraps server settings, channels, and messages from the server.
    pub fn bootstrap_from_server(
        &mut self,
        settings: impl Into<Option<ServerSettings>>,
        channels: Vec<Channel>,
        messages: Vec<Message>,
    ) {
        if let Some(s) = settings.into() {
            self.server_settings = Some(s);
        }
        self.channels = channels;
        self.messages = messages;
        if self.active_channel_id.is_none()
            || !self.channels.iter().any(|c| Some(c.id) == self.active_channel_id)
        {
            self.active_channel_id = self.channels.first().map(|c| c.id);
        }
        self.is_connecting = false;
        self.is_auth_modal_open = false;
    }

    /// Bootstraps server state using a `BootstrapData` payload.
    pub fn bootstrap_from_data(&mut self, data: crate::network::BootstrapData) {
        let active_id = data.active_channel_id;
        self.bootstrap_from_server(Some(data.settings), data.channels, data.initial_messages);
        if let Some(id) = active_id {
            self.active_channel_id = Some(id);
        }
    }

    /// Returns whether a specific user is currently typing in the specified channel.
    pub fn is_user_typing(&self, channel_id: u64, user_id: u64) -> bool {
        self.typing_users
            .get(&channel_id)
            .map(|set| set.contains(&user_id))
            .unwrap_or(false)
    }

    /// Retrieves all user IDs currently typing in the specified channel.
    pub fn typing_users_for_channel(&self, channel_id: u64) -> Vec<u64> {
        self.typing_users
            .get(&channel_id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Sets whether a specific user is typing in a given channel.
    pub fn set_user_typing(&mut self, channel_id: u64, user_id: u64, is_typing: bool) {
        if is_typing {
            self.typing_users.entry(channel_id).or_default().insert(user_id);
        } else if let Some(set) = self.typing_users.get_mut(&channel_id) {
            set.remove(&user_id);
        }
    }

    /// Clears all typing users in the specified channel.
    pub fn clear_typing_users(&mut self, channel_id: u64) {
        self.typing_users.remove(&channel_id);
    }

    /// Handles incoming real-time WebSocket subscription events and mutates the application state.
    pub fn handle_subscription_event(&mut self, event: SubscriptionEvent) {
        match event.event_type {
            EventType::MessageCreated => {
                let message: Option<Message> = serde_json::from_value(event.data.clone())
                    .ok()
                    .or_else(|| {
                        event
                            .data
                            .get("message")
                            .and_then(|m| serde_json::from_value(m.clone()).ok())
                    });

                if let Some(msg) = message {
                    // When a user sends a message, clear typing state for this user in the channel
                    if let Some(user_id) = msg.user_id {
                        if let Some(set) = self.typing_users.get_mut(&msg.channel_id) {
                            set.remove(&user_id);
                        }
                    }

                    // Append to active channel messages if it matches active channel
                    if self.active_channel_id == Some(msg.channel_id) {
                        if let Some(existing) = self.messages.iter_mut().find(|m| m.id == msg.id) {
                            *existing = msg;
                        } else {
                            self.messages.push(msg);
                        }
                    }
                }
            }

            EventType::MessageUpdated => {
                let message_opt: Option<Message> = serde_json::from_value(event.data.clone()).ok();
                let msg_id = event
                    .data
                    .get("id")
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                    .or_else(|| message_opt.as_ref().map(|m| m.id));

                if let Some(id) = msg_id {
                    if let Some(existing) = self.messages.iter_mut().find(|m| m.id == id) {
                        if let Some(full_msg) = message_opt {
                            *existing = full_msg;
                        } else {
                            if let Some(content) =
                                event.data.get("content").and_then(|v| v.as_str())
                            {
                                existing.content = Some(content.to_string());
                            }
                            if let Some(pinned) = event.data.get("pinned").and_then(|v| v.as_bool())
                            {
                                existing.pinned = pinned;
                            }
                            if let Some(pinned_at) = event
                                .data
                                .get("pinnedAt")
                                .or_else(|| event.data.get("pinned_at"))
                                .and_then(|v| v.as_i64())
                            {
                                existing.pinned_at = Some(pinned_at);
                            }
                            if let Some(edited_at) = event
                                .data
                                .get("editedAt")
                                .or_else(|| event.data.get("edited_at"))
                                .and_then(|v| v.as_i64())
                            {
                                existing.edited_at = Some(edited_at);
                            }
                            if let Some(comps) = event
                                .data
                                .get("components")
                                .and_then(|v| serde_json::from_value(v.clone()).ok())
                            {
                                existing.components = comps;
                            }
                            if let Some(reactions) = event
                                .data
                                .get("reactions")
                                .and_then(|v| serde_json::from_value(v.clone()).ok())
                            {
                                existing.reactions = reactions;
                            }
                        }
                    }
                }
            }

            EventType::MessageDeleted => {
                let msg_id = event
                    .data
                    .get("id")
                    .or_else(|| event.data.get("messageId"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));

                if let Some(id) = msg_id {
                    self.messages.retain(|m| m.id != id);
                }
            }

            EventType::ReactionAdded => {
                let msg_id = event
                    .data
                    .get("messageId")
                    .or_else(|| event.data.get("message_id"))
                    .or_else(|| event.data.get("id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
                let emoji = event.data.get("emoji").and_then(|v| v.as_str());
                let user_id = event
                    .data
                    .get("userId")
                    .or_else(|| event.data.get("user_id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));

                if let (Some(msg_id), Some(emoji), Some(user_id)) = (msg_id, emoji, user_id) {
                    if let Some(existing) = self.messages.iter_mut().find(|m| m.id == msg_id) {
                        if !existing
                            .reactions
                            .iter()
                            .any(|r| r.user_id == user_id && r.emoji == emoji)
                        {
                            existing.reactions.push(MessageReaction {
                                message_id: msg_id,
                                user_id,
                                emoji: emoji.to_string(),
                                plugin_id: event
                                    .data
                                    .get("pluginId")
                                    .or_else(|| event.data.get("plugin_id"))
                                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                                file_id: event
                                    .data
                                    .get("fileId")
                                    .or_else(|| event.data.get("file_id"))
                                    .and_then(|v| v.as_u64()),
                                created_at: event
                                    .data
                                    .get("createdAt")
                                    .or_else(|| event.data.get("created_at"))
                                    .and_then(|v| v.as_i64()),
                            });
                        }
                    }
                }
            }

            EventType::ReactionRemoved => {
                let msg_id = event
                    .data
                    .get("messageId")
                    .or_else(|| event.data.get("message_id"))
                    .or_else(|| event.data.get("id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
                let emoji = event.data.get("emoji").and_then(|v| v.as_str());
                let user_id = event
                    .data
                    .get("userId")
                    .or_else(|| event.data.get("user_id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));

                if let (Some(msg_id), Some(emoji)) = (msg_id, emoji) {
                    if let Some(existing) = self.messages.iter_mut().find(|m| m.id == msg_id) {
                        if let Some(uid) = user_id {
                            existing
                                .reactions
                                .retain(|r| !(r.user_id == uid && r.emoji == emoji));
                        } else {
                            existing.reactions.retain(|r| r.emoji != emoji);
                        }
                    }
                }
            }

            EventType::ChannelCreated => {
                let channel: Option<Channel> = serde_json::from_value(event.data.clone())
                    .ok()
                    .or_else(|| {
                        event
                            .data
                            .get("channel")
                            .and_then(|c| serde_json::from_value(c.clone()).ok())
                    });

                if let Some(ch) = channel {
                    if let Some(existing) = self.channels.iter_mut().find(|c| c.id == ch.id) {
                        *existing = ch;
                    } else {
                        self.channels.push(ch);
                    }
                }
            }

            EventType::ChannelUpdated => {
                let channel_opt: Option<Channel> = serde_json::from_value(event.data.clone()).ok();
                let ch_id = event
                    .data
                    .get("id")
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                    .or_else(|| channel_opt.as_ref().map(|c| c.id));

                if let Some(id) = ch_id {
                    if let Some(existing) = self.channels.iter_mut().find(|c| c.id == id) {
                        if let Some(ch) = channel_opt {
                            *existing = ch;
                        } else {
                            if let Some(name) = event.data.get("name").and_then(|v| v.as_str()) {
                                existing.name = name.to_string();
                            }
                            if let Some(topic) = event.data.get("topic").and_then(|v| v.as_str()) {
                                existing.topic = Some(topic.to_string());
                            }
                            if let Some(pos) =
                                event.data.get("position").and_then(|v| v.as_i64())
                            {
                                existing.position = pos as i32;
                            }
                        }
                    }
                }
            }

            EventType::ChannelDeleted => {
                let ch_id = event
                    .data
                    .get("id")
                    .or_else(|| event.data.get("channelId"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));

                if let Some(id) = ch_id {
                    self.channels.retain(|c| c.id != id);
                    self.messages.retain(|m| m.channel_id != id);
                    self.typing_users.remove(&id);
                    if self.active_channel_id == Some(id) {
                        self.active_channel_id = self.channels.first().map(|c| c.id);
                    }
                }
            }

            EventType::UserPresenceChanged => {
                let user_id = event
                    .data
                    .get("userId")
                    .or_else(|| event.data.get("id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
                let status = event.data.get("status").and_then(|v| v.as_str());

                if let (Some(uid), Some(st)) = (user_id, status) {
                    if let Some(u) = self.users.iter_mut().find(|u| u.id == uid) {
                        u.status = Some(st.to_string());
                    }
                    if let Some(curr) = &mut self.current_user {
                        if curr.id == uid {
                            curr.status = Some(st.to_string());
                        }
                    }
                }
            }

            EventType::UserTyping => {
                let channel_id = event
                    .data
                    .get("channelId")
                    .or_else(|| event.data.get("channel_id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                    .or(self.active_channel_id);
                let user_id = event
                    .data
                    .get("userId")
                    .or_else(|| event.data.get("user_id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
                let is_typing = event
                    .data
                    .get("isTyping")
                    .or_else(|| event.data.get("typing"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                if let (Some(cid), Some(uid)) = (channel_id, user_id) {
                    self.set_user_typing(cid, uid, is_typing);
                }
            }

            EventType::UserJoined | EventType::UserCreated | EventType::UserUpdated => {
                if let Ok(user) = serde_json::from_value::<User>(event.data.clone()) {
                    self.update_user(user);
                }
            }

            EventType::UserLeft | EventType::UserDeleted => {
                let user_id = event
                    .data
                    .get("userId")
                    .or_else(|| event.data.get("id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));

                if let Some(uid) = user_id {
                    self.remove_user(uid);
                }
            }

            EventType::VoiceStateUpdated => {
                if let Some(user_id) = event
                    .data
                    .get("userId")
                    .or_else(|| event.data.get("user_id"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                {
                    if let Some(muted) = event.data.get("muted").and_then(|v| v.as_bool()) {
                        if let Some(ref curr) = self.current_user {
                            if curr.id == user_id {
                                self.voice_bar_state.is_muted = muted;
                            }
                        }
                    }
                }
            }

            EventType::Unknown(_) => {}
        }
    }
}
