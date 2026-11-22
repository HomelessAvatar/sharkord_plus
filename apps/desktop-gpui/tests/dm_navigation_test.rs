use desktop_gpui::ui::dm_view::{DirectMessagesSidebarView, DmConversation, DmSidebarState};
use desktop_gpui::ui::server_rail::ServerRailView;
use desktop_gpui::ui::user_profile_modal::{UserProfileCardState, UserProfileModalView, UserProfilePopoverView};

#[test]
fn test_dm_sidebar_selection() {
    let mut state = DmSidebarState::new();
    assert_eq!(state.active_dm_channel_id, None);
    assert_eq!(state.conversations.len(), 0);
    assert_eq!(state.unread_total(), 0);

    let conv1 = DmConversation {
        channel_id: 101,
        user_id: 2,
        username: "alice".to_string(),
        display_name: Some("Alice Smith".to_string()),
        status: "online".to_string(),
        last_message: Some("Hey there! Check out GPUI".to_string()),
        unread_count: 3,
    };
    let conv2 = DmConversation {
        channel_id: 102,
        user_id: 3,
        username: "bob".to_string(),
        display_name: None,
        status: "idle".to_string(),
        last_message: Some("See you tomorrow".to_string()),
        unread_count: 0,
    };

    state.add_conversation(conv1);
    state.add_conversation(conv2);
    assert_eq!(state.conversations.len(), 2);
    assert_eq!(state.unread_total(), 3);

    // Active conversation selection
    state.set_active_dm(Some(101));
    assert_eq!(state.active_dm_channel_id, Some(101));
    assert_eq!(
        state.active_conversation().map(|c| c.username.as_str()),
        Some("alice")
    );
    assert_eq!(
        state.active_conversation().and_then(|c| c.display_name.as_deref()),
        Some("Alice Smith")
    );

    // Mark as read
    state.mark_read(101);
    assert_eq!(state.conversations[0].unread_count, 0);
    assert_eq!(state.unread_total(), 0);

    // Remove conversation
    let removed = state.remove_conversation(102);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().username, "bob");
    assert_eq!(state.conversations.len(), 1);

    // DirectMessagesSidebarView rendering check
    let sidebar_view = DirectMessagesSidebarView::new(state.clone());
    assert_eq!(sidebar_view.conversations_count(), 1);
    let _element = sidebar_view.render_element();
}

#[test]
fn test_user_profile_card_state() {
    let mut profile = UserProfileCardState::new();
    assert!(!profile.is_open);
    assert_eq!(profile.user_id, None);

    profile.open(42, "Bob", "bob@example.com", "online");
    assert!(profile.is_open);
    assert_eq!(profile.user_id, Some(42));
    assert_eq!(profile.username, "Bob");
    assert_eq!(profile.display_name.as_deref(), Some("bob@example.com"));
    assert_eq!(profile.status, "online");

    profile.set_bio("Rust & GPUI desktop parity engineer");
    assert_eq!(
        profile.bio.as_deref(),
        Some("Rust & GPUI desktop parity engineer")
    );

    profile.set_role("Core Contributor");
    assert_eq!(profile.role.as_deref(), Some("Core Contributor"));

    profile.set_avatar_url("https://example.com/avatar.png");
    assert_eq!(
        profile.avatar_url.as_deref(),
        Some("https://example.com/avatar.png")
    );

    profile.close();
    assert!(!profile.is_open);
}

#[test]
fn test_server_rail_home_toggle() {
    let mut rail = ServerRailView::default();
    assert!(rail.is_home_selected);
    assert!(rail.is_home_active);

    rail.set_home_selected(false);
    assert!(!rail.is_home_selected);
    assert!(!rail.is_home_active);

    rail.toggle_home();
    assert!(rail.is_home_selected);
    assert!(rail.is_home_active);

    rail.toggle_home();
    assert!(!rail.is_home_selected);
    assert!(!rail.is_home_active);
}

#[test]
fn test_user_profile_popover_view_rendering() {
    let mut profile = UserProfileCardState::new();
    profile.open(99, "alice", "Alice Wonderland", "online");
    profile.set_bio("Deep into the Rabbit Hole of Rust");
    profile.set_role("Administrator");
    profile.set_avatar_url("https://sharkord.example.com/avatars/alice.png");

    let popover = UserProfilePopoverView::new(profile.clone());
    assert_eq!(popover.username(), "alice");
    assert_eq!(popover.display_name(), "Alice Wonderland");
    assert_eq!(popover.status(), "online");
    assert_eq!(popover.bio(), Some("Deep into the Rabbit Hole of Rust"));
    assert_eq!(popover.role_badge(), Some("Administrator"));
    assert_eq!(popover.action_label(), "Send Direct Message");

    // Ensure render_element returns an element without panics
    let _element = popover.render_element();

    // Ensure UserProfileModalView alias functions interchangeably
    let modal_view: UserProfileModalView = UserProfileModalView::new(profile);
    let _modal_element = modal_view.render_element();
}
