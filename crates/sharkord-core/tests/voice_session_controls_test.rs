use sharkord_core::voice::session::{InputMode, VoiceSession};

#[test]
fn test_per_user_volume_and_local_mute() {
    let session = VoiceSession::new(101);

    // Default volume is 1.0 (100%)
    assert_eq!(session.get_user_volume(42), 1.0);
    assert!(!session.is_local_muted(42));
    assert_eq!(session.effective_output_gain(42), 1.0);

    // Adjust volume (e.g. 150%)
    session.set_user_volume(42, 1.5);
    assert_eq!(session.get_user_volume(42), 1.5);
    assert_eq!(session.effective_output_gain(42), 1.5);

    // Local mute overrides volume to 0.0
    session.set_local_mute(42, true);
    assert!(session.is_local_muted(42));
    assert_eq!(session.effective_output_gain(42), 0.0);

    // Unmute restores volume
    session.set_local_mute(42, false);
    assert_eq!(session.effective_output_gain(42), 1.5);

    // Clamping: max 2.0, min 0.0
    session.set_user_volume(42, 3.5);
    assert_eq!(session.get_user_volume(42), 2.0);
    session.set_user_volume(42, -1.0);
    assert_eq!(session.get_user_volume(42), 0.0);

    // Deafen forces effective gain to 0.0 for everyone
    session.toggle_deafen();
    assert_eq!(session.effective_output_gain(42), 0.0);
}

#[test]
fn test_input_mode_and_ptt_state() {
    let session = VoiceSession::new(101);

    // Default input mode is VoiceActivity
    assert_eq!(session.input_mode(), InputMode::VoiceActivity);

    // Switch to Push-to-Talk
    session.set_input_mode(InputMode::PushToTalk);
    assert_eq!(session.input_mode(), InputMode::PushToTalk);

    // In PTT mode, transmission is false until PTT key is active
    assert!(!session.is_ptt_active());
    session.set_ptt_active(true);
    assert!(session.is_ptt_active());
    session.set_ptt_active(false);
    assert!(!session.is_ptt_active());
}
