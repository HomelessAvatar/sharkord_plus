use sharkord_core::vault::{ServerProfile, SessionVault};
use std::sync::Mutex;
use tauri::State;

pub struct VaultState(pub Mutex<SessionVault>);

#[tauri::command]
pub fn get_saved_session(state: State<'_, VaultState>) -> Result<Option<ServerProfile>, String> {
    let vault = state.0.lock().map_err(|e| e.to_string())?;
    Ok(vault.get_active_profile().cloned())
}

#[tauri::command]
pub fn save_session(profile: ServerProfile, state: State<'_, VaultState>) -> Result<(), String> {
    let mut vault = state.0.lock().map_err(|e| e.to_string())?;
    vault.save_profile(profile).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn clear_session(server_id: String, state: State<'_, VaultState>) -> Result<(), String> {
    let mut vault = state.0.lock().map_err(|e| e.to_string())?;
    vault
        .remove_profile(&server_id)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_saved_servers(state: State<'_, VaultState>) -> Result<Vec<ServerProfile>, String> {
    let vault = state.0.lock().map_err(|e| e.to_string())?;
    Ok(vault.list_profiles())
}
