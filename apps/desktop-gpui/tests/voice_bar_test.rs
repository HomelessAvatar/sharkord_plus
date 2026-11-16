use desktop_gpui::state::AppState;
use desktop_gpui::ui::voice_bar::VoiceConnectionState;

#[test]
fn test_app_state_voice_controls() {
    let mut state = AppState::new();

    // Default state: disconnected
    assert_eq!(
        state.voice_bar_state.connection,
        VoiceConnectionState::Disconnected
    );
    assert!(!state.voice_bar_state.is_muted);
    assert!(!state.voice_bar_state.is_deafened);

    // 1. Connect to voice
    state.connect_voice(101, "General Voice".to_string());
    assert_eq!(
        state.voice_bar_state.connection,
        VoiceConnectionState::Connected {
            channel_id: 101,
            channel_name: "General Voice".to_string(),
            ping_ms: 0,
        }
    );

    // 2. Update Ping
    state.update_voice_ping(24);
    if let VoiceConnectionState::Connected { ping_ms, .. } = state.voice_bar_state.connection {
        assert_eq!(ping_ms, 24);
    } else {
        panic!("Voice connection should be Connected");
    }

    // 3. Toggle Mute
    let muted = state.toggle_voice_mute();
    assert!(muted);
    assert!(state.voice_bar_state.is_muted);

    // 4. Toggle Deafen
    let deafened = state.toggle_voice_deafen();
    assert!(deafened);
    assert!(state.voice_bar_state.is_deafened);
    assert!(state.voice_bar_state.is_muted); // deafen implies mute

    // 5. Disconnect
    state.disconnect_voice();
    assert_eq!(
        state.voice_bar_state.connection,
        VoiceConnectionState::Disconnected
    );
}
