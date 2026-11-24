// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use desktop_gpui::network::NetworkBridge;
use desktop_gpui::state::AppState;
use desktop_gpui::theme;
use desktop_gpui::tray::DesktopTray;
use desktop_gpui::ui::WorkspaceView;
use gpui::prelude::*;
use gpui::{px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions};
use sharkord_core::vault::SessionVault;

fn main() {
    println!(
        "Sharkord desktop-gpui starting with theme background base: {}",
        theme::BG_BASE_HEX
    );

    let mut vault = SessionVault::default();
    let _ = vault.load();

    let mut initial_state = AppState::new();
    initial_state.desktop_settings = vault.desktop_settings.clone();
    let _tray = DesktopTray::with_settings(vault.desktop_settings.clone());

    let bridge = match NetworkBridge::new() {
        Ok(b) => Some(Arc::new(b)),
        Err(e) => {
            eprintln!("Failed to initialize NetworkBridge: {e}");
            None
        }
    };

    if let Some(session) = vault.get_active().cloned() {
        println!(
            "Found active session for server '{}' (user: {}), applying...",
            session.server_name, session.username
        );
        initial_state.apply_saved_session(&session);
        initial_state.is_connecting = true;
        initial_state.is_auth_modal_open = false;

        if let Some(bridge_arc) = bridge.clone() {
            let bridge_task = bridge_arc.clone();
            bridge_arc.runtime().spawn(async move {
                match bridge_task
                    .connect_with_token(&session.server_url, &session.token)
                    .await
                {
                    Ok(_) => match bridge_task.bootstrap().await {
                        Ok(data) => {
                            println!(
                                "Background bootstrap succeeded: {} channels, {} messages",
                                data.channels.len(),
                                data.initial_messages.len()
                            );
                        }
                        Err(err) => {
                            eprintln!("Background bootstrap failed: {err}");
                        }
                    },
                    Err(err) => {
                        eprintln!(
                            "Failed to connect to server '{}': {err}",
                            session.server_url
                        );
                    }
                }
            });
        }
    } else {
        println!("No active session found in vault, opening auth modal");
        initial_state.is_auth_modal_open = true;
        initial_state.is_connecting = false;
    }

    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("VibeShark".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        let state_clone = initial_state.clone();
        let bridge_clone = bridge.clone();
        cx.open_window(window_options, |_, cx| {
            cx.new(|cx| WorkspaceView::new_with_bridge(state_clone, bridge_clone, cx))
        })
        .expect("failed to open window");

        cx.activate(true);
    });
}
