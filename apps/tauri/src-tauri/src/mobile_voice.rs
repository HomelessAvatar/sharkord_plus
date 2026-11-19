use std::sync::atomic::{AtomicBool, Ordering};
use tauri_plugin_notification::NotificationExt;

pub const MOBILE_VOICE_NOTIFICATION_ID: i32 = 4991;

static MOBILE_VOICE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn is_mobile_voice_active() -> bool {
    MOBILE_VOICE_ACTIVE.load(Ordering::SeqCst)
}

pub fn set_mobile_voice_state(active: bool) -> bool {
    MOBILE_VOICE_ACTIVE.store(active, Ordering::SeqCst);
    active
}

#[tauri::command]
pub fn set_mobile_voice_active(app: tauri::AppHandle, active: bool) -> Result<bool, String> {
    set_mobile_voice_state(active);

    if active {
        let _ = app
            .notification()
            .builder()
            .id(MOBILE_VOICE_NOTIFICATION_ID)
            .title("Sharkord Voice")
            .body("Connected to voice lounge")
            .show();
    }

    #[cfg(target_os = "android")]
    {
        trigger_android_service(&app, active);
    }

    Ok(active)
}

#[cfg(target_os = "android")]
fn trigger_android_service(_app: &tauri::AppHandle, _active: bool) {
    // Android foreground service triggered through JNI / MainActivity on Android runtime
}
