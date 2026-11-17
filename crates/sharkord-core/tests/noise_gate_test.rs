use sharkord_core::voice::noise_gate::{NoiseGate, NoiseGateConfig};

#[test]
fn test_noise_gate_attenuation_and_attack() {
    let config = NoiseGateConfig {
        open_threshold: 0.05,
        close_threshold: 0.02,
        attack_ms: 10.0,
        release_ms: 50.0,
        sample_rate: 48000,
    };
    let mut gate = NoiseGate::new(config);

    // Initial state: gate is closed
    assert!(!gate.is_open());

    // 1. Low level noise (amplitude 0.01 < close_threshold 0.02)
    let mut noise = vec![0.01f32; 480];
    gate.process_frame(&mut noise);
    assert!(!gate.is_open());
    let energy_after: f32 = noise.iter().map(|s| s.abs()).sum::<f32>() / 480.0;
    assert!(
        energy_after < 0.005,
        "Low noise must be attenuated, got: {}",
        energy_after
    );

    // 2. Loud speech frame (amplitude 0.2 > open_threshold 0.05)
    let mut speech = vec![0.2f32; 480];
    gate.process_frame(&mut speech);
    assert!(gate.is_open(), "Loud audio frame must open the gate");

    // 3. Subsequent quiet frames should maintain gate open for release period
    let mut quiet = vec![0.01f32; 480];
    gate.process_frame(&mut quiet);
    // Because of release curve, gate smoothly decays rather than abruptly cutting off
    assert!(gate.current_gain() > 0.0);
}
