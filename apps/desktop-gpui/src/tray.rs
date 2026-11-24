//! System tray menu state and action dispatcher for Sharkord desktop client.

use serde::{Deserialize, Serialize};

/// System tray menu actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayAction {
    ToggleMute,
    ToggleDeafen,
    DisconnectVoice,
    CheckForUpdates,
    FocusWindow,
    Quit,
}

impl TrayAction {
    /// Human-readable label for the menu action item.
    pub fn label(&self) -> &'static str {
        match self {
            TrayAction::ToggleMute => "Toggle Mute",
            TrayAction::ToggleDeafen => "Toggle Deafen",
            TrayAction::DisconnectVoice => "Disconnect Voice",
            TrayAction::CheckForUpdates => "Check for Updates...",
            TrayAction::FocusWindow => "Open VibeShark",
            TrayAction::Quit => "Quit VibeShark",
        }
    }
}

/// Representation of an individual item in the tray context menu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrayMenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub checked: Option<bool>,
    pub action: TrayAction,
}

/// State tracking the system tray menu properties and status flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrayMenuState {
    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_connected: bool,
    pub unread_count: usize,
}

impl Default for TrayMenuState {
    fn default() -> Self {
        Self {
            is_muted: false,
            is_deafened: false,
            is_connected: false,
            unread_count: 0,
        }
    }
}

impl TrayMenuState {
    /// Creates a new default `TrayMenuState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles the microphone mute state.
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }

    /// Toggles the audio deafen state (deafening also mutes).
    pub fn toggle_deafen(&mut self) {
        self.is_deafened = !self.is_deafened;
        if self.is_deafened {
            self.is_muted = true;
        }
    }

    /// Sets the voice connection status flag.
    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
    }

    /// Sets the current unread messages count.
    pub fn set_unread_count(&mut self, count: usize) {
        self.unread_count = count;
    }

    /// Builds the list of tray context menu items based on current state.
    pub fn menu_items(&self) -> Vec<TrayMenuItem> {
        vec![
            TrayMenuItem {
                id: "focus_window".to_string(),
                label: "Open VibeShark".to_string(),
                enabled: true,
                checked: None,
                action: TrayAction::FocusWindow,
            },
            TrayMenuItem {
                id: "toggle_mute".to_string(),
                label: if self.is_muted {
                    "Unmute Microphone".to_string()
                } else {
                    "Mute Microphone".to_string()
                },
                enabled: true,
                checked: Some(self.is_muted),
                action: TrayAction::ToggleMute,
            },
            TrayMenuItem {
                id: "toggle_deafen".to_string(),
                label: if self.is_deafened {
                    "Undeafen Audio".to_string()
                } else {
                    "Deafen Audio".to_string()
                },
                enabled: true,
                checked: Some(self.is_deafened),
                action: TrayAction::ToggleDeafen,
            },
            TrayMenuItem {
                id: "disconnect_voice".to_string(),
                label: "Disconnect Voice".to_string(),
                enabled: self.is_connected,
                checked: None,
                action: TrayAction::DisconnectVoice,
            },
            TrayMenuItem {
                id: "check_updates".to_string(),
                label: "Check for Updates...".to_string(),
                enabled: true,
                checked: None,
                action: TrayAction::CheckForUpdates,
            },
            TrayMenuItem {
                id: "quit".to_string(),
                label: "Quit VibeShark".to_string(),
                enabled: true,
                checked: None,
                action: TrayAction::Quit,
            },
        ]
    }
}

/// System tray action dispatcher and manager.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopTray {
    pub state: TrayMenuState,
    pub is_window_focused: bool,
    pub has_quit: bool,
}

impl DesktopTray {
    /// Creates a new `DesktopTray` instance.
    pub fn new() -> Self {
        Self {
            state: TrayMenuState::new(),
            is_window_focused: false,
            has_quit: false,
        }
    }

    /// Dispatches a tray menu action.
    pub fn dispatch_action(&mut self, action: TrayAction) {
        match action {
            TrayAction::ToggleMute => self.state.toggle_mute(),
            TrayAction::ToggleDeafen => self.state.toggle_deafen(),
            TrayAction::DisconnectVoice => self.state.set_connected(false),
            TrayAction::CheckForUpdates => {}
            TrayAction::FocusWindow => self.is_window_focused = true,
            TrayAction::Quit => self.has_quit = true,
        }
    }

    /// Generates tooltip text for the system tray icon.
    pub fn tooltip(&self) -> String {
        if self.state.unread_count > 0 {
            format!("VibeShark ({} unread)", self.state.unread_count)
        } else if self.state.is_connected {
            "VibeShark (Voice Connected)".to_string()
        } else {
            "VibeShark".to_string()
        }
    }
}
