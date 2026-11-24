use desktop_gpui::state::AppState;
use desktop_gpui::tray::{DesktopTray, TrayAction, TrayMenuState};
use desktop_gpui::ui::user_settings_modal::{
    InputMode, SettingsTab, UserSettingsModalView, UserSettingsState,
};
use desktop_gpui::ui::WorkspaceView;

#[test]
fn test_user_settings_tab_navigation() {
    let mut state = UserSettingsState::new();
    assert!(!state.is_open);
    assert_eq!(state.active_tab, SettingsTab::Account);
    assert_eq!(state.active_tab.label(), "Account");
    assert_eq!(SettingsTab::VoiceAndVideo.label(), "Voice & Video");
    assert_eq!(SettingsTab::Appearance.label(), "Appearance");
    assert_eq!(SettingsTab::Keybinds.label(), "Keybinds");
    assert_eq!(SettingsTab::Notifications.label(), "Notifications");

    state.open();
    assert!(state.is_open);

    state.set_tab(SettingsTab::VoiceAndVideo);
    assert_eq!(state.active_tab, SettingsTab::VoiceAndVideo);

    state.set_tab(SettingsTab::Appearance);
    assert_eq!(state.active_tab, SettingsTab::Appearance);

    state.set_tab(SettingsTab::Keybinds);
    assert_eq!(state.active_tab, SettingsTab::Keybinds);

    state.set_tab(SettingsTab::Notifications);
    assert_eq!(state.active_tab, SettingsTab::Notifications);

    state.set_tab(SettingsTab::Desktop);
    assert_eq!(state.active_tab, SettingsTab::Desktop);
    assert_eq!(SettingsTab::Desktop.label(), "Desktop");
    assert_eq!(SettingsTab::Desktop.icon(), "🖥");
    assert!(SettingsTab::Desktop.description().contains("GPU hardware acceleration"));

    state.close();
    assert!(!state.is_open);
}

#[test]
fn test_user_settings_audio_and_input_mode() {
    let mut state = UserSettingsState::new();
    assert_eq!(state.input_volume, 100.0);
    assert_eq!(state.output_volume, 100.0);
    assert_eq!(state.input_mode, InputMode::VoiceActivity);
    assert_eq!(state.vad_threshold, 0.5);
    assert!(state.noise_suppression);
    assert!(state.echo_cancellation);

    state.set_input_volume(85.0);
    assert_eq!(state.input_volume, 85.0);

    state.set_output_volume(60.0);
    assert_eq!(state.output_volume, 60.0);

    state.set_input_mode(InputMode::PushToTalk);
    assert_eq!(state.input_mode, InputMode::PushToTalk);

    state.set_vad_threshold(0.35);
    assert_eq!(state.vad_threshold, 0.35);

    state.toggle_noise_suppression();
    assert!(!state.noise_suppression);
    state.set_noise_suppression(true);
    assert!(state.noise_suppression);

    state.toggle_echo_cancellation();
    assert!(!state.echo_cancellation);
}

#[test]
fn test_desktop_settings_toggles_and_diagnostics() {
    let mut state = UserSettingsState::new();
    assert!(!state.desktop_settings.auto_start);
    assert!(state.desktop_settings.close_to_tray);
    assert!(!state.desktop_settings.start_to_tray);
    assert!(state.desktop_settings.dynamic_tray_icon);
    assert!(state.desktop_settings.gpu_encoding);

    state.toggle_auto_start();
    assert!(state.desktop_settings.auto_start);

    state.toggle_close_to_tray();
    assert!(!state.desktop_settings.close_to_tray);

    state.toggle_start_to_tray();
    assert!(state.desktop_settings.start_to_tray);

    state.toggle_dynamic_tray_icon();
    assert!(!state.desktop_settings.dynamic_tray_icon);

    state.toggle_gpu_encoding();
    assert!(!state.desktop_settings.gpu_encoding);

    assert!(!state.app_folder.is_empty());
    assert!(!state.logs_folder.is_empty());
    assert!(state.logs_folder.ends_with("logs"));

    assert!(!state.diagnostics.version.is_empty());
    assert!(!state.diagnostics.platform.is_empty());
    assert!(!state.diagnostics.arch.is_empty());
    assert!(!state.diagnostics.webview_version.is_empty());
}

#[test]
fn test_tray_menu_state_toggles() {
    let mut tray = TrayMenuState::new();
    assert!(!tray.is_muted);
    assert!(!tray.is_deafened);
    assert!(!tray.is_connected);
    assert_eq!(tray.unread_count, 0);

    tray.toggle_mute();
    assert!(tray.is_muted);
    tray.toggle_mute();
    assert!(!tray.is_muted);

    tray.toggle_deafen();
    assert!(tray.is_deafened);
    tray.toggle_deafen();
    assert!(!tray.is_deafened);

    tray.set_connected(true);
    assert!(tray.is_connected);

    tray.set_unread_count(5);
    assert_eq!(tray.unread_count, 5);

    let mut desktop_tray = DesktopTray::new();
    assert!(!desktop_tray.state.is_muted);
    desktop_tray.dispatch_action(TrayAction::ToggleMute);
    assert!(desktop_tray.state.is_muted);
    desktop_tray.dispatch_action(TrayAction::ToggleDeafen);
    assert!(desktop_tray.state.is_deafened);
    desktop_tray.dispatch_action(TrayAction::FocusWindow);
    assert!(desktop_tray.is_window_focused);
    desktop_tray.dispatch_action(TrayAction::OpenDesktopSettings);
    assert!(desktop_tray.is_settings_open);

    // Test dynamic tray tooltip
    desktop_tray.state.set_unread_count(3);
    assert_eq!(desktop_tray.tooltip(), "VibeShark (3 unread)");
    desktop_tray.desktop_settings.dynamic_tray_icon = false;
    assert_eq!(desktop_tray.tooltip(), "VibeShark");

    desktop_tray.dispatch_action(TrayAction::Quit);
    assert!(desktop_tray.has_quit);
}

#[test]
fn test_user_settings_modal_view_rendering() {
    let mut state = UserSettingsState::new();
    state.open();

    for tab in [
        SettingsTab::Account,
        SettingsTab::VoiceAndVideo,
        SettingsTab::Appearance,
        SettingsTab::Keybinds,
        SettingsTab::Notifications,
        SettingsTab::Desktop,
    ] {
        state.set_tab(tab);
        let view = UserSettingsModalView::new(state.clone());
        let _el = view.render_element();
    }
}

#[test]
fn test_workspace_view_user_settings_integration() {
    let app_state = AppState::new_mock();
    let mut workspace = WorkspaceView::new(app_state);
    assert!(!workspace.user_settings_modal.is_open);

    workspace.open_user_settings();
    assert!(workspace.user_settings_modal.is_open);
    assert_eq!(workspace.user_settings_modal.active_tab, SettingsTab::Account);

    workspace.open_desktop_settings_mock();
    assert_eq!(workspace.user_settings_modal.active_tab, SettingsTab::Desktop);

    workspace.close_user_settings();
    assert!(!workspace.user_settings_modal.is_open);
}
