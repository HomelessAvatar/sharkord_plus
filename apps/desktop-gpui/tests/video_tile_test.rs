use desktop_gpui::ui::video_tile::{
    aspect_ratio_box, VideoDisplayMode, VideoTileState, VideoTileView,
};
use sharkord_core::voice::video::StreamTelemetryStats;

#[test]
fn test_video_tile_state_and_telemetry() {
    let mut tile = VideoTileState::new(301, "Dave's Screen".to_string());

    assert_eq!(tile.streamer_id, 301);
    assert_eq!(tile.streamer_name, "Dave's Screen");
    assert_eq!(tile.display_mode, VideoDisplayMode::Standard);
    assert!(!tile.is_fullscreen);
    assert!(!tile.is_audio_muted);

    tile.toggle_fullscreen();
    assert!(tile.is_fullscreen);

    tile.toggle_audio_mute();
    assert!(tile.is_audio_muted);

    let stats = StreamTelemetryStats::new(1080, 60, 5_000_000, 0.001);
    tile.update_stats(stats);
    assert!(tile.stats.is_some());
    assert_eq!(tile.stats.as_ref().unwrap().fps, 60);
    assert_eq!(tile.stats.as_ref().unwrap().bitrate_mbps(), 5.0);
}

#[test]
fn test_video_tile_mode_and_popover_toggles() {
    let mut tile = VideoTileState::new(402, "Alice's Window".to_string());

    assert_eq!(tile.display_mode, VideoDisplayMode::Standard);
    assert!(!tile.show_stats_popover);

    // Toggle theater mode
    tile.toggle_theater();
    assert_eq!(tile.display_mode, VideoDisplayMode::Theater);
    assert!(!tile.is_fullscreen);

    tile.toggle_theater();
    assert_eq!(tile.display_mode, VideoDisplayMode::Standard);

    // Toggle popover
    tile.toggle_stats_popover();
    assert!(tile.show_stats_popover);
    tile.toggle_stats_popover();
    assert!(!tile.show_stats_popover);
}

#[test]
fn test_aspect_ratio_box_calculation() {
    // 16:9 box within 1920x1080 container
    let (w, h) = aspect_ratio_box(1920.0, 1080.0);
    assert!((w - 1920.0).abs() < 0.01);
    assert!((h - 1080.0).abs() < 0.01);

    // Ultra-wide container: 2400x1080 -> constrained by height
    let (w, h) = aspect_ratio_box(2400.0, 1080.0);
    assert!((h - 1080.0).abs() < 0.01);
    assert!((w - 1920.0).abs() < 0.01);

    // Tall container: 800x1200 -> constrained by width
    let (w, h) = aspect_ratio_box(800.0, 1200.0);
    assert!((w - 800.0).abs() < 0.01);
    assert!((h - 450.0).abs() < 0.01);
}

#[test]
fn test_video_tile_view_instantiation() {
    let mut state = VideoTileState::new(505, "Bob's IDE".to_string());
    state.update_stats(StreamTelemetryStats::new(720, 30, 2_500_000, 0.002));
    state.toggle_stats_popover();

    let view = VideoTileView::new(state).with_size(960.0, 540.0);
    assert_eq!(view.state.streamer_id, 505);
    assert_eq!(view.width, 960.0);
    assert_eq!(view.height, 540.0);
    assert!(view.state.show_stats_popover);
}
