use desktop_gpui::state::AppState;
use desktop_gpui::theme;

fn main() {
    let state = AppState::new();
    println!(
        "Sharkord desktop-gpui starting with theme background base: {}",
        theme::BG_BASE_HEX
    );
    let _ = state;
}
