# Server Findings

## Upstream server availability

- The `../mattermost` repository in this workspace does not include the calls plugin server implementation symbols needed for direct line-by-line behavior extraction (`calls_call_host_changed`, calls `/end` handler internals, etc.).
- Practical contract source for this iteration is Mattermost Mobile client behavior + Rustchat implementation/test compatibility.

## Rustchat behavior observed

1. End-call authorization path
- `backend/src/api/v4/calls_plugin/mod.rs:1345-1395` gates `/calls/{channel_id}/end`.
- Added stale-host/session recovery via `is_host_session_active(...)` and caller fallback when host session is inactive.

2. Host/admin permission model
- `backend/src/api/v4/calls_plugin/mod.rs:334-340` (`is_system_admin`, `can_manage_call`).
- Host-control routes already normalize stale host and use host/admin checks in current code path.

3. Reaction payload shape
- REST reaction emitter includes `session_id`, `reaction`, `timestamp`, `emoji` object at `backend/src/api/v4/calls_plugin/mod.rs:1558-1599`.
- Websocket reaction emitter includes same fields at `backend/src/api/v4/calls_plugin/mod.rs:3231-3275`.

4. Abrupt websocket disconnect cleanup
- Best-effort participant removal and follow-up host/state reconciliation is implemented at `backend/src/api/v4/calls_plugin/mod.rs:3079-3118`.

## Risk noted

- If host has multiple active sessions/connections, host-session activity heuristics can still be edge-casey; this should be validated with multi-device/manual scenarios.
