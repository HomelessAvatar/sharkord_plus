//! Quick Switcher (`Ctrl+K`) command palette modal component for fast channel navigation.
//! Implements a Raycast/Linear-style 560px centered modal overlay with search input,
//! channel type icon indicators, and keyboard selection highlight.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::{Channel, ChannelType};

use crate::theme;

/// An item displayed in the quick switcher results list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickSwitcherItem {
    pub id: u64,
    pub name: String,
    pub channel_type: ChannelType,
    pub category: Option<String>,
}

/// State for the quick switcher modal dialog.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct QuickSwitcherState {
    pub items: Vec<QuickSwitcherItem>,
    pub query: String,
    pub selected_index: usize,
}

impl QuickSwitcherState {
    /// Creates a new `QuickSwitcherState` initialized from a channel list.
    pub fn new(channels: &[Channel]) -> Self {
        let items = channels
            .iter()
            .map(|c| QuickSwitcherItem {
                id: c.id,
                name: c.name.clone(),
                channel_type: c.channel_type,
                category: None,
            })
            .collect();

        Self {
            items,
            query: String::new(),
            selected_index: 0,
        }
    }

    /// Updates the available items from a new channel list while maintaining valid selection.
    pub fn set_channels(&mut self, channels: &[Channel]) {
        self.items = channels
            .iter()
            .map(|c| QuickSwitcherItem {
                id: c.id,
                name: c.name.clone(),
                channel_type: c.channel_type,
                category: None,
            })
            .collect();
        self.clamp_selection();
    }

    /// Sets the search filter query and resets selected index to the first result.
    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.selected_index = 0;
    }

    /// Returns the list of items matching the current search query.
    pub fn filtered_items(&self) -> Vec<QuickSwitcherItem> {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            return self.items.clone();
        }

        self.items
            .iter()
            .filter(|item| {
                item.name.to_lowercase().contains(&q)
                    || item
                        .category
                        .as_deref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Moves the selection cursor to the next item, clamped at the list end.
    pub fn select_next(&mut self) {
        let count = self.filtered_items().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index + 1 < count {
            self.selected_index += 1;
        }
    }

    /// Moves the selection cursor to the previous item, clamped at index 0.
    pub fn select_prev(&mut self) {
        let count = self.filtered_items().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Returns a copy of the currently selected item, if any.
    pub fn selected_item(&self) -> Option<QuickSwitcherItem> {
        let filtered = self.filtered_items();
        filtered.get(self.selected_index).cloned()
    }

    fn clamp_selection(&mut self) {
        let count = self.filtered_items().len();
        if count == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= count {
            self.selected_index = count - 1;
        }
    }
}

/// View component rendering the Quick Switcher centered modal overlay.
#[derive(Debug, Clone, PartialEq)]
pub struct QuickSwitcherView {
    pub state: QuickSwitcherState,
}

impl QuickSwitcherView {
    /// Creates a new `QuickSwitcherView` with the given state.
    pub fn new(state: QuickSwitcherState) -> Self {
        Self { state }
    }

    /// Renders the modal overlay element.
    pub fn render_element(&self) -> impl IntoElement {
        let filtered = self.state.filtered_items();
        let is_placeholder = self.state.query.is_empty();
        let query_text = if is_placeholder {
            "Type a channel name to jump to..."
        } else {
            &self.state.query
        };

        let items_elements: Vec<AnyElement> = if filtered.is_empty() {
            vec![div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .py_8()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::text_secondary())
                        .child("No matching channels found"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child("Try searching with a different name"),
                )
                .into_any_element()]
        } else {
            filtered
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let is_selected = idx == self.state.selected_index;
                    let icon_str = match item.channel_type {
                        ChannelType::Text => "#",
                        ChannelType::Voice => "🔊",
                    };

                    let bg_color = if is_selected {
                        theme::bg_surface_4()
                    } else {
                        hsla(0.0, 0.0, 0.0, 0.0)
                    };

                    let text_color = if is_selected {
                        theme::text_primary()
                    } else {
                        theme::text_secondary()
                    };

                    let mut item_row = div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(bg_color)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme::text_muted())
                                        .child(icon_str),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(if is_selected {
                                            gpui::FontWeight::SEMIBOLD
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        })
                                        .text_color(text_color)
                                        .child(item.name.clone()),
                                ),
                        );

                    if let Some(ref cat) = item.category {
                        item_row = item_row.child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(cat.clone()),
                        );
                    } else {
                        item_row =
                            item_row.child(div().text_xs().text_color(theme::text_muted()).child(
                                match item.channel_type {
                                    ChannelType::Text => "text",
                                    ChannelType::Voice => "voice",
                                },
                            ));
                    }

                    item_row.into_any_element()
                })
                .collect()
        };

        // Absolute full-screen backdrop with centered 560px modal dialog
        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.65))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(560.0))
                    .max_h(px(440.0))
                    .bg(theme::bg_elevated())
                    .border_1()
                    .border_color(theme::border_subtle())
                    .rounded_xl()
                    .shadow_lg()
                    .overflow_hidden()
                    // Search bar header
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .px_4()
                            .py_3()
                            .border_b_1()
                            .border_color(theme::border_hairline())
                            .gap_3()
                            .child(div().text_sm().text_color(theme::text_muted()).child("🔍"))
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(if is_placeholder {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(query_text.to_string()),
                            )
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(theme::bg_surface())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("ESC"),
                            ),
                    )
                    // Results list
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p_2()
                            .gap_0p5()
                            .max_h(px(320.0))
                            .overflow_hidden()
                            .children(items_elements),
                    )
                    // Footer navigation hints
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_2()
                            .bg(theme::bg_surface())
                            .border_t_1()
                            .border_color(theme::border_hairline())
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .px_1()
                                                    .rounded_sm()
                                                    .bg(theme::bg_elevated())
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("↑↓"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("Navigate"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .px_1()
                                                    .rounded_sm()
                                                    .bg(theme::bg_elevated())
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("↵"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("Select"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(format!("{} channels", filtered.len())),
                            ),
                    ),
            )
    }
}
