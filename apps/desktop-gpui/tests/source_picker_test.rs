use desktop_gpui::ui::source_picker::{SourcePickerState, SourcePickerTab, SourcePickerView};
use sharkord_core::voice::video::{ScreenSource, VideoFrameRate, VideoResolution};

#[test]
fn test_source_picker_state_and_tabs() {
    let mut picker = SourcePickerState::new();

    assert_eq!(picker.current_tab, SourcePickerTab::Screens);
    assert_eq!(picker.selected_resolution, VideoResolution::R1080p);
    assert_eq!(picker.selected_framerate, VideoFrameRate::Fps60);
    assert!(picker.capture_window_audio);
    assert!(picker.selected_source.is_none());

    // Without a selected source, building config returns None
    assert!(picker.build_track_config().is_none());

    picker.select_tab(SourcePickerTab::Windows);
    assert_eq!(picker.current_tab, SourcePickerTab::Windows);

    picker.select_source(ScreenSource::Window {
        id: 101,
        title: "VS Code".to_string(),
    });
    assert!(picker.selected_source.is_some());
    assert_eq!(picker.selected_source.as_ref().unwrap().id(), 101);

    // Build configuration
    let config = picker.build_track_config().expect("should build config");
    assert_eq!(config.resolution, VideoResolution::R1080p);
    assert_eq!(config.framerate, VideoFrameRate::Fps60);
    assert!(config.with_audio);

    // Test adjustments
    picker.set_resolution(VideoResolution::R720p);
    assert_eq!(picker.selected_resolution, VideoResolution::R720p);

    picker.set_framerate(VideoFrameRate::Fps30);
    assert_eq!(picker.selected_framerate, VideoFrameRate::Fps30);

    picker.toggle_audio();
    assert!(!picker.capture_window_audio);

    let updated_config = picker.build_track_config().expect("updated config");
    assert_eq!(updated_config.resolution, VideoResolution::R720p);
    assert_eq!(updated_config.framerate, VideoFrameRate::Fps30);
    assert!(!updated_config.with_audio);
}

#[test]
fn test_source_picker_view_instantiation() {
    let mut state = SourcePickerState::new();
    state.available_sources = vec![
        ScreenSource::Monitor {
            id: 1,
            name: "Display 1".to_string(),
        },
        ScreenSource::Window {
            id: 101,
            title: "Browser".to_string(),
        },
    ];
    state.select_source(state.available_sources[0].clone());

    let view = SourcePickerView::new(state);
    assert_eq!(view.state.available_sources.len(), 2);
}

#[test]
fn test_source_picker_allows_4k_and_144fps_selection() {
    let mut state = SourcePickerState::new();
    state.select_source(ScreenSource::Monitor {
        id: 1,
        name: "Primary Display".to_string(),
    });
    state.set_resolution(VideoResolution::R4K);
    state.set_framerate(VideoFrameRate::Fps144);

    let config = state.build_track_config().expect("config should be built");
    assert_eq!(config.resolution, VideoResolution::R4K);
    assert_eq!(config.framerate, VideoFrameRate::Fps144);
    assert_eq!(config.resolution.width(), 3840);
    assert_eq!(config.framerate.fps(), 144);

    state.set_framerate(VideoFrameRate::Fps120);
    let config120 = state.build_track_config().expect("config should be built");
    assert_eq!(config120.resolution, VideoResolution::R4K);
    assert_eq!(config120.framerate, VideoFrameRate::Fps120);
    assert_eq!(config120.framerate.fps(), 120);
}

#[test]
fn test_source_picker_view_preset_badges() {
    assert!(SourcePickerView::AVAILABLE_RESOLUTIONS.contains(&VideoResolution::R4K));
    assert!(SourcePickerView::AVAILABLE_FRAMERATES.contains(&VideoFrameRate::Fps120));
    assert!(SourcePickerView::AVAILABLE_FRAMERATES.contains(&VideoFrameRate::Fps144));
}
