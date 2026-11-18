//! Video and screen sharing pipeline models, track configurations, and telemetry stats.

use serde::{Deserialize, Serialize};

/// Source selected for screen sharing (either an entire physical monitor or a specific application window).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScreenSource {
    /// Physical display monitor identified by its display index.
    Monitor { id: u64, name: String },
    /// Individual application window identified by its system window handle or ID.
    Window { id: u64, title: String },
}

impl ScreenSource {
    /// Returns the unique identifier for the screen source.
    pub fn id(&self) -> u64 {
        match self {
            ScreenSource::Monitor { id, .. } => *id,
            ScreenSource::Window { id, .. } => *id,
        }
    }

    /// Returns the display name or title for the screen source.
    pub fn name(&self) -> &str {
        match self {
            ScreenSource::Monitor { name, .. } => name.as_str(),
            ScreenSource::Window { title, .. } => title.as_str(),
        }
    }

    /// Alias for `name()` providing the title of the source.
    pub fn title(&self) -> &str {
        self.name()
    }

    /// Returns true if this source is a physical monitor display.
    pub fn is_monitor(&self) -> bool {
        matches!(self, ScreenSource::Monitor { .. })
    }

    /// Returns true if this source is an application window.
    pub fn is_window(&self) -> bool {
        matches!(self, ScreenSource::Window { .. })
    }
}

impl std::fmt::Display for ScreenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Target resolution preset for screen share encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VideoResolution {
    #[serde(rename = "720p", alias = "R720p", alias = "r720p")]
    R720p,
    #[default]
    #[serde(rename = "1080p", alias = "R1080p", alias = "r1080p")]
    R1080p,
    #[serde(rename = "1440p", alias = "R1440p", alias = "r1440p")]
    R1440p,
    #[serde(rename = "4k", alias = "4K", alias = "R4K", alias = "r4k", alias = "2160p", alias = "R2160p")]
    R4K,
    #[serde(rename = "custom", alias = "Custom")]
    Custom { width: u32, height: u32 },
}

impl VideoResolution {
    /// Returns the target width in pixels for the resolution preset.
    pub fn width(&self) -> u32 {
        match self {
            VideoResolution::R720p => 1280,
            VideoResolution::R1080p => 1920,
            VideoResolution::R1440p => 2560,
            VideoResolution::R4K => 3840,
            VideoResolution::Custom { width, .. } => *width,
        }
    }

    /// Returns the target height in pixels for the resolution preset.
    pub fn height(&self) -> u32 {
        match self {
            VideoResolution::R720p => 720,
            VideoResolution::R1080p => 1080,
            VideoResolution::R1440p => 1440,
            VideoResolution::R4K => 2160,
            VideoResolution::Custom { height, .. } => *height,
        }
    }

    /// Returns human-readable label for the resolution preset.
    pub fn label(&self) -> String {
        match self {
            VideoResolution::R720p => "720p".to_string(),
            VideoResolution::R1080p => "1080p".to_string(),
            VideoResolution::R1440p => "1440p".to_string(),
            VideoResolution::R4K => "4K".to_string(),
            VideoResolution::Custom { width, height } => format!("{width}x{height}"),
        }
    }
}

impl std::fmt::Display for VideoResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Target frame rate preset for screen share encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VideoFrameRate {
    #[serde(rename = "15", alias = "Fps15", alias = "fps15", alias = "15fps")]
    Fps15,
    #[serde(rename = "30", alias = "Fps30", alias = "fps30", alias = "30fps")]
    Fps30,
    #[default]
    #[serde(rename = "60", alias = "Fps60", alias = "fps60", alias = "60fps")]
    Fps60,
    #[serde(rename = "120", alias = "Fps120", alias = "fps120", alias = "120fps")]
    Fps120,
    #[serde(rename = "144", alias = "Fps144", alias = "fps144", alias = "144fps")]
    Fps144,
    #[serde(rename = "custom", alias = "Custom")]
    Custom(u32),
}

impl VideoFrameRate {
    /// Returns the target frames per second as a numeric integer.
    pub fn fps(&self) -> u32 {
        match self {
            VideoFrameRate::Fps15 => 15,
            VideoFrameRate::Fps30 => 30,
            VideoFrameRate::Fps60 => 60,
            VideoFrameRate::Fps120 => 120,
            VideoFrameRate::Fps144 => 144,
            VideoFrameRate::Custom(fps) => *fps,
        }
    }

    /// Returns human-readable label for the frame rate preset.
    pub fn label(&self) -> String {
        match self {
            VideoFrameRate::Fps15 => "15 FPS".to_string(),
            VideoFrameRate::Fps30 => "30 FPS".to_string(),
            VideoFrameRate::Fps60 => "60 FPS".to_string(),
            VideoFrameRate::Fps120 => "120 FPS".to_string(),
            VideoFrameRate::Fps144 => "144 FPS".to_string(),
            VideoFrameRate::Custom(fps) => format!("{fps} FPS"),
        }
    }
}

impl std::fmt::Display for VideoFrameRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Configuration settings for a video track capture pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoTrackConfig {
    /// The screen or application window capture source.
    pub source: ScreenSource,
    /// Video encoding target resolution.
    pub resolution: VideoResolution,
    /// Video encoding target framerate.
    pub framerate: VideoFrameRate,
    /// Whether application/system audio loopback is captured alongside the video.
    pub with_audio: bool,
}

impl VideoTrackConfig {
    /// Creates a new `VideoTrackConfig` with specified parameters.
    pub fn new(
        source: ScreenSource,
        resolution: VideoResolution,
        framerate: VideoFrameRate,
        with_audio: bool,
    ) -> Self {
        Self {
            source,
            resolution,
            framerate,
            with_audio,
        }
    }
}

/// Real-time streaming telemetry statistics for active video consumption or production.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamTelemetryStats {
    /// Resolution vertical height in pixels (e.g., 720, 1080, 1440).
    pub resolution_height: u32,
    /// Current rendered/received frames per second.
    pub fps: u32,
    /// Current transmission bitrate in bits per second.
    pub bitrate_bps: u64,
    /// Packet loss ratio represented as a fractional float between 0.0 and 1.0.
    pub packet_loss_ratio: f64,
}

impl StreamTelemetryStats {
    /// Creates a new `StreamTelemetryStats` record.
    pub fn new(resolution_height: u32, fps: u32, bitrate_bps: u64, packet_loss_ratio: f64) -> Self {
        Self {
            resolution_height,
            fps,
            bitrate_bps,
            packet_loss_ratio,
        }
    }

    /// Creates telemetry stats from raw packet loss counters, guarding against division by zero.
    pub fn from_packets(
        resolution_height: u32,
        fps: u32,
        bitrate_bps: u64,
        packets_lost: u64,
        packets_total: u64,
    ) -> Self {
        let packet_loss_ratio = if packets_total == 0 {
            0.0
        } else {
            packets_lost as f64 / packets_total as f64
        };
        Self {
            resolution_height,
            fps,
            bitrate_bps,
            packet_loss_ratio,
        }
    }

    /// Returns the transmission bitrate formatted in Megabits per second (Mbps).
    pub fn bitrate_mbps(&self) -> f64 {
        self.bitrate_bps as f64 / 1_000_000.0
    }

    /// Returns the packet loss percentage (0.0% to 100.0%).
    pub fn packet_loss_percent(&self) -> f64 {
        self.packet_loss_ratio * 100.0
    }

    /// Convenience getter for vertical resolution height.
    pub fn height(&self) -> u32 {
        self.resolution_height
    }
}

impl Default for StreamTelemetryStats {
    fn default() -> Self {
        Self {
            resolution_height: 0,
            fps: 0,
            bitrate_bps: 0,
            packet_loss_ratio: 0.0,
        }
    }
}

/// Raw RGBA or BGRA video frame payload for GPU rendering and transport encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoFrame {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Raw uncompressed image frame pixel buffer (typically RGBA32).
    pub data: Vec<u8>,
    /// Monotonic timestamp in milliseconds when the frame was captured.
    pub timestamp_ms: u64,
}

impl VideoFrame {
    /// Creates a new `VideoFrame`.
    pub fn new(width: u32, height: u32, data: Vec<u8>, timestamp_ms: u64) -> Self {
        Self {
            width,
            height,
            data,
            timestamp_ms,
        }
    }

    /// Calculates the frame aspect ratio (width / height).
    /// Safely returns 0.0 when height is 0 to avoid division-by-zero panics.
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            0.0
        } else {
            self.width as f64 / self.height as f64
        }
    }
}
