use gpui_kit as _;
use openh264 as _;
use rfd::FileDialog;
use webrtc as _;
use xcap as _;

#[test]
fn test_gpui_kit_and_media_crates_importable() {
    let _ = FileDialog::new();
    assert!(true);
}
