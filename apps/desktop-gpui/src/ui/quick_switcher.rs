//! Quick Switcher (`Ctrl+K`) command palette modal component for fast channel, member, and action navigation.
//! Implements a Raycast/Linear-style 560px centered modal overlay with search input,
//! grouped result categories (Channels, Members, Commands), and keyboard shortcut pills.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::{Channel, ChannelType, User};

use crate::theme;

/// Category/type of an item displayed in the quick switcher command palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuickSwitcherItemType {
    Channel,
    VoiceRoom,
    Member,
    Command,
}

impl QuickSwitcherItemType {
    /// User-facing short label for the item type badge.
    pub fn label(&self) -> &'static str {
        match self {
            QuickSwitcherItemType::Channel => "Channel",
            QuickSwitcherItemType::VoiceRoom => "Voice",
            QuickSwitcherItemType::Member => "Member",
            QuickSwitcherItemType::Command => "Command",
        }
    }

    /// Category section header title.
    pub fn section_header(&self) -> &'static str {
        match self {
            QuickSwitcherItemType::Channel => "CHANNELS",
            QuickSwitcherItemType::VoiceRoom => "VOICE ROOMS",
            QuickSwitcherItemType::Member => "MEMBERS",
            QuickSwitcherItemType::Command => "COMMANDS",
        }
    }
}

/// An item displayed in the quick switcher command palette results list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickSwitcherItem {
    pub id: u64,
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: String,
    pub item_type: QuickSwitcherItemType,
    // Backwards-compatibility fields with legacy tests and views:
    pub name: String,
    pub channel_type: ChannelType,
    pub category: Option<String>,
}

impl Default for QuickSwitcherItem {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            subtitle: None,
            icon: "#".to_string(),
            item_type: QuickSwitcherItemType::Channel,
            name: String::new(),
            channel_type: ChannelType::Text,
            category: None,
        }
    }
}

impl QuickSwitcherItem {
    /// Creates a new quick switcher item representing a channel or voice room.
    pub fn channel(
        id: u64,
        name: impl Into<String>,
        channel_type: ChannelType,
        category: Option<String>,
    ) -> Self {
        let name_str = name.into();
        let icon = match channel_type {
            ChannelType::Text => "#".to_string(),
            ChannelType::Voice => "🔊".to_string(),
        };
        let item_type = match channel_type {
            ChannelType::Text => QuickSwitcherItemType::Channel,
            ChannelType::Voice => QuickSwitcherItemType::VoiceRoom,
        };
        Self {
            id,
            title: name_str.clone(),
            subtitle: category.clone(),
            icon,
            item_type,
            name: name_str,
            channel_type,
            category,
        }
    }

    /// Creates a new quick switcher item representing a server member.
    pub fn member(id: u64, name: impl Into<String>, status: Option<String>) -> Self {
        let name_str = name.into();
        Self {
            id,
            title: name_str.clone(),
            subtitle: status.clone(),
            icon: "👤".to_string(),
            item_type: QuickSwitcherItemType::Member,
            name: name_str,
            channel_type: ChannelType::Text,
            category: status,
        }
    }

    /// Creates a new quick switcher item representing an action slash command.
    pub fn command(id: u64, command: impl Into<String>, description: impl Into<String>) -> Self {
        let cmd_str = command.into();
        let desc_str = description.into();
        Self {
            id,
            title: cmd_str.clone(),
            subtitle: Some(desc_str.clone()),
            icon: "⚡".to_string(),
            item_type: QuickSwitcherItemType::Command,
            name: cmd_str,
            channel_type: ChannelType::Text,
            category: Some(desc_str),
        }
    }
}

/// Returns the standard default global action commands for the command palette.
pub fn default_command_palette_items() -> Vec<QuickSwitcherItem> {
    vec![
        QuickSwitcherItem::command(1001, "/mute", "Toggle microphone mute"),
        QuickSwitcherItem::command(1002, "/deafen", "Toggle audio deafen"),
        QuickSwitcherItem::command(1003, "/settings", "Open settings dialog"),
        QuickSwitcherItem::command(1004, "/status", "Set presence status"),
        QuickSwitcherItem::command(1005, "/theme", "Toggle dark/light theme"),
        QuickSwitcherItem::command(1006, "/help", "View keyboard shortcuts and docs"),
    ]
}

/// State for the quick switcher command palette modal dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickSwitcherState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub items: Vec<QuickSwitcherItem>,
    pub channels: Vec<Channel>,
    pub members: Vec<User>,
    pub commands: Vec<QuickSwitcherItem>,
}

impl Default for QuickSwitcherState {
    fn default() -> Self {
        Self::new()
    }
}

impl QuickSwitcherState {
    /// Creates a new `QuickSwitcherState` initialized with default command palette items.
    pub fn new() -> Self {
        let commands = default_command_palette_items();
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            items: commands.clone(),
            channels: Vec::new(),
            members: Vec::new(),
            commands,
        }
    }

    /// Creates a new `QuickSwitcherState` initialized with a channel list (channels-only mode).
    pub fn from_channels(channels: &[Channel]) -> Self {
        let mut state = Self::new();
        state.commands.clear();
        state.set_channels(channels);
        state
    }

    /// Creates a new `QuickSwitcherState` initialized with a channel list.
    pub fn with_channels(channels: &[Channel]) -> Self {
        let mut state = Self::new();
        state.set_channels(channels);
        state
    }

    /// Opens the command palette modal.
    pub fn open(&mut self) {
        self.is_open = true;
        self.selected_index = 0;
    }

    /// Closes the command palette modal and clears search query.
    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Toggles the command palette open/closed state.
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Sets the search filter query and resets selected index to the first result.
    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.selected_index = 0;
    }

    /// Updates the available items from a new channel list.
    pub fn set_channels(&mut self, channels: &[Channel]) {
        self.channels = channels.to_vec();
        self.rebuild_items();
    }

    /// Updates the available items from a server member list.
    pub fn set_members(&mut self, members: &[User]) {
        self.members = members.to_vec();
        self.rebuild_items();
    }

    /// Appends an item to the palette items list.
    pub fn add_item(&mut self, item: QuickSwitcherItem) {
        self.items.push(item);
        self.clamp_selection();
    }

    fn rebuild_items(&mut self) {
        let mut all = Vec::new();

        // 1. Channels
        for c in &self.channels {
            all.push(QuickSwitcherItem::channel(
                c.id,
                &c.name,
                c.channel_type,
                None,
            ));
        }

        // 2. Members
        for m in &self.members {
            all.push(QuickSwitcherItem::member(
                m.id,
                &m.name,
                m.status.clone(),
            ));
        }

        // 3. Commands
        all.extend(self.commands.clone());

        self.items = all;
        self.clamp_selection();
    }

    /// Returns the list of items matching the current search query.
    pub fn filtered_items(&self) -> Vec<QuickSwitcherItem> {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            return self.items.clone();
        }

        let is_command_query = q.starts_with('/');

        let mut matches: Vec<QuickSwitcherItem> = self
            .items
            .iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&q)
                    || item.name.to_lowercase().contains(&q)
                    || item
                        .subtitle
                        .as_deref()
                        .map(|s| s.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || item
                        .category
                        .as_deref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        // If searching specifically with '/', rank action commands first
        if is_command_query {
            matches.sort_by_key(|item| {
                if item.item_type == QuickSwitcherItemType::Command {
                    0
                } else {
                    1
                }
            });
        }

        matches
    }

    /// Moves the selection cursor down to the next item, clamped at the list end.
    pub fn move_selection_down(&mut self) {
        let count = self.filtered_items().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index + 1 < count {
            self.selected_index += 1;
        }
    }

    /// Moves the selection cursor up to the previous item, clamped at index 0.
    pub fn move_selection_up(&mut self) {
        let count = self.filtered_items().len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Alias to `move_selection_down`.
    pub fn select_next(&mut self) {
        self.move_selection_down();
    }

    /// Alias to `move_selection_up`.
    pub fn select_prev(&mut self) {
        self.move_selection_up();
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

/// View component rendering the Quick Switcher / Command Palette modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct QuickSwitcherView {
    pub state: QuickSwitcherState,
}

impl QuickSwitcherView {
    /// Creates a new `QuickSwitcherView` with the given state.
    pub fn new(state: QuickSwitcherState) -> Self {
        Self { state }
    }

    /// Renders the modal overlay element without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the modal overlay element with interactive workspace selection handlers.
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
        let filtered = self.state.filtered_items();
        let is_placeholder = self.state.query.is_empty();
        let query_text = if is_placeholder {
            "Type a command or channel name to jump to..."
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
                .gap_1p5()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::text_secondary())
                        .child("No matching channels, members, or commands found"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child("Try searching for /mute, a channel name, or member"),
                )
                .into_any_element()]
        } else {
            filtered
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let is_selected = idx == self.state.selected_index;
                    let item_clone = item.clone();

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
                        .id(ElementId::Name(format!("quick_switcher_item_{}", idx).into()))
                        .cursor_pointer()
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
                                .gap_2p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(if is_selected {
                                            theme::text_primary()
                                        } else {
                                            theme::text_muted()
                                        })
                                        .child(item.icon.clone()),
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
                                        .child(item.title.clone()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .children(if let Some(ref sub) = item.subtitle {
                                    Some(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_muted())
                                            .child(sub.clone()),
                                    )
                                } else {
                                    None
                                })
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(theme::bg_surface())
                                        .border_1()
                                        .border_color(theme::border_hairline())
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme::text_muted())
                                        .child(item.item_type.label()),
                                ),
                        );

                    if let Some(ref mut cx) = cx {
                        item_row = item_row.on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |view, _, _window, cx| {
                                view.select_quick_switcher_item(item_clone.clone(), cx);
                            }),
                        );
                    }

                    item_row.into_any_element()
                })
                .collect()
        };

        // Modal container with backdrop overlay
        let mut backdrop = div()
            .id("quick_switcher_backdrop")
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.70));

        if let Some(ref mut cx) = cx {
            backdrop = backdrop.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.state.is_quick_switcher_open = false;
                    view.quick_switcher.close();
                    cx.notify();
                }),
            );
        }

        backdrop.child(
            div()
                .id("quick_switcher_modal")
                .flex()
                .flex_col()
                .w(px(560.0))
                .max_h(px(460.0))
                .bg(theme::bg_elevated())
                .border_1()
                .border_color(theme::border_subtle())
                .rounded_xl()
                .shadow_xl()
                .overflow_hidden()
                .on_mouse_down(MouseButton::Left, |_, _, _| {})
                // 1. Search bar input header
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
                        // Kbd badge for ESC shortcut
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(theme::bg_surface())
                                .border_1()
                                .border_color(theme::border_hairline())
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("ESC"),
                        ),
                )
                // 2. Results list
                .child(
                    div()
                        .id("quick_switcher_results")
                        .flex()
                        .flex_col()
                        .p_2()
                        .gap_1()
                        .max_h(px(320.0))
                        .overflow_y_scroll()
                        .children(items_elements),
                )
                // 3. Footer navigation hints and Kbd shortcut badges
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
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(theme::bg_elevated())
                                                .border_1()
                                                .border_color(theme::border_hairline())
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
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
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(theme::bg_elevated())
                                                .border_1()
                                                .border_color(theme::border_hairline())
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme::text_muted())
                                                .child("↵"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child("Select"),
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
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(theme::bg_elevated())
                                                .border_1()
                                                .border_color(theme::border_hairline())
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme::text_muted())
                                                .child("ESC"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child("Close"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(format!("{} results", filtered.len())),
                        ),
                ),
        )
    }
}
