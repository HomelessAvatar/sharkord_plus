use sharkord_tauri_lib::bridge::{BridgeState, HotkeyCapabilities, UnreadState};
use sharkord_tauri_lib::shortcuts::{self, HotkeyBinding, HotkeyError, ShortcutStateMap};
use std::sync::atomic::Ordering;

#[test]
fn test_bridge_contract_types() {
    let js_bridge = include_str!("../src/bridge_init.js");
    assert!(js_bridge.contains("window.sharkordDesktop"));
    assert!(js_bridge.contains("DESKTOP_BRIDGE_VERSION"));
}

#[test]
fn test_bridge_methods_present_in_js() {
    let js_bridge = include_str!("../src/bridge_init.js");

    let required_methods = [
        "getServerListVisible",
        "onServerListVisibleChange",
        "toggleServerList",
        "openSettings",
        "setUnreadState",
        "setConnectionState",
        "setVoiceActive",
        "startSystemAudio",
        "stopSystemAudio",
        "showNotification",
        "setHotkeys",
        "getHotkeyCapabilities",
        "onHotkey",
        "onHotkeyErrors",
        "onNotificationClick",
    ];

    for method in required_methods {
        assert!(
            js_bridge.contains(method),
            "bridge_init.js must implement {} to satisfy TDesktopBridge",
            method
        );
    }
}

#[test]
fn test_shortcuts_normalize_combo() {
    assert_eq!(
        shortcuts::normalize_combo("ctrl+KeyM"),
        Some("Ctrl+M".to_string())
    );
    assert_eq!(
        shortcuts::normalize_combo("ctrl+KeyV"),
        Some("Ctrl+V".to_string())
    );
    assert_eq!(
        shortcuts::normalize_combo("ctrl+shift+KeyA"),
        Some("Ctrl+Shift+A".to_string())
    );
    assert_eq!(shortcuts::normalize_combo("F8"), Some("F8".to_string()));
    assert_eq!(shortcuts::normalize_combo("F13"), Some("F13".to_string()));

    // Mouse buttons are rejected as unsupported for global OS shortcuts
    assert_eq!(shortcuts::normalize_combo("ctrl+Mouse4"), None);
    assert_eq!(shortcuts::normalize_combo(""), None);
    assert_eq!(shortcuts::normalize_combo("   "), None);
}

#[test]
fn test_bridge_data_serialization() {
    let unread = UnreadState {
        count: 5,
        mentioned: true,
    };
    let json_val = serde_json::to_value(&unread).unwrap();
    assert_eq!(json_val["count"], 5);
    assert_eq!(json_val["mentioned"], true);

    let binding = HotkeyBinding {
        action_id: "core:toggle-mic".into(),
        combo: "ctrl+KeyM".into(),
    };
    let binding_json = serde_json::to_value(&binding).unwrap();
    assert_eq!(binding_json["actionId"], "core:toggle-mic");
    assert_eq!(binding_json["combo"], "ctrl+KeyM");

    let error = HotkeyError {
        action_id: "core:push-to-talk".into(),
        reason: "unsupported".into(),
    };
    let error_json = serde_json::to_value(&error).unwrap();
    assert_eq!(error_json["actionId"], "core:push-to-talk");
    assert_eq!(error_json["reason"], "unsupported");

    let caps = HotkeyCapabilities {
        key_up: true,
        system_managed: false,
    };
    let caps_json = serde_json::to_value(&caps).unwrap();
    assert_eq!(caps_json["keyUp"], true);
    assert_eq!(caps_json["systemManaged"], false);
}

#[test]
fn test_bridge_default_state() {
    let state = BridgeState::default();
    assert!(!state.server_list_visible.load(Ordering::SeqCst));
    state.server_list_visible.store(true, Ordering::SeqCst);
    assert!(state.server_list_visible.load(Ordering::SeqCst));

    let shortcut_state = ShortcutStateMap::default();
    let map = shortcut_state.registered.lock().unwrap();
    assert_eq!(map.len(), 0);
}
