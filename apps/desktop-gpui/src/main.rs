use desktop_gpui::state::AppState;
use desktop_gpui::theme;
use desktop_gpui::ui::WorkspaceView;
use gpui::prelude::*;
use gpui::{px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions};
use sharkord_core::types::{Channel, ChannelType, ServerSettings, User};

fn main() {
    println!(
        "Sharkord desktop-gpui starting with theme background base: {}",
        theme::BG_BASE_HEX
    );

    let mut initial_state = AppState::new();
    initial_state.server_settings = Some(ServerSettings {
        server_id: "sharkord-main".to_string(),
        name: "Sharkord Plus HQ".to_string(),
        is_plus: Some(true),
        ..Default::default()
    });
    initial_state.current_user = Some(User {
        id: 1,
        name: "User".to_string(),
        status: Some("online".to_string()),
        ..Default::default()
    });
    initial_state.set_channels(vec![
        Channel {
            id: 1,
            name: "general".to_string(),
            channel_type: ChannelType::Text,
            topic: Some("General discussion for Sharkord Plus".to_string()),
            private: false,
            is_dm: false,
            position: 0,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
        Channel {
            id: 2,
            name: "voice-lounge".to_string(),
            channel_type: ChannelType::Voice,
            topic: None,
            private: false,
            is_dm: false,
            position: 1,
            category_id: None,
            created_at: Some(1000),
            updated_at: None,
        },
    ]);
    initial_state.select_channel(1);

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
            cx.new(|_| WorkspaceView::new(state_clone))
        })
        .expect("failed to open window");

        cx.activate(true);
    });
}
