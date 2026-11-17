use desktop_gpui::state::AppState;
use desktop_gpui::ui::member_list::{
    compose_member_groups, status_indicator_color, MemberListSidebarView,
};
use desktop_gpui::ui::WorkspaceView;
use sharkord_core::types::User;

#[test]
fn test_member_list_grouping_and_view() {
    let users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            status: Some("online".to_string()),
            ..Default::default()
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            status: Some("offline".to_string()),
            ..Default::default()
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
            status: Some("dnd".to_string()),
            ..Default::default()
        },
    ];

    let groups = compose_member_groups(&users);
    assert_eq!(groups.len(), 2); // ONLINE (Alice, Charlie) and OFFLINE (Bob)

    let online_group = groups
        .iter()
        .find(|g| g.title.starts_with("ONLINE"))
        .unwrap();
    assert_eq!(online_group.members.len(), 2);

    let offline_group = groups
        .iter()
        .find(|g| g.title.starts_with("OFFLINE"))
        .unwrap();
    assert_eq!(offline_group.members.len(), 1);

    let view = MemberListSidebarView::new(&users);
    assert_eq!(view.total_members_count(), 3);
}

#[test]
fn test_member_list_edge_cases_and_status() {
    // Empty list
    let empty_groups = compose_member_groups(&[]);
    assert!(empty_groups.is_empty());

    let empty_view = MemberListSidebarView::new(&[]);
    assert_eq!(empty_view.total_members_count(), 0);

    // Idle and None status
    let users = vec![
        User {
            id: 10,
            name: "Dave".to_string(),
            status: Some("idle".to_string()),
            ..Default::default()
        },
        User {
            id: 11,
            name: "Eve".to_string(),
            status: None,
            ..Default::default()
        },
    ];

    let groups = compose_member_groups(&users);
    assert_eq!(groups.len(), 2);

    let online = groups
        .iter()
        .find(|g| g.title.starts_with("ONLINE"))
        .unwrap();
    assert_eq!(online.members[0].name, "Dave");

    let offline = groups
        .iter()
        .find(|g| g.title.starts_with("OFFLINE"))
        .unwrap();
    assert_eq!(offline.members[0].name, "Eve");

    // Status colors
    assert_eq!(
        status_indicator_color(Some("online")),
        desktop_gpui::theme::status_online()
    );
    assert_eq!(
        status_indicator_color(Some("idle")),
        desktop_gpui::theme::status_idle()
    );
    assert_eq!(
        status_indicator_color(Some("dnd")),
        desktop_gpui::theme::status_dnd()
    );
    assert_eq!(
        status_indicator_color(Some("offline")),
        desktop_gpui::theme::status_offline()
    );
    assert_eq!(
        status_indicator_color(None),
        desktop_gpui::theme::status_offline()
    );
}

#[test]
fn test_workspace_view_member_list_integration() {
    let mut state = AppState::new();
    state.users = vec![
        User {
            id: 1,
            name: "Alice".to_string(),
            status: Some("online".to_string()),
            ..Default::default()
        },
        User {
            id: 2,
            name: "Bob".to_string(),
            status: Some("offline".to_string()),
            ..Default::default()
        },
    ];

    let mut workspace = WorkspaceView::new(state);
    assert_eq!(workspace.member_list.total_members_count(), 2);
    assert!(workspace.state.right_sidebar_open);

    workspace.toggle_right_sidebar();
    assert!(!workspace.state.right_sidebar_open);
}
