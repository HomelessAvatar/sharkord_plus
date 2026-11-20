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
    assert!(manifest.contains("VibeSharkApp"));
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

#[test]
fn test_android_release_signing_config() {
    let build_gradle = include_str!("../gen/android/app/build.gradle.kts");
    assert!(build_gradle.contains("signingConfigs"));
    assert!(build_gradle.contains("signingConfig = signingConfigs.getByName(\"release\")"));
    assert!(build_gradle.contains("defaultKeystore"));
    assert!(build_gradle.contains("compileSdk = 37"));
    assert!(build_gradle.contains("targetSdk = 34"));
    assert!(build_gradle.contains("isMinifyEnabled = false"));
    assert!(build_gradle.contains("useLegacyPackaging = false"));
    assert!(build_gradle.contains("jniLibs.srcDirs(\"src/main/jniLibs\")"));

    let keystore = include_bytes!("../gen/android/release.keystore");
    assert!(!keystore.is_empty());
}

#[test]
fn test_android_proguard_rules() {
    let rules = include_str!("../gen/android/app/proguard-rules.pro");
    assert!(rules.contains("-keep class app.tauri.** { *; }"));
    assert!(rules.contains("-keep class com.sharkord.plus.** { *; }"));
    assert!(rules.contains("-keep class androidx.webkit.** { *; }"));
}

#[test]
fn test_capabilities_default_permissions() {
    let cap_str = include_str!("../capabilities/default.json");
    let json: serde_json::Value = serde_json::from_str(cap_str).unwrap();
    let perms = json["permissions"].as_array().unwrap();
    assert!(perms.iter().any(|p| p == "core:default"));
    assert!(perms.iter().any(|p| p == "notification:default"));
}

#[test]
fn test_16kb_page_size_alignment_configuration() {
    let build_rs = include_str!("../build.rs");
    assert!(build_rs.contains("max-page-size=16384"));
    assert!(build_rs.contains("common-page-size=16384"));
    assert!(build_rs.contains("c++_shared"));

    let cargo_config = include_str!("../.cargo/config.toml");
    assert!(cargo_config.contains("max-page-size=16384"));
    assert!(cargo_config.contains("aarch64-linux-android"));
    assert!(cargo_config.contains("lc++_shared"));
}

#[test]
fn test_vibeshark_app_crash_logging() {
    let app_kt = include_str!(
        "../gen/android/app/src/main/java/com/sharkord/plus/VibeSharkApp.kt"
    );
    assert!(app_kt.contains("class VibeSharkApp : Application()"));
    assert!(app_kt.contains("Thread.setDefaultUncaughtExceptionHandler"));
    assert!(app_kt.contains("vibeshark_crash.log"));
    assert!(app_kt.contains("getExternalFilesDir"));
    assert!(app_kt.contains("VIBESHARK FATAL CRASH REPORT"));
    assert!(app_kt.contains("c++_shared"));
}

#[test]
fn test_android_app_name_strings() {
    let strings = include_str!("../gen/android/app/src/main/res/values/strings.xml");
    assert!(strings.contains("\"VibeShark\""));
}

#[test]
fn test_android_window_insets_safe_area() {
    let main_activity =
        include_str!("../gen/android/app/src/main/java/com/sharkord/plus/MainActivity.kt");
    assert!(main_activity.contains("setOnApplyWindowInsetsListener"));
    assert!(main_activity.contains("WindowInsetsCompat"));
    assert!(main_activity.contains("systemBars"));
    assert!(main_activity.contains("displayCutout"));
    assert!(main_activity.contains("requestApplyInsets"));

    let manifest = include_str!("../gen/android/app/src/main/AndroidManifest.xml");
    assert!(manifest.contains("android:windowSoftInputMode=\"adjustResize\""));
}
