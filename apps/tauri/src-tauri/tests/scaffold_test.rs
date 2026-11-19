#[test]
fn test_tauri_config_exists_and_valid() {
    let config_str = include_str!("../tauri.conf.json");
    let json: serde_json::Value =
        serde_json::from_str(config_str).expect("tauri.conf.json must be valid JSON");
    assert_eq!(json["productName"], "Sharkord Plus");
    assert_eq!(json["app"]["windows"][0]["title"], "Sharkord Plus");
    assert_eq!(json["build"]["beforeDevCommand"], "bun --filter client dev");
    assert_eq!(json["build"]["beforeBuildCommand"], "bun --filter client build");
}
