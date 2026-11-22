use desktop_gpui::network::{BootstrapData, NetworkBridge};
use desktop_gpui::state::AppState;
use sharkord_core::types::{Channel, ChannelType, Message, ServerSettings};
use sharkord_core::vault::SavedServerSession;

#[test]
fn test_app_state_auto_login_from_vault() {
    let mut state = AppState::new();
    let session = SavedServerSession {
        server_id: "test".to_string(),
        server_name: "Test Server".to_string(),
        server_url: "http://localhost:4991".to_string(),
        token: "mock_token".to_string(),
        user_id: 1,
        username: "admin".to_string(),
        display_name: "Admin".to_string(),
        avatar_url: None,
        last_accessed_at: 1000,
    };
    state.apply_saved_session(&session);
    assert!(state.current_user.is_some());
    assert_eq!(state.current_user.as_ref().unwrap().name, "admin");
    assert_eq!(state.active_server_id.as_deref(), Some("test"));
}

#[test]
fn test_app_state_bootstrap_from_server() {
    let mut state = AppState::new();
    state.is_connecting = true;
    state.is_auth_modal_open = true;

    let settings = ServerSettings {
        server_id: "server-1".to_string(),
        name: "My Server".to_string(),
        ..Default::default()
    };
    let channels = vec![
        Channel {
            id: 10,
            name: "lobby".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 20,
            name: "lounge".to_string(),
            channel_type: ChannelType::Voice,
            ..Default::default()
        },
    ];
    let messages = vec![Message {
        id: 100,
        channel_id: 10,
        content: Some("Welcome to lobby".to_string()),
        ..Default::default()
    }];

    state.bootstrap_from_server(settings, channels, messages);

    assert_eq!(state.is_connecting, false);
    assert_eq!(state.is_auth_modal_open, false);
    assert_eq!(state.channels.len(), 2);
    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.active_channel_id, Some(10));
    assert_eq!(
        state.server_settings.as_ref().map(|s| s.name.as_str()),
        Some("My Server")
    );
}

#[test]
fn test_app_state_bootstrap_from_data() {
    let mut state = AppState::new();
    state.is_connecting = true;

    let data = BootstrapData {
        settings: ServerSettings {
            server_id: "s1".to_string(),
            name: "Test Server".to_string(),
            ..Default::default()
        },
        channels: vec![Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        }],
        initial_messages: vec![],
        active_channel_id: Some(1),
    };

    state.bootstrap_from_data(data);
    assert!(!state.is_connecting);
    assert_eq!(state.active_channel_id, Some(1));
}

#[test]
fn test_network_bridge_lifecycle_and_client() {
    let bridge = NetworkBridge::new().expect("Failed to create NetworkBridge");
    assert!(!bridge.is_connected());
    assert!(bridge.client().is_none());

    let client = sharkord_core::client::SharkordClient::new("http://localhost:4991")
        .expect("Failed to create client");
    bridge.set_client(client);
    assert!(bridge.is_connected());
    assert!(bridge.client().is_some());

    bridge.clear_client();
    assert!(!bridge.is_connected());
    assert!(bridge.client().is_none());
}

#[test]
fn test_network_bridge_runtime_execution() {
    let bridge = NetworkBridge::new().expect("Failed to create NetworkBridge");
    let result = bridge.block_on(async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        42
    });
    assert_eq!(result, 42);
}
