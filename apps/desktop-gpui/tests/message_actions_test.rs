use desktop_gpui::ui::chat_canvas::{
    format_reactors_tooltip, render_copy_icon, render_edit_icon, render_pin_icon,
    render_react_icon, render_reply_icon, render_trash_icon, ChatCanvasView, MessageActionToolbar,
    ReactionBadge, ReplyTarget,
};
use desktop_gpui::ui::emoji_picker::{EmojiPickerState, EmojiPickerView};
use sharkord_core::types::{Message, MessageReaction};
use std::collections::HashMap;

#[test]
fn test_message_reply_preview_and_editing() {
    let mut canvas = ChatCanvasView::default();
    assert!(canvas.reply_target.is_none());
    assert!(canvas.editing_message_id.is_none());

    // 1. Setting and clearing ReplyTarget
    let reply = ReplyTarget::new(10, "Alice", "Hello world from Alice!");
    assert_eq!(reply.message_id, 10);
    assert_eq!(reply.sender_name, "Alice");
    assert_eq!(reply.author_name, "Alice");
    assert_eq!(reply.snippet, "Hello world from Alice!");

    canvas.set_reply_target(Some(reply));
    assert!(canvas.reply_target.is_some());
    assert_eq!(canvas.reply_target.as_ref().unwrap().sender_name, "Alice");
    assert_eq!(
        canvas.reply_target.as_ref().unwrap().snippet,
        "Hello world from Alice!"
    );

    canvas.clear_reply_target();
    assert!(canvas.reply_target.is_none());

    // 2. Starting inline editing
    canvas.start_editing(42, "Original message content");
    assert_eq!(canvas.editing_message_id, Some(42));
    assert_eq!(canvas.editing_text, "Original message content");

    // 3. Modifying and saving inline edit
    canvas.editing_text = "Updated message content".to_string();
    let saved = canvas.save_inline_edit();
    assert_eq!(saved, Some((42, "Updated message content".to_string())));
    assert_eq!(canvas.editing_message_id, None);
    assert!(canvas.editing_text.is_empty());

    // 4. Starting and canceling inline editing on Escape
    canvas.start_editing(99, "Another message");
    assert_eq!(canvas.editing_message_id, Some(99));
    canvas.cancel_editing();
    assert_eq!(canvas.editing_message_id, None);
    assert!(canvas.editing_text.is_empty());
}

#[test]
fn test_emoji_picker_categories_and_selection() {
    let mut picker = EmojiPickerState::new();
    assert!(!picker.is_open);
    assert_eq!(
        picker.quick_reactions,
        vec!["👍", "❤️", "😂", "🎉", "🔥", "🚀"]
    );
    assert!(!picker.categories.is_empty());

    // Open picker for a target message
    picker.open_for_message(101);
    assert!(picker.is_open);
    assert_eq!(picker.target_message_id, Some(101));

    // Search filtering
    picker.search("fire");
    let filtered = picker.filtered_emojis();
    assert!(!filtered.is_empty());
    assert!(filtered.iter().any(|e| e.emoji == "🔥"));

    picker.search("heart");
    let heart_filtered = picker.filtered_emojis();
    assert!(heart_filtered.iter().any(|e| e.emoji == "❤️"));

    // Reset search query
    picker.search("");
    let all_emojis = picker.filtered_emojis();
    assert!(all_emojis.len() > 10);

    // Selecting category
    picker.select_category(1);
    assert_eq!(picker.selected_category, 1);

    // Selecting an emoji closes picker and returns selected emoji
    let selected = picker.select_emoji("🚀");
    assert_eq!(selected, Some("🚀".to_string()));
    assert!(!picker.is_open);
    assert_eq!(picker.target_message_id, None);

    // EmojiPickerView element tree rendering
    let view = EmojiPickerView::new(picker);
    let _element = view.render_element();
}

#[test]
fn test_reaction_badges_and_tooltips() {
    // Tooltip formatting tests
    assert_eq!(format_reactors_tooltip(&[]), "");
    assert_eq!(format_reactors_tooltip(&["Alice".to_string()]), "Alice");
    assert_eq!(
        format_reactors_tooltip(&["Alice".to_string(), "Bob".to_string()]),
        "Alice and Bob"
    );
    assert_eq!(
        format_reactors_tooltip(&[
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string()
        ]),
        "Alice, Bob, and Charlie"
    );
    assert_eq!(
        format_reactors_tooltip(&[
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
            "Dave".to_string()
        ]),
        "Alice, Bob, and 2 others"
    );

    // ReactionBadge state
    let mut badge =
        ReactionBadge::new("🔥", 3, false, vec!["Alice".to_string(), "Bob".to_string()]);
    assert_eq!(badge.emoji, "🔥");
    assert_eq!(badge.count, 3);
    assert!(!badge.has_reacted);
    assert_eq!(badge.tooltip_text(), "Alice and Bob");

    // Toggle reaction
    badge.toggle_reacted();
    assert!(badge.has_reacted);
    assert_eq!(badge.count, 4);

    badge.toggle_reacted();
    assert!(!badge.has_reacted);
    assert_eq!(badge.count, 3);

    // Badge rendering
    let _badge_el = badge.render_element();

    // Grouping message reactions from Message data
    let reactions = vec![
        MessageReaction {
            message_id: 1,
            user_id: 10,
            emoji: "👍".to_string(),
            plugin_id: None,
            file_id: None,
            created_at: Some(100),
        },
        MessageReaction {
            message_id: 1,
            user_id: 20,
            emoji: "👍".to_string(),
            plugin_id: None,
            file_id: None,
            created_at: Some(101),
        },
        MessageReaction {
            message_id: 1,
            user_id: 10,
            emoji: "❤️".to_string(),
            plugin_id: None,
            file_id: None,
            created_at: Some(102),
        },
    ];

    let mut users = HashMap::new();
    users.insert(10, "Alice".to_string());
    users.insert(20, "Bob".to_string());

    let badges = ReactionBadge::from_reactions(&reactions, Some(10), &users);
    assert_eq!(badges.len(), 2);

    let thumbs = badges.iter().find(|b| b.emoji == "👍").unwrap();
    assert_eq!(thumbs.count, 2);
    assert!(thumbs.has_reacted); // User 10 reacted
    assert_eq!(thumbs.reactor_names, vec!["Alice", "Bob"]);

    let heart = badges.iter().find(|b| b.emoji == "❤️").unwrap();
    assert_eq!(heart.count, 1);
    assert!(heart.has_reacted); // User 10 reacted
}

#[test]
fn test_message_action_toolbar_and_context_menu() {
    // Vector icon renders
    let _react_icon = render_react_icon();
    let _reply_icon = render_reply_icon();
    let _edit_icon = render_edit_icon();
    let _pin_icon = render_pin_icon(false);
    let _unpin_icon = render_pin_icon(true);
    let _trash_icon = render_trash_icon();
    let _copy_icon = render_copy_icon();

    // Toolbar config
    let toolbar_own = MessageActionToolbar::new(101, true, false);
    assert_eq!(toolbar_own.message_id, 101);
    assert!(toolbar_own.can_edit);
    assert!(!toolbar_own.is_pinned);

    let _own_el = toolbar_own.render_element();

    let toolbar_pinned = MessageActionToolbar::new(102, false, true);
    assert!(!toolbar_pinned.can_edit);
    assert!(toolbar_pinned.is_pinned);

    let _pinned_el = toolbar_pinned.render_element();

    // Canvas view integration: rendering with reply banner & action toolbar
    let msg = Message {
        id: 101,
        channel_id: 1,
        user_id: Some(10),
        content: Some("Test message for actions".to_string()),
        reactions: vec![MessageReaction {
            message_id: 101,
            user_id: 10,
            emoji: "🚀".to_string(),
            plugin_id: None,
            file_id: None,
            created_at: Some(200),
        }],
        ..Default::default()
    };

    let mut users = HashMap::new();
    users.insert(10, "Alice".to_string());

    let mut canvas = ChatCanvasView::new("general", None, &[msg], &users, "");
    canvas.set_reply_target(Some(ReplyTarget::new(
        101,
        "Alice",
        "Test message for actions",
    )));
    canvas.start_editing(101, "Test message for actions");

    let _canvas_el = canvas.render_element();
}
