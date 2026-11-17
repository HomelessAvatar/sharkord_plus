# Screen Sharing, GPU Video Rendering & PiP Mode (Step 1.3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver Step 1.3 of the native voice & video roadmap: screen and window capture source selection, GPU-accelerated video rendering via GPUI canvas texture, isolated window audio loopback, Picture-in-Picture (PiP) mini floating player, and real-time streaming telemetry stats.

**Architecture:** Extend `crates/sharkord-core` with video capture sources (`ScreenSource`, `VideoTrackConfig`, `StreamTelemetryStats`), video consumer frame ingestion, and per-window audio loopback. In `apps/desktop-gpui`, implement the `SourcePickerModal`, the hardware-accelerated `VideoTileView` rendering 60 FPS video frames onto GPU textures, floating `PipPlayerView`, and stream telemetry popover.

**Tech Stack:** Rust 2021, GPUI Canvas, `cpal`, `serde`, Pure Neutral Dark design tokens (`#0a0b0e`, `#111216`, `#16171d`, `#3b82f6`, `#22c55e`).

**Spec:** `PRODUCT.md`, `DESIGN.md`, and Roadmap Step 1.3 specification.

## Global Constraints
- Zero Turkish text in code comments, test names, identifiers, commit messages, and documentation files. Direct communication with the user is in Turkish.
- `#![forbid(unsafe_code)]` remains strictly enforced in `crates/sharkord-core`.
- Maintain 100% test pass rate across the workspace with zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
- Pure Neutral Dark palette: background base `#0a0b0e`, surface `#111216`, elevated `#16171d`, accent `#3b82f6`, online `#22c55e`.

## Review Focus
- Screen source enum serialization and validation (monitors vs application windows).
- Video frame dimension and aspect ratio preservation (16:9 letterboxing).
- PiP player state transitions when switching channels (main stage vs floating mini player).
- Stream telemetry calculations (bitrate, fps, packet loss percentages) without division-by-zero panics.
- Audio loopback synchronization with video track frames.

---

### Task 1: Screen Source Enumeration & Video Track Pipeline in sharkord-core

**Files:**
- Create: `crates/sharkord-core/src/voice/video.rs`
- Modify: `crates/sharkord-core/src/voice/mod.rs`
- Test: `crates/sharkord-core/tests/video_pipeline_test.rs`

**Interfaces:**
- Consumes: `VoiceSession`, `ProducerOptions`
- Produces: `ScreenSource`, `VideoResolution`, `VideoFrameRate`, `VideoTrackConfig`, `StreamTelemetryStats`, `VideoFrame`

- [ ] **Step 1: Write failing unit test in `crates/sharkord-core/tests/video_pipeline_test.rs`**

```rust
use sharkord_core::voice::video::{
    ScreenSource, VideoFrame, VideoFrameRate, VideoResolution, VideoTrackConfig,
    StreamTelemetryStats,
};

#[test]
fn test_video_track_config_and_sources() {
    let monitor = ScreenSource::Monitor { id: 1, name: "Display 1 (2560x1440)".to_string() };
    let window = ScreenSource::Window { id: 4520, title: "Game - 60 FPS".to_string() };

    let config = VideoTrackConfig {
        source: monitor,
        resolution: VideoResolution::R1080p,
        framerate: VideoFrameRate::Fps60,
        with_audio: true,
    };

    assert_eq!(config.resolution.width(), 1920);
    assert_eq!(config.resolution.height(), 1080);
    assert_eq!(config.framerate.fps(), 60);
    assert!(config.with_audio);

    // Telemetry calculation
    let stats = StreamTelemetryStats::new(1080, 60, 4_500_000, 0.005);
    assert_eq!(stats.bitrate_mbps(), 4.5);
    assert_eq!(stats.packet_loss_percent(), 0.5);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test video_pipeline_test`
Expected: FAIL with unresolved module `video`

- [ ] **Step 3: Implement `crates/sharkord-core/src/voice/video.rs`**

Define `ScreenSource`, `VideoResolution` (720p, 1080p, 1440p), `VideoFrameRate` (15, 30, 60 FPS), `VideoTrackConfig`, `StreamTelemetryStats`, and `VideoFrame`. Export in `crates/sharkord-core/src/voice/mod.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p sharkord-core --test video_pipeline_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add crates/sharkord-core
git commit -m "feat(core): add screen source selection, video track configuration, and telemetry models"
```

---

### Task 2: Screen & Window Source Picker Modal in GPUI

**Files:**
- Create: `apps/desktop-gpui/src/ui/source_picker.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/source_picker_test.rs`

**Interfaces:**
- Consumes: `ScreenSource`, `VideoTrackConfig`, `VideoResolution`, `VideoFrameRate`
- Produces: `SourcePickerState`, `SourcePickerView`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/source_picker_test.rs`**

```rust
use desktop_gpui::ui::source_picker::{SourcePickerState, SourcePickerTab};
use sharkord_core::voice::video::{ScreenSource, VideoFrameRate, VideoResolution};

#[test]
fn test_source_picker_state_and_tabs() {
    let mut picker = SourcePickerState::new();

    assert_eq!(picker.current_tab, SourcePickerTab::Screens);
    assert_eq!(picker.selected_resolution, VideoResolution::R1080p);
    assert_eq!(picker.selected_framerate, VideoFrameRate::Fps60);
    assert!(picker.capture_window_audio);

    picker.select_tab(SourcePickerTab::Windows);
    assert_eq!(picker.current_tab, SourcePickerTab::Windows);

    picker.select_source(ScreenSource::Window { id: 101, title: "VS Code".to_string() });
    assert!(picker.selected_source.is_some());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test source_picker_test`
Expected: FAIL with unresolved module `source_picker`

- [ ] **Step 3: Implement `apps/desktop-gpui/src/ui/source_picker.rs`**

Implement source picker modal view with tabs (Screens vs Windows), thumbnail preview tiles, quality & FPS selector pills, and "Share Audio" checkbox. Export in `apps/desktop-gpui/src/ui/mod.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test source_picker_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement screen and application window source picker modal in gpui"
```

---

### Task 3: GPU-Accelerated VideoTile Player & Streaming Telemetry Popover

**Files:**
- Create: `apps/desktop-gpui/src/ui/video_tile.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Test: `apps/desktop-gpui/tests/video_tile_test.rs`

**Interfaces:**
- Consumes: `StreamTelemetryStats`, `VideoFrame`
- Produces: `VideoTileState`, `VideoTileView`, `aspect_ratio_box`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/video_tile_test.rs`**

```rust
use desktop_gpui::ui::video_tile::{VideoTileState, VideoDisplayMode};
use sharkord_core::voice::video::StreamTelemetryStats;

#[test]
fn test_video_tile_state_and_telemetry() {
    let mut tile = VideoTileState::new(301, "Dave's Screen".to_string());

    assert_eq!(tile.streamer_id, 301);
    assert_eq!(tile.display_mode, VideoDisplayMode::Standard);
    assert!(!tile.is_fullscreen);

    tile.toggle_fullscreen();
    assert!(tile.is_fullscreen);

    let stats = StreamTelemetryStats::new(1080, 60, 5_000_000, 0.001);
    tile.update_stats(stats);
    assert_eq!(tile.stats.as_ref().unwrap().fps, 60);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test video_tile_test`
Expected: FAIL with unresolved module `video_tile`

- [ ] **Step 3: Implement `apps/desktop-gpui/src/ui/video_tile.rs`**

Implement video player canvas container with letterboxed 16:9 aspect ratio, overlay controls (fullscreen, mute stream audio, disconnect), and live telemetry stats badge (bitrate / FPS / packet loss). Export in `apps/desktop-gpui/src/ui/mod.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test video_tile_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): implement gpu video tile player and stream telemetry overlay"
```

---

### Task 4: Picture-in-Picture (PiP) Floating Mini Player

**Files:**
- Create: `apps/desktop-gpui/src/ui/pip_player.rs`
- Modify: `apps/desktop-gpui/src/ui/mod.rs`
- Modify: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/pip_player_test.rs`

**Interfaces:**
- Consumes: `VideoTileState`, `AppState`
- Produces: `PipPlayerState`, `PipPlayerView`

- [ ] **Step 1: Write failing unit test in `apps/desktop-gpui/tests/pip_player_test.rs`**

```rust
use desktop_gpui::ui::pip_player::PipPlayerState;

#[test]
fn test_pip_player_state_transitions() {
    let mut pip = PipPlayerState::new(301, "Dave's Stream".to_string());

    assert_eq!(pip.streamer_id, 301);
    assert!(!pip.is_visible);

    pip.show();
    assert!(pip.is_visible);

    // Dimension checks (240x135 fixed 16:9)
    assert_eq!(pip.width, 240.0);
    assert_eq!(pip.height, 135.0);

    pip.hide();
    assert!(!pip.is_visible);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test pip_player_test`
Expected: FAIL with unresolved module `pip_player`

- [ ] **Step 3: Implement `apps/desktop-gpui/src/ui/pip_player.rs` and wire into `AppState`**

Implement docked floating mini video player fixed in the bottom-right corner when active video stream exists and user navigates away from the voice channel to text channels.

- [ ] **Step 4: Run test to verify it passes**

Run: `& "$HOME\.cargo\bin\cargo.exe" test -p desktop-gpui --test pip_player_test`
Expected: PASS

- [ ] **Step 5: Format and commit**

```bash
& "$HOME\.cargo\bin\cargo.exe" fmt
git add apps/desktop-gpui
git commit -m "feat(desktop): add picture-in-picture floating mini video player"
```

---

### Task 5: End-to-End Test Suite, Verification & Roadmap Update

**Files:**
- Modify: `ROADMAP.md`

- [ ] **Step 1: Run complete cargo test suite across all crates**
- [ ] **Step 2: Run formatting and clippy checks**
- [ ] **Step 3: Run monorepo type checks and linting (`bun run magic`)**
- [ ] **Step 4: Update roadmap and commit**
