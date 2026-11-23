//! Server Connect, Login, and Registration modal dialog views in GPUI.
//! Adheres strictly to the Pure Neutral Dark design specification (#0A0B0E base,
//! #111216 surface, #16171d subsurface, #3B82F6 accent, #F3F4F6 text).

use gpui::prelude::*;
use gpui::*;
use gpui_kit as _;
use serde::{Deserialize, Serialize};

use crate::theme;

/// Active tab in the authentication / connection modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AuthTab {
    ServerConnect,
    #[default]
    Login,
    Register,
}

impl AuthTab {
    /// Formatted user-facing display label for the tab button.
    pub fn label(&self) -> &'static str {
        match self {
            AuthTab::ServerConnect => "Connect",
            AuthTab::Login => "Login",
            AuthTab::Register => "Register",
        }
    }
}

/// State for the authentication and server connection dialog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthModalState {
    pub active_tab: AuthTab,
    pub server_url: String,
    pub identity: String,
    pub password: String,
    pub email: String,
    pub error_message: Option<String>,
    pub remember_session: bool,
    pub is_loading: bool,
}

impl Default for AuthModalState {
    fn default() -> Self {
        Self {
            active_tab: AuthTab::Login,
            server_url: "http://localhost:4991".to_string(),
            identity: String::new(),
            password: String::new(),
            email: String::new(),
            error_message: None,
            remember_session: true,
            is_loading: false,
        }
    }
}

impl AuthModalState {
    /// Creates a new `AuthModalState` initialized to default login values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the active tab.
    pub fn set_tab(&mut self, tab: AuthTab) {
        self.active_tab = tab;
    }

    /// Sets an error message banner.
    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
    }

    /// Clears any existing error message banner.
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Sets the loading indicator state.
    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    /// Updates the server URL.
    pub fn set_server_url(&mut self, url: impl Into<String>) {
        self.server_url = url.into();
    }

    /// Updates the identity (username or login handle).
    pub fn set_identity(&mut self, identity: impl Into<String>) {
        self.identity = identity.into();
    }

    /// Updates the password.
    pub fn set_password(&mut self, password: impl Into<String>) {
        self.password = password.into();
    }

    /// Updates the email address.
    pub fn set_email(&mut self, email: impl Into<String>) {
        self.email = email.into();
    }

    /// Sets whether to persist the session in `SessionVault`.
    pub fn set_remember_session(&mut self, remember: bool) {
        self.remember_session = remember;
    }

    /// Toggles the remember session preference.
    pub fn toggle_remember_session(&mut self) {
        self.remember_session = !self.remember_session;
    }
}

/// Dedicated view component for configuring and testing the connection to a Sharkord server URL.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerConnectView {
    pub state: AuthModalState,
}

impl ServerConnectView {
    /// Creates a new `ServerConnectView` with the provided modal state.
    pub fn new(state: AuthModalState) -> Self {
        Self { state }
    }

    /// Renders the server connect form element.
    pub fn render_element(&self) -> impl IntoElement {
        let btn_label = if self.state.is_loading {
            "Connecting..."
        } else {
            "Connect to Server"
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::text_secondary())
                            .child("SERVER ADDRESS / URL"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(theme::bg_subsurface())
                            .border_1()
                            .border_color(theme::border_subtle())
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if self.state.server_url.is_empty() {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(if self.state.server_url.is_empty() {
                                        "http://localhost:4991".to_string()
                                    } else {
                                        self.state.server_url.clone()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(
                                "Enter the full URL of your Sharkord instance (e.g. http://localhost:4991).",
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .pt_2()
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(if self.state.is_loading {
                                theme::bg_surface_4()
                            } else {
                                theme::accent()
                            })
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(if self.state.is_loading {
                                theme::text_muted()
                            } else {
                                theme::text_primary()
                            })
                            .cursor_pointer()
                            .child(btn_label),
                    ),
            )
    }
}

impl Render for ServerConnectView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}

/// Primary authentication modal dialog view supporting Server Connect, Login,
/// and Registration tabs adhering strictly to the Pure Neutral Dark design system.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthModalView {
    pub state: AuthModalState,
}

impl AuthModalView {
    /// Creates a new `AuthModalView` with the provided modal state.
    pub fn new(state: AuthModalState) -> Self {
        Self { state }
    }

    /// Renders tab button element.
    fn render_tab_button(&self, tab: AuthTab) -> impl IntoElement {
        let is_active = self.state.active_tab == tab;
        let (bg_color, text_color, font_weight) = if is_active {
            (
                theme::bg_elevated(),
                theme::text_primary(),
                gpui::FontWeight::BOLD,
            )
        } else {
            (
                theme::bg_surface(),
                theme::text_muted(),
                gpui::FontWeight::MEDIUM,
            )
        };

        div()
            .flex_1()
            .py_2()
            .text_center()
            .rounded_md()
            .bg(bg_color)
            .text_xs()
            .font_weight(font_weight)
            .text_color(text_color)
            .cursor_pointer()
            .child(tab.label())
    }

    /// Renders error banner if an error message is set.
    fn render_error_banner(&self) -> Option<impl IntoElement> {
        let err = self.state.error_message.as_ref()?;
        Some(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(hsla(0.0, 0.65, 0.20, 0.40))
                .border_1()
                .border_color(theme::status_dnd())
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::status_dnd())
                        .child("⚠️"),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::text_primary())
                        .child(err.clone()),
                ),
        )
    }

    /// Renders the full centered modal dialog with backdrop overlay.
    pub fn render_element(&self) -> impl IntoElement {
        let tabs = [AuthTab::ServerConnect, AuthTab::Login, AuthTab::Register];

        let content_element: AnyElement = match self.state.active_tab {
            AuthTab::ServerConnect => {
                ServerConnectView::new(self.state.clone()).render_element().into_any_element()
            }
            AuthTab::Login => {
                let submit_label = if self.state.is_loading {
                    "Signing in..."
                } else {
                    "Sign In"
                };

                let password_display = if self.state.password.is_empty() {
                    "••••••••".to_string()
                } else {
                    "•".repeat(self.state.password.len())
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_3p5()
                    .child(
                        // Server URL hint / pill
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(theme::bg_subsurface())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Connecting to:"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::accent_hover())
                                    .child(self.state.server_url.clone()),
                            ),
                    )
                    .child(
                        // Identity input
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("USERNAME OR EMAIL"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .text_sm()
                                    .text_color(if self.state.identity.is_empty() {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(if self.state.identity.is_empty() {
                                        "username or email".to_string()
                                    } else {
                                        self.state.identity.clone()
                                    }),
                            ),
                    )
                    .child(
                        // Password input
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("PASSWORD"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .text_sm()
                                    .text_color(if self.state.password.is_empty() {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(password_display),
                            ),
                    )
                    .child(
                        // Remember session toggle
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .pt_1()
                            .cursor_pointer()
                            .child(
                                div()
                                    .w(px(16.0))
                                    .h(px(16.0))
                                    .rounded_sm()
                                    .bg(if self.state.remember_session {
                                        theme::accent()
                                    } else {
                                        theme::bg_surface_4()
                                    })
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_xs()
                                    .text_color(theme::text_primary())
                                    .child(if self.state.remember_session {
                                        "✓"
                                    } else {
                                        ""
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child("Remember session (save to vault)"),
                            ),
                    )
                    .child(
                        // Submit button
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .pt_2()
                            .child(
                                div()
                                    .w_full()
                                    .py_2()
                                    .text_center()
                                    .rounded_md()
                                    .bg(if self.state.is_loading {
                                        theme::bg_surface_4()
                                    } else {
                                        theme::accent()
                                    })
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(if self.state.is_loading {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .cursor_pointer()
                                    .child(submit_label),
                            ),
                    )
                    .into_any_element()
            }
            AuthTab::Register => {
                let submit_label = if self.state.is_loading {
                    "Creating Account..."
                } else {
                    "Create Account"
                };

                let password_display = if self.state.password.is_empty() {
                    "••••••••".to_string()
                } else {
                    "•".repeat(self.state.password.len())
                };

                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        // Server URL hint
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(theme::bg_subsurface())
                            .border_1()
                            .border_color(theme::border_hairline())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Registering on:"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme::accent_hover())
                                    .child(self.state.server_url.clone()),
                            ),
                    )
                    .child(
                        // Username input
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("USERNAME"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .text_sm()
                                    .text_color(if self.state.identity.is_empty() {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(if self.state.identity.is_empty() {
                                        "desired username".to_string()
                                    } else {
                                        self.state.identity.clone()
                                    }),
                            ),
                    )
                    .child(
                        // Email input
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("EMAIL ADDRESS"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .text_sm()
                                    .text_color(if self.state.email.is_empty() {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(if self.state.email.is_empty() {
                                        "email@example.com".to_string()
                                    } else {
                                        self.state.email.clone()
                                    }),
                            ),
                    )
                    .child(
                        // Password input
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::text_secondary())
                                    .child("PASSWORD"),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .text_sm()
                                    .text_color(if self.state.password.is_empty() {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(password_display),
                            ),
                    )
                    .child(
                        // Submit button
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .pt_2()
                            .child(
                                div()
                                    .w_full()
                                    .py_2()
                                    .text_center()
                                    .rounded_md()
                                    .bg(if self.state.is_loading {
                                        theme::bg_surface_4()
                                    } else {
                                        theme::accent()
                                    })
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(if self.state.is_loading {
                                        theme::text_muted()
                                    } else {
                                        theme::text_primary()
                                    })
                                    .cursor_pointer()
                                    .child(submit_label),
                            ),
                    )
                    .into_any_element()
            }
        };

        // Dialog container box adhering strictly to Pure Neutral Dark
        let mut dialog_body = div()
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
                                    .child("Welcome to Sharkord"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child("Connect to an instance or authenticate your account"),
                            ),
                    ),
            )
            // Tabs navigation pill
            .child(
                div()
                    .flex()
                    .flex_row()
                    .mx_6()
                    .mb_4()
                    .p_1()
                    .rounded_lg()
                    .bg(theme::bg_surface())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .children(tabs.iter().map(|&t| self.render_tab_button(t))),
            );

        // Optional error banner
        if let Some(error_el) = self.render_error_banner() {
            dialog_body = dialog_body.child(div().mx_6().mb_3().child(error_el));
        }

        // Form content
        dialog_body = dialog_body.child(div().px_6().pb_6().child(content_element));

        // Full modal backdrop overlay
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
            .child(dialog_body)
    }
}

impl Render for AuthModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}
