use desktop_gpui::state::AppState;
use desktop_gpui::theme;
use desktop_gpui::ui::WorkspaceView;
use gpui::prelude::*;
use gpui::{px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions};

fn main() {
    println!(
        "Sharkord desktop-gpui starting with theme background base: {}",
        theme::BG_BASE_HEX
    );

    let initial_state = AppState::new_mock();

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Sharkord Plus".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        let state_clone = initial_state.clone();
        cx.open_window(window_options, |_, cx| {
            cx.new(|cx| WorkspaceView::new_in_context(state_clone, cx))
        })
        .expect("failed to open window");

        cx.activate(true);
    });
}
