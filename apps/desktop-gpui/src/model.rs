//! Reactive application state model for the Sharkord GPUI desktop client.
//! Provides dynamic state management for channels, messages, voice controls,
//! and composer input.

use serde::{Deserialize, Serialize};

/// Represents a channel in the reactive model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelModel {
    pub id: String,
    pub name: String,
    pub is_voice: bool,
    pub topic: Option<String>,
}

/// Represents a chat message in the reactive model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageModel {
    pub id: String,
    pub channel_id: String,
    pub sender_name: String,
    pub content: String,
    pub timestamp: i64,
}

/// Dynamic reactive application state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    pub active_server_id: String,
    pub active_channel_id: String,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_connected: bool,
    pub input_text: String,
    pub channels: Vec<ChannelModel>,
    pub messages: Vec<MessageModel>,
    pub current_user_name: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_server_id: "server_sharkord".to_string(),
            active_channel_id: "chan_general".to_string(),
            is_muted: false,
            is_deafened: false,
            is_connected: true,
            input_text: String::new(),
            channels: Vec::new(),
            messages: Vec::new(),
            current_user_name: "User".to_string(),
        }
    }
}

impl AppState {
    /// Creates a new `AppState` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a mock initialized `AppState` instance with preconfigured channels and messages.
    pub fn new_mock() -> Self {
        Self {
            active_server_id: "server_sharkord".to_string(),
            active_channel_id: "chan_general".to_string(),
            is_muted: false,
            is_deafened: false,
            is_connected: true,
            input_text: String::new(),
            channels: vec![
                ChannelModel {
                    id: "chan_general".to_string(),
                    name: "general".to_string(),
                    is_voice: false,
                    topic: Some("General discussion for Sharkord Plus".to_string()),
                },
                ChannelModel {
                    id: "chan_voice_lounge".to_string(),
                    name: "voice-lounge".to_string(),
                    is_voice: true,
                    topic: None,
                },
            ],
            messages: vec![MessageModel {
                id: "msg_1".to_string(),
                channel_id: "chan_general".to_string(),
                sender_name: "SharkordAdmin".to_string(),
                content: "Welcome to Sharkord Plus!".to_string(),
                timestamp: 1000,
            }],
            current_user_name: "User".to_string(),
        }
    }

    /// Selects an active channel by its string ID.
    pub fn select_channel(&mut self, channel_id: impl Into<String>) {
        self.active_channel_id = channel_id.into();
    }

    /// Toggles voice microphone mute state.
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    /// Toggles voice sound deafen state. When deafened, microphone is also muted.
    pub fn toggle_deafen(&mut self) {
        self.is_deafened = !self.is_deafened;
        if self.is_deafened {
            self.is_muted = true;
        }
    }

    /// Disconnects from the current voice channel.
    pub fn disconnect(&mut self) {
        self.is_connected = false;
    }

    /// Sets the text content in the message composer draft.
    pub fn set_input_text(&mut self, text: impl Into<String>) {
        self.input_text = text.into();
    }

    /// Submits a message to the currently active channel.
    /// Returns the created message if content is non-empty.
    pub fn send_message(&mut self, content: impl Into<String>) -> Option<MessageModel> {
        let content_str = content.into();
        let trimmed = content_str.trim();
        if trimmed.is_empty() {
            return None;
        }

        let next_id = format!("msg_{}", self.messages.len() + 1);
        let msg = MessageModel {
            id: next_id,
            channel_id: self.active_channel_id.clone(),
            sender_name: self.current_user_name.clone(),
            content: trimmed.to_string(),
            timestamp: 1000 + (self.messages.len() as i64 * 60),
        };

        self.messages.push(msg.clone());
        self.input_text.clear();
        Some(msg)
    }

    /// Submits the current input draft as a message.
    pub fn submit_draft(&mut self) -> Option<MessageModel> {
        let text = self.input_text.clone();
        self.send_message(text)
    }

    /// Resolves the currently active channel.
    pub fn active_channel(&self) -> Option<&ChannelModel> {
        self.channels
            .iter()
            .find(|c| c.id == self.active_channel_id)
    }

    /// Retrieves all messages belonging to the currently active channel.
    pub fn messages_for_active_channel(&self) -> Vec<&MessageModel> {
        self.messages
            .iter()
            .filter(|m| m.channel_id == self.active_channel_id)
            .collect()
    }
}
