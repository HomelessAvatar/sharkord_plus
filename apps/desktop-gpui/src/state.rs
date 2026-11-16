//! Application state management for the Sharkord GPUI desktop client.

use serde::{Deserialize, Serialize};
use sharkord_core::types::{Channel, Message, ServerSettings, User};

/// Top-level application state for Sharkord desktop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub active_server_id: Option<String>,
    pub active_channel_id: Option<u64>,
    pub channels: Vec<Channel>,
    pub messages: Vec<Message>,
    pub current_user: Option<User>,
    pub server_settings: Option<ServerSettings>,
    pub left_sidebar_open: bool,
    pub right_sidebar_open: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_server_id: None,
            active_channel_id: None,
            channels: Vec::new(),
            messages: Vec::new(),
            current_user: None,
            server_settings: None,
            left_sidebar_open: true,
            right_sidebar_open: true,
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
}
