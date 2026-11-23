use sharkord_core::client::SharkordClient;
use sharkord_core::types::{Channel, ChannelType, Message, User};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_client_toggle_reaction() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": null
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/messages.toggleReaction"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.toggle_reaction(101, "👍").await;
    assert!(res.is_ok(), "toggle_reaction failed: {:?}", res);
}

#[tokio::test]
async fn test_client_edit_message() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let edited_msg = Message {
        id: 102,
        channel_id: 1,
        content: Some("edited content".to_string()),
        ..Default::default()
    };

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": edited_msg
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/messages.edit"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.edit_message(102, "edited content").await;
    assert!(res.is_ok(), "edit_message failed: {:?}", res);
    let msg = res.unwrap();
    assert_eq!(msg.id, 102);
    assert_eq!(msg.content.as_deref(), Some("edited content"));
}

#[tokio::test]
async fn test_client_delete_message() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": null
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/messages.delete"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.delete_message(103).await;
    assert!(res.is_ok(), "delete_message failed: {:?}", res);
}

#[tokio::test]
async fn test_client_pin_message() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": null
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/messages.togglePin"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.pin_message(104, true).await;
    assert!(res.is_ok(), "pin_message failed: {:?}", res);
}

#[tokio::test]
async fn test_client_create_channel() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let created_channel = Channel {
        id: 201,
        name: "dev-room".to_string(),
        channel_type: ChannelType::Text,
        category_id: Some(5),
        ..Default::default()
    };

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": created_channel
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/channels.add"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client
        .create_channel("dev-room", ChannelType::Text, Some(5))
        .await;
    assert!(res.is_ok(), "create_channel failed: {:?}", res);
    let ch = res.unwrap();
    assert_eq!(ch.id, 201);
    assert_eq!(ch.name, "dev-room");
    assert_eq!(ch.category_id, Some(5));
}

#[tokio::test]
async fn test_client_update_channel() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let updated_channel = Channel {
        id: 201,
        name: "dev-chat".to_string(),
        topic: Some("Discussions".to_string()),
        channel_type: ChannelType::Text,
        ..Default::default()
    };

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": updated_channel
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/channels.update"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client
        .update_channel(201, Some("dev-chat"), Some("Discussions"))
        .await;
    assert!(res.is_ok(), "update_channel failed: {:?}", res);
    let ch = res.unwrap();
    assert_eq!(ch.id, 201);
    assert_eq!(ch.name, "dev-chat");
    assert_eq!(ch.topic.as_deref(), Some("Discussions"));
}

#[tokio::test]
async fn test_client_delete_channel() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": null
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/channels.delete"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.delete_channel(201).await;
    assert!(res.is_ok(), "delete_channel failed: {:?}", res);
}

#[tokio::test]
async fn test_client_get_dms() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let dms = vec![Channel {
        id: 301,
        name: "DM - alice".to_string(),
        channel_type: ChannelType::Voice,
        is_dm: true,
        ..Default::default()
    }];

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": dms
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/channels.getDms"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.get_dms().await;
    assert!(res.is_ok(), "get_dms failed: {:?}", res);
    let channels = res.unwrap();
    assert_eq!(channels.len(), 1);
    assert!(channels[0].is_dm);
    assert_eq!(channels[0].id, 301);
}

#[tokio::test]
async fn test_client_open_dm() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let dm_channel = Channel {
        id: 302,
        name: "DM - 42".to_string(),
        is_dm: true,
        ..Default::default()
    };

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": dm_channel
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/dms.open"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.open_dm(42).await;
    assert!(res.is_ok(), "open_dm failed: {:?}", res);
    let ch = res.unwrap();
    assert_eq!(ch.id, 302);
    assert!(ch.is_dm);
}

#[tokio::test]
async fn test_client_get_members() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let users = vec![
        User {
            id: 1,
            name: "admin".to_string(),
            ..Default::default()
        },
        User {
            id: 2,
            name: "member".to_string(),
            ..Default::default()
        },
    ];

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": users
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/users.getAll"))
        .and(header("authorization", "Bearer test-token-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.get_members().await;
    assert!(res.is_ok(), "get_members failed: {:?}", res);
    let members = res.unwrap();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].name, "admin");
    assert_eq!(members[1].name, "member");
}

#[tokio::test]
async fn test_client_upload_file() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    let upload_resp = serde_json::json!({
        "id": 999,
        "originalName": "attachment.png",
        "size": 4
    });

    Mock::given(method("POST"))
        .and(path("/upload"))
        .and(header("x-sharkord-token", "test-token-123"))
        .and(header("x-sharkord-original-name", "attachment.png"))
        .and(header("x-sharkord-content-length", "4"))
        .respond_with(ResponseTemplate::new(200).set_body_json(upload_resp))
        .mount(&mock_server)
        .await;

    let res = client.upload_file("attachment.png", vec![1, 2, 3, 4]).await;
    assert!(res.is_ok(), "upload_file failed: {:?}", res);
    assert_eq!(res.unwrap(), 999);
}

#[tokio::test]
async fn test_client_edit_message_fallback_to_edit_message() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    Mock::given(method("POST"))
        .and(path("/trpc/messages.edit"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "id": 105,
                        "content": "fallback edited"
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/messages.editMessage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.edit_message(105, "fallback edited").await;
    assert!(res.is_ok(), "fallback edit failed: {:?}", res);
    let msg = res.unwrap();
    assert_eq!(msg.id, 105);
    assert_eq!(msg.content.as_deref(), Some("fallback edited"));
}

#[tokio::test]
async fn test_client_create_channel_fallback_to_create_channel() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    Mock::given(method("POST"))
        .and(path("/trpc/channels.add"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let created_channel = Channel {
        id: 202,
        name: "fallback-channel".to_string(),
        channel_type: ChannelType::Text,
        ..Default::default()
    };

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": created_channel
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/channels.createChannel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client
        .create_channel("fallback-channel", ChannelType::Text, None)
        .await;
    assert!(res.is_ok(), "fallback create channel failed: {:?}", res);
    let ch = res.unwrap();
    assert_eq!(ch.id, 202);
    assert_eq!(ch.name, "fallback-channel");
}

#[tokio::test]
async fn test_client_open_dm_fallback_to_channels_open_dm() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    Mock::given(method("POST"))
        .and(path("/trpc/dms.open"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "channelId": 303
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/channels.openDm"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.open_dm(88).await;
    assert!(res.is_ok(), "fallback open dm failed: {:?}", res);
    let dm = res.unwrap();
    assert_eq!(dm.id, 303);
    assert!(dm.is_dm);
}

#[tokio::test]
async fn test_client_get_dms_fallback_to_channels() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    Mock::given(method("GET"))
        .and(path("/trpc/channels.getDms"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let all_channels = vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            is_dm: false,
            ..Default::default()
        },
        Channel {
            id: 2,
            name: "DM - alice".to_string(),
            is_dm: true,
            ..Default::default()
        },
    ];

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": all_channels
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/channels.getChannels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.get_dms().await;
    assert!(res.is_ok(), "fallback get_dms failed: {:?}", res);
    let dms = res.unwrap();
    assert_eq!(dms.len(), 1);
    assert_eq!(dms[0].id, 2);
    assert!(dms[0].is_dm);
}

#[tokio::test]
async fn test_client_get_members_fallback_to_get_users() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    client.set_session_token(Some("test-token-123".to_string()));

    Mock::given(method("GET"))
        .and(path("/trpc/users.getAll"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/trpc/users.getMembers"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let users = vec![User {
        id: 77,
        name: "fallback_user".to_string(),
        ..Default::default()
    }];

    let trpc_response = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": users
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/users.getUsers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(trpc_response))
        .mount(&mock_server)
        .await;

    let res = client.get_members().await;
    assert!(res.is_ok(), "fallback get_members failed: {:?}", res);
    let members = res.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].id, 77);
}

#[tokio::test]
async fn test_client_upload_file_unauthorized() {
    let mock_server = MockServer::start().await;
    let client = SharkordClient::new(&mock_server.uri()).unwrap();

    Mock::given(method("POST"))
        .and(path("/upload"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let res = client.upload_file("unauthorized.png", vec![0, 1]).await;
    assert!(res.is_err());
    match res {
        Err(sharkord_core::SharkordError::Unauthorized(_)) => {}
        other => panic!("Expected Unauthorized, got {:?}", other),
    }
}

