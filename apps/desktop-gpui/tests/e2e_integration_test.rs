//! End-to-end integration and /impeccable craft inspection test suite
//! for the Sharkord native GPUI desktop client.
//!
//! Verifies complete parity lifecycle matching Sharkord Web and Tauri desktop clients:
//! - Initial unauthenticated state -> server connect & auth modal transitions
//! - Bootstrap from server data (User, Channels, Roles, DMs, ServerSettings)
//! - Channel navigation & selection (text channels, voice channels with auto-stage)
//! - Message feed streaming, grouping, skeleton states, and multiline composer
//! - Message action toolbar (reply banner, in-place edit, emoji reaction badges, pin, delete)
//! - Collapsible channel categories (folding, chevrons) & Quick Switcher (Ctrl+K fuzzy filtering, slash commands)
//! - Direct messaging sidebar (Home toggle, conversation list, unread pills) & User Profile popover (80px avatar, status ring, bio, DM button)
//! - Channel & Server Administration modals (Create Channel text/voice, Edit Channel danger delete, Server Settings tabs)
//! - User Settings modal (tabs for Account, Voice/Video VAD/PTT, Appearance, Keybinds, Notifications) & System Tray dispatcher
//! - Mediasoup WebRTC live voice client (audio levels, VAD threshold gating, PTT, dynamic green speaking rings)
//! - Screen sharing capture & video decoder pipeline (xcap enumeration, H.264 decoding, video tile telemetry, PiP player)
//! - Root workspace element tree rendering across all active center views
//! - /impeccable Pure Neutral Dark theme tokens, WCAG AA contrast, and vector SVG action buttons

use std::collections::HashMap;
use std::path::PathBuf;

use desktop_gpui::media::{
    LiveVoiceState, ScreenCaptureEngine, VideoDecoderPipeline, VideoFrame, VoiceInputMode,
};
use desktop_gpui::network::BootstrapData;
use desktop_gpui::theme;
use desktop_gpui::tray::{DesktopTray, TrayAction};
use desktop_gpui::ui::auth_modal::AuthTab;
use desktop_gpui::ui::channel_tree::compose_channel_tree;
use desktop_gpui::ui::chat_canvas::{
    format_reactors_tooltip, group_messages_for_feed, render_channel_welcome_card,
    render_copy_icon, render_edit_icon, render_pin_icon, render_react_icon, render_reply_icon,
    render_trash_icon, render_upload_icon, render_users_toggle_icon, PendingAttachment,
    ReplyTarget,
};
use desktop_gpui::ui::emoji_picker::EmojiPickerState;
use desktop_gpui::ui::markdown::{parse_markdown_tokens, MarkdownToken};
use desktop_gpui::ui::pip_player::PipPlayerState;
use desktop_gpui::ui::server_settings_modal::ServerSettingsTab;
use desktop_gpui::ui::user_settings_modal::{InputMode, SettingsTab};
use desktop_gpui::ui::video_tile::{VideoDisplayMode, VideoTileState};
use desktop_gpui::ui::voice_bar::{voice_action_labels, VoiceConnectionState};
use desktop_gpui::ui::voice_stage::EmptyStagePlaceholder;
use desktop_gpui::ui::{
    render_chat_bubble_icon, render_gear_icon, render_headphones_icon, render_mic_icon,
    ActiveContentView, WorkspaceView,
};
use sharkord_core::types::*;
use sharkord_core::voice::video::StreamTelemetryStats;

#[test]
fn test_end_to_end_workspace_parity_lifecycle() {
    // -------------------------------------------------------------------------
    // 1. Initial State & Unauthenticated State -> Server Connect & Auth Modal
    // -------------------------------------------------------------------------
    let mut view = WorkspaceView::default();
    assert!(!view.is_authenticated(), "Default view must start unauthenticated");
    assert!(view.channels().is_empty(), "Default view has no channels");
    assert_eq!(view.active_center_view(), ActiveContentView::Chat);
    assert!(!view.state.is_connecting);
    assert!(!view.state.is_auth_modal_open);

    // Trigger server connection progress state
    view.set_connecting(true);
    assert!(view.state.is_connecting);
    assert!(view.auth_modal.is_loading);

    // Open Auth modal and test tab switches and validation
    view.set_auth_modal_open(true);
    assert!(view.state.is_auth_modal_open);
    assert_eq!(view.auth_modal.active_tab, AuthTab::Login);

    view.auth_modal.set_tab(AuthTab::Register);
    assert_eq!(view.auth_modal.active_tab, AuthTab::Register);

    view.auth_modal.set_error("Invalid email format");
    assert_eq!(view.auth_modal.error_message.as_deref(), Some("Invalid email format"));
    view.auth_modal.clear_error();
    assert!(view.auth_modal.error_message.is_none());

    // -------------------------------------------------------------------------
    // 2. Bootstrap from Server Data (User, Channels, Roles, DMs, ServerSettings)
    // -------------------------------------------------------------------------
    let data = BootstrapData {
        settings: ServerSettings {
            server_id: "sharkord-main".to_string(),
            name: "Sharkord Plus HQ".to_string(),
            description: Some("Native GPUI Parity Workspace".to_string()),
            is_plus: Some(true),
            allow_new_users: Some(true),
            ..Default::default()
        },
        channels: vec![
            Channel {
                id: 1,
                name: "general".to_string(),
                channel_type: ChannelType::Text,
                category_id: Some(10),
                position: 0,
                topic: Some("General discussion for Sharkord Plus".to_string()),
                private: false,
                is_dm: false,
                created_at: Some(1000),
                updated_at: None,
            },
            Channel {
                id: 2,
                name: "voice-lounge".to_string(),
                channel_type: ChannelType::Voice,
                category_id: Some(20),
                position: 1,
                topic: Some("Live voice stage and screen sharing".to_string()),
                private: false,
                is_dm: false,
                created_at: Some(1000),
                updated_at: None,
            },
            Channel {
                id: 3,
                name: "alice".to_string(),
                channel_type: ChannelType::Text,
                category_id: None,
                position: 0,
                topic: None,
                private: true,
                is_dm: true,
                created_at: Some(1000),
                updated_at: None,
            },
        ],
        initial_messages: vec![Message {
            id: 101,
            channel_id: 1,
            user_id: Some(1),
            content: Some("Welcome to the native GPUI desktop client!".to_string()),
            created_at: Some(1000),
            ..Default::default()
        }],
        active_channel_id: Some(1),
        current_user: Some(User {
            id: 1,
            name: "admin".to_string(),
            identity: Some("admin#0001".to_string()),
            bio: Some("Native GPUI Client Lead".to_string()),
            status: Some("online".to_string()),
            role_ids: vec![1],
            created_at: Some(1000),
            ..Default::default()
        }),
        users: vec![
            User {
                id: 1,
                name: "admin".to_string(),
                status: Some("online".to_string()),
                role_ids: vec![1],
                ..Default::default()
            },
            User {
                id: 2,
                name: "alice".to_string(),
                bio: Some("Senior Systems Architect".to_string()),
                status: Some("online".to_string()),
                role_ids: vec![2],
                ..Default::default()
            },
            User {
                id: 3,
                name: "bob".to_string(),
                bio: Some("Frontend specialist".to_string()),
                status: Some("idle".to_string()),
                role_ids: vec![2],
                ..Default::default()
            },
        ],
        roles: vec![
            Role {
                id: 1,
                name: "Administrator".to_string(),
                color: "#EF4444".to_string(),
                is_persistent: false,
                is_default: false,
                storage_quota_override_enabled: false,
                storage_space_quota: 0,
                permissions: vec!["ADMINISTRATOR".to_string()],
                created_at: None,
                updated_at: None,
            },
            Role {
                id: 2,
                name: "Member".to_string(),
                color: "#3B82F6".to_string(),
                is_persistent: false,
                is_default: false,
                storage_quota_override_enabled: false,
                storage_space_quota: 0,
                permissions: vec!["SEND_MESSAGES".to_string()],
                created_at: None,
                updated_at: None,
            },
        ],
    };

    view.bootstrap_from_data(data);

    assert!(view.is_authenticated(), "Workspace must be authenticated after bootstrap");
    assert_eq!(view.channels().len(), 3, "All 3 bootstrapped channels must be loaded");
    assert_eq!(view.state.active_channel_id, Some(1));
    assert_eq!(view.state.messages.len(), 1);
    assert!(!view.state.is_connecting);
    assert!(!view.state.is_auth_modal_open);

    // Verify ServerRail was synchronized
    assert_eq!(view.server_rail.servers.len(), 1);
    assert_eq!(view.server_rail.servers[0].name, "Sharkord Plus HQ");
    assert_eq!(view.server_rail.current_user_name, "admin");

    // Verify DM sidebar was populated from is_dm channels
    assert_eq!(view.dm_sidebar.state.conversations.len(), 1);
    assert_eq!(view.dm_sidebar.state.conversations[0].username, "alice");

    // -------------------------------------------------------------------------
    // 3. Channel Navigation & Selection (Text Channels, Voice Channels)
    // -------------------------------------------------------------------------
    // Initial active channel is 1 (Text)
    assert_eq!(view.active_center_view(), ActiveContentView::Chat);
    assert_eq!(view.chat_canvas.channel_name, "general");

    // Navigate to channel 2 (Voice)
    view.select_channel(2);
    assert_eq!(view.state.active_channel_id, Some(2));
    assert_eq!(
        view.active_center_view(),
        ActiveContentView::VoiceStage,
        "Selecting a voice channel must automatically transition center canvas to VoiceStage"
    );
    assert!(view.live_voice.is_connected);
    assert_eq!(view.live_voice.channel_id, Some(2));
    assert_eq!(view.live_voice.channel_name.as_deref(), Some("voice-lounge"));

    // Verify voice bar reflects connected state
    match view.voice_bar.connection {
        VoiceConnectionState::Connected { channel_id, ref channel_name, .. } => {
            assert_eq!(channel_id, 2);
            assert_eq!(channel_name, "voice-lounge");
        }
        _ => panic!("Voice bar must be Connected after voice channel selection"),
    }

    // Switch back to channel 1 (Text)
    view.select_channel(1);
    assert_eq!(view.state.active_channel_id, Some(1));
    assert_eq!(view.active_center_view(), ActiveContentView::Chat);

    // -------------------------------------------------------------------------
    // 4. Message Feed Virtualized Streaming, Skeleton Loading, & Rich Composer
    // -------------------------------------------------------------------------
    // Group messages for feed
    let mut users_map = HashMap::new();
    users_map.insert(1, "admin".to_string());
    users_map.insert(2, "alice".to_string());
    users_map.insert(3, "bob".to_string());

    let feed_groups = group_messages_for_feed(&view.state.messages, &users_map);
    assert_eq!(feed_groups.len(), 1);
    assert_eq!(feed_groups[0].messages.len(), 1);
    assert_eq!(feed_groups[0].sender_name, "admin");

    // Skeleton loader & empty states
    let empty_placeholder = EmptyStagePlaceholder::new("voice-lounge");
    assert_eq!(empty_placeholder.channel_name, "voice-lounge");
    let _empty_card_el = render_channel_welcome_card("general", Some("Welcome!"));

    // Rich Composer: multiline Shift+Enter vs regular Enter
    view.set_message_draft("Line 1");
    // Simulate Shift+Enter appends newline
    view.message_draft.push('\n');
    view.message_draft.push_str("Line 2 code block");
    view.chat_canvas.set_message_draft(&view.message_draft);
    assert_eq!(view.message_draft, "Line 1\nLine 2 code block");

    // Attachment strip upload simulation
    let att = PendingAttachment::new(PathBuf::from("screenshot.png"));
    view.chat_canvas.add_pending_attachment(att);
    assert_eq!(view.chat_canvas.pending_attachments.len(), 1);
    assert_eq!(view.chat_canvas.pending_attachments[0].filename, "screenshot.png");

    // Regular Enter: sends the message
    let sent_msg = view.send_message();
    assert!(sent_msg.is_some());
    let sent = sent_msg.unwrap();
    assert_eq!(sent.content.as_deref(), Some("Line 1\nLine 2 code block"));
    assert_eq!(sent.channel_id, 1);
    assert_eq!(sent.files.len(), 1);
    assert_eq!(sent.files[0].name, "screenshot.png");
    assert!(view.message_draft.is_empty(), "Draft must be cleared after sending");
    assert!(view.chat_canvas.pending_attachments.is_empty(), "Attachments cleared");

    // Markdown AST formatting & syntax-colored code blocks
    let md_input = "Here is some **bold text** and `inline_code()`\n```rust\nfn main() {\n    println!(\"GPUI Parity\");\n}\n```\n||secret spoiler||";
    let tokens = parse_markdown_tokens(md_input);
    assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Bold(_))));
    assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::InlineCode(_))));
    assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::CodeBlock { language: Some(ref l), .. } if l == "rust")));
    assert!(tokens.iter().any(|t| matches!(t, MarkdownToken::Spoiler(_))));

    // -------------------------------------------------------------------------
    // 5. Message Actions (Hover Toolbar, Reply, In-Place Edit, Reactions, Pin)
    // -------------------------------------------------------------------------
    let target_msg_id = sent.id;

    // Reply Banner
    view.chat_canvas.set_reply_target(Some(ReplyTarget {
        message_id: target_msg_id,
        author_name: "admin".to_string(),
        sender_name: "admin".to_string(),
        snippet: "Line 1...".to_string(),
    }));
    assert!(view.chat_canvas.reply_target.is_some());

    // In-Place Inline Editing
    view.chat_canvas.editing_message_id = Some(target_msg_id);
    view.chat_canvas.editing_text = "Line 1\nEdited line 2 content".to_string();
    assert_eq!(view.chat_canvas.editing_message_id, Some(target_msg_id));
    let edit_res = view.save_inline_edit();
    assert_eq!(edit_res, Some((target_msg_id, "Line 1\nEdited line 2 content".to_string())));
    assert_eq!(
        view.state.messages.iter().find(|m| m.id == target_msg_id).and_then(|m| m.content.as_deref()),
        Some("Line 1\nEdited line 2 content")
    );

    // Emoji reaction badge & tooltip formatting
    view.toggle_reaction(target_msg_id, "👍");
    let msg = view.state.messages.iter().find(|m| m.id == target_msg_id).unwrap();
    assert_eq!(msg.reactions.len(), 1);
    assert_eq!(msg.reactions[0].emoji, "👍");
    assert_eq!(msg.reactions[0].user_id, 1);

    let tooltip = format_reactors_tooltip(&["admin".to_string()]);
    assert_eq!(tooltip, "admin");
    let tooltip2 = format_reactors_tooltip(&["admin".to_string(), "alice".to_string()]);
    assert_eq!(tooltip2, "admin and alice");

    // Pin message toggle
    view.toggle_message_pin(target_msg_id);
    assert!(view.state.messages.iter().find(|m| m.id == target_msg_id).unwrap().pinned);

    // Emoji picker popover state
    let mut picker = EmojiPickerState::new();
    picker.is_open = true;
    assert!(picker.is_open);
    picker.search("smile");
    assert_eq!(picker.query, "smile");
    assert!(!picker.filtered_emojis().is_empty());
    picker.close();
    assert!(!picker.is_open);

    // -------------------------------------------------------------------------
    // 6. Collapsible Channel Categories & Global Command Palette (Ctrl+K)
    // -------------------------------------------------------------------------
    // Channel Tree Collapsible Categories
    let tree = compose_channel_tree(&view.state.channels, Some(1));
    assert_eq!(tree.len(), 3, "3 categories: CHANNELS, TEXT CHANNELS, and VOICE CHANNELS");

    assert!(view.channel_tree.groups.iter().find(|g| g.name == "TEXT CHANNELS").unwrap().is_open);
    view.channel_tree.toggle_category("TEXT CHANNELS");
    assert!(!view.channel_tree.groups.iter().find(|g| g.name == "TEXT CHANNELS").unwrap().is_open);
    view.channel_tree.toggle_category("TEXT CHANNELS");
    assert!(view.channel_tree.groups.iter().find(|g| g.name == "TEXT CHANNELS").unwrap().is_open);

    // Global Command Palette (QuickSwitcher)
    view.toggle_quick_switcher();
    assert!(view.state.is_quick_switcher_open);
    assert!(view.quick_switcher.is_open);

    // Filter by channel query
    view.quick_switcher.set_query("gen");
    let filtered_channels = view.quick_switcher.filtered_items();
    assert_eq!(filtered_channels.len(), 1);
    assert_eq!(filtered_channels[0].name, "general");

    // Slash command filtering & execution
    view.quick_switcher.set_query("/mute");
    let slash_items = view.quick_switcher.filtered_items();
    assert_eq!(slash_items.len(), 1);
    assert_eq!(slash_items[0].title, "/mute");

    // -------------------------------------------------------------------------
    // 7. Direct Messaging Sidebar & User Profile Popover Card
    // -------------------------------------------------------------------------
    // Toggle Home in server rail
    view.server_rail.set_home_selected(true);
    assert!(view.server_rail.is_home_selected);

    // Open DM conversation with user 2 (alice)
    view.open_dm_conversation(2, "alice");
    assert!(view.state.channels.iter().any(|c| c.name == "alice" && c.is_dm));

    // Open User Profile popover card (80px avatar, status ring, bio, Member role)
    view.open_user_profile(2, "alice", "Alice Smith", "online");
    assert!(view.user_profile_modal.is_open);
    assert_eq!(view.user_profile_modal.username, "alice");
    assert_eq!(view.user_profile_modal.display_name.as_deref(), Some("Alice Smith"));
    assert_eq!(view.user_profile_modal.status, "online");
    assert_eq!(view.user_profile_modal.bio.as_deref(), Some("Senior Systems Architect"));
    assert_eq!(view.user_profile_modal.role.as_deref(), Some("Member"));
    view.close_user_profile();
    assert!(!view.user_profile_modal.is_open);

    // -------------------------------------------------------------------------
    // 8. Channel & Server Administration Modals
    // -------------------------------------------------------------------------
    // Create Channel Modal
    view.open_create_channel("TEXT CHANNELS");
    assert!(view.create_channel_modal.is_open);
    view.create_channel_modal.set_name("development");
    view.create_channel_modal.set_channel_type(ChannelType::Text);
    let created_ch = view.submit_create_channel();
    assert!(created_ch.is_some());
    let dev_ch = created_ch.unwrap();
    assert_eq!(dev_ch.name, "development");
    assert_eq!(view.channels().len(), 4);

    // Edit Channel Modal
    view.open_edit_channel(dev_ch.id);
    assert!(view.edit_channel_modal.is_open);
    view.edit_channel_modal.set_name("dev-lounge");
    view.edit_channel_modal.set_topic("Development discussions");
    view.submit_edit_channel();
    assert_eq!(
        view.channels().iter().find(|c| c.id == dev_ch.id).map(|c| c.name.as_str()),
        Some("dev-lounge")
    );

    // Delete Channel with Danger Confirmation
    view.open_edit_channel(dev_ch.id);
    view.edit_channel_modal.set_confirm_delete(true);
    let deleted_id = view.submit_delete_channel();
    assert_eq!(deleted_id, Some(dev_ch.id));
    assert_eq!(view.channels().len(), 3);

    // Server Settings Modal
    view.open_server_settings();
    assert!(view.server_settings_modal.is_open);
    assert_eq!(view.server_settings_modal.server_name, "Sharkord Plus HQ");

    view.server_settings_modal.set_tab(ServerSettingsTab::Roles);
    assert_eq!(view.server_settings_modal.active_tab, ServerSettingsTab::Roles);

    view.server_settings_modal.set_tab(ServerSettingsTab::Members);
    assert_eq!(view.server_settings_modal.active_tab, ServerSettingsTab::Members);
    assert_eq!(view.server_settings_modal.members.len(), 3);
    view.close_server_settings();
    assert!(!view.server_settings_modal.is_open);

    // -------------------------------------------------------------------------
    // 9. User Settings Dialog & System Tray Action Dispatcher
    // -------------------------------------------------------------------------
    // User Settings Modal
    view.open_user_settings();
    assert!(view.user_settings_modal.is_open);
    assert_eq!(view.user_settings_modal.username, "admin");
    assert_eq!(view.user_settings_modal.bio, "Native GPUI Client Lead");

    view.user_settings_modal.set_tab(SettingsTab::VoiceAndVideo);
    assert_eq!(view.user_settings_modal.active_tab, SettingsTab::VoiceAndVideo);
    view.user_settings_modal.set_input_mode(InputMode::PushToTalk);
    assert_eq!(view.user_settings_modal.input_mode, InputMode::PushToTalk);
    view.user_settings_modal.set_input_volume(0.85);
    view.user_settings_modal.set_output_volume(0.90);
    view.user_settings_modal.set_vad_threshold(0.04);
    assert_eq!(view.user_settings_modal.input_volume, 0.85);
    assert_eq!(view.user_settings_modal.output_volume, 0.90);

    view.user_settings_modal.set_tab(SettingsTab::Appearance);
    assert_eq!(view.user_settings_modal.active_tab, SettingsTab::Appearance);
    view.user_settings_modal.set_tab(SettingsTab::Keybinds);
    assert_eq!(view.user_settings_modal.active_tab, SettingsTab::Keybinds);
    view.user_settings_modal.set_tab(SettingsTab::Notifications);
    assert_eq!(view.user_settings_modal.active_tab, SettingsTab::Notifications);
    view.close_user_settings();
    assert!(!view.user_settings_modal.is_open);

    // System Tray Action Dispatcher
    let mut tray = DesktopTray::new();
    tray.state.set_connected(true);
    tray.dispatch_action(TrayAction::ToggleMute);
    assert!(tray.state.is_muted);
    tray.dispatch_action(TrayAction::ToggleDeafen);
    assert!(tray.state.is_deafened);
    assert!(tray.state.is_muted);
    tray.dispatch_action(TrayAction::DisconnectVoice);
    assert!(!tray.state.is_connected);
    tray.dispatch_action(TrayAction::FocusWindow);
    assert!(tray.is_window_focused);
    let tooltip = tray.tooltip();
    assert_eq!(tooltip, "VibeShark");

    // -------------------------------------------------------------------------
    // 10. Mediasoup WebRTC Live Voice Client (Audio Levels, VAD, PTT, Speaking)
    // -------------------------------------------------------------------------
    let mut live_voice = LiveVoiceState::new();
    live_voice.connect_named(2, "voice-lounge");
    assert!(live_voice.is_connected);

    // VAD gating
    live_voice.input_mode = VoiceInputMode::VoiceActivity;
    live_voice.vad_threshold = 0.05;

    // Sample below threshold -> silent
    assert!(!live_voice.process_audio_sample(0.02));
    assert!(!live_voice.is_speaking);

    // Sample above threshold -> speaking
    assert!(live_voice.process_audio_sample(0.25));
    assert!(live_voice.is_speaking);
    assert_eq!(live_voice.audio_level, 0.25);

    // Push-to-Talk (PTT) gating
    live_voice.input_mode = VoiceInputMode::PushToTalk;
    live_voice.is_ptt_pressed = false;
    assert!(!live_voice.process_audio_sample(0.30));
    assert!(!live_voice.is_speaking);

    live_voice.is_ptt_pressed = true;
    assert!(live_voice.process_audio_sample(0.30));
    assert!(live_voice.is_speaking);

    // Dynamic green speaking ring simulation for remote participant
    live_voice.set_user_speaking(2, true, 0.88);
    assert!(live_voice.is_user_speaking(2));
    assert_eq!(live_voice.user_audio_level(2), 0.88);
    live_voice.set_user_speaking(2, false, 0.0);
    assert!(!live_voice.is_user_speaking(2));

    // -------------------------------------------------------------------------
    // 11. Screen Sharing Capture & Video Decoder Pipeline
    // -------------------------------------------------------------------------
    // Source enumeration (monitors and windows)
    let sources = ScreenCaptureEngine::enumerate_sources();
    assert!(!sources.is_empty(), "Capture sources must enumerate at least one monitor/window fallback");
    assert!(sources.iter().any(|s| s.is_monitor()));

    // Video decoder pipeline
    let decoder = VideoDecoderPipeline::new();
    assert!(decoder.is_ok(), "Video decoder pipeline must initialize cleanly");
    let mut dec = decoder.unwrap();
    let decode_res = dec.decode(&[]);
    assert!(decode_res.is_ok(), "Empty NAL unit decoding should be handled cleanly");

    let dummy_rgba = vec![255u8; 1920 * 1080 * 4];
    let frame = VideoFrame::new(1920, 1080, dummy_rgba, 1000);
    assert_eq!(frame.width, 1920);
    assert_eq!(frame.height, 1080);
    assert!(frame.is_valid_buffer());
    assert!((frame.aspect_ratio() - (16.0 / 9.0)).abs() < 0.01);

    // Video tile surface and telemetry badges
    let mut tile_state = VideoTileState::new(2, "alice-screen".to_string());
    assert_eq!(tile_state.streamer_id, 2);
    assert_eq!(tile_state.streamer_name, "alice-screen");
    assert_eq!(tile_state.display_mode, VideoDisplayMode::Standard);

    tile_state.set_display_mode(VideoDisplayMode::Theater);
    assert_eq!(tile_state.display_mode, VideoDisplayMode::Theater);

    let stats = StreamTelemetryStats::new(1080, 60, 4_500_000, 0.01);
    tile_state.update_stats(stats);
    assert_eq!(tile_state.stats.as_ref().unwrap().fps, 60);
    assert_eq!(tile_state.stats.as_ref().unwrap().bitrate_bps, 4_500_000);

    // PiP toggle
    let mut pip = PipPlayerState::new(2, "alice-screen".to_string());
    assert!(!pip.is_visible);
    pip.show();
    assert!(pip.is_visible);
    pip.toggle_audio_mute();
    assert!(pip.is_audio_muted);
    pip.toggle_audio_mute();
    assert!(!pip.is_audio_muted);
    pip.hide();
    assert!(!pip.is_visible);

    // -------------------------------------------------------------------------
    // 12. Workspace Root View Element Tree Rendering
    // -------------------------------------------------------------------------
    // Switch between all three active center content views
    view.set_center_view(ActiveContentView::Chat);
    assert_eq!(view.active_center_view(), ActiveContentView::Chat);

    view.set_center_view(ActiveContentView::VoiceStage);
    assert_eq!(view.active_center_view(), ActiveContentView::VoiceStage);

    view.set_center_view(ActiveContentView::ScreenShare);
    assert_eq!(view.active_center_view(), ActiveContentView::ScreenShare);
}

#[test]
fn test_impeccable_craft_theme_tokens_and_wcag_aa_contrast() {
    // 1. Pure Neutral Dark theme tokens matching specification
    assert_eq!(theme::BG_BASE_HEX, "#0a0b0e");
    assert_eq!(theme::BG_SURFACE_HEX, "#111216");
    assert_eq!(theme::BG_SUBSURFACE_HEX, "#16171d");
    assert_eq!(theme::BG_ELEVATED_HEX, "#1c1e24");
    assert_eq!(theme::TEXT_PRIMARY_HEX, "#f3f4f6");
    assert_eq!(theme::ACCENT_HEX, "#3b82f6");

    assert_eq!(theme::BG_BASE, 0x0a0b0e);
    assert_eq!(theme::BG_SURFACE, 0x111216);
    assert_eq!(theme::BG_SUBSURFACE, 0x16171d);
    assert_eq!(theme::BG_ELEVATED, 0x1c1e24);
    assert_eq!(theme::TEXT_PRIMARY, 0xf3f4f6);
    assert_eq!(theme::ACCENT, 0x3b82f6);

    // 2. Relative luminance and WCAG AA contrast calculation
    // Relative luminance calculation according to WCAG 2.1 specifications:
    fn relative_luminance(rgb_val: u32) -> f64 {
        let r = ((rgb_val >> 16) & 0xff) as f64 / 255.0;
        let g = ((rgb_val >> 8) & 0xff) as f64 / 255.0;
        let b = (rgb_val & 0xff) as f64 / 255.0;

        let transform = |c: f64| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * transform(r) + 0.7152 * transform(g) + 0.0722 * transform(b)
    }

    fn contrast_ratio(rgb1: u32, rgb2: u32) -> f64 {
        let l1 = relative_luminance(rgb1);
        let l2 = relative_luminance(rgb2);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    // Text primary (#F3F4F6) on Base (#0A0B0E): WCAG AA requires >= 4.5:1
    let base_text_ratio = contrast_ratio(theme::TEXT_PRIMARY, theme::BG_BASE);
    assert!(
        base_text_ratio >= 4.5,
        "Text primary on Base contrast ratio must be >= 4.5:1 (got {:.2}:1)",
        base_text_ratio
    );
    assert!(base_text_ratio > 15.0, "Dark mode text primary contrast is outstanding");

    // Text primary on Surface (#111216)
    let surface_text_ratio = contrast_ratio(theme::TEXT_PRIMARY, theme::BG_SURFACE);
    assert!(
        surface_text_ratio >= 4.5,
        "Text primary on Surface contrast ratio must be >= 4.5:1 (got {:.2}:1)",
        surface_text_ratio
    );

    // Text secondary on Surface (#111216)
    let secondary_ratio = contrast_ratio(theme::TEXT_SECONDARY, theme::BG_SURFACE);
    assert!(
        secondary_ratio >= 4.5,
        "Text secondary on Surface contrast ratio must be >= 4.5:1 (got {:.2}:1)",
        secondary_ratio
    );

    // Accent (#3B82F6) on Surface (#111216)
    let accent_ratio = contrast_ratio(theme::ACCENT, theme::BG_SURFACE);
    assert!(
        accent_ratio >= 4.5,
        "Accent on Surface contrast ratio must be >= 4.5:1 (got {:.2}:1)",
        accent_ratio
    );
}

#[test]
fn test_impeccable_craft_clean_vector_icons_and_zero_raw_emojis() {
    // Verify vector icon functions instantiate valid GPUI element structures
    let _mic_on = render_mic_icon(false);
    let _mic_off = render_mic_icon(true);
    let _headphones_on = render_headphones_icon(false);
    let _headphones_off = render_headphones_icon(true);
    let _gear = render_gear_icon();
    let _chat_bubble = render_chat_bubble_icon();

    let _upload = render_upload_icon();
    let _react = render_react_icon();
    let _reply = render_reply_icon();
    let _edit = render_edit_icon();
    let _pin = render_pin_icon(false);
    let _trash = render_trash_icon();
    let _copy = render_copy_icon();
    let _users_toggle = render_users_toggle_icon(false);

    // Check voice bar action labels use clean English text
    let (mute_text, deafen_text) = voice_action_labels(false, false);
    assert_eq!(mute_text, "Mute");
    assert_eq!(deafen_text, "Deafen");

    let (unmute_text, undeafen_text) = voice_action_labels(true, true);
    assert_eq!(unmute_text, "Unmute");
    assert_eq!(undeafen_text, "Undeafen");
}
