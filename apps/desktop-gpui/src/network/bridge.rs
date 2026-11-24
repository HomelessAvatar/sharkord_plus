//! Background Tokio runtime bridge and asynchronous network orchestration
//! for the Sharkord GPUI desktop client.

use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use sharkord_core::client::SharkordClient;
use sharkord_core::error::SharkordError;
use sharkord_core::protocol::events::SubscriptionEvent;
use sharkord_core::types::{Channel, ChannelType, Message, Role, ServerSettings, User};
use sharkord_core::vault::SavedServerSession;
use tokio::runtime::Runtime;

/// Payload holding initial server configuration, channel trees, and message feeds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BootstrapData {
    pub settings: ServerSettings,
    pub channels: Vec<Channel>,
    pub initial_messages: Vec<Message>,
    pub active_channel_id: Option<u64>,
    #[serde(default)]
    pub current_user: Option<User>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub roles: Vec<Role>,
}

/// Asynchronous network bridge wrapping Tokio runtime and SharkordClient.
#[derive(Clone)]
pub struct NetworkBridge {
    runtime: Arc<Runtime>,
    client: Arc<RwLock<Option<SharkordClient>>>,
}

impl std::fmt::Debug for NetworkBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkBridge").finish_non_exhaustive()
    }
}

impl PartialEq for NetworkBridge {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.runtime, &other.runtime)
    }
}

impl NetworkBridge {
    /// Creates a new `NetworkBridge` with a dedicated multi-threaded Tokio runtime.
    pub fn new() -> Result<Self, SharkordError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(SharkordError::Io)?;
        Ok(Self {
            runtime: Arc::new(runtime),
            client: Arc::new(RwLock::new(None)),
        })
    }

    /// Creates a `NetworkBridge` wrapping an existing Tokio runtime.
    pub fn with_runtime(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            client: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns a reference to the underlying Tokio runtime.
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Returns a handle to the Tokio runtime.
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }

    /// Blocks on an async task using the internal Tokio runtime.
    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Returns a cloned copy of the active `SharkordClient`, if connected.
    pub fn client(&self) -> Option<SharkordClient> {
        self.client.read().ok().and_then(|guard| guard.clone())
    }

    /// Sets or replaces the active `SharkordClient`.
    pub fn set_client(&self, client: SharkordClient) {
        if let Ok(mut guard) = self.client.write() {
            *guard = Some(client);
        }
    }

    /// Clears the active `SharkordClient`.
    pub fn clear_client(&self) {
        if let Ok(mut guard) = self.client.write() {
            *guard = None;
        }
    }

    /// Returns true if a client instance is currently initialized.
    pub fn is_connected(&self) -> bool {
        self.client.read().ok().map(|g| g.is_some()).unwrap_or(false)
    }

    /// Connects to a Sharkord server and fetches its public settings.
    pub async fn connect_server(&self, server_url: &str) -> Result<ServerSettings, SharkordError> {
        let client = SharkordClient::connect(server_url).await?;
        let settings = client.get_public_settings().await?;
        self.set_client(client);
        Ok(settings)
    }

    /// Alias for `connect_server`.
    pub async fn connect(&self, server_url: &str) -> Result<ServerSettings, SharkordError> {
        self.connect_server(server_url).await
    }

    /// Connects to a Sharkord server with an existing authentication token.
    pub async fn connect_with_token(
        &self,
        server_url: &str,
        token: &str,
    ) -> Result<ServerSettings, SharkordError> {
        let client = SharkordClient::new(server_url)?;
        client.set_session_token(Some(token.to_string()));
        let settings = client.get_public_settings().await?;
        self.set_client(client);
        Ok(settings)
    }

    /// Connects to a Sharkord server using a `SavedServerSession`.
    pub async fn connect_with_session(
        &self,
        session: &SavedServerSession,
    ) -> Result<ServerSettings, SharkordError> {
        self.connect_with_token(&session.server_url, &session.token).await
    }

    /// Authenticates with the server using identity and password.
    pub async fn login(&self, identity: &str, password: &str) -> Result<User, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol(
                "NetworkBridge has no active client connection. Call connect_server first.".to_string(),
            )
        })?;
        let user = client.login(identity, password).await?;
        self.set_client(client);
        Ok(user)
    }

    /// Connects to a server and logs in in a single flow.
    pub async fn connect_and_login(
        &self,
        server_url: &str,
        identity: &str,
        password: &str,
    ) -> Result<User, SharkordError> {
        let client = SharkordClient::new(server_url)?;
        let user = client.login(identity, password).await?;
        let _ = client.get_public_settings().await;
        self.set_client(client);
        Ok(user)
    }

    /// Bootstraps server settings, channels, and initial messages for the active text channel.
    pub async fn bootstrap(&self) -> Result<BootstrapData, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("NetworkBridge has no active client connection".to_string())
        })?;

        let settings = client
            .get_public_settings()
            .await
            .unwrap_or_else(|_| ServerSettings {
                server_id: "sharkord-server".to_string(),
                name: "Sharkord Server".to_string(),
                ..Default::default()
            });

        let channels = client.get_channels().await?;

        let active_channel = channels
            .iter()
            .find(|c| c.channel_type == ChannelType::Text)
            .or_else(|| channels.first());

        let active_channel_id = active_channel.map(|c| c.id);

        let initial_messages = if let Some(chan_id) = active_channel_id {
            client
                .get_messages(chan_id as u32, None)
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(BootstrapData {
            settings,
            channels,
            initial_messages,
            active_channel_id,
            current_user: None,
            users: Vec::new(),
            roles: Vec::new(),
        })
    }

    /// Alias for `bootstrap`.
    pub async fn bootstrap_channels_and_messages(&self) -> Result<BootstrapData, SharkordError> {
        self.bootstrap().await
    }

    /// Fetches all accessible channels.
    pub async fn get_channels(&self) -> Result<Vec<Channel>, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("Not connected to a server".to_string())
        })?;
        client.get_channels().await
    }

    /// Fetches messages for a channel.
    pub async fn get_messages(&self, channel_id: u64) -> Result<Vec<Message>, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("Not connected to a server".to_string())
        })?;
        client.get_messages(channel_id as u32, None).await
    }

    /// Sends a text message to a channel.
    pub async fn send_message(
        &self,
        channel_id: u64,
        content: &str,
    ) -> Result<Message, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("Not connected to a server".to_string())
        })?;
        client.send_message(channel_id as u32, content).await
    }

    /// Subscribes to real-time events via the active client connection.
    pub async fn subscribe_events(
        &self,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<SubscriptionEvent>, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("Not connected to a server".to_string())
        })?;
        client.subscribe_events().await
    }

    /// Spawns a background task that listens for incoming WebSocket events and dispatches
    /// them to the provided thread-safe `AppState` using `AppState::handle_subscription_event`.
    pub fn spawn_event_dispatcher(
        &self,
        state: Arc<RwLock<crate::state::AppState>>,
    ) -> Result<tokio::task::JoinHandle<()>, SharkordError> {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("Not connected to a server".to_string())
        })?;

        let handle = self.runtime.spawn(async move {
            if let Ok(mut rx) = client.subscribe_events().await {
                while let Some(event) = rx.recv().await {
                    if let Ok(mut app_state) = state.write() {
                        app_state.handle_subscription_event(event);
                    }
                }
            }
        });

        Ok(handle)
    }

    /// Spawns a background listener forwarding events to an arbitrary handler closure.
    pub fn spawn_event_handler<F>(
        &self,
        mut handler: F,
    ) -> Result<tokio::task::JoinHandle<()>, SharkordError>
    where
        F: FnMut(SubscriptionEvent) + Send + 'static,
    {
        let client = self.client().ok_or_else(|| {
            SharkordError::Protocol("Not connected to a server".to_string())
        })?;

        let handle = self.runtime.spawn(async move {
            if let Ok(mut rx) = client.subscribe_events().await {
                while let Some(event) = rx.recv().await {
                    handler(event);
                }
            }
        });

        Ok(handle)
    }

    /// Dispatches an event directly to an `Arc<RwLock<AppState>>`.
    pub fn dispatch_event(state: &Arc<RwLock<crate::state::AppState>>, event: SubscriptionEvent) {
        if let Ok(mut app_state) = state.write() {
            app_state.handle_subscription_event(event);
        }
    }
}
