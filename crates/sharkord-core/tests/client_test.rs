use sharkord_core::{Channel, ChannelType, Message, ServerSettings, SharkordClient, User};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_url_normalization_and_validation() {
    // Normalization: strip trailing slashes
    let client1 = SharkordClient::new("https://example.com/").expect("valid url");
    assert_eq!(client1.base_url(), "https://example.com");

    let client2 = SharkordClient::new("http://127.0.0.1:3000///").expect("valid url");
    assert_eq!(client2.base_url(), "http://127.0.0.1:3000");

    let client3 = SharkordClient::new("https://sharkord.example.com/prefix/").expect("valid url");
    assert_eq!(client3.base_url(), "https://sharkord.example.com/prefix");

    // Scheme validation: must be http or https
    let invalid_scheme = SharkordClient::new("ftp://example.com");
    assert!(invalid_scheme.is_err());

    let invalid_url = SharkordClient::new("not-a-valid-url");
    assert!(invalid_url.is_err());
}

#[tokio::test]
async fn test_capability_negotiation_vanilla_server() {
    let mock_server = MockServer::start().await;

    let vanilla_settings_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "serverId": "vanilla-server-1",
                        "name": "Vanilla Sharkord",
                        "allowNewUsers": true
                    }
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/others.getPublicSettings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vanilla_settings_json))
        .mount(&mock_server)
        .await;

    let client = SharkordClient::connect(&mock_server.uri())
        .await
        .expect("Failed to connect to mock server");

    let caps = client.capabilities();
    assert!(!caps.is_plus, "Vanilla server must not have is_plus flag");
    assert!(
        !caps.high_profile_codecs,
        "Vanilla server must not have high profile codecs unlocked"
    );
    assert!(
        !caps.isolated_window_audio,
        "Vanilla server must not have isolated window audio unlocked"
    );
}

#[tokio::test]
async fn test_capability_negotiation_plus_server() {
    let mock_server = MockServer::start().await;

    let plus_settings_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "serverId": "plus-server-1",
                        "name": "Sharkord Plus Server",
                        "isPlus": true,
                        "highProfileCodecs": true,
                        "isolatedWindowAudio": true,
                        "serverVersion": "1.0.0-plus"
                    }
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/others.getPublicSettings"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-sharkord-plus", "true")
                .set_body_json(plus_settings_json),
        )
        .mount(&mock_server)
        .await;

    let client = SharkordClient::connect(&mock_server.uri())
        .await
        .expect("Failed to connect to mock server");

    let caps = client.capabilities();
    assert!(caps.is_plus, "Sharkord Plus server must have is_plus true");
    assert!(
        caps.high_profile_codecs,
        "Plus server must have high profile codecs unlocked"
    );
    assert!(
        caps.isolated_window_audio,
        "Plus server must have isolated window audio unlocked"
    );
    assert_eq!(caps.server_version.as_deref(), Some("1.0.0-plus"));
}

#[tokio::test]
async fn test_client_trpc_request_execution() {
    let mock_server = MockServer::start().await;

    // 1. Mock get_public_settings
    let settings_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "serverId": "test-srv",
                        "name": "Test Server",
                        "allowNewUsers": true
                    }
                }
            }
        }
    });
    Mock::given(method("GET"))
        .and(path("/trpc/others.getPublicSettings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(settings_json))
        .mount(&mock_server)
        .await;

    // 2. Mock login
    let login_response_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "token": "session-jwt-token-xyz",
                        "user": {
                            "id": 42,
                            "name": "Alice",
                            "identity": "alice",
                            "roleIds": []
                        }
                    }
                }
            }
        }
    });
    Mock::given(method("POST"))
        .and(path("/trpc/auth.login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header(
                    "Set-Cookie",
                    "token=session-jwt-token-xyz; Path=/; HttpOnly",
                )
                .set_body_json(login_response_json),
        )
        .mount(&mock_server)
        .await;

    // 3. Mock get_channels (expects Authorization header)
    let channels_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": [
                        {
                            "id": 1,
                            "name": "general",
                            "type": "TEXT",
                            "private": false,
                            "isDm": false,
                            "position": 0
                        },
                        {
                            "id": 2,
                            "name": "Voice Lounge",
                            "type": "VOICE",
                            "private": false,
                            "isDm": false,
                            "position": 1
                        }
                    ]
                }
            }
        }
    });
    Mock::given(method("GET"))
        .and(path("/trpc/channels.getChannels"))
        .and(header("authorization", "Bearer session-jwt-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(channels_json))
        .mount(&mock_server)
        .await;

    // 4. Mock send_message (expects Authorization header)
    let message_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "id": 101,
                        "channelId": 1,
                        "content": "Hello Sharkord Plus!",
                        "pinned": false
                    }
                }
            }
        }
    });
    Mock::given(method("POST"))
        .and(path("/trpc/messages.sendMessage"))
        .and(header("authorization", "Bearer session-jwt-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(message_json))
        .mount(&mock_server)
        .await;

    // 5. Mock get_messages (expects Authorization header)
    let messages_list_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": [
                        {
                            "id": 101,
                            "channelId": 1,
                            "content": "Hello Sharkord Plus!",
                            "pinned": false
                        }
                    ]
                }
            }
        }
    });
    Mock::given(method("GET"))
        .and(path("/trpc/messages.getMessages"))
        .and(header("authorization", "Bearer session-jwt-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(messages_list_json))
        .mount(&mock_server)
        .await;

    // Execute requests through SharkordClient
    let client = SharkordClient::connect(&mock_server.uri())
        .await
        .expect("Failed to connect");

    // Query settings
    let settings: ServerSettings = client
        .get_public_settings()
        .await
        .expect("Failed to fetch public settings");
    assert_eq!(settings.name, "Test Server");

    // Login
    let user: User = client
        .login("alice", "password123")
        .await
        .expect("Failed to login");
    assert_eq!(user.id, 42);
    assert_eq!(user.name, "Alice");
    assert_eq!(
        client.session_token().as_deref(),
        Some("session-jwt-token-xyz")
    );

    // Channels with session auth
    let channels: Vec<Channel> = client
        .get_channels()
        .await
        .expect("Failed to fetch channels");
    assert_eq!(channels.len(), 2);
    assert_eq!(channels[0].name, "general");
    assert_eq!(channels[0].channel_type, ChannelType::Text);
    assert_eq!(channels[1].name, "Voice Lounge");
    assert_eq!(channels[1].channel_type, ChannelType::Voice);

    // Send message
    let msg: Message = client
        .send_message(1, "Hello Sharkord Plus!")
        .await
        .expect("Failed to send message");
    assert_eq!(msg.id, 101);
    assert_eq!(msg.content.as_deref(), Some("Hello Sharkord Plus!"));

    // Get messages
    let msgs: Vec<Message> = client
        .get_messages(1, None)
        .await
        .expect("Failed to get messages");
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].id, 101);
    assert_eq!(msgs[0].content.as_deref(), Some("Hello Sharkord Plus!"));
}

#[tokio::test]
async fn test_session_token_and_header_injection() {
    let mock_server = MockServer::start().await;

    let channels_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": []
                }
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/trpc/channels.getChannels"))
        .and(header("authorization", "Bearer manual-auth-token-999"))
        .respond_with(ResponseTemplate::new(200).set_body_json(channels_json))
        .mount(&mock_server)
        .await;

    let client = SharkordClient::new(&mock_server.uri()).expect("client new");
    assert_eq!(client.session_token(), None);

    // Explicitly set session token
    client.set_session_token(Some("manual-auth-token-999".to_string()));
    assert_eq!(
        client.session_token().as_deref(),
        Some("manual-auth-token-999")
    );

    // Should include Authorization header
    let channels = client.get_channels().await.expect("channels request");
    assert!(channels.is_empty());

    // Clear session token
    client.set_session_token(None);
    assert_eq!(client.session_token(), None);
}
