//! Discord-style Voice Stage / Grid View component.
//! Renders active voice participants in a responsive grid with avatar cards,
//! glowing green speaking indicator rings, and status badges.

use gpui::prelude::*;
use gpui::*;

use crate::theme;

/// Computes the optimal grid column count based on the number of participants.
pub fn calculate_grid_columns(count: usize) -> usize {
    match count {
        0..=1 => 1,
        2..=4 => 2,
        5..=9 => 3,
        10..=16 => 4,
        _ => 5,
    }
}

/// Data representation for an individual participant card on the voice stage.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceStageCard {
    pub user_id: u64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub is_speaking: bool,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub volume: f32,
}

impl VoiceStageCard {
    /// Returns the active border color (green) if currently speaking, or None.
    pub fn border_color(&self) -> Option<u32> {
        if self.is_speaking {
            Some(theme::STATUS_ONLINE)
        } else {
            None
        }
    }

    /// Renders an individual participant card element.
    pub fn render_card(&self) -> impl IntoElement {
        let border_color = self.border_color();
        let initial = self.username.chars().next().unwrap_or('?').to_string();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p_4()
            .rounded_xl()
            .bg(theme::bg_surface())
            .border_2()
            .border_color(if let Some(c) = border_color {
                rgb(c).into()
            } else {
                theme::border_hairline()
            })
            .gap_3()
            // Avatar with animated speaking border ring
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(72.0))
                    .h(px(72.0))
                    .rounded_full()
                    .bg(theme::bg_surface_4())
                    .border_3()
                    .border_color(if let Some(c) = border_color {
                        rgb(c).into()
                    } else {
                        hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child(initial),
            )
            // Username and status pill
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child(self.username.clone()),
                    )
                    .children(if self.is_deafened {
                        Some(div().text_xs().text_color(theme::status_dnd()).child("🎧"))
                    } else if self.is_muted {
                        Some(div().text_xs().text_color(theme::status_dnd()).child("🔇"))
                    } else {
                        None
                    }),
            )
    }
}

/// Voice Stage view component representing the primary voice channel stage room.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VoiceStageView {
    pub channel_id: u64,
    pub channel_name: String,
    pub participants: Vec<VoiceStageCard>,
}

impl VoiceStageView {
    pub fn new(
        channel_id: u64,
        channel_name: impl Into<String>,
        participants: Vec<VoiceStageCard>,
    ) -> Self {
        Self {
            channel_id,
            channel_name: channel_name.into(),
            participants,
        }
    }

    /// Renders the voice stage layout.
    pub fn render_element(&self) -> impl IntoElement {
        let cols = calculate_grid_columns(self.participants.len());

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(theme::bg_base())
            .p_4()
            .gap_4()
            // Top Stage Header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .pb_2()
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme::status_online())
                                    .child("🔊"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child(self.channel_name.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(format!("{} in call", self.participants.len())),
                    ),
            )
            // Center Stage Grid
            .child(
                div()
                    .id("voice_stage_grid")
                    .flex()
                    .flex_wrap()
                    .gap_4()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.participants.iter().map(|p| {
                        div()
                            .w(match cols {
                                1 => px(360.0),
                                2 => px(280.0),
                                3 => px(220.0),
                                _ => px(180.0),
                            })
                            .child(p.render_card())
                    })),
            )
    }
}

impl Render for VoiceStageView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
