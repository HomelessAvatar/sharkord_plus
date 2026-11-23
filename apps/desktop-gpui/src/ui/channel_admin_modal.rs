//! Channel Administration Modal dialog views in GPUI.
//! Provides `CreateChannelModalView` (Text vs Voice radio selection, name, category, actions)
//! and `EditChannelModalView` (Rename input, topic textarea, delete channel with confirmation).
//! Adheres strictly to Pure Neutral Dark design specification (#0A0B0E base,
//! #111216 surface, #16171d subsurface, #1c1e24 elevated, #3B82F6 accent, #F3F4F6 text).

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::ChannelType;

use crate::theme;

/// State for the Create Channel modal dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateChannelModalState {
    pub is_open: bool,
    pub channel_type: ChannelType,
    pub name: String,
    pub category: String,
}

impl Default for CreateChannelModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            channel_type: ChannelType::Text,
            name: String::new(),
            category: String::new(),
        }
    }
}

impl CreateChannelModalState {
    /// Creates a new closed `CreateChannelModalState` with default Text channel type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the modal dialog for a given category.
    pub fn open(&mut self, category: impl Into<String>) {
        self.is_open = true;
        self.category = category.into();
    }

    /// Closes the modal dialog.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Sets the channel type (Text or Voice).
    pub fn set_channel_type(&mut self, channel_type: ChannelType) {
        self.channel_type = channel_type;
    }

    /// Sets the channel name draft.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Sets the category designation.
    pub fn set_category(&mut self, category: impl Into<String>) {
        self.category = category.into();
    }

    /// Resets the modal state to defaults and closes it.
    pub fn reset(&mut self) {
        self.is_open = false;
        self.channel_type = ChannelType::Text;
        self.name.clear();
        self.category.clear();
    }
}

/// View component rendering the Create Channel modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateChannelModalView {
    pub state: CreateChannelModalState,
}

impl CreateChannelModalView {
    /// Creates a new `CreateChannelModalView` with the provided state.
    pub fn new(state: CreateChannelModalState) -> Self {
        Self { state }
    }

    /// Renders the Create Channel modal element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the Create Channel modal element tree with interactive handlers.
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
        let is_text = self.state.channel_type == ChannelType::Text;
        let is_voice = self.state.channel_type == ChannelType::Voice;

        // 1. Text option card
        let mut text_card = div()
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .p_3()
            .rounded_lg()
            .bg(if is_text {
                theme::bg_surface_4()
            } else {
                theme::bg_subsurface()
            })
            .border_1()
            .border_color(if is_text {
                theme::accent()
            } else {
                theme::border_hairline()
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(if is_text {
                                theme::accent()
                            } else {
                                theme::text_muted()
                            })
                            .child("#"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child("Text"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Post messages, images, memes, and opinions"),
                            ),
                    ),
            )
            .child(
                div()
                    .w(px(18.0))
                    .h(px(18.0))
                    .rounded_full()
                    .border_2()
                    .border_color(if is_text {
                        theme::accent()
                    } else {
                        theme::border_subtle()
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .children(if is_text {
                        Some(
                            div()
                                .w(px(8.0))
                                .h(px(8.0))
                                .rounded_full()
                                .bg(theme::accent()),
                        )
                    } else {
                        None
                    }),
            );

        // 2. Voice option card
        let mut voice_card = div()
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .p_3()
            .rounded_lg()
            .bg(if is_voice {
                theme::bg_surface_4()
            } else {
                theme::bg_subsurface()
            })
            .border_1()
            .border_color(if is_voice {
                theme::accent()
            } else {
                theme::border_hairline()
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_base()
                            .text_color(if is_voice {
                                theme::accent()
                            } else {
                                theme::text_muted()
                            })
                            .child("🔊"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child("Voice"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Hang out together with voice, video, and screen share"),
                            ),
                    ),
            )
            .child(
                div()
                    .w(px(18.0))
                    .h(px(18.0))
                    .rounded_full()
                    .border_2()
                    .border_color(if is_voice {
                        theme::accent()
                    } else {
                        theme::border_subtle()
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .children(if is_voice {
                        Some(
                            div()
                                .w(px(8.0))
                                .h(px(8.0))
                                .rounded_full()
                                .bg(theme::accent()),
                        )
                    } else {
                        None
                    }),
            );

        let mut close_btn = div()
            .cursor_pointer()
            .text_sm()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(theme::text_muted())
            .hover(|s| s.text_color(theme::text_primary()))
            .child("✕");

        let mut cancel_btn = div()
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_md()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_muted())
            .hover(|s| s.text_color(theme::text_primary()).bg(theme::bg_surface_4()))
            .child("Cancel");

        let mut submit_btn = div()
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(theme::accent())
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::accent_hover()))
            .child("Create Channel");

        if let Some(ref mut cx) = cx {
            text_card = text_card.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.create_channel_modal
                        .set_channel_type(ChannelType::Text);
                    cx.notify();
                }),
            );

            voice_card = voice_card.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.create_channel_modal
                        .set_channel_type(ChannelType::Voice);
                    cx.notify();
                }),
            );

            close_btn = close_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_create_channel_modal(cx);
                }),
            );

            cancel_btn = cancel_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_create_channel_modal(cx);
                }),
            );

            submit_btn = submit_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.submit_create_channel();
                    cx.notify();
                }),
            );
        }

        let category_subtitle = if !self.state.category.is_empty() {
            format!("in {}", self.state.category)
        } else {
            "Select type and name for your new channel".to_string()
        };

        let dialog_card = div()
            .flex()
            .flex_col()
            .w(px(460.0))
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .rounded_xl()
            .shadow_lg()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .pt_5()
                    .pb_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child("Create Channel"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(category_subtitle),
                            ),
                    )
                    .child(close_btn),
            )
            // Body
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .px_6()
                    .py_3()
                    // Channel Type Radio
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_secondary())
                                    .child("CHANNEL TYPE"),
                            )
                            .child(text_card)
                            .child(voice_card),
                    )
                    // Channel Name input
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_secondary())
                                    .child("CHANNEL NAME"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme::text_muted())
                                            .child(if is_text { "#" } else { "🔊" }),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(if self.state.name.is_empty() {
                                                theme::text_muted()
                                            } else {
                                                theme::text_primary()
                                            })
                                            .child(if self.state.name.is_empty() {
                                                "new-channel".to_string()
                                            } else {
                                                self.state.name.clone()
                                            }),
                                    ),
                            ),
                    ),
            )
            // Footer
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .items_center()
                    .gap_3()
                    .px_6()
                    .py_4()
                    .bg(theme::bg_surface())
                    .border_t_1()
                    .border_color(theme::border_hairline())
                    .child(cancel_btn)
                    .child(submit_btn),
            );

        let mut backdrop = div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.65))
            .child(dialog_card);

        if let Some(ref mut cx) = cx {
            backdrop = backdrop.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_create_channel_modal(cx);
                }),
            );
        }

        backdrop
    }
}

impl Render for CreateChannelModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}

/// State for the Edit Channel modal dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditChannelModalState {
    pub is_open: bool,
    pub channel_id: u64,
    pub name: String,
    pub topic: String,
    pub confirm_delete: bool,
}

impl Default for EditChannelModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            channel_id: 0,
            name: String::new(),
            topic: String::new(),
            confirm_delete: false,
        }
    }
}

impl EditChannelModalState {
    /// Creates a new closed `EditChannelModalState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the modal for editing a specific channel.
    pub fn open(
        &mut self,
        channel_id: u64,
        name: impl Into<String>,
        topic: impl Into<String>,
    ) {
        self.is_open = true;
        self.channel_id = channel_id;
        self.name = name.into();
        self.topic = topic.into();
        self.confirm_delete = false;
    }

    /// Closes the edit channel modal.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Sets the channel name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Sets the channel topic description.
    pub fn set_topic(&mut self, topic: impl Into<String>) {
        self.topic = topic.into();
    }

    /// Toggles the delete channel confirmation prompt.
    pub fn toggle_confirm_delete(&mut self) {
        self.confirm_delete = !self.confirm_delete;
    }

    /// Explicitly sets the delete confirmation state.
    pub fn set_confirm_delete(&mut self, confirm: bool) {
        self.confirm_delete = confirm;
    }

    /// Resets all fields and closes the modal.
    pub fn reset(&mut self) {
        self.is_open = false;
        self.channel_id = 0;
        self.name.clear();
        self.topic.clear();
        self.confirm_delete = false;
    }
}

/// View component rendering the Edit Channel modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct EditChannelModalView {
    pub state: EditChannelModalState,
}

impl EditChannelModalView {
    /// Creates a new `EditChannelModalView` from the provided state.
    pub fn new(state: EditChannelModalState) -> Self {
        Self { state }
    }

    /// Renders the Edit Channel modal element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the Edit Channel modal element tree with interactive handlers.
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
        let mut close_btn = div()
            .cursor_pointer()
            .text_sm()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(theme::text_muted())
            .hover(|s| s.text_color(theme::text_primary()))
            .child("✕");

        let mut cancel_btn = div()
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_md()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_muted())
            .hover(|s| s.text_color(theme::text_primary()).bg(theme::bg_surface_4()))
            .child("Cancel");

        let mut save_btn = div()
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(theme::accent())
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_primary())
            .hover(|s| s.bg(theme::accent_hover()))
            .child("Save Changes");

        let mut delete_btn = div()
            .cursor_pointer()
            .px_3()
            .py_1p5()
            .rounded_md()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::status_dnd())
            .hover(|s| s.bg(hsla(0.0, 0.65, 0.20, 0.40)))
            .child("Delete Channel");

        let mut confirm_delete_btn = div()
            .cursor_pointer()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(theme::status_dnd())
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(theme::text_primary())
            .child("Confirm Delete");

        let mut cancel_delete_btn = div()
            .cursor_pointer()
            .px_3()
            .py_2()
            .rounded_md()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_muted())
            .hover(|s| s.text_color(theme::text_primary()))
            .child("Nevermind");

        if let Some(ref mut cx) = cx {
            close_btn = close_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_edit_channel_modal(cx);
                }),
            );

            cancel_btn = cancel_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_edit_channel_modal(cx);
                }),
            );

            save_btn = save_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.submit_edit_channel();
                    cx.notify();
                }),
            );

            delete_btn = delete_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.edit_channel_modal.set_confirm_delete(true);
                    cx.notify();
                }),
            );

            confirm_delete_btn = confirm_delete_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.submit_delete_channel();
                    cx.notify();
                }),
            );

            cancel_delete_btn = cancel_delete_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.edit_channel_modal.set_confirm_delete(false);
                    cx.notify();
                }),
            );
        }

        let danger_zone = if self.state.confirm_delete {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .rounded_lg()
                .bg(hsla(0.0, 0.65, 0.20, 0.25))
                .border_1()
                .border_color(theme::status_dnd())
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::status_dnd())
                        .child("⚠️ Are you sure you want to delete this channel? This cannot be undone."),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(confirm_delete_btn)
                        .child(cancel_delete_btn),
                )
        } else {
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .pt_2()
                .border_t_1()
                .border_color(theme::border_hairline())
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child("Permanently delete this channel and its history"),
                )
                .child(delete_btn)
        };

        let dialog_card = div()
            .flex()
            .flex_col()
            .w(px(480.0))
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .rounded_xl()
            .shadow_lg()
            .overflow_hidden()
            // Header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .pt_5()
                    .pb_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child("Channel Settings"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(format!("#{}", self.state.name)),
                            ),
                    )
                    .child(close_btn),
            )
            // Body
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .px_6()
                    .py_3()
                    // Name field
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_secondary())
                                    .child("CHANNEL NAME"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme::text_muted())
                                            .child("#"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(if self.state.name.is_empty() {
                                                theme::text_muted()
                                            } else {
                                                theme::text_primary()
                                            })
                                            .child(if self.state.name.is_empty() {
                                                "channel-name".to_string()
                                            } else {
                                                self.state.name.clone()
                                            }),
                                    ),
                            ),
                    )
                    // Topic field
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_secondary())
                                    .child("CHANNEL TOPIC"),
                            )
                            .child(
                                div()
                                    .min_h(px(64.0))
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if self.state.topic.is_empty() {
                                                theme::text_muted()
                                            } else {
                                                theme::text_primary()
                                            })
                                            .child(if self.state.topic.is_empty() {
                                                "Let everyone know how to use this channel!".to_string()
                                            } else {
                                                self.state.topic.clone()
                                            }),
                                    ),
                            ),
                    )
                    // Danger Zone / Delete Channel
                    .child(danger_zone),
            )
            // Footer
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .items_center()
                    .gap_3()
                    .px_6()
                    .py_4()
                    .bg(theme::bg_surface())
                    .border_t_1()
                    .border_color(theme::border_hairline())
                    .child(cancel_btn)
                    .child(save_btn),
            );

        let mut backdrop = div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.65))
            .child(dialog_card);

        if let Some(ref mut cx) = cx {
            backdrop = backdrop.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_edit_channel_modal(cx);
                }),
            );
        }

        backdrop
    }
}

impl Render for EditChannelModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
