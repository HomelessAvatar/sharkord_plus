# Voice Quality, Advanced Controls & Voice Grid View (Step 1.2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver Step 1.2 of the native voice roadmap: per-user volume control, local muting, noise gate DSP filtering, Push-to-Talk (PTT) input mode, and a Discord-style Voice Grid/Stage view in the GPUI desktop client.

**Architecture:** Extend `crates/sharkord-core` with per-user audio gain routing, local mute management, noise gate DSP frame processor, and PTT state handling. In `apps/desktop-gpui`, implement the interactive Voice Stage / Grid View (`voice_stage.rs`) displaying active participants in a dynamic responsive grid with pulsing speaking borders, along with participant volume adjustment and local mute context controls.

**Tech Stack:** Rust 2021, GPUI, `cpal`, `serde`, Pure Neutral Dark design tokens (`#0a0b0e`, `#111216`, `#16171d`, `#22c55e`).

**Spec:** `PRODUCT.md`, `DESIGN.md`, and Roadmap Step 1.2 specification.

## Global Constraints
- Zero Turkish text in code comments, test names, identifiers, commit messages, and documentation files. Direct communication with the user is in Turkish.
- `#![forbid(unsafe_code)]` remains strictly enforced in `crates/sharkord-core`.
- Maintain 100% test pass rate across the workspace with zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
- Pure Neutral Dark palette: background base `#0a0b0e`, surface `#111216`, elevated `#16171d`, speaking accent `#22c55e`.

## Review Focus
- Per-user volume clamping (0.0..=2.0) ensuring gain never causes overflow or panics.
- Local mute completely suppresses audio output for that specific user regardless of their speaking state.
- Noise gate attenuation smoothly silences background audio without abrupt audio clipping or popping.
- Push-to-Talk cleanly gates transmission and respects release delay without dropping words.
- Responsive Voice Grid handles 1 to 25+ participants gracefully without overflowing the UI canvas.

---

### Task 1: Per-User Volume Routing, Local Mute & PTT in VoiceSession

**Files:**
- Modify: `crates/sharkord-core/src/voice/session.rs`
- Test: `crates/sharkord-core/tests/voice_session_controls_test.rs`

**Interfaces:**
- Consumes: `VoiceSession`, `VoiceEvent`
- Produces: `InputMode`, `set_user_volume`, `get_user_volume`, `set_local_mute`, `is_local_muted`, `effective_output_gain`, `set_input_mode`, `set_ptt_active`

- [ ] **Step 1: Write failing unit test in `crates/sharkord-core/tests/voice_session_controls_test.rs`**

```rust
use sharkord_core::voice::session::{InputMode, VoiceSession};

#[test]
fn test_per_user_volume_and_local_mute() {
    let session = VoiceSession::new();

    // Default volume is 1.0 (100%)
    assert_eq!(session.get_user_volume(42), 1.0);
    assert!(!session.is_local_muted(42));
    assert_eq!(session.effective_output_gain(42), 1.0);

    // Adjust volume (e.g. 150%)
    session.set_user_volume(42, 1.5);
    assert_eq!(session.get_user_volume(42), 1.5);
    assert_eq!(session.effective_output_gain(42), 1.5);

    // Local mute overrides volume to 0.0
    session.set_local_mute(42, true);
    assert!(session.is_local_muted(42));
    assert_eq!(session.effective_output_gain(42), 0.0);

    // Unmute restores volume
    session.set_local_mute(42, false);
    assert_eq!(session.effective_output_gain(42), 1.5);

    // Clamping: max 2.0, min 0.0
    session.set_user_volume(42, 3.5);
    assert_eq!(session.get_user_volume(42), 2.0);
    session.set_user_volume(42, -1.0);
    assert_eq!(session.get_user_volume(42), 0.0);
}

#[test]
fn test_input_mode_and_ptt_state() {
    let session = VoiceSession::new();

    // Default input mode is VoiceActivity
    assert_eq!(session.input_mode(), InputMode::VoiceActivity);

    // Switch to Push-to-Talk
    session.set_input_mode(InputMode::PushToTalk);
    assert_eq!(session.input_mode(), InputMode::PushToTalk);

    // In PTT mode, transmission is false until PTT key is active
    assert!(!session.is_ptt_active());
    session.set_ptt_active(true);
    assert!(session.is_ptt_active());
    session.set_ptt_active(false);
    assert!(!session.is_ptt_active());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test voice_session_controls_test`
Expected: FAIL with missing types/methods

- [ ] **Step 3: Implement per-user volume, local mute, and PTT in `crates/sharkord-core/src/voice/session.rs`**

Add `InputMode` enum (`VoiceActivity`, `PushToTalk`), `user_volumes: std::sync::RwLock<HashMap<u32, f32>>`, `local_mutes: std::sync::RwLock<HashSet<u32>>`, `input_mode: std::sync::RwLock<InputMode>`, `is_ptt_active: AtomicBool`. Implement getter and setter methods with clamping and effective output gain calculation.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test voice_session_controls_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add crates/sharkord-core
git commit -m "feat(voice): implement per-user volume routing, local mute, and ptt input mode"
```

---

### Task 2: Noise Gate DSP Audio Filter

**Files:**
- Create: `crates/sharkord-core/src/voice/noise_gate.rs`
- Modify: `crates/sharkord-core/src/voice/mod.rs`
- Test: `crates/sharkord-core/tests/noise_gate_test.rs`

**Interfaces:**
- Consumes: PCM audio slice `&mut [f32]`
- Produces: `NoiseGate`, `NoiseGateConfig`, `process_frame`

- [ ] **Step 1: Write failing unit test in `crates/sharkord-core/tests/noise_gate_test.rs`**

```rust
use sharkord_core::voice::noise_gate::{NoiseGate, NoiseGateConfig};

#[test]
fn test_noise_gate_attenuation() {
    let config = NoiseGateConfig {
        open_threshold: 0.05,
        close_threshold: 0.02,
        attack_ms: 10.0,
        release_ms: 50.0,
        sample_rate: 48000,
    };
    let mut gate = NoiseGate::new(config);

    // 1. Quiet noise (below close_threshold, e.g. amplitude 0.01)
    let mut noise = vec![0.01f32; 480];
    gate.process_frame(&mut noise);
    // Attenuation should silence or strongly reduce the noise
    let energy_after: f32 = noise.iter().map(|s| s.abs()).sum::<f32>() / 480.0;
    assert!(energy_after < 0.005, "Low noise should be attenuated by gate");

    // 2. Loud speech (above open_threshold, e.g. amplitude 0.2)
    let mut speech = vec![0.2f32; 480];
    gate.process_frame(&mut speech);
    assert!(gate.is_open(), "Loud audio should open gate");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test noise_gate_test`
Expected: FAIL with unresolved module `noise_gate`

- [ ] **Step 3: Implement `NoiseGate` in `crates/sharkord-core/src/voice/noise_gate.rs`**

Calculate frame RMS. When above `open_threshold`, smoothly ramp gain up to 1.0 (attack). When below `close_threshold`, smoothly decay gain toward 0.0 (release/attenuation). Apply gain curve in-place across PCM sample frames. Export module in `crates/sharkord-core/src/voice/mod.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test noise_gate_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add crates/sharkord-core
git commit -m "feat(voice): add noise gate dsp audio filter with smooth attack and release"
```

---

### Task 3: Interactive Voice Grid / Stage View in GPUI

**Files:**
- Create: `apps/desktop-gpui/src/ui/voice_stage.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/voice_stage_test.rs`

**Interfaces:**
- Consumes: `VoiceParticipantDisplay`, `AppState`
- Produces: `VoiceStageView`, `VoiceStageCard`, `calculate_grid_columns`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/voice_stage_test.rs`**

```rust
use desktop_gpui::ui::voice_stage::{calculate_grid_columns, VoiceStageCard, VoiceStageView};
use desktop_gpui::theme;

#[test]
fn test_voice_stage_grid_layout_and_cards() {
    // Column count based on participant count
    assert_eq!(calculate_grid_columns(1), 1);
    assert_eq!(calculate_grid_columns(2), 2);
    assert_eq!(calculate_grid_columns(4), 2);
    assert_eq!(calculate_grid_columns(6), 3);
    assert_eq!(calculate_grid_columns(12), 4);

    let card = VoiceStageCard {
        user_id: 101,
        username: "Alex".to_string(),
        avatar_url: None,
        is_speaking: true,
        is_muted: false,
        is_deafened: false,
        volume: 1.0,
    };

    assert!(card.is_speaking);
    assert_eq!(card.border_color(), theme::STATUS_ONLINE);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test voice_stage_test`
Expected: FAIL with unresolved module `voice_stage`

- [ ] **Step 3: Implement `VoiceStageView` in `apps/desktop-gpui/src/ui/voice_stage.rs`**

Implement Discord-style participant grid view with:
- Dynamic column calculation (`calculate_grid_columns`).
- Participant cards with large avatar/initials, glowing green ring border when `is_speaking`, mute/deafen status badges, and username header.
- Empty stage illustration/placeholder when no other participants are connected.
- Export in `apps/desktop-gpui/src/ui/mod.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test voice_stage_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement discord-style voice grid stage view in gpui"
```

---

### Task 4: Participant Context Controls (Per-User Volume Slider & Local Mute)

**Files:**
- Create: `apps/desktop-gpui/src/ui/voice_context_menu.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/voice_context_test.rs`

**Interfaces:**
- Consumes: `AppState`, `VoiceParticipantDisplay`
- Produces: `VoiceParticipantMenuState`, `set_user_volume`, `toggle_user_local_mute`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/voice_context_test.rs`**

```rust
use desktop_gpui::ui::voice_context_menu::VoiceParticipantMenuState;

#[test]
fn test_participant_context_menu_state() {
    let mut menu = VoiceParticipantMenuState::new(202, "Charlie".to_string());

    assert_eq!(menu.user_id, 202);
    assert_eq!(menu.volume, 1.0);
    assert!(!menu.is_local_muted);

    menu.set_volume(1.25);
    assert_eq!(menu.volume, 1.25);
    assert_eq!(menu.volume_percentage(), 125);

    menu.toggle_local_mute();
    assert!(menu.is_local_muted);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test voice_context_test`
Expected: FAIL with unresolved module `voice_context_menu`

- [ ] **Step 3: Implement `VoiceParticipantMenuState` and UI in `apps/desktop-gpui/src/ui/voice_context_menu.rs`**

Implement volume adjustment controls, volume percentage badge, and "Mute for me" toggle button. Integrate user volume map into `AppState`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test voice_context_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): add per-user volume slider and local mute context controls"
```

---

### Task 5: End-to-End Test Suite & Verification

**Files:**
- Modify: `ROADMAP.md`

- [ ] **Step 1: Run complete cargo test suite**

Run: `& "$HOME\.cargo\bin\cargo.exe" test --all`
Expected: PASS with 0 failures across all crates

- [ ] **Step 2: Run formatting and clippy checks**

Run: `& "$HOME\.cargo\bin\cargo.exe" fmt --check`
Run: `& "$HOME\.cargo\bin\cargo.exe" clippy --all-targets -- -D warnings`
Expected: PASS with 0 warnings

- [ ] **Step 3: Run monorepo type checks and linting**

Run: `& "$HOME\.bun\bin\bun.exe" run magic`
Expected: PASS with exit code 0

- [ ] **Step 4: Update roadmap and commit**

Update `ROADMAP.md` reflecting Step 1.2 completion.
Commit: `docs(roadmap): mark Step 1.2 voice controls and stage view complete`
