use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyBinding {
    pub action_id: String,
    pub combo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyError {
    pub action_id: String,
    pub reason: String,
}

#[derive(Default)]
pub struct ShortcutStateMap {
    pub registered: Mutex<HashMap<Shortcut, String>>,
}

/// Normalizes web-style key combo strings (e.g. "ctrl+KeyM", "shift+alt+KeyP", "F8")
/// into Tauri Shortcut-compatible strings (e.g. "Ctrl+M", "Shift+Alt+P", "F8").
pub fn normalize_combo(combo: &str) -> Option<String> {
    let trimmed = combo.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Mouse buttons are not supported by the global keyboard shortcut engine
    if trimmed.to_lowercase().contains("mouse") {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    for raw_part in trimmed.split('+') {
        let part = raw_part.trim();
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => parts.push("Ctrl".to_string()),
            "shift" => parts.push("Shift".to_string()),
            "alt" | "option" => parts.push("Alt".to_string()),
            "cmd" | "command" | "super" | "meta" => parts.push("Super".to_string()),
            _ => {
                if let Some(rest) = part.strip_prefix("Key") {
                    parts.push(rest.to_uppercase());
                } else if let Some(rest) = part.strip_prefix("Digit") {
                    parts.push(rest.to_string());
                } else {
                    parts.push(part.to_string());
                }
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("+"))
    }
}

pub fn register_hotkeys<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &ShortcutStateMap,
    bindings: &[HotkeyBinding],
) -> Vec<HotkeyError> {
    let mut errors: Vec<HotkeyError> = Vec::new();
    let mut registered_map = state.registered.lock().unwrap();

    // Unregister any previously registered shortcuts
    for (shortcut, _) in registered_map.iter() {
        let _ = app.global_shortcut().unregister(*shortcut);
    }
    registered_map.clear();

    for binding in bindings {
        let normalized = match normalize_combo(&binding.combo) {
            Some(n) => n,
            None => {
                errors.push(HotkeyError {
                    action_id: binding.action_id.clone(),
                    reason: "unsupported".to_string(),
                });
                continue;
            }
        };

        let shortcut = match Shortcut::from_str(&normalized) {
            Ok(s) => s,
            Err(_) => {
                errors.push(HotkeyError {
                    action_id: binding.action_id.clone(),
                    reason: "unsupported".to_string(),
                });
                continue;
            }
        };

        match app.global_shortcut().register(shortcut) {
            Ok(()) => {
                registered_map.insert(shortcut, binding.action_id.clone());
            }
            Err(_) => {
                errors.push(HotkeyError {
                    action_id: binding.action_id.clone(),
                    reason: "taken".to_string(),
                });
            }
        }
    }

    errors
}

pub fn handle_shortcut_event<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &ShortcutStateMap,
    shortcut: &Shortcut,
    event: ShortcutEvent,
) {
    let registered_map = state.registered.lock().unwrap();
    if let Some(action_id) = registered_map.get(shortcut) {
        let phase = match event.state() {
            ShortcutState::Pressed => "down",
            ShortcutState::Released => "up",
        };

        // Standard single dispatch path through Tauri event bus
        let _ = app.emit(
            "desktop:hotkey",
            serde_json::json!({
                "actionId": action_id,
                "phase": phase,
            }),
        );
    }
}
