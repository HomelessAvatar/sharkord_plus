use std::fs;
use std::path::Path;

#[test]
fn test_packager_configuration_file_validity() {
    let config_path = Path::new("packager.toml");
    let full_path = if config_path.exists() {
        config_path.to_path_buf()
    } else {
        Path::new("apps/desktop-gpui/packager.toml").to_path_buf()
    };

    assert!(full_path.exists(), "packager.toml must exist");
    let content = fs::read_to_string(&full_path).expect("Failed to read packager.toml");

    // Must define product name and bundle identifier
    assert!(content.contains("product_name = \"VibeShark\""));
    assert!(content.contains("identifier = \"com.homelessavatar.sharkord\""));

    // Must define authors and description
    assert!(content.contains("authors = [\"HomelessAvatar\"]"));
    assert!(content.contains("description = "));

    // Must define binaries
    assert!(content.contains("binaries = ["));
    assert!(content.contains("desktop-gpui"));

    // Must define icons
    assert!(content.contains("icons = ["));

    // Verify icon assets exist on disk
    let icon_dir = full_path.parent().unwrap().join("assets").join("icons");
    assert!(icon_dir.join("icon.ico").exists(), "icon.ico must exist");
    assert!(icon_dir.join("icon.icns").exists(), "icon.icns must exist");
    assert!(
        icon_dir.join("128x128.png").exists(),
        "128x128.png must exist"
    );
    assert!(icon_dir.join("icon.png").exists(), "icon.png must exist");

    // Verify icon file formats / magic bytes
    let png_magic = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let icon_png = fs::read(icon_dir.join("icon.png")).expect("Failed to read icon.png");
    assert!(
        icon_png.starts_with(&png_magic),
        "icon.png must have valid PNG magic bytes"
    );

    let png_128 = fs::read(icon_dir.join("128x128.png")).expect("Failed to read 128x128.png");
    assert!(
        png_128.starts_with(&png_magic),
        "128x128.png must have valid PNG magic bytes"
    );

    let ico_data = fs::read(icon_dir.join("icon.ico")).expect("Failed to read icon.ico");
    // Windows ICO magic: 0x00, 0x00, 0x01, 0x00
    assert!(
        ico_data.starts_with(&[0x00, 0x00, 0x01, 0x00]),
        "icon.ico must have valid ICO magic bytes"
    );

    let icns_data = fs::read(icon_dir.join("icon.icns")).expect("Failed to read icon.icns");
    // Apple ICNS magic: 'icns' (0x69, 0x63, 0x6E, 0x73)
    assert!(
        icns_data.starts_with(b"icns"),
        "icon.icns must have valid ICNS magic bytes"
    );

    // Verify valid packager sections
    assert!(
        content.contains("[deb]"),
        "packager.toml should configure deb packages under [deb]"
    );
    assert!(
        !content.contains("[linux]"),
        "packager.toml must not use invalid [linux] table"
    );
}
