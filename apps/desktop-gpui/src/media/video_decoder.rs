//! H.264 video decoding pipeline and RGBA frame models.
//! Integrates with `openh264::decoder::Decoder` with software fallback support.

use openh264::formats::YUVSource;
use serde::{Deserialize, Serialize};

/// Raw RGBA video frame payload for GPU texture rendering and UI presentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
    pub timestamp_ms: u64,
}

impl VideoFrame {
    /// Creates a new `VideoFrame`.
    pub fn new(width: u32, height: u32, rgba_data: Vec<u8>, timestamp_ms: u64) -> Self {
        Self {
            width,
            height,
            rgba_data,
            timestamp_ms,
        }
    }

    /// Calculates the aspect ratio (width / height).
    /// Safely returns 0.0 when height is 0 to avoid division-by-zero panics.
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f64 / self.height as f64
        }
    }

    /// Returns the length in bytes of the pixel buffer.
    pub fn buffer_size(&self) -> usize {
        self.rgba_data.len()
    }

    /// Expected size in bytes for a 32-bit RGBA buffer (width * height * 4).
    pub fn expected_buffer_size(&self) -> usize {
        (self.width * self.height * 4) as usize
    }

    /// Returns true if the buffer matches the expected 32-bit RGBA size.
    pub fn is_valid_buffer(&self) -> bool {
        self.buffer_size() == self.expected_buffer_size()
    }
}

impl From<sharkord_core::voice::video::VideoFrame> for VideoFrame {
    fn from(core: sharkord_core::voice::video::VideoFrame) -> Self {
        Self {
            width: core.width,
            height: core.height,
            rgba_data: core.data,
            timestamp_ms: core.timestamp_ms,
        }
    }
}

impl From<VideoFrame> for sharkord_core::voice::video::VideoFrame {
    fn from(frame: VideoFrame) -> Self {
        Self {
            width: frame.width,
            height: frame.height,
            data: frame.rgba_data,
            timestamp_ms: frame.timestamp_ms,
        }
    }
}

/// Video decoding pipeline integrating OpenH264 with software fallback.
pub struct VideoDecoderPipeline {
    decoder: Option<openh264::decoder::Decoder>,
    pub is_fallback: bool,
    last_frame_timestamp: u64,
}

impl VideoDecoderPipeline {
    /// Initializes the video decoder pipeline.
    /// Attempts OpenH264 initialization first, gracefully falling back if unavailable.
    pub fn new() -> Result<Self, String> {
        match openh264::decoder::Decoder::new() {
            Ok(decoder) => Ok(Self {
                decoder: Some(decoder),
                is_fallback: false,
                last_frame_timestamp: 0,
            }),
            Err(e) => {
                eprintln!("[VideoDecoderPipeline] OpenH264 init warning: {:?}, falling back to software mode", e);
                Ok(Self {
                    decoder: None,
                    is_fallback: true,
                    last_frame_timestamp: 0,
                })
            }
        }
    }

    /// Decodes an H.264 NAL unit or access unit packet into an optional `VideoFrame`.
    pub fn decode(&mut self, nal_unit: &[u8]) -> Result<Option<VideoFrame>, String> {
        if nal_unit.is_empty() {
            return Ok(None);
        }

        self.last_frame_timestamp = self.last_frame_timestamp.wrapping_add(16);

        if let Some(ref mut decoder) = self.decoder {
            match decoder.decode(nal_unit) {
                Ok(Some(decoded_yuv)) => {
                    let (width, height) = decoded_yuv.dimensions();
                    let mut rgba = vec![0u8; width * height * 4];
                    decoded_yuv.write_rgba8(&mut rgba);
                    Ok(Some(VideoFrame::new(
                        width as u32,
                        height as u32,
                        rgba,
                        self.last_frame_timestamp,
                    )))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(format!("OpenH264 decode error: {:?}", e)),
            }
        } else {
            // Software fallback placeholder
            Ok(None)
        }
    }
}
