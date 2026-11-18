use sharkord_core::updater::{
    find_platform_asset, is_newer_version, parse_semver, ReleaseAsset, ReleaseInfo, UpdateStatus,
    UpdaterClient,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_semver_parsing_and_version_comparison() {
    assert_eq!(parse_semver("0.1.0"), Some((0, 1, 0)));
    assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_semver("v2.0.0-beta.1"), Some((2, 0, 0)));
    assert_eq!(parse_semver("desktop-v0.2.4"), Some((0, 2, 4)));
    assert_eq!(parse_semver("mobile-v1.0.5"), Some((1, 0, 5)));
    assert_eq!(parse_semver("invalid"), None);

    assert!(is_newer_version("0.1.0", "0.1.1"));
    assert!(is_newer_version("0.1.0", "v0.2.0"));
    assert!(is_newer_version("0.2.3", "desktop-v0.2.4"));
    assert!(is_newer_version("1.0.0", "2.0.0"));
    assert!(!is_newer_version("0.2.0", "0.1.9"));
    assert!(!is_newer_version("1.0.0", "1.0.0"));
}

#[test]
fn test_platform_asset_precedence_matching() {
    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "### Changelog\n- Added voice grid".to_string(),
        published_at: "2026-11-17T00:00:00Z".to_string(),
        assets: vec![
            ReleaseAsset {
                name: "sharkord-plus-windows-x64.zip".to_string(),
                size: 15_000_000,
                browser_download_url: "https://example.com/win.zip".to_string(),
                content_type: "application/zip".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-windows-x64.msi".to_string(),
                size: 16_000_000,
                browser_download_url: "https://example.com/win.msi".to_string(),
                content_type: "application/x-msi".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-macos-aarch64.dmg".to_string(),
                size: 18_000_000,
                browser_download_url: "https://example.com/mac.dmg".to_string(),
                content_type: "application/x-apple-diskimage".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-darwin-x86_64.dmg".to_string(),
                size: 18_500_000,
                browser_download_url: "https://example.com/mac-x64.dmg".to_string(),
                content_type: "application/x-apple-diskimage".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-linux-x86_64.AppImage".to_string(),
                size: 20_000_000,
                browser_download_url: "https://example.com/linux.AppImage".to_string(),
                content_type: "application/x-executable".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-linux-amd64.deb".to_string(),
                size: 14_000_000,
                browser_download_url: "https://example.com/linux.deb".to_string(),
                content_type: "application/vnd.debian.binary-package".to_string(),
            },
        ],
    };

    // Windows preference: .msi over .zip
    let win_asset = find_platform_asset(&release, "windows", "x86_64").unwrap();
    assert_eq!(win_asset.name, "sharkord-plus-windows-x64.msi");

    // macOS preference: .dmg
    let mac_asset = find_platform_asset(&release, "macos", "aarch64").unwrap();
    assert_eq!(mac_asset.name, "sharkord-plus-macos-aarch64.dmg");

    // macOS / Darwin preference and substring collision check (darwin vs win)
    let darwin_arm_asset = find_platform_asset(&release, "darwin", "aarch64").unwrap();
    assert_eq!(darwin_arm_asset.name, "sharkord-plus-macos-aarch64.dmg");

    let darwin_x64_asset = find_platform_asset(&release, "darwin", "x86_64").unwrap();
    assert_eq!(darwin_x64_asset.name, "sharkord-plus-darwin-x86_64.dmg");

    // Linux preference: .AppImage over .deb
    let linux_asset = find_platform_asset(&release, "linux", "x86_64").unwrap();
    assert_eq!(linux_asset.name, "sharkord-plus-linux-x86_64.AppImage");
}

#[test]
fn test_darwin_win_substring_collision() {
    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "".to_string(),
        published_at: "".to_string(),
        assets: vec![
            ReleaseAsset {
                name: "sharkord-plus-windows-x64.msi".to_string(),
                size: 10_000,
                browser_download_url: "https://example.com/win.msi".to_string(),
                content_type: "".to_string(),
            },
            ReleaseAsset {
                name: "sharkord-plus-darwin-x64.tar.gz".to_string(),
                size: 10_000,
                browser_download_url: "https://example.com/darwin.tar.gz".to_string(),
                content_type: "".to_string(),
            },
        ],
    };

    let asset = find_platform_asset(&release, "darwin", "x86_64").unwrap();
    assert_eq!(asset.name, "sharkord-plus-darwin-x64.tar.gz");
    assert_ne!(asset.name, "sharkord-plus-windows-x64.msi");
}

#[test]
fn test_update_status_default_and_serialization() {
    let status = UpdateStatus::default();
    assert_eq!(status, UpdateStatus::Idle);

    let serialized = serde_json::to_string(&status).unwrap();
    let deserialized: UpdateStatus = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, UpdateStatus::Idle);

    let downloading = UpdateStatus::Downloading {
        percent: 0.5,
        downloaded_bytes: 500,
        total_bytes: 1000,
    };
    let serialized_downloading = serde_json::to_string(&downloading).unwrap();
    let deserialized_downloading: UpdateStatus =
        serde_json::from_str(&serialized_downloading).unwrap();
    assert_eq!(deserialized_downloading, downloading);
}

#[tokio::test]
async fn test_updater_client_check_for_update() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "tag_name": "v0.3.0",
        "name": "Sharkord Plus 0.3.0",
        "body": "New release features",
        "published_at": "2026-11-17T12:00:00Z",
        "assets": [
            {
                "name": "sharkord-plus-windows-x64.msi",
                "size": 18000000,
                "browser_download_url": "https://example.com/download.msi",
                "content_type": "application/x-msi"
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/repos/HomelessAvatar/sharkord_plus/releases/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let client = UpdaterClient::with_base_url("HomelessAvatar", "sharkord_plus", mock_server.uri());

    let status = client.check_for_update("0.2.0").await.unwrap();
    match status {
        UpdateStatus::UpdateAvailable(info) => {
            assert_eq!(info.tag_name, "v0.3.0");
            assert_eq!(info.version, "0.3.0");
            assert_eq!(info.assets.len(), 1);
        }
        other => panic!("Expected UpdateAvailable, got {:?}", other),
    }

    let up_to_date_status = client.check_for_update("0.3.0").await.unwrap();
    assert_eq!(up_to_date_status, UpdateStatus::UpToDate);
}
