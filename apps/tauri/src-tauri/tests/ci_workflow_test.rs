#[test]
fn test_desktop_workflow_validity() {
    let workflow = include_str!("../../../../.github/workflows/desktop.yml");
    assert!(workflow.contains("tauri build"), "desktop.yml must contain 'tauri build'");
    assert!(workflow.contains("append_body: true"), "desktop.yml release step must append body");
    assert!(!workflow.contains("always() && !cancelled()"), "desktop.yml must not publish release on build failure");
    assert!(workflow.contains("windows-latest"));
    assert!(workflow.contains("ubuntu-24.04"));
    assert!(workflow.contains("macos-latest"));
}

#[test]
fn test_mobile_tauri_workflow_validity() {
    let workflow = include_str!("../../../../.github/workflows/mobile-tauri.yml");
    assert!(workflow.contains("tauri android build"), "mobile-tauri.yml must contain 'tauri android build'");
    assert!(workflow.contains("armv7-linux-androideabi"), "must use valid Rust target armv7-linux-androideabi");
    assert!(!workflow.contains("armv7-linux-android,"), "must not use invalid Rust target armv7-linux-android");
    assert!(workflow.contains("actions/setup-java@v4"), "must setup java 17");
    assert!(workflow.contains("android-actions/setup-android@v3"), "must setup android sdk");
    assert!(workflow.contains("append_body: true"), "mobile-tauri.yml release step must append body");
    assert!(!workflow.contains("always() && !cancelled()"), "mobile-tauri.yml must not publish release on build failure");
}
