# Reconnect State Sync + Conservative Presence

- Topic: Mattermost-compatible reconnect behavior with conservative presence updates
- Date: 2026-02-17
- Scope: Rustchat backend websocket (`/api/v4/websocket`) + presence lifecycle

## Compatibility Contract

1. Presence must not flap to offline when any active connection still exists.
2. Manual presence must win over disconnect-driven offline transitions.
3. Reliable websocket reconnect uses `connection_id` + `sequence_number` and must preserve stream order.
4. Mobile reconnect flow is driven by `hello` and then REST `entry(...)` sync in Mattermost Mobile.
5. Rustchat should proactively provide a reconnect snapshot to reduce empty-list windows after reconnect.

## Observed Upstream Behavior

- Mattermost server only marks offline after confirming no active user websockets (including cluster), with conservative error handling.
- Mattermost server preserves manual status over non-manual offline updates.
- Mattermost mobile reconnects with reliable websocket query params, checks `hello.connection_id`, and triggers reconnect sync (`entry(...)`) when connection changes.
- Mattermost mobile background behavior closes websocket after ~15s.

## Open Questions

- Mattermost upstream does not expose a dedicated websocket `initial_load` event; Rustchat added one as an additive compatibility-safe optimization.

## SERVER_FINDINGS.md

- Endpoint or component:
- Source path:
- Source lines:
- Observed behavior:
- Notes:

## MOBILE_FINDINGS.md

- Screen, store, or service:
- Source path:
- Source lines:
- Observed behavior:
- Notes:

## GAP_PLAN.md

- Rustchat target path:
- Required behavior:
- Current gap:
- Planned change:
- Verification test:
- Status:
