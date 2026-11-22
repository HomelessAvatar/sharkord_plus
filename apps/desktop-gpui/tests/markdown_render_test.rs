use desktop_gpui::ui::chat_canvas::{ChatCanvasView, ComposerAction, ComposerTextarea};
use desktop_gpui::ui::markdown::{
    parse_markdown_tokens, render_code_block, render_markdown_element, MarkdownToken, SpoilerState,
};
use sharkord_core::types::Message;
use std::collections::HashMap;

#[test]
fn test_parse_markdown_bold_codeblock_spoiler() {
    let input = "**bold text** and `inline code` and ||spoiler||";
    let tokens = parse_markdown_tokens(input);
    assert!(tokens.contains(&MarkdownToken::Bold("bold text".to_string())));
    assert!(tokens.contains(&MarkdownToken::InlineCode("inline code".to_string())));
    assert!(tokens.contains(&MarkdownToken::Spoiler("spoiler".to_string())));
}

#[test]
fn test_parse_markdown_all_token_types() {
    let input = r#"# Main Header
This is *italic text* and ~~strikethrough~~.
> This is a quoted block
Check out [Sharkord](https://sharkord.example.com) now.
```rust
fn main() {
    println!("Hello, Sharkord!");
}
```"#;

    let tokens = parse_markdown_tokens(input);

    assert!(tokens.contains(&MarkdownToken::Heading("Main Header".to_string())));
    assert!(tokens.contains(&MarkdownToken::Italic("italic text".to_string())));
    assert!(tokens.contains(&MarkdownToken::Strikethrough("strikethrough".to_string())));
    assert!(tokens.contains(&MarkdownToken::Blockquote("This is a quoted block".to_string())));
    assert!(tokens.contains(&MarkdownToken::Link {
        title: "Sharkord".to_string(),
        url: "https://sharkord.example.com".to_string(),
    }));

    let code_block_token = tokens.iter().find(|t| matches!(t, MarkdownToken::CodeBlock { .. }));
    assert!(code_block_token.is_some());
    if let Some(MarkdownToken::CodeBlock { language, code }) = code_block_token {
        assert_eq!(language.as_deref(), Some("rust"));
        assert!(code.contains("Hello, Sharkord!"));
    }
}

#[test]
fn test_markdown_rendering_elements_and_spoiler_state() {
    let element = render_markdown_element("**Bold** and `code` and ||hidden||");
    let _any = element;

    let code_block_elem = render_code_block(Some("rust"), "let x = 42;");
    let _any_code = code_block_elem;

    let code_block_no_lang = render_code_block(None, "plain text code");
    let _any_no_lang = code_block_no_lang;

    let mut spoiler = SpoilerState::new("secret answer");
    assert!(!spoiler.is_revealed);
    assert_eq!(spoiler.content, "secret answer");

    spoiler.reveal();
    assert!(spoiler.is_revealed);

    spoiler.toggle();
    assert!(!spoiler.is_revealed);
}

#[test]
fn test_chat_canvas_skeleton_loading_state() {
    let mut canvas = ChatCanvasView::default();
    assert!(!canvas.is_loading);

    canvas.set_loading(true);
    assert!(canvas.is_loading);

    let _rendered = canvas.render_element();

    canvas.set_loading(false);
    assert!(!canvas.is_loading);
}

#[test]
fn test_chat_canvas_virtualized_feed_item_handling() {
    let mut messages = Vec::new();
    for i in 1..=50 {
        messages.push(Message {
            id: i,
            channel_id: 1,
            user_id: Some(10),
            content: Some(format!("Message #{}", i)),
            created_at: Some(1_700_000_000 + (i as i64 * 60)),
            ..Default::default()
        });
    }

    let mut users = HashMap::new();
    users.insert(10, "Tester".to_string());

    let canvas = ChatCanvasView::new("general", Some("Discussions".to_string()), &messages, &users, "");
    assert_eq!(canvas.total_messages_count(), 50);
    assert!(canvas.is_virtualized);

    let _rendered = canvas.render_element();
}

#[test]
fn test_chat_canvas_rich_composer_multiline_and_shortcuts() {
    let mut textarea = ComposerTextarea::default();
    assert_eq!(textarea.text, "");
    assert_eq!(textarea.cursor, 0);
    assert_eq!(textarea.line_count(), 1);

    textarea.insert_text("First line");
    assert_eq!(textarea.text, "First line");
    assert_eq!(textarea.cursor, 10);

    textarea.insert_newline();
    assert_eq!(textarea.text, "First line\n");
    assert_eq!(textarea.line_count(), 2);

    textarea.insert_text("Second line");
    assert_eq!(textarea.text, "First line\nSecond line");
    assert_eq!(textarea.line_count(), 2);

    textarea.move_cursor_left();
    assert_eq!(textarea.cursor, textarea.text.len() - 1);

    textarea.select_all();
    assert_eq!(textarea.selection, Some((0, textarea.text.len())));
    assert_eq!(textarea.selected_text().as_deref(), Some("First line\nSecond line"));

    let mut canvas = ChatCanvasView::default();
    let action_shift_enter = canvas.handle_composer_key("enter", true);
    assert_eq!(action_shift_enter, ComposerAction::Newline);
    assert_eq!(canvas.textarea.text, "\n");

    canvas.textarea.clear();
    canvas.textarea.insert_text("Ready to send");
    let action_enter = canvas.handle_composer_key("enter", false);
    assert_eq!(action_enter, ComposerAction::Send);

    let action_char = canvas.handle_composer_key("a", false);
    assert_eq!(action_char, ComposerAction::InputUpdated);
}
