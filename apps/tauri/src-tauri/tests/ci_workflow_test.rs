#[test]
fn test_desktop_workflow_validity() {
    let workflow = include_str!("../../../../.github/workflows/desktop.yml");
    assert!(workflow.contains("tauri build"), "desktop.yml must contain 'tauri build'");
    assert!(workflow.contains("append_body: true"), "desktop.yml release step must append body");
    assert!(workflow.contains("generate_release_notes.py"), "desktop.yml must run release notes generator");
    assert!(workflow.contains("body_path:"), "desktop.yml must pass body_path to action-gh-release");
    assert!(!workflow.contains("always() && !cancelled()"), "desktop.yml must not publish release on build failure");
    assert!(workflow.contains("windows-latest"));
    assert!(workflow.contains("ubuntu-24.04"));
    assert!(workflow.contains("macos-latest") || workflow.contains("macos-15-intel"));
}

#[test]
fn test_mobile_tauri_workflow_validity() {
    let workflow = include_str!("../../../../.github/workflows/mobile-tauri.yml");
    assert!(workflow.contains("tauri android build"), "mobile-tauri.yml must contain 'tauri android build'");
    assert!(workflow.contains("armv7-linux-androideabi"), "must use valid Rust target armv7-linux-androideabi");
    assert!(!workflow.contains("armv7-linux-android,"), "must not use invalid Rust target armv7-linux-android");
    assert!(workflow.contains("actions/setup-java@v"), "must setup java");
    assert!(workflow.contains("android-actions/setup-android@v3"), "must setup android sdk");
    assert!(workflow.contains("append_body: true"), "mobile-tauri.yml release step must append body");
    assert!(workflow.contains("generate_release_notes.py"), "mobile-tauri.yml must run release notes generator");
    assert!(workflow.contains("body_path:"), "mobile-tauri.yml must pass body_path to action-gh-release");
    assert!(!workflow.contains("always() && !cancelled()"), "mobile-tauri.yml must not publish release on build failure");
    assert!(workflow.contains("ANDROID_KEYSTORE_PATH"), "mobile-tauri.yml must configure keystore");
    assert!(workflow.contains("APKSIGNER"), "mobile-tauri.yml must locate and use apksigner");
    assert!(workflow.contains("sign"), "mobile-tauri.yml must sign APK");
    assert!(workflow.contains("libc++_shared.so"), "mobile-tauri.yml must bundle libc++_shared.so");
    assert!(workflow.contains("ndk;27."), "mobile-tauri.yml must use NDK r27 for 16kb page-size support");
}
