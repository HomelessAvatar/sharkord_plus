use sharkord_core::types::{
    InteractiveComponent, Message, PluginActionPayload, PluginButton, PluginCommand, PluginPoll,
    PluginPollOption,
};

#[test]
fn test_plugin_command_serialization() {
    let cmd = PluginCommand {
        plugin_id: "soundboard".to_string(),
        name: "play".to_string(),
        description: "Play sound bite into voice channel".to_string(),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("soundboard"));
    assert!(json.contains("play"));

    let deserialized: PluginCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.plugin_id, "soundboard");
    assert_eq!(deserialized.name, "play");
    assert_eq!(deserialized.description, "Play sound bite into voice channel");
}

#[test]
fn test_plugin_action_payload_serialization() {
    let payload = PluginActionPayload {
        plugin_id: "poll".to_string(),
        action_name: "vote".to_string(),
        channel_id: Some(42),
        payload: Some(serde_json::json!({ "optionId": "opt-1" })),
    };

    let json = serde_json::to_string(&payload).unwrap();
    assert!(json.contains("poll"));
    assert!(json.contains("vote"));
    assert!(json.contains("42"));
    assert!(json.contains("opt-1"));

    let deserialized: PluginActionPayload = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.plugin_id, "poll");
    assert_eq!(deserialized.action_name, "vote");
    assert_eq!(deserialized.channel_id, Some(42));
}

#[test]
fn test_interactive_button_serialization() {
    let btn = PluginButton {
        id: "btn-confirm".to_string(),
        label: "Confirm".to_string(),
        plugin_id: Some("moderation".to_string()),
        action_name: Some("confirm_ban".to_string()),
        style: Some("danger".to_string()),
        disabled: Some(false),
        payload: Some(serde_json::json!({ "targetUserId": 100 })),
    };

    let component = InteractiveComponent::Button(btn.clone());
    let json = serde_json::to_string(&component).unwrap();
    assert!(json.contains("btn-confirm"));
    assert!(json.contains("Confirm"));
    assert!(json.contains("danger"));

    let deserialized: InteractiveComponent = serde_json::from_str(&json).unwrap();
    match deserialized {
        InteractiveComponent::Button(b) => {
            assert_eq!(b.id, "btn-confirm");
            assert_eq!(b.label, "Confirm");
            assert_eq!(b.plugin_id.as_deref(), Some("moderation"));
        }
        _ => panic!("Expected InteractiveComponent::Button"),
    }
}

#[test]
fn test_interactive_poll_serialization() {
    let poll = PluginPoll {
        id: "poll-1".to_string(),
        question: "Favorite programming language?".to_string(),
        options: vec![
            PluginPollOption {
                id: "rust".to_string(),
                text: "Rust".to_string(),
                votes: 42,
                voted: true,
            },
            PluginPollOption {
                id: "dart".to_string(),
                text: "Dart".to_string(),
                votes: 18,
                voted: false,
            },
        ],
        plugin_id: Some("polls".to_string()),
        action_name: Some("submit_vote".to_string()),
        total_votes: Some(60),
        allows_multiple: false,
        closed: false,
    };

    let component = InteractiveComponent::Poll(poll);
    let json = serde_json::to_string(&component).unwrap();
    assert!(json.contains("Favorite programming language?"));
    assert!(json.contains("Rust"));
    assert!(json.contains("Dart"));

    let deserialized: InteractiveComponent = serde_json::from_str(&json).unwrap();
    match deserialized {
        InteractiveComponent::Poll(p) => {
            assert_eq!(p.id, "poll-1");
            assert_eq!(p.options.len(), 2);
            assert_eq!(p.options[0].text, "Rust");
            assert_eq!(p.options[0].votes, 42);
            assert!(p.options[0].voted);
        }
        _ => panic!("Expected InteractiveComponent::Poll"),
    }
}

#[test]
fn test_message_with_components_deserialization() {
    let json = serde_json::json!({
        "id": 101,
        "channelId": 5,
        "userId": 1,
        "content": "Please vote on this proposal:",
        "components": [
            {
                "type": "button",
                "id": "btn-vote",
                "label": "Click me",
                "pluginId": "quickvote",
                "actionName": "vote"
            }
        ]
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    assert_eq!(msg.id, 101);
    assert_eq!(msg.content.as_deref(), Some("Please vote on this proposal:"));
    assert_eq!(msg.components.len(), 1);
    match &msg.components[0] {
        InteractiveComponent::Button(btn) => {
            assert_eq!(btn.label, "Click me");
            assert_eq!(btn.plugin_id.as_deref(), Some("quickvote"));
        }
        _ => panic!("Expected button component"),
    }
}

#[tokio::test]
async fn test_client_call_plugin_action() {
    use sharkord_core::SharkordClient;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!([
        {
            "result": {
                "data": {
                    "success": true,
                    "count": 42
                }
            }
        }
    ]);

    Mock::given(method("POST"))
        .and(path("/trpc/plugins.executeAction"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    let result = client
        .call_plugin_action(
            "polls",
            "vote",
            Some(serde_json::json!({ "optionId": "opt-1" })),
            Some(10),
        )
        .await
        .expect("call_plugin_action success");

    assert_eq!(result.get("success").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(result.get("count").and_then(|v| v.as_u64()), Some(42));
}

#[tokio::test]
async fn test_client_get_plugin_commands() {
    use sharkord_core::SharkordClient;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!([
        {
            "result": {
                "data": {
                    "soundboard": [
                        {
                            "pluginId": "soundboard",
                            "name": "play",
                            "description": "Play sound in channel"
                        }
                    ],
                    "polls": [
                        {
                            "pluginId": "polls",
                            "name": "create",
                            "description": "Create a new poll"
                        }
                    ]
                }
            }
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/trpc/plugins.getCommands"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let client = SharkordClient::new(&mock_server.uri()).unwrap();
    let commands = client
        .get_plugin_commands(None)
        .await
        .expect("get_plugin_commands success");

    assert_eq!(commands.len(), 2);
    let names: Vec<String> = commands.into_iter().map(|c| c.name).collect();
    assert!(names.contains(&"play".to_string()));
    assert!(names.contains(&"create".to_string()));
}
