use serde::{Deserialize, Serialize};
use sharkord_core::{
    decode_trpc_response, encode_mutation_body, encode_query_input, SharkordError, TrpcData,
    TrpcErrorPayload, TrpcResponse, TrpcResult,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ChannelQueryInput {
    #[serde(rename = "channelId")]
    channel_id: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MessageMutationInput {
    content: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ChannelOutput {
    id: u32,
    name: String,
}

#[test]
fn test_encode_query_input() {
    let input = ChannelQueryInput { channel_id: 1 };
    let query_param = encode_query_input(Some(&input)).expect("failed to encode query input");
    assert_eq!(
        query_param,
        "batch=1&input=%7B%220%22%3A%7B%22json%22%3A%7B%22channelId%22%3A1%7D%7D%7D"
    );

    // Test query input with None
    let none_query = encode_query_input::<()>(None).expect("failed to encode none query");
    assert_eq!(none_query, "batch=1");
}

#[test]
fn test_encode_mutation_body() {
    let input = MessageMutationInput {
        content: "hello".to_string(),
    };
    let body = encode_mutation_body(Some(&input)).expect("failed to encode mutation body");
    assert_eq!(body, r#"{"0":{"json":{"content":"hello"}}}"#);

    // Test mutation body with None
    let none_body = encode_mutation_body::<()>(None).expect("failed to encode none mutation");
    assert_eq!(none_body, r#"{"0":{"json":null}}"#);
}

#[test]
fn test_decode_trpc_response_array_format() {
    let payload = r#"[{"result":{"data":{"json":{"id":1,"name":"general"}}}}] "#;
    let channel: ChannelOutput =
        decode_trpc_response(payload).expect("failed to decode array response");
    assert_eq!(
        channel,
        ChannelOutput {
            id: 1,
            name: "general".to_string(),
        }
    );
}

#[test]
fn test_decode_trpc_response_object_envelope_format() {
    let payload = r#"{"0":{"result":{"data":{"json":{"id":2,"name":"voice"}}}}} "#;
    let channel: ChannelOutput =
        decode_trpc_response(payload).expect("failed to decode object envelope response");
    assert_eq!(
        channel,
        ChannelOutput {
            id: 2,
            name: "voice".to_string(),
        }
    );
}

#[test]
fn test_decode_trpc_error_payload() {
    let payload = r#"[{"error":{"json":{"message":"Invalid input","code":-32600}}}]"#;
    let result: Result<ChannelOutput, SharkordError> = decode_trpc_response(payload);
    match result {
        Err(SharkordError::RpcError { code, message }) => {
            assert_eq!(code, -32600);
            assert_eq!(message, "Invalid input");
        }
        other => panic!("Expected SharkordError::RpcError, got: {:?}", other),
    }

    // Also test object envelope error format
    let obj_payload = r#"{"0":{"error":{"json":{"message":"Unauthorized","code":-32001}}}}"#;
    let obj_result: Result<ChannelOutput, SharkordError> = decode_trpc_response(obj_payload);
    match obj_result {
        Err(SharkordError::RpcError { code, message }) => {
            assert_eq!(code, -32001);
            assert_eq!(message, "Unauthorized");
        }
        other => panic!("Expected SharkordError::RpcError, got: {:?}", other),
    }
}

#[test]
fn test_trpc_envelope_structures() {
    let response = TrpcResponse {
        result: Some(TrpcResult {
            data: TrpcData {
                json: ChannelOutput {
                    id: 42,
                    name: "random".to_string(),
                },
            },
        }),
        error: None,
    };

    let serialized = serde_json::to_string(&response).expect("failed to serialize TrpcResponse");
    assert!(serialized.contains(r#""result":{"data":{"json":{"id":42,"name":"random"}}}"#));

    let deserialized: TrpcResponse<ChannelOutput> =
        serde_json::from_str(&serialized).expect("failed to deserialize TrpcResponse");
    assert_eq!(response, deserialized);

    let err_payload = TrpcErrorPayload {
        message: "Bad Request".to_string(),
        code: -32600,
        data: None,
    };
    let err_str = serde_json::to_string(&err_payload).expect("failed to serialize error payload");
    assert!(err_str.contains(r#""message":"Bad Request""#));
    assert!(err_str.contains(r#""code":-32600"#));
}
