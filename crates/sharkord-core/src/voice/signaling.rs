use crate::client::SharkordClient;
use crate::error::SharkordError;
use crate::protocol::{decode_trpc_response, encode_mutation_body};
use crate::voice::types::{
    DtlsParameters, ProduceResponse, ProducerOptions, TransportOptions, VoiceJoinResponse,
    VoiceStateInput,
};

impl SharkordClient {
    /// Joins a voice channel (`voice.join`).
    pub async fn join_voice(
        &self,
        channel_id: u32,
        state: VoiceStateInput,
    ) -> Result<VoiceJoinResponse, SharkordError> {
        let input = serde_json::json!({
            "channelId": channel_id,
            "state": state
        });
        let body = encode_mutation_body(Some(&input))?;
        let url = format!("{}/trpc/voice.join", self.base_url());

        let resp = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} joining voice channel",
                resp.status()
            )));
        }

        let body_str = resp.text().await?;
        decode_trpc_response(&body_str)
    }

    /// Leaves the current voice channel (`voice.leave`).
    pub async fn leave_voice(&self) -> Result<(), SharkordError> {
        let body = encode_mutation_body::<()>(None)?;
        let url = format!("{}/trpc/voice.leave", self.base_url());

        let resp = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} leaving voice channel",
                resp.status()
            )));
        }

        Ok(())
    }

    /// Creates a WebRTC transport on the server for sending audio/video (`voice.createProducerTransport`).
    pub async fn create_producer_transport(&self) -> Result<TransportOptions, SharkordError> {
        let body = encode_mutation_body::<()>(None)?;
        let url = format!("{}/trpc/voice.createProducerTransport", self.base_url());

        let resp = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} creating producer transport",
                resp.status()
            )));
        }

        let body_str = resp.text().await?;
        decode_trpc_response(&body_str)
    }

    /// Connects a WebRTC producer transport with client DTLS parameters (`voice.connectProducerTransport`).
    pub async fn connect_producer_transport(
        &self,
        dtls_parameters: DtlsParameters,
    ) -> Result<(), SharkordError> {
        let input = serde_json::json!({
            "dtlsParameters": dtls_parameters
        });
        let body = encode_mutation_body(Some(&input))?;
        let url = format!("{}/trpc/voice.connectProducerTransport", self.base_url());

        let resp = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} connecting producer transport",
                resp.status()
            )));
        }

        Ok(())
    }

    /// Starts producing an audio or video track on the Mediasoup SFU (`voice.produce`).
    pub async fn produce_track(
        &self,
        options: ProducerOptions,
    ) -> Result<ProduceResponse, SharkordError> {
        let input = serde_json::to_value(&options)
            .map_err(SharkordError::Serialization)?;
        let body = encode_mutation_body(Some(&input))?;
        let url = format!("{}/trpc/voice.produce", self.base_url());

        let resp = self
            .http_client()
            .post(&url)
            .headers(self.auth_headers())
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "HTTP {} producing voice track",
                resp.status()
            )));
        }

        let body_str = resp.text().await?;
        decode_trpc_response(&body_str)
    }
}
