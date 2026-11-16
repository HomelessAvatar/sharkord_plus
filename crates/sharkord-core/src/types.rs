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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChannelType {
    Text,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

pub type ServerCapabilities = Capabilities;
