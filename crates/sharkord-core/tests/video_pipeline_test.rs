use sharkord_core::voice::video::{
    ScreenSource, StreamTelemetryStats, VideoFrame, VideoFrameRate, VideoResolution,
    VideoTrackConfig,
};

#[test]
fn test_video_track_config_and_sources() {
    let monitor = ScreenSource::Monitor {
        id: 1,
        name: "Display 1 (2560x1440)".to_string(),
    };
    let window = ScreenSource::Window {
        id: 4520,
        title: "Game - 60 FPS".to_string(),
    };

    let config = VideoTrackConfig {
        source: monitor.clone(),
        resolution: VideoResolution::R1080p,
        framerate: VideoFrameRate::Fps60,
        with_audio: true,
    };

    assert_eq!(config.resolution.width(), 1920);
    assert_eq!(config.resolution.height(), 1080);
    assert_eq!(config.framerate.fps(), 60);
    assert!(config.with_audio);

    // Telemetry calculation
    let stats = StreamTelemetryStats::new(1080, 60, 4_500_000, 0.005);
    assert_eq!(stats.bitrate_mbps(), 4.5);
    assert_eq!(stats.packet_loss_percent(), 0.5);

    // Serialization test
    let json = serde_json::to_string(&window).expect("serialize window source");
    let deserialized: ScreenSource =
        serde_json::from_str(&json).expect("deserialize window source");
    assert_eq!(window, deserialized);
}

#[test]
fn test_video_frame_dimensions() {
    let frame = VideoFrame {
        width: 1920,
        height: 1080,
        data: vec![0u8; 1920 * 1080 * 4],
        timestamp_ms: 1000,
    };
    assert!((frame.aspect_ratio() - (16.0 / 9.0)).abs() < 0.001);

    let zero_frame = VideoFrame {
        width: 0,
        height: 0,
        data: vec![],
        timestamp_ms: 0,
    };
    assert_eq!(zero_frame.aspect_ratio(), 0.0);
}

#[test]
fn test_video_resolutions_and_framerates() {
    assert_eq!(VideoResolution::R720p.width(), 1280);
    assert_eq!(VideoResolution::R720p.height(), 720);
    assert_eq!(VideoResolution::R1440p.width(), 2560);
    assert_eq!(VideoResolution::R1440p.height(), 1440);

    assert_eq!(VideoFrameRate::Fps15.fps(), 15);
    assert_eq!(VideoFrameRate::Fps30.fps(), 30);
    assert_eq!(VideoFrameRate::Fps60.fps(), 60);
}
