# Task 8 Report: File Uploads, Drag-and-Drop, and Image Lightbox Modal (`gpui_component::Attachment` & `Dialog`)

**Date:** 2026-11-22
**Status:** DONE
**Commit:** `fdd50dc4 feat(desktop-gpui): add file upload attachment surface and image lightbox Dialog`

## Summary of Changes

1. **`crates/sharkord-core/src/types.rs`**:
   - Added `MessageFile` struct:
     - `id: u64`, `name: String`, `original_name: String`, `size: i64`, `mime_type: String`, `extension: String`, `url: Option<String>`.
   - Added `#[serde(default)] pub files: Vec<MessageFile>` to `Message`.
   - Maintained full backwards-compatibility with tRPC serialization/deserialization.

2. **`apps/desktop-gpui/src/ui/lightbox_modal.rs`**:
   - Implemented `LightboxState`:
     - `is_open: bool`, `image_url: Option<String>`, `filename: Option<String>`, `zoom_level: f32`.
     - Methods: `new()`, `open(url)`, `open_with_name(url, name)`, `close()`, `zoom_in()`, `zoom_out()`, `reset_zoom()`, and `zoom_percentage_text()`.
   - Implemented `LightboxModalView`:
     - Full-screen dark overlay with backdrop (`hsla(0.0, 0.0, 0.0, 0.88)`).
     - Header control bar (52px height) with filename, vector image icon, zoom pill (`−`, zoom %, `+`, `1:1 Reset`), "Download" action button, and "Close" button with `[ESC]` badge.
     - Centered zoom-scaled image viewport responding to `zoom_level`.
     - Bottom keyboard navigation hint bar (`ESC to close • Scroll or + / - to zoom • 0 to reset`).
     - Strictly styled using Pure Neutral Dark design tokens (`#0A0B0E` base, `#111216` surface, `#16171d` subsurface, `#1c1e24` elevated, `#3B82F6` accent, `#F3F4F6` text primary).

3. **`apps/desktop-gpui/src/ui/chat_canvas.rs`**:
   - Implemented `format_file_size(bytes: u64) -> String` with B, KB, MB, and GB scale units.
   - Implemented `is_image_extension(ext: &str) -> bool` supporting png, jpg, jpeg, gif, webp, svg, bmp, and ico.
   - Implemented `PendingAttachment` struct:
     - Fields: `id: String`, `path: PathBuf`, `filename: String`, `size_bytes: u64`, `is_image: bool`, `is_uploading: bool`, `progress: f32`, `uploaded_url: Option<String>`.
     - Methods: `new(path)`, `with_size(size)`, `with_progress(progress)`, `formatted_size()`.
   - Implemented `render_upload_icon() -> impl IntoElement`:
     - Clean vector `+` icon for file picker trigger.
   - Implemented `render_message_attachments(files: &[MessageFile]) -> Vec<AnyElement>`:
     - Clickable image preview cards for images with preview thumbnail, original name, and formatted size.
     - File attachment pills for documents/binaries with document icon, name, size, and download indicator.
   - Enhanced `ChatCanvasView`:
     - Added fields: `pending_attachments: Vec<PendingAttachment>`, `lightbox: LightboxState`.
     - Added methods: `add_pending_attachment`, `remove_pending_attachment`, `clear_pending_attachments`, `handle_file_drop` (for drag-and-drop), `pick_files_dialog` (integrating with `rfd::FileDialog`), `open_lightbox`, `close_lightbox`.
     - Rendered pending attachments preview strip above text input in the composer with file icon/thumbnail, name, formatted size, and remove `×` button.
     - Added composer `+` upload button in composer input container.
     - Rendered message attachments beneath markdown content in message feed items.
     - Rendered `LightboxModalView` overlay conditionally when `lightbox.is_open`.

4. **`apps/desktop-gpui/src/ui/mod.rs`**:
   - Exported `pub mod lightbox_modal;`.
   - Re-exported `LightboxModalView`, `LightboxState`, `PendingAttachment`, `format_file_size`, `render_upload_icon`.
   - Updated `WorkspaceView::sync_views` to preserve pending attachments and lightbox state across view updates.
   - Updated `WorkspaceView::send_message` to convert pending attachments into `MessageFile` attachments on the created `Message`, clearing the queue upon dispatch.

5. **`crates/sharkord-mobile/tests/bridge_test.rs`**:
   - Updated exhaustive `Message` literal to include `files: Vec::new()`.

6. **`apps/desktop-gpui/tests/attachment_upload_test.rs`**:
   - Implemented comprehensive TDD test cases:
     - `test_lightbox_open_and_close`: Initial state, opening with URL and name, closing, zooming in/out/reset, zoom percentage badges.
     - `test_composer_attachments_management`: File size formatting (B, KB, MB, GB), image vs document detection, adding, removing, clearing pending attachments, and `handle_file_drop`.
     - `test_lightbox_modal_view_rendering`: Lightbox element rendering, zoom labels, controls.
     - `test_chat_canvas_attachment_bar_and_upload_button`: Upload icon vector rendering, composer preview strip rendering with pending attachments, message attachments rendering (image preview card and file pill), and canvas lightbox open/close transitions.

## Verification

- **TDD Red Phase**: `cargo test -p desktop-gpui --test attachment_upload_test` failed with 26 compilation errors confirming absent interfaces and fields.
- **TDD Green Phase**: `cargo test -p desktop-gpui --test attachment_upload_test` passed all 4 tests in 0.00s.
- **Package Test Suite**: `cargo test -p desktop-gpui` passed all 69 tests across all 20 test binaries.
- **Workspace Test Suite**: `cargo test --workspace` passed cleanly across all crates (`desktop-gpui`, `sharkord-core`, `sharkord-mobile`, `sharkord-tauri`) with 0 failures and 0 regressions.
