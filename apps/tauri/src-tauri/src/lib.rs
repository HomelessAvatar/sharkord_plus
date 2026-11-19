pub mod bridge;
pub mod mobile_voice;
pub mod shortcuts;
pub mod tray;
pub mod vault;

use bridge::{BridgeState, NotificationState};
use sharkord_core::vault::SessionVault;
use shortcuts::ShortcutStateMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use vault::VaultState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let vault = SessionVault::load_from_path(SessionVault::default_path())
        .unwrap_or_else(|_| SessionVault::default());

    #[cfg(desktop)]
    let shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            let state = app.state::<ShortcutStateMap>();
            shortcuts::handle_shortcut_event(app, &state, shortcut, event);
        })
        .build();

    let bridge_plugin = tauri::plugin::Builder::<tauri::Wry>::new("desktop-bridge")
        .js_init_script(include_str!("bridge_init.js"))
        .build();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .manage(VaultState(Mutex::new(vault)))
        .manage(ShortcutStateMap::default())
        .manage(BridgeState::default())
        .manage(NotificationState::default())
        .plugin(tauri_plugin_notification::init())
        .plugin(bridge_plugin);

    #[cfg(desktop)]
    {
        builder = builder.plugin(shortcut_plugin);
    }

    builder
        .invoke_handler(tauri::generate_handler![
            vault::get_saved_session,
            vault::save_session,
            vault::clear_session,
            vault::list_saved_servers,
            bridge::get_server_list_visible,
            bridge::toggle_server_list,
            bridge::open_settings,
            bridge::set_unread_state,
            bridge::set_connection_state,
            bridge::set_voice_active,
            bridge::start_system_audio,
            bridge::stop_system_audio,
            bridge::show_notification,
            bridge::set_hotkeys,
            bridge::get_hotkey_capabilities,
            mobile_voice::set_mobile_voice_active,
        ])
        .setup(|app| {
            let notif_handle = app.handle().clone();
            let _ = app.notification().on_action(move |performed| {
                if performed.action_id() == "tap" {
                    tray::show_main_window(&notif_handle);
                    let notif_state = notif_handle.state::<NotificationState>();
                    let maybe_id = performed.notification().map(|n| n.id());
                    if let Some(target) = notif_state.get_or_latest(maybe_id) {
                        let _ = notif_handle.emit("desktop:notification-click", target);
                    }
                }
            });

            #[cfg(desktop)]
            {
                tray::setup_tray(app.handle())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
