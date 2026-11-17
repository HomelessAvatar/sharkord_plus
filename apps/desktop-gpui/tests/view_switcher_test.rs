use desktop_gpui::state::AppState;
use desktop_gpui::ui::{ActiveContentView, WorkspaceView};
use sharkord_core::types::{Channel, ChannelType};

#[test]
fn test_workspace_center_view_switching() {
    let mut state = AppState::new();
    state.set_channels(vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            topic: None,
            private: false,
            is_dm: false,
            position: 0,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
        Channel {
            id: 2,
            name: "lounge-voice".to_string(),
            channel_type: ChannelType::Voice,
            topic: None,
            private: false,
            is_dm: false,
            position: 1,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
    ]);

    let mut workspace = WorkspaceView::new(state);

    // Default view for text channel is Chat
    workspace.select_channel(1);
    assert_eq!(workspace.active_center_view, ActiveContentView::Chat);
    assert_eq!(workspace.active_center_view(), ActiveContentView::Chat);

    // Selecting a voice channel automatically switches center view to VoiceStage
    workspace.select_channel(2);
    assert_eq!(workspace.active_center_view, ActiveContentView::VoiceStage);
    assert_eq!(
        workspace.active_center_view(),
        ActiveContentView::VoiceStage
    );

    // User can manually toggle center view mode
    workspace.set_center_view(ActiveContentView::Chat);
    assert_eq!(workspace.active_center_view, ActiveContentView::Chat);
    assert_eq!(workspace.active_center_view(), ActiveContentView::Chat);

    // User can manually toggle center view mode to ScreenShare
    workspace.set_center_view(ActiveContentView::ScreenShare);
    assert_eq!(workspace.active_center_view, ActiveContentView::ScreenShare);
    assert_eq!(
        workspace.active_center_view(),
        ActiveContentView::ScreenShare
    );
}
