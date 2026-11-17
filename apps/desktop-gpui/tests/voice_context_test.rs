use desktop_gpui::state::AppState;
use desktop_gpui::ui::voice_context_menu::VoiceParticipantMenuState;

#[test]
fn test_participant_context_menu_state() {
    let mut menu = VoiceParticipantMenuState::new(202, "Charlie".to_string());

    assert_eq!(menu.user_id, 202);
    assert_eq!(menu.username, "Charlie");
    assert_eq!(menu.volume, 1.0);
    assert!(!menu.is_local_muted);

    menu.set_volume(1.25);
    assert_eq!(menu.volume, 1.25);
    assert_eq!(menu.volume_percentage(), 125);

    menu.toggle_local_mute();
    assert!(menu.is_local_muted);
}

#[test]
fn test_app_state_user_volume_and_local_mute() {
    let mut state = AppState::new();

    // Default volume is 1.0
    assert_eq!(state.get_user_volume(202), 1.0);
    assert!(!state.is_local_muted(202));

    state.set_user_volume(202, 1.5);
    assert_eq!(state.get_user_volume(202), 1.5);

    // Clamping
    state.set_user_volume(202, 2.5);
    assert_eq!(state.get_user_volume(202), 2.0);
    state.set_user_volume(202, -0.5);
    assert_eq!(state.get_user_volume(202), 0.0);

    // Toggle local mute
    assert!(state.toggle_local_mute(202));
    assert!(state.is_local_muted(202));
    assert!(!state.toggle_local_mute(202));
    assert!(!state.is_local_muted(202));
}
