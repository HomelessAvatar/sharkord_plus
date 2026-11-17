//! GPU-Accelerated VideoTile Player and Stream Telemetry Popover component.
//! Renders active screen share and video streams with 16:9 letterbox preservation,
//! top streamer badge, live overlay pill, bottom control bar, and telemetry popover.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::voice::video::StreamTelemetryStats;

use crate::theme;

/// Computes dimensions `(width, height)` preserving a 16:9 aspect ratio
/// letterboxed inside a given container bounding box.
pub fn aspect_ratio_box(max_width: f32, max_height: f32) -> (f32, f32) {
    if max_width <= 0.0 || max_height <= 0.0 {
        return (0.0, 0.0);
    }
    const ASPECT_16_9: f32 = 16.0 / 9.0;
    let current_aspect = max_width / max_height;
    if current_aspect > ASPECT_16_9 {
        // Container is wider than 16:9 -> constrain by height
        (max_height * ASPECT_16_9, max_height)
    } else {
        // Container is taller or equal -> constrain by width
        (max_width, max_width / ASPECT_16_9)
    }
}

/// Display mode for the video tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VideoDisplayMode {
    #[default]
    Standard,
    Theater,
    Fullscreen,
}

impl VideoDisplayMode {
    /// Returns the human-readable label for the display mode.
    pub fn label(&self) -> &'static str {
        match self {
            VideoDisplayMode::Standard => "Standard",
            VideoDisplayMode::Theater => "Theater",
            VideoDisplayMode::Fullscreen => "Fullscreen",
        }
    }
}

/// State for an active video stream tile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoTileState {
    pub streamer_id: u64,
    pub streamer_name: String,
    pub display_mode: VideoDisplayMode,
    pub is_fullscreen: bool,
    pub is_audio_muted: bool,
    pub stats: Option<StreamTelemetryStats>,
    pub show_stats_popover: bool,
}

impl Default for VideoTileState {
    fn default() -> Self {
        Self::new(0, String::new())
    }
}

impl VideoTileState {
    /// Creates a new `VideoTileState` with default Standard mode, unmuted audio, and hidden telemetry popover.
    pub fn new(streamer_id: u64, streamer_name: String) -> Self {
        Self {
            streamer_id,
            streamer_name,
            display_mode: VideoDisplayMode::Standard,
            is_fullscreen: false,
            is_audio_muted: false,
            stats: None,
            show_stats_popover: false,
        }
    }

    /// Toggles fullscreen mode and updates display mode accordingly.
    pub fn toggle_fullscreen(&mut self) -> bool {
        self.is_fullscreen = !self.is_fullscreen;
        if self.is_fullscreen {
            self.display_mode = VideoDisplayMode::Fullscreen;
        } else {
            self.display_mode = VideoDisplayMode::Standard;
        }
        self.is_fullscreen
    }

    /// Toggles audio mute state for the stream.
    pub fn toggle_audio_mute(&mut self) -> bool {
        self.is_audio_muted = !self.is_audio_muted;
        self.is_audio_muted
    }

    /// Toggles theater mode display state.
    pub fn toggle_theater(&mut self) -> bool {
        if self.display_mode == VideoDisplayMode::Theater {
            self.display_mode = VideoDisplayMode::Standard;
            self.is_fullscreen = false;
            false
        } else {
            self.display_mode = VideoDisplayMode::Theater;
            self.is_fullscreen = false;
            true
        }
    }

    /// Sets the active display mode.
    pub fn set_display_mode(&mut self, mode: VideoDisplayMode) {
        self.display_mode = mode;
        self.is_fullscreen = matches!(mode, VideoDisplayMode::Fullscreen);
    }

    /// Updates live streaming telemetry statistics.
    pub fn update_stats(&mut self, stats: StreamTelemetryStats) {
        self.stats = Some(stats);
    }

    /// Toggles the visibility of the stream telemetry popover.
    pub fn toggle_stats_popover(&mut self) -> bool {
        self.show_stats_popover = !self.show_stats_popover;
        self.show_stats_popover
    }
}

/// View component rendering the GPU-accelerated video tile canvas, overlay badges, and telemetry popover.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoTileView {
    pub state: VideoTileState,
    pub width: f32,
    pub height: f32,
}

impl Default for VideoTileView {
    fn default() -> Self {
        Self {
            state: VideoTileState::default(),
            width: 800.0,
            height: 450.0,
        }
    }
}

impl VideoTileView {
    /// Creates a new `VideoTileView` initialized with default dimensions.
    pub fn new(state: VideoTileState) -> Self {
        Self {
            state,
            width: 800.0,
            height: 450.0,
        }
    }

    /// Sets explicit rendering dimensions for the viewport container.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    fn telemetry_row(label: &'static str, value: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xs()
                    .text_color(theme::text_secondary())
                    .child(label),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme::text_primary())
                    .child(value.to_string()),
            )
    }

    /// Renders the complete video tile element tree.
    pub fn render_element(&self) -> impl IntoElement {
        let (box_w, box_h) = aspect_ratio_box(self.width, self.height);

        let is_muted = self.state.is_audio_muted;
        let is_fullscreen = self.state.is_fullscreen;
        let is_theater = self.state.display_mode == VideoDisplayMode::Theater;
        let show_popover = self.state.show_stats_popover;

        let resolution_label = self
            .state
            .stats
            .as_ref()
            .map(|s| format!("{}p", s.resolution_height))
            .unwrap_or_else(|| "1080p".to_string());

        let fps_label = self
            .state
            .stats
            .as_ref()
            .map(|s| format!("{} FPS", s.fps))
            .unwrap_or_else(|| "60 FPS".to_string());

        let bitrate_label = self
            .state
            .stats
            .as_ref()
            .map(|s| format!("{:.2} Mbps", s.bitrate_mbps()))
            .unwrap_or_else(|| "N/A".to_string());

        let packet_loss_label = self
            .state
            .stats
            .as_ref()
            .map(|s| format!("{:.2}%", s.packet_loss_percent()))
            .unwrap_or_else(|| "0.0%".to_string());

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w(px(self.width))
            .h(px(self.height))
            .bg(theme::bg_base())
            .child(
                // 16:9 Letterboxed Viewport
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .w(px(box_w))
                    .h(px(box_h))
                    .p_4()
                    .rounded_xl()
                    .overflow_hidden()
                    .bg(theme::bg_surface())
                    .border_1()
                    .border_color(theme::border_hairline())
                    // Top Row: Streamer Badge and Overlay Pill
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .w_full()
                            // Top Streamer Badge (left)
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_3()
                                    .py_1()
                                    .rounded_lg()
                                    .bg(theme::bg_elevated())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .child(
                                        div()
                                            .w(px(8.0))
                                            .h(px(8.0))
                                            .rounded_full()
                                            .bg(theme::status_online()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme::status_online())
                                            .child("LIVE"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme::text_primary())
                                            .child(self.state.streamer_name.clone()),
                                    ),
                            )
                            // Live Resolution / FPS Overlay Pill (right)
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_3()
                                    .py_1()
                                    .rounded_lg()
                                    .bg(theme::bg_elevated())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_secondary())
                                            .child(format!("{resolution_label} {fps_label}")),
                                    ),
                            ),
                    )
                    // Center Canvas Area / Stream Viewport Placeholder
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .flex_1()
                            .gap_2()
                            .child(div().text_3xl().child("🖥️"))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_muted())
                                    .child(format!("Watching {}", self.state.streamer_name)),
                            ),
                    )
                    // Stream Telemetry Popover (Conditionally displayed)
                    .children(if show_popover {
                        Some(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .max_w(px(320.0))
                                .p_3()
                                .mb_2()
                                .rounded_lg()
                                .bg(theme::bg_elevated())
                                .border_1()
                                .border_color(theme::border_hairline())
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme::text_secondary())
                                                .child("STREAM TELEMETRY"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme::status_online())
                                                .child("60 FPS GPU"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(Self::telemetry_row("Bitrate", &bitrate_label))
                                        .child(Self::telemetry_row("Framerate", &fps_label))
                                        .child(Self::telemetry_row("Resolution", &resolution_label))
                                        .child(Self::telemetry_row(
                                            "Packet Loss",
                                            &packet_loss_label,
                                        )),
                                ),
                        )
                    } else {
                        None
                    })
                    // Bottom Control Bar
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .p_2()
                            .rounded_lg()
                            .bg(theme::bg_elevated())
                            .border_1()
                            .border_color(theme::border_hairline())
                            // Left controls: Audio mute & Telemetry stats toggle
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    // Mute Stream Audio Button
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .bg(if is_muted {
                                                theme::status_dnd()
                                            } else {
                                                theme::bg_surface_4()
                                            })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_primary())
                                            .child(if is_muted {
                                                "🔇 Unmute Audio"
                                            } else {
                                                "🔊 Mute Audio"
                                            }),
                                    )
                                    // Telemetry Stats Popover Toggle
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .bg(if show_popover {
                                                theme::accent()
                                            } else {
                                                theme::bg_surface_4()
                                            })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_primary())
                                            .child("📊 Stats"),
                                    ),
                            )
                            // Right controls: Theater, Fullscreen & Stop Watching
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    // Theater Mode Button
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .bg(if is_theater {
                                                theme::accent()
                                            } else {
                                                theme::bg_surface_4()
                                            })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_primary())
                                            .child(if is_theater {
                                                "🔲 Standard"
                                            } else {
                                                "🔲 Theater"
                                            }),
                                    )
                                    // Fullscreen Button
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .bg(if is_fullscreen {
                                                theme::accent()
                                            } else {
                                                theme::bg_surface_4()
                                            })
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme::text_primary())
                                            .child(if is_fullscreen {
                                                "⛶ Windowed"
                                            } else {
                                                "⛶ Fullscreen"
                                            }),
                                    )
                                    // Stop Watching Button
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .px_3()
                                            .py_1()
                                            .rounded_md()
                                            .bg(theme::status_dnd())
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme::text_primary())
                                            .child("✕ Stop Watching"),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for VideoTileView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
