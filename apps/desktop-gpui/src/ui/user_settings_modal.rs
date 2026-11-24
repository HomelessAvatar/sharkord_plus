//! User Settings modal dialog view in GPUI.
//! Adheres strictly to Pure Neutral Dark design specification (#0A0B0E base,
//! #111216 surface, #16171d subsurface, #1c1e24 elevated, #3B82F6 accent, #F3F4F6 text).
//! Provides sidebar navigation tab rail (Account, Voice & Video, Appearance, Keybinds, Notifications),
//! tab panels with device selectors, sliders, switches, inputs, and ESC dismissal.

use gpui::prelude::*;
use gpui::*;
use serde::{Deserialize, Serialize};
use sharkord_core::voice::audio::AudioDevice;

use crate::theme;

/// Navigation tabs in the User Settings modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SettingsTab {
    #[default]
    Account,
    VoiceAndVideo,
    Appearance,
    Keybinds,
    Notifications,
}

impl SettingsTab {
    /// User-facing display label for the tab button.
    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::Account => "Account",
            SettingsTab::VoiceAndVideo => "Voice & Video",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::Keybinds => "Keybinds",
            SettingsTab::Notifications => "Notifications",
        }
    }

    /// Symbol / icon indicator for the tab button.
    pub fn icon(&self) -> &'static str {
        match self {
            SettingsTab::Account => "👤",
            SettingsTab::VoiceAndVideo => "🎙",
            SettingsTab::Appearance => "🎨",
            SettingsTab::Keybinds => "⌨",
            SettingsTab::Notifications => "🔔",
        }
    }

    /// Short description of the tab contents.
    pub fn description(&self) -> &'static str {
        match self {
            SettingsTab::Account => "Manage your profile, username, email, and security settings.",
            SettingsTab::VoiceAndVideo => {
                "Configure input and output audio devices, volumes, and sensitivity."
            }
            SettingsTab::Appearance => "Customize color theme, scaling, and chat display layout.",
            SettingsTab::Keybinds => "Set up hotkeys, shortcuts, and push-to-talk triggers.",
            SettingsTab::Notifications => {
                "Manage desktop alerts, sound effects, and unread badges."
            }
        }
    }
}

/// Input transmission mode for audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InputMode {
    #[default]
    VoiceActivity,
    PushToTalk,
}

impl InputMode {
    /// User-facing display label for the input mode.
    pub fn label(&self) -> &'static str {
        match self {
            InputMode::VoiceActivity => "Voice Activity",
            InputMode::PushToTalk => "Push to Talk",
        }
    }
}

/// State for the User Settings modal dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettingsState {
    pub is_open: bool,
    pub active_tab: SettingsTab,

    // Account details
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub bio: String,
    pub avatar_url: Option<String>,

    // Voice & Video settings
    pub input_devices: Vec<AudioDevice>,
    pub output_devices: Vec<AudioDevice>,
    pub selected_input_device: Option<String>,
    pub selected_output_device: Option<String>,
    pub input_volume: f32,
    pub output_volume: f32,
    pub input_mode: InputMode,
    pub vad_threshold: f32,
    pub noise_suppression: bool,
    pub echo_cancellation: bool,
    pub auto_gain_control: bool,
    pub is_testing_mic: bool,
    pub test_mic_level: f32,

    // Appearance settings
    pub theme_name: String,
    pub compact_mode: bool,
    pub font_size: u32,

    // Keybinds settings
    pub ptt_keybind: String,
    pub toggle_mute_keybind: String,
    pub toggle_deafen_keybind: String,

    // Notifications settings
    pub desktop_notifications: bool,
    pub sound_notifications: bool,
    pub unread_badge: bool,
}

impl Default for UserSettingsState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: SettingsTab::Account,
            username: "user".to_string(),
            display_name: "Sharkord User".to_string(),
            email: "user@sharkord.local".to_string(),
            bio: "Exploring the native GPUI desktop experience.".to_string(),
            avatar_url: None,

            input_devices: Vec::new(),
            output_devices: Vec::new(),
            selected_input_device: Some("Default Microphone".to_string()),
            selected_output_device: Some("Default Speakers".to_string()),
            input_volume: 100.0,
            output_volume: 100.0,
            input_mode: InputMode::VoiceActivity,
            vad_threshold: 0.5,
            noise_suppression: true,
            echo_cancellation: true,
            auto_gain_control: true,
            is_testing_mic: false,
            test_mic_level: 0.0,

            theme_name: "Pure Neutral Dark".to_string(),
            compact_mode: false,
            font_size: 14,

            ptt_keybind: "ControlLeft".to_string(),
            toggle_mute_keybind: "Ctrl+Shift+M".to_string(),
            toggle_deafen_keybind: "Ctrl+Shift+D".to_string(),

            desktop_notifications: true,
            sound_notifications: true,
            unread_badge: true,
        }
    }
}

impl UserSettingsState {
    /// Creates a new closed `UserSettingsState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the user settings modal.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the user settings modal.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Switches the active navigation tab.
    pub fn set_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
    }

    /// Sets the input volume (0.0 - 200.0).
    pub fn set_input_volume(&mut self, volume: f32) {
        self.input_volume = volume.clamp(0.0, 200.0);
    }

    /// Sets the output volume (0.0 - 200.0).
    pub fn set_output_volume(&mut self, volume: f32) {
        self.output_volume = volume.clamp(0.0, 200.0);
    }

    /// Sets the input transmission mode.
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    /// Sets the voice activity detection threshold (0.0 - 1.0).
    pub fn set_vad_threshold(&mut self, threshold: f32) {
        self.vad_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Sets noise suppression toggle.
    pub fn set_noise_suppression(&mut self, enabled: bool) {
        self.noise_suppression = enabled;
    }

    /// Toggles noise suppression.
    pub fn toggle_noise_suppression(&mut self) {
        self.noise_suppression = !self.noise_suppression;
    }

    /// Sets echo cancellation toggle.
    pub fn set_echo_cancellation(&mut self, enabled: bool) {
        self.echo_cancellation = enabled;
    }

    /// Toggles echo cancellation.
    pub fn toggle_echo_cancellation(&mut self) {
        self.echo_cancellation = !self.echo_cancellation;
    }

    /// Sets automatic gain control toggle.
    pub fn set_auto_gain_control(&mut self, enabled: bool) {
        self.auto_gain_control = enabled;
    }

    /// Toggles automatic gain control.
    pub fn toggle_auto_gain_control(&mut self) {
        self.auto_gain_control = !self.auto_gain_control;
    }

    /// Sets compact mode toggle.
    pub fn set_compact_mode(&mut self, enabled: bool) {
        self.compact_mode = enabled;
    }

    /// Toggles compact mode.
    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
    }

    /// Sets desktop notifications toggle.
    pub fn set_desktop_notifications(&mut self, enabled: bool) {
        self.desktop_notifications = enabled;
    }

    /// Toggles desktop notifications.
    pub fn toggle_desktop_notifications(&mut self) {
        self.desktop_notifications = !self.desktop_notifications;
    }

    /// Sets sound notifications toggle.
    pub fn set_sound_notifications(&mut self, enabled: bool) {
        self.sound_notifications = enabled;
    }

    /// Toggles sound notifications.
    pub fn toggle_sound_notifications(&mut self) {
        self.sound_notifications = !self.sound_notifications;
    }

    /// Sets unread badge toggle.
    pub fn set_unread_badge(&mut self, enabled: bool) {
        self.unread_badge = enabled;
    }

    /// Toggles unread badge.
    pub fn toggle_unread_badge(&mut self) {
        self.unread_badge = !self.unread_badge;
    }

    /// Sets push to talk keybind.
    pub fn set_ptt_keybind(&mut self, key: impl Into<String>) {
        self.ptt_keybind = key.into();
    }

    /// Sets username.
    pub fn set_username(&mut self, name: impl Into<String>) {
        self.username = name.into();
    }

    /// Sets display name.
    pub fn set_display_name(&mut self, name: impl Into<String>) {
        self.display_name = name.into();
    }

    /// Sets user email.
    pub fn set_email(&mut self, email: impl Into<String>) {
        self.email = email.into();
    }

    /// Sets user bio.
    pub fn set_bio(&mut self, bio: impl Into<String>) {
        self.bio = bio.into();
    }

    /// Resets all settings to defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// View component rendering the full User Settings modal dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct UserSettingsModalView {
    pub state: UserSettingsState,
}

impl UserSettingsModalView {
    /// Creates a new `UserSettingsModalView` from state.
    pub fn new(state: UserSettingsState) -> Self {
        Self { state }
    }

    /// Renders the modal element tree without interactive handlers.
    pub fn render_element(&self) -> impl IntoElement {
        self.render_internal(None)
    }

    /// Renders the modal element tree with interactive handlers.
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
            SettingsTab::Account,
            SettingsTab::VoiceAndVideo,
            SettingsTab::Appearance,
            SettingsTab::Keybinds,
            SettingsTab::Notifications,
        ];

        // 1. Left navigation sidebar rail (200px fixed width)
        let mut nav_rail = div()
            .flex()
            .flex_col()
            .w(px(200.0))
            .h_full()
            .bg(theme::bg_subsurface())
            .border_r_1()
            .border_color(theme::border_hairline())
            .p_3()
            // Header
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
                            .child("⚙"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child("USER SETTINGS"),
                    ),
            );

        // Tab list items
        let mut tab_items = div().flex().flex_col().gap_1();
        for &t in &tabs {
            let is_active = self.state.active_tab == t;
            let mut btn = div()
                .id(ElementId::Name(format!("settings_tab_{:?}", t).into()))
                .cursor_pointer()
                .flex()
                .flex_row()
                .items_center()
                .gap_2p5()
                .px_2p5()
                .py_2()
                .rounded_md()
                .bg(if is_active {
                    theme::bg_elevated()
                } else {
                    hsla(0.0, 0.0, 0.0, 0.0)
                })
                .text_color(if is_active {
                    theme::text_primary()
                } else {
                    theme::text_secondary()
                })
                .font_weight(if is_active {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::NORMAL
                })
                .hover(|s| {
                    s.bg(theme::bg_surface_4())
                        .text_color(theme::text_primary())
                })
                .child(
                    div()
                        .w(px(18.0))
                        .flex()
                        .justify_center()
                        .child(t.icon()),
                )
                .child(div().text_sm().child(t.label()));

            if let Some(ref mut cx) = cx {
                btn = btn.on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |view, _, _window, cx| {
                        view.user_settings_modal.set_tab(t);
                        cx.notify();
                    }),
                );
            }
            tab_items = tab_items.child(btn);
        }

        // Bottom close button on rail
        let mut close_rail_btn = div()
            .id("settings_rail_close_btn")
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap_1p5()
            .mt_auto()
            .p_2()
            .rounded_md()
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_hairline())
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme::text_secondary())
            .hover(|s| {
                s.bg(theme::bg_surface_4())
                    .text_color(theme::text_primary())
            })
            .child(div().child("✕"))
            .child(div().child("Close Settings"));

        if let Some(ref mut cx) = cx {
            close_rail_btn = close_rail_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_user_settings_modal(cx);
                }),
            );
        }

        nav_rail = nav_rail.child(tab_items).child(close_rail_btn);

        // 2. Active Tab Content Panel
        let content_element = match self.state.active_tab {
            SettingsTab::Account => self.render_account_tab(&mut cx),
            SettingsTab::VoiceAndVideo => self.render_voice_and_video_tab(&mut cx),
            SettingsTab::Appearance => self.render_appearance_tab(&mut cx),
            SettingsTab::Keybinds => self.render_keybinds_tab(&mut cx),
            SettingsTab::Notifications => self.render_notifications_tab(&mut cx),
        };

        // Right content panel header
        let mut close_header_btn = div()
            .id("settings_header_close_btn")
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .w(px(28.0))
            .h(px(28.0))
            .rounded_full()
            .bg(theme::bg_elevated())
            .border_1()
            .border_color(theme::border_hairline())
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(theme::text_secondary())
            .hover(|s| {
                s.bg(theme::bg_surface_4())
                    .text_color(theme::text_primary())
            })
            .child("✕");

        if let Some(ref mut cx) = cx {
            close_header_btn = close_header_btn.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_user_settings_modal(cx);
                }),
            );
        }

        let right_panel = div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .bg(theme::bg_surface())
            // Panel Header
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_4()
                    .border_b_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(self.state.active_tab.label()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(self.state.active_tab.description()),
                            ),
                    )
                    .child(close_header_btn),
            )
            // Scrollable Panel Content
            .child(
                div()
                    .id("user_settings_content_scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scroll()
                    .p_6()
                    .gap_5()
                    .child(content_element),
            );

        // Modal backdrop overlay (centers the 800px x 560px modal dialog)
        let mut modal_backdrop = div()
            .id("user_settings_backdrop")
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x000000AA))
            // Click outside to close
            .child(
                div()
                    .id("user_settings_modal_window")
                    .on_mouse_down(MouseButton::Left, |_, _, _| {})
                    .flex()
                    .flex_row()
                    .w(px(800.0))
                    .h(px(560.0))
                    .rounded_xl()
                    .bg(theme::bg_surface())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .shadow_xl()
                    .overflow_hidden()
                    .child(nav_rail)
                    .child(right_panel),
            );

        if let Some(ref mut cx) = cx {
            modal_backdrop = modal_backdrop.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.close_user_settings_modal(cx);
                }),
            );
        }

        modal_backdrop
    }

    /// Renders the Account settings tab.
    fn render_account_tab(
        &self,
        _cx: &mut Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> Div {
        let initials = self
            .state
            .display_name
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase();
        let initials_display = if initials.is_empty() {
            "U".to_string()
        } else {
            initials
        };

        div()
            .flex()
            .flex_col()
            .gap_4()
            // Profile Card Preview
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_4()
                    .p_4()
                    .rounded_lg()
                    .bg(theme::bg_elevated())
                    .border_1()
                    .border_color(theme::border_hairline())
                    .child(
                        div()
                            .relative()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(56.0))
                            .h(px(56.0))
                            .rounded_full()
                            .bg(theme::accent())
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .child(initials_display)
                            .child(
                                div()
                                    .absolute()
                                    .bottom_0()
                                    .right_0()
                                    .w(px(14.0))
                                    .h(px(14.0))
                                    .rounded_full()
                                    .bg(rgb(theme::STATUS_ONLINE))
                                    .border_2()
                                    .border_color(theme::bg_elevated()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(self.state.display_name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_secondary())
                                    .child(format!("@{}", self.state.username)),
                            ),
                    ),
            )
            // Account Details Form
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .rounded_lg()
                    .bg(theme::bg_elevated())
                    .border_1()
                    .border_color(theme::border_hairline())
                    // Display Name Field
                    .child(render_setting_input("Display Name", &self.state.display_name))
                    // Username Field
                    .child(render_setting_input("Username", &self.state.username))
                    // Email Field
                    .child(render_setting_input("Email", &self.state.email))
                    // Bio Field
                    .child(render_setting_input("About Me", &self.state.bio)),
            )
            // Action Buttons
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .child(
                        div()
                            .cursor_pointer()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(theme::accent())
                            .text_xs()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme::text_primary())
                            .child("Edit User Profile"),
                    )
                    .child(
                        div()
                            .cursor_pointer()
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .bg(theme::bg_surface_4())
                            .text_xs()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::text_primary())
                            .child("Change Password"),
                    ),
            )
    }

    /// Renders the Voice & Video settings tab with device selects, volume sliders, VAD threshold, and switches.
    fn render_voice_and_video_tab(
        &self,
        cx: &mut Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> Div {
        let input_vol_text = format!("{:.0}%", self.state.input_volume);
        let output_vol_text = format!("{:.0}%", self.state.output_volume);
        let vad_thresh_text = format!("{:.0}%", self.state.vad_threshold * 100.0);

        let input_selected = self
            .state
            .selected_input_device
            .as_deref()
            .unwrap_or("Default Microphone (Realtek Audio)");
        let output_selected = self
            .state
            .selected_output_device
            .as_deref()
            .unwrap_or("Default Speakers (Realtek Audio)");

        // Noise suppression switch toggle
        let mut noise_supp_switch = render_switch_control(
            "Noise Suppression",
            "Attenuates background keyboard clicks, computer fan noise, and room echo.",
            self.state.noise_suppression,
        );

        // Echo cancellation switch toggle
        let mut echo_canc_switch = render_switch_control(
            "Echo Cancellation",
            "Eliminates audio feedback loops between your microphone and speakers.",
            self.state.echo_cancellation,
        );

        // Auto gain control switch toggle
        let auto_gain_switch = render_switch_control(
            "Automatic Gain Control",
            "Normalizes input volume so you are always heard at a consistent level.",
            self.state.auto_gain_control,
        );

        if let Some(ref mut cx) = cx {
            noise_supp_switch = noise_supp_switch.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.user_settings_modal.toggle_noise_suppression();
                    cx.notify();
                }),
            );
            echo_canc_switch = echo_canc_switch.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.user_settings_modal.toggle_echo_cancellation();
                    cx.notify();
                }),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            // Devices Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(render_select_dropdown(
                        "INPUT DEVICE",
                        input_selected,
                        "🎙",
                    ))
                    .child(render_select_dropdown(
                        "OUTPUT DEVICE",
                        output_selected,
                        "🔊",
                    )),
            )
            // Volume Sliders Section
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(render_slider_control(
                        "Input Volume",
                        &input_vol_text,
                        (self.state.input_volume / 2.0).clamp(0.0, 100.0),
                    ))
                    .child(render_slider_control(
                        "Output Volume",
                        &output_vol_text,
                        (self.state.output_volume / 2.0).clamp(0.0, 100.0),
                    )),
            )
            // Input Mode Segmented Control
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
                            .child("INPUT MODE"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(
                                div()
                                    .cursor_pointer()
                                    .flex_1()
                                    .p_3()
                                    .rounded_lg()
                                    .bg(if self.state.input_mode == InputMode::VoiceActivity {
                                        theme::bg_elevated()
                                    } else {
                                        theme::bg_subsurface()
                                    })
                                    .border_1()
                                    .border_color(
                                        if self.state.input_mode == InputMode::VoiceActivity {
                                            theme::accent()
                                        } else {
                                            theme::border_hairline()
                                        },
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("Voice Activity"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("Automatically transmits audio when speaking"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .flex_1()
                                    .p_3()
                                    .rounded_lg()
                                    .bg(if self.state.input_mode == InputMode::PushToTalk {
                                        theme::bg_elevated()
                                    } else {
                                        theme::bg_subsurface()
                                    })
                                    .border_1()
                                    .border_color(
                                        if self.state.input_mode == InputMode::PushToTalk {
                                            theme::accent()
                                        } else {
                                            theme::border_hairline()
                                        },
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("Push to Talk"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child(format!(
                                                        "Hold [{}] key to transmit voice",
                                                        self.state.ptt_keybind
                                                    )),
                                            ),
                                    ),
                            ),
                    ),
            )
            // VAD Threshold Sensitivity Slider
            .child(render_slider_control(
                "Input Sensitivity (VAD Threshold)",
                &vad_thresh_text,
                self.state.vad_threshold * 100.0,
            ))
            // Advanced Audio Processing Switches
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
                            .child("AUDIO PROCESSING"),
                    )
                    .child(noise_supp_switch)
                    .child(echo_canc_switch)
                    .child(auto_gain_switch),
            )
    }

    /// Renders the Appearance settings tab.
    fn render_appearance_tab(
        &self,
        cx: &mut Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> Div {
        let mut compact_switch = render_switch_control(
            "Compact Mode",
            "Reduces message margins and compacts channel padding for higher information density.",
            self.state.compact_mode,
        );

        if let Some(ref mut cx) = cx {
            compact_switch = compact_switch.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.user_settings_modal.toggle_compact_mode();
                    cx.notify();
                }),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_4()
            // Theme selection cards
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
                            .child("THEME"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_3()
                            .child(
                                div()
                                    .cursor_pointer()
                                    .flex_1()
                                    .p_3()
                                    .rounded_lg()
                                    .bg(theme::bg_elevated())
                                    .border_1()
                                    .border_color(theme::accent())
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("Pure Neutral Dark"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("#0A0B0E base with #111216 surface"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .flex_1()
                                    .p_3()
                                    .rounded_lg()
                                    .bg(theme::bg_subsurface())
                                    .border_1()
                                    .border_color(theme::border_hairline())
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("Sharkord Midnight"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("Deep navy blue accents"),
                                            ),
                                    ),
                            ),
                    ),
            )
            // Compact Mode Switch
            .child(compact_switch)
            // Font scale selector
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
                            .child("FONT SIZE"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(render_badge_button("12px (Small)", false))
                            .child(render_badge_button("14px (Default)", true))
                            .child(render_badge_button("16px (Large)", false)),
                    ),
            )
    }

    /// Renders the Keybinds settings tab.
    fn render_keybinds_tab(
        &self,
        _cx: &mut Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::text_secondary())
                    .child("GLOBAL & APPLICATION SHORTCUTS"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(render_keybind_row("Push to Talk", &self.state.ptt_keybind))
                    .child(render_keybind_row(
                        "Toggle Mute",
                        &self.state.toggle_mute_keybind,
                    ))
                    .child(render_keybind_row(
                        "Toggle Deafen",
                        &self.state.toggle_deafen_keybind,
                    ))
                    .child(render_keybind_row("Quick Switcher", "Ctrl+K"))
                    .child(render_keybind_row("User Settings", "Ctrl+,")),
            )
    }

    /// Renders the Notifications settings tab.
    fn render_notifications_tab(
        &self,
        cx: &mut Option<&mut Context<crate::ui::WorkspaceView>>,
    ) -> Div {
        let mut desktop_switch = render_switch_control(
            "Enable Desktop Notifications",
            "Show native OS popup banners when receiving mentions or direct messages.",
            self.state.desktop_notifications,
        );

        let mut sound_switch = render_switch_control(
            "Enable Notification Sounds",
            "Play subtle sound effects on incoming messages and voice room events.",
            self.state.sound_notifications,
        );

        let mut badge_switch = render_switch_control(
            "Unread Message Badge",
            "Display an unread counter badge on the dock/taskbar icon and system tray.",
            self.state.unread_badge,
        );

        if let Some(ref mut cx) = cx {
            desktop_switch = desktop_switch.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.user_settings_modal.toggle_desktop_notifications();
                    cx.notify();
                }),
            );
            sound_switch = sound_switch.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.user_settings_modal.toggle_sound_notifications();
                    cx.notify();
                }),
            );
            badge_switch = badge_switch.on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _, _window, cx| {
                    view.user_settings_modal.toggle_unread_badge();
                    cx.notify();
                }),
            );
        }

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(desktop_switch)
            .child(sound_switch)
            .child(badge_switch)
    }
}

impl Render for UserSettingsModalView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.render_element()
    }
}

// -----------------------------------------------------------------------------
// Component Helpers for Switch, Slider, Select, Keybind, and Input
// -----------------------------------------------------------------------------

/// Renders a switch toggle control with label and description.
fn render_switch_control(
    label: &'static str,
    description: &'static str,
    is_checked: bool,
) -> Div {
    div()
        .cursor_pointer()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .p_3()
        .rounded_lg()
        .bg(theme::bg_elevated())
        .border_1()
        .border_color(theme::border_hairline())
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::text_primary())
                        .child(label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child(description),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .w(px(40.0))
                .h(px(22.0))
                .p_0p5()
                .rounded_full()
                .bg(if is_checked {
                    theme::accent()
                } else {
                    theme::bg_surface_4()
                })
                .child(
                    div()
                        .w(px(18.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(rgb(0xFFFFFF))
                        .ml(if is_checked { px(18.0) } else { px(0.0) }),
                ),
        )
}

/// Renders a slider control with label, value badge, and horizontal track.
fn render_slider_control(
    label: &'static str,
    value_display: &str,
    percent: f32,
) -> Div {
    let clamped = percent.clamp(0.0, 100.0);
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .rounded_lg()
        .bg(theme::bg_elevated())
        .border_1()
        .border_color(theme::border_hairline())
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::text_primary())
                        .child(label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::text_secondary())
                        .child(value_display.to_string()),
                ),
        )
        .child(
            div()
                .relative()
                .flex()
                .items_center()
                .w_full()
                .h(px(16.0))
                .child(
                    div()
                        .w_full()
                        .h(px(6.0))
                        .rounded_full()
                        .bg(theme::bg_surface_4())
                        .child(
                            div()
                                .h_full()
                                .w(relative(clamped / 100.0))
                                .rounded_full()
                                .bg(theme::accent()),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .left(relative((clamped / 100.0).clamp(0.0, 0.95)))
                        .w(px(14.0))
                        .h(px(14.0))
                        .rounded_full()
                        .bg(rgb(0xFFFFFF))
                        .border_2()
                        .border_color(theme::accent()),
                ),
        )
}

/// Renders a simulated Select dropdown box.
fn render_select_dropdown(
    label: &'static str,
    selected_value: &str,
    icon: &'static str,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme::text_secondary())
                .child(label),
        )
        .child(
            div()
                .cursor_pointer()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(theme::bg_elevated())
                .border_1()
                .border_color(theme::border_hairline())
                .hover(|s| s.border_color(theme::accent()))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(div().child(icon))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme::text_primary())
                                .child(selected_value.to_string()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .child("▼"),
                ),
        )
}

/// Renders a labeled form text field.
fn render_setting_input(label: &'static str, value: &str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme::text_secondary())
                .child(label),
        )
        .child(
            div()
                .px_3()
                .py_1p5()
                .rounded_md()
                .bg(theme::bg_surface())
                .border_1()
                .border_color(theme::border_hairline())
                .text_sm()
                .text_color(theme::text_primary())
                .child(value.to_string()),
        )
}

/// Renders a single keybind action and shortcut pill.
fn render_keybind_row(action: &'static str, keybind: &str) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .p_3()
        .rounded_lg()
        .bg(theme::bg_elevated())
        .border_1()
        .border_color(theme::border_hairline())
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme::text_primary())
                .child(action),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(theme::bg_surface_4())
                        .border_1()
                        .border_color(theme::border_hairline())
                        .text_xs()
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(keybind.to_string()),
                )
                .child(
                    div()
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .text_xs()
                        .text_color(theme::text_muted())
                        .hover(|s| s.text_color(theme::text_primary()))
                        .child("Edit"),
                ),
        )
}

/// Renders a selectable badge button (e.g. font scale option).
fn render_badge_button(label: &'static str, is_active: bool) -> Div {
    div()
        .cursor_pointer()
        .px_3()
        .py_1p5()
        .rounded_md()
        .bg(if is_active {
            theme::accent()
        } else {
            theme::bg_elevated()
        })
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme::text_primary())
        .child(label)
}
