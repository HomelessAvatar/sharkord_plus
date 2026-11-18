# Full Client Parity & Discord-Surpassing Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver 100% feature parity between the Sharkord Plus web client, native GPUI desktop client, and Flutter mobile client using the shared Rust core: persistent authentication & multi-server vault (never prompt for login again), real WebRTC Mediasoup voice engine (eliminate static 24ms ping and mock state), ultra-high-definition 4K & 120/144 FPS screen/video sharing, and hybrid native/bridge plugin architecture.

**Architecture:**
- **Shared Rust Core (`crates/sharkord-core`):** `SessionVault` manages multi-server token and session persistence to OS-standard config directories. `VideoResolution` and `VideoFrameRate` expanded with 4K (3840×2160), 120 FPS, 144 FPS, and arbitrary Custom resolution/framerate support.
- **Desktop GPUI (`apps/desktop-gpui`):** Source picker upgraded with 4K, 120 FPS, 144 FPS, and Custom options. Multi-server rail and persistent session restore on startup. Native slash command autocomplete and interactive message components.
- **Flutter Mobile (`apps/mobile`):** `shared_preferences` persistent vault automatically restores authentication on app launch without prompting for credentials. `flutter_webrtc` integration replaces mock voice states with real Mediasoup audio/video tracks, dynamic WebRTC RTT stats, real-time speaking glow indicators, microphone capture, and camera/screen sharing.
- **Hybrid Plugin Architecture:**
  - *Layer 1 (100% Native UI):* Slash commands (`/poll`, `/remind`, `/soundboard`, etc.) autocomplete in chat and execute via tRPC JSON; interactive message components (buttons, polls, form cards) render using native Flutter / GPUI widgets and dispatch actions directly to `SharkordClient::call_plugin_action`.
  - *Layer 2 (Isolated Webview Bridge):* Dedicated plugin tabs (full-screen soundboard panels, mini-games, complex plugin settings) render via lightweight embedded webviews injecting `window.__SHARKORD_STORE__` bridge so web plugins run unmodified across all platforms.
  - *Scope Exclusions (Per User Decision):* Built-in soundboard excluded (runs as an external plugin), Rich Presence game detection excluded, in-game transparent HUD overlay deferred to future releases.

**Tech Stack:** Rust (GPUI, `sharkord-core`, `cpal`, `serde`), Flutter 3.x / Dart 3.x (Riverpod, `shared_preferences`, `flutter_webrtc`, `http`), Bun + tRPC + Mediasoup SFU backend.

**Spec:** [full-client-parity-design-doc.md](file:///C:/Users/ersan/.gemini/antigravity-cli/brain/92ef2d26-4db1-4ae6-b877-cfb7748a2f67/full-client-parity-design-doc.md)

## Global Constraints

- Zero Turkish text across all GitHub surfaces (commits, code comments, test names, identifiers, documentation). English only. Direct conversation with user in Turkish.
- Strict backward compatibility with existing Mediasoup SFU router and tRPC endpoints.
- No regressions in existing test suites: `cargo test --workspace`, `flutter test`, and `bun run magic` must pass at every task gate.
- No hardcoded personal server domains (generic examples like `your-server.com` or `192.168.1.100` only).
- Never prompt for login if a valid stored session exists in the vault.
- Follow Pure Neutral Dark design system (`#0A0A0A` base, `#121212` surface, `#1E1E1E` elevated, `#3B82F6` accent) adhering to `/impeccable` design guidelines.

## Review Focus

1. **Session Vault Auto-Login on Launch:** Opening the app with a previously saved valid token must directly enter the workspace without displaying the login form.
2. **Dynamic WebRTC RTT & Speaking Detection:** Voice channel connection must report live variable ping (not static 24ms) and dynamically activate speaking indicators based on real audio level.
3. **4K & High Framerate Track Negotiation:** Requesting 4K (3840×2160) or 120/144 FPS must correctly compute dimensions, calculate appropriate target bitrate, and construct valid `VideoTrackConfig`.
4. **Multi-Server State Isolation:** Switching between multiple configured servers must isolate active channels, member lists, and tokens without state leakage.
5. **TLS / Network Error Handling:** Failed host lookup, self-signed certificates, or connection timeouts must surface descriptive user-friendly error banners without crashing the client.

---

### Task 1: Core Engine Video Resolution & Framerate Expansion (4K, 120/144 FPS & Custom)

**Files:**
- Modify: `crates/sharkord-core/src/voice/video.rs`
- Test: `crates/sharkord-core/tests/video_resolution_test.rs`

**Interfaces:**
- Consumes: `serde::{Serialize, Deserialize}`
- Produces: `VideoResolution::{R720p, R1080p, R1440p, R4K, Custom { width: u32, height: u32 }}`
- Produces: `VideoFrameRate::{Fps15, Fps30, Fps60, Fps120, Fps144, Custom(u32)}`

- [ ] **Step 1: Write the failing tests for 4K, 120/144 FPS, and Custom presets**

```rust
// crates/sharkord-core/tests/video_resolution_test.rs
use sharkord_core::voice::video::{VideoFrameRate, VideoResolution};

#[test]
fn test_4k_resolution_dimensions_and_label() {
    let res_4k = VideoResolution::R4K;
    assert_eq!(res_4k.width(), 3840);
    assert_eq!(res_4k.height(), 2160);
    assert_eq!(res_4k.label(), "4K");
}

#[test]
fn test_custom_resolution_dimensions() {
    let custom = VideoResolution::Custom { width: 1920, height: 1200 };
    assert_eq!(custom.width(), 1920);
    assert_eq!(custom.height(), 1200);
    assert_eq!(custom.label(), "1920x1200");
}

#[test]
fn test_esports_framerates() {
    let fps_120 = VideoFrameRate::Fps120;
    let fps_144 = VideoFrameRate::Fps144;
    let fps_custom = VideoFrameRate::Custom(240);

    assert_eq!(fps_120.fps(), 120);
    assert_eq!(fps_120.label(), "120 FPS");
    assert_eq!(fps_144.fps(), 144);
    assert_eq!(fps_144.label(), "144 FPS");
    assert_eq!(fps_custom.fps(), 240);
    assert_eq!(fps_custom.label(), "240 FPS");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test video_resolution_test`
Expected: FAIL due to missing variants `R4K`, `Custom`, `Fps120`, `Fps144`.

- [ ] **Step 3: Implement `R4K`, `Custom`, `Fps120`, `Fps144` in `crates/sharkord-core/src/voice/video.rs`**

Add `R4K` and `Custom { width: u32, height: u32 }` to `VideoResolution` with serde aliases and dimension mappings. Add `Fps120`, `Fps144`, and `Custom(u32)` to `VideoFrameRate` with label and numeric getters.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test video_resolution_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sharkord-core/src/voice/video.rs crates/sharkord-core/tests/video_resolution_test.rs
git commit -m "feat(core): add 4K, 120 FPS, 144 FPS, and custom video streaming presets"
```

---

### Task 2: Desktop GPUI 4K & Esports 120/144 FPS Source Picker UI

**Files:**
- Modify: `apps/desktop-gpui/src/ui/source_picker.rs`
- Test: `apps/desktop-gpui/tests/source_picker_test.rs`

**Interfaces:**
- Consumes: `sharkord_core::voice::video::{VideoResolution, VideoFrameRate, VideoTrackConfig}`
- Produces: `SourcePickerState::set_resolution`, `SourcePickerState::set_framerate`, `SourcePickerView`

- [ ] **Step 1: Write the failing tests for source picker presets**

```rust
// apps/desktop-gpui/tests/source_picker_test.rs
use desktop_gpui::ui::source_picker::SourcePickerState;
use sharkord_core::voice::video::{ScreenSource, VideoFrameRate, VideoResolution};

#[test]
fn test_source_picker_allows_4k_and_144fps_selection() {
    let mut state = SourcePickerState::new();
    state.select_source(ScreenSource::Monitor {
        id: 1,
        name: "Primary Display".to_string(),
    });
    state.set_resolution(VideoResolution::R4K);
    state.set_framerate(VideoFrameRate::Fps144);

    let config = state.build_track_config().expect("config should be built");
    assert_eq!(config.resolution, VideoResolution::R4K);
    assert_eq!(config.framerate, VideoFrameRate::Fps144);
    assert_eq!(config.resolution.width(), 3840);
    assert_eq!(config.framerate.fps(), 144);
}
```

- [ ] **Step 2: Run test to verify it fails or passes**

Run: `cargo test --test source_picker_test`
Expected: Verify compilation and behavior against `SourcePickerState`.

- [ ] **Step 3: Update `apps/desktop-gpui/src/ui/source_picker.rs`**

Expand `resolutions` slice in `SourcePickerView::render_element` to include `VideoResolution::R4K`. Expand `framerates` slice to include `VideoFrameRate::Fps120` and `VideoFrameRate::Fps144`. Adjust container widths to accommodate the new badges cleanly.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p desktop-gpui`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add apps/desktop-gpui/src/ui/source_picker.rs apps/desktop-gpui/tests/source_picker_test.rs
git commit -m "feat(desktop-gpui): add 4K and 120/144 FPS options to screen share source picker"
```

---

### Task 3: Core Multi-Server Session Vault (`SessionVault`)

**Files:**
- Create: `crates/sharkord-core/src/vault.rs`
- Modify: `crates/sharkord-core/src/lib.rs`
- Test: `crates/sharkord-core/tests/vault_test.rs`

**Interfaces:**
- Consumes: `serde::{Serialize, Deserialize}`, `std::path::PathBuf`
- Produces: `SavedServerSession`, `SessionVault` (`load`, `save`, `add_or_update`, `remove`, `get_active`, `set_active`)

- [ ] **Step 1: Write the failing test for `SessionVault`**

```rust
// crates/sharkord-core/tests/vault_test.rs
use sharkord_core::vault::{SavedServerSession, SessionVault};

#[test]
fn test_session_vault_add_save_and_retrieve_active() {
    let temp_dir = std::env::temp_dir().join(format!("sharkord_test_vault_{}", std::process::id()));
    let vault_path = temp_dir.join("vault.json");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut vault = SessionVault::new(vault_path.clone());
    assert!(vault.get_active().is_none());

    let server = SavedServerSession {
        server_id: "srv_123".to_string(),
        server_name: "Test Community".to_string(),
        server_url: "https://test.sharkord.com".to_string(),
        token: "jwt_token_sample".to_string(),
        user_id: 42,
        username: "testuser".to_string(),
        display_name: "Test User".to_string(),
        avatar_url: None,
        last_accessed_at: 1700000000,
    };

    vault.add_or_update(server.clone(), true).expect("should save");
    assert_eq!(vault.get_active().unwrap().server_id, "srv_123");

    // Reopen vault from disk
    let loaded_vault = SessionVault::load_from_path(vault_path).expect("should load");
    assert_eq!(loaded_vault.get_active().unwrap().token, "jwt_token_sample");
    assert_eq!(loaded_vault.servers.len(), 1);

    let _ = std::fs::remove_dir_all(&temp_dir);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test vault_test`
Expected: FAIL due to missing module `sharkord_core::vault`.

- [ ] **Step 3: Implement `SessionVault` in `crates/sharkord-core/src/vault.rs`**

Implement `SavedServerSession` and `SessionVault` with JSON file serialization/deserialization, atomic file writes, active server pointer, and error handling. Expose `pub mod vault;` in `crates/sharkord-core/src/lib.rs`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test vault_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sharkord-core/src/vault.rs crates/sharkord-core/src/lib.rs crates/sharkord-core/tests/vault_test.rs
git commit -m "feat(core): implement multi-server SessionVault for persistent client sessions"
```

---

### Task 4: Mobile Persistent Vault & Multi-Server State (`shared_preferences`)

**Files:**
- Modify: `apps/mobile/pubspec.yaml`
- Create: `apps/mobile/lib/core/storage/vault_service.dart`
- Modify: `apps/mobile/lib/providers/auth_provider.dart`
- Modify: `apps/mobile/lib/main.dart`
- Modify: `apps/mobile/lib/widgets/channel_drawer.dart`
- Test: `apps/mobile/test/vault_service_test.dart`

**Interfaces:**
- Consumes: `shared_preferences`
- Produces: `VaultService` (`getSavedServers`, `saveServer`, `getActiveServer`, `setActiveServerId`, `clearAll`)
- Produces: `AuthProvider.tryRestoreSession() -> Future<bool>`

- [ ] **Step 1: Add `shared_preferences: ^2.3.3` to `apps/mobile/pubspec.yaml`**

Run: `cd apps/mobile; flutter pub add shared_preferences:^2.3.3`

- [ ] **Step 2: Write failing test for `VaultService` and persistent authentication restore**

```dart
// apps/mobile/test/vault_service_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:sharkord_mobile/core/storage/vault_service.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('VaultService persists and retrieves server session without prompt', () async {
    SharedPreferences.setMockInitialValues({});
    final vault = VaultService();

    final testServer = SavedServerSession(
      serverId: 'srv_alpha',
      serverName: 'Sharkord Plus HQ',
      serverUrl: 'https://hq.sharkord.com',
      token: 'persisted_jwt_token_999',
      userId: '42',
      username: 'alice',
      displayName: 'Alice',
    );

    await vault.saveServer(testServer, setAsActive: true);

    final active = await vault.getActiveServer();
    expect(active, isNotNull);
    expect(active!.serverId, 'srv_alpha');
    expect(active.token, 'persisted_jwt_token_999');

    final all = await vault.getSavedServers();
    expect(all.length, 1);
  });
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd apps/mobile; flutter test test/vault_service_test.dart`
Expected: FAIL due to missing `vault_service.dart`.

- [ ] **Step 4: Implement `VaultService` and wire into `AuthNotifier` and `main.dart`**

1. Create `apps/mobile/lib/core/storage/vault_service.dart`.
2. Add `tryRestoreSession()` to `AuthNotifier` which loads the active server from `VaultService`. If found, it populates `AuthState` with stored user and token and triggers WebSocket reconnection in background.
3. Update `main.dart`: On app launch, run `tryRestoreSession()` before deciding whether to show `WorkspaceScreen` or `LoginScreen`.
4. Update `apps/mobile/lib/widgets/channel_drawer.dart` to list all saved servers on the left rail with active selection, add-server modal, and delete action.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd apps/mobile; flutter test`
Expected: PASS (all tests pass)

- [ ] **Step 6: Commit**

```bash
git add apps/mobile/pubspec.yaml apps/mobile/lib/ apps/mobile/test/
git commit -m "feat(mobile): implement persistent authentication and multi-server vault"
```

---

### Task 5: Real Mobile WebRTC Mediasoup Voice & Dynamic Ping

**Files:**
- Modify: `apps/mobile/pubspec.yaml`
- Create: `apps/mobile/lib/core/voice/webrtc_voice_engine.dart`
- Modify: `apps/mobile/lib/providers/voice_provider.dart`
- Modify: `apps/mobile/lib/widgets/voice_bar.dart`
- Modify: `apps/mobile/lib/widgets/voice_stage_sheet.dart`
- Test: `apps/mobile/test/voice_engine_test.dart`

**Interfaces:**
- Consumes: `flutter_webrtc`, Mediasoup RTP parameters over tRPC WebSocket
- Produces: `WebRTCVoiceEngine` (`join`, `leave`, `setMuted`, `setDeafened`, `onPingChanged`, `onSpeakerLevelsChanged`)

- [ ] **Step 1: Add `flutter_webrtc: ^0.11.7` to `apps/mobile/pubspec.yaml`**

Run: `cd apps/mobile; flutter pub add flutter_webrtc:^0.11.7`

- [ ] **Step 2: Write unit tests for dynamic ping and real voice state transitions**

```dart
// apps/mobile/test/voice_engine_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:sharkord_mobile/models/user.dart';
import 'package:sharkord_mobile/providers/voice_provider.dart';

void main() {
  test('VoiceNotifier updates dynamic ping and active speaker states', () {
    final notifier = VoiceNotifier();
    expect(notifier.state.isConnected, false);
    expect(notifier.state.pingMs, 0);

    notifier.joinVoice(
      channelId: 'voice_1',
      channelName: 'General Voice',
      currentUser: const User(id: '1', username: 'alex', displayName: 'Alex'),
    );

    expect(notifier.state.isConnected, true);
    // Dynamic ping update from WebRTC stats
    notifier.updatePing(48);
    expect(notifier.state.pingMs, 48);

    notifier.setSpeaking('1', true);
    expect(notifier.state.activeSpeakerIds, contains('1'));

    notifier.setSpeaking('1', false);
    expect(notifier.state.activeSpeakerIds, isNot(contains('1')));

    notifier.leaveVoice();
    expect(notifier.state.isConnected, false);
    expect(notifier.state.pingMs, 0);
  });
}
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cd apps/mobile; flutter test test/voice_engine_test.dart`
Expected: PASS

- [ ] **Step 4: Implement `WebRTCVoiceEngine` in `apps/mobile/lib/core/voice/webrtc_voice_engine.dart`**

Implement Mediasoup transport signaling over tRPC socket (`transport.connect`, `transport.produce`, `transport.consume`), audio track acquisition via `navigator.mediaDevices.getUserMedia`, periodic stats polling for dynamic RTT (`stats.roundTripTime`) and audio volume calculation (`audioLevel` > threshold -> active speaking). Wire callbacks into `VoiceNotifier`.

- [ ] **Step 5: Run tests and verify**

Run: `cd apps/mobile; flutter test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add apps/mobile/pubspec.yaml apps/mobile/lib/ apps/mobile/test/
git commit -m "feat(mobile): integrate WebRTC Mediasoup engine with dynamic RTT and speaking detection"
```

---

### Task 6: Native Plugin Protocol & Interactive Component Handlers

**Files:**
- Modify: `crates/sharkord-core/src/types.rs`
- Modify: `apps/desktop-gpui/src/ui/chat.rs`
- Modify: `apps/mobile/lib/models/message.dart`
- Modify: `apps/mobile/lib/widgets/chat_canvas.dart`
- Test: `apps/mobile/test/interactive_components_test.dart`
- Test: `crates/sharkord-core/tests/plugin_protocol_test.rs`

**Interfaces:**
- Consumes: tRPC plugin command schema & interactive JSON components (`PluginButton`, `PluginPoll`, `PluginCard`)
- Produces: Native UI rendering of interactive message attachments and slash command dispatch (`SharkordClient::call_plugin_action`)

- [ ] **Step 1: Write test for interactive component parsing and slash command invocation**

```rust
// crates/sharkord-core/tests/plugin_protocol_test.rs
use sharkord_core::types::{PluginActionPayload, PluginCommand};

#[test]
fn test_plugin_command_serialization() {
    let cmd = PluginCommand {
        plugin_id: "soundboard".to_string(),
        name: "play".to_string(),
        description: "Play sound bite into voice channel".to_string(),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("soundboard"));
}
```

- [ ] **Step 2: Run test to verify it passes or fails**

Run: `cargo test --test plugin_protocol_test`

- [ ] **Step 3: Implement interactive component renderers and action dispatch**

1. Parse interactive component JSON in message payloads (`buttons`, `actions`).
2. In GPUI and Flutter, render these elements using native buttons/widgets.
3. On tap/click, dispatch `call_plugin_action(plugin_id, action_id, payload)` over tRPC socket.
4. Provide slash command auto-complete popover when user inputs `/` in the chat input bar.

- [ ] **Step 4: Run tests to verify**

Run: `cargo test --workspace`
Run: `cd apps/mobile; flutter test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/sharkord-core/ apps/desktop-gpui/ apps/mobile/
git commit -m "feat(plugins): implement native slash commands and interactive message component rendering"
```

---

### Task 7: Comprehensive Verification, Lint & Pipeline Readiness

**Files:**
- Modify: `.github/workflows/mobile.yml`
- Modify: `.github/workflows/desktop.yml`
- Run: Full repository validation

**Interfaces:**
- Consumes: All tests across Rust, Flutter, and TypeScript workspaces
- Produces: Green builds and release-ready pipelines

- [ ] **Step 1: Run Rust workspace tests**

Run: `cargo test --workspace`
Expected: PASS (all crates pass)

- [ ] **Step 2: Run Flutter mobile tests**

Run: `cd apps/mobile; flutter test`
Expected: PASS (all tests pass)

- [ ] **Step 3: Run backend & shared TypeScript checks**

Run: `bun run magic`
Expected: PASS (lint, format, and typecheck pass)

- [ ] **Step 4: Verify zero Turkish strings on GitHub surfaces**

Run: `git grep -i "lobi.genclink" ; git grep -P "[\x{00c0}-\x{017f}]" crates/ apps/mobile/lib/ .github/` (checking for stray Turkish or hardcoded private domains)
Expected: No unwanted personal domain references.

- [ ] **Step 5: Commit and Push to `development`**

```bash
git add .
git commit -m "chore: verify full client parity suite and release readiness"
git push origin development
```
