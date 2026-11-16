# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

Self-hosters, gaming communities, developer teams, and privacy-conscious real-time communication groups seeking an independent, lightweight, self-hosted alternative to Discord without bloatware, tracking, or resource hogging.

## Product Purpose

Sharkord Plus provides a blazingly fast, privacy-respecting, self-hosted chat, voice, and video platform. It solves high CPU/memory consumption and platform limitations by offering a high-performance architecture:
- Modernized, responsive web and desktop frontend.
- Discord Phoenix-style native Rust + GPUI (`gpui-kit`) desktop client with GPU rendering and near-zero memory footprint.
- Full backwards compatibility with upstream/standard Sharkord servers.

## Positioning

The only self-hosted communication suite combining a rich Bun/tRPC server with a native GPU-accelerated (GPUI) desktop client, capable of connecting seamlessly to both standard Sharkord and Sharkord Plus instances with dynamic capability negotiation, hardware-accelerated video codecs, and isolated process audio sharing.

## Operating Context

- **Usage Environments:** Desktop multi-monitor setups during intense gaming or programming sessions; mobile devices on the go.
- **Rituals & Workflows:** Voice channel hangouts, screen/window streaming with low latency, text channels with markdown & media preview, permission management, and rapid server switching.
- **Operating Mode:** Operate mode — fast scanning, immediate responsiveness, high information density without clutter, frictionless keyboard shortcuts.

## Capabilities and Constraints

- **Protocol & Compatibility:** Full backwards compatibility with standard Sharkord tRPC routers and Mediasoup WebRTC topologies. Dynamic capability negotiation enables Sharkord Plus exclusive features (enhanced H.264 High profiles, isolated window audio) without breaking upstream compatibility.
- **Desktop Client:** Rust + GPUI (`gpui-kit`), DirectX 12 / Vulkan / Metal GPU pipeline.
- **Navigation Model:** Classic Discord panel hierarchy (collapsible left drawer for servers & channels, center chat/voice canvas, right collapsible drawer for member list) optimized for fluid desktop sizing and mobile touch/swipe drawers.
- **Visual Tone:** Pure Neutral Dark — low-saturation, industrial, refined dark grays inspired by Linear and Raycast, prioritizing high clarity, typography, and content over neon decorations.

## Brand Commitments

- **Aesthetic:** Pure Neutral Dark (zinc/slate neutrals, `#0e0f12` background, subtle `#18191e` panels, crisp `#ffffff` primary text, `#8b8f9a` muted text, precision hairline borders).
- **Layout Paradigm:** Classic 3-pane layout with sliding drawer mechanics on touch/mobile viewports.
- **Voice & Tone:** Functional, crisp, developer-grade, distraction-free.

## Evidence on Hand

- Existing codebase: Monorepo with Bun workspaces (`apps/server`, `apps/client`, `apps/desktop`, `packages/*`).
- Completed fixes: Hardware encoding across codecs (Issue #942) and isolated window audio sharing (Issue #925).
- Rust toolchain: `cargo 1.100.0` and `rustc 1.100.0` installed.

## Product Principles

1. **Lightweight & Blazing Fast:** Minimal CPU and RAM footprint; rendering never drops frames or lags behind input.
2. **Backwards Compatible First:** A Sharkord Plus client can connect to any vanilla Sharkord server out-of-the-box.
3. **Respect the Muscle Memory:** Stick to the proven Discord-style spatial layout so users feel immediately at home, elevated by superior responsiveness.
4. **Content-First Simplicity:** Pure neutral dark tones ensure avatars, shared media, and text are the center of attention.

## Accessibility & Inclusion

- High contrast text ratios adhering to WCAG AA.
- Minimum 44px touch targets on mobile drawer toggles and voice controls.
- Full keyboard navigation support (shortcuts, command palette, focus rings).
