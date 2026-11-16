use std::sync::{Arc, RwLock};

use futures_util::{SinkExt, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::{
    HeaderValue as WsHeaderValue, AUTHORIZATION as WS_AUTHORIZATION, COOKIE as WS_COOKIE,
};
use url::Url;

use crate::error::SharkordError;
use crate::protocol::events::{decode_websocket_frame, SubscriptionEvent};
use crate::protocol::{decode_trpc_response, encode_mutation_body, encode_query_params};
use crate::types::{Capabilities, Channel, Message, ServerSettings, User};

/// High-level asynchronous Sharkord client handling tRPC HTTP queries, mutations,
/// session persistence, and capability negotiation.
#[derive(Clone, Debug)]
pub struct SharkordClient {
    base_url: String,
    http_client: reqwest::Client,
    session_token: Arc<RwLock<Option<String>>>,
    capabilities: Arc<RwLock<Capabilities>>,
}

impl SharkordClient {
    /// Creates a new SharkordClient with URL normalization and scheme validation.
    ///
    /// Validates that `server_url` has an `http` or `https` scheme and trims trailing slashes.
    pub fn new(server_url: &str) -> Result<Self, SharkordError> {
        let trimmed = server_url.trim().trim_end_matches('/');
        let parsed = Url::parse(trimmed)?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(SharkordError::Protocol(format!(
                "Invalid URL scheme '{}': must be http or https",
                parsed.scheme()
            )));
        }

        let http_client = reqwest::Client::builder().cookie_store(true).build()?;

        Ok(Self {
            base_url: trimmed.to_string(),
            http_client,
            session_token: Arc::new(RwLock::new(None)),
            capabilities: Arc::new(RwLock::new(Capabilities::default())),
        })
    }

    /// Initializes client and immediately calls `get_public_settings` to negotiate capabilities.
    pub async fn connect(server_url: &str) -> Result<Self, SharkordError> {
        let client = Self::new(server_url)?;
        client.get_public_settings().await?;
        Ok(client)
    }

    /// Returns the normalized base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns the current negotiated server capabilities.
    pub fn capabilities(&self) -> Capabilities {
        self.capabilities
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Returns the active session token, if logged in.
    pub fn session_token(&self) -> Option<String> {
        self.session_token
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Updates or clears the session token.
    pub fn set_session_token(&self, token: Option<String>) {
        if let Ok(mut guard) = self.session_token.write() {
            *guard = token;
        }
    }

    /// Exposes internal reqwest client for crate-internal procedure extensions.
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Helper to inject Authorization and Cookie headers based on active session token.
    pub(crate) fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = self.session_token() {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, val);
            }
            if let Ok(val) = HeaderValue::from_str(&format!("token={}", token)) {
                headers.insert(COOKIE, val);
            }
        }
        headers
    }

    /// Fetches public server settings (`others.getPublicSettings`) and negotiates capabilities.
    pub async fn get_public_settings(&self) -> Result<ServerSettings, SharkordError> {
        let url = format!("{}/trpc/others.getPublicSettings?batch=1", self.base_url);
        let resp = self
            .http_client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} fetching public settings",
                resp.status()
            )));
        }

        let is_plus_header = resp
            .headers()
            .get("x-sharkord-plus")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);

        let body = resp.text().await?;
        let settings: ServerSettings = decode_trpc_response(&body)?;

        let negotiated = Capabilities::from_settings(&settings, is_plus_header);
        if let Ok(mut caps) = self.capabilities.write() {
            *caps = negotiated;
        }

        Ok(settings)
    }

    /// Authenticates with the server via tRPC mutation `auth.login` (or `/login`).
    ///
    /// Persists session token and returns the authenticated User.
    pub async fn login(&self, identity: &str, password: &str) -> Result<User, SharkordError> {
        let payload = serde_json::json!({
            "identity": identity,
            "password": password
        });

        let trpc_url = format!("{}/trpc/auth.login?batch=1", self.base_url);
        let mutation_body = encode_mutation_body(Some(&payload))?;

        let mut resp = self
            .http_client
            .post(&trpc_url)
            .header("content-type", "application/json")
            .body(mutation_body)
            .send()
            .await?;

        // Fallback to HTTP /login if tRPC auth.login route does not exist
        if resp.status().as_u16() == 404 {
            let http_url = format!("{}/login", self.base_url);
            resp = self
                .http_client
                .post(&http_url)
                .header("content-type", "application/json")
                .json(&payload)
                .send()
                .await?;
        }

        let status = resp.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(SharkordError::Unauthorized(
                "Invalid login credentials".to_string(),
            ));
        }
        if !status.is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} during login",
                status
            )));
        }

        // Check for Set-Cookie header for session token
        if let Some(cookie) = resp
            .headers()
            .get("set-cookie")
            .and_then(|v| v.to_str().ok())
        {
            for part in cookie.split(';') {
                let part = part.trim();
                if let Some(token) = part.strip_prefix("token=") {
                    self.set_session_token(Some(token.to_string()));
                    break;
                }
            }
        }

        let body_str = resp.text().await?;
        let json_val: serde_json::Value = match decode_trpc_response::<serde_json::Value>(&body_str)
        {
            Ok(v) => v,
            Err(_) => serde_json::from_str(&body_str)?,
        };

        // Extract token from JSON if present
        if let Some(token) = json_val.get("token").and_then(|t| t.as_str()) {
            self.set_session_token(Some(token.to_string()));
        }

        // Extract User
        let user: User = if let Some(user_val) = json_val.get("user") {
            serde_json::from_value(user_val.clone())?
        } else {
            serde_json::from_value(json_val)?
        };

        Ok(user)
    }

    /// Fetches visible channels for the authenticated user (`channels.getChannels`).
    pub async fn get_channels(&self) -> Result<Vec<Channel>, SharkordError> {
        let url = format!("{}/trpc/channels.getChannels?batch=1", self.base_url);
        let resp = self
            .http_client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            return Err(SharkordError::Unauthorized(
                "Authentication required to get channels".to_string(),
            ));
        }
        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} fetching channels",
                resp.status()
            )));
        }

        let body = resp.text().await?;
        let channels: Vec<Channel> = decode_trpc_response(&body)?;
        Ok(channels)
    }

    /// Fetches message history for a channel (`messages.getMessages`).
    pub async fn get_messages(
        &self,
        channel_id: u32,
        cursor: Option<u32>,
    ) -> Result<Vec<Message>, SharkordError> {
        let input = serde_json::json!({
            "channelId": channel_id,
            "cursor": cursor
        });
        let query_params = encode_query_params(Some(&input))?;
        let url = format!(
            "{}/trpc/messages.getMessages?{}",
            self.base_url, query_params
        );

        let resp = self
            .http_client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            return Err(SharkordError::Unauthorized(
                "Authentication required to get messages".to_string(),
            ));
        }
        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} fetching messages",
                resp.status()
            )));
        }

        let body = resp.text().await?;
        let messages: Vec<Message> = decode_trpc_response(&body)?;
        Ok(messages)
    }

    /// Sends a text message to a channel (`messages.sendMessage`).
    pub async fn send_message(
        &self,
        channel_id: u32,
        content: &str,
    ) -> Result<Message, SharkordError> {
        let input = serde_json::json!({
            "channelId": channel_id,
            "content": content
        });
        let body = encode_mutation_body(Some(&input))?;
        let url = format!("{}/trpc/messages.sendMessage?batch=1", self.base_url);

        let resp = self
            .http_client
            .post(&url)
            .headers(self.auth_headers())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            return Err(SharkordError::Unauthorized(
                "Authentication required to send message".to_string(),
            ));
        }
        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} sending message",
                resp.status()
            )));
        }

        let body_str = resp.text().await?;
        match decode_trpc_response::<Message>(&body_str) {
            Ok(msg) => Ok(msg),
            Err(_) => {
                // If the server returns just the message ID
                let id: u64 = decode_trpc_response(&body_str)?;
                Ok(Message {
                    id,
                    channel_id: channel_id as u64,
                    content: Some(content.to_string()),
                    ..Default::default()
                })
            }
        }
    }

    /// Connects to the WebSocket endpoint and spawns a background listener delivering real-time SubscriptionEvents.
    pub async fn subscribe_events(
        &self,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<SubscriptionEvent>, SharkordError> {
        let ws_scheme = if self.base_url.starts_with("https://") {
            "wss"
        } else {
            "ws"
        };

        let mut parsed_url = Url::parse(&self.base_url)?;
        parsed_url
            .set_scheme(ws_scheme)
            .map_err(|_| SharkordError::Protocol("Failed to set WebSocket scheme".to_string()))?;

        if parsed_url.path().is_empty() {
            parsed_url.set_path("/");
        }

        if let Some(token) = self.session_token() {
            let mut pairs = parsed_url.query_pairs().into_owned().collect::<Vec<_>>();
            if !pairs.iter().any(|(k, _)| k == "token") {
                pairs.push(("token".to_string(), token));
                parsed_url.query_pairs_mut().clear().extend_pairs(pairs);
            }
        }

        let full_url = parsed_url.to_string();

        let mut request = full_url
            .into_client_request()
            .map_err(|e| SharkordError::Protocol(format!("Invalid WebSocket request: {e}")))?;

        if let Some(token) = self.session_token() {
            if let Ok(val) = WsHeaderValue::from_str(&format!("Bearer {token}")) {
                request.headers_mut().insert(WS_AUTHORIZATION, val);
            }
            if let Ok(val) = WsHeaderValue::from_str(&format!("token={token}")) {
                request.headers_mut().insert(WS_COOKIE, val);
            }
        }

        let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| SharkordError::Network(format!("WebSocket connection failed: {e}")))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SubscriptionEvent>();

        tokio::spawn(async move {
            while let Some(msg_res) = read.next().await {
                match msg_res {
                    Ok(msg) => match msg {
                        tokio_tungstenite::tungstenite::Message::Text(text) => {
                            match decode_websocket_frame(&text) {
                                Ok(Some(event)) => {
                                    if tx.send(event).is_err() {
                                        break;
                                    }
                                }
                                Ok(None) => {}
                                Err(_) => {}
                            }
                        }
                        tokio_tungstenite::tungstenite::Message::Ping(payload) => {
                            let _ = write
                                .send(tokio_tungstenite::tungstenite::Message::Pong(payload))
                                .await;
                        }
                        tokio_tungstenite::tungstenite::Message::Close(_) => {
                            break;
                        }
                        _ => {}
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}
