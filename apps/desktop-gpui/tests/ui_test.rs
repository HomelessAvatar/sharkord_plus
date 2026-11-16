use std::collections::HashMap;

use desktop_gpui::state::AppState;
use desktop_gpui::theme;
use desktop_gpui::ui::channel_tree::{channel_icon, compose_channel_tree, ChannelDisplayItem};
use desktop_gpui::ui::chat_canvas::{
    format_message_timestamp, group_messages_for_feed, MessageFeedGroup,
};
use desktop_gpui::ui::server_rail::{server_initials, server_pill_height, ServerItem};
use desktop_gpui::ui::voice_bar::{
    voice_action_labels, voice_status_indicator_color, voice_status_text, VoiceBarState,
    VoiceConnectionState,
};
use desktop_gpui::ui::WorkspaceView;
use sharkord_core::types::{Channel, ChannelType, Message, User};

#[test]
fn test_server_rail_helpers() {
    // Formatting initials
    assert_eq!(server_initials("Sharkord Plus"), "SP");
    assert_eq!(server_initials("Gaming Community Lounge"), "GCL");
    assert_eq!(server_initials("Rustaceans"), "RU");
    assert_eq!(server_initials("A"), "A");
    assert_eq!(server_initials(""), "?");

    // Indicator pill height calculation
    assert_eq!(server_pill_height(true, false), 40.0); // Active server: full pill
    assert_eq!(server_pill_height(false, true), 8.0); // Inactive with unread: small dot pill
    assert_eq!(server_pill_height(false, false), 0.0); // Inactive without unread: hidden

    // ServerItem representation
    let item = ServerItem {
        id: "srv-1".to_string(),
        name: "Sharkord HQ".to_string(),
        icon_url: None,
        is_active: true,
        has_unread: false,
    };
    assert_eq!(item.id, "srv-1");
    assert!(item.is_active);
}

#[test]
fn test_channel_tree_composition() {
    // Channel type icons
    assert_eq!(channel_icon(ChannelType::Text), "#");
    assert_eq!(channel_icon(ChannelType::Voice), "🔊");

    let ch1 = Channel {
        id: 1,
        name: "announcements".to_string(),
        channel_type: ChannelType::Text,
        topic: Some("Important updates".to_string()),
        private: false,
        is_dm: false,
        position: 0,
        category_id: None,
        created_at: Some(1000),
        updated_at: None,
    };
    let ch2 = Channel {
        id: 2,
        name: "general".to_string(),
        channel_type: ChannelType::Text,
        topic: Some("General discussion".to_string()),
        private: false,
        is_dm: false,
        position: 0,
        category_id: Some(10),
        created_at: Some(1000),
        updated_at: None,
    };
    let ch3 = Channel {
        id: 3,
        name: "help".to_string(),
        channel_type: ChannelType::Text,
        topic: None,
        private: false,
        is_dm: false,
        position: 1,
        category_id: Some(10),
        created_at: Some(1000),
        updated_at: None,
    };
    let ch4 = Channel {
        id: 4,
        name: "voice-lounge".to_string(),
        channel_type: ChannelType::Voice,
        topic: None,
        private: false,
        is_dm: false,
        position: 0,
        category_id: Some(20),
        created_at: Some(1000),
        updated_at: None,
    };

    let channels = vec![ch1, ch2, ch3, ch4];
    let groups = compose_channel_tree(&channels, Some(2));

    // Groups should partition channels and mark active item
    assert!(!groups.is_empty());

    // Find the channel with id 2 and verify is_active is true
    let active_item: Option<&ChannelDisplayItem> = groups
        .iter()
        .flat_map(|g| &g.channels)
        .find(|item| item.channel.id == 2);
    assert!(active_item.is_some());
    assert!(active_item.unwrap().is_active);

    // Channel 1 should not be active
    let inactive_item = groups
        .iter()
        .flat_map(|g| &g.channels)
        .find(|item| item.channel.id == 1)
        .expect("channel 1 should be in tree");
    assert!(!inactive_item.is_active);
}

#[test]
fn test_message_feed_item_grouping() {
    let mut users = HashMap::new();
    users.insert(10u64, "Alice".to_string());
    users.insert(20u64, "Bob".to_string());

    let msg1 = Message {
        id: 1,
        channel_id: 101,
        user_id: Some(10),
        content: Some("First message from Alice".to_string()),
        created_at: Some(1000),
        ..Default::default()
    };
    let msg2 = Message {
        id: 2,
        channel_id: 101,
        user_id: Some(10),
        content: Some("Second message from Alice shortly after".to_string()),
        created_at: Some(1100), // +100s <= 300s window -> grouped
        ..Default::default()
    };
    let msg3 = Message {
        id: 3,
        channel_id: 101,
        user_id: Some(20),
        content: Some("Hello from Bob".to_string()),
        created_at: Some(1150), // Different user -> new group
        ..Default::default()
    };
    let msg4 = Message {
        id: 4,
        channel_id: 101,
        user_id: Some(20),
        content: Some("Bob speaks again much later".to_string()),
        created_at: Some(1600), // +450s > 300s window -> new group
        ..Default::default()
    };

    let messages = vec![msg1, msg2, msg3, msg4];
    let groups: Vec<MessageFeedGroup> = group_messages_for_feed(&messages, &users);

    assert_eq!(groups.len(), 3);

    // Group 0: Alice (2 messages)
    assert_eq!(groups[0].user_id, Some(10));
    assert_eq!(groups[0].sender_name, "Alice");
    assert_eq!(groups[0].messages.len(), 2);
    assert_eq!(groups[0].timestamp, 1000);

    // Group 1: Bob (1 message)
    assert_eq!(groups[1].user_id, Some(20));
    assert_eq!(groups[1].sender_name, "Bob");
    assert_eq!(groups[1].messages.len(), 1);
    assert_eq!(groups[1].timestamp, 1150);

    // Group 2: Bob (1 message, new window)
    assert_eq!(groups[2].user_id, Some(20));
    assert_eq!(groups[2].sender_name, "Bob");
    assert_eq!(groups[2].messages.len(), 1);
    assert_eq!(groups[2].timestamp, 1600);

    // Timestamp formatting test
    let formatted = format_message_timestamp(1700000000);
    assert!(!formatted.is_empty());
}

#[test]
fn test_voice_bar_state_indicators() {
    // 1. Connected state
    let connected = VoiceConnectionState::Connected {
        channel_id: 42,
        channel_name: "voice-lounge".to_string(),
        ping_ms: 18,
    };
    assert_eq!(
        voice_status_indicator_color(&connected),
        theme::STATUS_ONLINE
    );
    let status_text = voice_status_text(&connected);
    assert!(status_text.contains("voice-lounge"));
    assert!(status_text.contains("18ms"));

    // 2. Connecting state
    let connecting = VoiceConnectionState::Connecting;
    assert_eq!(
        voice_status_indicator_color(&connecting),
        theme::STATUS_IDLE
    );
    assert_eq!(voice_status_text(&connecting), "Connecting...");

    // 3. Disconnected state
    let disconnected = VoiceConnectionState::Disconnected;
    assert_eq!(
        voice_status_indicator_color(&disconnected),
        theme::STATUS_OFFLINE
    );
    assert_eq!(voice_status_text(&disconnected), "Disconnected");

    // Action button labels (Mute / Deafen toggles)
    assert_eq!(voice_action_labels(false, false), ("Mute", "Deafen"));
    assert_eq!(voice_action_labels(true, false), ("Unmute", "Deafen"));
    assert_eq!(voice_action_labels(false, true), ("Mute", "Undeafen"));
    assert_eq!(voice_action_labels(true, true), ("Unmute", "Undeafen"));

    // VoiceBarState struct
    let mut vb = VoiceBarState::new();
    assert_eq!(vb.connection, VoiceConnectionState::Disconnected);
    assert!(!vb.is_muted);
    assert!(!vb.is_deafened);

    vb.toggle_mute();
    assert!(vb.is_muted);

    vb.toggle_deafen();
    assert!(vb.is_deafened);
}

#[test]
fn test_workspace_view_initialization_and_mutations() {
    let mut state = AppState::new();
    state.current_user = Some(User {
        id: 1,
        name: "SharkordAdmin".to_string(),
        status: Some("online".to_string()),
        ..Default::default()
    });

    let mut workspace = WorkspaceView::new(state);

    // Initial draft is empty
    assert_eq!(workspace.message_draft, "");

    // Draft typing
    workspace.set_message_draft("Hello world from GPUI!");
    assert_eq!(workspace.message_draft, "Hello world from GPUI!");

    // Channel selection via workspace
    workspace.select_channel(101);
    assert_eq!(workspace.state.active_channel_id, Some(101));

    // Send message clears draft and appends to state
    let sent = workspace.send_message();
    assert!(sent.is_some());
    let sent_msg = sent.unwrap();
    assert_eq!(sent_msg.channel_id, 101);
    assert_eq!(sent_msg.content, Some("Hello world from GPUI!".to_string()));
    assert_eq!(sent_msg.user_id, Some(1));
    assert_eq!(workspace.message_draft, "");

    // Voice controls in workspace
    workspace.toggle_voice_mute();
    assert!(workspace.voice_bar.is_muted);
    workspace.toggle_voice_deafen();
    assert!(workspace.voice_bar.is_deafened);
    workspace.disconnect_voice();
    assert_eq!(
        workspace.voice_bar.connection,
        VoiceConnectionState::Disconnected
    );
}
