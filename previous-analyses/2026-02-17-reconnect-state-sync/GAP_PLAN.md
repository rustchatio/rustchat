# Gap Plan and Verification

## Item 1: Reconnect full-state snapshot over websocket

- Rustchat target path: `backend/src/api/v4/websocket.rs`
- Required behavior:
  - On reconnect, push complete state (channels, channel membership/unreads, user statuses) immediately.
- Current gap:
  - Only `hello` + replay existed; no proactive snapshot payload.
- Planned/implemented change:
  - Added `initial_load` websocket event with:
    - `channels`
    - `channel_members`
    - `channel_unreads` (msg_count + mention_count placeholder)
    - `statuses`
  - Added explicit handling of client actions `reconnect|get_initial_load|initial_load` to trigger snapshot.
  - Snapshot messages are sequence-numbered and queued into `connection_store` to preserve reliable websocket ordering.
- Verification test:
  - `cargo test --lib api::v4::websocket` (new unit test: `reconnect_snapshot_trigger_matches_resume_signals`)
- Status: Completed

## Item 2: Reliable reconnect detection contract

- Rustchat target path: `backend/src/api/v4/websocket.rs`
- Required behavior:
  - Trigger reconnect snapshot only for reconnect-like sessions (`connection_id` present or sequence > 0).
- Current gap:
  - No dedicated reconnect trigger logic.
- Planned/implemented change:
  - Added `should_send_reconnect_snapshot(...)` helper and wired call after hello/replay.
- Verification test:
  - `cargo test --lib api::v4::websocket`
- Status: Completed

## Item 3: Presence compatibility checks (manual + conservative offline)

- Rustchat target path: `backend/src/api/websocket_core.rs`, `backend/src/api/v4/websocket.rs`, `backend/src/api/ws.rs`, `backend/src/realtime/websocket_actor.rs`
- Required behavior:
  - Manual status precedence and offline only when global connection count is zero.
  - 30s heartbeat with 2-miss timeout.
- Current gap:
  - Previously implemented in this iteration’s prior patch set; revalidated here.
- Verification test:
  - `cargo test --lib websocket_core`
  - `cargo test --test api_v4_mobile_presence --no-run`
- Status: Completed

## Remaining Risks

1. Mattermost mobile upstream currently relies on REST `entry(...)` after reconnect rather than consuming a websocket `initial_load` event; this Rustchat event is additive and safe, but may be unused by upstream clients.
2. `mention_count` in `channel_unreads` is currently `0` (no direct mention aggregation in current schema path).
