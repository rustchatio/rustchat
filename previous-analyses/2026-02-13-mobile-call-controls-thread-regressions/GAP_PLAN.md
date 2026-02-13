# Gap Plan

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs:3678`
  - Required behavior: calls signaling events must satisfy mobile websocket client gating and peer parser (`connID` + serialized `data`).
  - Current gap: server signaling payload lacked `connID/conn_id` and serialized `data`, so mobile dropped/ignored signaling events.
  - Planned change: include `connID`, `conn_id`, `data`, and `signal` in `custom_com.mattermost.calls_signal` payload.
  - Verification test: `backend/tests/api_calls_signaling.rs:258` and `backend/tests/api_calls_signaling.rs:261` plus compile (`cargo test --manifest-path backend/Cargo.toml --test api_calls_signaling --no-run`).
  - Status: Completed.

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs:759`
  - Required behavior: every active call must have a thread root id usable by mobile call-thread navigation.
  - Current gap: calls could exist with `thread_id: None`, causing undefined `threadId` in mobile state and thread-query exceptions.
  - Planned change: create call thread post when missing and persist via call state manager.
  - Verification test: `backend/tests/api_calls_signaling.rs:325` and `backend/tests/api_calls_signaling.rs:355` plus compile-only calls target.
  - Status: Completed.

- Rustchat target path: `backend/src/api/v4/calls_plugin/state.rs:274`
  - Required behavior: thread id must be persisted in mutable call state.
  - Current gap: no explicit mutator for setting thread id after call creation.
  - Planned change: add `set_thread_id(call_id, Option<Uuid>)` to state manager.
  - Verification test: compile-only calls test target.
  - Status: Completed.

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs:2635`
  - Required behavior: websocket join/reconnect should publish full state for mobile consistency.
  - Current gap: ws join path emitted `user_joined` but did not emit `calls_call_state` snapshot.
  - Planned change: broadcast `calls_call_state` after ws join handling.
  - Verification test: compile-only calls test target.
  - Status: Completed.

## Compatibility Gate Update

- API Contract
  - Method/route compatibility for existing calls endpoints remains unchanged: Completed.
  - Response shape for calls signaling and call state updated to mobile-required fields: Completed.
- Realtime Contract
  - Event names unchanged; payload shape for `calls_signal` and call thread state hardened: Completed.
  - Join path now emits state snapshot: Completed.
- Data Semantics
  - `thread_id` no longer omitted for threadless calls; server now backfills by creating a call thread post: Completed.
- Auth/Permissions
  - No changes in auth semantics in this iteration: Verified.
- Client Expectations
  - Mobile `connID` gating and thread routing assumptions are now explicitly satisfied by server payloads: Completed.

## Test Evidence

- `cargo test --manifest-path backend/Cargo.toml --test api_calls_signaling --no-run`
  - Result: Pass (build succeeds for calls integration target).
- `cargo test --manifest-path backend/Cargo.toml --test api_calls_signaling`
  - Result: Could not validate runtime due local Postgres unavailable (`Connection refused` in test harness).

## Remaining Risks

- End-to-end runtime verification on a live mobile client + running Postgres-backed integration environment is still required.
- Existing in-flight calls created before this patch may retain missing thread ids until refreshed by state rebuild paths.
