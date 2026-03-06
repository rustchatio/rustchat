---
name: mm-websocket-calls-parity
description: Parity workflow for websocket reconnect semantics and calls plugin event/payload compatibility with upstream mobile expectations.
license: MIT
---

# MM WebSocket and Calls Parity

Use this skill when changing websocket behavior, calls plugin routes, or custom websocket events.

## Trigger Conditions

- Changes in `backend/src/api/v4/websocket.rs`.
- Changes in `backend/src/api/v4/calls_plugin/**`.
- Changes in websocket event names/payload fields/sequence behavior.

## Required Inputs

- Analysis folder for current topic.
- Upstream websocket sources:
  - `../mattermost/server/channels/api4/websocket.go`
  - `../mattermost/server/channels/wsapi/**`
- Mobile calls websocket sources:
  - `../mattermost-mobile/app/products/calls/connection/websocket_client.ts`
  - `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`

## Workflow

1. Enumerate expected client actions and server events.
2. Verify connection_id/sequence/reconnect behavior parity.
3. Verify event names exactly (`custom_com.mattermost.calls_*`, typing, initial_load, etc.).
4. Verify payload field names/types and broadcast envelope shape.
5. Validate fallback behavior for unsupported actions and auth failures.
6. Update findings and gap register with exact source evidence.

## Expected Outputs

- `previous-analyses/<iteration>/SERVER_FINDINGS.md`
- `previous-analyses/<iteration>/MOBILE_FINDINGS.md`
- `previous-analyses/<iteration>/ARCHITECTURE_GAPS.md`
- `previous-analyses/<iteration>/GAP_REGISTER.md`

## Command Checklist

```bash
rg -n "websocket|connection_id|sequence_number|custom_com\.mattermost\.calls" \
  backend/src/api/v4 ../mattermost/server/channels ../mattermost-mobile/app/products/calls

cargo test --test api_calls_signaling -- --nocapture
cargo test --test api_v4_mobile_presence -- --nocapture
```

## Failure Handling

- If integration tests cannot run (DB/env), mark verification as blocked and capture exact blocker.
- If event payload parity is uncertain, add golden fixture tests before merge.

## Definition of Done

- Event names and payload keys are evidence-checked against upstream/mobile sources.
- Reconnect semantics are covered by automated test or documented manual trace.
- Any remaining mismatch is recorded as a gap with severity and owner.
