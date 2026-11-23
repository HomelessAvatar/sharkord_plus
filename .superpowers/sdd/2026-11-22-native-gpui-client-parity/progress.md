# SDD ledger — plan: docs/superpowers/plans/2026-11-22-native-gpui-client-parity.md

## Pre-flight Conflict Scan

| Pair / Task | Produces vs Consumes | Findings | Ruling |
|---|---|---|---|
| Task 1 -> Tasks 5-15 | `gpui-kit`, `rfd`, `webrtc`, `xcap`, `openh264` dependencies | Dependency crates match toolchain and workspace settings | Clean |
| Task 2 -> Tasks 4-12 | `SharkordClient` tRPC methods & models | Signatures match server endpoints in `apps/server/src/routers` | Clean |
| Task 3 -> Task 6 | `EventType` variants & WebSocket frames | Event variants match `events.ts` server publishers | Clean |
| Task 4 -> Tasks 5-15 | `NetworkBridge` & Tokio runtime in GPUI | Uses `tokio::sync::mpsc` and `cx.update(...)` safely | Clean |
| Task 7 -> Tasks 8, 9 | `VirtualList` feed, `Textarea` composer | Modular files under `apps/desktop-gpui/src/ui/` | Clean |
| Task 10 -> Task 11 | Direct messaging & user profiles | Adheres to `is_dm: true` 1-on-1 channel model | Clean |
| Task 13, 14 -> Existing VoiceBar/Stage/Tiles | Live WebRTC audio & GPU video textures | Clean integration with existing UI prototypes | Clean |

Pre-flight scan complete and clean. Proceeding to task execution.

- Task 1: complete (commits 128f211..ea8f842, review clean)
- Task 2: fix round 1/5 (2 addressed, 0 open — workspace tests & route precedence; commits ea8f842..62adbb2)
- Task 2: complete (commits ea8f842..62adbb2, review clean)
- Task 3: complete (commits 62adbb2..e368bb7, review clean)
- Task 4: complete (commits e368bb7..5ecbe12, review clean)
- Task 5: fix round 1/5 (1 addressed, 0 open — MSVC duplicate manifest linker conflict resolved; commits 052d028..9858f2c)
- Task 5: complete (commits 5ecbe12..9858f2c, review clean)
- Task 6: complete (commits 9858f2c..d04e0f4, review clean)
- Task 7: complete (commits d04e0f4..00037ea, review clean)
- Task 8: complete (commits 00037ea..fdd50dc, review clean)
- Task 9: complete (commits 0c417a1..74b2c3f, review clean)
- Task 10: complete (commits 74b2c3f..b9568d7, review clean)
- Task 11: complete (commits b9568d7..f79051e, review clean)
- Task 12: complete (commits f79051e..bbf17fa, review clean)
- Task 13: complete (commits fda30c4..ba581e8, review clean)
- Task 14: complete (commits 615b07b..468787c, review clean)
- Task 15: complete (commits 9a8150f..6bbc38c, review clean)




