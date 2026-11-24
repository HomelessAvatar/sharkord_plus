use sharkord_core::vault::{ServerProfile, SessionVault};

#[test]
fn test_vault_save_and_retrieve_profile() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("test_vault.json");
    let mut vault = SessionVault::with_path(vault_path);

    let profile = ServerProfile {
        id: "srv_test".into(),
        name: "Test Server".into(),
        server_url: "http://127.0.0.1:4991".into(),
        token: "jwt_token_123".into(),
        user: None,
    };

    vault.save_profile(profile.clone()).unwrap();
    assert_eq!(
        vault.get_profile("srv_test").unwrap().server_url,
        "http://127.0.0.1:4991"
    );
}

#[test]
fn test_vault_active_profile_and_list() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("test_vault.json");
    let mut vault = SessionVault::with_path(vault_path.clone());

    let profile1 = ServerProfile {
        id: "srv_1".into(),
        name: "Server One".into(),
        server_url: "https://srv1.example.com".into(),
        token: "token_1".into(),
        user: None,
    };

    let profile2 = ServerProfile {
        id: "srv_2".into(),
        name: "Server Two".into(),
        server_url: "https://srv2.example.com".into(),
        token: "token_2".into(),
        user: None,
    };

    vault.save_profile(profile1).unwrap();
    vault.save_profile(profile2).unwrap();

    // First saved profile becomes active by default
    assert_eq!(vault.get_active_profile().unwrap().id, "srv_1");

    let list = vault.list_profiles();
    assert_eq!(list.len(), 2);

    // Verify disk reload
    let reloaded = SessionVault::load_from_path(&vault_path).unwrap();
    assert_eq!(reloaded.list_profiles().len(), 2);
    assert_eq!(reloaded.get_active_profile().unwrap().name, "Server One");
}

#[test]
fn test_vault_remove_profile() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("test_vault.json");
    let mut vault = SessionVault::with_path(vault_path.clone());

    let profile = ServerProfile {
        id: "srv_rem".into(),
        name: "To Remove".into(),
        server_url: "http://localhost:4991".into(),
        token: "tok".into(),
        user: None,
    };

    vault.save_profile(profile).unwrap();
    assert!(vault.get_profile("srv_rem").is_some());

    let removed = vault.remove_profile("srv_rem").unwrap();
    assert!(removed.is_some());
    assert!(vault.get_profile("srv_rem").is_none());
    assert_eq!(vault.list_profiles().len(), 0);

    let reloaded = SessionVault::load_from_path(&vault_path).unwrap();
    assert_eq!(reloaded.list_profiles().len(), 0);
}

#[test]
fn test_vault_startup_persistence_across_separate_instances() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("startup_vault.json");

    // Simulate first run: app creates vault, saves a session, then terminates
    {
        let mut vault_instance_1 = SessionVault::with_path(vault_path.clone());
        let profile = ServerProfile {
            id: "srv_persist".into(),
            name: "Persistent Server".into(),
            server_url: "https://community.sharkord.com".into(),
            token: "session_token_xyz".into(),
            user: None,
        };
        vault_instance_1.save_profile(profile).unwrap();
        assert_eq!(
            vault_instance_1.get_active_profile().unwrap().id,
            "srv_persist"
        );
    }

    // Simulate startup behavior in lib.rs: load_from_path with default fallback
    {
        let vault_instance_2 =
            SessionVault::load_from_path(&vault_path).unwrap_or_else(|_| SessionVault::default());

        let active = vault_instance_2.get_active_profile();
        assert!(
            active.is_some(),
            "Active profile must not be lost across restarts"
        );
        assert_eq!(active.unwrap().id, "srv_persist");
        assert_eq!(active.unwrap().server_url, "https://community.sharkord.com");

        let profiles = vault_instance_2.list_profiles();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].token, "session_token_xyz");
    }
}

#[test]
fn test_desktop_settings_persistence_across_instances() {
    let temp_dir = tempfile::tempdir().unwrap();
    let vault_path = temp_dir.path().join("settings_vault.json");

    // Default settings
    {
        let mut vault_1 = SessionVault::with_path(vault_path.clone());
        let mut settings = vault_1.get_desktop_settings().clone();
        assert!(settings.close_to_tray);
        assert!(!settings.start_to_tray);

        settings.close_to_tray = false;
        settings.start_to_tray = true;
        settings.auto_start = true;
        vault_1.update_desktop_settings(settings).unwrap();
    }

    // Reload from disk
    {
        let vault_2 = SessionVault::load_from_path(&vault_path).unwrap();
        let loaded = vault_2.get_desktop_settings();
        assert!(!loaded.close_to_tray);
        assert!(loaded.start_to_tray);
        assert!(loaded.auto_start);
        assert!(loaded.gpu_encoding);
    }
}
