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

/// Focused text input field in the authentication and server connection dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AuthInputField {
    #[default]
    Identity,
    Password,
    ServerUrl,
    Email,
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
    pub active_field: AuthInputField,
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
            active_field: AuthInputField::Identity,
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

    /// Sets the active input field for keyboard focus.
    pub fn set_active_field(&mut self, field: AuthInputField) {
        self.active_field = field;
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

    /// Renders the server connect form element headlessly (for tests or static rendering).
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the server connect form element with interactive click listeners.
    pub fn render_interactive(
        &self,
        cx: &mut Context<crate::ui::WorkspaceView>,
    ) -> impl IntoElement {
        self.render_internal(Some(cx))
    }

    fn render_internal(
        &self,
        cx: Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> impl IntoElement {
        let btn_label = if self.state.is_loading {
            "Connecting..."
        } else {
            "Connect to Server"
        };

        let is_active = self.state.active_field == AuthInputField::ServerUrl;
        let border_color = if is_active {
            theme::accent()
        } else {
            theme::border_subtle()
        };

        let url_display = if self.state.server_url.is_empty() {
            if is_active {
                "|".to_string()
            } else {
                "http://localhost:4991".to_string()
            }
        } else {
            if is_active {
                format!("{}|", self.state.server_url)
            } else {
                self.state.server_url.clone()
            }
        };

        let mut input_box = div()
            .id("server_url_input_box")
            .flex()
            .items_center()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(theme::bg_subsurface())
            .border_1()
            .border_color(border_color)
            .cursor_pointer()
            .child(
                div()
                    .text_sm()
                    .text_color(if self.state.server_url.is_empty() && !is_active {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(url_display),
            );

        let is_loading = self.state.is_loading;
        let mut action_btn = div()
            .id("server_connect_submit_btn")
            .px_4()
            .py_2()
            .rounded_md()
            .bg(if is_loading {
                theme::bg_surface_4()
            } else {
                theme::accent()
            })
            .hover(|s| {
                if is_loading {
                    s
                } else {
                    s.bg(theme::accent_hover())
                }
            })
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(if is_loading {
                theme::text_muted()
            } else {
                theme::text_primary()
            })
            .cursor_pointer()
            .child(btn_label);

        if let Some(cx) = cx {
            input_box = input_box.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, window, cx| {
                    view.auth_modal.active_field = AuthInputField::ServerUrl;
                    if let Some(ref handle) = view.focus_handle {
                        window.focus(handle);
                    }
                    cx.notify();
                }),
            );

            action_btn = action_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _, cx| {
                    view.submit_auth(cx);
                }),
            );
        }

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
                    .child(input_box)
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
                    .child(action_btn),
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
    fn render_tab_button(&self, tab: AuthTab) -> Div {
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

    /// Renders the full centered modal dialog headlessly (for unit tests).
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None, None)
    }

    /// Renders the modal with full interactive click listeners and focus hooks.
    pub fn render_interactive(
        &self,
        focus_handle: Option<&FocusHandle>,
        cx: &mut Context<crate::ui::WorkspaceView>,
    ) -> impl IntoElement {
        self.render_internal(focus_handle, Some(cx))
    }

    fn render_internal(
        &self,
        _focus_handle: Option<&FocusHandle>,
        mut cx: Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> impl IntoElement {
        let tabs = [AuthTab::ServerConnect, AuthTab::Login, AuthTab::Register];

        let content_element: AnyElement = match self.state.active_tab {
            AuthTab::ServerConnect => {
                let sc = ServerConnectView::new(self.state.clone());
                if let Some(ref mut c) = cx {
                    sc.render_interactive(c).into_any_element()
                } else {
                    sc.render_element().into_any_element()
                }
            }
            AuthTab::Login => {
                let submit_label = if self.state.is_loading {
                    "Signing in..."
                } else {
                    "Sign In"
                };

                let is_identity_active = self.state.active_field == AuthInputField::Identity;
                let identity_border = if is_identity_active {
                    theme::accent()
                } else {
                    theme::border_subtle()
                };
                let identity_display = if self.state.identity.is_empty() {
                    if is_identity_active {
                        "|".to_string()
                    } else {
                        "username or email".to_string()
                    }
                } else {
                    if is_identity_active {
                        format!("{}|", self.state.identity)
                    } else {
                        self.state.identity.clone()
                    }
                };

                let is_password_active = self.state.active_field == AuthInputField::Password;
                let password_border = if is_password_active {
                    theme::accent()
                } else {
                    theme::border_subtle()
                };
                let password_display = if self.state.password.is_empty() {
                    if is_password_active {
                        "|".to_string()
                    } else {
                        "••••••••".to_string()
                    }
                } else {
                    let bullets = "•".repeat(self.state.password.len());
                    if is_password_active {
                        format!("{}|", bullets)
                    } else {
                        bullets
                    }
                };

                let mut server_hint = div()
                    .id("auth_server_hint_pill")
                    .cursor_pointer()
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
                    );

                let mut identity_box = div()
                    .id("auth_identity_input_box")
                    .cursor_pointer()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(theme::bg_subsurface())
                    .border_1()
                    .border_color(identity_border)
                    .text_sm()
                    .text_color(if self.state.identity.is_empty() && !is_identity_active {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(identity_display);

                let mut password_box = div()
                    .id("auth_password_input_box")
                    .cursor_pointer()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(theme::bg_subsurface())
                    .border_1()
                    .border_color(password_border)
                    .text_sm()
                    .text_color(if self.state.password.is_empty() && !is_password_active {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(password_display);

                let mut remember_toggle = div()
                    .id("auth_remember_session_toggle")
                    .cursor_pointer()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .pt_1()
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
                    );

                let is_loading = self.state.is_loading;
                let mut submit_btn = div()
                    .id("auth_login_submit_btn")
                    .cursor_pointer()
                    .w_full()
                    .py_2()
                    .text_center()
                    .rounded_md()
                    .bg(if is_loading {
                        theme::bg_surface_4()
                    } else {
                        theme::accent()
                    })
                    .hover(|s| {
                        if is_loading {
                            s
                        } else {
                            s.bg(theme::accent_hover())
                        }
                    })
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(if is_loading {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(submit_label);

                if let Some(ref mut c) = cx {
                    server_hint = server_hint.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.set_tab(AuthTab::ServerConnect);
                            view.auth_modal.active_field = AuthInputField::ServerUrl;
                            view.auth_modal.clear_error();
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    identity_box = identity_box.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.active_field = AuthInputField::Identity;
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    password_box = password_box.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.active_field = AuthInputField::Password;
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    remember_toggle = remember_toggle.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, _, cx| {
                            view.auth_modal.toggle_remember_session();
                            cx.notify();
                        }),
                    );

                    submit_btn = submit_btn.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, _, cx| {
                            view.submit_auth(cx);
                        }),
                    );
                }

                div()
                    .flex()
                    .flex_col()
                    .gap_3p5()
                    .child(server_hint)
                    .child(
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
                            .child(identity_box),
                    )
                    .child(
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
                            .child(password_box),
                    )
                    .child(remember_toggle)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .pt_2()
                            .child(submit_btn),
                    )
                    .into_any_element()
            }
            AuthTab::Register => {
                let submit_label = if self.state.is_loading {
                    "Creating Account..."
                } else {
                    "Create Account"
                };

                let is_identity_active = self.state.active_field == AuthInputField::Identity;
                let identity_border = if is_identity_active {
                    theme::accent()
                } else {
                    theme::border_subtle()
                };
                let identity_display = if self.state.identity.is_empty() {
                    if is_identity_active {
                        "|".to_string()
                    } else {
                        "desired username".to_string()
                    }
                } else {
                    if is_identity_active {
                        format!("{}|", self.state.identity)
                    } else {
                        self.state.identity.clone()
                    }
                };

                let is_email_active = self.state.active_field == AuthInputField::Email;
                let email_border = if is_email_active {
                    theme::accent()
                } else {
                    theme::border_subtle()
                };
                let email_display = if self.state.email.is_empty() {
                    if is_email_active {
                        "|".to_string()
                    } else {
                        "email@example.com".to_string()
                    }
                } else {
                    if is_email_active {
                        format!("{}|", self.state.email)
                    } else {
                        self.state.email.clone()
                    }
                };

                let is_password_active = self.state.active_field == AuthInputField::Password;
                let password_border = if is_password_active {
                    theme::accent()
                } else {
                    theme::border_subtle()
                };
                let password_display = if self.state.password.is_empty() {
                    if is_password_active {
                        "|".to_string()
                    } else {
                        "••••••••".to_string()
                    }
                } else {
                    let bullets = "•".repeat(self.state.password.len());
                    if is_password_active {
                        format!("{}|", bullets)
                    } else {
                        bullets
                    }
                };

                let mut server_hint = div()
                    .id("auth_register_server_hint_pill")
                    .cursor_pointer()
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
                    );

                let mut identity_box = div()
                    .id("auth_register_identity_box")
                    .cursor_pointer()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(theme::bg_subsurface())
                    .border_1()
                    .border_color(identity_border)
                    .text_sm()
                    .text_color(if self.state.identity.is_empty() && !is_identity_active {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(identity_display);

                let mut email_box = div()
                    .id("auth_register_email_box")
                    .cursor_pointer()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(theme::bg_subsurface())
                    .border_1()
                    .border_color(email_border)
                    .text_sm()
                    .text_color(if self.state.email.is_empty() && !is_email_active {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(email_display);

                let mut password_box = div()
                    .id("auth_register_password_box")
                    .cursor_pointer()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(theme::bg_subsurface())
                    .border_1()
                    .border_color(password_border)
                    .text_sm()
                    .text_color(if self.state.password.is_empty() && !is_password_active {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(password_display);

                let is_loading = self.state.is_loading;
                let mut submit_btn = div()
                    .id("auth_register_submit_btn")
                    .cursor_pointer()
                    .w_full()
                    .py_2()
                    .text_center()
                    .rounded_md()
                    .bg(if is_loading {
                        theme::bg_surface_4()
                    } else {
                        theme::accent()
                    })
                    .hover(|s| {
                        if is_loading {
                            s
                        } else {
                            s.bg(theme::accent_hover())
                        }
                    })
                    .text_xs()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(if is_loading {
                        theme::text_muted()
                    } else {
                        theme::text_primary()
                    })
                    .child(submit_label);

                if let Some(ref mut c) = cx {
                    server_hint = server_hint.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.set_tab(AuthTab::ServerConnect);
                            view.auth_modal.active_field = AuthInputField::ServerUrl;
                            view.auth_modal.clear_error();
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    identity_box = identity_box.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.active_field = AuthInputField::Identity;
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    email_box = email_box.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.active_field = AuthInputField::Email;
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    password_box = password_box.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, window, cx| {
                            view.auth_modal.active_field = AuthInputField::Password;
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );

                    submit_btn = submit_btn.on_mouse_down(
                        MouseButton::Left,
                        c.listener(|view, _, _, cx| {
                            view.submit_auth(cx);
                        }),
                    );
                }

                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(server_hint)
                    .child(
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
                            .child(identity_box),
                    )
                    .child(
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
                            .child(email_box),
                    )
                    .child(
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
                            .child(password_box),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .pt_2()
                            .child(submit_btn),
                    )
                    .into_any_element()
            }
        };

        // Tab buttons container
        let tab_buttons: Vec<AnyElement> = tabs
            .iter()
            .map(|&t| {
                let mut tab_btn = self.render_tab_button(t);
                if let Some(ref mut c) = cx {
                    let tab_target = t;
                    tab_btn = tab_btn.on_mouse_down(
                        MouseButton::Left,
                        c.listener(move |view, _, window, cx| {
                            view.auth_modal.set_tab(tab_target);
                            view.auth_modal.active_field = match tab_target {
                                AuthTab::ServerConnect => AuthInputField::ServerUrl,
                                AuthTab::Login => AuthInputField::Identity,
                                AuthTab::Register => AuthInputField::Identity,
                            };
                            view.auth_modal.clear_error();
                            if let Some(ref handle) = view.focus_handle {
                                window.focus(handle);
                            }
                            cx.notify();
                        }),
                    );
                }
                tab_btn.into_any_element()
            })
            .collect();

        // Dialog container box adhering strictly to Pure Neutral Dark
        let mut dialog_body = div()
            .id("auth_modal_dialog_card")
            .on_mouse_down(MouseButton::Left, |_, _, _| {}) // stop click propagation to backdrop
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
                    .children(tab_buttons),
            );

        // Optional error banner
        if let Some(error_el) = self.render_error_banner() {
            dialog_body = dialog_body.child(div().mx_6().mb_3().child(error_el));
        }

        // Form content
        dialog_body = dialog_body.child(div().px_6().pb_6().child(content_element));

        // Full modal backdrop overlay
        div()
            .id("auth_modal_backdrop")
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
