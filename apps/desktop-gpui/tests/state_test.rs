use desktop_gpui::state::AppState;
use desktop_gpui::theme::*;
use sharkord_core::types::{Channel, ChannelType, Message};

#[test]
fn test_app_state_default_instantiation() {
    let state = AppState::new();
    assert!(state.channels.is_empty());
    assert!(state.messages.is_empty());
    assert_eq!(state.active_channel_id, None);
    assert_eq!(state.active_channel(), None);
    assert!(state.left_sidebar_open);
    assert!(state.right_sidebar_open);
    assert!(state.current_user.is_none());
    assert!(state.server_settings.is_none());
}

#[test]
fn test_channels_selection_and_active_resolution() {
    let mut state = AppState::new();
    let ch1 = Channel {
        id: 101,
        name: "general".to_string(),
        channel_type: ChannelType::Text,
        topic: Some("General chat".to_string()),
        private: false,
        is_dm: false,
        position: 0,
        category_id: None,
        created_at: None,
        updated_at: None,
    };
    let ch2 = Channel {
        id: 102,
        name: "voice-lounge".to_string(),
        channel_type: ChannelType::Voice,
        topic: None,
        private: false,
        is_dm: false,
        position: 1,
        category_id: None,
        created_at: None,
        updated_at: None,
    };

    state.set_channels(vec![ch1.clone(), ch2.clone()]);
    assert_eq!(state.channels.len(), 2);
    assert_eq!(state.active_channel(), None);

    state.select_channel(101);
    assert_eq!(state.active_channel_id, Some(101));
    let active = state
        .active_channel()
        .expect("channel 101 should be active");
    assert_eq!(active.name, "general");
    assert_eq!(active.channel_type, ChannelType::Text);

    state.select_channel(102);
    assert_eq!(
        state.active_channel().map(|c| c.name.as_str()),
        Some("voice-lounge")
    );

    // Selecting non-existent channel
    state.select_channel(999);
    assert_eq!(state.active_channel(), None);
}

#[test]
fn test_add_message_and_messages_for_active_channel() {
    let mut state = AppState::new();
    state.select_channel(101);

    let msg1 = Message {
        id: 1,
        channel_id: 101,
        content: Some("Hello in 101".to_string()),
        user_id: Some(42),
        ..Default::default()
    };
    let msg2 = Message {
        id: 2,
        channel_id: 102,
        content: Some("Message in 102".to_string()),
        user_id: Some(43),
        ..Default::default()
    };
    let msg3 = Message {
        id: 3,
        channel_id: 101,
        content: Some("Another message in 101".to_string()),
        user_id: Some(42),
        ..Default::default()
    };

    state.add_message(msg1);
    state.add_message(msg2);
    state.add_message(msg3);

    let active_msgs = state.messages_for_active_channel();
    assert_eq!(active_msgs.len(), 2);
    assert_eq!(active_msgs[0].id, 1);
    assert_eq!(active_msgs[0].content, Some("Hello in 101".to_string()));
    assert_eq!(active_msgs[1].id, 3);
    assert_eq!(
        active_msgs[1].content,
        Some("Another message in 101".to_string())
    );

    // Switch channel to 102
    state.select_channel(102);
    let ch2_msgs = state.messages_for_active_channel();
    assert_eq!(ch2_msgs.len(), 1);
    assert_eq!(ch2_msgs[0].id, 2);

    // Non-existent channel
    state.select_channel(999);
    assert!(state.messages_for_active_channel().is_empty());
}

#[test]
fn test_sidebar_toggles() {
    let mut state = AppState::new();
    assert!(state.left_sidebar_open);
    assert!(state.right_sidebar_open);

    state.toggle_left_sidebar();
    assert!(!state.left_sidebar_open);
    state.toggle_left_sidebar();
    assert!(state.left_sidebar_open);

    state.toggle_right_sidebar();
    assert!(!state.right_sidebar_open);
    state.toggle_right_sidebar();
    assert!(state.right_sidebar_open);
}

#[test]
fn test_theme_tokens_match_design() {
    assert_eq!(BG_BASE_HEX, "#0a0b0e");
    assert_eq!(BG_SURFACE_HEX, "#111216");
    assert_eq!(BG_SUBSURFACE_HEX, "#16171d");
    assert_eq!(BG_ELEVATED_HEX, "#1c1e24");
    assert_eq!(BORDER_HAIRLINE_HEX, "rgba(255, 255, 255, 0.08)");
    assert_eq!(BORDER_SUBTLE_HEX, "rgba(255, 255, 255, 0.12)");
    assert_eq!(TEXT_PRIMARY_HEX, "#f3f4f6");
    assert_eq!(TEXT_SECONDARY_HEX, "#9ca3af");
    assert_eq!(TEXT_MUTED_HEX, "#6b7280");
    assert_eq!(ACCENT_HEX, "#3b82f6");
    assert_eq!(ACCENT_HOVER_HEX, "#60a5fa");
    assert_eq!(STATUS_ONLINE_HEX, "#22c55e");
    assert_eq!(STATUS_IDLE_HEX, "#eab308");
    assert_eq!(STATUS_DND_HEX, "#ef4444");
    assert_eq!(STATUS_OFFLINE_HEX, "#6b7280");
    assert_eq!(STATUS_STREAMING_HEX, "#8b5cf6");

    assert_eq!(BG_BASE, 0x0a0b0e);
    assert_eq!(BG_SURFACE, 0x111216);
    assert_eq!(BG_SUBSURFACE, 0x16171d);
    assert_eq!(BG_ELEVATED, 0x1c1e24);
    assert_eq!(BORDER_HAIRLINE.0, 0xffffff);
    assert!((BORDER_HAIRLINE.1 - 0.08).abs() < f32::EPSILON);
    assert_eq!(BORDER_SUBTLE.0, 0xffffff);
    assert!((BORDER_SUBTLE.1 - 0.12).abs() < f32::EPSILON);
    assert_eq!(TEXT_PRIMARY, 0xf3f4f6);
    assert_eq!(TEXT_SECONDARY, 0x9ca3af);
    assert_eq!(TEXT_MUTED, 0x6b7280);
    assert_eq!(ACCENT, 0x3b82f6);
    assert_eq!(ACCENT_HOVER, 0x60a5fa);
    assert_eq!(STATUS_ONLINE, 0x22c55e);
    assert_eq!(STATUS_IDLE, 0xeab308);
    assert_eq!(STATUS_DND, 0xef4444);
    assert_eq!(STATUS_OFFLINE, 0x6b7280);
    assert_eq!(STATUS_STREAMING, 0x8b5cf6);

    // Verify conversion helpers work and produce gpui colors
    let _bg = bg_base();
    let _acc = accent();
    let _border = border_hairline();
    let _txt = text_primary();

    let bg_rgba = bg_base_rgba();
    let acc_rgba = accent_rgba();
    let border_rgba = border_hairline_rgba();
    let txt_rgba = text_primary_rgba();

    assert!((border_rgba.a - 0.08).abs() < 0.01);
    assert_eq!(bg_rgba.a, 1.0);
    assert_eq!(acc_rgba.a, 1.0);
    assert_eq!(txt_rgba.a, 1.0);
}

#[test]
fn test_user_state_mutations() {
    use sharkord_core::types::User;

    let mut state = AppState::new();
    let user1 = User {
        id: 1,
        name: "Alice".to_string(),
        identity: Some("alice".to_string()),
        profile_color: Some("#3b82f6".to_string()),
        bio: Some("Hello world".to_string()),
        avatar_id: None,
        banner_id: None,
        banned: Some(false),
        created_at: Some(1000),
        role_ids: vec![1, 2],
        status: Some("ONLINE".to_string()),
        session_count: None,
    };

    state.current_user = Some(user1.clone());
    state.add_user(user1.clone());

    assert_eq!(state.users.len(), 1);
    assert_eq!(state.user_by_id(1).unwrap().name, "Alice");

    // Real-time update simulation
    let mut updated_user = user1.clone();
    updated_user.name = "Alice In Wonderland".to_string();
    updated_user.bio = Some("Updated bio".to_string());

    state.update_user(updated_user);

    assert_eq!(state.users.len(), 1);
    assert_eq!(state.user_by_id(1).unwrap().name, "Alice In Wonderland");
    assert_eq!(
        state.current_user.as_ref().unwrap().name,
        "Alice In Wonderland"
    );
    assert_eq!(
        state.current_user.as_ref().unwrap().bio.as_deref(),
        Some("Updated bio")
    );

    state.remove_user(1);
    assert!(state.users.is_empty());
    assert_eq!(state.user_by_id(1), None);
}
