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
