use sharkord_mobile::{
    authenticate, fetch_channels, fetch_messages, init_mobile_client, send_message,
    toggle_voice_mute, MobileChannel, MobileMessage, MobileUser, MobileVoiceState,
};

#[test]
fn test_init_mobile_client() {
    // Empty URL should fail
    let res = init_mobile_client("");
    assert!(res.is_err());

    // Invalid scheme should fail
    let res = init_mobile_client("ftp://example.com");
    assert!(res.is_err());

    // Valid HTTP URL should succeed
    let res = init_mobile_client("http://localhost:3000");
    assert!(res.is_ok());
}

#[test]
fn test_authenticate() {
    init_mobile_client("http://localhost:3000").expect("Failed to init client");

    // Empty token should fail
    let res = authenticate("");
    assert!(res.is_err());

    // Valid token should succeed and return MobileUser
    let user = authenticate("session_token_xyz_123").expect("Auth should succeed");
    assert!(!user.username.is_empty());
    assert!(!user.display_name.is_empty());
    assert!(user.is_online);
}

#[test]
fn test_fetch_channels() {
    init_mobile_client("http://localhost:3000").expect("Failed to init client");
    authenticate("session_token_xyz_123").expect("Auth should succeed");

    let channels = fetch_channels().expect("Failed to fetch channels");
    assert!(!channels.is_empty());

    // Ensure we have at least text and voice channel
    let has_text = channels.iter().any(|c| !c.is_voice);
    let has_voice = channels.iter().any(|c| c.is_voice);
    assert!(has_text, "Should contain at least one text channel");
    assert!(has_voice, "Should contain at least one voice channel");
}

#[test]
fn test_send_and_fetch_messages_with_serialization() {
    init_mobile_client("http://localhost:3000").expect("Failed to init client");
    authenticate("session_token_xyz_123").expect("Auth should succeed");

    let channel_id = 1u64;

    // Empty content should fail
    let res = send_message(channel_id, "   ");
    assert!(res.is_err());

    // Valid message should succeed
    let msg =
        send_message(channel_id, "Hello from mobile bridge test!").expect("Failed to send message");
    assert_eq!(msg.channel_id, channel_id);
    assert_eq!(msg.content, "Hello from mobile bridge test!");
    assert_eq!(msg.author.username, "mobile_user");

    // Fetch messages should include the newly sent message
    let messages = fetch_messages(channel_id).expect("Failed to fetch messages");
    assert!(!messages.is_empty());
    assert!(messages
        .iter()
        .any(|m| m.content == "Hello from mobile bridge test!"));

    // Message JSON serialization and deserialization
    let json_str = serde_json::to_string(&msg).expect("Failed to serialize MobileMessage");
    assert!(json_str.contains("Hello from mobile bridge test!"));
    assert!(json_str.contains("channelId"));

    let deserialized: MobileMessage =
        serde_json::from_str(&json_str).expect("Failed to deserialize MobileMessage");
    assert_eq!(deserialized.id, msg.id);
    assert_eq!(deserialized.content, msg.content);
}

#[test]
fn test_voice_mute_toggle() {
    init_mobile_client("http://localhost:3000").expect("Failed to init client");

    let muted = toggle_voice_mute(true).expect("Failed to mute voice");
    assert!(muted);

    let unmuted = toggle_voice_mute(false).expect("Failed to unmute voice");
    assert!(!unmuted);
}

#[test]
fn test_core_type_conversions() {
    let core_user = sharkord_core::User {
        id: 42,
        name: "test_core_user".to_string(),
        identity: Some("Core Tester".to_string()),
        profile_color: None,
        bio: None,
        avatar_id: Some(99),
        banner_id: None,
        banned: Some(false),
        created_at: Some(1700000000),
        role_ids: vec![1, 2],
        status: Some("online".to_string()),
        session_count: None,
    };

    let mobile_user: MobileUser = core_user.into();
    assert_eq!(mobile_user.id, 42);
    assert_eq!(mobile_user.username, "test_core_user");
    assert_eq!(mobile_user.display_name, "Core Tester");
    assert_eq!(mobile_user.avatar_url, Some("/api/avatars/99".to_string()));
    assert!(mobile_user.is_online);

    let core_channel = sharkord_core::Channel {
        id: 10,
        name: "Voice Lounge".to_string(),
        channel_type: sharkord_core::ChannelType::Voice,
        topic: Some("Chat here".to_string()),
        private: false,
        is_dm: false,
        position: 1,
        category_id: Some(5),
        created_at: Some(1700000000),
        updated_at: None,
    };

    let mobile_channel: MobileChannel = core_channel.into();
    assert_eq!(mobile_channel.id, 10);
    assert_eq!(mobile_channel.name, "Voice Lounge");
    assert!(mobile_channel.is_voice);
    assert_eq!(mobile_channel.category, Some("Category 5".to_string()));

    let core_msg = sharkord_core::Message {
        id: 777,
        channel_id: 10,
        user_id: Some(42),
        content: Some("Core msg test".to_string()),
        plugin_id: None,
        parent_message_id: None,
        reply_to_message_id: None,
        pinned: false,
        pinned_at: None,
        pinned_by: None,
        editable: None,
        edited_at: None,
        edited_by: None,
        created_at: Some(1700000050),
        updated_at: None,
        components: Vec::new(),
        reactions: Vec::new(),
    };

    let mobile_msg: MobileMessage = core_msg.into();
    assert_eq!(mobile_msg.id, 777);
    assert_eq!(mobile_msg.channel_id, 10);
    assert_eq!(mobile_msg.content, "Core msg test");
    assert_eq!(mobile_msg.timestamp, 1700000050);

    // Verify MobileVoiceState serialization
    let voice_state = MobileVoiceState {
        channel_id: Some(10),
        channel_name: Some("Voice Lounge".to_string()),
        is_connected: true,
        is_muted: false,
        is_deafened: false,
        ping_ms: 35,
        participants: vec![mobile_user],
    };
    let voice_json = serde_json::to_string(&voice_state).expect("Voice state serialization failed");
    assert!(voice_json.contains("Voice Lounge"));
    let deserialized_voice: MobileVoiceState =
        serde_json::from_str(&voice_json).expect("Voice state deserialization failed");
    assert_eq!(deserialized_voice.channel_id, Some(10));
}
