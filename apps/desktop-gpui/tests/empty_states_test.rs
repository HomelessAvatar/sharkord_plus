use desktop_gpui::ui::chat_canvas::render_channel_welcome_card;
use desktop_gpui::ui::voice_stage::render_empty_stage_placeholder;
use desktop_gpui::ui::{ChannelWelcomeCard, EmptyStagePlaceholder};

#[test]
fn test_empty_states_rendering_functions() {
    let welcome = render_channel_welcome_card("general", Some("General discussion topic"));
    assert!(!welcome.channel_name.is_empty());
    assert_eq!(welcome.channel_name, "general");
    assert_eq!(welcome.title_text(), "Welcome to #general!");

    let stage_placeholder = render_empty_stage_placeholder("Voice Lounge");
    assert_eq!(stage_placeholder.channel_name, "Voice Lounge");
    assert_eq!(stage_placeholder.headline(), "It's quiet in here...");
}

#[test]
fn test_welcome_card_without_topic_and_custom_names() {
    let welcome = render_channel_welcome_card("announcements", None);
    assert_eq!(welcome.channel_name, "announcements");
    assert_eq!(welcome.title_text(), "Welcome to #announcements!");
    assert_eq!(welcome.topic, None);

    let welcome_custom = ChannelWelcomeCard::new("dev-chat", Some("Development talk".to_string()));
    assert_eq!(welcome_custom.channel_name, "dev-chat");
    assert_eq!(welcome_custom.title_text(), "Welcome to #dev-chat!");
    assert_eq!(welcome_custom.topic.as_deref(), Some("Development talk"));
}

#[test]
fn test_empty_stage_placeholder_constructors_and_defaults() {
    let stage = render_empty_stage_placeholder("Gaming");
    assert_eq!(stage.channel_name, "Gaming");
    assert_eq!(stage.headline(), "It's quiet in here...");

    let stage_custom = EmptyStagePlaceholder::new("Music Room");
    assert_eq!(stage_custom.channel_name, "Music Room");
    assert_eq!(stage_custom.headline(), "It's quiet in here...");
}
