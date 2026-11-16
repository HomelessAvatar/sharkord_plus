# Native WebRTC Voice & Audio Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate a pure native Rust WebRTC voice and audio engine into `crates/sharkord-core` and the GPUI desktop client (`apps/desktop-gpui`), enabling real-time voice channel participation, microphone/speaker capture, speaking indicators, and audio settings without any Electron or browser dependency.

**Architecture:** 
- `crates/sharkord-core`: Implements Mediasoup voice signaling (`joinVoice`, `connectTransport`, `produce`, `consume`), device discovery and audio buffer streaming via `cpal`, Voice Activity Detection (VAD) with noise threshold, and session state management.
- `apps/desktop-gpui`: Hooks voice session events into GPUI's async runtime via `cx.spawn()`, rendering real-time voice indicators, docked voice controls (Mute, Deafen, Disconnect, Latency), active speaker rings in the channel roster, and audio device preferences in the settings view.

**Tech Stack:** Rust 1.85+, Tokio, `cpal` (cross-platform audio), Serde, GPUI, `#![forbid(unsafe_code)]` where possible.

---

## Global Constraints

- **Design Tokens:** Strict adherence to `DESIGN.md` Pure Neutral Dark palette (`#0a0b0e` canvas, `#111216` panels, `#16171d` surfaces, `#22c55e` speaking green).
- **Asynchronous Execution:** All audio thread and network communication must run on dedicated worker threads / Tokio tasks, communicating with GPUI views via unbounded channels or `cx.spawn()`.
- **Backward Compatibility:** All tRPC signaling must interoperate flawlessly with both vanilla Sharkord and Sharkord Plus Mediasoup SFU servers.
- **Verification:** Every task must include unit/integration tests running with `cargo test --all` and zero compile warnings.

---

### Task 1: Mediasoup Voice Signaling & tRPC Client Methods

**Files:**
- Create: `crates/sharkord-core/src/voice/mod.rs`
- Create: `crates/sharkord-core/src/voice/types.rs`
- Create: `crates/sharkord-core/src/voice/signaling.rs`
- Modify: `crates/sharkord-core/src/client.rs`
- Modify: `crates/sharkord-core/src/lib.rs`
- Test: `crates/sharkord-core/tests/voice_signaling_test.rs`

**Interfaces:**
- `RtpCapabilities`, `TransportOptions`, `ProducerOptions`, `ConsumerOptions` in `types.rs`.
- `SharkordClient::join_voice(channel_id: u64) -> Result<VoiceJoinResponse>`
- `SharkordClient::connect_voice_transport(...) -> Result<()>`
- `SharkordClient::produce_voice_track(...) -> Result<ProducerId>`
- `SharkordClient::leave_voice(channel_id: u64) -> Result<()>`

- [ ] **Step 1: Write the failing test**
  In `crates/sharkord-core/tests/voice_signaling_test.rs`, mock the tRPC `voice.joinVoice` and `voice.connectTransport` endpoints with `wiremock` and assert correct payload serialization and response parsing.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test voice_signaling_test`
  Expected: FAIL (modules not found)

- [ ] **Step 3: Implement signaling types and client methods**
  - Define Mediasoup WebRTC types in `crates/sharkord-core/src/voice/types.rs`.
  - Implement tRPC voice procedures in `signaling.rs` and attach to `SharkordClient`.
  - Re-export `voice` module in `crates/sharkord-core/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test voice_signaling_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add crates/sharkord-core/
  git commit -m "feat(core): implement mediasoup voice signaling and trpc procedures"
  ```

---

### Task 2: Cross-Platform Audio Device Discovery & Voice Activity Detection (VAD)

**Files:**
- Create: `crates/sharkord-core/src/voice/audio.rs`
- Create: `crates/sharkord-core/src/voice/vad.rs`
- Modify: `crates/sharkord-core/Cargo.toml`
- Test: `crates/sharkord-core/tests/audio_vad_test.rs`

**Interfaces:**
- `AudioDevice`: name, ID, is_default, kind (Input/Output).
- `AudioDeviceManager::list_input_devices() -> Vec<AudioDevice>`
- `AudioDeviceManager::list_output_devices() -> Vec<AudioDevice>`
- `VadEngine::process_samples(pcm: &[f32]) -> (bool, f32)` (returns `(is_speaking, rms_level)` with hysteresis).

- [ ] **Step 1: Add `cpal` to `crates/sharkord-core/Cargo.toml`**
  Add `cpal = "0.15"` to dependencies.

- [ ] **Step 2: Write the failing test**
  In `crates/sharkord-core/tests/audio_vad_test.rs`, test RMS calculation, silence vs speaking detection, and threshold transitions with attack/release hangover logic.

- [ ] **Step 3: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test audio_vad_test`
  Expected: FAIL

- [ ] **Step 4: Implement `audio.rs` and `vad.rs`**
  - Implement `AudioDeviceManager` querying `cpal::default_host()`.
  - Implement `VadEngine` with configurable threshold (dB), attack time, and release hangover duration to avoid speech clipping.

- [ ] **Step 5: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test audio_vad_test`
  Expected: PASS

- [ ] **Step 6: Commit**
  ```bash
  git add crates/sharkord-core/
  git commit -m "feat(core): add cross-platform audio device manager and vad engine"
  ```

---

### Task 3: Voice Session State Machine & Event Dispatcher

**Files:**
- Create: `crates/sharkord-core/src/voice/session.rs`
- Modify: `crates/sharkord-core/src/voice/mod.rs`
- Test: `crates/sharkord-core/tests/voice_session_test.rs`

**Interfaces:**
- `VoiceSession`: Coordinates signaling, audio capture stream, VAD state, and local mute/deafen status.
- `VoiceEvent`:
  - `Connected { channel_id: u64, ping_ms: u32 }`
  - `Disconnected`
  - `SpeakingChanged { is_speaking: bool, level: f32 }`
  - `UserSpeaking { user_id: u64, is_speaking: bool }`
  - `ParticipantJoined { user_id: u64, channel_id: u64 }`
  - `ParticipantLeft { user_id: u64, channel_id: u64 }`

- [ ] **Step 1: Write the failing test**
  In `crates/sharkord-core/tests/voice_session_test.rs`, simulate connection lifecycle, mute/unmute toggles, and event channel dispatching.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test voice_session_test`
  Expected: FAIL

- [ ] **Step 3: Implement `VoiceSession` in `crates/sharkord-core/src/voice/session.rs`**
  Implement the session struct with thread-safe atomic flags for mute/deafen and unbounded broadcast channels for events.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test voice_session_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add crates/sharkord-core/
  git commit -m "feat(core): implement voice session coordinator and event dispatcher"
  ```

---

### Task 4: GPUI Voice State Integration & Interactive Voice Bar

**Files:**
- Modify: `apps/desktop-gpui/src/state.rs`
- Modify: `apps/desktop-gpui/src/ui/voice_bar.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/voice_bar_test.rs`

**Interfaces:**
- `AppState::active_voice_session: Option<VoiceBarState>`
- `AppState::toggle_mute()`, `AppState::toggle_deafen()`, `AppState::disconnect_voice()`
- `VoiceBar::render()` binds click events to app state mutations and displays live connection stats.

- [ ] **Step 1: Write the failing test**
  Update `apps/desktop-gpui/tests/voice_bar_test.rs` to verify mute, deafen, and disconnect actions update state cleanly and generate correct UI tokens.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p desktop-gpui --test voice_bar_test`
  Expected: FAIL

- [ ] **Step 3: Update `state.rs` and `voice_bar.rs`**
  - Connect `VoiceBarState` to `AppState` with toggle handlers.
  - Implement interactive click callbacks with visual hover/active feedback adhering to `DESIGN.md`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p desktop-gpui --test voice_bar_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add apps/desktop-gpui/
  git commit -m "feat(desktop): wire interactive voice bar with mute, deafen, and disconnect controls"
  ```

---

### Task 5: Voice Channel Tree Roster & Green Speaking Indicator Ring

**Files:**
- Modify: `apps/desktop-gpui/src/ui/channel_tree.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/speaking_indicator_test.rs`

**Interfaces:**
- `AppState::is_user_speaking(user_id: u64) -> bool`
- `AppState::voice_channel_participants(channel_id: u64) -> Vec<VoiceParticipant>`
- `ChannelTree::render_voice_channel()` expands connected users with avatars and an animated green border (`#22c55e`) when speaking.

- [ ] **Step 1: Write the failing test**
  In `apps/desktop-gpui/tests/speaking_indicator_test.rs`, assert that active speakers return true and speaking ring tokens are correctly assigned.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p desktop-gpui --test speaking_indicator_test`
  Expected: FAIL

- [ ] **Step 3: Implement channel roster and speaking ring in `channel_tree.rs`**
  - Render indented participant rows under active voice channels.
  - Wrap user avatars in a dynamic speaking ring border based on `is_speaking`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p desktop-gpui --test speaking_indicator_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add apps/desktop-gpui/
  git commit -m "feat(desktop): add voice channel participant list with speaking indicator rings"
  ```

---

### Task 6: Audio Settings View (Device Selection & Mic VU Meter)

**Files:**
- Create: `apps/desktop-gpui/src/ui/audio_settings.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/audio_settings_test.rs`

**Interfaces:**
- `AudioSettingsState`: selected input device, selected output device, input volume, input mode (VoiceActivity vs PushToTalk), noise gate threshold.
- `AudioSettingsModal`: GPUI component displaying dropdowns for devices and live RMS VU meter.

- [ ] **Step 1: Write the failing test**
  In `apps/desktop-gpui/tests/audio_settings_test.rs`, test setting device preferences and audio configuration state persistence.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p desktop-gpui --test audio_settings_test`
  Expected: FAIL

- [ ] **Step 3: Implement `audio_settings.rs`**
  - Build UI layout with device selectors, volume sliders, and input sensitivity meter.
  - Follow Pure Neutral Dark theme rules from `DESIGN.md`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p desktop-gpui --test audio_settings_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add apps/desktop-gpui/
  git commit -m "feat(desktop): create audio settings view with device picker and vu meter"
  ```

---

### Task 7: Full Test Suite & Verification Pass

**Files:**
- All modified files
- Documentation updates

- [ ] **Step 1: Run complete Rust test suite**
  Run: `cargo test --all`
  Assert: All unit, protocol, UI, and voice tests pass with 0 failures.

- [ ] **Step 2: Run Rust linter and formatting**
  Run: `cargo fmt --check`
  Run: `cargo clippy --all-targets -- -D warnings`

- [ ] **Step 3: Run TypeScript/Bun workspace checks**
  Run: `bun run magic`
  Assert: Prettier, ESLint, and TypeScript check-types pass cleanly.

- [ ] **Step 4: Impeccable Design & Brainstorming Audit**
  Review UI tokens, contrast ratios, and layout flow against `impeccable` craft floor to verify zero design regressions.

- [ ] **Step 5: Final Commit & Push**
  ```bash
  git push origin development
  ```
