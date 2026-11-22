use desktop_gpui::state::AppState;
use sharkord_core::protocol::events::{EventType, SubscriptionEvent};
use sharkord_core::types::{Channel, ChannelType, Message, User};

#[test]
fn test_dispatch_message_created_event() {
    let mut state = AppState::new();
    state.active_channel_id = Some(1);

    // Message created for active channel
    let event = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageCreated,
        data: serde_json::json!({
            "id": 42,
            "channelId": 1,
            "content": "Live WebSocket message",
            "userId": 2,
            "createdAt": 1000
        }),
    };
    state.handle_subscription_event(event);

    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].id, 42);
    assert_eq!(
        state.messages[0].content.as_deref(),
        Some("Live WebSocket message")
    );
    assert_eq!(state.messages[0].user_id, Some(2));

    // Message created for a different channel should not pollute active channel messages
    let other_channel_event = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageCreated,
        data: serde_json::json!({
            "id": 43,
            "channelId": 99,
            "content": "Message for other channel",
            "userId": 3,
            "createdAt": 1005
        }),
    };
    state.handle_subscription_event(other_channel_event);

    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].id, 42);
}

#[test]
fn test_dispatch_message_updated_event() {
    let mut state = AppState::new();
    state.active_channel_id = Some(1);
    state.add_message(Message {
        id: 42,
        channel_id: 1,
        content: Some("Original content".to_string()),
        pinned: false,
        ..Default::default()
    });

    let update_event = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageUpdated,
        data: serde_json::json!({
            "id": 42,
            "channelId": 1,
            "content": "Updated content via WebSocket",
            "pinned": true,
            "editedAt": 1050
        }),
    };
    state.handle_subscription_event(update_event);

    assert_eq!(state.messages.len(), 1);
    assert_eq!(
        state.messages[0].content.as_deref(),
        Some("Updated content via WebSocket")
    );
    assert!(state.messages[0].pinned);
    assert_eq!(state.messages[0].edited_at, Some(1050));
}

#[test]
fn test_dispatch_message_deleted_event() {
    let mut state = AppState::new();
    state.active_channel_id = Some(1);
    state.add_message(Message {
        id: 41,
        channel_id: 1,
        content: Some("First message".to_string()),
        ..Default::default()
    });
    state.add_message(Message {
        id: 42,
        channel_id: 1,
        content: Some("To be deleted".to_string()),
        ..Default::default()
    });

    let delete_event = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageDeleted,
        data: serde_json::json!({
            "id": 42,
            "channelId": 1
        }),
    };
    state.handle_subscription_event(delete_event);

    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].id, 41);
}

#[test]
fn test_dispatch_reaction_added_and_removed_events() {
    let mut state = AppState::new();
    state.active_channel_id = Some(1);
    state.add_message(Message {
        id: 42,
        channel_id: 1,
        content: Some("Message for reactions".to_string()),
        ..Default::default()
    });

    // Add reaction
    let add_reaction_event = SubscriptionEvent {
        id: None,
        event_type: EventType::ReactionAdded,
        data: serde_json::json!({
            "messageId": 42,
            "emoji": "🚀",
            "userId": 10
        }),
    };
    state.handle_subscription_event(add_reaction_event);

    assert_eq!(state.messages[0].reactions.len(), 1);
    assert_eq!(state.messages[0].reactions[0].emoji, "🚀");
    assert_eq!(state.messages[0].reactions[0].user_id, 10);

    // Remove reaction
    let remove_reaction_event = SubscriptionEvent {
        id: None,
        event_type: EventType::ReactionRemoved,
        data: serde_json::json!({
            "messageId": 42,
            "emoji": "🚀",
            "userId": 10
        }),
    };
    state.handle_subscription_event(remove_reaction_event);

    assert!(state.messages[0].reactions.is_empty());
}

#[test]
fn test_dispatch_channel_lifecycle_events() {
    let mut state = AppState::new();
    state.set_channels(vec![Channel {
        id: 1,
        name: "general".to_string(),
        channel_type: ChannelType::Text,
        ..Default::default()
    }]);

    // Channel Created
    let create_event = SubscriptionEvent {
        id: None,
        event_type: EventType::ChannelCreated,
        data: serde_json::json!({
            "id": 2,
            "name": "dev-announcements",
            "type": "TEXT"
        }),
    };
    state.handle_subscription_event(create_event);

    assert_eq!(state.channels.len(), 2);
    assert!(state.channels.iter().any(|c| c.id == 2 && c.name == "dev-announcements"));

    // Channel Updated
    let update_event = SubscriptionEvent {
        id: None,
        event_type: EventType::ChannelUpdated,
        data: serde_json::json!({
            "id": 2,
            "name": "developer-chat"
        }),
    };
    state.handle_subscription_event(update_event);

    let updated_channel = state.channels.iter().find(|c| c.id == 2).unwrap();
    assert_eq!(updated_channel.name, "developer-chat");

    // Channel Deleted
    let delete_event = SubscriptionEvent {
        id: None,
        event_type: EventType::ChannelDeleted,
        data: serde_json::json!({
            "id": 2
        }),
    };
    state.handle_subscription_event(delete_event);

    assert_eq!(state.channels.len(), 1);
    assert_eq!(state.channels[0].id, 1);
}

#[test]
fn test_dispatch_user_presence_changed_event() {
    let mut state = AppState::new();
    state.add_user(User {
        id: 5,
        name: "Alice".to_string(),
        status: Some("offline".to_string()),
        ..Default::default()
    });
    state.current_user = Some(User {
        id: 5,
        name: "Alice".to_string(),
        status: Some("offline".to_string()),
        ..Default::default()
    });

    let presence_event = SubscriptionEvent {
        id: None,
        event_type: EventType::UserPresenceChanged,
        data: serde_json::json!({
            "userId": 5,
            "status": "online"
        }),
    };
    state.handle_subscription_event(presence_event);

    let user = state.user_by_id(5).expect("user 5 should exist");
    assert_eq!(user.status.as_deref(), Some("online"));
    assert_eq!(
        state.current_user.as_ref().and_then(|u| u.status.as_deref()),
        Some("online")
    );
}

#[test]
fn test_dispatch_user_typing_event() {
    let mut state = AppState::new();
    state.active_channel_id = Some(1);

    // User starts typing
    let typing_start = SubscriptionEvent {
        id: None,
        event_type: EventType::UserTyping,
        data: serde_json::json!({
            "channelId": 1,
            "userId": 7,
            "isTyping": true
        }),
    };
    state.handle_subscription_event(typing_start);

    assert!(state.is_user_typing(1, 7));

    // User stops typing explicitly
    let typing_stop = SubscriptionEvent {
        id: None,
        event_type: EventType::UserTyping,
        data: serde_json::json!({
            "channelId": 1,
            "userId": 7,
            "isTyping": false
        }),
    };
    state.handle_subscription_event(typing_stop);

    assert!(!state.is_user_typing(1, 7));

    // User typing cleared automatically on MessageCreated
    let typing_again = SubscriptionEvent {
        id: None,
        event_type: EventType::UserTyping,
        data: serde_json::json!({
            "channelId": 1,
            "userId": 7
        }),
    };
    state.handle_subscription_event(typing_again);
    assert!(state.is_user_typing(1, 7));

    let message_sent = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageCreated,
        data: serde_json::json!({
            "id": 99,
            "channelId": 1,
            "content": "Done typing!",
            "userId": 7
        }),
    };
    state.handle_subscription_event(message_sent);
    assert!(!state.is_user_typing(1, 7));
}

#[test]
fn test_network_bridge_dispatch_event_helper() {
    use std::sync::{Arc, RwLock};

    let state = Arc::new(RwLock::new(AppState::new()));
    {
        let mut s = state.write().unwrap();
        s.active_channel_id = Some(1);
    }

    let event = SubscriptionEvent {
        id: None,
        event_type: EventType::MessageCreated,
        data: serde_json::json!({
            "id": 1001,
            "channelId": 1,
            "content": "Message via NetworkBridge::dispatch_event",
            "userId": 1,
            "createdAt": 2000
        }),
    };

    desktop_gpui::network::NetworkBridge::dispatch_event(&state, event);

    let read_state = state.read().unwrap();
    assert_eq!(read_state.messages.len(), 1);
    assert_eq!(read_state.messages[0].id, 1001);
    assert_eq!(
        read_state.messages[0].content.as_deref(),
        Some("Message via NetworkBridge::dispatch_event")
    );
}
