//! Coordinates voice session state, mute/deafen status, and event distribution.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Input transmission mode for local audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    VoiceActivity,
    PushToTalk,
}

/// Events emitted during an active voice session.
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceEvent {
    Connected {
        channel_id: u32,
        ping_ms: u32,
    },
    Disconnected,
    SpeakingChanged {
        is_speaking: bool,
        level: f32,
    },
    UserSpeaking {
        user_id: u32,
        is_speaking: bool,
    },
    ParticipantJoined {
        user_id: u32,
        channel_id: u32,
    },
    ParticipantLeft {
        user_id: u32,
        channel_id: u32,
    },
    StateUpdated {
        mic_muted: bool,
        sound_deafened: bool,
    },
}

/// Thread-safe voice session controller.
#[derive(Clone)]
pub struct VoiceSession {
    channel_id: u32,
    mic_muted: Arc<AtomicBool>,
    sound_deafened: Arc<AtomicBool>,
    is_speaking: Arc<AtomicBool>,
    user_volumes: Arc<RwLock<HashMap<u32, f32>>>,
    local_mutes: Arc<RwLock<HashSet<u32>>>,
    input_mode: Arc<RwLock<InputMode>>,
    is_ptt_active: Arc<AtomicBool>,
    event_tx: broadcast::Sender<VoiceEvent>,
}

impl VoiceSession {
    pub fn new(channel_id: u32) -> Self {
        let (event_tx, _) = broadcast::channel(128);
        Self {
            channel_id,
            mic_muted: Arc::new(AtomicBool::new(false)),
            sound_deafened: Arc::new(AtomicBool::new(false)),
            is_speaking: Arc::new(AtomicBool::new(false)),
            user_volumes: Arc::new(RwLock::new(HashMap::new())),
            local_mutes: Arc::new(RwLock::new(HashSet::new())),
            input_mode: Arc::new(RwLock::new(InputMode::VoiceActivity)),
            is_ptt_active: Arc::new(AtomicBool::new(false)),
            event_tx,
        }
    }

    /// Returns the active voice channel ID.
    pub fn channel_id(&self) -> u32 {
        self.channel_id
    }

    /// Returns whether the microphone is currently muted.
    pub fn is_muted(&self) -> bool {
        self.mic_muted.load(Ordering::SeqCst)
    }

    /// Returns whether sound is currently deafened.
    pub fn is_deafened(&self) -> bool {
        self.sound_deafened.load(Ordering::SeqCst)
    }

    /// Returns whether this user is currently actively speaking.
    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::SeqCst)
    }

    /// Toggles microphone mute state and emits `StateUpdated`.
    pub fn toggle_mute(&self) -> bool {
        let new_state = !self.is_muted();
        self.mic_muted.store(new_state, Ordering::SeqCst);
        let _ = self.event_tx.send(VoiceEvent::StateUpdated {
            mic_muted: new_state,
            sound_deafened: self.is_deafened(),
        });
        new_state
    }

    /// Toggles deafen state. When deafened, microphone is automatically muted.
    pub fn toggle_deafen(&self) -> bool {
        let new_deafen = !self.is_deafened();
        self.sound_deafened.store(new_deafen, Ordering::SeqCst);
        if new_deafen {
            self.mic_muted.store(true, Ordering::SeqCst);
        }
        let _ = self.event_tx.send(VoiceEvent::StateUpdated {
            mic_muted: self.is_muted(),
            sound_deafened: new_deafen,
        });
        new_deafen
    }

    /// Updates local speaking state and broadcasts `SpeakingChanged`.
    pub fn set_speaking(&self, is_speaking: bool, level: f32) {
        self.is_speaking.store(is_speaking, Ordering::SeqCst);
        let _ = self
            .event_tx
            .send(VoiceEvent::SpeakingChanged { is_speaking, level });
    }

    /// Returns the configured volume multiplier for a specific user (defaults to 1.0).
    pub fn get_user_volume(&self, user_id: u32) -> f32 {
        self.user_volumes
            .read()
            .unwrap()
            .get(&user_id)
            .copied()
            .unwrap_or(1.0)
    }

    /// Sets the volume multiplier for a specific user (clamped to 0.0..=2.0).
    pub fn set_user_volume(&self, user_id: u32, volume: f32) {
        let clamped = volume.clamp(0.0, 2.0);
        self.user_volumes.write().unwrap().insert(user_id, clamped);
    }

    /// Returns whether a specific user is locally muted.
    pub fn is_local_muted(&self, user_id: u32) -> bool {
        self.local_mutes.read().unwrap().contains(&user_id)
    }

    /// Sets local mute status for a specific user.
    pub fn set_local_mute(&self, user_id: u32, muted: bool) {
        let mut mutes = self.local_mutes.write().unwrap();
        if muted {
            mutes.insert(user_id);
        } else {
            mutes.remove(&user_id);
        }
    }

    /// Computes effective output playback gain for a user.
    /// Returns 0.0 if locally muted or if master sound is deafened, else user's volume.
    pub fn effective_output_gain(&self, user_id: u32) -> f32 {
        if self.is_deafened() || self.is_local_muted(user_id) {
            0.0
        } else {
            self.get_user_volume(user_id)
        }
    }

    /// Returns the current input transmission mode.
    pub fn input_mode(&self) -> InputMode {
        *self.input_mode.read().unwrap()
    }

    /// Sets the input transmission mode.
    pub fn set_input_mode(&self, mode: InputMode) {
        *self.input_mode.write().unwrap() = mode;
    }

    /// Returns whether Push-to-Talk is currently actively transmitting.
    pub fn is_ptt_active(&self) -> bool {
        self.is_ptt_active.load(Ordering::SeqCst)
    }

    /// Sets Push-to-Talk active state.
    pub fn set_ptt_active(&self, active: bool) {
        self.is_ptt_active.store(active, Ordering::SeqCst);
    }

    /// Dispatches an arbitrary voice event (e.g. participant joined or remote user speaking).
    pub fn dispatch_event(&self, event: VoiceEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Subscribes to the broadcast stream of voice events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<VoiceEvent> {
        self.event_tx.subscribe()
    }
}
