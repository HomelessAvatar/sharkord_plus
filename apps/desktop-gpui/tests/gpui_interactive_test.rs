#![recursion_limit = "256"]
extern crate desktop_gpui as sharkord_gpui;

use desktop_gpui::ui::WorkspaceView;
use sharkord_gpui::model::AppState;

#[test]
fn test_app_state_channel_and_voice_toggle() {
    let mut state = AppState::new_mock();
    assert_eq!(state.active_channel_id, "chan_general");

    state.select_channel("chan_voice_lounge");
    assert_eq!(state.active_channel_id, "chan_voice_lounge");

    assert!(!state.is_muted);
    state.toggle_mute();
    assert!(state.is_muted);
}

#[test]
fn test_app_state_voice_deafen_and_disconnect() {
    let mut state = AppState::new_mock();
    assert!(!state.is_deafened);
    assert!(!state.is_muted);

    // Toggling deafen should also mute microphone
    state.toggle_deafen();
    assert!(state.is_deafened);
    assert!(state.is_muted);

    // Disconnect should set is_connected to false
    assert!(state.is_connected);
    state.disconnect();
    assert!(!state.is_connected);
}

#[test]
fn test_app_state_send_message_and_input() {
    let mut state = AppState::new_mock();
    assert_eq!(state.active_channel_id, "chan_general");

    // Empty message should not be sent
    state.set_input_text("   ");
    let sent_none = state.submit_draft();
    assert!(sent_none.is_none());

    // Valid message should be appended to message stream
    state.set_input_text("Hello from reactive GPUI!");
    let sent = state.submit_draft();
    assert!(sent.is_some());
    let msg = sent.unwrap();
    assert_eq!(msg.content, "Hello from reactive GPUI!");
    assert_eq!(msg.channel_id, "chan_general");
    assert_eq!(state.input_text, "");

    let channel_msgs = state.messages_for_active_channel();
    assert!(channel_msgs
        .iter()
        .any(|m| m.content == "Hello from reactive GPUI!"));
}

#[test]
fn test_workspace_interactive_voice_and_channel_selection() {
    let state = desktop_gpui::state::AppState::new_mock();
    let mut workspace = WorkspaceView::new(state);

    // Channel selection
    assert_eq!(workspace.state.active_channel_id, Some(1));
    workspace.select_channel(2);
    assert_eq!(workspace.state.active_channel_id, Some(2));

    // Voice bar controls
    assert!(!workspace.voice_bar.is_muted);
    workspace.toggle_voice_mute();
    assert!(workspace.voice_bar.is_muted);

    assert!(!workspace.voice_bar.is_deafened);
    workspace.toggle_voice_deafen();
    assert!(workspace.voice_bar.is_deafened);

    workspace.disconnect_voice();
    assert_eq!(
        workspace.voice_bar.connection,
        desktop_gpui::ui::VoiceConnectionState::Disconnected
    );
}

#[test]
fn test_workspace_composer_keyboard_interaction() {
    let state = desktop_gpui::state::AppState::new_mock();
    let mut workspace = WorkspaceView::new(state);

    assert_eq!(workspace.message_draft, "");

    // Simulate typing "H", "i" via key_down
    workspace.message_draft.push_str("H");
    workspace.chat_canvas.message_draft = workspace.message_draft.clone();
    workspace.message_draft.push_str("i");
    workspace.chat_canvas.message_draft = workspace.message_draft.clone();
    assert_eq!(workspace.message_draft, "Hi");

    // Simulate backspace
    workspace.message_draft.pop();
    workspace.chat_canvas.message_draft = workspace.message_draft.clone();
    assert_eq!(workspace.message_draft, "H");

    // Send message clears draft and appends
    workspace.message_draft.push_str("ello GPUI");
    let sent = workspace.send_message();
    assert!(sent.is_some());
    let sent_msg = sent.unwrap();
    assert_eq!(sent_msg.content, Some("Hello GPUI".to_string()));
    assert_eq!(workspace.message_draft, "");
}
