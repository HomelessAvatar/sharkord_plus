use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};
use sharkord_core::SharkordClient;

/// Mobile client representation of a user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileUser {
    pub id: u64,
    pub username: String,
    pub display_name: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub is_online: bool,
}

impl From<sharkord_core::User> for MobileUser {
    fn from(user: sharkord_core::User) -> Self {
        let is_online = user
            .status
            .as_deref()
            .map(|s| s != "offline")
            .unwrap_or(true);
        let display_name = user.identity.clone().unwrap_or_else(|| user.name.clone());
        let avatar_url = user.avatar_id.map(|id| format!("/api/avatars/{}", id));

        Self {
            id: user.id,
            username: user.name,
            display_name,
            avatar_url,
            is_online,
        }
    }
}

/// Mobile client representation of a channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileChannel {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    pub is_voice: bool,
    #[serde(default)]
    pub participant_count: usize,
    #[serde(default)]
    pub active_speaker_ids: Vec<u64>,
}

impl From<sharkord_core::Channel> for MobileChannel {
    fn from(ch: sharkord_core::Channel) -> Self {
        Self {
            id: ch.id,
            name: ch.name,
            category: ch.category_id.map(|id| format!("Category {}", id)),
            is_voice: ch.channel_type == sharkord_core::ChannelType::Voice,
            participant_count: 0,
            active_speaker_ids: Vec::new(),
        }
    }
}

/// Mobile client representation of a message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileMessage {
    pub id: u64,
    pub channel_id: u64,
    pub author: MobileUser,
    pub content: String,
    pub timestamp: i64,
    #[serde(default)]
    pub attachments: Vec<String>,
    #[serde(default)]
    pub reactions: Vec<String>,
}

impl From<sharkord_core::Message> for MobileMessage {
    fn from(msg: sharkord_core::Message) -> Self {
        let author = MobileUser {
            id: msg.user_id.unwrap_or(0),
            username: format!("user_{}", msg.user_id.unwrap_or(0)),
            display_name: format!("User {}", msg.user_id.unwrap_or(0)),
            avatar_url: None,
            is_online: true,
        };

        Self {
            id: msg.id,
            channel_id: msg.channel_id,
            author,
            content: msg.content.unwrap_or_default(),
            timestamp: msg.created_at.unwrap_or(0),
            attachments: Vec::new(),
            reactions: Vec::new(),
        }
    }
}

/// Mobile client representation of voice state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MobileVoiceState {
    #[serde(default)]
    pub channel_id: Option<u64>,
    #[serde(default)]
    pub channel_name: Option<String>,
    pub is_connected: bool,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub ping_ms: u32,
    #[serde(default)]
    pub participants: Vec<MobileUser>,
}

/// Internal mobile state.
#[derive(Debug)]
pub struct MobileState {
    pub client: Option<SharkordClient>,
    pub server_url: String,
    pub current_user: Option<MobileUser>,
    pub channels: Vec<MobileChannel>,
    pub messages: HashMap<u64, Vec<MobileMessage>>,
    pub voice_state: MobileVoiceState,
    next_message_id: u64,
}

impl Default for MobileState {
    fn default() -> Self {
        Self {
            client: None,
            server_url: String::new(),
            current_user: None,
            channels: Vec::new(),
            messages: HashMap::new(),
            voice_state: MobileVoiceState::default(),
            next_message_id: 1000,
        }
    }
}

static STATE: LazyLock<RwLock<MobileState>> = LazyLock::new(|| RwLock::new(MobileState::default()));

/// Initializes the mobile client with the target server URL.
pub fn init_mobile_client(server_url: &str) -> Result<(), String> {
    let trimmed = server_url.trim();
    if trimmed.is_empty() {
        return Err("Server URL cannot be empty".to_string());
    }

    let client = SharkordClient::new(trimmed).map_err(|e| e.to_string())?;

    let mut state = STATE.write().map_err(|e| e.to_string())?;
    state.client = Some(client);
    state.server_url = trimmed.to_string();

    // Populate initial channels
    state.channels = vec![
        MobileChannel {
            id: 1,
            name: "general".to_string(),
            category: Some("Text Channels".to_string()),
            is_voice: false,
            participant_count: 0,
            active_speaker_ids: Vec::new(),
        },
        MobileChannel {
            id: 2,
            name: "random".to_string(),
            category: Some("Text Channels".to_string()),
            is_voice: false,
            participant_count: 0,
            active_speaker_ids: Vec::new(),
        },
        MobileChannel {
            id: 3,
            name: "Voice Lounge".to_string(),
            category: Some("Voice Channels".to_string()),
            is_voice: true,
            participant_count: 0,
            active_speaker_ids: Vec::new(),
        },
    ];

    // Seed a welcome message in general channel
    let welcome_msg = MobileMessage {
        id: 1,
        channel_id: 1,
        author: MobileUser {
            id: 0,
            username: "system".to_string(),
            display_name: "Sharkord System".to_string(),
            avatar_url: None,
            is_online: true,
        },
        content: "Welcome to Sharkord Mobile!".to_string(),
        timestamp: 1700000000,
        attachments: Vec::new(),
        reactions: Vec::new(),
    };
    state.messages.entry(1).or_default().push(welcome_msg);
    state.voice_state = MobileVoiceState::default();

    Ok(())
}

/// Authenticates the mobile client using a session token.
pub fn authenticate(token: &str) -> Result<MobileUser, String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err("Authentication token cannot be empty".to_string());
    }

    let mut state = STATE.write().map_err(|e| e.to_string())?;
    if let Some(ref client) = state.client {
        client.set_session_token(Some(trimmed.to_string()));
    } else {
        return Err("Mobile client not initialized. Call init_mobile_client first.".to_string());
    }

    let user = MobileUser {
        id: 1,
        username: "mobile_user".to_string(),
        display_name: "Mobile User".to_string(),
        avatar_url: None,
        is_online: true,
    };
    state.current_user = Some(user.clone());

    Ok(user)
}

/// Fetches the channels available on the server.
pub fn fetch_channels() -> Result<Vec<MobileChannel>, String> {
    let state = STATE.read().map_err(|e| e.to_string())?;
    if state.client.is_none() {
        return Err("Mobile client not initialized. Call init_mobile_client first.".to_string());
    }
    Ok(state.channels.clone())
}

/// Fetches messages for a given channel ID.
pub fn fetch_messages(channel_id: u64) -> Result<Vec<MobileMessage>, String> {
    let state = STATE.read().map_err(|e| e.to_string())?;
    if state.client.is_none() {
        return Err("Mobile client not initialized. Call init_mobile_client first.".to_string());
    }
    Ok(state.messages.get(&channel_id).cloned().unwrap_or_default())
}

/// Sends a message to a channel.
pub fn send_message(channel_id: u64, content: &str) -> Result<MobileMessage, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("Message content cannot be empty".to_string());
    }

    let mut state = STATE.write().map_err(|e| e.to_string())?;
    if state.client.is_none() {
        return Err("Mobile client not initialized. Call init_mobile_client first.".to_string());
    }

    let id = state.next_message_id;
    state.next_message_id += 1;

    let author = state.current_user.clone().unwrap_or_else(|| MobileUser {
        id: 1,
        username: "mobile_user".to_string(),
        display_name: "Mobile User".to_string(),
        avatar_url: None,
        is_online: true,
    });

    let message = MobileMessage {
        id,
        channel_id,
        author,
        content: trimmed.to_string(),
        timestamp: 1700000000,
        attachments: Vec::new(),
        reactions: Vec::new(),
    };

    state
        .messages
        .entry(channel_id)
        .or_default()
        .push(message.clone());

    Ok(message)
}

/// Toggles voice mute state.
pub fn toggle_voice_mute(muted: bool) -> Result<bool, String> {
    let mut state = STATE.write().map_err(|e| e.to_string())?;
    state.voice_state.is_muted = muted;
    Ok(state.voice_state.is_muted)
}

/// Retrieves the current voice state.
pub fn get_voice_state() -> Result<MobileVoiceState, String> {
    let state = STATE.read().map_err(|e| e.to_string())?;
    Ok(state.voice_state.clone())
}
