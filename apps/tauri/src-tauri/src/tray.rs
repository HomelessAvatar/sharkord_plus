#[cfg(desktop)]
use tauri::menu::{Menu, MenuItem};
#[cfg(desktop)]
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
#[cfg(desktop)]
use tauri::Emitter;
use tauri::Manager;

pub const TRAY_ID: &str = "main";

/// Generates a press-and-release pair of hotkey phases for discrete tray actions.
pub fn create_toggle_events(action_id: &str) -> [(String, &'static str); 2] {
    [
        (action_id.to_string(), "down"),
        (action_id.to_string(), "up"),
    ]
}

#[cfg(desktop)]
pub fn setup_tray<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Sharkord", true, None::<&str>)?;
    let mute_item = MenuItem::with_id(app, "mute", "Mute Microphone", true, None::<&str>)?;
    let deafen_item = MenuItem::with_id(app, "deafen", "Deafen", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &mute_item, &deafen_item, &quit_item])?;

    let mut builder = TrayIconBuilder::<R>::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("VibeShark")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    show_main_window(app);
                }
                "mute" => {
                    // Discrete click must emit down followed by up to complete the cycle
                    for (action_id, phase) in create_toggle_events("core:toggle-mic") {
                        let _ = app.emit(
                            "desktop:hotkey",
                            serde_json::json!({
                                "actionId": action_id,
                                "phase": phase
                            }),
                        );
                    }
                }
                "deafen" => {
                    // Discrete click must emit down followed by up to complete the cycle
                    for (action_id, phase) in create_toggle_events("core:toggle-sound") {
                        let _ = app.emit(
                            "desktop:hotkey",
                            serde_json::json!({
                                "actionId": action_id,
                                "phase": phase
                            }),
                        );
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

#[cfg(not(desktop))]
pub fn setup_tray<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn show_main_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        #[cfg(desktop)]
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn update_tray_tooltip<R: tauri::Runtime>(app: &tauri::AppHandle<R>, tooltip: &str) {
    #[cfg(desktop)]
    {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_tooltip(Some(tooltip));
        }
    }
    #[cfg(not(desktop))]
    {
        let _ = (app, tooltip);
    }
}

pub fn set_tray_connection_state<R: tauri::Runtime>(app: &tauri::AppHandle<R>, state: &str) {
    let tooltip = format!("VibeShark - {}", state);
    update_tray_tooltip(app, &tooltip);
}

pub fn set_tray_voice_active<R: tauri::Runtime>(app: &tauri::AppHandle<R>, active: bool) {
    let tooltip = if active {
        "VibeShark - Voice Active"
    } else {
        "VibeShark"
    };
    update_tray_tooltip(app, tooltip);
}
