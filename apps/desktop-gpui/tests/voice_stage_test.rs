use desktop_gpui::theme;
use desktop_gpui::ui::voice_stage::{calculate_grid_columns, VoiceStageCard};

#[test]
fn test_voice_stage_grid_layout_and_cards() {
    // Column count based on participant count
    assert_eq!(calculate_grid_columns(0), 1);
    assert_eq!(calculate_grid_columns(1), 1);
    assert_eq!(calculate_grid_columns(2), 2);
    assert_eq!(calculate_grid_columns(3), 2);
    assert_eq!(calculate_grid_columns(4), 2);
    assert_eq!(calculate_grid_columns(6), 3);
    assert_eq!(calculate_grid_columns(9), 3);
    assert_eq!(calculate_grid_columns(12), 4);
    assert_eq!(calculate_grid_columns(25), 5);

    let card = VoiceStageCard {
        user_id: 101,
        username: "Alex".to_string(),
        avatar_url: None,
        is_speaking: true,
        is_muted: false,
        is_deafened: false,
        volume: 1.0,
    };

    assert!(card.is_speaking);
    assert_eq!(card.border_color(), Some(theme::STATUS_ONLINE));

    let quiet_card = VoiceStageCard {
        user_id: 102,
        username: "Bob".to_string(),
        avatar_url: None,
        is_speaking: false,
        is_muted: true,
        is_deafened: false,
        volume: 1.0,
    };
    assert!(!quiet_card.is_speaking);
    assert_eq!(quiet_card.border_color(), None);
}
