pub mod events;
pub use events::*;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::SharkordError;

/// tRPC error payload structure containing error message and code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrpcErrorPayload {
    pub message: String,
    pub code: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// tRPC wrapper for error payloads under the "json" field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrpcErrorWrapper {
    pub json: TrpcErrorPayload,
}

/// Inner tRPC data wrapper holding the actual typed JSON data payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrpcData<T> {
    pub json: T,
}

/// tRPC result envelope wrapping the data container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrpcResult<T> {
    pub data: TrpcData<T>,
}

/// Standard tRPC response envelope containing either a result or an error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct TrpcResponse<T> {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<TrpcResult<T>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<TrpcErrorWrapper>,
}

/// Encodes query parameters for a batched tRPC GET request.
///
/// If input is provided, formats as: `batch=1&input=<url-encoded {"0": {"json": <input>}}>`
/// If input is `None`, formats as: `batch=1`
pub fn encode_query_input<T: Serialize>(input: Option<&T>) -> Result<String, SharkordError> {
    match input {
        Some(val) => {
            let envelope = serde_json::json!({
                "0": {
                    "json": val
                }
            });
            let json_str = serde_json::to_string(&envelope)?;
            let encoded = urlencoding::encode(&json_str);
            Ok(format!("batch=1&input={}", encoded))
        }
        None => Ok("batch=1".to_string()),
    }
}

/// Alias for [`encode_query_input`].
pub fn encode_query_params<T: Serialize>(input: Option<&T>) -> Result<String, SharkordError> {
    encode_query_input(input)
}

/// Encodes JSON request body for a batched tRPC POST mutation request.
///
/// Formats as: `{"0": {"json": <input>}}` or `{"0": {"json": null}}` when `input` is `None`.
pub fn encode_mutation_body<T: Serialize>(input: Option<&T>) -> Result<String, SharkordError> {
    let envelope = match input {
        Some(val) => serde_json::json!({
            "0": {
                "json": val
            }
        }),
        None => serde_json::json!({
            "0": {
                "json": null
            }
        }),
    };
    Ok(serde_json::to_string(&envelope)?)
}

/// Decodes a batched or single tRPC response payload into the target type `T`.
///
/// Supports:
/// - Batched array responses: `[{"result": {"data": {"json": ...}}}]`
/// - Batched object envelope responses: `{"0": {"result": {"data": {"json": ...}}}}`
/// - Single responses: `{"result": {"data": {"json": ...}}}`
/// - Error envelopes mapping to [`SharkordError::RpcError`].
pub fn decode_trpc_response<T: DeserializeOwned>(json_str: &str) -> Result<T, SharkordError> {
    let root: serde_json::Value = serde_json::from_str(json_str)?;

    let item = if let Some(arr) = root.as_array() {
        arr.first()
            .ok_or_else(|| SharkordError::Protocol("Empty tRPC response array".to_string()))?
    } else if let Some(obj) = root.as_object() {
        if let Some(first) = obj.get("0") {
            first
        } else {
            &root
        }
    } else {
        &root
    };

    // Check for error payload
    if let Some(error_val) = item.get("error") {
        if let Some(err_str) = error_val.as_str() {
            return Err(SharkordError::RpcError {
                code: -1,
                message: err_str.to_string(),
            });
        }

        let target = error_val.get("json").unwrap_or(error_val);
        let code = target
            .get("code")
            .and_then(|c| {
                c.as_i64()
                    .map(|n| n as i32)
                    .or_else(|| c.as_str().and_then(|s| s.parse::<i32>().ok()))
            })
            .unwrap_or(-1);

        let message = target
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown tRPC error")
            .to_string();

        return Err(SharkordError::RpcError { code, message });
    }

    // Check for result payload
    if let Some(result_val) = item.get("result") {
        if let Some(data_val) = result_val.get("data") {
            if let Some(json_val) = data_val.get("json") {
                return serde_json::from_value::<T>(json_val.clone())
                    .map_err(SharkordError::Serialization);
            } else {
                return serde_json::from_value::<T>(data_val.clone())
                    .map_err(SharkordError::Serialization);
            }
        }
    }

    // Check for direct data payload
    if let Some(data_val) = item.get("data") {
        if let Some(json_val) = data_val.get("json") {
            return serde_json::from_value::<T>(json_val.clone())
                .map_err(SharkordError::Serialization);
        } else {
            return serde_json::from_value::<T>(data_val.clone())
                .map_err(SharkordError::Serialization);
        }
    }

    Err(SharkordError::Protocol(format!(
        "Invalid tRPC response payload structure: {}",
        json_str
    )))
}
