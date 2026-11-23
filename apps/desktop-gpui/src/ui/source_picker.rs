//! Screen & Window Source Picker modal dialog component for GPUI.
//! Allows users to select an entire monitor display or specific application window to share,
//! configure resolution (720p/1080p/1440p/4K) and framerate (15/30/60/120/144 FPS), and toggle audio capture.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::voice::video::{
    ScreenSource, VideoFrameRate, VideoResolution, VideoTrackConfig,
};

use crate::media::screen_capture::{CaptureSource, CaptureSourceType, ScreenCaptureEngine};
use crate::theme;

/// Category tabs for source selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SourcePickerTab {
    #[default]
    Screens,
    Windows,
}

impl SourcePickerTab {
    /// Returns the human-readable tab label.
    pub fn label(&self) -> &'static str {
        match self {
            SourcePickerTab::Screens => "Screens",
            SourcePickerTab::Windows => "Applications",
        }
    }
}

/// State for the screen & application window source picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePickerState {
    pub current_tab: SourcePickerTab,
    pub available_sources: Vec<ScreenSource>,
    pub capture_sources: Vec<CaptureSource>,
    pub selected_source: Option<ScreenSource>,
    pub selected_capture_source: Option<CaptureSource>,
    pub selected_resolution: VideoResolution,
    pub selected_framerate: VideoFrameRate,
    pub capture_window_audio: bool,
}

impl Default for SourcePickerState {
    fn default() -> Self {
        Self {
            current_tab: SourcePickerTab::Screens,
            available_sources: Vec::new(),
            capture_sources: Vec::new(),
            selected_source: None,
            selected_capture_source: None,
            selected_resolution: VideoResolution::R1080p,
            selected_framerate: VideoFrameRate::Fps60,
            capture_window_audio: true,
        }
    }
}

impl SourcePickerState {
    /// Creates a new `SourcePickerState` with default presets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a `SourcePickerState` initialized and populated from `ScreenCaptureEngine`.
    pub fn from_engine() -> Self {
        let mut state = Self::default();
        state.populate_from_engine();
        state
    }

    /// Switches the active picker tab.
    pub fn select_tab(&mut self, tab: SourcePickerTab) {
        self.current_tab = tab;
    }

    /// Sets the currently selected screen or application window source.
    pub fn select_source(&mut self, source: ScreenSource) {
        self.selected_source = Some(source);
    }

    /// Populates available sources directly from `ScreenCaptureEngine::enumerate_sources()`.
    pub fn populate_from_engine(&mut self) {
        let sources = ScreenCaptureEngine::enumerate_sources();
        self.populate_from_capture_sources(sources);
    }

    /// Alias for `populate_from_engine`.
    pub fn refresh_sources(&mut self) {
        self.populate_from_engine();
    }

    /// Populates capture sources and synchronizes `available_sources`.
    pub fn populate_from_capture_sources(&mut self, sources: Vec<CaptureSource>) {
        self.capture_sources = sources.clone();
        self.available_sources = sources
            .into_iter()
            .enumerate()
            .map(|(idx, cs)| match cs.source_type {
                CaptureSourceType::Monitor => ScreenSource::Monitor {
                    id: cs
                        .id
                        .strip_prefix("monitor:")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(idx as u64 + 1),
                    name: cs.name,
                },
                CaptureSourceType::Window => ScreenSource::Window {
                    id: cs
                        .id
                        .strip_prefix("window:")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(idx as u64 + 1000),
                    title: cs.name,
                },
            })
            .collect();
    }

    /// Selects an active `CaptureSource`.
    pub fn select_capture_source(&mut self, source: CaptureSource) {
        let screen_source = match source.source_type {
            CaptureSourceType::Monitor => ScreenSource::Monitor {
                id: source
                    .id
                    .strip_prefix("monitor:")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1),
                name: source.name.clone(),
            },
            CaptureSourceType::Window => ScreenSource::Window {
                id: source
                    .id
                    .strip_prefix("window:")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1001),
                title: source.name.clone(),
            },
        };
        self.selected_capture_source = Some(source);
        self.selected_source = Some(screen_source);
    }

    /// Returns the capture sources filtered by the current tab selection.
    pub fn current_tab_capture_sources(&self) -> Vec<&CaptureSource> {
        self.capture_sources
            .iter()
            .filter(|source| match self.current_tab {
                SourcePickerTab::Screens => source.is_monitor(),
                SourcePickerTab::Windows => source.is_window(),
            })
            .collect()
    }

    /// Sets the desired target encoding resolution.
    pub fn set_resolution(&mut self, resolution: VideoResolution) {
        self.selected_resolution = resolution;
    }

    /// Sets the desired target encoding framerate.
    pub fn set_framerate(&mut self, framerate: VideoFrameRate) {
        self.selected_framerate = framerate;
    }

    /// Toggles the window/application audio capture setting. Returns the updated boolean.
    pub fn toggle_audio(&mut self) -> bool {
        self.capture_window_audio = !self.capture_window_audio;
        self.capture_window_audio
    }

    /// Explicitly sets the window/application audio capture flag.
    pub fn set_capture_audio(&mut self, capture: bool) {
        self.capture_window_audio = capture;
    }

    /// Constructs the final `VideoTrackConfig` if a valid source has been selected.
    pub fn build_track_config(&self) -> Option<VideoTrackConfig> {
        let source = self.selected_source.clone().or_else(|| {
            self.selected_capture_source
                .as_ref()
                .map(|cs| match cs.source_type {
                    CaptureSourceType::Monitor => ScreenSource::Monitor {
                        id: cs
                            .id
                            .strip_prefix("monitor:")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1),
                        name: cs.name.clone(),
                    },
                    CaptureSourceType::Window => ScreenSource::Window {
                        id: cs
                            .id
                            .strip_prefix("window:")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1001),
                        title: cs.name.clone(),
                    },
                })
        })?;

        Some(VideoTrackConfig {
            source,
            resolution: self.selected_resolution,
            framerate: self.selected_framerate,
            with_audio: self.capture_window_audio,
        })
    }

    /// Returns the available sources filtered by the current tab selection.
    pub fn current_tab_sources(&self) -> Vec<&ScreenSource> {
        self.available_sources
            .iter()
            .filter(|source| match self.current_tab {
                SourcePickerTab::Screens => source.is_monitor(),
                SourcePickerTab::Windows => source.is_window(),
            })
            .collect()
    }
}

/// View component rendering the screen/window source picker modal.
#[derive(Debug, Clone, PartialEq)]
pub struct SourcePickerView {
    pub state: SourcePickerState,
}

impl SourcePickerView {
    /// Available resolution presets displayed in the source picker.
    pub const AVAILABLE_RESOLUTIONS: &'static [VideoResolution] = &[
        VideoResolution::R720p,
        VideoResolution::R1080p,
        VideoResolution::R1440p,
        VideoResolution::R4K,
    ];

    /// Available framerate presets displayed in the source picker.
    pub const AVAILABLE_FRAMERATES: &'static [VideoFrameRate] = &[
        VideoFrameRate::Fps15,
        VideoFrameRate::Fps30,
        VideoFrameRate::Fps60,
        VideoFrameRate::Fps120,
        VideoFrameRate::Fps144,
    ];

    /// Creates a new `SourcePickerView` with the provided state.
    pub fn new(state: SourcePickerState) -> Self {
        Self { state }
    }

    /// Renders the modal dialog element.
    pub fn render_element(&self) -> impl IntoElement {
        let current_tab = self.state.current_tab;
        let use_capture_sources = !self.state.capture_sources.is_empty();
        let selected_capture_id = self.state.selected_capture_source.as_ref().map(|s| &s.id);
        let selected_id = self.state.selected_source.as_ref().map(|s| s.id());

        let resolutions = Self::AVAILABLE_RESOLUTIONS;
        let framerates = Self::AVAILABLE_FRAMERATES;

        div()
            .flex()
            .flex_col()
            .w(px(580.0))
            .p_6()
            .bg(theme::bg_surface())
            .rounded_2xl()
            .border_1()
            .border_color(theme::border_hairline())
            .gap_5()
            // Header
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child("Share Your Screen"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_secondary())
                            .child("Select a display or window to stream to the voice channel."),
                    ),
            )
            // Tab Bar (Screens vs Applications)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .p_1()
                    .rounded_lg()
                    .bg(theme::bg_elevated())
                    .child(
                        div()
                            .flex_1()
                            .py_1p5()
                            .text_center()
                            .rounded_md()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .bg(if current_tab == SourcePickerTab::Screens {
                                theme::bg_surface_4()
                            } else {
                                hsla(0.0, 0.0, 0.0, 0.0)
                            })
                            .text_color(if current_tab == SourcePickerTab::Screens {
                                theme::text_primary()
                            } else {
                                theme::text_secondary()
                            })
                            .child(SourcePickerTab::Screens.label()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .py_1p5()
                            .text_center()
                            .rounded_md()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .bg(if current_tab == SourcePickerTab::Windows {
                                theme::bg_surface_4()
                            } else {
                                hsla(0.0, 0.0, 0.0, 0.0)
                            })
                            .text_color(if current_tab == SourcePickerTab::Windows {
                                theme::text_primary()
                            } else {
                                theme::text_secondary()
                            })
                            .child(SourcePickerTab::Windows.label()),
                    ),
            )
            // Source Grid / List
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .max_h(px(220.0))
                    .overflow_hidden()
                    .children(if use_capture_sources {
                        let capture_sources = self.state.current_tab_capture_sources();
                        if capture_sources.is_empty() {
                            vec![div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .w_full()
                                .py_8()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme::text_secondary())
                                        .child(match current_tab {
                                            SourcePickerTab::Screens => "No display screens detected",
                                            SourcePickerTab::Windows => {
                                                "No application windows available"
                                            }
                                        }),
                                )
                                .into_any_element()]
                        } else {
                            capture_sources
                                .into_iter()
                                .map(|source| {
                                    let is_selected = selected_capture_id == Some(&source.id);
                                    let icon = if source.is_monitor() { "🖥️" } else { "🪟" };

                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .w(px(160.0))
                                        .h(px(100.0))
                                        .p_2()
                                        .rounded_lg()
                                        .bg(if is_selected {
                                            theme::bg_surface_4()
                                        } else {
                                            theme::bg_elevated()
                                        })
                                        .border_2()
                                        .border_color(if is_selected {
                                            theme::accent()
                                        } else {
                                            theme::border_hairline()
                                        })
                                        .gap_1()
                                        .child(div().text_2xl().child(icon))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme::text_primary())
                                                .child(source.name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child(format!("{}x{}", source.width, source.height)),
                                        )
                                        .into_any_element()
                                })
                                .collect()
                        }
                    } else {
                        let sources = self.state.current_tab_sources();
                        if sources.is_empty() {
                            vec![div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .w_full()
                                .py_8()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme::text_secondary())
                                        .child(match current_tab {
                                            SourcePickerTab::Screens => "No display screens detected",
                                            SourcePickerTab::Windows => {
                                                "No application windows available"
                                            }
                                        }),
                                )
                                .into_any_element()]
                        } else {
                            sources
                                .into_iter()
                                .map(|source| {
                                    let is_selected = selected_id == Some(source.id());
                                    let icon = if source.is_monitor() {
                                        "🖥️"
                                    } else {
                                        "🪟"
                                    };

                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .w(px(160.0))
                                        .h(px(100.0))
                                        .p_2()
                                        .rounded_lg()
                                        .bg(if is_selected {
                                            theme::bg_surface_4()
                                        } else {
                                            theme::bg_elevated()
                                        })
                                        .border_2()
                                        .border_color(if is_selected {
                                            theme::accent()
                                        } else {
                                            theme::border_hairline()
                                        })
                                        .gap_2()
                                        .child(div().text_2xl().child(icon))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme::text_primary())
                                                .child(source.title().to_string()),
                                        )
                                        .into_any_element()
                                })
                                .collect()
                        }
                    }),
            )
            // Stream Settings (Resolution & Framerate)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_3()
                    .rounded_lg()
                    .bg(theme::bg_elevated())
                    // Resolution selector row
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("RESOLUTION"),
                            )
                            .child(div().flex().flex_row().gap_2().children(
                                resolutions.iter().map(|&res| {
                                    let is_active = self.state.selected_resolution == res;
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .bg(if is_active {
                                            theme::accent()
                                        } else {
                                            theme::bg_surface()
                                        })
                                        .text_color(if is_active {
                                            theme::text_primary()
                                        } else {
                                            theme::text_secondary()
                                        })
                                        .child(res.label())
                                }),
                            )),
                    )
                    // Framerate selector row
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("FRAME RATE"),
                            )
                            .child(div().flex().flex_row().gap_2().children(
                                framerates.iter().map(|&fps| {
                                    let is_active = self.state.selected_framerate == fps;
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .bg(if is_active {
                                            theme::accent()
                                        } else {
                                            theme::bg_surface()
                                        })
                                        .text_color(if is_active {
                                            theme::text_primary()
                                        } else {
                                            theme::text_secondary()
                                        })
                                        .child(fps.label())
                                }),
                            )),
                    ),
            )
            // Audio Capture Toggle
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_primary())
                                    .child("Share System / Application Audio"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Stream sounds played by the selected source."),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .bg(if self.state.capture_window_audio {
                                theme::status_online()
                            } else {
                                theme::bg_elevated()
                            })
                            .text_color(theme::text_primary())
                            .child(if self.state.capture_window_audio {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }),
                    ),
            )
            // Action Buttons (Cancel / Go Live)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .gap_3()
                    .pt_2()
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_secondary())
                            .child("Cancel"),
                    )
                    .child(
                        div()
                            .px_5()
                            .py_2()
                            .rounded_md()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .bg(if self.state.selected_source.is_some() || self.state.selected_capture_source.is_some() {
                                theme::accent()
                            } else {
                                theme::bg_elevated()
                            })
                            .text_color(if self.state.selected_source.is_some() || self.state.selected_capture_source.is_some() {
                                theme::text_primary()
                            } else {
                                theme::text_muted()
                            })
                            .child("Go Live"),
                    ),
            )
    }
}

impl Render for SourcePickerView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
