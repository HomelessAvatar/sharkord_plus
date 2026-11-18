use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub identity: Option<String>,
    #[serde(default)]
    pub profile_color: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub avatar_id: Option<u64>,
    #[serde(default)]
    pub banner_id: Option<u64>,
    #[serde(default)]
    pub banned: Option<bool>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub role_ids: Vec<u64>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChannelType {
    #[default]
    Text,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub channel_type: ChannelType,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub is_dm: bool,
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub category_id: Option<u64>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: u64,
    pub channel_id: u64,
    #[serde(default)]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub plugin_id: Option<String>,
    #[serde(default)]
    pub parent_message_id: Option<u64>,
    #[serde(default)]
    pub reply_to_message_id: Option<u64>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub pinned_at: Option<i64>,
    #[serde(default)]
    pub pinned_by: Option<u64>,
    #[serde(default)]
    pub editable: Option<bool>,
    #[serde(default)]
    pub edited_at: Option<i64>,
    #[serde(default)]
    pub edited_by: Option<u64>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
    #[serde(default)]
    pub components: Vec<InteractiveComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginCommand {
    #[serde(alias = "plugin_id")]
    pub plugin_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginActionPayload {
    #[serde(alias = "plugin_id")]
    pub plugin_id: String,
    #[serde(alias = "action_name", alias = "actionId", alias = "action_id")]
    pub action_name: String,
    #[serde(default, alias = "channel_id")]
    pub channel_id: Option<u64>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginButton {
    pub id: String,
    pub label: String,
    #[serde(default, alias = "plugin_id")]
    pub plugin_id: Option<String>,
    #[serde(
        default,
        alias = "action_name",
        alias = "actionId",
        alias = "action_id"
    )]
    pub action_name: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub disabled: Option<bool>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginPollOption {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub votes: u32,
    #[serde(default)]
    pub voted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginPoll {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub options: Vec<PluginPollOption>,
    #[serde(default, alias = "plugin_id")]
    pub plugin_id: Option<String>,
    #[serde(
        default,
        alias = "action_name",
        alias = "actionId",
        alias = "action_id"
    )]
    pub action_name: Option<String>,
    #[serde(default, alias = "total_votes")]
    pub total_votes: Option<u32>,
    #[serde(default, alias = "allows_multiple")]
    pub allows_multiple: bool,
    #[serde(default)]
    pub closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InteractiveComponent {
    #[serde(rename = "button")]
    Button(PluginButton),
    #[serde(rename = "poll")]
    Poll(PluginPoll),
    #[serde(rename = "card", rename_all = "camelCase")]
    Card {
        title: String,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        buttons: Vec<PluginButton>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub is_persistent: bool,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub storage_quota_override_enabled: bool,
    #[serde(default)]
    pub storage_space_quota: i64,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub created_at: Option<i64>,
    #[serde(default)]
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettings {
    pub server_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub allow_new_users: Option<bool>,
    #[serde(default)]
    pub direct_messages_enabled: Option<bool>,
    #[serde(default)]
    pub storage_upload_enabled: Option<bool>,
    #[serde(default)]
    pub storage_quota: Option<i64>,
    #[serde(default)]
    pub storage_upload_max_file_size: Option<i64>,
    #[serde(default)]
    pub enable_plugins: Option<bool>,
    #[serde(default)]
    pub enable_search: Option<bool>,
    #[serde(default)]
    pub show_welcome_dialog: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_plus: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high_profile_codecs: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isolated_window_audio: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(default)]
    pub is_plus: bool,
    #[serde(default)]
    pub high_profile_codecs: bool,
    #[serde(default)]
    pub isolated_window_audio: bool,
    #[serde(default)]
    pub server_version: Option<String>,
}

impl Capabilities {
    pub fn standard() -> Self {
        Self {
            is_plus: false,
            high_profile_codecs: false,
            isolated_window_audio: false,
            server_version: None,
        }
    }

    pub fn from_settings(settings: &ServerSettings, is_plus_header: bool) -> Self {
        let is_plus = is_plus_header
            || settings.is_plus.unwrap_or(false)
            || settings
                .server_version
                .as_deref()
                .map(|v| v.to_lowercase().contains("plus"))
                .unwrap_or(false);

        if is_plus {
            Self {
                is_plus: true,
                high_profile_codecs: settings.high_profile_codecs.unwrap_or(true),
                isolated_window_audio: settings.isolated_window_audio.unwrap_or(true),
                server_version: settings.server_version.clone(),
            }
        } else {
            Self {
                is_plus: false,
                high_profile_codecs: false,
                isolated_window_audio: false,
                server_version: settings.server_version.clone(),
            }
        }
    }
}

pub type ServerCapabilities = Capabilities;
