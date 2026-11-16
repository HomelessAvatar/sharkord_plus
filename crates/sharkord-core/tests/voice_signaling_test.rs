use sharkord_core::voice::{DtlsParameters, ProducerOptions, VoiceStateInput};
use sharkord_core::SharkordClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_voice_signaling_join_and_leave() {
    let mock_server = MockServer::start().await;

    let join_response_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "routerRtpCapabilities": {
                            "codecs": [
                                {
                                    "mimeType": "audio/opus",
                                    "kind": "audio",
                                    "clockRate": 48000,
                                    "channels": 2
                                }
                            ]
                        }
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/voice.join"))
        .respond_with(ResponseTemplate::new(200).set_body_json(join_response_json))
        .mount(&mock_server)
        .await;

    let leave_response_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": null
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/voice.leave"))
        .respond_with(ResponseTemplate::new(200).set_body_json(leave_response_json))
        .mount(&mock_server)
        .await;

    let client = SharkordClient::new(&mock_server.uri()).expect("valid client");

    let join_res = client
        .join_voice(
            101,
            VoiceStateInput {
                mic_muted: false,
                sound_muted: false,
            },
        )
        .await
        .expect("join_voice succeeds");

    assert!(join_res.router_rtp_capabilities.is_object());

    client.leave_voice().await.expect("leave_voice succeeds");
}

#[tokio::test]
async fn test_voice_transport_and_produce() {
    let mock_server = MockServer::start().await;

    let create_transport_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "id": "prod-transport-1",
                        "iceParameters": { "usernameFragment": "test-ufrag" },
                        "iceCandidates": [],
                        "dtlsParameters": { "role": "auto" }
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/voice.createProducerTransport"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_transport_json))
        .mount(&mock_server)
        .await;

    let connect_transport_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": null
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/voice.connectProducerTransport"))
        .respond_with(ResponseTemplate::new(200).set_body_json(connect_transport_json))
        .mount(&mock_server)
        .await;

    let produce_json = serde_json::json!({
        "0": {
            "result": {
                "data": {
                    "json": {
                        "id": "producer-audio-99"
                    }
                }
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/trpc/voice.produce"))
        .respond_with(ResponseTemplate::new(200).set_body_json(produce_json))
        .mount(&mock_server)
        .await;

    let client = SharkordClient::new(&mock_server.uri()).expect("valid client");

    let transport = client
        .create_producer_transport()
        .await
        .expect("create_producer_transport succeeds");
    assert_eq!(transport.id, "prod-transport-1");

    client
        .connect_producer_transport(DtlsParameters {
            role: Some("client".to_string()),
            fingerprints: vec![],
        })
        .await
        .expect("connect_producer_transport succeeds");

    let produce_res = client
        .produce_track(ProducerOptions {
            kind: "audio".to_string(),
            rtp_parameters: serde_json::json!({ "codecs": [] }),
            app_data: None,
        })
        .await
        .expect("produce_track succeeds");

    assert_eq!(produce_res.id, "producer-audio-99");
}
