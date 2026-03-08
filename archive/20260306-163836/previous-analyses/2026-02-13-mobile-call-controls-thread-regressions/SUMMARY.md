# Mobile Call Controls/Thread Regressions Analysis

- Topic: Mattermost Mobile call controls, connection signaling, speaking status, and call-thread compatibility.
- Date: 2026-02-13.
- Scope: `mattermost-mobile` behavior + Rustchat calls plugin websocket/REST payload compatibility.

## Compatibility Contract

- Calls signaling websocket events consumed by mobile calls connection must include `data.connID` matching current/original connection id, otherwise messages are dropped.
  - Evidence: `../mattermost-mobile/app/products/calls/connection/websocket_client.ts:108`.
- Calls signaling payload consumed by mobile peer logic must include `data.data` as serialized JSON for `peer.signal(...)`.
  - Evidence: `../mattermost-mobile/app/products/calls/connection/connection.ts:450`.
- Call thread navigation assumes `currentCall.threadId` exists and is passed directly to thread screen routing.
  - Evidence: `../mattermost-mobile/app/products/calls/screens/call_screen/call_screen.tsx:485`.
- Mobile call-state mapping reads `call.thread_id` and stores it as `threadId`; missing key produces `undefined` in call state.
  - Evidence: `../mattermost-mobile/app/products/calls/actions/calls.ts:163`.
- Mobile follow-thread behavior reads `call.threadId` and queries thread state during call lifecycle.
  - Evidence: `../mattermost-mobile/app/products/calls/state/actions.ts:507`.

## Server Contract (Rustchat)

- Calls state must provide a stable, non-missing thread id for active calls.
- Calls signaling events generated from SFU-forwarded messages must include mobile-compatible `connID` and serialized `data` fields.
- Join/reconnect flows should publish a full `calls_call_state` snapshot to reduce state drift in mobile reducers.

## Implemented in Rustchat

- Added call-thread post creation and persistence for calls missing `thread_id`.
  - `backend/src/api/v4/calls_plugin/mod.rs:759`
  - `backend/src/api/v4/calls_plugin/mod.rs:822`
  - `backend/src/api/v4/calls_plugin/state.rs:274`
- Ensured `thread_id` is populated for REST start and websocket-created calls.
  - `backend/src/api/v4/calls_plugin/mod.rs:986`
  - `backend/src/api/v4/calls_plugin/mod.rs:2719`
- Added `calls_call_state` broadcast on websocket join path.
  - `backend/src/api/v4/calls_plugin/mod.rs:2756`
- Updated SFU signaling websocket payload to include `connID`/`conn_id` and serialized `data` plus structured `signal`.
  - `backend/src/api/v4/calls_plugin/mod.rs:3713`

## Open Questions

- Runtime integration tests requiring a local Postgres instance could not be executed in this workspace (connection refused), so end-to-end behavior remains to be re-verified with DB available.
