# Architecture & Design: Cross-Platform Mobile Client (Android & iOS)

## 1. Intent & Problem Statement

Sharkord Plus currently provides:
1. Full-stack web client (`apps/client`) in React, Redux, and Tailwind.
2. Ultra-fast native desktop client (`apps/desktop-gpui`) in Rust and GPUI with native voice/video engine.
3. Backend server (`apps/server`) with tRPC, SQLite/Drizzle, and mediasoup WebRTC SFU.

To enable users to communicate seamlessly on mobile, Sharkord Plus requires a **high-performance, touch-first native mobile application** for **Android and iOS** built with Flutter and Rust FFI.

---

## 2. Goals, Non-Goals & Anti-Goals

### Goals
- **Single Cross-Platform Codebase:** Built with **Flutter (Dart 3+)** inside `apps/mobile/`, compiling to native ARM64 binaries for Android (`.apk` / `.aab`) and iOS (`.ipa` / Runner).
- **Core Rust Engine Reuse:** Connect `crates/sharkord-core` to Dart via `flutter_rust_bridge` (FRB v2), handling tRPC protocol transport, authentication, WebSocket subscriptions, channel/message state, and voice signaling.
- **Robust WebRTC Media Transport:**
  - Utilize `flutter_webrtc` (Google WebRTC C++ engine) for mobile microphone/speaker hardware access with built-in hardware Acoustic Echo Cancellation (AEC), Noise Suppression (NS), and RTP streaming.
  - Video receiving capability via native `RTCVideoRenderer` / `RTCVideoView` to watch remote screen shares and participant video feeds.
- **Discord-Grade Mobile UX:**
  - Drawer-based navigation (left swipe for Server Rail & Channel Tree, right swipe for Member Roster).
  - Floating/docked bottom mini voice bar with active latency, mute, deafen, and disconnect controls.
  - Full-screen or bottom-sheet interactive Voice Stage with avatar cards and green speaking rings.
  - Virtualized message stream with optimistic delivery, reactions, and image/attachment rendering.
- **Background Voice & Hardware Integration:**
  - **Android:** Foreground Service with persistent audio notification (`android.permission.FOREGROUND_SERVICE_MICROPHONE`).
  - **iOS:** Background audio execution mode with `AVAudioSessionCategoryPlayAndRecord` and Bluetooth/speaker routing.
- **Design System Parity:** Strict adherence to Pure Neutral Dark tokens (`#0a0b0e`, `#111216`, `#16171d`, `#1c1e24`, `#3b82f6`, `#22c55e`).

### Non-Goals
- Mobile screen share broadcasting from phone screen in initial phase (receiving and viewing remote screen shares is supported via `RTCVideoView`).
- Server-side APNs/FCM push notifications in Phase 3 (messaging operates via WebSocket real-time subscriptions while app is foregrounded; background execution is reserved for active voice channels).
- Simultaneous multi-server socket multiplexing (the mobile client connects to one active server at a time, switching gracefully when tapped).

### Anti-Goals
- Sluggish webviews or hybrid wrapper shells (no Cordova/Capacitor/Electron wrappers).
- Audio drops or disconnects when switching apps or locking the phone screen.

---

## 3. Global Constraints
- **Language Policy:** Zero Turkish text across all code comments, test names, identifiers, commit messages, and documentation files. Direct communication with the user is in Turkish.
- **State Management:** Reactive state management using **Riverpod** for predictable, testable state flows.
- **Design Tokens:** Strict adherence to Pure Neutral Dark palette (`#0a0b0e`, `#111216`, `#16171d`, `#1c1e24`, `#3b82f6`, `#22c55e`).
- **Quality Gates:** 100% test pass rate across both Rust crates and Flutter test suites.

---

## 4. Evaluated Approaches & Selection

### Approach A: Flutter (`flutter_webrtc`) + Rust FFI Core (Selected)
- **Mechanism:**
  - Flutter provides the UI at 60/120 FPS.
  - `flutter_webrtc` manages native audio/video hardware capture, hardware AEC, and RTP transports on iOS/Android.
  - `crates/sharkord-core` (Rust FFI via `flutter_rust_bridge`) manages tRPC signaling, session tokens, WebSocket subscriptions, message caching, and models.
- **Pros:**
  - 100% code sharing between Android and iOS.
  - Best battery efficiency: leverages OS hardware audio processing (VoiceProcessingIO on iOS, AAudio on Android).
  - Native video texture rendering with zero custom video decoding plumbing in Rust.
  - Complete reuse of `sharkord-core` business logic and models.
- **Cons:** Requires configuring Android NDK and iOS Xcode toolchains for cross-compiling Rust.

### Approach B: Custom Native WebRTC Pipeline in Rust for Mobile
- **Mechanism:** Compile native WebRTC C++ or `webrtc-rs` inside Rust and bind directly to Android Oboe/AAudio and iOS CoreAudio.
- **Pros:** Full audio pipeline entirely in Rust.
- **Cons:** Massive build footprint, high compilation complexity across NDK/Xcode, and difficulty integrating hardware echo cancellation on Android.

**Decision:** Approach A is selected for optimal performance, battery life, stability, and fast delivery.

---

## 5. Detailed Architecture & Components

### 5.1 Project Directory Structure (`apps/mobile/`)
```
apps/mobile/
├── android/             # Native Android project (Gradle, AndroidManifest, Proguard)
├── ios/                 # Native iOS project (Xcode workspace, Podfile, Info.plist)
├── lib/
│   ├── bridge/          # Generated flutter_rust_bridge API and bindings
│   ├── core/            # Theme, constants, networking, storage, audio service
│   ├── providers/       # Riverpod state providers (auth, channels, messages, voice)
│   ├── screens/         # Main workspace, login, server settings, channel chat
│   └── widgets/         # Voice bar, drawer, message card, member list, avatar
├── test/                # Unit and widget test suite
├── rust/                # Bridge crate interfacing with sharkord-core
└── pubspec.yaml         # Flutter dependencies
```

### 5.2 Rust Mobile Bridge Crate (`crates/sharkord-mobile/`)
- Intermediary bridge crate compiling `sharkord-core` into a C-ABI / FRB static/dynamic library (`.so` for Android, `.a` / Framework for iOS).
- Exported asynchronous APIs:
  - `init_client(base_url: String) -> Result<()>`
  - `authenticate(token: String) -> Result<User>`
  - `fetch_channels(server_id: String) -> Result<Vec<Channel>>`
  - `fetch_messages(channel_id: u64, before_id: Option<u64>) -> Result<Vec<Message>>`
  - `send_message(channel_id: u64, content: String) -> Result<Message>`
  - `request_voice_signaling(channel_id: u64) -> Result<VoiceSignalingParams>`
  - `listen_events() -> StreamSink<MobileEvent>` (dispatches speaking state, incoming messages, member updates).

### 5.3 UI & Navigation Flow
1. **Auth & Gateway Screen:**
   - Server URL input, email/password or guest login.
   - Saves token in secure storage (`flutter_secure_storage`).
2. **Main Workspace Screen (Three-Pane Drawer UX):**
   - **Left Drawer:** Server Rail (vertical icon bar) + Channel Tree (text & voice channels with active speaking indicators).
   - **Center Screen:**
     - Channel header with title and member toggle button.
     - Scrollable message history with auto-scroll to bottom on new messages.
     - Composer input with attachment and send buttons.
   - **Right Drawer:** Grouped member list (Online / Offline) with role colors and status dots.
3. **Docked Mini Voice Bar:**
   - Pinned above bottom navigation bar when connected to voice.
   - Displays channel name, ping indicator, mute toggle, deafen toggle, and disconnect button.
   - Tap opens the **Voice Stage Modal / Bottom Sheet**.
4. **Voice Stage Bottom Sheet:**
   - 2x2 or 3x3 responsive participant grid.
   - Speaking indicator green glowing border.
   - Native `RTCVideoView` tile when a participant is sharing their screen.

### 5.4 Background Audio Services
- **Android:**
  - Foreground Service with notification channel `voice_service`.
  - Notification displays "Connected to #[channel_name]" with Disconnect and Mute quick actions.
  - WakeLock and WiFiLock to prevent socket suspension.
- **iOS:**
  - Background Modes: `audio`, `voip`.
  - Audio session setup with `AVAudioSessionCategoryPlayAndRecord` and `allowBluetooth`.

---

## 6. What's Left to the Builder
- Generating Android app signing keys for debug/release builds.
- App icons and launch splash screens matching Pure Neutral Dark shark logo.
