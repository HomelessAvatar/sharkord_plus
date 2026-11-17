use desktop_gpui::state::AppState;
use desktop_gpui::ui::updater_modal::{UpdaterBadgeView, UpdaterModalView};
use desktop_gpui::ui::WorkspaceView;
use sharkord_core::updater::{ReleaseAsset, ReleaseInfo, UpdateStatus};

#[test]
fn test_updater_state_and_modal_view_transitions() {
    let mut state = AppState::new();
    assert_eq!(state.update_status, UpdateStatus::Idle);
    assert!(!state.is_update_modal_open);

    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "### What's Changed\n- Screen sharing\n- Discord parity polish".to_string(),
        published_at: "2026-11-17T00:00:00Z".to_string(),
        assets: vec![],
    };

    state.update_status = UpdateStatus::UpdateAvailable(release.clone());
    state.toggle_update_modal();
    assert!(state.is_update_modal_open);

    let modal = UpdaterModalView::new(&state.update_status);
    assert_eq!(modal.action_button_label(), "Download & Install");

    let downloading_status = UpdateStatus::Downloading {
        percent: 0.65,
        downloaded_bytes: 13_000_000,
        total_bytes: 20_000_000,
    };
    let downloading_modal = UpdaterModalView::new(&downloading_status);
    assert_eq!(downloading_modal.progress_percent(), Some(0.65));

    let ready_status = UpdateStatus::ReadyToInstall {
        installer_path: std::path::PathBuf::from("/tmp/installer.msi"),
        version: "0.2.0".to_string(),
    };
    let ready_modal = UpdaterModalView::new(&ready_status);
    assert_eq!(ready_modal.action_button_label(), "Restart & Install");
}

#[test]
fn test_updater_badge_view() {
    let badge = UpdaterBadgeView::new("v0.2.0");
    assert_eq!(badge.badge_label(), "✨ Update Available (v0.2.0)");

    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "Notes".to_string(),
        published_at: "2026-11-17T00:00:00Z".to_string(),
        assets: vec![],
    };

    let status = UpdateStatus::UpdateAvailable(release);
    let from_status_badge = UpdaterBadgeView::from_status(&status);
    assert!(from_status_badge.is_some());
    assert_eq!(
        from_status_badge.unwrap().badge_label(),
        "✨ Update Available (v0.2.0)"
    );

    let idle_badge = UpdaterBadgeView::from_status(&UpdateStatus::Idle);
    assert!(idle_badge.is_none());

    let _element = badge.render_element();
}

#[test]
fn test_app_state_and_workspace_updater_modal_integration() {
    let mut state = AppState::new();
    assert_eq!(state.update_status, UpdateStatus::Idle);
    assert!(!state.is_update_modal_open);

    state.toggle_update_modal();
    assert!(state.is_update_modal_open);

    state.toggle_update_modal();
    assert!(!state.is_update_modal_open);

    let release = ReleaseInfo {
        tag_name: "v0.2.0".to_string(),
        version: "0.2.0".to_string(),
        name: "Sharkord Plus 0.2.0".to_string(),
        body: "### What's Changed\n- Screen sharing".to_string(),
        published_at: "2026-11-17T00:00:00Z".to_string(),
        assets: vec![ReleaseAsset {
            name: "sharkord-plus-windows-x64.msi".to_string(),
            size: 15_000_000,
            browser_download_url: "https://example.com/win.msi".to_string(),
            content_type: "application/x-msi".to_string(),
        }],
    };

    state.set_update_status(UpdateStatus::UpdateAvailable(release));
    assert!(state.update_status.is_update_available());

    let mut workspace = WorkspaceView::new(state);
    assert!(!workspace.state.is_update_modal_open);

    workspace.toggle_update_modal();
    assert!(workspace.state.is_update_modal_open);

    let modal = UpdaterModalView::new(&workspace.state.update_status);
    let _element = modal.render_element();

    let downloading = UpdaterModalView::new(&UpdateStatus::Downloading {
        percent: 0.5,
        downloaded_bytes: 5_000_000,
        total_bytes: 10_000_000,
    });
    let _element_downloading = downloading.render_element();

    let ready = UpdaterModalView::new(&UpdateStatus::ReadyToInstall {
        installer_path: std::path::PathBuf::from("/tmp/installer.msi"),
        version: "0.2.0".to_string(),
    });
    let _element_ready = ready.render_element();
}
