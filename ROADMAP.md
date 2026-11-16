# Sharkord Plus Strategic Roadmap

This document outlines the official development roadmap for **Sharkord Plus**.

---

## 🚀 Active Development Phases

### Phase 1: Native WebRTC Voice & Audio Engine in Rust 🎙️ *(In Progress)*
- **Core Voice Architecture (`crates/sharkord-core`):**
  - Implement Mediasoup SFU client in Rust (tRPC voice signaling for `joinVoice`, `connectTransport`, `produce`, `consume`).
  - Low-latency cross-platform audio capture & playback via `cpal` (WASAPI on Windows, PipeWire/ALSA on Linux, CoreAudio on macOS).
  - High-fidelity Opus encoder & decoder pipeline with jitter buffer and packet loss concealment.
- **GPUI Desktop Voice Interface (`apps/desktop-gpui`):**
  - Interactive voice channel bar with Connect / Disconnect controls and connection latency stats.
  - Microphone mute / deafen toggles, Push-to-Talk (PTT) with global hotkey support.
  - Real-time green speaking indicator ring for active speakers.
  - Audio input/output device selector and input sensitivity noise gate in Settings.

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
