# Hardware Encoding Across All Codecs & Window-Only Audio Sharing Design Spec

## Intent & Why
Screen sharing in Sharkord currently suffers from two major limitations on Windows:
1. **Software Encoding Bottlenecks (Issue #942):** Even with dedicated GPUs (NVIDIA RTX, AMD Radeon, Intel Arc), screen sharing defaults to software encoding because of codec negotiation (Constrained Baseline H.264 `42e01f`), hardcoded VP8 simulcast, and missing Chromium hardware acceleration flags on Windows. When users play a game or run heavy applications while sharing, CPU usage spikes to 95%+, resulting in dropped frames, degraded resolution, and a choppy stream.
2. **Audio Leakage during Window Sharing (Issue #925):** When a user shares a specific window or game, all desktop audio (music, notifications, system chimes, and other users' voices) is broadcast. Users expect Discord-like behavior where sharing a specific application isolates and broadcasts only that application's audio stream.

## Goals
- Enable GPU hardware acceleration across all supported video codecs (H.264 High/Main, H.265/HEVC, AV1, VP8/VP9) on Windows, macOS, and Linux.
- Allow simulcast to use the user's selected codec (or High profile) instead of hardcoding VP8.
- Add H.265/HEVC to `VideoCodec` and Mediasoup router capabilities.
- Recognize Windows hardware encoders (`mediafoundation`, `d3d11`, `d3d12`, `dxva`) in the stats overlay.
- Provide Chromium flags for Windows video acceleration in the desktop app.
- Capture isolated audio for single-window shares (process loopback) while preserving full audio loopback for entire-screen shares.

## Non-Goals
- Replacing Electron with Rust + GPUI in this phase (scheduled for Phase 2).
- Server-side video transcoding (Mediasoup remains a zero-transcode SFU).

## Architecture & Detailed Design

### 1. Codec Negotiation & Hardware Profile Prioritization
- **H.264:** Chromium on Windows only uses OpenH264 (software) for Constrained Baseline (`profile-level-id` starting with `42e0`). Baseline (`4200`), Main (`4d00`), and High (`6400`) utilize `D3D12VideoEncodeAccelerator` / `MediaFoundationVideoEncodeAccelerator`. The client's codec picker will filter and prioritize High profile (`64...`), then Main (`4d...`), then Baseline.
- **H.265 / HEVC:**
  - Add `H265 = 'video/H265'` to `VideoCodec` enum in `apps/client/src/types.ts`.
  - Add `video/H265` to `defaultRouterOptions.mediaCodecs` in `apps/server/src/runtimes/voice.ts`.
  - Add Chromium feature flag `PlatformHEVCDecoderSupport` in desktop startup flags.
- **AV1:**
  - Support AV1 hardware encoding when hardware profiles are present, maintaining `libaom` as software fallback.
- **Simulcast:**
  - In `apps/client/src/components/voice-provider/helpers.ts`, update `getSimulcastCodec` to accept the user's preferred codec (matching mimeType and profile preference) instead of hardcoding `VideoCodec.VP8`.

### 2. Electron Startup Flags
In `apps/desktop/src/main/startup/enable-chromium-features.ts`:
- Append Windows-specific acceleration flags:
  - `AcceleratedVideoEncoder`
  - `MediaFoundationVideoEncodeAccelerator`
  - `D3D11VideoDecoder`
  - `PlatformHEVCDecoderSupport`

### 3. Hardware Encoder Detection in Stats Popover
In `apps/client/src/components/left-sidebar/stats-popover.tsx`:
- Add `'mediafoundation'`, `'d3d11'`, `'d3d12'`, `'dxva'` to `hardwareEncoders` array.

### 4. Window-Only Audio Isolation
- **Source Picker:** Pass the window's process ID (PID) alongside the screen/window source ID.
- **Audio Capture Routing:**
  - For full-screen shares: Capture all desktop audio (with exclusion of Sharkord's own audio nodes).
  - For window shares: Target the window's PID using WASAPI Process Loopback or venmic application targeting (`include: [{ 'application.process.id': pid }]`).
  - WebRTC streams only the isolated audio track.
