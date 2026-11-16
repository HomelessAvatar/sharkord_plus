# Rust Core & GPUI Desktop Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust Core shared library (`crates/sharkord-core`) with 100% standard Sharkord backwards compatibility, and the high-performance native desktop client (`apps/desktop-gpui`) powered by Zed's GPUI and `gpui-component` with the Pure Neutral Dark theme.

**Architecture:** Monorepo Cargo workspace with `crates/sharkord-core` providing platform-independent tRPC query/mutation/subscription handling, and `apps/desktop-gpui` providing a GPU-rendered, Discord-style 3-column UI consuming the core library asynchronously.

**Tech Stack:** Rust (1.80+), Tokio, Reqwest, Serde, Tokio-Tungstenite, GPUI (0.2+), `gpui-component`, MSVC/DirectX 12 (Windows) / Metal (macOS) / Vulkan (Linux).

**Spec:** `docs/superpowers/specs/2026-11-16-rust-core-and-gpui-desktop-design.md`

## Global Constraints

- **Backwards Compatibility:** All tRPC queries and mutations must accept and produce standard vanilla Sharkord JSON shapes (`batch=1`, `0: { json: ... }`).
- **No Unsafe Code:** `crates/sharkord-core` must be `#![forbid(unsafe_code)]` or `#![deny(unsafe_code)]`.
- **Pure Neutral Dark Design:** UI colors and typography must strictly match `DESIGN.md` (no saturated purple defaults).
- **Asynchronous UI Execution:** Network and WebSocket I/O must never run on GPUI's main thread; always dispatch via `cx.spawn()`.

## Review Focus

1. **tRPC Batch Encoding:** Verify query strings with encoded JSON match vanilla tRPC client behavior (`batch=1&input=%7B%220%22%3A%7B...%7D%7D`).
2. **Standard Server Fallback:** When connected to a vanilla server lacking Plus capabilities, client operates cleanly without crashing or throwing unknown schema errors.
3. **Session Cookie / Token Persistence:** Session token must attach to both HTTP `Cookie` / `Authorization` headers and WebSocket initial connection payloads.
4. **WebSocket Reconnection:** Automatic backoff reconnect if server restarts or network disconnects.
5. **Memory and Frame Rates:** GPUI render loop stays fluid at 60/120 FPS with minimal memory usage.

---

### Task 1: Root Cargo Workspace & `sharkord-core` Data Models

**Files:**
- Create: `Cargo.toml` (root workspace)
- Create: `crates/sharkord-core/Cargo.toml`
- Create: `crates/sharkord-core/src/lib.rs`
- Create: `crates/sharkord-core/src/types.rs`
- Create: `crates/sharkord-core/src/error.rs`
- Test: `crates/sharkord-core/tests/models_test.rs`

- [ ] **Step 1: Write the failing test**
  In `crates/sharkord-core/tests/models_test.rs`, test deserialization of vanilla Sharkord API JSON payloads (user, channel, message, settings) into Rust structs with serde.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test models_test`
  Expected: FAIL (types not defined yet)

- [ ] **Step 3: Implement root workspace, crate manifest, types, and error types**
  - Create root `Cargo.toml` with `members = ["crates/sharkord-core"]`.
  - Define `User`, `Channel`, `ChannelType`, `Message`, `ServerSettings`, `Role`, `Capabilities` in `types.rs`.
  - Define `SharkordError` in `error.rs`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test models_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add Cargo.toml crates/sharkord-core/
  git commit -m "feat(core): setup rust workspace and define sharkord data models"
  ```

---

### Task 2: tRPC Protocol Encoders & Decoders

**Files:**
- Create: `crates/sharkord-core/src/protocol/mod.rs`
- Modify: `crates/sharkord-core/src/lib.rs`
- Test: `crates/sharkord-core/tests/protocol_test.rs`

- [ ] **Step 1: Write the failing test**
  In `crates/sharkord-core/tests/protocol_test.rs`, test:
  - Building tRPC batched query URLs and JSON bodies.
  - Parsing tRPC batch responses (`{ "0": { "result": { "data": { "json": ... } } } }`).
  - Handling tRPC error envelopes (`{ "0": { "error": { "json": { "message": "...", "code": -32600 } } } }`).

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test protocol_test`
  Expected: FAIL

- [ ] **Step 3: Implement protocol encoders and decoders**
  In `crates/sharkord-core/src/protocol/mod.rs`, write `encode_query_params`, `encode_mutation_body`, `decode_response`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test protocol_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add crates/sharkord-core/src/protocol/ crates/sharkord-core/tests/protocol_test.rs
  git commit -m "feat(core): implement trpc query and mutation protocol serialization"
  ```

---

### Task 3: `SharkordClient` HTTP Client & Capability Negotiation

**Files:**
- Create: `crates/sharkord-core/src/client.rs`
- Modify: `crates/sharkord-core/src/lib.rs`
- Test: `crates/sharkord-core/tests/client_test.rs`

- [ ] **Step 1: Write the failing test**
  In `crates/sharkord-core/tests/client_test.rs`, test client instantiation, URL normalization, capability detection (Plus vs Vanilla server), and authenticated request header injection.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test client_test`
  Expected: FAIL

- [ ] **Step 3: Implement `SharkordClient` methods**
  - Implement `connect`, `login`, `get_public_settings`, `get_channels`, `get_messages`, `send_message`.
  - Handle cookie jar / session header storage.
  - Implement capability negotiation: detect server type and feature flags.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test client_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add crates/sharkord-core/src/client.rs crates/sharkord-core/tests/client_test.rs
  git commit -m "feat(core): implement SharkordClient http methods and capability negotiation"
  ```

---

### Task 4: Real-Time WebSocket Subscription Dispatcher

**Files:**
- Create: `crates/sharkord-core/src/protocol/events.rs`
- Modify: `crates/sharkord-core/src/client.rs`
- Modify: `crates/sharkord-core/src/lib.rs`
- Test: `crates/sharkord-core/tests/events_test.rs`

- [ ] **Step 1: Write the failing test**
  In `crates/sharkord-core/tests/events_test.rs`, test parsing real-time event frames (`messageCreated`, `messageUpdated`, `userJoined`, `voiceStateChanged`).

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p sharkord-core --test events_test`
  Expected: FAIL

- [ ] **Step 3: Implement WebSocket subscription protocol**
  - Frame subscription start/stop packets according to tRPC WebSocket specification.
  - Route deserialized event payloads to async Tokio `mpsc` channels.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p sharkord-core --test events_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add crates/sharkord-core/src/protocol/events.rs crates/sharkord-core/tests/events_test.rs
  git commit -m "feat(core): implement trpc websocket event subscription dispatcher"
  ```

---

### Task 5: `apps/desktop-gpui` Crate Setup, Theme & App State

**Files:**
- Modify: `Cargo.toml` (add `apps/desktop-gpui` to members)
- Create: `apps/desktop-gpui/Cargo.toml`
- Create: `apps/desktop-gpui/src/main.rs`
- Create: `apps/desktop-gpui/src/theme.rs`
- Create: `apps/desktop-gpui/src/state.rs`
- Test: `apps/desktop-gpui/tests/state_test.rs`

- [ ] **Step 1: Write the failing test**
  In `apps/desktop-gpui/tests/state_test.rs`, test state mutation logic: selecting a channel, appending a received message, toggling left/right sidebars.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test -p desktop-gpui --test state_test`
  Expected: FAIL

- [ ] **Step 3: Implement state and theme**
  - Configure `Cargo.toml` with `gpui = "0.2"`, `gpui-component = "0.10"`, `sharkord-core`.
  - Implement `theme.rs` with Pure Neutral Dark tokens from `DESIGN.md`.
  - Implement `AppState` managing active server, active channel, channels list, messages, and user session.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test -p desktop-gpui --test state_test`
  Expected: PASS

- [ ] **Step 5: Commit**
  ```bash
  git add Cargo.toml apps/desktop-gpui/
  git commit -m "feat(desktop-gpui): setup gpui crate, Pure Neutral Dark theme, and app state"
  ```

---

### Task 6: GPUI 3-Column Interface (Server Rail, Channel Tree, Chat Canvas)

**Files:**
- Create: `apps/desktop-gpui/src/ui/mod.rs`
- Create: `apps/desktop-gpui/src/ui/server_rail.rs`
- Create: `apps/desktop-gpui/src/ui/channel_tree.rs`
- Create: `apps/desktop-gpui/src/ui/chat_canvas.rs`
- Create: `apps/desktop-gpui/src/ui/voice_bar.rs`
- Modify: `apps/desktop-gpui/src/main.rs`

- [ ] **Step 1: Implement Server Rail component**
  Render left vertical pill list: Home/DM icon, server icon avatars with active indicators, add server button.

- [ ] **Step 2: Implement Channel Tree component**
  Render server header, categories (collapsible), text channels (`#`), voice channels (`🔊`), unread badges, and docked voice controls bar.

- [ ] **Step 3: Implement Chat Canvas component**
  Render header bar with channel title and topic, message history scroll view with sender avatars and timestamps, and bottom composer input field with send shortcut (`Enter`).

- [ ] **Step 4: Connect UI with `sharkord-core` Async Client**
  In `main.rs`, spawn async tasks to poll channels and receive messages, updating GPUI state via `cx.update()`.

- [ ] **Step 5: Compile & Verify**
  Run: `cargo check -p desktop-gpui` and `cargo test --all`
  Expected: Clean compilation with 0 errors.

- [ ] **Step 6: Commit**
  ```bash
  git add apps/desktop-gpui/src/ui/ apps/desktop-gpui/src/main.rs
  git commit -m "feat(desktop-gpui): build 3-column workspace with server rail, channel tree, and chat canvas"
  ```

---

### Task 7: Full Verification, Cargo Tests & CI/CD GitHub Actions

**Files:**
- Create: `.github/workflows/rust-ci.yml`
- Modify: `docs/superpowers/plans/2026-11-16-rust-core-and-gpui-desktop.md`

- [ ] **Step 1: Run complete test suite**
  Run: `cargo test --all`
  Expected: All Rust unit and integration tests pass.

- [ ] **Step 2: Check formatting and linter**
  Run: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
  Expected: 0 warnings, clean formatting.

- [ ] **Step 3: Add GitHub Actions workflow for Rust**
  Create `.github/workflows/rust-ci.yml` for automated multi-platform builds (Linux, Windows, macOS).

- [ ] **Step 4: Final commit & push**
  ```bash
  git add .github/workflows/rust-ci.yml
  git commit -m "ci: add GitHub Actions workflow for Rust core and GPUI desktop"
  git push origin development
  ```
