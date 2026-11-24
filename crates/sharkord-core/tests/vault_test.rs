use sharkord_core::vault::{SavedServerSession, SessionVault};

#[test]
fn test_session_vault_add_save_and_retrieve_active() {
    let temp_dir = std::env::temp_dir().join(format!("sharkord_test_vault_{}", std::process::id()));
    let vault_path = temp_dir.join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut vault = SessionVault::new(vault_path.clone());
    assert!(vault.get_active().is_none());

    let server = SavedServerSession {
        server_id: "srv_123".to_string(),
        server_name: "Test Community".to_string(),
        server_url: "https://test.sharkord.com".to_string(),
        token: "jwt_token_sample".to_string(),
        user_id: 42,
        username: "testuser".to_string(),
        display_name: "Test User".to_string(),
        avatar_url: None,
        last_accessed_at: 1700000000,
    };

    vault
        .add_or_update(server.clone(), true)
        .expect("should save");
    assert_eq!(vault.get_active().unwrap().server_id, "srv_123");

    // Reopen vault from disk
    let loaded_vault = SessionVault::load_from_path(vault_path).expect("should load");
    assert_eq!(loaded_vault.get_active().unwrap().token, "jwt_token_sample");
    assert_eq!(loaded_vault.servers.len(), 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_vault_update_existing_server() {
    let temp_dir =
        std::env::temp_dir().join(format!("sharkord_test_vault_update_{}", std::process::id()));
    let vault_path = temp_dir.join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut vault = SessionVault::new(vault_path.clone());

    let server1 = SavedServerSession {
        server_id: "srv_1".to_string(),
        server_name: "Initial Name".to_string(),
        server_url: "https://srv1.sharkord.com".to_string(),
        token: "old_token".to_string(),
        user_id: 10,
        username: "user1".to_string(),
        display_name: "User One".to_string(),
        avatar_url: None,
        last_accessed_at: 1000,
    };

    vault
        .add_or_update(server1, true)
        .expect("should save server1");
    assert_eq!(vault.servers.len(), 1);
    assert_eq!(vault.get_active().unwrap().server_name, "Initial Name");

    // Update server1 with new token and updated name
    let server1_updated = SavedServerSession {
        server_id: "srv_1".to_string(),
        server_name: "Updated Name".to_string(),
        server_url: "https://srv1.sharkord.com".to_string(),
        token: "new_token".to_string(),
        user_id: 10,
        username: "user1".to_string(),
        display_name: "User One".to_string(),
        avatar_url: Some("https://srv1.sharkord.com/avatar.png".to_string()),
        last_accessed_at: 2000,
    };

    vault
        .add_or_update(server1_updated, true)
        .expect("should update server1");
    assert_eq!(
        vault.servers.len(),
        1,
        "should not duplicate server in list"
    );
    assert_eq!(vault.get_active().unwrap().server_name, "Updated Name");
    assert_eq!(vault.get_active().unwrap().token, "new_token");
    assert_eq!(
        vault.get_active().unwrap().avatar_url.as_deref(),
        Some("https://srv1.sharkord.com/avatar.png")
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_vault_multiple_servers_set_active_and_remove() {
    let temp_dir =
        std::env::temp_dir().join(format!("sharkord_test_vault_multi_{}", std::process::id()));
    let vault_path = temp_dir.join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut vault = SessionVault::new(vault_path.clone());

    let server_a = SavedServerSession {
        server_id: "srv_a".to_string(),
        server_name: "Server A".to_string(),
        server_url: "https://a.sharkord.com".to_string(),
        token: "token_a".to_string(),
        user_id: 1,
        username: "alice".to_string(),
        display_name: "Alice".to_string(),
        avatar_url: None,
        last_accessed_at: 100,
    };

    let server_b = SavedServerSession {
        server_id: "srv_b".to_string(),
        server_name: "Server B".to_string(),
        server_url: "https://b.sharkord.com".to_string(),
        token: "token_b".to_string(),
        user_id: 2,
        username: "bob".to_string(),
        display_name: "Bob".to_string(),
        avatar_url: None,
        last_accessed_at: 200,
    };

    vault.add_or_update(server_a, true).expect("save a");
    vault
        .add_or_update(server_b, false)
        .expect("save b without making active");

    assert_eq!(vault.servers.len(), 2);
    assert_eq!(vault.get_active().unwrap().server_id, "srv_a");

    // Switch active to server B
    vault.set_active("srv_b").expect("switch to b");
    assert_eq!(vault.get_active().unwrap().server_id, "srv_b");

    // Verify persistence of active state
    let reloaded = SessionVault::load_from_path(&vault_path).expect("reload");
    assert_eq!(reloaded.get_active().unwrap().server_id, "srv_b");

    // Remove active server (srv_b)
    let removed = vault.remove("srv_b").expect("remove b");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().server_id, "srv_b");
    assert_eq!(vault.servers.len(), 1);

    // Active should fall back to remaining server (srv_a)
    assert_eq!(vault.get_active().unwrap().server_id, "srv_a");

    // Remove remaining server
    let removed_last = vault.remove("srv_a").expect("remove a");
    assert!(removed_last.is_some());
    assert_eq!(vault.servers.len(), 0);
    assert!(vault.get_active().is_none());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_vault_nested_parent_dirs_created_atomically() {
    let temp_dir =
        std::env::temp_dir().join(format!("sharkord_test_vault_nested_{}", std::process::id()));
    let nested_vault_path = temp_dir.join("level1").join("level2").join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut vault = SessionVault::new(nested_vault_path.clone());
    let server = SavedServerSession {
        server_id: "srv_nested".to_string(),
        server_name: "Nested Server".to_string(),
        server_url: "https://nested.sharkord.com".to_string(),
        token: "token_nested".to_string(),
        user_id: 99,
        username: "nested_user".to_string(),
        display_name: "Nested User".to_string(),
        avatar_url: None,
        last_accessed_at: 500,
    };

    vault
        .add_or_update(server, true)
        .expect("should create nested parent directories");
    assert!(nested_vault_path.exists());

    let loaded =
        SessionVault::load_from_path(nested_vault_path).expect("should load from nested path");
    assert_eq!(loaded.get_active().unwrap().server_id, "srv_nested");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_vault_set_active_not_found_and_clear() {
    let temp_dir =
        std::env::temp_dir().join(format!("sharkord_test_vault_errors_{}", std::process::id()));
    let vault_path = temp_dir.join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut vault = SessionVault::new(vault_path);
    let result = vault.set_active("non_existent_srv");
    assert!(
        result.is_err(),
        "setting non-existent active server must fail"
    );

    let server = SavedServerSession {
        server_id: "srv_valid".to_string(),
        server_name: "Valid Server".to_string(),
        server_url: "https://valid.sharkord.com".to_string(),
        token: "valid_token".to_string(),
        user_id: 1,
        username: "valid".to_string(),
        display_name: "Valid User".to_string(),
        avatar_url: None,
        last_accessed_at: 10,
    };
    vault.add_or_update(server, true).unwrap();
    assert!(vault.get_active().is_some());

    vault.clear_active().unwrap();
    assert!(vault.get_active().is_none());

    vault.clear().unwrap();
    assert_eq!(vault.servers.len(), 0);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_vault_load_non_existent_and_corrupt() {
    let temp_dir = std::env::temp_dir().join(format!(
        "sharkord_test_vault_corrupt_{}",
        std::process::id()
    ));
    let non_existent_path = temp_dir.join("missing.json");

    assert!(SessionVault::load_from_path(&non_existent_path).is_err());

    let corrupt_path = temp_dir.join("corrupt.json");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(&corrupt_path, b"not valid json").unwrap();

    assert!(SessionVault::load_from_path(&corrupt_path).is_err());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_vault_default_path_is_not_empty() {
    let path = SessionVault::default_path();
    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().ends_with("vault.json"));
}

#[test]
fn test_desktop_settings_default_and_backward_compatibility() {
    let temp_dir = std::env::temp_dir().join(format!(
        "sharkord_test_vault_desktop_settings_{}",
        std::process::id()
    ));
    let file_path = temp_dir.join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    // Old vault JSON without desktopSettings
    let old_json = r#"{"servers":[],"profiles":[]}"#;
    std::fs::write(&file_path, old_json).unwrap();

    let mut vault = SessionVault::load_from_path(&file_path).unwrap();
    assert_eq!(vault.desktop_settings.close_to_tray, true);
    assert_eq!(vault.desktop_settings.gpu_encoding, true);
    assert_eq!(vault.desktop_settings.dynamic_tray_icon, true);
    assert_eq!(vault.desktop_settings.auto_start, false);
    assert_eq!(vault.desktop_settings.start_to_tray, false);
    assert_eq!(vault.get_desktop_settings().close_to_tray, true);

    // Update desktop settings
    vault.desktop_settings.auto_start = true;
    vault.desktop_settings.close_to_tray = false;
    vault.save().unwrap();

    let reloaded = SessionVault::load_from_path(&file_path).unwrap();
    assert_eq!(reloaded.desktop_settings.auto_start, true);
    assert_eq!(reloaded.desktop_settings.close_to_tray, false);
    assert_eq!(reloaded.get_desktop_settings().auto_start, true);

    // Update via update_desktop_settings helper
    let mut updated = reloaded.get_desktop_settings().clone();
    updated.dynamic_tray_icon = false;
    updated.gpu_encoding = false;
    let mut vault_to_update = reloaded;
    vault_to_update.update_desktop_settings(updated).unwrap();

    let reloaded_after_helper = SessionVault::load_from_path(&file_path).unwrap();
    assert_eq!(reloaded_after_helper.desktop_settings.dynamic_tray_icon, false);
    assert_eq!(reloaded_after_helper.desktop_settings.gpu_encoding, false);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

