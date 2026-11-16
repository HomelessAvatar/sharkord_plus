use desktop_gpui::ui::audio_settings::{
    AudioSettingsState, DEFAULT_INPUT_VOLUME, DEFAULT_OUTPUT_VOLUME, DEFAULT_VAD_THRESHOLD,
};

#[test]
fn test_audio_settings_state_and_vu_meter() {
    let mut settings = AudioSettingsState::new();

    // Default values
    assert_eq!(settings.input_volume, DEFAULT_INPUT_VOLUME);
    assert_eq!(settings.output_volume, DEFAULT_OUTPUT_VOLUME);
    assert_eq!(settings.vad_threshold, DEFAULT_VAD_THRESHOLD);
    assert!(!settings.is_testing_mic);
    assert_eq!(settings.test_mic_level, 0.0);

    // Volume clamps
    settings.set_input_volume(2.5); // Should clamp to 2.0
    assert_eq!(settings.input_volume, 2.0);

    settings.set_input_volume(-0.5); // Should clamp to 0.0
    assert_eq!(settings.input_volume, 0.0);

    // Mic test state toggle
    settings.start_mic_test();
    assert!(settings.is_testing_mic);

    settings.update_mic_level(0.75);
    assert_eq!(settings.test_mic_level, 0.75);
    assert_eq!(settings.vu_meter_percentage(), 75);

    settings.stop_mic_test();
    assert!(!settings.is_testing_mic);
    assert_eq!(settings.test_mic_level, 0.0);
    assert_eq!(settings.vu_meter_percentage(), 0);
}
