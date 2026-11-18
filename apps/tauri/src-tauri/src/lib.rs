pub mod vault;

use sharkord_core::vault::SessionVault;
use std::sync::Mutex;
use vault::VaultState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let vault = SessionVault::load_from_path(SessionVault::default_path())
        .unwrap_or_else(|_| SessionVault::default());

    tauri::Builder::default()
        .manage(VaultState(Mutex::new(vault)))
        .invoke_handler(tauri::generate_handler![
            vault::get_saved_session,
            vault::save_session,
            vault::clear_session,
            vault::list_saved_servers,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
