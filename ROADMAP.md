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

#### Step 1.4: Desktop UX Polish & Discord Parity Views ✅ *(Completed)*
- Dynamic Center View Switcher (`CenterStageView`): Segmented tab controls ("Chat" vs "Voice Stage") in the channel header for seamless switching between text messaging and voice stage grid when connected to voice.
- Right-Hand Member List Sidebar Drawer (`MemberList`): 240px collapsible roster grouping channel members by status (Online, Idle, DND, Offline) with role tags, avatar indicators, and member count headers.
- Discord-Grade Empty States & Welcome Cards (`EmptyStates`): Channel welcome header cards (hashtag badge, channel title, topic description, creation date, action shortcuts) and animated voice stage waiting placeholders ("It's quiet in here...").
- Raycast-Style Quick Switcher Modal (`QuickSwitcherModal`): `Ctrl+K` / `Cmd+K` global quick switcher overlay with search input, keyboard up/down navigation, quick channel/DM switching, and visual channel type badges.

---

### Phase 2: Desktop Packaging, Installers & Auto-Updater 📦 ✅ *(Completed)*
- **Cross-Platform Packaging Engine (`apps/desktop-gpui/packager.toml`):**
  - **Windows Packaging:** Automated `.msi` Windows installer and portable standalone `.zip` distributions.
  - **macOS Packaging:** Universal `.dmg` drag-and-drop installer and `.app` bundle with high-resolution Apple icon (`icon.icns`).
  - **Linux Packaging:** Standalone `.AppImage` and Debian `.deb` packages with desktop launcher integration and category metadata.
  - **Desktop Branding Assets:** High-resolution icons generated across all native formats (`.ico`, `.icns`, `.png`).
- **Core Update Engine (`crates/sharkord-core`):**
  - Strict SemVer parsing with pre-release tag filtering and update target selection.
  - GitHub Releases API integration matching OS architectures (`x86_64`, `aarch64`) and file extensions (`.msi`, `.zip`, `.dmg`, `.AppImage`, `.deb`).
  - Asynchronous background download engine with real-time byte progress tracking and system browser fallback.
- **In-App Desktop Updater Modal (`apps/desktop-gpui`):**
  - Discord-grade update modal dialog styled with Sharkord's Pure Neutral Dark palette (`#111216`, `#16171d`, `#1c1e24`, `#6366f1`).
  - Real-time download progress bar with percentage indicator, release notes markdown viewer, and "Update Now" / "Relaunch" flow.
  - Non-intrusive status bar update notification badge with glowing amber accent.
- **Automated CI/CD Release Pipeline (`.github/workflows/desktop-release.yml`):**
  - Matrix release workflow compiling and packaging native artifacts on Windows (`windows-latest`), macOS (`macos-latest`), and Linux (`ubuntu-latest`).
  - Automated draft GitHub Releases generation triggered on semantic version tags (`v*.*.*`).

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
