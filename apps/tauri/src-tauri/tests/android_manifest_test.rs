use sharkord_tauri_lib::mobile_voice;

#[test]
fn test_android_manifest_permissions() {
    let manifest = include_str!("../gen/android/app/src/main/AndroidManifest.xml");
    assert!(manifest.contains("android.permission.RECORD_AUDIO"));
    assert!(manifest.contains("android.permission.CAMERA"));
    assert!(manifest.contains("android.permission.MODIFY_AUDIO_SETTINGS"));
    assert!(manifest.contains("android.permission.FOREGROUND_SERVICE_MICROPHONE"));
    assert!(manifest.contains("android.permission.FOREGROUND_SERVICE"));
    assert!(manifest.contains("android.permission.POST_NOTIFICATIONS"));
    assert!(manifest.contains("android.permission.INTERNET"));
    assert!(manifest.contains("VoiceForegroundService"));
}

#[test]
fn test_main_activity_webrtc_permissions() {
    let main_activity =
        include_str!("../gen/android/app/src/main/java/com/sharkord/plus/MainActivity.kt");
    assert!(main_activity.contains("onPermissionRequest"));
    assert!(main_activity.contains("RESOURCE_AUDIO_CAPTURE"));
    assert!(main_activity.contains("RESOURCE_VIDEO_CAPTURE"));
    assert!(main_activity.contains("tauri://localhost"));
    assert!(main_activity.contains("http://localhost"));
    assert!(main_activity.contains("VoiceForegroundService"));
}

#[test]
fn test_voice_foreground_service_declaration() {
    let service_kt = include_str!(
        "../gen/android/app/src/main/java/com/sharkord/plus/VoiceForegroundService.kt"
    );
    assert!(service_kt.contains("class VoiceForegroundService : Service()"));
    assert!(service_kt.contains("startForeground"));
    assert!(service_kt.contains("sharkord_voice_channel"));
    assert!(service_kt.contains("Voice Lounge"));
}

#[test]
fn test_mobile_voice_state_management() {
    assert!(!mobile_voice::is_mobile_voice_active());
    assert!(mobile_voice::set_mobile_voice_state(true));
    assert!(mobile_voice::is_mobile_voice_active());
    assert!(!mobile_voice::set_mobile_voice_state(false));
    assert!(!mobile_voice::is_mobile_voice_active());
}

#[test]
fn test_tauri_conf_android_config() {
    let config_str = include_str!("../tauri.conf.json");
    let json: serde_json::Value = serde_json::from_str(config_str).unwrap();
    assert_eq!(json["bundle"]["android"]["minSdkVersion"], 24);
}

#[test]
fn test_android_neutral_dark_theme() {
    let colors = include_str!("../gen/android/app/src/main/res/values/colors.xml");
    assert!(colors.contains("#FF0A0A0A"));
    let themes = include_str!("../gen/android/app/src/main/res/values/themes.xml");
    assert!(themes.contains("pure_neutral_dark"));
}
