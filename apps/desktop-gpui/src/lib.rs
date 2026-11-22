pub mod model;
pub mod network;
pub mod state;
pub mod theme;
pub mod ui;

use gpui_kit as _;
use openh264 as _;
use rfd as _;
use webrtc as _;
use xcap as _;

pub use model::AppState as ReactiveAppState;
pub use network::{BootstrapData, NetworkBridge};
pub use state::AppState;
pub use ui::WorkspaceView;
