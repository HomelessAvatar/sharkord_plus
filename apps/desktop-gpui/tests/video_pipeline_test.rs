use desktop_gpui::media::screen_capture::{CaptureSource, CaptureSourceType, ScreenCaptureEngine};
use desktop_gpui::media::video_decoder::{VideoDecoderPipeline, VideoFrame};
use desktop_gpui::ui::source_picker::{SourcePickerState, SourcePickerTab, SourcePickerView};
use desktop_gpui::ui::video_tile::{VideoDisplayMode, VideoTileState, VideoTileView};
use sharkord_core::voice::video::StreamTelemetryStats;

#[test]
fn test_video_decoder_initialization() {
    let decoder = VideoDecoderPipeline::new();
    assert!(decoder.is_ok(), "VideoDecoderPipeline should initialize successfully");
    let mut dec = decoder.unwrap();
    let decode_result = dec.decode(&[]);
    assert!(decode_result.is_ok(), "Empty NAL unit decoding should be handled cleanly");
}

#[test]
fn test_screen_capture_source_enumeration() {
    let sources = ScreenCaptureEngine::enumerate_sources();
    assert!(
        !sources.is_empty(),
        "ScreenCaptureEngine should return at least one source (or fallback)"
    );

    let has_monitors = sources
        .iter()
        .any(|s| matches!(s.source_type, CaptureSourceType::Monitor));
    let has_windows = sources
        .iter()
        .any(|s| matches!(s.source_type, CaptureSourceType::Window));
    assert!(
        has_monitors || has_windows,
        "Should enumerate monitors or windows"
    );

    let first: Option<&CaptureSource> = sources.first();
    assert!(first.is_some());
    for source in &sources {
        assert!(!source.id.is_empty(), "Source ID must not be empty");
        assert!(!source.name.is_empty(), "Source name must not be empty");
        assert!(source.width > 0, "Source width must be positive");
        assert!(source.height > 0, "Source height must be positive");
    }
}

#[test]
fn test_video_frame_dimensions_and_conversion() {
    let width = 1920;
    let height = 1080;
    let raw_rgba = vec![128u8; (width * height * 4) as usize];
    let frame = VideoFrame::new(width, height, raw_rgba, 54321);

    assert_eq!(frame.width, 1920);
    assert_eq!(frame.height, 1080);
    assert_eq!(frame.timestamp_ms, 54321);
    assert_eq!(frame.buffer_size(), (1920 * 1080 * 4) as usize);
    assert_eq!(frame.rgba_data.len(), (1920 * 1080 * 4) as usize);

    // Aspect ratio calculation
    let aspect = frame.aspect_ratio();
    assert!((aspect - (16.0 / 9.0)).abs() < 0.001);

    // Zero-height frame safety
    let zero_frame = VideoFrame::new(100, 0, vec![], 0);
    assert_eq!(zero_frame.aspect_ratio(), 0.0);

    // Conversion to and from sharkord_core::voice::video::VideoFrame
    let core_frame: sharkord_core::voice::video::VideoFrame = frame.clone().into();
    assert_eq!(core_frame.width, 1920);
    assert_eq!(core_frame.height, 1080);
    assert_eq!(core_frame.data.len(), (1920 * 1080 * 4) as usize);
    assert_eq!(core_frame.timestamp_ms, 54321);

    let converted_back: VideoFrame = core_frame.into();
    assert_eq!(converted_back.width, 1920);
    assert_eq!(converted_back.height, 1080);
    assert_eq!(converted_back.rgba_data.len(), (1920 * 1080 * 4) as usize);
    assert_eq!(converted_back.timestamp_ms, 54321);
}

#[test]
fn test_source_picker_integration() {
    let mut state = SourcePickerState::new();
    assert_eq!(state.current_tab, SourcePickerTab::Screens);

    // Populate state from ScreenCaptureEngine
    state.populate_from_engine();
    assert!(
        !state.capture_sources.is_empty(),
        "populate_from_engine should populate capture_sources"
    );

    if let Some(source) = state.capture_sources.first().cloned() {
        state.select_capture_source(source.clone());
        assert_eq!(
            state.selected_capture_source.as_ref().map(|s| &s.id),
            Some(&source.id)
        );
    }

    let view = SourcePickerView::new(state);
    let _element = view.render_element();
}

#[test]
fn test_video_tile_rendering_with_frame() {
    let mut state = VideoTileState::new(101, "ScreenStreamer".to_string());

    let stats = StreamTelemetryStats::new(1080, 60, 6_000_000, 0.002);
    state.update_stats(stats);

    let frame = VideoFrame::new(1280, 720, vec![200u8; 1280 * 720 * 4], 1000);
    state.current_frame = Some(frame.clone());

    assert!(!state.is_fullscreen);
    assert!(!state.is_pip);
    state.toggle_fullscreen();
    assert!(state.is_fullscreen);
    assert_eq!(state.display_mode, VideoDisplayMode::Fullscreen);
    state.toggle_pip();
    assert!(state.is_pip);

    let view = VideoTileView::new(state)
        .with_size(1280.0, 720.0)
        .with_frame(frame);

    assert!(view.current_frame.is_some());
    let _element = view.render_element();
}
