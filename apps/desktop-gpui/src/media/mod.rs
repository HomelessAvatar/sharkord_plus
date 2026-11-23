//! Live WebRTC voice and media pipeline subsystem.
//! Interfaces with Mediasoup SFU via WebRTC and manages local audio capture/playback via cpal.

pub mod voice_client;

pub use voice_client::{LiveVoiceClient, LiveVoiceState, VoiceInputMode};
