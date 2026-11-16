use sharkord_core::voice::session::{VoiceEvent, VoiceSession};

#[tokio::test]
async fn test_voice_session_mute_deafen_lifecycle() {
    let session = VoiceSession::new(42);
    let mut rx = session.subscribe_events();

    assert_eq!(session.channel_id(), 42);
    assert!(!session.is_muted());
    assert!(!session.is_deafened());

    // 1. Toggle Mute
    let now_muted = session.toggle_mute();
    assert!(now_muted);
    assert!(session.is_muted());

    // Verify event
    if let Ok(VoiceEvent::StateUpdated {
        mic_muted,
        sound_deafened,
    }) = rx.recv().await
    {
        assert!(mic_muted);
        assert!(!sound_deafened);
    } else {
        panic!("Expected StateUpdated event on mute toggle");
    }

    // 2. Toggle Deafen (deafening automatically mutes)
    let now_deafened = session.toggle_deafen();
    assert!(now_deafened);
    assert!(session.is_deafened());
    assert!(session.is_muted());

    // Verify deafen event
    if let Ok(VoiceEvent::StateUpdated {
        mic_muted,
        sound_deafened,
    }) = rx.recv().await
    {
        assert!(mic_muted);
        assert!(sound_deafened);
    } else {
        panic!("Expected StateUpdated event on deafen toggle");
    }

    // 3. Speaking event dispatch
    session.set_speaking(true, 0.45);
    if let Ok(VoiceEvent::SpeakingChanged { is_speaking, level }) = rx.recv().await {
        assert!(is_speaking);
        assert!((level - 0.45).abs() < 1e-4);
    } else {
        panic!("Expected SpeakingChanged event");
    }
}
