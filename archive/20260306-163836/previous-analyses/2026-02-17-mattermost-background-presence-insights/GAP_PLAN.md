# GAP Plan (Rustchat vs Mattermost Behavior)

## P0 - Preserve manual statuses on websocket-driven offline transitions

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs` and `/Users/scolak/Projects/rustchat/backend/src/api/websocket_core.rs`.
- Required behavior:
  - Non-manual disconnect/offline transitions must not override manual busy/dnd/ooo.
- Verification test:
  - Add/update test where user is `busy`, websocket disconnect occurs, resulting status remains `busy`.
- Implemented change:
  - `handle_disconnect` now checks `presence_manual` before writing offline.
  - `initialize_connection_state` no longer forces `online` if status is manual.
- Rustchat evidence:
  - `/Users/scolak/Projects/rustchat/backend/src/api/websocket_core.rs`
  - `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
  - `/Users/scolak/Projects/rustchat/backend/src/api/ws.rs`
- Test evidence:
  - `/Users/scolak/Projects/rustchat/backend/tests/api_v4_mobile_presence.rs` (`websocket_disconnect_preserves_manual_status`)
- Status: Completed.

## P0 - Gate offline by active connection count (and cluster-aware count if applicable)

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`.
- Required behavior:
  - On websocket unregister, set offline only when user has zero active connections globally.
- Verification test:
  - Multi-connection test: one connection closes while another remains active; user must not go offline.
- Implemented change:
  - Added Redis-backed connection registry with `SADD`/`SREM`/`SCARD`.
  - Disconnect flow writes offline only when local count is zero and global `SCARD` is zero.
  - On Redis failure/count read failure, disconnect handling is conservative (no forced offline).
- Rustchat evidence:
  - `/Users/scolak/Projects/rustchat/backend/src/api/websocket_core.rs`
- Test evidence:
  - `/Users/scolak/Projects/rustchat/backend/tests/api_v4_mobile_presence.rs` (`user_stays_online_until_last_websocket_disconnects`)
- Status: Completed.

## P1 - Keep Mattermost-compatible reconnect semantics

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs` and mobile/websocket integration points.
- Required behavior:
  - Background mobile websocket closure should be treated as normal lifecycle; foreground should reconnect cleanly.
- Verification test:
  - Simulate background close and foreground reconnect; verify status/event ordering.
- Implemented change:
  - New connection handling now registers/unregisters global presence connection IDs.
  - Reconnect no longer inherently rewrites manual status to online.
- Rustchat evidence:
  - `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
  - `/Users/scolak/Projects/rustchat/backend/src/api/ws.rs`
  - `/Users/scolak/Projects/rustchat/backend/src/api/websocket_core.rs`
- Test evidence:
  - `/Users/scolak/Projects/rustchat/backend/tests/api_v4_mobile_presence.rs` (full test suite compile via `cargo test --test api_v4_mobile_presence --no-run`)
- Status: Completed (backend-side).

## P1 - Align websocket-state UX with reconnect events

- Rustchat target path: `/Users/scolak/Projects/rustchat/frontend` connection banner/status components.
- Required behavior:
  - "Connection restored" style indicators should appear on reconnect transitions, not on initial connect.
- Verification test:
  - UI state-machine tests for initial connect vs reconnect after disconnect.
- Status: Not implemented in this repository (no Rustchat React Native mobile client code present under `/Users/scolak/Projects/rustchat`).
- Remaining risk:
  - Mobile UX reconnect banner logic still depends on the external mobile app implementation.

## P2 - Add explicit compatibility regression suite

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/tests/api_v4_mobile_presence.rs` plus websocket integration tests.
- Required behavior:
  - Automated checks for:
    - manual status protection,
    - multi-connection offline gating,
    - reconnect after background close.
- Verification test:
  - CI test pass with deterministic websocket lifecycle tests.
- Implemented change:
  - Added/renamed coverage for:
    - non-manual disconnect -> offline,
    - manual disconnect -> no offline override,
    - multi-connection disconnect ordering.
- Test evidence:
  - `cargo test --lib websocket_core`
  - `cargo test --lib websocket_actor`
  - `cargo test --test api_v4_mobile_presence --no-run`
- Runtime blocker:
  - Full integration run requires local Postgres test DB (not available in this environment).
- Status: Completed with runtime-environment caveat.
