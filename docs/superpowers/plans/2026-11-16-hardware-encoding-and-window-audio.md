# Hardware Video Encoding & Isolated Window Audio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable hardware-accelerated screen sharing on Windows across all supported video codecs (H.264 High/Main, H.265/HEVC, AV1, VP8/VP9) and support isolated per-window audio capture.

**Architecture:** Update Mediasoup router codecs on the server to register H.265 and prioritize High-profile H.264. Update client-side codec negotiation to select hardware-friendly profiles and dynamic simulcast codecs. Enable Windows hardware video encode Chromium flags in Electron, recognize Windows encoders in UI stats, and extend desktop bridge system audio capture with target process filtering.

**Tech Stack:** Bun, TypeScript, React 19, mediasoup / mediasoup-client, Electron, WebRTC.

**Spec:** `docs/superpowers/specs/2026-11-16-hardware-encoding-and-window-audio-design.md`

## Global Constraints
- Follow existing Sharkord architectural patterns: named exports, kebab-case files, `T`-prefixed types, no `any`.
- Must pass `bun run test` and `bun run magic` (lint:fix, format, check-types).
- Do not break backward compatibility with existing clients or servers.

## Review Focus
1. **Fallback when hardware codec is missing:** If a client requests H.264 High profile or AV1 but the device only supports baseline or software, codec negotiation must cleanly fallback without failing the connection.
2. **Simulcast codec match:** If simulcast is enabled with H.264 High, all layers must negotiate H.264 rather than silently desyncing with VP8.
3. **Audio capture fallback:** If window-specific audio capture fails or PID is missing, fall back gracefully to full loopback or log a clear warning rather than dropping the entire screen share stream.
4. **Desktop bridge backward compatibility:** Old desktop apps must still connect without error (`targetProcessId` optional).
5. **Stats popover unknown encoder:** Vendor-prefixed Windows Media Foundation strings like `MediaFoundationVideoEncodeAccelerator (NVIDIA H.264 Encoder MFT)` must be classified as GPU.

---

### Task 1: Server and Shared Codec Capabilities (H265 & Router Profiles)

**Files:**
- Modify: `apps/client/src/types.ts`
- Modify: `apps/server/src/runtimes/voice.ts`
- Test: `apps/server/src/runtimes/__tests__/voice-runtime.test.ts`

**Interfaces:**
- `VideoCodec.H265 = 'video/H265'`
- `defaultRouterOptions.mediaCodecs` contains `video/H265` and orders `640032` and `4d0032` before `42e01f`.

- [ ] **Step 1: Write the failing test**

Add tests to `apps/server/src/runtimes/__tests__/voice-runtime.test.ts` asserting that `defaultRouterOptions.mediaCodecs` includes `video/H265` and that H264 codecs prioritize High profile (`640032`) over Constrained Baseline (`42e01f`).

```ts
test('should include video/H265 and prioritize H264 High profile in defaultRouterOptions', () => {
  const h265 = defaultRouterOptions.mediaCodecs?.find((c) => c.mimeType === 'video/H265');
  expect(h265).toBeDefined();

  const h264Codecs = defaultRouterOptions.mediaCodecs?.filter((c) => c.mimeType === 'video/H264');
  expect(h264Codecs?.[0].parameters?.['profile-level-id']).toBe('640032');
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `bun test apps/server/src/runtimes/__tests__/voice-runtime.test.ts`
Expected: FAIL due to missing H265 or codec order.

- [ ] **Step 3: Implement H265 in `apps/client/src/types.ts` and `apps/server/src/runtimes/voice.ts`**

Update `VideoCodec` enum in `types.ts` and reorder `defaultRouterOptions.mediaCodecs` in `voice.ts` to place `640032` and `4d0032` first, adding `video/H265`.

- [ ] **Step 4: Run test to verify it passes**

Run: `bun test apps/server/src/runtimes/__tests__/voice-runtime.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/client/src/types.ts apps/server/src/runtimes/voice.ts apps/server/src/runtimes/__tests__/voice-runtime.test.ts
git commit -m "feat(voice): add H265 codec support and prioritize H264 High profile"
```

---

### Task 2: Client Hardware Codec Selection & Dynamic Simulcast

**Files:**
- Modify: `apps/client/src/components/voice-provider/helpers.ts`
- Modify: `apps/client/src/components/voice-provider/index.tsx`
- Test: `apps/client/src/components/voice-provider/__tests__/helpers.test.ts`

**Interfaces:**
- Consumes: `VideoCodec`, `RtpCodecCapability`
- Produces:
  - `getPreferredVideoCodec(codecs: RtpCodecCapability[], preferredCodec: VideoCodec): RtpCodecCapability | undefined`
  - `getSimulcastCodec(rtpCapabilities: RtpCapabilities | null, preferredCodec?: VideoCodec): RtpCodecCapability | undefined`

- [ ] **Step 1: Write the failing test**

In `apps/client/src/components/voice-provider/__tests__/helpers.test.ts`, add test cases for `getPreferredVideoCodec` and `getSimulcastCodec` ensuring:
1. When `VideoCodec.H264` is preferred, High profile `640032` is selected over `42e01f`.
2. When `VideoCodec.AUTO` is preferred, High profile `640032` is preferred.
3. `getSimulcastCodec` selects the preferred codec if present in router capabilities, falling back to VP8.

- [ ] **Step 2: Run test to verify it fails**

Run: `bun test apps/client/src/components/voice-provider/__tests__/helpers.test.ts`
Expected: FAIL

- [ ] **Step 3: Implement `getPreferredVideoCodec` and dynamic `getSimulcastCodec`**

Implement helper functions in `helpers.ts` and integrate into `index.tsx` where screen share producer codec options are set.

- [ ] **Step 4: Run test to verify it passes**

Run: `bun test apps/client/src/components/voice-provider/__tests__/helpers.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/client/src/components/voice-provider/helpers.ts apps/client/src/components/voice-provider/index.tsx apps/client/src/components/voice-provider/__tests__/helpers.test.ts
git commit -m "feat(voice): select hardware H264 profile and dynamic simulcast codec"
```

---

### Task 3: Recognize Windows Hardware Encoders in Stats Popover

**Files:**
- Modify: `apps/client/src/components/left-sidebar/stats-popover.tsx`
- Test: `apps/client/src/components/left-sidebar/__tests__/stats-popover.test.ts`

**Interfaces:**
- `isHardwareEncoder(encoderImplementation: string): boolean | null`

- [ ] **Step 1: Write the failing test**

Create `apps/client/src/components/left-sidebar/__tests__/stats-popover.test.ts` testing encoder name classification:
- `MediaFoundationVideoEncodeAccelerator (NVIDIA H.264 Encoder MFT)` -> `true`
- `D3D12VideoEncodeAccelerator` -> `true`
- `OpenH264` -> `false`
- `libaom` -> `false`

- [ ] **Step 2: Run test to verify it fails**

Run: `bun test apps/client/src/components/left-sidebar/__tests__/stats-popover.test.ts`
Expected: FAIL

- [ ] **Step 3: Update `hardwareEncoders` and export `isHardwareEncoder` in `stats-popover.tsx`**

Add `'mediafoundation'`, `'d3d11'`, `'d3d12'`, `'dxva'` to `hardwareEncoders`.

- [ ] **Step 4: Run test to verify it passes**

Run: `bun test apps/client/src/components/left-sidebar/__tests__/stats-popover.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/client/src/components/left-sidebar/stats-popover.tsx apps/client/src/components/left-sidebar/__tests__/stats-popover.test.ts
git commit -m "feat(stats): detect Windows MediaFoundation and D3D12 hardware encoders"
```

---

### Task 4: Enable Windows GPU Video Acceleration Flags in Desktop App

**Files:**
- Modify: `apps/desktop/src/main/startup/enable-chromium-features.ts`
- Test: `apps/desktop/src/main/startup/__tests__/enable-chromium-features.test.ts`

**Interfaces:**
- `getHardwareVideoFeatureList(platform: NodeJS.Platform): string[]`

- [ ] **Step 1: Write the failing test**

Create `apps/desktop/src/main/startup/__tests__/enable-chromium-features.test.ts` asserting that on Windows (`win32`), features include `AcceleratedVideoEncoder`, `MediaFoundationVideoEncodeAccelerator`, `D3D11VideoDecoder`, and `PlatformHEVCDecoderSupport`.

- [ ] **Step 2: Run test to verify it fails**

Run: `bun test apps/desktop/src/main/startup/__tests__/enable-chromium-features.test.ts`
Expected: FAIL

- [ ] **Step 3: Implement platform-aware feature selection in `enable-chromium-features.ts`**

Export `getHardwareVideoFeatureList` and append Windows acceleration features when `process.platform === 'win32'`.

- [ ] **Step 4: Run test to verify it passes**

Run: `bun test apps/desktop/src/main/startup/__tests__/enable-chromium-features.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop/src/main/startup/enable-chromium-features.ts apps/desktop/src/main/startup/__tests__/enable-chromium-features.test.ts
git commit -m "feat(desktop): enable Windows GPU encoding and HEVC Chromium flags"
```

---

### Task 5: Isolated Window Audio Sharing Support

**Files:**
- Modify: `packages/shared/src/desktop.ts`
- Modify: `apps/desktop/src/main/screen-share/system-audio.ts`
- Modify: `apps/desktop/src/main/views/server-views.ts`
- Modify: `apps/client/src/helpers/desktop-system-audio.ts`
- Test: `apps/desktop/src/main/screen-share/__tests__/system-audio.test.ts`

**Interfaces:**
- Consumes: `TDesktopBridge.startSystemAudio(options?: { targetProcessId?: number })`
- Produces: Audio patch bay / loopback rules targeting specific `targetProcessId` when sharing a single window.

- [ ] **Step 1: Write the failing test**

In `apps/desktop/src/main/screen-share/__tests__/system-audio.test.ts`, test rule generation:
- With `targetProcessId`: loopback includes only `application.process.id` of the target PID.
- Without `targetProcessId`: loopback captures default speakers / all outputs excluding Sharkord's own nodes.

- [ ] **Step 2: Run test to verify it fails**

Run: `bun test apps/desktop/src/main/screen-share/__tests__/system-audio.test.ts`
Expected: FAIL

- [ ] **Step 3: Implement PID-targeted loopback in `system-audio.ts` and bridge**

Update `startSystemAudio` to accept optional `targetProcessId`, construct the targeted rule set, and enable Windows capture support.

- [ ] **Step 4: Run test to verify it passes**

Run: `bun test apps/desktop/src/main/screen-share/__tests__/system-audio.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add packages/shared/src/desktop.ts apps/desktop/src/main/screen-share/system-audio.ts apps/desktop/src/main/screen-share/__tests__/system-audio.test.ts apps/desktop/src/main/views/server-views.ts apps/client/src/helpers/desktop-system-audio.ts
git commit -m "feat(desktop): support isolated application audio capture for window sharing"
```

---

### Task 6: End-to-End Verification and Repository Polish

- [ ] **Step 1: Run full test suite**
Run: `bun run test`
Expected: All unit & integration tests across server, client, desktop pass.

- [ ] **Step 2: Run magic formatting & linting**
Run: `bun run magic`
Expected: Code clean, types check with 0 errors.

- [ ] **Step 3: Final branch verification & push**
Run: `git status` and verify clean working tree, then push to `origin/development`.
