use std::path::PathBuf;
use desktop_gpui::ui::chat_canvas::{
    format_file_size, render_upload_icon, ChatCanvasView, PendingAttachment,
};
use desktop_gpui::ui::lightbox_modal::{LightboxModalView, LightboxState};
use sharkord_core::types::{Message, MessageFile};

#[test]
fn test_lightbox_open_and_close() {
    let mut lightbox = LightboxState::new();
    assert!(!lightbox.is_open);
    assert_eq!(lightbox.image_url, None);
    assert_eq!(lightbox.filename, None);
    assert_eq!(lightbox.zoom_level, 1.0);

    lightbox.open("http://localhost:4991/uploads/test.png");
    assert!(lightbox.is_open);
    assert_eq!(
        lightbox.image_url.as_deref(),
        Some("http://localhost:4991/uploads/test.png")
    );
    assert_eq!(lightbox.filename, None);

    lightbox.open_with_name(
        "http://localhost:4991/uploads/photo.jpg",
        "vacation_photo.jpg",
    );
    assert!(lightbox.is_open);
    assert_eq!(
        lightbox.image_url.as_deref(),
        Some("http://localhost:4991/uploads/photo.jpg")
    );
    assert_eq!(lightbox.filename.as_deref(), Some("vacation_photo.jpg"));

    // Zoom manipulations
    lightbox.zoom_in();
    assert!(lightbox.zoom_level > 1.0);
    assert_eq!(lightbox.zoom_percentage_text(), "125%");

    lightbox.zoom_out();
    lightbox.zoom_out();
    assert!(lightbox.zoom_level < 1.0);
    assert_eq!(lightbox.zoom_percentage_text(), "75%");

    lightbox.reset_zoom();
    assert_eq!(lightbox.zoom_level, 1.0);
    assert_eq!(lightbox.zoom_percentage_text(), "100%");

    // Close resets state
    lightbox.close();
    assert!(!lightbox.is_open);
    assert_eq!(lightbox.image_url, None);
    assert_eq!(lightbox.filename, None);
    assert_eq!(lightbox.zoom_level, 1.0);
}

#[test]
fn test_composer_attachments_management() {
    // 1. File size formatting
    assert_eq!(format_file_size(0), "0 B");
    assert_eq!(format_file_size(512), "512 B");
    assert_eq!(format_file_size(1024), "1.0 KB");
    assert_eq!(format_file_size(1024 * 512), "512.0 KB");
    assert_eq!(format_file_size(1024 * 1024 * 5), "5.0 MB");
    assert_eq!(format_file_size(1024 * 1024 * 1024 * 2), "2.00 GB");

    // 2. Image vs document detection
    let img_att = PendingAttachment::new("photos/screenshot.png").with_size(1024 * 200);
    assert!(img_att.is_image);
    assert_eq!(img_att.filename, "screenshot.png");
    assert_eq!(img_att.formatted_size(), "200.0 KB");

    let doc_att = PendingAttachment::new("docs/archive.zip").with_size(1024 * 1024 * 12);
    assert!(!doc_att.is_image);
    assert_eq!(doc_att.filename, "archive.zip");
    assert_eq!(doc_att.formatted_size(), "12.0 MB");

    let code_att = PendingAttachment::new("main.rs");
    assert!(!code_att.is_image);
    assert_eq!(code_att.filename, "main.rs");

    // 3. Adding, removing, clearing on ChatCanvasView
    let mut canvas = ChatCanvasView::default();
    assert!(canvas.pending_attachments.is_empty());

    canvas.add_pending_attachment(img_att.clone());
    assert_eq!(canvas.pending_attachments.len(), 1);
    assert_eq!(canvas.pending_attachments[0].filename, "screenshot.png");

    canvas.add_pending_attachment(doc_att.clone());
    assert_eq!(canvas.pending_attachments.len(), 2);

    canvas.remove_pending_attachment(&img_att.id);
    assert_eq!(canvas.pending_attachments.len(), 1);
    assert_eq!(canvas.pending_attachments[0].filename, "archive.zip");

    // handle_file_drop
    canvas.handle_file_drop(&[PathBuf::from("dropped_image.webp")]);
    assert_eq!(canvas.pending_attachments.len(), 2);
    assert_eq!(canvas.pending_attachments[1].filename, "dropped_image.webp");
    assert!(canvas.pending_attachments[1].is_image);

    canvas.clear_pending_attachments();
    assert!(canvas.pending_attachments.is_empty());
}

#[test]
fn test_lightbox_modal_view_rendering() {
    let mut state = LightboxState::new();
    state.open_with_name(
        "http://localhost:4991/uploads/diagram.png",
        "network_diagram.png",
    );
    assert_eq!(state.zoom_percentage_text(), "100%");

    state.zoom_in();
    assert_eq!(state.zoom_percentage_text(), "125%");

    let view = LightboxModalView::new(state);
    let _element = view.render_element();
}

#[test]
fn test_chat_canvas_attachment_bar_and_upload_button() {
    let _upload_icon = render_upload_icon();

    let mut canvas = ChatCanvasView::default();
    let _render_empty = canvas.render_element();

    // Add pending attachment and verify rendering includes preview strip
    let pending = PendingAttachment::new("mock_upload.png").with_size(1024 * 150);
    canvas.add_pending_attachment(pending);
    assert_eq!(canvas.pending_attachments.len(), 1);

    let _render_with_pending = canvas.render_element();

    // Verify message attachments rendering
    let msg_with_attachments = Message {
        id: 42,
        channel_id: 1,
        content: Some("Check out these files:".to_string()),
        files: vec![
            MessageFile {
                id: 101,
                name: "file_101.png".to_string(),
                original_name: "screenshot.png".to_string(),
                size: 204800,
                mime_type: "image/png".to_string(),
                extension: "png".to_string(),
                url: Some("http://localhost:4991/uploads/file_101.png".to_string()),
            },
            MessageFile {
                id: 102,
                name: "file_102.pdf".to_string(),
                original_name: "document.pdf".to_string(),
                size: 1048576,
                mime_type: "application/pdf".to_string(),
                extension: "pdf".to_string(),
                url: Some("http://localhost:4991/uploads/file_102.pdf".to_string()),
            },
        ],
        ..Default::default()
    };

    let mut users = std::collections::HashMap::new();
    users.insert(1, "Alice".to_string());
    let canvas_with_msg = ChatCanvasView::new("general", None, &[msg_with_attachments], &users, "");
    let _render_msg_attachments = canvas_with_msg.render_element();

    // Lightbox open inside canvas
    let mut canvas_lightbox = ChatCanvasView::default();
    assert!(!canvas_lightbox.lightbox.is_open);
    canvas_lightbox.open_lightbox("http://localhost:4991/uploads/test.png", Some("test.png".to_string()));
    assert!(canvas_lightbox.lightbox.is_open);
    let _render_lightbox_open = canvas_lightbox.render_element();

    canvas_lightbox.close_lightbox();
    assert!(!canvas_lightbox.lightbox.is_open);
}
