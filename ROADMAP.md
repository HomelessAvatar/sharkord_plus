# Sharkord Plus Strategic Roadmap

This document outlines the official development roadmap for **Sharkord Plus**.

---

## 🚀 Active Development Phases

### Phase 1: Native WebRTC Voice, Video & Screen Share in Rust 🎙️ *(Completed)*

#### Step 1.1: Core Audio Engine & Docked Voice Controls ✅ *(Completed)*
- **Core Voice Architecture (`crates/sharkord-core`):**
  - Mediasoup SFU voice client in Rust (tRPC signaling: `joinVoice`, `createTransport`, `connectTransport`, `produce`, `consume`).
  - Cross-platform audio device management via `cpal` (WASAPI, PipeWire, ALSA, CoreAudio).
  - VAD (Voice Activity Detection) engine with hysteresis threshold and live audio level RMS meter.
  - Thread-safe `VoiceSession` state coordinator and event broadcasting.
- **GPUI Desktop Voice Interface (`apps/desktop-gpui`):**
  - Docked voice bar with Connect / Disconnect controls and connection latency stats.
  - Microphone mute / deafen toggles and reactive state integration.
  - Real-time green speaking indicator ring for active speakers and channel roster integration.
  - Native Audio Settings view with device selection, volume sliders, and live VU meter bar.

#### Step 1.2: Voice Quality, Advanced Controls & Voice Grid View ✅ *(Completed)*
- Per-user volume gain routing (0% - 200%) and local mute ("Mute for me").
- Noise Gate DSP filter with smooth attack and release curves.
- Push-to-Talk (PTT) input mode and keybind transmission gating.
- Discord-style interactive Voice Stage / Grid View with responsive cards and speaking animations.
- Participant context menu with live volume adjustment and mute toggles.

#### Step 1.3: Screen Sharing, GPU Video Render & PiP Mode ✅ *(Completed)*
- Screen / Window source selection modal with resolution (720p/1080p/1440p) & framerate (15/30/60 FPS) toggles.
- GPU-accelerated video texture rendering in GPUI (`VideoTileView`) with letterboxed 16:9 viewport and live stream telemetry popover.
- Isolated per-window audio loopback capture.
- Picture-in-Picture (`PipPlayerView`) floating mini player docked in bottom-right corner during text channel navigation.

---

### Phase 2: Desktop Packaging, Installers & Auto-Updater 📦
- **Windows Packaging:**
  - Automated `.msi` Windows installer and portable `.zip` releases.
- **Linux Packaging:**
  - Standalone `.AppImage` and `.deb` packages with desktop launcher integration.
- **Background Auto-Updater:**
  - In-app release detector querying GitHub Releases API with seamless one-click background upgrade.

---

### Phase 3: Cross-Platform Mobile Client (Android & iOS) 📱
- **Shared Rust Core (`crates/sharkord-core`):**
  - Expose core protocol and audio engine via C ABI / UniFFI bindings.
- **Modern Touch-First Native Interface:**
  - Fast, responsive mobile interface (Flutter / React Native / Kotlin-Swift).
  - Background voice service, mobile push notifications, and lock-screen media controls.
  - Swipe-to-navigate drawers and camera/microphone permission handling.

---

### Phase 4: Rich Media & Web UI/UX Polish ✨
- **Enhanced Markdown Rendering:**
  - Syntax-highlighted code blocks with line numbering and one-click copy.
- **Rich Media & Link Previews (OpenGraph / OEmbed):**
  - Discord-style rich embed cards for YouTube, Twitter/X, GitHub, and external links.
- **Inline Audio & Video Players:**
  - Native inline playback for uploaded voice notes and video clips.

---

### Phase 5: Automated Upstream Synchronization Watcher 🔄
- Scheduled GitHub Action monitoring `upstream/development` (`https://github.com/Sharkord/sharkord`).
- Automated notification and draft cherry-pick pull requests for newly released security patches and core features.
