use desktop_gpui::state::AppState;
use desktop_gpui::ui::channel_admin_modal::{
    CreateChannelModalState, CreateChannelModalView, EditChannelModalState, EditChannelModalView,
};
use desktop_gpui::ui::server_settings_modal::{
    ServerSettingsModalState, ServerSettingsModalView, ServerSettingsTab,
};
use desktop_gpui::ui::WorkspaceView;
use sharkord_core::types::{ChannelType, Role, User};

#[test]
fn test_create_channel_modal_type_selection() {
    let mut state = CreateChannelModalState::new();
    assert!(!state.is_open);
    assert_eq!(state.channel_type, ChannelType::Text);
    assert_eq!(state.name, "");
    assert_eq!(state.category, "");

    state.open("TEXT CHANNELS");
    assert!(state.is_open);
    assert_eq!(state.category, "TEXT CHANNELS");

    state.set_channel_type(ChannelType::Voice);
    assert_eq!(state.channel_type, ChannelType::Voice);

    state.set_name("general-voice");
    assert_eq!(state.name, "general-voice");

    state.set_category("VOICE CHANNELS");
    assert_eq!(state.category, "VOICE CHANNELS");

    state.close();
    assert!(!state.is_open);
}

#[test]
fn test_edit_channel_modal_rename_and_deletion() {
    let mut state = EditChannelModalState::new();
    assert!(!state.is_open);
    assert_eq!(state.channel_id, 0);
    assert_eq!(state.name, "");
    assert_eq!(state.topic, "");
    assert!(!state.confirm_delete);

    state.open(101, "dev-chat", "Discussion for developers");
    assert!(state.is_open);
    assert_eq!(state.channel_id, 101);
    assert_eq!(state.name, "dev-chat");
    assert_eq!(state.topic, "Discussion for developers");

    state.set_name("developer-lounge");
    assert_eq!(state.name, "developer-lounge");

    state.set_topic("General hangout for dev team");
    assert_eq!(state.topic, "General hangout for dev team");

    state.toggle_confirm_delete();
    assert!(state.confirm_delete);

    state.toggle_confirm_delete();
    assert!(!state.confirm_delete);

    state.set_confirm_delete(true);
    assert!(state.confirm_delete);

    state.reset();
    assert!(!state.is_open);
    assert!(!state.confirm_delete);
    assert_eq!(state.channel_id, 0);
    assert_eq!(state.name, "");
    assert_eq!(state.topic, "");
}

#[test]
fn test_server_settings_modal_tabs() {
    let mut state = ServerSettingsModalState::new();
    assert!(!state.is_open);
    assert_eq!(state.active_tab, ServerSettingsTab::Overview);
    assert_eq!(state.active_tab.label(), "Overview");
    assert_eq!(ServerSettingsTab::Roles.label(), "Roles");
    assert_eq!(ServerSettingsTab::Members.label(), "Members");
    assert_eq!(ServerSettingsTab::Invites.label(), "Invites");

    state.open();
    assert!(state.is_open);

    state.set_tab(ServerSettingsTab::Roles);
    assert_eq!(state.active_tab, ServerSettingsTab::Roles);

    state.set_tab(ServerSettingsTab::Members);
    assert_eq!(state.active_tab, ServerSettingsTab::Members);

    state.set_tab(ServerSettingsTab::Invites);
    assert_eq!(state.active_tab, ServerSettingsTab::Invites);

    state.set_server_name("Sharkord Production Server");
    assert_eq!(state.server_name, "Sharkord Production Server");

    let roles = vec![Role {
        id: 1,
        name: "Admin".to_string(),
        color: "#3B82F6".to_string(),
        is_persistent: true,
        is_default: false,
        storage_quota_override_enabled: false,
        storage_space_quota: 0,
        permissions: vec!["ADMINISTRATOR".to_string()],
        created_at: Some(100),
        updated_at: None,
    }];
    state.set_roles(roles);
    assert_eq!(state.roles.len(), 1);
    assert_eq!(state.roles[0].name, "Admin");

    let members = vec![User {
        id: 1,
        name: "Alice".to_string(),
        status: Some("online".to_string()),
        ..Default::default()
    }];
    state.set_members(members);
    assert_eq!(state.members.len(), 1);
    assert_eq!(state.members[0].name, "Alice");

    state.close();
    assert!(!state.is_open);
}

#[test]
fn test_admin_modals_view_rendering() {
    let mut create_state = CreateChannelModalState::new();
    create_state.open("TEXT CHANNELS");
    create_state.set_name("general");
    let create_view = CreateChannelModalView::new(create_state.clone());
    let _el1 = create_view.render_element();

    create_state.set_channel_type(ChannelType::Voice);
    let _el2 = CreateChannelModalView::new(create_state).render_element();

    let mut edit_state = EditChannelModalState::new();
    edit_state.open(1, "general", "General discussion");
    let edit_view = EditChannelModalView::new(edit_state.clone());
    let _el3 = edit_view.render_element();

    edit_state.set_confirm_delete(true);
    let _el4 = EditChannelModalView::new(edit_state).render_element();

    let mut settings_state = ServerSettingsModalState::new();
    settings_state.open();
    settings_state.set_server_name("Sharkord Pro");
    settings_state.set_roles(vec![Role {
        id: 1,
        name: "Moderator".to_string(),
        color: "#10B981".to_string(),
        is_persistent: false,
        is_default: false,
        storage_quota_override_enabled: false,
        storage_space_quota: 0,
        permissions: vec!["MANAGE_MESSAGES".to_string()],
        created_at: None,
        updated_at: None,
    }]);
    settings_state.set_members(vec![User {
        id: 1,
        name: "Bob".to_string(),
        status: Some("online".to_string()),
        ..Default::default()
    }]);

    settings_state.set_tab(ServerSettingsTab::Overview);
    let _el5 = ServerSettingsModalView::new(settings_state.clone()).render_element();

    settings_state.set_tab(ServerSettingsTab::Roles);
    let _el6 = ServerSettingsModalView::new(settings_state.clone()).render_element();

    settings_state.set_tab(ServerSettingsTab::Members);
    let _el7 = ServerSettingsModalView::new(settings_state.clone()).render_element();

    settings_state.set_tab(ServerSettingsTab::Invites);
    let _el8 = ServerSettingsModalView::new(settings_state.clone()).render_element();
}

#[test]
fn test_workspace_view_admin_modals_integration() {
    let app_state = AppState::new_mock();
    let mut workspace = WorkspaceView::new(app_state);

    assert!(!workspace.create_channel_modal.is_open);
    assert!(!workspace.edit_channel_modal.is_open);
    assert!(!workspace.server_settings_modal.is_open);

    workspace.open_create_channel("TEXT CHANNELS");
    assert!(workspace.create_channel_modal.is_open);
    assert_eq!(workspace.create_channel_modal.category, "TEXT CHANNELS");
    workspace.close_create_channel();
    assert!(!workspace.create_channel_modal.is_open);

    workspace.open_edit_channel(1);
    assert!(workspace.edit_channel_modal.is_open);
    assert_eq!(workspace.edit_channel_modal.channel_id, 1);
    assert_eq!(workspace.edit_channel_modal.name, "general");
    assert_eq!(
        workspace.edit_channel_modal.topic,
        "General discussion for VibeShark"
    );
    workspace.close_edit_channel();
    assert!(!workspace.edit_channel_modal.is_open);

    workspace.open_server_settings();
    assert!(workspace.server_settings_modal.is_open);
    assert_eq!(
        workspace.server_settings_modal.server_name,
        "VibeShark HQ"
    );
    workspace.close_server_settings();
    assert!(!workspace.server_settings_modal.is_open);

    workspace.open_create_channel("TEXT CHANNELS");
    workspace.create_channel_modal.set_name("random");
    let new_ch = workspace.submit_create_channel();
    assert!(new_ch.is_some());
    let ch = new_ch.unwrap();
    assert_eq!(ch.name, "random");
    assert!(!workspace.create_channel_modal.is_open);
    assert_eq!(workspace.state.channels.len(), 3);

    workspace.open_edit_channel(1);
    workspace.edit_channel_modal.set_name("general-chat");
    workspace
        .edit_channel_modal
        .set_topic("Updated topic discussion");
    let edited_id = workspace.submit_edit_channel();
    assert_eq!(edited_id, Some(1));
    assert_eq!(workspace.state.channels[0].name, "general-chat");
    assert_eq!(
        workspace.state.channels[0].topic.as_deref(),
        Some("Updated topic discussion")
    );
    assert!(!workspace.edit_channel_modal.is_open);

    workspace.open_edit_channel(2);
    workspace.edit_channel_modal.set_confirm_delete(true);
    let deleted_id = workspace.submit_delete_channel();
    assert_eq!(deleted_id, Some(2));
    assert_eq!(workspace.state.channels.len(), 2);
    assert!(workspace.state.channels.iter().all(|c| c.id != 2));
    assert!(!workspace.edit_channel_modal.is_open);
}
