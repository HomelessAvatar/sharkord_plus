//! Server Settings modal dialog view in GPUI.
//! Adheres strictly to Pure Neutral Dark design specification (#0A0B0E base,
//! #111216 surface, #16171d subsurface, #1c1e24 elevated, #3B82F6 accent, #F3F4F6 text).
//! Provides sidebar navigation tab rail (Overview, Roles, Members, Invites),
//! tab panels with server configuration, role permissions, member directory, and ESC dismissal.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::types::{Role, User};

use crate::theme;

/// Navigation tabs in the Server Settings modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ServerSettingsTab {
    #[default]
    Overview,
    Roles,
    Members,
    Invites,
}

impl ServerSettingsTab {
    /// User-facing display label for the tab button.
    pub fn label(&self) -> &'static str {
        match self {
            ServerSettingsTab::Overview => "Overview",
            ServerSettingsTab::Roles => "Roles",
            ServerSettingsTab::Members => "Members",
            ServerSettingsTab::Invites => "Invites",
        }
    }

    /// Symbol / icon indicator for the tab button.
    pub fn icon(&self) -> &'static str {
        match self {
            ServerSettingsTab::Overview => "⚙",
            ServerSettingsTab::Roles => "🛡",
            ServerSettingsTab::Members => "👥",
            ServerSettingsTab::Invites => "✉",
        }
    }
}

/// State for the Server Settings modal dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSettingsModalState {
    pub is_open: bool,
    pub active_tab: ServerSettingsTab,
    pub server_name: String,
    pub roles: Vec<Role>,
    pub members: Vec<User>,
}

impl Default for ServerSettingsModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: ServerSettingsTab::Overview,
            server_name: "Sharkord Server".to_string(),
            roles: Vec::new(),
            members: Vec::new(),
        }
    }
}

impl ServerSettingsModalState {
    /// Creates a new closed `ServerSettingsModalState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the server settings modal.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the server settings modal.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Switches the active navigation tab.
    pub fn set_tab(&mut self, tab: ServerSettingsTab) {
        self.active_tab = tab;
    }

    /// Sets the server name.
    pub fn set_server_name(&mut self, name: impl Into<String>) {
        self.server_name = name.into();
    }

    /// Sets the server roles list.
    pub fn set_roles(&mut self, roles: Vec<Role>) {
        self.roles = roles;
    }

    /// Sets the server members list.
    pub fn set_members(&mut self, members: Vec<User>) {
        self.members = members;
    }

    /// Resets modal state to defaults and closes it.
    pub fn reset(&mut self) {
        self.is_open = false;
        self.active_tab = ServerSettingsTab::Overview;
        self.server_name.clear();
        self.roles.clear();
        self.members.clear();
    }
}

/// View component rendering the full Server Settings modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerSettingsModalView {
    pub state: ServerSettingsModalState,
}

impl ServerSettingsModalView {
    /// Creates a new `ServerSettingsModalView` from the provided state.
    pub fn new(state: ServerSettingsModalState) -> Self {
        Self { state }
    }

    /// Renders the Server Settings modal element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the Server Settings modal element tree with interactive handlers.
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
        let tabs = [
            ServerSettingsTab::Overview,
            ServerSettingsTab::Roles,
            ServerSettingsTab::Members,
            ServerSettingsTab::Invites,
        ];

        // 1. Navigation rail on the left
        let mut nav_rail = div()
            .flex()
            .flex_col()
            .w(px(200.0))
            .h_full()
            .bg(theme::bg_subsurface())
            .border_r_1()
            .border_color(theme::border_hairline())
            .p_3()
            // Server Header / Badge
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_2p5()
                    .mb_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(28.0))
                            .h(px(28.0))
                            .rounded_md()
                            .bg(theme::accent())
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(
                                self.state
                                    .server_name
                                    .chars()
                                    .next()
                                    .unwrap_or('S')
                                    .to_string(),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(self.state.server_name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Server Settings"),
                            ),
                    ),
            )
            // Tabs List
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .children(tabs.iter().map(|&t| {
                        let is_active = self.state.active_tab == t;
                        let mut tab_btn = div()
                            .cursor_pointer()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if is_active {
                                theme::bg_elevated()
                            } else {
                                theme::bg_subsurface()
                            })
                            .text_xs()
                            .font_weight(if is_active {
                                gpui::FontWeight::BOLD
                            } else {
                                gpui::FontWeight::MEDIUM
                            })
                            .text_color(if is_active {
                                theme::text_primary()
                            } else {
                                theme::text_muted()
                            })
                            .hover(|s| {
                                s.text_color(theme::text_primary())
                                    .bg(theme::bg_surface_4())
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_active {
                                        theme::accent()
                                    } else {
                                        theme::text_muted()
                                    })
                                    .child(t.icon()),
                            )
                            .child(div().child(t.label()));

                        if let Some(ref mut cx) = cx {
                            tab_btn = tab_btn.on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |view, _, _window, cx| {
                                    view.server_settings_modal.set_tab(t);
                                    cx.notify();
                                }),
                            );
                        }

                        tab_btn
                    })),
            )
            .child(div().flex_1()); // Spacer

        // Close button in bottom left
        let mut close_pill = div()
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(theme::bg_surface_4())
            .border_1()
            .border_color(theme::border_subtle())
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(theme::text_secondary())
            .hover(|s| s.text_color(theme::text_primary()).bg(theme::bg_elevated()))
            .child(div().child("Close Settings"))
            .child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_sm()
                    .bg(theme::bg_elevated())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::text_muted())
                    .child("ESC"),
            );

        if let Some(ref mut cx) = cx {
            close_pill = close_pill.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_server_settings_modal(cx);
                }),
            );
        }

        nav_rail = nav_rail.child(close_pill);

        // 2. Active Tab Content Panel
        let content_panel: AnyElement = match self.state.active_tab {
            ServerSettingsTab::Overview => {
                let mut save_overview_btn = div()
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

                if let Some(ref mut cx) = cx {
                    save_overview_btn = save_overview_btn.on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _, _window, cx| {
                            view.close_server_settings_modal(cx);
                        }),
                    );
                }

                div()
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child("Server Overview"),
                    )
                    // Server icon and name row
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(72.0))
                                    .h(px(72.0))
                                    .rounded_full()
                                    .bg(theme::bg_surface_4())
                                    .border_2()
                                    .border_color(theme::border_subtle())
                                    .text_xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(
                                        self.state
                                            .server_name
                                            .chars()
                                            .next()
                                            .unwrap_or('S')
                                            .to_string(),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme::text_secondary())
                                            .child("SERVER NAME"),
                                    )
                                    .child(
                                        div()
                                            .w(px(320.0))
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(theme::bg_subsurface())
                                            .border_1()
                                            .border_color(theme::border_subtle())
                                            .text_sm()
                                            .text_color(theme::text_primary())
                                            .child(self.state.server_name.clone()),
                                    ),
                            ),
                    )
                    // Capabilities badges
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
                                    .child("SERVER CAPABILITIES"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_2()
                                    .child(
                                        div()
                                            .px_2p5()
                                            .py_1()
                                            .rounded_full()
                                            .bg(theme::bg_surface_4())
                                            .border_1()
                                            .border_color(theme::border_hairline())
                                            .text_xs()
                                            .text_color(theme::text_primary())
                                            .child("⚡ Pure Neutral Dark"),
                                    )
                                    .child(
                                        div()
                                            .px_2p5()
                                            .py_1()
                                            .rounded_full()
                                            .bg(theme::bg_surface_4())
                                            .border_1()
                                            .border_color(theme::border_hairline())
                                            .text_xs()
                                            .text_color(theme::text_primary())
                                            .child("🔒 Self-Hosted"),
                                    )
                                    .child(
                                        div()
                                            .px_2p5()
                                            .py_1()
                                            .rounded_full()
                                            .bg(theme::bg_surface_4())
                                            .border_1()
                                            .border_color(theme::border_hairline())
                                            .text_xs()
                                            .text_color(theme::text_primary())
                                            .child("🔊 Voice SFU"),
                                    ),
                            ),
                    )
                    .child(div().flex_1())
                    // Save footer
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .pt_4()
                            .border_t_1()
                            .border_color(theme::border_hairline())
                            .child(save_overview_btn),
                    )
                    .into_any_element()
            }
            ServerSettingsTab::Roles => {
                let role_elements: Vec<AnyElement> = if self.state.roles.is_empty() {
                    vec![div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .gap_2()
                        .child(
                            div()
                                .text_2xl()
                                .text_color(theme::text_muted())
                                .child("🛡"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::text_secondary())
                                .child("No custom roles configured"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Create roles to organize members and manage channel permissions"),
                        )
                        .into_any_element()]
                } else {
                    self.state
                        .roles
                        .iter()
                        .map(|role| {
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p_3()
                                .rounded_lg()
                                .bg(theme::bg_subsurface())
                                .border_1()
                                .border_color(theme::border_hairline())
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2p5()
                                        .child(
                                            div()
                                                .w(px(12.0))
                                                .h(px(12.0))
                                                .rounded_full()
                                                .bg(theme::accent()),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(theme::text_primary())
                                                .child(role.name.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(theme::bg_surface_4())
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child(format!(
                                                    "{} permissions",
                                                    role.permissions.len()
                                                )),
                                        ),
                                )
                                .into_any_element()
                        })
                        .collect()
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
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
                                            .child("Roles"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_muted())
                                            .child("Configure member groups and administrative permissions"),
                                    ),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .px_3()
                                    .py_1p5()
                                    .rounded_md()
                                    .bg(theme::accent())
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_primary())
                                    .child("+ Create Role"),
                            ),
                    )
                    .child(div().flex().flex_col().gap_2().children(role_elements))
                    .into_any_element()
            }
            ServerSettingsTab::Members => {
                let member_elements: Vec<AnyElement> = if self.state.members.is_empty() {
                    vec![div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .gap_2()
                        .child(
                            div()
                                .text_2xl()
                                .text_color(theme::text_muted())
                                .child("👥"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::text_secondary())
                                .child("No members found"),
                        )
                        .into_any_element()]
                } else {
                    self.state
                        .members
                        .iter()
                        .map(|member| {
                            let initial = member.name.chars().next().unwrap_or('?').to_string();
                            let status = member.status.as_deref().unwrap_or("offline");

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p_2p5()
                                .rounded_lg()
                                .bg(theme::bg_subsurface())
                                .border_1()
                                .border_color(theme::border_hairline())
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_2p5()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .w(px(28.0))
                                                .h(px(28.0))
                                                .rounded_full()
                                                .bg(theme::bg_surface_4())
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child(initial),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                                        .text_color(theme::text_primary())
                                                        .child(member.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::text_muted())
                                                        .child(status.to_string()),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_full()
                                        .bg(theme::bg_surface_4())
                                        .border_1()
                                        .border_color(theme::border_subtle())
                                        .text_xs()
                                        .text_color(theme::text_secondary())
                                        .child("Member"),
                                )
                                .into_any_element()
                        })
                        .collect()
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
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
                                            .child("Server Members"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme::text_muted())
                                            .child(format!("{} Members", self.state.members.len())),
                                    ),
                            ),
                    )
                    .child(div().flex().flex_col().gap_2().children(member_elements))
                    .into_any_element()
            }
            ServerSettingsTab::Invites => div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
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
                                        .child("Instant Invites"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child("Manage active share links for this server"),
                                ),
                        )
                        .child(
                            div()
                                .cursor_pointer()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(theme::accent())
                                .text_xs()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child("+ Create Invite"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .gap_2()
                        .child(
                            div()
                                .text_2xl()
                                .text_color(theme::text_muted())
                                .child("✉"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::text_secondary())
                                .child("No active invite links"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Generate an invite link to invite friends to this server"),
                        ),
                )
                .into_any_element(),
        };

        let dialog_card = div()
            .flex()
            .flex_row()
            .w(px(760.0))
            .h(px(500.0))
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .rounded_xl()
            .shadow_lg()
            .overflow_hidden()
            .child(nav_rail)
            .child(
                div()
                    .id("server_settings_content_scroll")
                    .flex_1()
                    .h_full()
                    .p_6()
                    .bg(theme::bg_surface())
                    .overflow_y_scroll()
                    .child(content_panel),
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
                    view.close_server_settings_modal(cx);
                }),
            );
        }

        backdrop
    }
}

impl Render for ServerSettingsModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
