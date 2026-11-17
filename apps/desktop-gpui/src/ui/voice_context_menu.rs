//! Participant context menu and controls component.
//! Provides per-user volume adjustments (0% - 200%) and local mute ("Mute for me")
//! according to the Pure Neutral Dark design specification.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// State for an individual participant's context menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceParticipantMenuState {
    pub user_id: u64,
    pub username: String,
    pub volume: f32,
    pub is_local_muted: bool,
}

impl VoiceParticipantMenuState {
    /// Creates a new `VoiceParticipantMenuState` with default volume (100%) and unmuted.
    pub fn new(user_id: u64, username: String) -> Self {
        Self {
            user_id,
            username,
            volume: 1.0,
            is_local_muted: false,
        }
    }

    /// Sets the volume multiplier, clamped between 0.0 (0%) and 2.0 (200%).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0);
    }

    /// Returns the volume percentage as an integer (e.g. 100 for 1.0, 150 for 1.5).
    pub fn volume_percentage(&self) -> u32 {
        (self.volume * 100.0).round() as u32
    }

    /// Toggles the local mute state ("Mute for me"). Returns the new muted state.
    pub fn toggle_local_mute(&mut self) -> bool {
        self.is_local_muted = !self.is_local_muted;
        self.is_local_muted
    }

    /// Sets the local mute state explicitly.
    pub fn set_local_mute(&mut self, is_local_muted: bool) {
        self.is_local_muted = is_local_muted;
    }
}

/// View component rendering the participant context menu.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceParticipantMenuView {
    pub state: VoiceParticipantMenuState,
}

impl VoiceParticipantMenuView {
    /// Creates a new `VoiceParticipantMenuView`.
    pub fn new(state: VoiceParticipantMenuState) -> Self {
        Self { state }
    }

    /// Renders the participant context menu element.
    pub fn render_element(&self) -> impl IntoElement {
        let initial = self
            .state
            .username
            .chars()
            .next()
            .unwrap_or('?')
            .to_string();
        let volume_percent = self.state.volume_percentage();
        let fill_fraction = (self.state.volume / 2.0).clamp(0.0, 1.0);

        div()
            .flex()
            .flex_col()
            .w(px(260.0))
            .p_3()
            .rounded_xl()
            .bg(theme::bg_surface())
            .border_1()
            .border_color(theme::border_hairline())
            .gap_3()
            // Header: User identity
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(36.0))
                            .h(px(36.0))
                            .rounded_full()
                            .bg(theme::bg_surface_4())
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(initial),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child(self.state.username.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Participant Controls"),
                            ),
                    ),
            )
            // Volume Slider Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .rounded_lg()
                    .bg(theme::bg_subsurface())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child("USER VOLUME"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(if self.state.is_local_muted {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(format!("{volume_percent}%")),
                            ),
                    )
                    // Volume Bar Indicator
                    .child(
                        div()
                            .h(px(8.0))
                            .w_full()
                            .rounded_full()
                            .bg(theme::bg_elevated())
                            .child(
                                div()
                                    .h_full()
                                    .w(gpui::DefiniteLength::Fraction(fill_fraction))
                                    .rounded_full()
                                    .bg(if self.state.is_local_muted {
                                        theme::text_muted()
                                    } else {
                                        theme::accent()
                                    }),
                            ),
                    ),
            )
            // Local Mute ("Mute for me") Section
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .rounded_lg()
                    .bg(if self.state.is_local_muted {
                        theme::status_dnd()
                    } else {
                        theme::bg_subsurface()
                    })
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_primary())
                            .child(if self.state.is_local_muted {
                                "Muted for me"
                            } else {
                                "Mute for me"
                            }),
                    )
                    .child(div().text_xs().child(if self.state.is_local_muted {
                        "🔇"
                    } else {
                        "🔊"
                    })),
            )
    }
}
