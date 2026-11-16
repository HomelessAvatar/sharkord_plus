//! Audio Settings view component.
//! Provides audio input/output device selection, volume sliders, VAD sensitivity threshold,
//! and an interactive microphone test with a real-time VU meter bar.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::voice::audio::AudioDevice;

use crate::theme;

pub const DEFAULT_INPUT_VOLUME: f32 = 1.0;
pub const DEFAULT_OUTPUT_VOLUME: f32 = 1.0;
pub const DEFAULT_VAD_THRESHOLD: f32 = 0.02;

/// State for the audio settings panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSettingsState {
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub input_volume: f32,
    pub output_volume: f32,
    pub vad_threshold: f32,
    pub is_testing_mic: bool,
    pub test_mic_level: f32,
}

impl Default for AudioSettingsState {
    fn default() -> Self {
        Self {
            input_device: None,
            output_device: None,
            input_volume: DEFAULT_INPUT_VOLUME,
            output_volume: DEFAULT_OUTPUT_VOLUME,
            vad_threshold: DEFAULT_VAD_THRESHOLD,
            is_testing_mic: false,
            test_mic_level: 0.0,
        }
    }
}

impl AudioSettingsState {
    /// Creates a new `AudioSettingsState` with default configurations.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the input volume, clamped between 0.0 and 2.0.
    pub fn set_input_volume(&mut self, volume: f32) {
        self.input_volume = volume.clamp(0.0, 2.0);
    }

    /// Sets the output volume, clamped between 0.0 and 2.0.
    pub fn set_output_volume(&mut self, volume: f32) {
        self.output_volume = volume.clamp(0.0, 2.0);
    }

    /// Sets the voice activity detection threshold, clamped between 0.0 and 1.0.
    pub fn set_vad_threshold(&mut self, threshold: f32) {
        self.vad_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Starts microphone testing.
    pub fn start_mic_test(&mut self) {
        self.is_testing_mic = true;
    }

    /// Stops microphone testing and resets the test level.
    pub fn stop_mic_test(&mut self) {
        self.is_testing_mic = false;
        self.test_mic_level = 0.0;
    }

    /// Updates the current test microphone audio level (RMS 0.0..=1.0).
    pub fn update_mic_level(&mut self, level: f32) {
        if self.is_testing_mic {
            self.test_mic_level = level.clamp(0.0, 1.0);
        }
    }

    /// Returns the current VU meter fill percentage (0..=100).
    pub fn vu_meter_percentage(&self) -> u32 {
        if !self.is_testing_mic {
            0
        } else {
            (self.test_mic_level * 100.0).round() as u32
        }
    }
}

/// View component rendering audio settings and device selection.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioSettingsView {
    pub state: AudioSettingsState,
    pub input_devices: Vec<AudioDevice>,
    pub output_devices: Vec<AudioDevice>,
}

impl AudioSettingsView {
    pub fn new(
        state: AudioSettingsState,
        input_devices: Vec<AudioDevice>,
        output_devices: Vec<AudioDevice>,
    ) -> Self {
        Self {
            state,
            input_devices,
            output_devices,
        }
    }

    /// Renders the audio settings panel.
    pub fn render_element(&self) -> impl IntoElement {
        let vu_percent = self.state.vu_meter_percentage();
        let mic_btn_text = if self.state.is_testing_mic {
            "Stop Testing"
        } else {
            "Let's Check"
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap_4()
            .p_4()
            .bg(theme::bg_surface())
            .rounded_lg()
            .border_1()
            .border_color(theme::border_hairline())
            // Section Title
            .child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .child("Voice & Audio Settings"),
            )
            // Device Selection Row
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(
                        // Input Device Box
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child("INPUT DEVICE"),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .text_sm()
                                    .text_color(theme::text_primary())
                                    .child(self.state.input_device.clone().unwrap_or_else(|| {
                                        self.input_devices
                                            .first()
                                            .map(|d| d.name.clone())
                                            .unwrap_or_else(|| "Default Microphone".to_string())
                                    })),
                            ),
                    )
                    .child(
                        // Output Device Box
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child("OUTPUT DEVICE"),
                            )
                            .child(
                                div()
                                    .p_2()
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .text_sm()
                                    .text_color(theme::text_primary())
                                    .child(self.state.output_device.clone().unwrap_or_else(|| {
                                        self.output_devices
                                            .first()
                                            .map(|d| d.name.clone())
                                            .unwrap_or_else(|| "Default Headphones".to_string())
                                    })),
                            ),
                    ),
            )
            // Volume Controls Row
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child(format!(
                                        "INPUT VOLUME ({}%)",
                                        (self.state.input_volume * 100.0).round() as u32
                                    )),
                            )
                            .child(
                                div()
                                    .h(px(8.0))
                                    .w_full()
                                    .rounded_full()
                                    .bg(theme::bg_elevated())
                                    .child(
                                        div()
                                            .h_full()
                                            .w(gpui::DefiniteLength::Fraction(
                                                self.state.input_volume / 2.0,
                                            ))
                                            .rounded_full()
                                            .bg(theme::accent()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child(format!(
                                        "OUTPUT VOLUME ({}%)",
                                        (self.state.output_volume * 100.0).round() as u32
                                    )),
                            )
                            .child(
                                div()
                                    .h(px(8.0))
                                    .w_full()
                                    .rounded_full()
                                    .bg(theme::bg_elevated())
                                    .child(
                                        div()
                                            .h_full()
                                            .w(gpui::DefiniteLength::Fraction(
                                                self.state.output_volume / 2.0,
                                            ))
                                            .rounded_full()
                                            .bg(theme::accent()),
                                    ),
                            ),
                    ),
            )
            // Mic Test & Real-time VU Meter Bar
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_secondary())
                            .child("MIC TEST & SENSITIVITY"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            // Test Button
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if self.state.is_testing_mic {
                                        theme::status_dnd()
                                    } else {
                                        theme::accent()
                                    })
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_primary())
                                    .child(mic_btn_text),
                            )
                            // Dynamic VU Meter Bar
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(14.0))
                                    .rounded_full()
                                    .bg(theme::bg_elevated())
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(gpui::DefiniteLength::Fraction(
                                                (vu_percent as f32) / 100.0,
                                            ))
                                            .rounded_full()
                                            .bg(theme::status_online()),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for AudioSettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
