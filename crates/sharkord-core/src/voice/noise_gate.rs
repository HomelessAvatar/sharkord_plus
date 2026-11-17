//! Noise Gate DSP filter for voice transmission.
//! Smoothly attenuates audio below a configurable threshold using attack and release curves.

/// Configuration parameters for the Noise Gate.
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseGateConfig {
    pub open_threshold: f32,
    pub close_threshold: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub sample_rate: u32,
}

impl Default for NoiseGateConfig {
    fn default() -> Self {
        Self {
            open_threshold: 0.03,
            close_threshold: 0.015,
            attack_ms: 10.0,
            release_ms: 50.0,
            sample_rate: 48000,
        }
    }
}

/// Noise Gate audio processor.
#[derive(Debug, Clone)]
pub struct NoiseGate {
    config: NoiseGateConfig,
    is_open: bool,
    gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
}

impl NoiseGate {
    /// Creates a new `NoiseGate` with the given configuration.
    pub fn new(config: NoiseGateConfig) -> Self {
        let sample_rate = config.sample_rate as f32;
        let attack_samples = (config.attack_ms * 0.001 * sample_rate).max(1.0);
        let release_samples = (config.release_ms * 0.001 * sample_rate).max(1.0);

        let attack_coeff = (1.0 / attack_samples).min(1.0);
        let release_coeff = (1.0 / release_samples).min(1.0);

        Self {
            config,
            is_open: false,
            gain: 0.0,
            attack_coeff,
            release_coeff,
        }
    }

    /// Returns whether the gate is currently open (transmitting audio).
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the current audio attenuation multiplier (0.0 = muted, 1.0 = pass-through).
    pub fn current_gain(&self) -> f32 {
        self.gain
    }

    /// Processes a slice of PCM floating-point audio samples in-place.
    pub fn process_frame(&mut self, samples: &mut [f32]) {
        if samples.is_empty() {
            return;
        }

        // Calculate root-mean-square energy of the frame
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();

        // Hysteresis thresholding
        if !self.is_open && rms >= self.config.open_threshold {
            self.is_open = true;
        } else if self.is_open && rms < self.config.close_threshold {
            self.is_open = false;
        }

        let target_gain = if self.is_open { 1.0 } else { 0.0 };

        for sample in samples.iter_mut() {
            if self.gain < target_gain {
                self.gain = (self.gain + self.attack_coeff).min(target_gain);
            } else if self.gain > target_gain {
                self.gain = (self.gain - self.release_coeff).max(target_gain);
            }
            *sample *= self.gain;
        }
    }
}
