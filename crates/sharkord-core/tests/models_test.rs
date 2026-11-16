use sharkord_core::{Capabilities, Channel, ChannelType, Message, Role, ServerSettings, User};

#[test]
fn test_user_serialization_and_deserialization() {
    let json_data = r##"{
        "id": 1,
        "name": "Admin",
        "identity": "admin@example.com",
        "profileColor": "#ff0000",
        "bio": "Server owner",
        "avatarId": null,
        "bannerId": null,
        "banned": false,
        "createdAt": 1700000000000,
        "roleIds": [1, 2],
        "status": "online"
    }"##;

    let user: User = serde_json::from_str(json_data).expect("Failed to deserialize user");
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Admin");
    assert_eq!(user.identity.as_deref(), Some("admin@example.com"));
    assert_eq!(user.profile_color.as_deref(), Some("#ff0000"));
    assert_eq!(user.bio.as_deref(), Some("Server owner"));
    assert_eq!(user.avatar_id, None);
    assert_eq!(user.banner_id, None);
    assert_eq!(user.banned, Some(false));
    assert_eq!(user.role_ids, vec![1, 2]);
    assert_eq!(user.status.as_deref(), Some("online"));

    let serialized = serde_json::to_string(&user).expect("Failed to serialize user");
    assert!(serialized.contains(r#""profileColor":"#));
    assert!(serialized.contains(r#""roleIds":"#));

    let roundtrip: User = serde_json::from_str(&serialized).expect("Failed to roundtrip user");
    assert_eq!(user, roundtrip);
}

#[test]
fn test_channel_serialization_and_deserialization() {
    let text_json = r##"{
        "id": 1,
        "name": "general",
        "type": "TEXT",
        "topic": "General chat",
        "private": false,
        "isDm": false,
        "position": 0,
        "categoryId": 10,
        "createdAt": 1700000000000,
        "updatedAt": null
    }"##;

    let channel: Channel =
        serde_json::from_str(text_json).expect("Failed to deserialize text channel");
    assert_eq!(channel.id, 1);
    assert_eq!(channel.name, "general");
    assert_eq!(channel.channel_type, ChannelType::Text);
    assert_eq!(channel.topic.as_deref(), Some("General chat"));
    assert!(!channel.private);
    assert!(!channel.is_dm);
    assert_eq!(channel.position, 0);
    assert_eq!(channel.category_id, Some(10));

    let voice_json = r##"{
        "id": 2,
        "name": "Voice Lounge",
        "type": "VOICE",
        "position": 1
    }"##;

    let voice_channel: Channel = serde_json::from_str(voice_json)
        .expect("Failed to deserialize voice channel with defaults");
    assert_eq!(voice_channel.id, 2);
    assert_eq!(voice_channel.channel_type, ChannelType::Voice);
    assert!(!voice_channel.private);
    assert_eq!(voice_channel.topic, None);

    let serialized = serde_json::to_string(&channel).expect("Failed to serialize channel");
    assert!(serialized.contains(r#""type":"TEXT""#));
    assert!(serialized.contains(r#""categoryId":10"#));

    let roundtrip: Channel =
        serde_json::from_str(&serialized).expect("Failed to roundtrip channel");
    assert_eq!(channel, roundtrip);
}

#[test]
fn test_message_serialization_and_deserialization() {
    let json_data = r##"{
        "id": 42,
        "channelId": 1,
        "userId": 1,
        "content": "Hello, world!",
        "pinned": false,
        "createdAt": 1700000005000,
        "updatedAt": null
    }"##;

    let message: Message = serde_json::from_str(json_data).expect("Failed to deserialize message");
    assert_eq!(message.id, 42);
    assert_eq!(message.channel_id, 1);
    assert_eq!(message.user_id, Some(1));
    assert_eq!(message.content.as_deref(), Some("Hello, world!"));
    assert!(!message.pinned);
    assert_eq!(message.created_at, Some(1700000005000));

    let serialized = serde_json::to_string(&message).expect("Failed to serialize message");
    assert!(serialized.contains(r#""channelId":1"#));
    assert!(serialized.contains(r#""userId":1"#));

    let roundtrip: Message =
        serde_json::from_str(&serialized).expect("Failed to roundtrip message");
    assert_eq!(message, roundtrip);
}

#[test]
fn test_role_serialization_and_deserialization() {
    let json_data = r##"{
        "id": 1,
        "name": "Admin",
        "color": "#3b82f6",
        "isPersistent": true,
        "isDefault": false,
        "storageQuotaOverrideEnabled": false,
        "storageSpaceQuota": 0,
        "permissions": ["MANAGE_SERVER", "MANAGE_CHANNELS"],
        "createdAt": 1700000000000,
        "updatedAt": null
    }"##;

    let role: Role = serde_json::from_str(json_data).expect("Failed to deserialize role");
    assert_eq!(role.id, 1);
    assert_eq!(role.name, "Admin");
    assert_eq!(role.color, "#3b82f6");
    assert!(role.is_persistent);
    assert!(!role.is_default);
    assert_eq!(role.permissions, vec!["MANAGE_SERVER", "MANAGE_CHANNELS"]);

    let serialized = serde_json::to_string(&role).expect("Failed to serialize role");
    assert!(serialized.contains(r#""isPersistent":true"#));
    assert!(serialized.contains(r#""isDefault":false"#));

    let roundtrip: Role = serde_json::from_str(&serialized).expect("Failed to roundtrip role");
    assert_eq!(role, roundtrip);
}

#[test]
fn test_server_settings_serialization_and_deserialization() {
    let json_data = r##"{
        "serverId": "sharkord-main",
        "name": "Main Server",
        "description": "Primary community server",
        "allowNewUsers": true,
        "directMessagesEnabled": true,
        "storageUploadEnabled": true,
        "storageQuota": 10737418240,
        "storageUploadMaxFileSize": 104857600,
        "enablePlugins": true,
        "enableSearch": true,
        "showWelcomeDialog": true
    }"##;

    let settings: ServerSettings =
        serde_json::from_str(json_data).expect("Failed to deserialize server settings");
    assert_eq!(settings.server_id, "sharkord-main");
    assert_eq!(settings.name, "Main Server");
    assert_eq!(
        settings.description.as_deref(),
        Some("Primary community server")
    );
    assert_eq!(settings.allow_new_users, Some(true));
    assert_eq!(settings.storage_quota, Some(10737418240));

    let serialized = serde_json::to_string(&settings).expect("Failed to serialize settings");
    assert!(serialized.contains(r#""serverId":"sharkord-main""#));
    assert!(serialized.contains(r#""allowNewUsers":true"#));

    let roundtrip: ServerSettings =
        serde_json::from_str(&serialized).expect("Failed to roundtrip settings");
    assert_eq!(settings, roundtrip);
}

#[test]
fn test_capabilities_serialization_and_deserialization() {
    let json_data = r##"{
        "isPlus": true,
        "highProfileCodecs": true,
        "isolatedWindowAudio": true,
        "serverVersion": "1.0.0-plus"
    }"##;

    let caps: Capabilities =
        serde_json::from_str(json_data).expect("Failed to deserialize capabilities");
    assert!(caps.is_plus);
    assert!(caps.high_profile_codecs);
    assert!(caps.isolated_window_audio);
    assert_eq!(caps.server_version.as_deref(), Some("1.0.0-plus"));

    let serialized = serde_json::to_string(&caps).expect("Failed to serialize capabilities");
    assert!(serialized.contains(r#""isPlus":true"#));
    assert!(serialized.contains(r#""highProfileCodecs":true"#));

    let roundtrip: Capabilities =
        serde_json::from_str(&serialized).expect("Failed to roundtrip capabilities");
    assert_eq!(caps, roundtrip);
}
