pub mod bridge;
pub mod mobile_voice;
pub mod shortcuts;
pub mod tray;
pub mod vault;

use bridge::{BridgeState, NotificationState};
use sharkord_core::vault::SessionVault;
use shortcuts::ShortcutStateMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::Emitter;
#[allow(unused_imports)]
use tauri::Manager;
#[cfg(desktop)]
use tauri_plugin_notification::NotificationExt;
use vault::VaultState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let vault = SessionVault::load_from_path(SessionVault::default_path())
        .unwrap_or_else(|_| SessionVault::default());

    #[cfg(target_os = "windows")]
    if vault.desktop_settings.gpu_encoding {
        let existing = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default();
        let hw_args = "--enable-features=WebRtcAllowHwAv1Encoding,ZeroCopyVideoStream,RawDraw --ignore-gpu-blocklist --enable-gpu-rasterization --enable-zero-copy";
        let combined = if existing.is_empty() {
            hw_args.to_string()
        } else {
            format!("{} {}", existing, hw_args)
        };
        std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", combined);
    }

    let bridge_state = BridgeState {
        server_list_visible: AtomicBool::new(true),
        close_to_tray: AtomicBool::new(vault.desktop_settings.close_to_tray),
        start_to_tray: AtomicBool::new(vault.desktop_settings.start_to_tray),
        dynamic_tray_icon: AtomicBool::new(vault.desktop_settings.dynamic_tray_icon),
    };

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
        .manage(bridge_state)
        .manage(NotificationState::default())
        .plugin(tauri_plugin_notification::init())
        .plugin(bridge_plugin)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(desktop)]
                {
                    let bridge_state = window.state::<BridgeState>();
                    if bridge_state.close_to_tray.load(std::sync::atomic::Ordering::SeqCst) {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        });

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(shortcut_plugin)
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--autostart"]),
            ));
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
            bridge::get_desktop_settings,
            bridge::update_desktop_settings,
            bridge::get_app_folder,
            bridge::open_app_folder,
            bridge::get_logs_folder,
            bridge::open_logs_folder,
            bridge::get_desktop_diagnostics,
            mobile_voice::set_mobile_voice_active,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
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

                tray::setup_tray(app.handle())?;

                let args: Vec<String> = std::env::args().collect();
                let is_autostart = args.iter().any(|arg| arg == "--hidden" || arg == "--autostart");
                let bridge_state = app.state::<BridgeState>();
                if is_autostart && bridge_state.start_to_tray.load(std::sync::atomic::Ordering::SeqCst) {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
