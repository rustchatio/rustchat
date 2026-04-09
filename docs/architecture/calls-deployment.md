# Calls State Modes and Limits

This document defines how the Calls subsystem behaves in single-node and multi-node deployments.

## Configuration

Set `RUSTCHAT_CALLS_STATE_BACKEND`:

- `memory`: single-node mode. Call state is process-local only.
- `redis`: multi-node mode. Call state is persisted to Redis and shared across instances.
- `auto` (default): use Redis when available; if Redis is unavailable for an operation, fall back to local in-memory state for that operation.

`RUSTCHAT_REDIS_URL` must point to the shared Redis instance in multi-node deployments.

## Data Model in Redis

When Redis mode is active, calls plugin state is stored as:

- `rustchat:calls:state:<call_id>`: serialized call state JSON.
- `rustchat:calls:channel:<channel_id>`: `call_id` lookup.
- `rustchat:calls:active`: set of active call IDs.

## Single-Node Behavior (`memory`)

- All call state is in process memory.
- Restarting the backend clears active call state.
- Suitable for development and one-instance deployments.
- No cross-instance visibility.

## Multi-Node Behavior (`redis`/`auto`)

- Read/write operations use Redis-backed call state.
- Multiple backend instances can observe the same active calls.
- SFU peer connections remain instance-local; this baseline shares control-plane call metadata only.

## Fallback Semantics (`auto` and Redis operation failures)

- If Redis is unavailable during an operation, handlers continue with local memory state for continuity.
- This allows degraded single-node behavior instead of hard failure.
- During degraded operation, consistency across instances is not guaranteed.

## WebSocket Signaling Delivery

Server-generated signaling messages are emitted as websocket event:

- Event: `custom_com.mattermost.calls_signal`
- Delivery: direct to the target user via websocket hub
- Payload fields:
  - `channel_id`, `channel_id_raw`
  - `user_id`, `user_id_raw`
  - `session_id`, `session_id_raw`
  - `signal` (`answer`, `ice-candidate`, `ice-state`, `connection-state`, etc.)

## Current Limits in This Baseline

- No distributed SFU media plane yet; SFU sessions are local to the instance that owns the peer connection.
- Redis operations are simple last-write-wins updates (no compare-and-swap conflict handling).
- ICE candidate processing is complete for add/queue/flush flow, but advanced ICE retries/backoff policies are not yet implemented.

## Test Coverage

- Integration tests cover call lifecycle websocket events and signaling delivery.
- Unit tests cover ICE candidate parsing and call state backend mode parsing.
