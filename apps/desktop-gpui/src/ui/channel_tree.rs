//! Channel Tree sidebar view component (240px fixed width, bg_subsurface).
//! Renders the server header, categorized text and voice channels, active channel
//! selection highlights, and unread badges.

use gpui::prelude::*;
use gpui::*;
use sharkord_core::types::{Channel, ChannelType};

use crate::theme;

/// Returns the display icon for a given channel type.
pub fn channel_icon(channel_type: ChannelType) -> &'static str {
    match channel_type {
        ChannelType::Text => "#",
        ChannelType::Voice => "🔊",
    }
}

/// Channel item prepared for display in the channel tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelDisplayItem {
    pub channel: Channel,
    pub is_active: bool,
    pub has_unread: bool,
}

/// A group of channels under a named category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorizedChannelGroup {
    pub category_name: String,
    pub channels: Vec<ChannelDisplayItem>,
}

/// Composes and categorizes a flat list of channels for hierarchical sidebar display.
pub fn compose_channel_tree(
    channels: &[Channel],
    active_channel_id: Option<u64>,
) -> Vec<CategorizedChannelGroup> {
    if channels.is_empty() {
        return Vec::new();
    }

    let mut uncategorized: Vec<ChannelDisplayItem> = Vec::new();
    let mut categorized_map: std::collections::BTreeMap<u64, Vec<ChannelDisplayItem>> =
        std::collections::BTreeMap::new();

    for ch in channels {
        let is_active = active_channel_id == Some(ch.id);
        let item = ChannelDisplayItem {
            channel: ch.clone(),
            is_active,
            has_unread: false,
        };

        match ch.category_id {
            Some(cat_id) => {
                categorized_map.entry(cat_id).or_default().push(item);
            }
            None => {
                uncategorized.push(item);
            }
        }
    }

    let mut groups = Vec::new();

    // 1. Uncategorized top-level channels
    if !uncategorized.is_empty() {
        uncategorized.sort_by_key(|i| (i.channel.position, i.channel.id));
        groups.push(CategorizedChannelGroup {
            category_name: "CHANNELS".to_string(),
            channels: uncategorized,
        });
    }

    // 2. Categorized channel groups
    for (cat_id, mut items) in categorized_map {
        items.sort_by_key(|i| (i.channel.position, i.channel.id));

        let category_name = if items
            .iter()
            .all(|i| i.channel.channel_type == ChannelType::Voice)
        {
            "VOICE CHANNELS".to_string()
        } else if items
            .iter()
            .all(|i| i.channel.channel_type == ChannelType::Text)
        {
            "TEXT CHANNELS".to_string()
        } else {
            format!("CATEGORY {cat_id}")
        };

        groups.push(CategorizedChannelGroup {
            category_name,
            channels: items,
        });
    }

    groups
}

/// Channel Tree view state.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTreeView {
    pub server_name: String,
    pub groups: Vec<CategorizedChannelGroup>,
    pub active_channel_id: Option<u64>,
}

impl Default for ChannelTreeView {
    fn default() -> Self {
        Self {
            server_name: "Sharkord Plus".to_string(),
            groups: Vec::new(),
            active_channel_id: None,
        }
    }
}

impl ChannelTreeView {
    pub fn new(
        server_name: impl Into<String>,
        channels: &[Channel],
        active_id: Option<u64>,
    ) -> Self {
        let groups = compose_channel_tree(channels, active_id);
        Self {
            server_name: server_name.into(),
            groups,
            active_channel_id: active_id,
        }
    }

    /// Renders the Channel Tree element tree.
    pub fn render_element(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .flex_1()
            .overflow_hidden()
            .bg(theme::bg_subsurface())
            // 1. Server name header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(48.0))
                    .px_4()
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .text_color(theme::text_primary())
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(self.server_name.clone()),
                    )
                    .child(div().text_color(theme::text_muted()).child("▼")),
            )
            // 2. Channel categories and list
            .child(
                div()
                    .id("channel_tree_list")
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_3()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.groups.iter().map(|group| {
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            // Category header
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1()
                                    .py_1()
                                    .px_1()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_muted())
                                    .child(group.category_name.clone()),
                            )
                            // Channels in this category
                            .children(group.channels.iter().map(|item| {
                                let icon = channel_icon(item.channel.channel_type);
                                let is_active = item.is_active;

                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if is_active {
                                        theme::bg_elevated()
                                    } else {
                                        theme::bg_subsurface()
                                    })
                                    .text_color(if is_active {
                                        theme::text_primary()
                                    } else {
                                        theme::text_secondary()
                                    })
                                    .child(
                                        div()
                                            .text_color(if is_active {
                                                theme::text_primary()
                                            } else {
                                                theme::text_muted()
                                            })
                                            .child(icon),
                                    )
                                    .child(
                                        div()
                                            .font_weight(if is_active {
                                                gpui::FontWeight::MEDIUM
                                            } else {
                                                gpui::FontWeight::NORMAL
                                            })
                                            .child(item.channel.name.clone()),
                                    )
                            }))
                    })),
            )
    }
}

impl Render for ChannelTreeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
