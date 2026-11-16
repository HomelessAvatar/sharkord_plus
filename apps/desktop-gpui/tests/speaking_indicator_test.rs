use std::collections::HashMap;

use desktop_gpui::theme;
use desktop_gpui::ui::channel_tree::{
    compose_channel_tree_with_participants, VoiceParticipantDisplay,
};
use sharkord_core::types::{Channel, ChannelType};

#[test]
fn test_voice_channel_participants_and_speaking_indicator() {
    let channels = vec![
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
            name: "Lobby Voice".to_string(),
            channel_type: ChannelType::Voice,
            topic: None,
            private: false,
            is_dm: false,
            position: 1,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
    ];

    let mut participants = HashMap::new();
    participants.insert(
        2,
        vec![
            VoiceParticipantDisplay {
                user_id: 10,
                name: "Alice".to_string(),
                is_speaking: true,
                is_muted: false,
                is_deafened: false,
            },
            VoiceParticipantDisplay {
                user_id: 20,
                name: "Bob".to_string(),
                is_speaking: false,
                is_muted: true,
                is_deafened: false,
            },
        ],
    );

    let groups = compose_channel_tree_with_participants(&channels, Some(2), &participants);
    assert_eq!(groups.len(), 1);
    let voice_channel = groups[0]
        .channels
        .iter()
        .find(|c| c.channel.id == 2)
        .expect("voice channel exists");

    assert_eq!(voice_channel.voice_participants.len(), 2);

    let alice = &voice_channel.voice_participants[0];
    assert_eq!(alice.name, "Alice");
    assert!(alice.is_speaking);
    assert_eq!(alice.speaking_border_color(), Some(theme::STATUS_ONLINE));

    let bob = &voice_channel.voice_participants[1];
    assert_eq!(bob.name, "Bob");
    assert!(!bob.is_speaking);
    assert_eq!(bob.speaking_border_color(), None);
    assert!(bob.is_muted);
}
