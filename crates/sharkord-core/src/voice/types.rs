use serde::{Deserialize, Serialize};

/// Initial voice state when joining a voice channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VoiceStateInput {
    pub mic_muted: bool,
    pub sound_muted: bool,
}

/// Response returned from joining a voice channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceJoinResponse {
    pub router_rtp_capabilities: serde_json::Value,
}

/// Mediasoup WebRTC Transport options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportOptions {
    pub id: String,
    pub ice_parameters: serde_json::Value,
    #[serde(default)]
    pub ice_candidates: Vec<serde_json::Value>,
    pub dtls_parameters: serde_json::Value,
}

/// DTLS parameters for connecting a transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtlsParameters {
    pub role: Option<String>,
    #[serde(default)]
    pub fingerprints: Vec<serde_json::Value>,
}

/// Options for producing a media track (audio or video).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerOptions {
    pub kind: String,
    pub rtp_parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<serde_json::Value>,
}

/// Response when a producer is successfully created on Mediasoup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProduceResponse {
    pub id: String,
}
