use sharkord_core::voice::audio::AudioDeviceManager;
use sharkord_core::voice::vad::{VadConfig, VadEngine};

#[test]
fn test_vad_silence_vs_speech() {
    let mut vad = VadEngine::new(VadConfig {
        threshold_db: -30.0,
        attack_frames: 1,
        hangover_frames: 3,
    });

    // 1. Pure silence
    let silence = vec![0.0f32; 480];
    let (is_speaking, rms) = vad.process_frames(&silence);
    assert!(!is_speaking, "Silence should not trigger VAD");
    assert_eq!(rms, 0.0);

    // 2. High amplitude speech (e.g. 0.5 amplitude sine wave)
    let speech: Vec<f32> = (0..480)
        .map(|i| (i as f32 * 0.1).sin() * 0.5)
        .collect();
    let (is_speaking, rms) = vad.process_frames(&speech);
    assert!(is_speaking, "Loud speech should trigger VAD");
    assert!(rms > 0.2, "RMS should be significant");

    // 3. Silence immediately after speech should stay true due to hangover
    let (is_speaking_hangover_1, _) = vad.process_frames(&silence);
    assert!(
        is_speaking_hangover_1,
        "VAD should remain speaking during hangover frame 1"
    );

    let (is_speaking_hangover_2, _) = vad.process_frames(&silence);
    assert!(
        is_speaking_hangover_2,
        "VAD should remain speaking during hangover frame 2"
    );

    let (is_speaking_hangover_3, _) = vad.process_frames(&silence);
    assert!(
        is_speaking_hangover_3,
        "VAD should remain speaking during hangover frame 3"
    );

    // 4. After hangover frames expire, it should drop to false
    let (is_speaking_after_hangover, _) = vad.process_frames(&silence);
    assert!(
        !is_speaking_after_hangover,
        "VAD should return to silence after hangover expires"
    );
}

#[test]
fn test_audio_device_manager_discovery() {
    let manager = AudioDeviceManager::new();
    let inputs = manager.list_input_devices().expect("can query inputs");
    let outputs = manager.list_output_devices().expect("can query outputs");

    // System should have at least 0 devices without throwing a panic
    println!("Found {} input devices, {} output devices", inputs.len(), outputs.len());
    for dev in &inputs {
        assert!(!dev.name.is_empty());
    }
    for dev in &outputs {
        assert!(!dev.name.is_empty());
    }
}
