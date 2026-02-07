# WebSocket Architecture (v1 + v4)

## Goal

Reduce complexity and drift between `/api/v1/ws` and `/api/v4/websocket` while keeping both client contracts stable.

## Endpoint Responsibilities

| Endpoint | Primary clients | Wire protocol | Adapter-specific responsibilities |
|---|---|---|---|
| `/api/v1/ws` | RustChat web app and legacy internal clients | Internal envelope (`type`, `event`, `data`, `channel_id`) | Query/protocol token handshake, envelope hello (`WsEnvelope::hello`) |
| `/api/v4/websocket` | Mattermost-compatible clients (mobile/desktop) | Mattermost websocket message shape (`event`, `data`, `broadcast`, `seq`) | Optional auth challenge exchange, MM event mapping, session resumption (`connection_id`, `sequence_number`) |

## Shared Core (Strict Boundary)

Both adapters now use `backend/src/api/websocket_core.rs` for:

1. Auth token normalization and extraction (`query`, `Authorization`, optional protocol fallback).
2. Connection limit enforcement.
3. Connection bootstrap:
1. default team/channel subscriptions (policy controlled per adapter),
2. presence transition to `online`.
4. Presence teardown when the user loses their last connection (`offline` transition).
5. Shared envelope command handling:
1. `subscribe_channel`,
2. `unsubscribe_channel`,
3. `typing` / `typing_start` / `typing_stop`,
4. `presence`,
5. `ping` -> `pong`,
6. optional `send_message` for v1.

## Contract Boundaries

### Stable adapter behavior

1. v1 keeps envelope-based transport and keeps returning HTTP `401`/`429` on handshake failures.
2. v4 keeps Mattermost-style framing and keeps auth-challenge/session-resume behavior.
3. v4 keeps mapping internal hub events to Mattermost event names (`posted`, `typing`, `post_edited`, etc.).

### Shared behavior guarantees

1. Connection limits are enforced the same way for both endpoints.
2. Team/channel subscription bootstrap is centralized and explicit.
3. Presence lifecycle (`online` on connect, `offline` when last connection closes) is consistent.
4. Envelope commands that are shared by both adapters are processed by one implementation.

## Event Contract Comparison

### Client -> server

| Category | v1 (`/api/v1/ws`) | v4 (`/api/v4/websocket`) |
|---|---|---|
| Auth at handshake | token query/protocol/auth header | token query/auth header |
| Auth after connect | n/a | `action=authentication_challenge` |
| Subscribe | `event=subscribe_channel` | same envelope command |
| Unsubscribe | `event=unsubscribe_channel` | same envelope command |
| Typing | `event=typing` / `typing_start` / `typing_stop` | envelope typing + MM `action=user_typing` |
| Presence | `event=presence` | same envelope command |
| Ping | `event=ping` | same envelope command + protocol ping/pong frames |

### Server -> client

| Category | v1 (`/api/v1/ws`) | v4 (`/api/v4/websocket`) |
|---|---|---|
| Hello | internal `hello` envelope | MM `hello` with `connection_id`, protocol/server version |
| Real-time events | internal envelope events | mapped MM events (`posted`, `typing`, `status_change`, etc.) |
| Targeting | shared hub broadcast rules | shared hub broadcast rules + MM broadcast serialization |

## Test Coverage Added

1. Auth challenge parsing tests in `backend/src/api/v4/websocket.rs`.
2. Reconnection safety tests in `backend/src/realtime/connection_store.rs`.
3. Event delivery routing tests in `backend/src/realtime/hub.rs`.
