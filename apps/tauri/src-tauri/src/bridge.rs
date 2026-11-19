use crate::shortcuts::{self, HotkeyBinding, HotkeyError, ShortcutStateMap};
use crate::tray;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

#[derive(Default)]
pub struct BridgeState {
    pub server_list_visible: AtomicBool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyCapabilities {
    pub key_up: bool,
    pub system_managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnreadState {
    pub count: u32,
    pub mentioned: bool,
}

#[tauri::command]
pub fn get_server_list_visible(state: tauri::State<'_, BridgeState>) -> Result<bool, String> {
    Ok(state.server_list_visible.load(Ordering::SeqCst))
}

#[tauri::command]
pub fn toggle_server_list(
    app: tauri::AppHandle,
    state: tauri::State<'_, BridgeState>,
) -> Result<bool, String> {
    let next = !state.server_list_visible.load(Ordering::SeqCst);
    state.server_list_visible.store(next, Ordering::SeqCst);

    let _ = app.emit("desktop:server-list-visible", next);
    if let Some(window) = app.get_webview_window("main") {
        let js = format!(
            "if (window.__sharkordDesktopDispatch) window.__sharkordDesktopDispatch('server-list-visible', {});",
            next
        );
        let _ = window.eval(&js);
    }

    Ok(next)
}

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    tray::show_main_window(&app);
    let _ = app.emit("desktop:open-settings", ());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.eval(
            "if (window.__sharkordDesktopDispatch) window.__sharkordDesktopDispatch('open-settings');",
        );
    }
    Ok(())
}

#[tauri::command]
pub fn set_unread_state(
    app: tauri::AppHandle,
    count: u32,
    mentioned: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if count > 0 || mentioned {
            let attention = if mentioned {
                tauri::UserAttentionType::Critical
            } else {
                tauri::UserAttentionType::Informational
            };
            let _ = window.request_user_attention(Some(attention));
            let _ = window.set_badge_count(Some(count as i64));
        } else {
            let _ = window.request_user_attention(None);
            let _ = window.set_badge_count(None);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_connection_state(app: tauri::AppHandle, state: String) -> Result<(), String> {
    tray::set_tray_connection_state(&app, &state);
    Ok(())
}

#[tauri::command]
pub fn set_voice_active(app: tauri::AppHandle, active: bool) -> Result<(), String> {
    tray::set_tray_voice_active(&app, active);
    Ok(())
}

#[tauri::command]
pub fn start_system_audio(_options: Option<serde_json::Value>) -> Result<bool, String> {
    // Loopback virtual audio capture is not yet attached
    Ok(false)
}

#[tauri::command]
pub fn stop_system_audio() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn show_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
    icon: Option<String>,
    target: Option<serde_json::Value>,
) -> Result<(), String> {
    let mut builder = app.notification().builder().title(&title).body(&body);

    if let Some(ref icon_str) = icon {
        builder = builder.icon(icon_str);
    }

    builder.show().map_err(|e| e.to_string())?;

    // If target exists, register target for possible notification clicks
    if let Some(t) = target {
        let _ = app.emit("desktop:notification-target", &t);
    }

    Ok(())
}

#[tauri::command]
pub fn set_hotkeys(
    app: tauri::AppHandle,
    bindings: Vec<HotkeyBinding>,
    state: tauri::State<'_, ShortcutStateMap>,
) -> Result<Vec<HotkeyError>, String> {
    let errors = shortcuts::register_hotkeys(&app, &state, &bindings);
    Ok(errors)
}

#[tauri::command]
pub fn get_hotkey_capabilities() -> Result<HotkeyCapabilities, String> {
    Ok(HotkeyCapabilities {
        key_up: true,
        system_managed: false,
    })
}
