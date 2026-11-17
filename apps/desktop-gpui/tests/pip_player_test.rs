use desktop_gpui::state::AppState;
use desktop_gpui::ui::pip_player::{PipPlayerState, PipPlayerView};

#[test]
fn test_pip_player_state_transitions() {
    let mut pip = PipPlayerState::new(301, "Dave's Stream".to_string());

    assert_eq!(pip.streamer_id, 301);
    assert_eq!(pip.streamer_name, "Dave's Stream");
    assert!(!pip.is_visible);
    assert!(!pip.is_audio_muted);

    pip.show();
    assert!(pip.is_visible);

    // Dimension checks (240x135 fixed 16:9)
    assert_eq!(pip.width, 240.0);
    assert_eq!(pip.height, 135.0);

    pip.toggle_audio_mute();
    assert!(pip.is_audio_muted);

    pip.hide();
    assert!(!pip.is_visible);
}

#[test]
fn test_app_state_pip_integration() {
    let mut state = AppState::new();
    assert!(state.active_pip_player.is_none());

    let pip = PipPlayerState::new(505, "Game Stream".to_string());
    state.set_pip_player(pip);
    assert!(state.active_pip_player.is_some());
    assert_eq!(state.active_pip_player.as_ref().unwrap().streamer_id, 505);

    state.close_pip_player();
    assert!(state.active_pip_player.is_none());
}

#[test]
fn test_pip_player_view_instantiation() {
    let mut pip = PipPlayerState::new(777, "Live Coding".to_string());
    pip.show();
    pip.toggle_audio_mute();

    let view = PipPlayerView::new(pip.clone());
    assert_eq!(view.state.streamer_id, 777);
    assert_eq!(view.state.streamer_name, "Live Coding");
    assert!(view.state.is_visible);
    assert!(view.state.is_audio_muted);
}
