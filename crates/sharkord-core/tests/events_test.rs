use serde_json::json;
use sharkord_core::{
    decode_websocket_frame, encode_subscription_request, EventType, SharkordClient,
    SubscriptionEvent,
};
use tokio::sync::mpsc;

#[test]
fn test_encode_subscription_request() {
    // With input
    let input = json!({ "channelId": 42 });
    let encoded = encode_subscription_request("1", "messages.events", Some(&input));
    let parsed: serde_json::Value = serde_json::from_str(&encoded).unwrap();

    assert_eq!(parsed["id"], "1");
    assert_eq!(parsed["method"], "subscription");
    assert_eq!(parsed["params"]["path"], "messages.events");
    assert_eq!(parsed["params"]["input"]["channelId"], 42);

    // Without input
    let encoded_none = encode_subscription_request("2", "users.presence", None);
    let parsed_none: serde_json::Value = serde_json::from_str(&encoded_none).unwrap();

    assert_eq!(parsed_none["id"], "2");
    assert_eq!(parsed_none["method"], "subscription");
    assert_eq!(parsed_none["params"]["path"], "users.presence");
    assert!(parsed_none["params"].get("input").is_none());
}

#[test]
fn test_decode_websocket_message_frames() {
    // 1. messageCreated
    let created_frame = r#"{
        "id": "1",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "messageCreated",
                    "data": {
                        "id": 101,
                        "channelId": 1,
                        "content": "Hello tRPC WebSocket!"
                    }
                }
            }
        }
    }"#;
    let event = decode_websocket_frame(created_frame)
        .expect("decode failed")
        .expect("missing event");
    assert_eq!(event.id.as_deref(), Some("1"));
    assert_eq!(event.event_type, EventType::MessageCreated);
    assert_eq!(event.data["id"], 101);
    assert_eq!(event.data["content"], "Hello tRPC WebSocket!");

    // 2. messageUpdated
    let updated_frame = r#"{
        "id": "2",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "messageUpdated",
                    "data": {
                        "id": 101,
                        "channelId": 1,
                        "content": "Updated content!"
                    }
                }
            }
        }
    }"#;
    let event = decode_websocket_frame(updated_frame)
        .expect("decode failed")
        .expect("missing event");
    assert_eq!(event.id.as_deref(), Some("2"));
    assert_eq!(event.event_type, EventType::MessageUpdated);
    assert_eq!(event.data["content"], "Updated content!");

    // 3. messageDeleted
    let deleted_frame = r#"{
        "id": "3",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "messageDeleted",
                    "data": {
                        "id": 101,
                        "channelId": 1
                    }
                }
            }
        }
    }"#;
    let event = decode_websocket_frame(deleted_frame)
        .expect("decode failed")
        .expect("missing event");
    assert_eq!(event.id.as_deref(), Some("3"));
    assert_eq!(event.event_type, EventType::MessageDeleted);
    assert_eq!(event.data["id"], 101);

    // 4. userJoined
    let user_joined_frame = r#"{
        "id": "4",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "userJoined",
                    "data": {
                        "userId": 5,
                        "name": "Alice"
                    }
                }
            }
        }
    }"#;
    let event = decode_websocket_frame(user_joined_frame)
        .expect("decode failed")
        .expect("missing event");
    assert_eq!(event.id.as_deref(), Some("4"));
    assert_eq!(event.event_type, EventType::UserJoined);
    assert_eq!(event.data["name"], "Alice");

    // 5. voiceStateUpdated
    let voice_frame = r#"{
        "id": "5",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "voiceStateUpdated",
                    "data": {
                        "userId": 5,
                        "channelId": 2,
                        "muted": false
                    }
                }
            }
        }
    }"#;
    let event = decode_websocket_frame(voice_frame)
        .expect("decode failed")
        .expect("missing event");
    assert_eq!(event.id.as_deref(), Some("5"));
    assert_eq!(event.event_type, EventType::VoiceStateUpdated);
    assert_eq!(event.data["muted"], false);

    // 6. userLeft
    let user_left_frame = r#"{
        "id": "6",
        "result": {
            "type": "data",
            "data": {
                "json": {
                    "type": "userLeft",
                    "data": {
                        "userId": 5
                    }
                }
            }
        }
    }"#;
    let event = decode_websocket_frame(user_left_frame)
        .expect("decode failed")
        .expect("missing event");
    assert_eq!(event.id.as_deref(), Some("6"));
    assert_eq!(event.event_type, EventType::UserLeft);

    // 7. Lifecycle frame (started) should return Ok(None)
    let started_frame = r#"{"id": "1", "result": {"type": "started"}}"#;
    let none_event = decode_websocket_frame(started_frame).expect("decode failed");
    assert!(none_event.is_none());

    // 8. Error frame should return Err(RpcError)
    let error_frame =
        r#"{"id": "1", "error": {"json": {"message": "Unauthorized", "code": -32600}}}"#;
    let err = decode_websocket_frame(error_frame).unwrap_err();
    assert!(err.to_string().contains("Unauthorized"));
}

#[tokio::test]
async fn test_dispatch_events_to_unbounded_mpsc_channel() {
    let (tx, mut rx) = mpsc::unbounded_channel::<SubscriptionEvent>();

    let frames = vec![
        r#"{"id":"1","result":{"type":"data","data":{"json":{"type":"messageCreated","data":{"id":1,"content":"first"}}}}}"#,
        r#"{"id":"1","result":{"type":"started"}}"#,
        r#"{"id":"2","result":{"type":"data","data":{"json":{"type":"voiceStateUpdated","data":{"userId":10,"muted":true}}}}}"#,
        r#"{"id":"3","result":{"type":"data","data":{"json":{"type":"messageDeleted","data":{"id":1}}}}}"#,
    ];

    for frame in frames {
        if let Some(event) = decode_websocket_frame(frame).unwrap() {
            tx.send(event).unwrap();
        }
    }

    // Verify first event
    let event1 = rx.recv().await.expect("event 1");
    assert_eq!(event1.event_type, EventType::MessageCreated);
    assert_eq!(event1.data["content"], "first");

    // Verify second event
    let event2 = rx.recv().await.expect("event 2");
    assert_eq!(event2.event_type, EventType::VoiceStateUpdated);
    assert_eq!(event2.data["userId"], 10);

    // Verify third event
    let event3 = rx.recv().await.expect("event 3");
    assert_eq!(event3.event_type, EventType::MessageDeleted);
    assert_eq!(event3.data["id"], 1);
}

#[tokio::test]
async fn test_client_subscribe_events_integration() {
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::Message;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn mock server
    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            if let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await {
                let frame = r#"{
                    "id": "1",
                    "result": {
                        "type": "data",
                        "data": {
                            "json": {
                                "type": "messageCreated",
                                "data": {
                                    "id": 999,
                                    "channelId": 1,
                                    "content": "From mock server"
                                }
                            }
                        }
                    }
                }"#;
                let _ = ws.send(Message::Text(frame.to_string().into())).await;
                while ws.next().await.is_some() {}
            }
        }
    });

    let client = SharkordClient::new(&format!("http://127.0.0.1:{}", addr.port())).unwrap();
    client.set_session_token(Some("mock-token-xyz".to_string()));

    let mut event_rx = client
        .subscribe_events()
        .await
        .expect("subscribe_events failed");

    let received = tokio::time::timeout(std::time::Duration::from_secs(3), event_rx.recv())
        .await
        .expect("timed out waiting for event")
        .expect("stream ended");

    assert_eq!(received.event_type, EventType::MessageCreated);
    assert_eq!(received.data["id"], 999);
    assert_eq!(received.data["content"], "From mock server");
}
