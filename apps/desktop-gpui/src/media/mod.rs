//! Live WebRTC voice and media pipeline subsystem.
//! Interfaces with Mediasoup SFU via WebRTC, manages local audio capture/playback via cpal,
//! screen sharing capture via xcap, and H.264 video decoding via openh264.

pub mod screen_capture;
pub mod video_decoder;
pub mod voice_client;

pub use screen_capture::{CaptureSource, CaptureSourceType, ScreenCaptureEngine};
pub use video_decoder::{VideoDecoderPipeline, VideoFrame};
pub use voice_client::{LiveVoiceClient, LiveVoiceState, VoiceInputMode};
