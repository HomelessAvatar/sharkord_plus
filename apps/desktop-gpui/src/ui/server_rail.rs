//! Server Rail view component (72px fixed width, bg_surface).
//! Renders the vertical navigation rail with the Home/DM button, server icons,
//! active/unread pill indicators, add server button, and user status widget.

use gpui::prelude::*;
use gpui::*;

use crate::theme;

/// Represents an individual server item rendered on the rail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerItem {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub is_active: bool,
    pub has_unread: bool,
}

/// Generates compact initials for a server name.
/// - Multiple words: first char of each word up to 3 (e.g. "Sharkord Plus" -> "SP")
/// - Single word: first 2 characters uppercase (e.g. "Rustaceans" -> "RU")
/// - Single char: 1 uppercase char (e.g. "A" -> "A")
/// - Empty: "?"
pub fn server_initials(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "?".to_string();
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() > 1 {
        words
            .into_iter()
            .take(3)
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

/// Calculates the height in pixels of the active/unread pill indicator on the left margin.
/// - Active server: 40px
/// - Inactive with unread messages: 8px
/// - Inactive without unread: 0px (hidden)
pub fn server_pill_height(is_active: bool, has_unread: bool) -> f32 {
    if is_active {
        40.0
    } else if has_unread {
        8.0
    } else {
        0.0
    }
}

/// Server rail view state.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerRailView {
    pub servers: Vec<ServerItem>,
    pub is_home_active: bool,
    pub current_user_name: String,
    pub current_user_status: String,
}

impl Default for ServerRailView {
    fn default() -> Self {
        Self {
            servers: Vec::new(),
            is_home_active: true,
            current_user_name: "User".to_string(),
            current_user_status: "online".to_string(),
        }
    }
}

impl ServerRailView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the Server Rail element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the Server Rail element tree with interactive server selection handlers.
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
        let home_pill_height = server_pill_height(self.is_home_active, false);

        let mut home_btn = div()
            .id("server_rail_home_btn")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(48.0))
            .h(px(48.0))
            .rounded_2xl()
            .bg(if self.is_home_active {
                theme::accent()
            } else {
                theme::bg_elevated()
            })
            .text_color(theme::text_primary())
            .child("💬");

        if let Some(ref mut cx) = cx {
            home_btn = home_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.on_server_click(None, cx);
                }),
            );
        }

        div()
            .flex()
            .flex_col()
            .items_center()
            .w(px(72.0))
            .h_full()
            .py_3()
            .bg(theme::bg_surface())
            .border_r_1()
            .border_color(theme::border_hairline())
            // 1. Home / DM button
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w_full()
                    .py_1()
                    // Pill indicator
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .w(px(4.0))
                            .h(px(home_pill_height))
                            .rounded_r_md()
                            .bg(theme::text_primary()),
                    )
                    // Home icon avatar
                    .child(home_btn),
            )
            // Separator divider
            .child(
                div()
                    .w(px(32.0))
                    .h(px(1.0))
                    .my_2()
                    .bg(theme::border_hairline()),
            )
            // 2. Server list
            .child(
                div()
                    .id("server_rail_list")
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .w_full()
                    .flex_1()
                    .overflow_y_scroll()
                    .children(self.servers.iter().map(|srv| {
                        let pill_h = server_pill_height(srv.is_active, srv.has_unread);
                        let initials = server_initials(&srv.name);
                        let srv_id = srv.id.clone();

                        let mut srv_btn = div()
                            .id(ElementId::Name(format!("server_item_{}", srv.id).into()))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(48.0))
                            .h(px(48.0))
                            .rounded_2xl()
                            .bg(if srv.is_active {
                                theme::accent()
                            } else {
                                theme::bg_elevated()
                            })
                            .text_color(theme::text_primary())
                            .child(initials);

                        if let Some(ref mut cx) = cx {
                            let srv_id_clone = srv_id.clone();
                            srv_btn = srv_btn.on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |view, _, _window, cx| {
                                    view.on_server_click(Some(srv_id_clone.clone()), cx);
                                }),
                            );
                        }

                        div()
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_full()
                            .py_1()
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .w(px(4.0))
                                    .h(px(pill_h))
                                    .rounded_r_md()
                                    .bg(theme::text_primary()),
                            )
                            .child(srv_btn)
                    })),
            )
            // 3. Add server button
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(48.0))
                    .h(px(48.0))
                    .my_2()
                    .rounded_full()
                    .bg(theme::bg_elevated())
                    .border_1()
                    .border_dashed()
                    .border_color(theme::border_hairline())
                    .text_color(theme::accent())
                    .child("+"),
            )
            // 4. User status widget at bottom
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(48.0))
                    .h(px(48.0))
                    .mt_auto()
                    .rounded_full()
                    .bg(theme::bg_surface_4())
                    .text_color(theme::text_primary())
                    .child(server_initials(&self.current_user_name))
                    // Status indicator dot
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .right_0()
                            .w(px(12.0))
                            .h(px(12.0))
                            .rounded_full()
                            .border_1()
                            .border_color(theme::bg_surface())
                            .bg(match self.current_user_status.as_str() {
                                "online" => theme::status_online(),
                                "idle" => theme::status_idle(),
                                "dnd" => theme::status_dnd(),
                                _ => theme::status_offline(),
                            }),
                    ),
            )
    }
}

impl Render for ServerRailView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
