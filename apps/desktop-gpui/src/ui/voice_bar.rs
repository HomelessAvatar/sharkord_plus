//! Docked Voice Bar view component (bg_surface).
//! Renders the connection indicator (green dot, channel name, latency),
//! and Mute, Deafen, and Disconnect control buttons docked at the bottom of the channel panel.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// Voice connection state for the active voice session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceConnectionState {
    Disconnected,
    Connecting,
    Connected {
        channel_id: u64,
        channel_name: String,
        ping_ms: u32,
    },
}

/// Returns the status indicator color (hex u32) for the connection state.
pub fn voice_status_indicator_color(state: &VoiceConnectionState) -> u32 {
    match state {
        VoiceConnectionState::Connected { .. } => theme::STATUS_ONLINE,
        VoiceConnectionState::Connecting => theme::STATUS_IDLE,
        VoiceConnectionState::Disconnected => theme::STATUS_OFFLINE,
    }
}

/// Returns the human-readable status text for the voice connection.
pub fn voice_status_text(state: &VoiceConnectionState) -> String {
    match state {
        VoiceConnectionState::Connected {
            channel_name,
            ping_ms,
            ..
        } => format!("Connected / {} ({}ms)", channel_name, ping_ms),
        VoiceConnectionState::Connecting => "Connecting...".to_string(),
        VoiceConnectionState::Disconnected => "Disconnected".to_string(),
    }
}

/// Returns the toggle action labels for mute and deafen buttons based on their current states.
pub fn voice_action_labels(is_muted: bool, is_deafened: bool) -> (&'static str, &'static str) {
    (
        if is_muted { "Unmute" } else { "Mute" },
        if is_deafened { "Undeafen" } else { "Deafen" },
    )
}

/// State for the voice bar controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceBarState {
    pub connection: VoiceConnectionState,
    pub is_muted: bool,
    pub is_deafened: bool,
}

impl Default for VoiceBarState {
    fn default() -> Self {
        Self {
            connection: VoiceConnectionState::Disconnected,
            is_muted: false,
            is_deafened: false,
        }
    }
}

impl VoiceBarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    pub fn toggle_deafen(&mut self) {
        self.is_deafened = !self.is_deafened;
    }

    pub fn disconnect(&mut self) {
        self.connection = VoiceConnectionState::Disconnected;
    }
}

/// Voice Bar view component.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceBarView {
    pub state: VoiceBarState,
}

impl Default for VoiceBarView {
    fn default() -> Self {
        Self {
            state: VoiceBarState::new(),
        }
    }
}

impl VoiceBarView {
    pub fn new(state: VoiceBarState) -> Self {
        Self { state }
    }

    /// Renders the docked Voice Bar element tree.
    pub fn render_element(&self) -> impl IntoElement {
        let (mute_label, deafen_label) =
            voice_action_labels(self.state.is_muted, self.state.is_deafened);
        let dot_color = voice_status_indicator_color(&self.state.connection);
        let status_desc = voice_status_text(&self.state.connection);

        div()
            .flex()
            .flex_col()
            .w_full()
            .p_2()
            .bg(theme::bg_surface())
            .border_t_1()
            .border_color(theme::border_hairline())
            // Top row: Connection status indicator
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_1()
                    .py_1()
                    // Status dot
                    .child(
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(rgb(dot_color)),
                    )
                    // Status description
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(
                                if self.state.connection != VoiceConnectionState::Disconnected {
                                    theme::status_online()
                                } else {
                                    theme::text_muted()
                                },
                            )
                            .child(status_desc),
                    ),
            )
            // Bottom row: Mute, Deafen, Disconnect action buttons
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .mt_1()
                    .gap_1()
                    // Mute button
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_1()
                            .py_1()
                            .rounded_md()
                            .bg(if self.state.is_muted {
                                theme::bg_surface_4()
                            } else {
                                theme::bg_elevated()
                            })
                            .text_xs()
                            .text_color(if self.state.is_muted {
                                theme::status_dnd()
                            } else {
                                theme::text_primary()
                            })
                            .child(mute_label),
                    )
                    // Deafen button
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_1()
                            .py_1()
                            .rounded_md()
                            .bg(if self.state.is_deafened {
                                theme::bg_surface_4()
                            } else {
                                theme::bg_elevated()
                            })
                            .text_xs()
                            .text_color(if self.state.is_deafened {
                                theme::status_dnd()
                            } else {
                                theme::text_primary()
                            })
                            .child(deafen_label),
                    )
                    // Disconnect button
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(theme::bg_elevated())
                            .text_xs()
                            .text_color(theme::status_dnd())
                            .child("✕"),
                    ),
            )
    }
}

impl Render for VoiceBarView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
