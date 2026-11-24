use desktop_gpui::state::AppState;
use desktop_gpui::ui::auth_modal::{
    AuthInputField, AuthModalState, AuthModalView, AuthTab, ServerConnectView,
};
use desktop_gpui::ui::WorkspaceView;
use sharkord_core::types::{Channel, ChannelType, ServerSettings};

#[test]
fn test_auth_modal_tabs_and_error_handling() {
    let mut state = AuthModalState::new();
    assert_eq!(state.active_tab, AuthTab::Login);
    assert_eq!(state.active_field, AuthInputField::Identity);

    state.set_tab(AuthTab::Register);
    assert_eq!(state.active_tab, AuthTab::Register);

    state.set_active_field(AuthInputField::Password);
    assert_eq!(state.active_field, AuthInputField::Password);

    state.set_active_field(AuthInputField::Email);
    assert_eq!(state.active_field, AuthInputField::Email);

    state.set_active_field(AuthInputField::ServerUrl);
    assert_eq!(state.active_field, AuthInputField::ServerUrl);

    state.set_error("Invalid credentials");
    assert_eq!(state.error_message.as_deref(), Some("Invalid credentials"));
}

#[test]
fn test_auth_modal_server_connect_and_field_helpers() {
    let mut state = AuthModalState::new();
    assert_eq!(state.server_url, "http://localhost:4991");
    assert_eq!(state.identity, "");
    assert_eq!(state.password, "");
    assert_eq!(state.email, "");
    assert!(!state.is_loading);
    assert!(state.remember_session);

    state.set_server_url("https://sharkord.example.com");
    assert_eq!(state.server_url, "https://sharkord.example.com");

    state.set_identity("alice");
    assert_eq!(state.identity, "alice");

    state.set_password("p@ssword123");
    assert_eq!(state.password, "p@ssword123");

    state.set_email("alice@example.com");
    assert_eq!(state.email, "alice@example.com");

    state.set_tab(AuthTab::ServerConnect);
    assert_eq!(state.active_tab, AuthTab::ServerConnect);

    state.set_loading(true);
    assert!(state.is_loading);

    state.set_error("Connection refused");
    assert_eq!(state.error_message.as_deref(), Some("Connection refused"));
    state.clear_error();
    assert_eq!(state.error_message, None);

    state.set_remember_session(false);
    assert!(!state.remember_session);
    state.toggle_remember_session();
    assert!(state.remember_session);
}

#[test]
fn test_auth_modal_view_and_server_connect_view_rendering() {
    let mut state = AuthModalState::new();
    let modal_view = AuthModalView::new(state.clone());
    let _element = modal_view.render_element();

    state.set_tab(AuthTab::ServerConnect);
    let connect_view = ServerConnectView::new(state.clone());
    let _connect_elem = connect_view.render_element();
    let _modal_connect_elem = AuthModalView::new(state.clone()).render_element();

    state.set_tab(AuthTab::Register);
    let _modal_register_elem = AuthModalView::new(state.clone()).render_element();

    state.set_error("Failed to connect");
    let _modal_error_elem = AuthModalView::new(state.clone()).render_element();
}

#[test]
fn test_workspace_view_auth_modal_and_bootstrap_transitions() {
    let mut state = AppState::new();
    assert!(!state.is_auth_modal_open);
    assert!(!state.is_connecting);

    state.set_auth_modal_open(true);
    assert!(state.is_auth_modal_open);

    let mut workspace = WorkspaceView::new(state);
    assert!(workspace.state.is_auth_modal_open);

    workspace.set_connecting(true);
    assert!(workspace.state.is_connecting);

    let settings = ServerSettings {
        server_id: "srv-production".to_string(),
        name: "Production Sharkord".to_string(),
        ..Default::default()
    };
    let channels = vec![
        Channel {
            id: 10,
            name: "announcements".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
    ];

    workspace.bootstrap_from_server(Some(settings), channels, vec![]);
    assert!(!workspace.state.is_connecting);
    assert!(!workspace.state.is_auth_modal_open);
    assert_eq!(workspace.state.active_channel_id, Some(10));
    assert_eq!(workspace.channel_tree.server_name, "Production Sharkord");
}
