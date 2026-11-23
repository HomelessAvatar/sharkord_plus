//! Image Lightbox Modal dialog and zoom controls in GPUI.
//! Implements full-screen dark modal overlay with image viewport, zoom controls,
//! download button, and keyboard navigation hints adhering strictly to Pure Neutral Dark.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// Renders a keyboard shortcut badge (Kbd).
fn render_kbd(key: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .bg(theme::bg_surface_4())
        .border_1()
        .border_color(theme::border_hairline())
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(theme::text_muted())
        .child(key.to_string())
}

/// State for the image lightbox modal dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightboxState {
    pub is_open: bool,
    pub image_url: Option<String>,
    pub filename: Option<String>,
    pub zoom_level: f32,
}

impl Default for LightboxState {
    fn default() -> Self {
        Self::new()
    }
}

impl LightboxState {
    /// Creates a new closed `LightboxState` at 1.0 (100%) zoom.
    pub fn new() -> Self {
        Self {
            is_open: false,
            image_url: None,
            filename: None,
            zoom_level: 1.0,
        }
    }

    /// Opens the lightbox with the specified image URL at 100% zoom.
    pub fn open(&mut self, url: impl Into<String>) {
        self.is_open = true;
        self.image_url = Some(url.into());
        self.filename = None;
        self.zoom_level = 1.0;
    }

    /// Opens the lightbox with image URL and original filename.
    pub fn open_with_name(&mut self, url: impl Into<String>, name: impl Into<String>) {
        self.is_open = true;
        self.image_url = Some(url.into());
        self.filename = Some(name.into());
        self.zoom_level = 1.0;
    }

    /// Closes the lightbox and resets zoom and image references.
    pub fn close(&mut self) {
        self.is_open = false;
        self.image_url = None;
        self.filename = None;
        self.zoom_level = 1.0;
    }

    /// Zooms in by +25% up to 500%.
    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + 0.25).min(5.0);
    }

    /// Zooms out by -25% down to 25%.
    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - 0.25).max(0.25);
    }

    /// Resets zoom level back to 1.0 (100%).
    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
    }

    /// Returns human-readable zoom percentage string (e.g. "100%", "125%").
    pub fn zoom_percentage_text(&self) -> String {
        format!("{:.0}%", self.zoom_level * 100.0)
    }
}

/// Full-screen image lightbox modal view overlay.
#[derive(Debug, Clone, PartialEq)]
pub struct LightboxModalView {
    pub state: LightboxState,
}

impl LightboxModalView {
    /// Creates a new `LightboxModalView` with the given state.
    pub fn new(state: LightboxState) -> Self {
        Self { state }
    }

    /// Renders the complete lightbox modal element tree.
    pub fn render_element(&self) -> impl IntoElement {
        let filename_text = self
            .state
            .filename
            .clone()
            .unwrap_or_else(|| "Image Preview".to_string());
        let url_text = self.state.image_url.clone().unwrap_or_default();
        let zoom_text = self.state.zoom_percentage_text();

        // Calculate viewport dimensions based on zoom level
        let base_width = 640.0;
        let base_height = 420.0;
        let zoomed_w = (base_width * self.state.zoom_level).max(160.0);
        let zoomed_h = (base_height * self.state.zoom_level).max(120.0);

        div()
            .id("image_lightbox_overlay")
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .bg(hsla(0.0, 0.0, 0.0, 0.88))
            .flex()
            .flex_col()
            .justify_between()
            // 1. Header Control Bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(52.0))
                    .px_5()
                    .bg(theme::bg_surface())
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    // Left: Filename & icon
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2p5()
                            .child(
                                // Clean vector image icon
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(24.0))
                                    .h(px(24.0))
                                    .rounded_md()
                                    .bg(theme::bg_surface_4())
                                    .text_xs()
                                    .text_color(theme::accent())
                                    .child("🖼"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child(filename_text),
                            ),
                    )
                    // Center: Zoom controls
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            // Zoom Out (-)
                            .child(
                                div()
                                    .id("lightbox_zoom_out_btn")
                                    .cursor_pointer()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_surface_4())
                                    .hover(|s| s.bg(theme::bg_elevated()))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_secondary())
                                    .child("−"),
                            )
                            // Zoom % Badge
                            .child(
                                div()
                                    .id("lightbox_zoom_badge")
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_primary())
                                    .child(zoom_text),
                            )
                            // Zoom In (+)
                            .child(
                                div()
                                    .id("lightbox_zoom_in_btn")
                                    .cursor_pointer()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_surface_4())
                                    .hover(|s| s.bg(theme::bg_elevated()))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_secondary())
                                    .child("+"),
                            )
                            // Reset (1:1)
                            .child(
                                div()
                                    .id("lightbox_zoom_reset_btn")
                                    .cursor_pointer()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_surface_4())
                                    .hover(|s| s.bg(theme::bg_elevated()))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_muted())
                                    .child("1:1 Reset"),
                            ),
                    )
                    // Right: Actions (Download & Close)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .id("lightbox_download_btn")
                                    .cursor_pointer()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1p5()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_surface_4())
                                    .hover(|s| s.bg(theme::bg_elevated()))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_primary())
                                    .child("Download"),
                            )
                            .child(
                                div()
                                    .id("lightbox_close_btn")
                                    .cursor_pointer()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1p5()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme::bg_elevated())
                                    .hover(|s| s.bg(theme::bg_surface_4()))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::text_secondary())
                                    .child("Close")
                                    .child(render_kbd("ESC")),
                            ),
                    ),
            )
            // 2. Centered Zoomable Image Viewport
            .child(
                div()
                    .id("lightbox_image_viewport")
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .p_6()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .w(px(zoomed_w))
                            .h(px(zoomed_h))
                            .rounded_xl()
                            .bg(theme::bg_elevated())
                            .border_1()
                            .border_color(theme::border_subtle())
                            .shadow_xl()
                            .p_6()
                            .gap_3()
                            // Inner Image Display / Placeholder
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .flex_1()
                                    .w_full()
                                    .rounded_lg()
                                    .bg(theme::bg_base())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .p_4()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_3xl()
                                            .child("🖼"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_muted())
                                            .child(url_text),
                                    ),
                            ),
                    ),
            )
            // 3. Bottom Keyboard Hint Footer
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .h(px(36.0))
                    .px_4()
                    .bg(theme::bg_surface())
                    .border_t_1()
                    .border_color(theme::border_hairline())
                    .text_xs()
                    .text_color(theme::text_muted())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(render_kbd("ESC"))
                            .child(div().child("to close")),
                    )
                    .child(div().child("•"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(render_kbd("+"))
                            .child(div().child("/"))
                            .child(render_kbd("−"))
                            .child(div().child("to zoom")),
                    )
                    .child(div().child("•"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
                            .child(render_kbd("0"))
                            .child(div().child("to reset")),
                    ),
            )
    }
}

impl Render for LightboxModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
