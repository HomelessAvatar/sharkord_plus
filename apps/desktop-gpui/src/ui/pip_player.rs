//! Picture-in-Picture (PiP) Floating Mini Video Player component.
//! Allows users to continue watching active screen shares in a floating mini player
//! docked in the bottom-right corner (fixed 240x135 16:9 ratio) when navigating
//! away to other channels.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// State for the Picture-in-Picture (PiP) floating mini video player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipPlayerState {
    pub streamer_id: u64,
    pub streamer_name: String,
    pub is_visible: bool,
    pub is_audio_muted: bool,
    pub width: f32,
    pub height: f32,
}

impl Default for PipPlayerState {
    fn default() -> Self {
        Self::new(0, String::new())
    }
}

impl PipPlayerState {
    /// Creates a new `PipPlayerState` for a streamer with default 240x135 dimensions.
    pub fn new(streamer_id: u64, streamer_name: String) -> Self {
        Self {
            streamer_id,
            streamer_name,
            is_visible: false,
            is_audio_muted: false,
            width: 240.0,
            height: 135.0,
        }
    }

    /// Makes the PiP player visible.
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// Hides the PiP player.
    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Toggles stream audio mute state.
    pub fn toggle_audio_mute(&mut self) -> bool {
        self.is_audio_muted = !self.is_audio_muted;
        self.is_audio_muted
    }
}

/// View component rendering the floating mini video player docked at the bottom-right corner.
#[derive(Debug, Clone, PartialEq)]
pub struct PipPlayerView {
    pub state: PipPlayerState,
}

impl PipPlayerView {
    /// Creates a new `PipPlayerView` with the given state.
    pub fn new(state: PipPlayerState) -> Self {
        Self { state }
    }

    /// Renders the floating mini video player element.
    pub fn render_element(&self) -> impl IntoElement {
        let is_muted = self.state.is_audio_muted;

        div()
            .absolute()
            .bottom_4()
            .right_4()
            .w(px(self.state.width))
            .h(px(self.state.height))
            .rounded_xl()
            .overflow_hidden()
            .bg(theme::bg_surface())
            .border_1()
            .border_color(theme::border_hairline())
            .shadow_lg()
            .flex()
            .flex_col()
            .justify_between()
            .p_2()
            // Top Header: Streamer badge + Controls
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    // Streamer Header Badge
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(theme::bg_elevated())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .child(
                                div()
                                    .w(px(6.0))
                                    .h(px(6.0))
                                    .rounded_full()
                                    .bg(theme::status_online()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child(self.state.streamer_name.clone()),
                            ),
                    )
                    // Action Buttons: Audio Mute, Expand, Close
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            // Audio Mute Button
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if is_muted {
                                        theme::accent()
                                    } else {
                                        theme::bg_elevated()
                                    })
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .text_xs()
                                    .text_color(theme::text_primary())
                                    .child(if is_muted { "🔇" } else { "🔊" }),
                            )
                            // Expand / Maximize Button
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .text_xs()
                                    .text_color(theme::text_primary())
                                    .child("⛶"),
                            )
                            // Close Button
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("✕"),
                            ),
                    ),
            )
            // Center Canvas Area (mini video placeholder)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .h_full()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("🖥️ Screen Share"),
                    ),
            )
            // Bottom Info Bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child("PiP Mode"),
                    )
                    .child(
                        div()
                            .w(px(6.0))
                            .h(px(6.0))
                            .rounded_full()
                            .bg(theme::accent()),
                    ),
            )
    }
}

impl Render for PipPlayerView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
