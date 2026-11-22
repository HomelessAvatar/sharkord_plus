//! Channel Tree sidebar view component (240px fixed width, bg_subsurface).
//! Renders the server header, collapsible categorized text and voice channels,
//! active channel selection highlights, unread count badges, and channel creation button.

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

/// Display state for a participant in a voice channel roster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceParticipantDisplay {
    pub user_id: u64,
    pub name: String,
    pub is_speaking: bool,
    pub is_muted: bool,
    pub is_deafened: bool,
}

impl VoiceParticipantDisplay {
    /// Returns the border color (hex u32) for the speaking indicator ring,
    /// or None if the participant is not currently speaking.
    pub fn speaking_border_color(&self) -> Option<u32> {
        if self.is_speaking {
            Some(theme::STATUS_ONLINE)
        } else {
            None
        }
    }
}

/// Channel item prepared for display in the channel tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelDisplayItem {
    pub channel: Channel,
    pub is_active: bool,
    pub has_unread: bool,
    pub voice_participants: Vec<VoiceParticipantDisplay>,
}

/// A group of channels under a named category with collapsible state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorizedChannelGroup {
    pub category_name: String,
    pub name: String,
    pub is_open: bool,
    pub channels: Vec<ChannelDisplayItem>,
    pub unread_count: usize,
}

impl CategorizedChannelGroup {
    /// Creates a new open channel group with zero unread channels.
    pub fn new(name: impl Into<String>) -> Self {
        let name_str = name.into();
        Self {
            category_name: name_str.clone(),
            name: name_str,
            is_open: true,
            channels: Vec::new(),
            unread_count: 0,
        }
    }

    /// Toggles the collapsible open/closed state.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Explicitly sets the collapsible open/closed state.
    pub fn set_open(&mut self, is_open: bool) {
        self.is_open = is_open;
    }

    /// Adds a channel to this category group and aggregates unread status.
    pub fn add_channel(&mut self, channel: ChannelDisplayItem) {
        if channel.has_unread {
            self.unread_count += 1;
        }
        self.channels.push(channel);
    }

    /// Recalculates the aggregated unread count from the current channel list.
    pub fn recalculate_unread(&mut self) {
        self.unread_count = self.channels.iter().filter(|c| c.has_unread).count();
    }
}

/// Composes and categorizes a flat list of channels for hierarchical sidebar display.
pub fn compose_channel_tree(
    channels: &[Channel],
    active_channel_id: Option<u64>,
) -> Vec<CategorizedChannelGroup> {
    let empty_participants = std::collections::HashMap::new();
    compose_channel_tree_with_participants(channels, active_channel_id, &empty_participants)
}

/// Composes channel tree including voice participants for active voice channels.
pub fn compose_channel_tree_with_participants(
    channels: &[Channel],
    active_channel_id: Option<u64>,
    participants: &std::collections::HashMap<u64, Vec<VoiceParticipantDisplay>>,
) -> Vec<CategorizedChannelGroup> {
    if channels.is_empty() {
        return Vec::new();
    }

    let mut uncategorized: Vec<ChannelDisplayItem> = Vec::new();
    let mut categorized_map: std::collections::BTreeMap<u64, Vec<ChannelDisplayItem>> =
        std::collections::BTreeMap::new();

    for ch in channels {
        let is_active = active_channel_id == Some(ch.id);
        let voice_participants = participants.get(&ch.id).cloned().unwrap_or_default();
        let item = ChannelDisplayItem {
            channel: ch.clone(),
            is_active,
            has_unread: false,
            voice_participants,
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
        let unread = uncategorized.iter().filter(|i| i.has_unread).count();
        groups.push(CategorizedChannelGroup {
            category_name: "CHANNELS".to_string(),
            name: "CHANNELS".to_string(),
            is_open: true,
            channels: uncategorized,
            unread_count: unread,
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

        let unread = items.iter().filter(|i| i.has_unread).count();
        groups.push(CategorizedChannelGroup {
            category_name: category_name.clone(),
            name: category_name,
            is_open: true,
            channels: items,
            unread_count: unread,
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

    /// Toggles the collapsible open/closed state of a named category.
    pub fn toggle_category(&mut self, category_name: &str) {
        if let Some(group) = self
            .groups
            .iter_mut()
            .find(|g| g.category_name == category_name || g.name == category_name)
        {
            group.toggle();
        }
    }

    /// Sets the open/closed state of a named category.
    pub fn set_category_open(&mut self, category_name: &str, is_open: bool) {
        if let Some(group) = self
            .groups
            .iter_mut()
            .find(|g| g.category_name == category_name || g.name == category_name)
        {
            group.set_open(is_open);
        }
    }

    /// Renders the Channel Tree element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the Channel Tree element tree with interactive channel selection handlers.
    pub fn render_interactive(
        &self,
        cx: &mut Context<crate::ui::WorkspaceView>,
    ) -> impl IntoElement {
        self.render_internal(Some(cx))
    }

    fn render_internal(
        &self,
        mut cx: Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> impl IntoElement {
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
            // 2. Channel categories and collapsible list
            .child(
                div()
                    .id("channel_tree_list")
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_3()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.groups.iter().enumerate().map(|(group_idx, group)| {
                        let is_open = group.is_open;
                        let unread_count = group.unread_count;
                        let cat_name = group.category_name.clone();

                        let mut header = div()
                            .id(ElementId::Name(format!("category_header_{}", group_idx).into()))
                            .cursor_pointer()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .py_1()
                            .px_1()
                            .rounded_sm()
                            .hover(|s| s.bg(theme::bg_surface_4()))
                            // Left side: chevron, title, unread count badge
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_1p5()
                                    // Chevron indicator (rotated when collapsed/expanded)
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme::text_muted())
                                            .child(if is_open { "▼" } else { "▶" }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme::text_muted())
                                            .child(group.category_name.clone()),
                                    )
                                    // Unread badge when collapsed and unread_count > 0
                                    .children(if !is_open && unread_count > 0 {
                                        Some(
                                            div()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_full()
                                                .bg(theme::accent())
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child(unread_count.to_string()),
                                        )
                                    } else {
                                        None
                                    }),
                            )
                            // Right side: Admin '+' vector button for channel creation
                            .child(
                                div()
                                    .id(ElementId::Name(
                                        format!("create_channel_btn_{}", group_idx).into(),
                                    ))
                                    .cursor_pointer()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(16.0))
                                    .h(px(16.0))
                                    .rounded_sm()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_muted())
                                    .hover(|s| {
                                        s.text_color(theme::text_primary())
                                            .bg(theme::bg_elevated())
                                    })
                                    .child("+"),
                            );

                        if let Some(ref mut cx) = cx {
                            let cat_toggle = cat_name.clone();
                            header = header.on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |view, _, _window, cx| {
                                    view.toggle_channel_category(&cat_toggle, cx);
                                }),
                            );
                        }

                        let mut group_container = div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(header);

                        // Only render channels if category is open
                        if is_open {
                            group_container = group_container.children(group.channels.iter().map(|item| {
                                let icon = channel_icon(item.channel.channel_type);
                                let is_active = item.is_active;
                                let channel_id = item.channel.id;

                                let mut channel_btn = div()
                                    .id(ElementId::Name(
                                        format!("channel_item_{}", item.channel.id).into(),
                                    ))
                                    .cursor_pointer()
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
                                    );

                                if let Some(ref mut cx) = cx {
                                    channel_btn = channel_btn.on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |view, _, _window, cx| {
                                            view.on_channel_click(channel_id, cx);
                                        }),
                                    );
                                }

                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .child(channel_btn)
                                    // Voice participants list under active voice channel
                                    .children(item.voice_participants.iter().map(|p| {
                                        let border_color = p.speaking_border_color();

                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .pl_6()
                                            .pr_2()
                                            .py_1()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap_2()
                                                    // Participant avatar with speaking indicator ring
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .w(px(20.0))
                                                            .h(px(20.0))
                                                            .rounded_full()
                                                            .bg(theme::bg_surface_4())
                                                            .border_2()
                                                            .border_color(
                                                                if let Some(c) = border_color {
                                                                    rgb(c)
                                                                } else {
                                                                    rgba(0x00000000)
                                                                },
                                                            )
                                                            .text_xs()
                                                            .text_color(theme::text_primary())
                                                            .child(
                                                                p.name
                                                                    .chars()
                                                                    .next()
                                                                    .unwrap_or('?')
                                                                    .to_string(),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(theme::text_primary())
                                                            .child(p.name.clone()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap_1()
                                                    .children(if p.is_deafened {
                                                        Some(
                                                            div()
                                                                  .text_xs()
                                                                .text_color(theme::status_dnd())
                                                                .child("🎧"),
                                                        )
                                                    } else if p.is_muted {
                                                        Some(
                                                            div()
                                                                .text_xs()
                                                                .text_color(theme::status_dnd())
                                                                .child("🔇"),
                                                        )
                                                    } else {
                                                        None
                                                    }),
                                            )
                                    }))
                            }));
                        }

                        group_container
                    })),
            )
    }
}

impl Render for ChannelTreeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
