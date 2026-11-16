//! Voice Activity Detection (VAD) and RMS power measurement.

/// Configuration parameters for Voice Activity Detection.
#[derive(Debug, Clone, PartialEq)]
pub struct VadConfig {
    /// Decibel threshold above which audio is considered voice (e.g., -35.0 dB).
    pub threshold_db: f32,
    /// Number of consecutive frames above threshold required to trigger voice (attack).
    pub attack_frames: usize,
    /// Number of frames voice remains active after dropping below threshold (hangover/release).
    pub hangover_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            threshold_db: -35.0,
            attack_frames: 2,
            hangover_frames: 10,
        }
    }
}

/// Voice Activity Detection engine with attack and hangover hysteresis.
pub struct VadEngine {
    config: VadConfig,
    is_speaking: bool,
    attack_counter: usize,
    hangover_counter: usize,
}

impl VadEngine {
    pub fn new(config: VadConfig) -> Self {
        Self {
            config,
            is_speaking: false,
            attack_counter: 0,
            hangover_counter: 0,
        }
    }

    /// Processes a slice of float PCM samples, returning `(is_speaking, rms_linear)`.
    pub fn process_frames(&mut self, pcm: &[f32]) -> (bool, f32) {
        if pcm.is_empty() {
            return (self.is_speaking, 0.0);
        }

        // Calculate Root Mean Square (RMS)
        let sum_sq: f32 = pcm.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / pcm.len() as f32).sqrt();

        // Convert RMS to decibels (dBFS)
        let db = if rms > 1e-6 {
            20.0 * rms.log10()
        } else {
            -120.0
        };

        if db >= self.config.threshold_db {
            self.attack_counter += 1;
            if self.attack_counter >= self.config.attack_frames {
                self.is_speaking = true;
                self.hangover_counter = self.config.hangover_frames;
            }
        } else {
            self.attack_counter = 0;
            if self.hangover_counter > 0 {
                self.hangover_counter -= 1;
            } else {
                self.is_speaking = false;
            }
        }

        (self.is_speaking, rms)
    }

    /// Returns whether the engine is currently in a speaking state.
    pub fn is_speaking(&self) -> bool {
        self.is_speaking
    }

    /// Resets VAD state.
    pub fn reset(&mut self) {
        self.is_speaking = false;
        self.attack_counter = 0;
        self.hangover_counter = 0;
    }
}
