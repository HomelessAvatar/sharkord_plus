use sharkord_core::voice::video::{VideoFrameRate, VideoResolution};

#[test]
fn test_4k_resolution_dimensions_and_label() {
    let res_4k = VideoResolution::R4K;
    assert_eq!(res_4k.width(), 3840);
    assert_eq!(res_4k.height(), 2160);
    assert_eq!(res_4k.label(), "4K");
}

#[test]
fn test_custom_resolution_dimensions() {
    let custom = VideoResolution::Custom {
        width: 1920,
        height: 1200,
    };
    assert_eq!(custom.width(), 1920);
    assert_eq!(custom.height(), 1200);
    assert_eq!(custom.label(), "1920x1200");
}

#[test]
fn test_esports_framerates() {
    let fps_120 = VideoFrameRate::Fps120;
    let fps_144 = VideoFrameRate::Fps144;
    let fps_custom = VideoFrameRate::Custom(240);

    assert_eq!(fps_120.fps(), 120);
    assert_eq!(fps_120.label(), "120 FPS");
    assert_eq!(fps_144.fps(), 144);
    assert_eq!(fps_144.label(), "144 FPS");
    assert_eq!(fps_custom.fps(), 240);
    assert_eq!(fps_custom.label(), "240 FPS");
}

#[test]
fn test_video_resolution_serde_backward_compatibility() {
    // Standard presets
    let json_720p: VideoResolution = serde_json::from_str("\"720p\"").unwrap();
    assert_eq!(json_720p, VideoResolution::R720p);

    let json_1080p: VideoResolution = serde_json::from_str("\"1080p\"").unwrap();
    assert_eq!(json_1080p, VideoResolution::R1080p);

    let json_1440p: VideoResolution = serde_json::from_str("\"1440p\"").unwrap();
    assert_eq!(json_1440p, VideoResolution::R1440p);

    // 4K presets and aliases
    let json_4k: VideoResolution = serde_json::from_str("\"4k\"").unwrap();
    assert_eq!(json_4k, VideoResolution::R4K);

    let json_4k_upper: VideoResolution = serde_json::from_str("\"4K\"").unwrap();
    assert_eq!(json_4k_upper, VideoResolution::R4K);

    let json_2160p: VideoResolution = serde_json::from_str("\"2160p\"").unwrap();
    assert_eq!(json_2160p, VideoResolution::R4K);

    // Custom resolution roundtrip
    let custom = VideoResolution::Custom {
        width: 2560,
        height: 1080,
    };
    let serialized = serde_json::to_string(&custom).unwrap();
    let deserialized: VideoResolution = serde_json::from_str(&serialized).unwrap();
    assert_eq!(custom, deserialized);
}

#[test]
fn test_video_framerate_serde_backward_compatibility() {
    // Standard presets
    let json_15: VideoFrameRate = serde_json::from_str("\"15\"").unwrap();
    assert_eq!(json_15, VideoFrameRate::Fps15);

    let json_30: VideoFrameRate = serde_json::from_str("\"30\"").unwrap();
    assert_eq!(json_30, VideoFrameRate::Fps30);

    let json_60: VideoFrameRate = serde_json::from_str("\"60\"").unwrap();
    assert_eq!(json_60, VideoFrameRate::Fps60);

    // Esports framerates
    let json_120: VideoFrameRate = serde_json::from_str("\"120\"").unwrap();
    assert_eq!(json_120, VideoFrameRate::Fps120);

    let json_144: VideoFrameRate = serde_json::from_str("\"144\"").unwrap();
    assert_eq!(json_144, VideoFrameRate::Fps144);

    // Custom framerate roundtrip
    let custom_fps = VideoFrameRate::Custom(240);
    let serialized = serde_json::to_string(&custom_fps).unwrap();
    let deserialized: VideoFrameRate = serde_json::from_str(&serialized).unwrap();
    assert_eq!(custom_fps, deserialized);
}
