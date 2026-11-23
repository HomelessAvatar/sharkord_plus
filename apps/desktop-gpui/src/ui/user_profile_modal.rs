//! User Profile Popover / Modal component.
//! Adheres strictly to Pure Neutral Dark design specification with banner, avatar,
//! status indicator ring, username handle, role badge pill, and DM action button.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};

use crate::theme;

/// State for the user profile popover/modal card.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UserProfileCardState {
    pub is_open: bool,
    pub user_id: Option<u64>,
    pub username: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub status: String,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
}

impl UserProfileCardState {
    /// Creates a new closed `UserProfileCardState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the profile card for a specific user.
    pub fn open(
        &mut self,
        user_id: u64,
        username: impl Into<String>,
        display_name: impl Into<String>,
        status: impl Into<String>,
    ) {
        self.is_open = true;
        self.user_id = Some(user_id);
        self.username = username.into();
        self.display_name = Some(display_name.into());
        self.status = status.into();
    }

    /// Closes the user profile card.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Sets the user bio biography text.
    pub fn set_bio(&mut self, bio: impl Into<String>) {
        self.bio = Some(bio.into());
    }

    /// Sets the user role designation.
    pub fn set_role(&mut self, role: impl Into<String>) {
        self.role = Some(role.into());
    }

    /// Sets the user avatar URL.
    pub fn set_avatar_url(&mut self, avatar_url: impl Into<String>) {
        self.avatar_url = Some(avatar_url.into());
    }
}

/// Type aliases for flexible consumer naming conventions.
pub type UserProfileState = UserProfileCardState;
pub type UserProfilePopover = UserProfilePopoverView;
pub type UserProfileModalView = UserProfilePopoverView;

/// View component for rendering the user profile popover / modal card.
#[derive(Debug, Clone, PartialEq)]
pub struct UserProfilePopoverView {
    pub state: UserProfileCardState,
}

impl UserProfilePopoverView {
    /// Creates a new `UserProfilePopoverView` from the provided card state.
    pub fn new(state: UserProfileCardState) -> Self {
        Self { state }
    }

    /// Returns the target username handle.
    pub fn username(&self) -> &str {
        &self.state.username
    }

    /// Returns the target user's display name or fallback username.
    pub fn display_name(&self) -> &str {
        if let Some(ref name) = self.state.display_name {
            if !name.trim().is_empty() {
                return name.as_str();
            }
        }
        &self.state.username
    }

    /// Returns the user presence status string.
    pub fn status(&self) -> &str {
        &self.state.status
    }

    /// Returns the biography string if set.
    pub fn bio(&self) -> Option<&str> {
        self.state.bio.as_deref()
    }

    /// Returns the role designation string if set.
    pub fn role_badge(&self) -> Option<&str> {
        self.state.role.as_deref()
    }

    /// Returns the primary action button label.
    pub fn action_label(&self) -> &'static str {
        "Send Direct Message"
    }

    /// Generates compact initials for avatar display.
    pub fn initials(&self) -> String {
        let name = self.display_name();
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return "?".to_string();
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() > 1 {
            words
                .into_iter()
                .take(2)
                .filter_map(|w| w.chars().next())
                .flat_map(|c| c.to_uppercase())
                .collect()
        } else {
            let chars: Vec<char> = trimmed.chars().collect();
            if chars.len() == 1 {
                chars[0].to_uppercase().to_string()
            } else {
                chars
                    .into_iter()
                    .take(2)
                    .flat_map(|c| c.to_uppercase())
                    .collect()
            }
        }
    }

    /// Resolves the presence indicator color according to user status.
    pub fn status_color(&self) -> Hsla {
        match self.state.status.as_str() {
            "online" => theme::status_online(),
            "idle" => theme::status_idle(),
            "dnd" => theme::status_dnd(),
            _ => theme::status_offline(),
        }
    }

    /// Renders the user profile card element tree non-interactively.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the user profile card with interactive click handlers wired to `WorkspaceView`.
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
        let initials = self.initials();
        let display_name = self.display_name().to_string();
        let username_handle = format!("@{}", self.state.username);
        let status_color = self.status_color();
        let user_id = self.state.user_id;

        let mut close_btn = div()
            .id("profile_modal_close_btn")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(24.0))
            .h(px(24.0))
            .rounded_full()
            .bg(theme::bg_surface_4())
            .hover(|s| s.bg(theme::bg_elevated()))
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(theme::text_secondary())
            .child("✕");

        let mut send_dm_btn = div()
            .id("profile_send_dm_btn")
            .cursor_pointer()
            .w_full()
            .mt_4()
            .py_2()
            .rounded_lg()
            .bg(theme::accent())
            .hover(|s| s.bg(theme::accent_hover()))
            .text_sm()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_primary())
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .child(crate::ui::render_chat_bubble_icon())
            .child(self.action_label());

        if let Some(ref mut cx) = cx {
            close_btn = close_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_user_profile();
                    cx.notify();
                }),
            );

            let u_id_val = user_id;
            let u_name_val = self.state.username.clone();
            send_dm_btn = send_dm_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |view, _, _window, cx| {
                    if let Some(uid) = u_id_val {
                        view.open_dm_conversation(uid, &u_name_val);
                    }
                    view.close_user_profile();
                    cx.notify();
                }),
            );
        }

        let mut card = div()
            .id("user_profile_card")
            .flex()
            .flex_col()
            .w(px(340.0))
            .rounded_2xl()
            .bg(theme::bg_surface())
            .border_1()
            .border_color(theme::border_subtle())
            .shadow_2xl()
            .overflow_hidden()
            // 1. Banner Header (100px)
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(100.0))
                    .bg(theme::bg_elevated())
                    .child(
                        div()
                            .absolute()
                            .top_3()
                            .right_3()
                            .child(close_btn),
                    ),
            )
            // 2. Card Body
            .child(
                div()
                    .flex()
                    .flex_col()
                    .px_4()
                    .pb_5()
                    // Avatar overlapping banner
                    .child(
                        div()
                            .relative()
                            .mt(px(-40.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(80.0))
                            .h(px(80.0))
                            .rounded_full()
                            .bg(theme::bg_surface_4())
                            .border_4()
                            .border_color(theme::bg_surface())
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(initials)
                            // Status indicator ring dot
                            .child(
                                div()
                                    .absolute()
                                    .bottom_0()
                                    .right_0()
                                    .w(px(20.0))
                                    .h(px(20.0))
                                    .rounded_full()
                                    .border_3()
                                    .border_color(theme::bg_surface())
                                    .bg(status_color),
                            ),
                    )
                    // Names and handle
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .mt_2()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(display_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child(username_handle),
                            ),
                    ),
            );

        // Role badge pill
        if let Some(role) = self.role_badge() {
            card = card.child(
                div()
                    .px_4()
                    .pt_2()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .px_2p5()
                            .py_0p5()
                            .rounded_md()
                            .bg(theme::bg_elevated())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::accent())
                            .child(format!("🏷 {}", role)),
                    ),
            );
        }

        // Divider
        card = card.child(
            div()
                .mx_4()
                .my_3()
                .h(px(1.0))
                .bg(theme::border_hairline()),
        );

        // Bio section
        if let Some(bio) = self.bio() {
            card = card.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .px_4()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_muted())
                            .child("ABOUT ME"),
                    )
                    .child(
                        div()
                            .p_2p5()
                            .rounded_lg()
                            .bg(theme::bg_subsurface())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .text_xs()
                            .text_color(theme::text_primary())
                            .child(bio.to_string()),
                    ),
            );
        }

        // Action button container
        card = card.child(
            div()
                .px_4()
                .child(send_dm_btn),
        );

        // Wrap in full backdrop overlay
        let mut backdrop = div()
            .id("user_profile_modal_backdrop")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::hsla(0.0, 0.0, 0.0, 0.6))
            .child(card);

        if let Some(ref mut cx) = cx {
            backdrop = backdrop.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_user_profile();
                    cx.notify();
                }),
            );
        }

        backdrop
    }
}

impl Render for UserProfilePopoverView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
