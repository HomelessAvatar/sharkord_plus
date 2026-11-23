use desktop_gpui::ui::channel_tree::{
    channel_icon, compose_channel_tree, CategorizedChannelGroup, ChannelDisplayItem,
    ChannelTreeView,
};
use desktop_gpui::ui::quick_switcher::{
    QuickSwitcherItem, QuickSwitcherItemType, QuickSwitcherState, QuickSwitcherView,
};
use sharkord_core::types::{Channel, ChannelType, User};

#[test]
fn test_category_collapsible_state() {
    let mut group = CategorizedChannelGroup::new("TEXT CHANNELS");
    assert!(group.is_open);
    assert_eq!(group.name, "TEXT CHANNELS");
    assert_eq!(group.category_name, "TEXT CHANNELS");
    assert_eq!(group.unread_count, 0);

    // Toggle collapse state
    group.toggle();
    assert!(!group.is_open);

    group.toggle();
    assert!(group.is_open);

    group.set_open(false);
    assert!(!group.is_open);

    group.set_open(true);
    assert!(group.is_open);

    // Unread count aggregation and channel list
    let ch1 = ChannelDisplayItem {
        channel: Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        is_active: false,
        has_unread: false,
        voice_participants: Vec::new(),
    };
    let ch2 = ChannelDisplayItem {
        channel: Channel {
            id: 2,
            name: "announcements".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        is_active: false,
        has_unread: true,
        voice_participants: Vec::new(),
    };

    group.add_channel(ch1);
    assert_eq!(group.channels.len(), 1);
    assert_eq!(group.unread_count, 0);

    group.add_channel(ch2);
    assert_eq!(group.channels.len(), 2);
    assert_eq!(group.unread_count, 1);

    // Adding another unread channel
    let ch3 = ChannelDisplayItem {
        channel: Channel {
            id: 3,
            name: "random".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        is_active: false,
        has_unread: true,
        voice_participants: Vec::new(),
    };
    group.add_channel(ch3);
    assert_eq!(group.unread_count, 2);

    // Recalculate unread
    group.channels[1].has_unread = false;
    group.recalculate_unread();
    assert_eq!(group.unread_count, 1);
}

#[test]
fn test_quick_switcher_command_palette_filtering() {
    let mut state = QuickSwitcherState::new();
    assert!(!state.is_open);

    state.open();
    assert!(state.is_open);

    // Set up channels, members, and commands
    let channels = vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            ..Default::default()
        },
        Channel {
            id: 2,
            name: "voice-lounge".to_string(),
            channel_type: ChannelType::Voice,
            ..Default::default()
        },
    ];
    state.set_channels(&channels);

    let members = vec![
        User {
            id: 10,
            name: "Alice".to_string(),
            status: Some("online".to_string()),
            ..Default::default()
        },
        User {
            id: 20,
            name: "Bob".to_string(),
            status: Some("idle".to_string()),
            ..Default::default()
        },
    ];
    state.set_members(&members);

    // Check item presence across categories
    let all = state.filtered_items();
    assert!(all
        .iter()
        .any(|i| i.item_type == QuickSwitcherItemType::Channel && i.title == "general"));
    assert!(all
        .iter()
        .any(|i| i.item_type == QuickSwitcherItemType::VoiceRoom && i.title == "voice-lounge"));
    assert!(all
        .iter()
        .any(|i| i.item_type == QuickSwitcherItemType::Member && i.title == "Alice"));
    assert!(all
        .iter()
        .any(|i| i.item_type == QuickSwitcherItemType::Command && i.title.starts_with("/mute")));

    // Query filtering by channel name
    state.set_query("voice");
    assert_eq!(state.query, "voice");
    let voice_matches = state.filtered_items();
    assert!(voice_matches.iter().any(|i| i.title == "voice-lounge"));

    // Query filtering by member name
    state.set_query("Alice");
    let member_matches = state.filtered_items();
    assert_eq!(member_matches.len(), 1);
    assert_eq!(member_matches[0].item_type, QuickSwitcherItemType::Member);
    assert_eq!(member_matches[0].title, "Alice");

    // Query filtering by command
    state.set_query("/settings");
    let cmd_matches = state.filtered_items();
    assert_eq!(cmd_matches.len(), 1);
    assert_eq!(cmd_matches[0].item_type, QuickSwitcherItemType::Command);
    assert_eq!(cmd_matches[0].title, "/settings");

    // Selection cursor navigation
    state.set_query("");
    assert_eq!(state.selected_index, 0);

    state.move_selection_down();
    assert_eq!(state.selected_index, 1);

    state.move_selection_up();
    assert_eq!(state.selected_index, 0);

    // Upper boundary clamp
    state.move_selection_up();
    assert_eq!(state.selected_index, 0);

    // Add a custom quick switcher item
    let custom_cmd = QuickSwitcherItem::command(999, "/custom", "Custom action");
    state.add_item(custom_cmd);

    // Select item
    let selected = state.selected_item();
    assert!(selected.is_some());

    // Close
    state.close();
    assert!(!state.is_open);
    assert_eq!(state.query, "");
}

#[test]
fn test_channel_tree_view_rendering() {
    let channels = vec![
        Channel {
            id: 1,
            name: "welcome".to_string(),
            channel_type: ChannelType::Text,
            category_id: Some(10),
            ..Default::default()
        },
        Channel {
            id: 2,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            category_id: Some(10),
            ..Default::default()
        },
        Channel {
            id: 3,
            name: "voice-stage".to_string(),
            channel_type: ChannelType::Voice,
            category_id: Some(20),
            ..Default::default()
        },
    ];

    assert_eq!(channel_icon(ChannelType::Text), "#");
    assert_eq!(channel_icon(ChannelType::Voice), "🔊");

    let composed = compose_channel_tree(&channels, Some(2));
    assert!(!composed.is_empty());

    let mut view = ChannelTreeView::new("Test Server", &channels, Some(2));
    assert_eq!(view.server_name, "Test Server");
    assert_eq!(view.active_channel_id, Some(2));
    assert!(!view.groups.is_empty());

    // Check categories and collapse behavior
    assert!(view.groups[0].is_open);
    view.groups[0].unread_count = 3;
    view.groups[0].toggle();
    assert!(!view.groups[0].is_open);

    // Render tree with collapsed and unread groups
    let element = view.render_element();
    let _ = element;
}

#[test]
fn test_quick_switcher_view_rendering() {
    let mut state = QuickSwitcherState::new();
    state.open();

    let channels = vec![Channel {
        id: 1,
        name: "announcements".to_string(),
        channel_type: ChannelType::Text,
        ..Default::default()
    }];
    state.set_channels(&channels);

    let view = QuickSwitcherView::new(state.clone());
    let _element = view.render_element();

    // Query with filter results
    state.set_query("ann");
    let view2 = QuickSwitcherView::new(state.clone());
    let _element2 = view2.render_element();

    // Empty query results
    state.set_query("xyz_non_existent");
    let view3 = QuickSwitcherView::new(state);
    let _element3 = view3.render_element();
}
