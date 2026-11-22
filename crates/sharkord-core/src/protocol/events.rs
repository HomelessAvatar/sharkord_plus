use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::SharkordError;

/// Real-time event types supported across Sharkord WebSocket subscriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    MessageCreated,
    MessageUpdated,
    MessageDeleted,
    UserJoined,
    UserLeft,
    UserUpdated,
    UserCreated,
    UserDeleted,
    VoiceStateUpdated,
    ReactionAdded,
    ReactionRemoved,
    ChannelCreated,
    ChannelUpdated,
    ChannelDeleted,
    UserPresenceChanged,
    UserTyping,
    Unknown(String),
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::MessageCreated => "messageCreated",
            EventType::MessageUpdated => "messageUpdated",
            EventType::MessageDeleted => "messageDeleted",
            EventType::UserJoined => "userJoined",
            EventType::UserLeft => "userLeft",
            EventType::UserUpdated => "userUpdated",
            EventType::UserCreated => "userCreated",
            EventType::UserDeleted => "userDeleted",
            EventType::VoiceStateUpdated => "voiceStateUpdated",
            EventType::ReactionAdded => "reactionAdded",
            EventType::ReactionRemoved => "reactionRemoved",
            EventType::ChannelCreated => "channelCreated",
            EventType::ChannelUpdated => "channelUpdated",
            EventType::ChannelDeleted => "channelDeleted",
            EventType::UserPresenceChanged => "userPresenceChanged",
            EventType::UserTyping => "userTyping",
            EventType::Unknown(s) => s.as_str(),
        }
    }
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        match s {
            "messageCreated" | "newMessage" => EventType::MessageCreated,
            "messageUpdated" | "messageUpdate" => EventType::MessageUpdated,
            "messageDeleted" | "messageDelete" => EventType::MessageDeleted,
            "userJoined" | "userJoin" => EventType::UserJoined,
            "userLeft" | "userLeave" => EventType::UserLeft,
            "userUpdated" | "userUpdate" => EventType::UserUpdated,
            "userCreated" | "userCreate" => EventType::UserCreated,
            "userDeleted" | "userDelete" => EventType::UserDeleted,
            "voiceStateUpdated" | "userVoiceStateUpdate" | "voiceChannelStateUpdate" => {
                EventType::VoiceStateUpdated
            }
            "reactionAdded" | "reactionAdd" | "reaction:added" => EventType::ReactionAdded,
            "reactionRemoved" | "reactionRemove" | "reaction:removed" => {
                EventType::ReactionRemoved
            }
            "channelCreated" | "channelCreate" => EventType::ChannelCreated,
            "channelUpdated" | "channelUpdate" => EventType::ChannelUpdated,
            "channelDeleted" | "channelDelete" => EventType::ChannelDeleted,
            "userPresenceChanged" | "presenceUpdate" | "userPresenceUpdate" => {
                EventType::UserPresenceChanged
            }
            "userTyping" | "messageTyping" => EventType::UserTyping,
            other => EventType::Unknown(other.to_string()),
        }
    }
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(EventType::from(s.as_str()))
    }
}

/// Payload envelope representing typed event data inside WebSocket frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventPayload {
    #[serde(rename = "type", alias = "eventType")]
    pub event_type: EventType,
    #[serde(default)]
    pub data: serde_json::Value,
}

impl EventPayload {
    pub fn new(event_type: EventType, data: serde_json::Value) -> Self {
        Self { event_type, data }
    }
}

/// Decoded real-time subscription event dispatched to listener streams.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionEvent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", alias = "eventType")]
    pub event_type: EventType,
    #[serde(default)]
    pub data: serde_json::Value,
}

impl SubscriptionEvent {
    pub fn new(id: Option<String>, event_type: EventType, data: serde_json::Value) -> Self {
        Self {
            id,
            event_type,
            data,
        }
    }

    pub fn payload(&self) -> &serde_json::Value {
        &self.data
    }
}

/// Encodes a tRPC WebSocket subscription start request.
///
/// Shape:
/// ```json
/// {
///   "id": "1",
///   "method": "subscription",
///   "params": {
///     "path": "messages.events",
///     "input": ...
///   }
/// }
/// ```
pub fn encode_subscription_request(
    id: &str,
    path: &str,
    input: Option<&serde_json::Value>,
) -> String {
    let mut params = serde_json::Map::new();
    params.insert(
        "path".to_string(),
        serde_json::Value::String(path.to_string()),
    );
    if let Some(inp) = input {
        params.insert("input".to_string(), inp.clone());
    }

    let envelope = serde_json::json!({
        "id": id,
        "method": "subscription",
        "params": params,
    });

    envelope.to_string()
}

/// Decodes an incoming tRPC WebSocket message frame.
///
/// Returns:
/// - `Ok(Some(SubscriptionEvent))` when a valid data frame with event payload is received.
/// - `Ok(None)` for non-data lifecycle frames (e.g. `started`, `stopped`).
/// - `Err(SharkordError)` for server error envelopes or unparseable frames.
pub fn decode_websocket_frame(text: &str) -> Result<Option<SubscriptionEvent>, SharkordError> {
    let root: serde_json::Value = serde_json::from_str(text)?;

    // Check for error payload
    if let Some(error_val) = root.get("error") {
        if let Some(err_str) = error_val.as_str() {
            return Err(SharkordError::RpcError {
                code: -1,
                message: err_str.to_string(),
            });
        }

        let target = error_val.get("json").unwrap_or(error_val);
        let code = target
            .get("code")
            .and_then(|c| {
                c.as_i64()
                    .map(|n| n as i32)
                    .or_else(|| c.as_str().and_then(|s| s.parse::<i32>().ok()))
            })
            .unwrap_or(-1);

        let message = target
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown tRPC WebSocket error")
            .to_string();

        return Err(SharkordError::RpcError { code, message });
    }

    // Extract ID (may be string or numeric)
    let id = root.get("id").and_then(|v| {
        v.as_str()
            .map(|s| s.to_string())
            .or_else(|| v.as_i64().map(|n| n.to_string()))
    });

    // Check result field
    if let Some(result_val) = root.get("result") {
        let result_type = result_val
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        if result_type != "data" {
            // Lifecycle frames like "started", "stopped", etc.
            return Ok(None);
        }

        if let Some(data_val) = result_val.get("data") {
            let inner = data_val.get("json").unwrap_or(data_val);

            // Parse event type and payload data
            if let Some(type_val) = inner.get("type").and_then(|t| t.as_str()) {
                let event_type = EventType::from(type_val);
                let payload_data = inner.get("data").cloned().unwrap_or_else(|| inner.clone());

                return Ok(Some(SubscriptionEvent {
                    id,
                    event_type,
                    data: payload_data,
                }));
            } else {
                return Ok(Some(SubscriptionEvent {
                    id,
                    event_type: EventType::Unknown("unknown".to_string()),
                    data: inner.clone(),
                }));
            }
        }
    }

    // Direct event envelope (without result wrapper)
    if let Some(type_val) = root.get("type").and_then(|t| t.as_str()) {
        let event_type = EventType::from(type_val);
        let payload_data = root.get("data").cloned().unwrap_or_else(|| root.clone());
        return Ok(Some(SubscriptionEvent {
            id,
            event_type,
            data: payload_data,
        }));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_serialization_and_parsing() {
        assert_eq!(EventType::from("userUpdated"), EventType::UserUpdated);
        assert_eq!(EventType::from("userUpdate"), EventType::UserUpdated);
        assert_eq!(EventType::from("userCreated"), EventType::UserCreated);
        assert_eq!(EventType::from("userCreate"), EventType::UserCreated);
        assert_eq!(EventType::from("userDeleted"), EventType::UserDeleted);
        assert_eq!(EventType::from("userDelete"), EventType::UserDeleted);
        assert_eq!(EventType::from("reactionAdded"), EventType::ReactionAdded);
        assert_eq!(EventType::from("reactionAdd"), EventType::ReactionAdded);
        assert_eq!(EventType::from("reactionRemoved"), EventType::ReactionRemoved);
        assert_eq!(EventType::from("reactionRemove"), EventType::ReactionRemoved);
        assert_eq!(EventType::from("channelCreated"), EventType::ChannelCreated);
        assert_eq!(EventType::from("channelCreate"), EventType::ChannelCreated);
        assert_eq!(EventType::from("channelUpdated"), EventType::ChannelUpdated);
        assert_eq!(EventType::from("channelUpdate"), EventType::ChannelUpdated);
        assert_eq!(EventType::from("channelDeleted"), EventType::ChannelDeleted);
        assert_eq!(EventType::from("channelDelete"), EventType::ChannelDeleted);
        assert_eq!(EventType::from("userPresenceChanged"), EventType::UserPresenceChanged);
        assert_eq!(EventType::from("presenceUpdate"), EventType::UserPresenceChanged);
        assert_eq!(EventType::from("userPresenceUpdate"), EventType::UserPresenceChanged);
        assert_eq!(EventType::from("userTyping"), EventType::UserTyping);
        assert_eq!(EventType::from("messageTyping"), EventType::UserTyping);
        assert_eq!(EventType::UserUpdated.as_str(), "userUpdated");
        assert_eq!(EventType::ReactionAdded.as_str(), "reactionAdded");
        assert_eq!(EventType::ChannelCreated.as_str(), "channelCreated");
    }

    #[test]
    fn test_decode_user_update_event_frame() {
        let frame = r#"{
            "id": "sub-1",
            "result": {
                "type": "data",
                "data": {
                    "type": "userUpdate",
                    "data": {
                        "id": 42,
                        "name": "UpdatedUser",
                        "bio": "New bio"
                    }
                }
            }
        }"#;

        let decoded = decode_websocket_frame(frame).unwrap().unwrap();
        assert_eq!(decoded.event_type, EventType::UserUpdated);
        assert_eq!(decoded.id.as_deref(), Some("sub-1"));
        assert_eq!(
            decoded.data.get("name").unwrap().as_str(),
            Some("UpdatedUser")
        );
    }
}
