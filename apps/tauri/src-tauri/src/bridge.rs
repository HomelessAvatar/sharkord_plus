use crate::shortcuts::{self, HotkeyBinding, HotkeyError, ShortcutStateMap};
use crate::tray;
use serde::{Deserialize, Serialize};
use sharkord_core::vault::{DesktopSettings, SessionVault};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub struct BridgeState {
    pub server_list_visible: AtomicBool,
    pub close_to_tray: AtomicBool,
    pub start_to_tray: AtomicBool,
    pub dynamic_tray_icon: AtomicBool,
}

impl Default for BridgeState {
    fn default() -> Self {
        Self {
            server_list_visible: AtomicBool::new(true),
            close_to_tray: AtomicBool::new(true),
            start_to_tray: AtomicBool::new(false),
            dynamic_tray_icon: AtomicBool::new(true),
        }
    }
}

#[derive(Default)]
pub struct NotificationState {
    pub pending_targets: Mutex<HashMap<i32, serde_json::Value>>,
    pub latest_target: Mutex<Option<serde_json::Value>>,
}

impl NotificationState {
    pub fn store(&self, id: i32, target: serde_json::Value) {
        self.pending_targets
            .lock()
            .unwrap()
            .insert(id, target.clone());
        *self.latest_target.lock().unwrap() = Some(target);
    }

    pub fn take_by_id(&self, id: i32) -> Option<serde_json::Value> {
        self.pending_targets.lock().unwrap().remove(&id)
    }

    pub fn get_or_latest(&self, maybe_id: Option<i32>) -> Option<serde_json::Value> {
        if let Some(id) = maybe_id {
            if let Some(target) = self.take_by_id(id) {
                return Some(target);
            }
        }
        self.latest_target.lock().unwrap().clone()
    }
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
    Ok(next)
}

#[tauri::command]
pub fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    tray::show_main_window(&app);
    let _ = app.emit("desktop:open-settings", ());
    Ok(())
}

#[tauri::command]
pub fn set_unread_state(
    app: tauri::AppHandle,
    bridge: tauri::State<'_, BridgeState>,
    count: u32,
    mentioned: bool,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(desktop)]
        {
            if bridge.dynamic_tray_icon.load(Ordering::SeqCst) {
                if count > 0 || mentioned {
                    let attention = if mentioned {
                        tauri::UserAttentionType::Critical
                    } else {
                        tauri::UserAttentionType::Informational
                    };
                    let _ = window.request_user_attention(Some(attention));
                    let _ = window.set_badge_count(Some(count as i64));
                    let tooltip = if count > 0 {
                        format!("VibeShark - {} unread", count)
                    } else {
                        "VibeShark - Mentioned".to_string()
                    };
                    tray::update_tray_tooltip(&app, &tooltip);
                } else {
                    let _ = window.request_user_attention(None);
                    let _ = window.set_badge_count(None);
                    tray::update_tray_tooltip(&app, "VibeShark");
                }
            } else {
                let _ = window.request_user_attention(None);
                let _ = window.set_badge_count(None);
            }
        }
        #[cfg(not(desktop))]
        {
            let _ = (window, count, mentioned, bridge);
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
    let _ = crate::mobile_voice::set_mobile_voice_active(app.clone(), active);
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
    notif_state: tauri::State<'_, NotificationState>,
) -> Result<(), String> {
    static NEXT_ID: AtomicI32 = AtomicI32::new(1);
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

    if let Some(target_val) = target {
        notif_state.store(id, target_val);
    }

    let mut builder = app
        .notification()
        .builder()
        .id(id)
        .title(&title)
        .body(&body);

    if let Some(ref icon_str) = icon {
        builder = builder.icon(icon_str);
    }

    builder.show().map_err(|e| e.to_string())?;

    // Note: Do not emit click event here.
    // The click event is emitted exclusively in on_action when the user clicks the notification.
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DesktopDiagnostics {
    pub version: String,
    pub platform: String,
    pub arch: String,
    pub os_version: String,
    pub webview_version: String,
}

pub fn get_diagnostics_info() -> DesktopDiagnostics {
    DesktopDiagnostics {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        os_version: format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
        webview_version: "WebView2 / WebKit".to_string(),
    }
}

pub fn resolve_app_folder() -> std::path::PathBuf {
    let path = SessionVault::default_path();
    path.parent().unwrap_or(&path).to_path_buf()
}

pub fn resolve_logs_folder() -> std::path::PathBuf {
    resolve_app_folder().join("logs")
}

pub fn open_path_in_explorer<P: AsRef<std::path::Path>>(path: P) -> Result<(), String> {
    let path_ref = path.as_ref();
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path_ref)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path_ref)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path_ref)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_desktop_settings(
    vault: tauri::State<'_, crate::vault::VaultState>,
) -> Result<DesktopSettings, String> {
    let v = vault.0.lock().map_err(|e| e.to_string())?;
    Ok(v.desktop_settings.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDesktopSettingsInput {
    pub auto_start: Option<bool>,
    pub close_to_tray: Option<bool>,
    pub start_to_tray: Option<bool>,
    pub dynamic_tray_icon: Option<bool>,
    pub gpu_encoding: Option<bool>,
}

impl UpdateDesktopSettingsInput {
    pub fn apply_to(&self, target: &mut DesktopSettings) {
        if let Some(val) = self.auto_start {
            target.auto_start = val;
        }
        if let Some(val) = self.close_to_tray {
            target.close_to_tray = val;
        }
        if let Some(val) = self.start_to_tray {
            target.start_to_tray = val;
        }
        if let Some(val) = self.dynamic_tray_icon {
            target.dynamic_tray_icon = val;
        }
        if let Some(val) = self.gpu_encoding {
            target.gpu_encoding = val;
        }
    }
}

#[tauri::command]
pub fn update_desktop_settings(
    app: tauri::AppHandle,
    vault: tauri::State<'_, crate::vault::VaultState>,
    bridge: tauri::State<'_, BridgeState>,
    settings: UpdateDesktopSettingsInput,
) -> Result<(), String> {
    let updated = {
        let mut v = vault.0.lock().map_err(|e| e.to_string())?;
        settings.apply_to(&mut v.desktop_settings);
        v.save().map_err(|e| e.to_string())?;
        v.desktop_settings.clone()
    };

    bridge
        .close_to_tray
        .store(updated.close_to_tray, Ordering::SeqCst);
    bridge
        .start_to_tray
        .store(updated.start_to_tray, Ordering::SeqCst);
    bridge
        .dynamic_tray_icon
        .store(updated.dynamic_tray_icon, Ordering::SeqCst);

    #[cfg(desktop)]
    if let Some(enable_autostart) = settings.auto_start {
        use tauri_plugin_autostart::ManagerExt;
        let autostart_manager = app.autolaunch();
        if enable_autostart {
            let _ = autostart_manager.enable();
        } else {
            let _ = autostart_manager.disable();
        }
    }

    let _ = app.emit("desktop:settings-changed", updated);
    Ok(())
}

#[tauri::command]
pub fn get_app_folder() -> Result<String, String> {
    Ok(resolve_app_folder().to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_app_folder() -> Result<(), String> {
    let folder = resolve_app_folder();
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    open_path_in_explorer(&folder)
}

#[tauri::command]
pub fn get_logs_folder() -> Result<String, String> {
    Ok(resolve_logs_folder().to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_logs_folder() -> Result<(), String> {
    let folder = resolve_logs_folder();
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    open_path_in_explorer(&folder)
}

#[tauri::command]
pub fn get_desktop_diagnostics() -> Result<DesktopDiagnostics, String> {
    Ok(get_diagnostics_info())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_structure() {
        let diag = get_diagnostics_info();
        assert!(!diag.version.is_empty());
        assert!(!diag.platform.is_empty());
        assert!(!diag.arch.is_empty());
        assert!(!diag.os_version.is_empty());
        assert!(!diag.webview_version.is_empty());
    }

    #[test]
    fn test_folder_paths() {
        let app_folder = resolve_app_folder();
        let logs_folder = resolve_logs_folder();
        assert_eq!(logs_folder, app_folder.join("logs"));
        assert!(!app_folder.to_string_lossy().is_empty());
    }
}

