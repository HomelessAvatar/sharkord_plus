use desktop_gpui::media::voice_client::{LiveVoiceState, VoiceInputMode};
use desktop_gpui::theme;
use desktop_gpui::ui::voice_bar::{
    voice_status_text, VoiceBarState, VoiceBarView, VoiceConnectionState,
};
use desktop_gpui::ui::voice_stage::{VoiceStageCard, VoiceStageView};

#[test]
fn test_live_voice_state_transitions() {
    let mut voice = LiveVoiceState::new();
    assert!(!voice.is_connected);
    assert_eq!(voice.channel_id, None);
    assert!(!voice.is_muted);
    assert!(!voice.is_deafened);
    assert!(!voice.is_speaking);
    assert_eq!(voice.audio_level, 0.0);

    // Connect
    voice.connect(101);
    assert!(voice.is_connected);
    assert_eq!(voice.channel_id, Some(101));

    // Speaking state & audio level
    voice.set_speaking(true, 0.85);
    assert!(voice.is_speaking);
    assert_eq!(voice.audio_level, 0.85);

    // Mute transitions
    voice.toggle_mute();
    assert!(voice.is_muted);
    voice.toggle_mute();
    assert!(!voice.is_muted);

    // Deafen transitions (deafen automatically mutes)
    voice.toggle_deafen();
    assert!(voice.is_deafened);
    assert!(voice.is_muted);
    voice.toggle_deafen();
    assert!(!voice.is_deafened);

    // Ping / Latency
    voice.set_ping(45);
    assert_eq!(voice.ping_ms, 45);

    // Disconnect
    voice.disconnect();
    assert!(!voice.is_connected);
    assert_eq!(voice.channel_id, None);
    assert!(!voice.is_speaking);
}

#[test]
fn test_live_voice_participant_speaking_rings() {
    let mut voice = LiveVoiceState::new();

    // User 42 starts speaking with level 0.72
    voice.set_user_speaking(42, true, 0.72);
    assert!(voice.is_user_speaking(42));
    assert_eq!(voice.user_audio_level(42), 0.72);

    // User 99 starts speaking
    voice.set_user_speaking(99, true, 0.45);
    assert!(voice.is_user_speaking(99));
    assert_eq!(voice.active_speakers().len(), 2);
    assert!(voice.active_speakers().contains(&42));
    assert!(voice.active_speakers().contains(&99));

    // User 42 stops speaking
    voice.set_user_speaking(42, false, 0.0);
    assert!(!voice.is_user_speaking(42));
    assert_eq!(voice.user_audio_level(42), 0.0);
    assert_eq!(voice.active_speakers().len(), 1);

    // Cleanup / clear participants
    voice.clear_participants();
    assert_eq!(voice.active_speakers().len(), 0);
    assert!(!voice.is_user_speaking(99));
}

#[test]
fn test_live_voice_vad_and_ptt_gating() {
    let mut voice = LiveVoiceState::new();
    voice.set_input_mode(VoiceInputMode::VoiceActivity);
    voice.set_vad_threshold(0.15);

    // Below VAD threshold
    let speaking = voice.process_audio_sample(0.08);
    assert!(!speaking);
    assert!(!voice.is_speaking);

    // Above VAD threshold
    let speaking = voice.process_audio_sample(0.35);
    assert!(speaking);
    assert!(voice.is_speaking);
    assert_eq!(voice.audio_level, 0.35);

    // Muted mic prevents speaking even above threshold
    voice.set_muted(true);
    let speaking = voice.process_audio_sample(0.80);
    assert!(!speaking);
    assert!(!voice.is_speaking);
    voice.set_muted(false);

    // Push-to-Talk Mode
    voice.set_input_mode(VoiceInputMode::PushToTalk);
    assert!(!voice.is_ptt_pressed);

    // Audio input while PTT key is not pressed -> gated out
    let speaking = voice.process_audio_sample(0.90);
    assert!(!speaking);
    assert!(!voice.is_speaking);

    // Activate PTT
    voice.set_ptt_pressed(true);
    let speaking = voice.process_audio_sample(0.90);
    assert!(speaking);
    assert!(voice.is_speaking);
    assert_eq!(voice.audio_level, 0.90);

    // Release PTT
    voice.set_ptt_pressed(false);
    let speaking = voice.process_audio_sample(0.90);
    assert!(!speaking);
    assert!(!voice.is_speaking);
}

#[test]
fn test_voice_stage_and_voice_bar_rendering() {
    // 1. Participant card on Voice Stage
    let speaking_card = VoiceStageCard {
        user_id: 10,
        username: "Alice".to_string(),
        avatar_url: None,
        is_speaking: true,
        is_muted: false,
        is_deafened: false,
        volume: 1.0,
    };
    assert!(speaking_card.is_speaking);
    assert_eq!(speaking_card.border_color(), Some(theme::STATUS_ONLINE));
    let _card_el = speaking_card.render_card();

    let silent_card = VoiceStageCard {
        user_id: 20,
        username: "Bob".to_string(),
        avatar_url: None,
        is_speaking: false,
        is_muted: true,
        is_deafened: false,
        volume: 1.0,
    };
    assert!(!silent_card.is_speaking);
    assert_eq!(silent_card.border_color(), None);
    let _silent_el = silent_card.render_card();

    // Voice Stage view with active speaking cards
    let stage_view = VoiceStageView::new(501, "Stage Room", vec![speaking_card, silent_card]);
    assert_eq!(stage_view.participants.len(), 2);
    let _stage_el = stage_view.render_element();

    // 2. Docked Voice Bar with latency badge
    let bar_state = VoiceBarState {
        connection: VoiceConnectionState::Connected {
            channel_id: 501,
            channel_name: "Stage Room".to_string(),
            ping_ms: 18,
        },
        is_muted: false,
        is_deafened: false,
    };
    assert_eq!(
        voice_status_text(&bar_state.connection),
        "Connected / Stage Room (18ms)"
    );
    let bar_view = VoiceBarView::new(bar_state);
    let _bar_el = bar_view.render_element();
}
