use sharkord_core::protocol::events::{decode_websocket_frame, EventPayload, EventType, SubscriptionEvent};

#[test]
fn test_decode_reaction_and_channel_events() {
    let raw = r#"{"type":"reactionAdded","data":{"messageId":1,"emoji":"👍","userId":2}}"#;
    let evt = decode_websocket_frame(raw).unwrap().unwrap();
    assert_eq!(evt.event_type, EventType::ReactionAdded);
    assert_eq!(evt.data["emoji"], "👍");
    assert_eq!(evt.data["messageId"], 1);
    assert_eq!(evt.data["userId"], 2);

    let raw_removed = r#"{"type":"reactionRemove","data":{"messageId":1,"emoji":"👍","userId":2}}"#;
    let evt_rem = decode_websocket_frame(raw_removed).unwrap().unwrap();
    assert_eq!(evt_rem.event_type, EventType::ReactionRemoved);

    let raw_colon = r#"{"type":"reaction:added","data":{"messageId":10,"emoji":"🎉"}}"#;
    let evt_colon = decode_websocket_frame(raw_colon).unwrap().unwrap();
    assert_eq!(evt_colon.event_type, EventType::ReactionAdded);

    let raw_colon_rem = r#"{"type":"reaction:removed","data":{"messageId":10,"emoji":"🎉"}}"#;
    let evt_colon_rem = decode_websocket_frame(raw_colon_rem).unwrap().unwrap();
    assert_eq!(evt_colon_rem.event_type, EventType::ReactionRemoved);
}

#[test]
fn test_decode_channel_lifecycle_events() {
    // Channel created (trpc envelope with channelCreate)
    let raw_create = r#"{
        "id": "sub-c1",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "channelCreate",
                    "data": {
                        "id": 10,
                        "name": "general-text",
                        "type": "text"
                    }
                }
            }
        }
    }"#;
    let evt_create = decode_websocket_frame(raw_create).unwrap().unwrap();
    assert_eq!(evt_create.id.as_deref(), Some("sub-c1"));
    assert_eq!(evt_create.event_type, EventType::ChannelCreated);
    assert_eq!(evt_create.data["name"], "general-text");

    // Channel updated (channelUpdated and channelUpdate)
    let raw_update = r#"{
        "id": "sub-c2",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "channelUpdate",
                    "data": {
                        "id": 10,
                        "name": "general-chat"
                    }
                }
            }
        }
    }"#;
    let evt_update = decode_websocket_frame(raw_update).unwrap().unwrap();
    assert_eq!(evt_update.event_type, EventType::ChannelUpdated);
    assert_eq!(evt_update.data["name"], "general-chat");

    let raw_updated = r#"{"type":"channelUpdated","data":{"id":10,"name":"general-chat-2"}}"#;
    let evt_updated = decode_websocket_frame(raw_updated).unwrap().unwrap();
    assert_eq!(evt_updated.event_type, EventType::ChannelUpdated);

    // Channel deleted (channelDelete and channelDeleted)
    let raw_delete = r#"{
        "id": "sub-c3",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "channelDelete",
                    "data": {
                        "id": 10
                    }
                }
            }
        }
    }"#;
    let evt_delete = decode_websocket_frame(raw_delete).unwrap().unwrap();
    assert_eq!(evt_delete.event_type, EventType::ChannelDeleted);
    assert_eq!(evt_delete.data["id"], 10);

    let raw_deleted = r#"{"type":"channelDeleted","data":{"id":10}}"#;
    let evt_deleted = decode_websocket_frame(raw_deleted).unwrap().unwrap();
    assert_eq!(evt_deleted.event_type, EventType::ChannelDeleted);
}

#[test]
fn test_decode_presence_and_typing_events() {
    // User presence changed (presenceUpdate, userPresenceUpdate, userPresenceChanged)
    let raw_pres1 = r#"{"type":"presenceUpdate","data":{"userId":5,"status":"online"}}"#;
    let evt_pres1 = decode_websocket_frame(raw_pres1).unwrap().unwrap();
    assert_eq!(evt_pres1.event_type, EventType::UserPresenceChanged);
    assert_eq!(evt_pres1.data["status"], "online");

    let raw_pres2 = r#"{"type":"userPresenceUpdate","data":{"userId":6,"status":"idle"}}"#;
    let evt_pres2 = decode_websocket_frame(raw_pres2).unwrap().unwrap();
    assert_eq!(evt_pres2.event_type, EventType::UserPresenceChanged);

    let raw_pres3 = r#"{"type":"userPresenceChanged","data":{"userId":7,"status":"dnd"}}"#;
    let evt_pres3 = decode_websocket_frame(raw_pres3).unwrap().unwrap();
    assert_eq!(evt_pres3.event_type, EventType::UserPresenceChanged);

    // User typing (messageTyping, userTyping)
    let raw_typing1 = r#"{
        "id": "sub-t1",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "messageTyping",
                    "data": {
                        "channelId": 1,
                        "userId": 3
                    }
                }
            }
        }
    }"#;
    let evt_typing1 = decode_websocket_frame(raw_typing1).unwrap().unwrap();
    assert_eq!(evt_typing1.event_type, EventType::UserTyping);
    assert_eq!(evt_typing1.data["userId"], 3);

    let raw_typing2 = r#"{"type":"userTyping","data":{"channelId":1,"userId":4}}"#;
    let evt_typing2 = decode_websocket_frame(raw_typing2).unwrap().unwrap();
    assert_eq!(evt_typing2.event_type, EventType::UserTyping);
}

#[test]
fn test_event_type_string_and_serde_roundtrip() {
    let variants = vec![
        (EventType::ReactionAdded, "reactionAdded"),
        (EventType::ReactionRemoved, "reactionRemoved"),
        (EventType::ChannelCreated, "channelCreated"),
        (EventType::ChannelUpdated, "channelUpdated"),
        (EventType::ChannelDeleted, "channelDeleted"),
        (EventType::UserPresenceChanged, "userPresenceChanged"),
        (EventType::UserTyping, "userTyping"),
    ];

    for (variant, canonical) in variants {
        assert_eq!(variant.as_str(), canonical);
        let serialized = serde_json::to_string(&variant).unwrap();
        assert_eq!(serialized, format!("\"{}\"", canonical));
        let deserialized: EventType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, variant);
        assert_eq!(EventType::from(canonical), variant);
    }
}

#[test]
fn test_event_payload_and_subscription_event_envelope() {
    let payload = EventPayload::new(
        EventType::ReactionAdded,
        serde_json::json!({
            "messageId": 100,
            "emoji": "🚀"
        }),
    );
    let payload_str = serde_json::to_string(&payload).unwrap();
    let deserialized_payload: EventPayload = serde_json::from_str(&payload_str).unwrap();
    assert_eq!(deserialized_payload.event_type, EventType::ReactionAdded);
    assert_eq!(deserialized_payload.data["emoji"], "🚀");

    let sub_event = SubscriptionEvent::new(
        Some("sub-99".to_string()),
        EventType::UserTyping,
        serde_json::json!({ "userId": 42 }),
    );
    assert_eq!(sub_event.event_type, EventType::UserTyping);
    assert_eq!(sub_event.payload()["userId"], 42);
}
