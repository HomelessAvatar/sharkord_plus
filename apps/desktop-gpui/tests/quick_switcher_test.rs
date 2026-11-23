use desktop_gpui::state::AppState;
use desktop_gpui::ui::quick_switcher::{QuickSwitcherItem, QuickSwitcherState, QuickSwitcherView};
use desktop_gpui::ui::WorkspaceView;
use sharkord_core::types::{Channel, ChannelType};

#[test]
fn test_quick_switcher_filtering_and_navigation() {
    let channels = vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 2,
            name: "announcements".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 3,
            name: "voice-lounge".to_string(),
            channel_type: ChannelType::Voice,
            ..Default::default()
        },
    ];

    let mut switcher = QuickSwitcherState::from_channels(&channels);
    assert_eq!(switcher.filtered_items().len(), 3);
    assert_eq!(switcher.selected_index, 0);
    assert_eq!(
        switcher.selected_item(),
        Some(QuickSwitcherItem {
            id: 1,
            name: "general".to_string(),
            title: "general".to_string(),
            channel_type: ChannelType::Text,
            category: None,
            ..Default::default()
        })
    );

    // Search query filtering
    switcher.set_query("voice");
    let filtered = switcher.filtered_items();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "voice-lounge");
    assert_eq!(switcher.selected_index, 0);
    assert_eq!(
        switcher.selected_item().map(|item| item.name),
        Some("voice-lounge".to_string())
    );

    // Index navigation
    switcher.set_query("");
    switcher.select_next();
    assert_eq!(switcher.selected_index, 1);
    switcher.select_prev();
    assert_eq!(switcher.selected_index, 0);
}

#[test]
fn test_quick_switcher_navigation_boundaries_and_empty() {
    let channels = vec![
        Channel {
            id: 10,
            name: "dev-chat".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 20,
            name: "dev-voice".to_string(),
            channel_type: ChannelType::Voice,
            ..Default::default()
        },
    ];

    let mut switcher = QuickSwitcherState::from_channels(&channels);

    // prev at 0 clamps to 0
    switcher.select_prev();
    assert_eq!(switcher.selected_index, 0);

    // next advances to 1
    switcher.select_next();
    assert_eq!(switcher.selected_index, 1);

    // next at last item clamps to last item
    switcher.select_next();
    assert_eq!(switcher.selected_index, 1);

    // Query with no matches
    switcher.set_query("nonexistent");
    assert!(switcher.filtered_items().is_empty());
    assert_eq!(switcher.selected_item(), None);

    // prev and next do not panic on empty list
    switcher.select_next();
    assert_eq!(switcher.selected_index, 0);
    switcher.select_prev();
    assert_eq!(switcher.selected_index, 0);
}

#[test]
fn test_app_state_and_workspace_quick_switcher_integration() {
    let mut state = AppState::new();
    assert!(!state.is_quick_switcher_open);

    state.toggle_quick_switcher();
    assert!(state.is_quick_switcher_open);

    state.toggle_quick_switcher();
    assert!(!state.is_quick_switcher_open);

    let mut workspace = WorkspaceView::new(state);
    assert!(!workspace.state.is_quick_switcher_open);

    workspace.toggle_quick_switcher();
    assert!(workspace.state.is_quick_switcher_open);

    // QuickSwitcherView can be instantiated
    let view = QuickSwitcherView::new(workspace.quick_switcher.clone());
    let _element = view.render_element();
}
