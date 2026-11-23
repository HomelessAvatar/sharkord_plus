//! Live Mediasoup WebRTC voice client and audio state coordinator.
//! Bridges audio capture/playback via `cpal`, VAD power measurement via `sharkord_core::voice`,
//! and WebRTC RTP transport connectivity.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use serde::{Deserialize, Serialize};
use sharkord_core::error::SharkordError;
use sharkord_core::voice::{AudioDevice, AudioDeviceManager, VadConfig, VadEngine, VoiceSession};
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;

/// Input transmission mode for local microphone audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VoiceInputMode {
    #[default]
    VoiceActivity,
    PushToTalk,
}

/// Dynamic live voice connection state, local VAD gating, and participant speaking levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveVoiceState {
    pub is_connected: bool,
    pub channel_id: Option<u64>,
    pub channel_name: Option<String>,
    pub is_muted: bool,
    pub is_deafened: bool,
    pub is_speaking: bool,
    pub audio_level: f32,
    pub ping_ms: u32,
    pub vad_threshold: f32,
    pub input_mode: VoiceInputMode,
    pub is_ptt_pressed: bool,
    pub speaking_users: HashMap<u64, f32>,
}

impl Default for LiveVoiceState {
    fn default() -> Self {
        Self {
            is_connected: false,
            channel_id: None,
            channel_name: None,
            is_muted: false,
            is_deafened: false,
            is_speaking: false,
            audio_level: 0.0,
            ping_ms: 0,
            vad_threshold: 0.05,
            input_mode: VoiceInputMode::VoiceActivity,
            is_ptt_pressed: false,
            speaking_users: HashMap::new(),
        }
    }
}

impl LiveVoiceState {
    /// Creates a new `LiveVoiceState` with default disconnected state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Connects to a voice channel by ID.
    pub fn connect(&mut self, channel_id: u64) {
        self.is_connected = true;
        self.channel_id = Some(channel_id);
    }

    /// Connects to a named voice channel.
    pub fn connect_named(&mut self, channel_id: u64, name: impl Into<String>) {
        self.is_connected = true;
        self.channel_id = Some(channel_id);
        self.channel_name = Some(name.into());
    }

    /// Disconnects from the current voice channel and resets speaking levels.
    pub fn disconnect(&mut self) {
        self.is_connected = false;
        self.channel_id = None;
        self.channel_name = None;
        self.is_speaking = false;
        self.audio_level = 0.0;
        self.speaking_users.clear();
    }

    /// Toggles microphone mute. When muted, local speaking state is immediately cleared.
    pub fn toggle_mute(&mut self) -> bool {
        self.is_muted = !self.is_muted;
        if self.is_muted {
            self.is_speaking = false;
            self.audio_level = 0.0;
        }
        self.is_muted
    }

    /// Sets explicit microphone mute state.
    pub fn set_muted(&mut self, muted: bool) {
        self.is_muted = muted;
        if self.is_muted {
            self.is_speaking = false;
            self.audio_level = 0.0;
        }
    }

    /// Toggles sound deafen. Deafen automatically mutes the microphone.
    pub fn toggle_deafen(&mut self) -> bool {
        self.is_deafened = !self.is_deafened;
        if self.is_deafened {
            self.is_muted = true;
            self.is_speaking = false;
            self.audio_level = 0.0;
        }
        self.is_deafened
    }

    /// Sets explicit deafen state.
    pub fn set_deafened(&mut self, deafened: bool) {
        self.is_deafened = deafened;
        if self.is_deafened {
            self.is_muted = true;
            self.is_speaking = false;
            self.audio_level = 0.0;
        }
    }

    /// Sets the local speaking indicator state and power level.
    pub fn set_speaking(&mut self, speaking: bool, level: f32) {
        self.is_speaking = speaking;
        self.audio_level = level;
    }

    /// Sets the local audio power level.
    pub fn set_audio_level(&mut self, level: f32) {
        self.audio_level = level;
    }

    /// Updates the round-trip latency in milliseconds.
    pub fn set_ping(&mut self, ping_ms: u32) {
        self.ping_ms = ping_ms;
    }

    /// Switches the audio input mode between VoiceActivity and PushToTalk.
    pub fn set_input_mode(&mut self, mode: VoiceInputMode) {
        self.input_mode = mode;
        if mode == VoiceInputMode::PushToTalk && !self.is_ptt_pressed {
            self.is_speaking = false;
        }
    }

    /// Configures the linear VAD threshold.
    pub fn set_vad_threshold(&mut self, threshold: f32) {
        self.vad_threshold = threshold;
    }

    /// Updates push-to-talk button press state.
    pub fn set_ptt_pressed(&mut self, pressed: bool) {
        self.is_ptt_pressed = pressed;
        if !pressed && self.input_mode == VoiceInputMode::PushToTalk {
            self.is_speaking = false;
        }
    }

    /// Sets a remote participant's speaking status and level.
    pub fn set_user_speaking(&mut self, user_id: u64, speaking: bool, level: f32) {
        if speaking && level > 0.0 {
            self.speaking_users.insert(user_id, level);
        } else {
            self.speaking_users.remove(&user_id);
        }
    }

    /// Checks if a participant is actively speaking.
    pub fn is_user_speaking(&self, user_id: u64) -> bool {
        self.speaking_users.get(&user_id).copied().unwrap_or(0.0) > 0.0
    }

    /// Returns a participant's current audio level.
    pub fn user_audio_level(&self, user_id: u64) -> f32 {
        self.speaking_users.get(&user_id).copied().unwrap_or(0.0)
    }

    /// Removes a participant from the active speaking map.
    pub fn remove_user(&mut self, user_id: u64) {
        self.speaking_users.remove(&user_id);
    }

    /// Clears all participants from the speaking map.
    pub fn clear_participants(&mut self) {
        self.speaking_users.clear();
    }

    /// Returns a list of user IDs currently actively speaking.
    pub fn active_speakers(&self) -> Vec<u64> {
        self.speaking_users
            .iter()
            .filter(|(_, &lvl)| lvl > 0.0)
            .map(|(&uid, _)| uid)
            .collect()
    }

    /// Evaluates an audio sample level against mute/deafen status and active input mode.
    /// Returns true if voice should be transmitted.
    pub fn process_audio_sample(&mut self, sample: f32) -> bool {
        if self.is_muted || self.is_deafened {
            self.is_speaking = false;
            self.audio_level = 0.0;
            return false;
        }

        let level = sample.abs();
        match self.input_mode {
            VoiceInputMode::VoiceActivity => {
                self.audio_level = level;
                if level >= self.vad_threshold {
                    self.is_speaking = true;
                    true
                } else {
                    self.is_speaking = false;
                    false
                }
            }
            VoiceInputMode::PushToTalk => {
                if !self.is_ptt_pressed {
                    self.is_speaking = false;
                    self.audio_level = 0.0;
                    false
                } else {
                    self.audio_level = level;
                    self.is_speaking = true;
                    true
                }
            }
        }
    }

    /// Computes RMS power across an audio PCM sample buffer and evaluates gating.
    pub fn process_audio_samples(&mut self, samples: &[f32]) -> bool {
        if samples.is_empty() {
            return self.process_audio_sample(0.0);
        }
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();
        self.process_audio_sample(rms)
    }
}

/// Live WebRTC voice engine client for Mediasoup voice channels.
pub struct LiveVoiceClient {
    state: Arc<RwLock<LiveVoiceState>>,
    vad_engine: Arc<Mutex<VadEngine>>,
    audio_manager: AudioDeviceManager,
    session: Option<VoiceSession>,
}

impl Default for LiveVoiceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveVoiceClient {
    /// Creates a new `LiveVoiceClient` initialized with audio manager and default VAD engine.
    pub fn new() -> Self {
        let vad_config = VadConfig {
            threshold_db: -35.0,
            attack_frames: 2,
            hangover_frames: 10,
        };
        Self {
            state: Arc::new(RwLock::new(LiveVoiceState::new())),
            vad_engine: Arc::new(Mutex::new(VadEngine::new(vad_config))),
            audio_manager: AudioDeviceManager::new(),
            session: None,
        }
    }

    /// Returns the thread-safe reference to the live voice state.
    pub fn state(&self) -> Arc<RwLock<LiveVoiceState>> {
        Arc::clone(&self.state)
    }

    /// Returns a snapshot copy of the current `LiveVoiceState`.
    pub fn current_state(&self) -> LiveVoiceState {
        self.state.read().unwrap().clone()
    }

    /// Connects to a voice channel by ID and channel name.
    pub fn connect(&mut self, channel_id: u64, channel_name: impl Into<String>) {
        let name = channel_name.into();
        {
            let mut state = self.state.write().unwrap();
            state.connect_named(channel_id, name);
        }
        self.session = Some(VoiceSession::new(channel_id as u32));
    }

    /// Disconnects from the active voice channel and tears down transports.
    pub fn disconnect(&mut self) {
        {
            let mut state = self.state.write().unwrap();
            state.disconnect();
        }
        self.session = None;
    }

    /// Toggles microphone mute state and updates the underlying voice session.
    pub fn toggle_mute(&mut self) -> bool {
        let is_muted = {
            let mut state = self.state.write().unwrap();
            state.toggle_mute()
        };
        if let Some(ref session) = self.session {
            if session.is_muted() != is_muted {
                session.toggle_mute();
            }
        }
        is_muted
    }

    /// Toggles deafen state.
    pub fn toggle_deafen(&mut self) -> bool {
        let is_deafened = {
            let mut state = self.state.write().unwrap();
            state.toggle_deafen()
        };
        if let Some(ref session) = self.session {
            if session.is_deafened() != is_deafened {
                session.toggle_deafen();
            }
        }
        is_deafened
    }

    /// Updates push-to-talk key pressed state.
    pub fn set_ptt_pressed(&mut self, pressed: bool) {
        self.state.write().unwrap().set_ptt_pressed(pressed);
    }

    /// Configures voice transmission mode (`VoiceActivity` or `PushToTalk`).
    pub fn set_input_mode(&mut self, mode: VoiceInputMode) {
        self.state.write().unwrap().set_input_mode(mode);
    }

    /// Sets VAD threshold.
    pub fn set_vad_threshold(&mut self, threshold: f32) {
        self.state.write().unwrap().set_vad_threshold(threshold);
    }

    /// Sets round-trip latency (ping in milliseconds).
    pub fn update_ping(&mut self, ping_ms: u32) {
        self.state.write().unwrap().set_ping(ping_ms);
    }

    /// Records a remote participant's speaking status and level.
    pub fn handle_user_speaking(&mut self, user_id: u64, speaking: bool, level: f32) {
        self.state
            .write()
            .unwrap()
            .set_user_speaking(user_id, speaking, level);
    }

    /// Processes captured audio PCM frames, evaluating VAD/PTT and updating local state.
    pub fn process_input_audio(&mut self, pcm: &[f32]) -> (bool, f32) {
        let (speaking, rms) = {
            let mut vad = self.vad_engine.lock().unwrap();
            vad.process_frames(pcm)
        };

        let mut state = self.state.write().unwrap();
        let gated_speaking = match state.input_mode {
            VoiceInputMode::VoiceActivity => {
                if state.is_muted || state.is_deafened {
                    false
                } else {
                    speaking || rms >= state.vad_threshold
                }
            }
            VoiceInputMode::PushToTalk => {
                if state.is_muted || state.is_deafened {
                    false
                } else {
                    state.is_ptt_pressed
                }
            }
        };

        let level = if gated_speaking { rms } else { 0.0 };
        state.set_speaking(gated_speaking, level);
        (gated_speaking, level)
    }

    /// Lists host input audio recording devices.
    pub fn list_input_devices(&self) -> Result<Vec<AudioDevice>, SharkordError> {
        self.audio_manager.list_input_devices()
    }

    /// Lists host output playback devices.
    pub fn list_output_devices(&self) -> Result<Vec<AudioDevice>, SharkordError> {
        self.audio_manager.list_output_devices()
    }

    /// Creates a WebRTC API instance configured for Mediasoup RTP transports.
    pub fn create_webrtc_api() -> webrtc::api::API {
        let mut media_engine = MediaEngine::default();
        let _ = media_engine.register_default_codecs();
        APIBuilder::new().with_media_engine(media_engine).build()
    }
}
