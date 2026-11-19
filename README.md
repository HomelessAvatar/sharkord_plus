<div align="center">
  <h1>🦈 Sharkord Plus</h1>
  <p><strong>High-performance, lightweight, self-hosted real-time communication platform</strong></p>
  <p><i>An enhanced community fork of <a href="https://github.com/Sharkord/sharkord">Sharkord</a> with Universal Tauri 2.0 (Desktop & Mobile), Pure Native GPUI, Full Hardware Acceleration, and Selective Window Audio Sharing.</i></p>

  [![CI](https://github.com/HomelessAvatar/sharkord_plus/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/HomelessAvatar/sharkord_plus/actions/workflows/rust-ci.yml)
  [![Version](https://img.shields.io/github/v/release/HomelessAvatar/sharkord_plus?include_prereleases&label=version)](https://github.com/HomelessAvatar/sharkord_plus/releases)
  [![License](https://img.shields.io/github/license/HomelessAvatar/sharkord_plus)](LICENSE)
  [![Upstream](https://img.shields.io/badge/Upstream-Sharkord%20v0.0.31-blue.svg)](https://github.com/Sharkord/sharkord)
  [![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org)
  [![Tauri](https://img.shields.io/badge/Tauri-2.0%20Universal-24C8DB.svg)](https://tauri.app)
  [![GPUI](https://img.shields.io/badge/UI-GPUI%20Native-purple.svg)](https://github.com/zed-industries/zed)
  [![Bun](https://img.shields.io/badge/Bun-v1.4.4-green.svg)](https://bun.sh)
  [![Mediasoup](https://img.shields.io/badge/Mediasoup-v3.19.19-green.svg)](https://mediasoup.org)
</div>

---

## 📌 Attribution & Upstream Relationship

**Sharkord Plus** is built upon the exceptional open-source foundation of [Sharkord](https://github.com/Sharkord/sharkord), created by [diogomartino](https://github.com/diogomartino) and the Sharkord community.

* **Upstream Repository:** [https://github.com/Sharkord/sharkord](https://github.com/Sharkord/sharkord)
* **Official Website & Docs:** [https://sharkord.com](https://sharkord.com)
* **Our Sync Philosophy:** Sharkord Plus maintains **100% backward compatibility** with standard vanilla Sharkord servers. Upstream updates, security patches, and features are selectively incorporated through a structured cherry-pick workflow. See [docs/UPSTREAM_SYNC.md](docs/UPSTREAM_SYNC.md) for sync procedures.

---

## ✨ What Makes Sharkord Plus Different?

While standard Sharkord provides a fantastic Discord-like self-hosted experience, **Sharkord Plus** pushes performance, multi-platform client parity, and audio/video capabilities to production-grade levels:

### 🌐 1. Universal Tauri 2.0 Client (`apps/tauri`) — Full Parity on Desktop & Mobile
- **Instant 100% Parity:** Wraps the battle-tested React web client (`apps/client`), bringing complete feature parity to **Desktop (Windows `.exe`/`.msi`, Linux `.AppImage`/`.deb`, macOS `.dmg`)** and **Mobile (Android `.apk`/`.aab`)** simultaneously.
- **Native OS Superpowers:**
  - **Desktop:** System tray icon (status, mute, deafen, quit), global push-to-talk hotkeys (even when minimized or playing games), and native OS notifications.
  - **Mobile:** Android WebRTC hardware permissions (`RECORD_AUDIO`, `CAMERA`), auto-grant in WebChromeClient, and an Android Voice Foreground Service to keep voice rooms active when the screen is locked.
  - **Encrypted SessionVault:** Seamless auto-login using the shared Rust core (`crates/sharkord-core/src/vault.rs`) across app restarts.
- **Lightweight Footprint:** ~120–180 MB RAM idle, ~15 MB installer size (zero Electron or Chromium distribution bloat).

### ⚡ 2. Pure Native Rust + GPUI Desktop Client (`apps/desktop-gpui`)
- **GPU-Accelerated 120+ FPS Rendering:** Built with Zed's GPU-accelerated [GPUI](https://github.com/zed-industries/zed) framework (`#0A0A0A` Pure Neutral Dark theme following [DESIGN.md](DESIGN.md)).
- **Sub-50MB RAM Footprint:** Ultra-low resource consumption for competitive esports and memory-constrained environments.
- **Interactive State Model:** Reactive `AppState` coordinating channel navigation, voice controls, and text input composition.

### 🎥 3. 4K & 120/144 FPS Hardware Video Streaming & Presets
- **Esports Frame Rates:** 4K 60 FPS, 1440p 120 FPS, 1080p 144 FPS, and Custom bitrate/framerate screen-sharing presets.
- **Zero-CPU Video Streaming:** Full hardware encoding support across all major codecs (H.264, AV1, VP8/VP9, HEVC) via D3D11/D3D12/MediaFoundation (Windows), VideoToolbox (macOS), and VA-API (Linux).

### 🔊 4. Selective Per-Window & Application Audio Loopback
- **Targeted Audio Sharing:** Stream audio exclusively from a specific game or application window without broadcasting background music, private voice calls, or browser sounds.
- Powered by high-speed low-latency WASAPI loopback (`@vencord/venmic`) with process ID filtering.

---

## 🛠️ Project Architecture

Sharkord Plus is structured as a unified monorepo leveraging Bun workspaces for TypeScript and Cargo workspaces for Rust:

```
sharkord_plus/
├── apps/
│   ├── client/           # Web client (React 18 + Vite + Redux Toolkit + Tailwind CSS)
│   ├── server/           # Backend (Bun + tRPC + Drizzle SQLite + Mediasoup SFU)
│   ├── tauri/            # Universal Tauri 2.0 client (Windows, macOS, Linux, Android)
│   │   └── src-tauri/    # Rust backend (TDesktopBridge, SessionVault, tray, shortcuts)
│   └── desktop-gpui/     # Pure Native Rust + GPUI Desktop client (Parallel track)
├── crates/
│   ├── sharkord-core/    # Core Rust SDK (SessionVault, video presets, tRPC client, types)
│   └── sharkord-mobile/  # Native mobile bindings
├── packages/
│   ├── shared/           # Types, enums, permissions shared across client, server, native
│   ├── ui/               # Presentational UI components (Radix UI + Tailwind)
│   └── plugin-sdk/       # Public API surface for plugin developers
└── docs/                 # Architecture, design guidelines, and implementation specs
```

---

## 🚀 Getting Started

### 1. Running the Universal Tauri 2.0 App

```bash
# Install dependencies
bun install

# Run Desktop Client in development (runs Vite and launches Tauri window)
bun run tauri:dev

# Build Desktop Release Binaries (Windows .msi/.exe, Linux .AppImage, macOS .dmg)
bun run tauri:build

# Run / Build for Android (requires Android Studio / Android SDK)
bun run tauri:android:dev
bun run tauri:android:build
```

### 2. Running the Native GPUI Desktop App (Rust)

```bash
cargo run -p desktop-gpui
```

### 3. Running the Server (Docker)

```bash
docker run -d \
  -p 4991:4991/tcp \
  -p 40000-40050:40000-40050/udp \
  -p 40000-40050:40000-40050/tcp \
  -v ./data:/home/bun/.config/sharkord \
  --name sharkord-plus \
  ghcr.io/homelessavatar/sharkord_plus:0.2.6
```

### 4. Local Web & Server Development

Requirements: [Bun 1.4+](https://bun.sh) and [Rust 1.85+](https://www.rust-lang.org).

```bash
# Start local server and client
bun dev

# Run all checks (ESLint, Prettier, TypeScript type check)
bun run magic

# Run Rust unit and protocol tests
cargo test --all
```

---

## 🏷️ Versioning & Compatibility

Sharkord Plus follows [Semantic Versioning (SemVer 2.0.0)](https://semver.org/):

* **Current Version:** `v0.2.6`
* **Upstream Compatibility Matrix:**
  | Sharkord Plus Version | Compatible Sharkord Upstream Server |
  |-----------------------|-------------------------------------|
  | `v0.1.x` - `v0.2.x`   | `v0.0.31`+ (and backward compatible to `0.0.28`) |

---

## 📄 License & Credits

- Licensed under the **MIT License** — see [LICENSE](LICENSE) for details.
- Original Sharkord code & architecture Copyright © [diogomartino](https://github.com/diogomartino).
- Extended and maintained by [HomelessAvatar](https://github.com/HomelessAvatar) and contributors.
