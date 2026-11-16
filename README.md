<div align="center">
  <h1>🦈 Sharkord Plus</h1>
  <p><strong>High-performance, lightweight, self-hosted real-time communication platform</strong></p>
  <p><i>An enhanced community fork of <a href="https://github.com/Sharkord/sharkord">Sharkord</a> with Pure Native Rust + GPUI Desktop, Full Hardware Acceleration, and Selective Window Audio Sharing.</i></p>

  [![CI](https://github.com/HomelessAvatar/sharkord_plus/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/HomelessAvatar/sharkord_plus/actions/workflows/rust-ci.yml)
  [![Version](https://img.shields.io/github/v/release/HomelessAvatar/sharkord_plus?include_prereleases&label=version)](https://github.com/HomelessAvatar/sharkord_plus/releases)
  [![License](https://img.shields.io/github/license/HomelessAvatar/sharkord_plus)](LICENSE)
  [![Upstream](https://img.shields.io/badge/Upstream-Sharkord%20v0.0.31-blue.svg)](https://github.com/Sharkord/sharkord)
  [![Rust](https://img.shields.io/badge/Rust-1.85+-orange.svg)](https://www.rust-lang.org)
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

While standard Sharkord provides a fantastic Discord-like self-hosted experience, **Sharkord Plus** pushes the performance, desktop integration, and audio/video pipeline to production-grade levels:

### ⚡ 1. Pure Native Rust Core & GPUI Desktop Client (`apps/desktop-gpui`)
- **GPU-Accelerated 120 FPS Rendering:** Built with Zed's GPU-accelerated [GPUI](https://github.com/zed-industries/zed) framework (`#0a0b0e` Pure Neutral Dark theme following [DESIGN.md](DESIGN.md)).
- **Instant Launch & Minimal Footprint:** Launches in milliseconds and consumes $\sim$35MB RAM (an 85%+ memory reduction compared to standard Electron apps).
- **Universal Protocol Client (`crates/sharkord-core`):** Standalone, `#![forbid(unsafe_code)]` Rust SDK featuring batched tRPC encoding, automatic capability negotiation (`vanilla` vs `plus`), and WebSocket realtime event subscriptions.

### 🎥 2. Universal Hardware Video Encoding & Simulcast (Issue #942 Fix)
- **Zero-CPU Video Streaming:** Full hardware encoding support across all major codecs:
  - **H.264:** High and Main profiles with D3D11 / D3D12 / MediaFoundation (Windows), VideoToolbox (macOS), and VA-API (Linux).
  - **AV1 & VP8/VP9:** Hardware acceleration flags and dynamic WebRTC simulcast codec resolution.
  - **HEVC / H.265:** Future-ready codec definitions for 4K 60fps low-latency screen casting.
- **Hardware Encoder Telemetry:** Real-time WebRTC stats popup displaying active hardware encoder engines (`MediaFoundation`, `D3D12`, `DXVA`).

### 🔊 3. Selective Per-Window & Application Audio Loopback (Issue #925 Fix)
- **Targeted Audio Sharing:** Stream sound exclusively from a specific game or application window without broadcasting background music, Discord notifications, or private browser audio.
- Powered by high-speed low-latency WASAPI loopback (`@vencord/venmic`) with process ID filtering.

### 📱 4. Responsive UI & Mobile Drawer Experience
- **Impeccable Touch Design:** Slide-over navigation drawer for channel lists and member sidebars on mobile and compact viewports.
- **Zero UI Clipping:** Responsive headers with hamburger toggles, touch-optimized user cards, and full-screen overlay fixes.

### 🚀 5. Automated Multi-Platform CI/CD
- **Native Cross-Platform Builds:** Automated GitHub Actions workflows testing and compiling release binaries for **Windows (`.exe`)**, **Linux (`x86_64`)**, and **macOS (`arm64`)**.

---

## 📸 Screenshots

*(Desktop GPUI client & Web UI running Pure Neutral Dark theme)*

---

## 🛠️ Project Architecture

Sharkord Plus is structured as a unified monorepo leveraging Bun workspaces for TypeScript and Cargo workspaces for Rust:

```
sharkord_plus/
├── apps/
│   ├── client/           # Web client (React + Vite + Redux Toolkit + Tailwind)
│   ├── server/           # Backend (Bun + tRPC + Drizzle SQLite + Mediasoup SFU)
│   └── desktop-gpui/     # Pure Native Rust + GPUI Desktop application (No Electron)
├── crates/
│   └── sharkord-core/    # Core Rust SDK (Models, tRPC client, WebSockets, Capabilities)
├── packages/
│   ├── shared/           # Types, enums, permissions shared across client, server, native
│   ├── ui/               # Presentational UI components (Radix UI + Tailwind)
│   └── plugin-sdk/       # Public API surface for plugin developers
├── docs/                 # Architecture, design guidelines, and upstream sync docs
└── patches/              # Compatibility patches for cross-platform dependencies
```

---

## 🚀 Getting Started

### 🔄 Migrating from Vanilla Sharkord?

Already running a standard vanilla Sharkord server on your VPS? You can upgrade to **Sharkord Plus** with **zero data loss**, preserving all users, messages, and uploaded files.

👉 **Follow the step-by-step migration guide: [docs/MIGRATION_GUIDE.md](docs/MIGRATION_GUIDE.md)**

---

### 1. Running the Server (Docker)

```bash
docker run -d \
  -p 4991:4991/tcp \
  -p 40000-40050:40000-40050/udp \
  -p 40000-40050:40000-40050/tcp \
  -v ./data:/home/bun/.config/sharkord \
  --name sharkord-plus \
  ghcr.io/homelessavatar/sharkord_plus:0.1.1
```

> [!WARNING]
> Upon first launch, the server prints an owner secret token to stdout. Keep this token safe; it grants full administrative ownership and signs session cookies.

### 2. Running the Native GPUI Desktop App (Rust)

Download the pre-compiled binary for your operating system from [Releases](https://github.com/HomelessAvatar/sharkord_plus/releases), or build from source:

```bash
# Clone the repository
git clone https://github.com/HomelessAvatar/sharkord_plus.git
cd sharkord_plus

# Build and run the native GPUI desktop app
cargo run -p desktop-gpui
```

### 3. Local Web & Server Development

Requirements: [Bun 1.4+](https://bun.sh) and [Rust 1.85+](https://www.rust-lang.org).

```bash
# Install dependencies
bun install

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

* **Format:** `vMAJOR.MINOR.PATCH` (e.g. `v0.1.1`)
* **Upstream Compatibility Matrix:**
  | Sharkord Plus Version | Compatible Sharkord Upstream Server |
  |-----------------------|-------------------------------------|
  | `v0.1.x`              | `v0.0.31`+ (and backward compatible to `0.0.28`) |

Because Sharkord Plus implements automatic **Capability Negotiation**, our desktop and web clients gracefully detect whether they are connected to a vanilla Sharkord server or a Sharkord Plus server, enabling standard features on vanilla instances and unlocking enhanced codecs, per-window audio, and server capabilities when available.

---

## 🤝 Contributing

We welcome contributions, bug fixes, and suggestions!
- Read our [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for code conventions.
- All code follows modern ES6+ in TypeScript and `#![forbid(unsafe_code)]` in Rust.
- Run `bun run magic` and `cargo test --all` before opening a pull request.

---

## 📄 License & Credits

- Licensed under the **MIT License** — see [LICENSE](LICENSE) for details.
- Original Sharkord code & architecture Copyright © [diogomartino](https://github.com/diogomartino).
- Extended and maintained by [HomelessAvatar](https://github.com/HomelessAvatar) and contributors.
