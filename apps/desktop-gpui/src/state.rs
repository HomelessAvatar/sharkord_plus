//! Application state management for the Sharkord GPUI desktop client.

use serde::{Deserialize, Serialize};
use sharkord_core::types::{Channel, Message, ServerSettings, User};

use crate::ui::voice_bar::{VoiceBarState, VoiceConnectionState};

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
        }
    }
}

impl AppState {
    /// Creates a new `AppState` instance with default settings.
    pub fn new() -> Self {
        Self::default()
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

    /// Toggles sound deafen. When deafened, microphone is also muted.
    /// Returns the new deafened state.
    pub fn toggle_voice_deafen(&mut self) -> bool {
        self.voice_bar_state.toggle_deafen();
        if self.voice_bar_state.is_deafened {
            self.voice_bar_state.is_muted = true;
        }
        self.voice_bar_state.is_deafened
    }

    /// Disconnects from active voice channel.
    pub fn disconnect_voice(&mut self) {
        self.voice_bar_state.disconnect();
    }
}
