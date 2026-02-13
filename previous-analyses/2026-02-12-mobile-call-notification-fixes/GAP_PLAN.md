# Gap Plan

## Completed checks

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: Starting calls in DM/GM channels auto-notifies peers with ringing event.
- Upstream source reference: `../mattermost-mobile/app/products/calls/state/actions.ts:78-132` (incoming-call/ringing conditions).
- Rustchat implementation reference: `backend/src/api/v4/calls_plugin/mod.rs:951-968`, `backend/src/api/v4/calls_plugin/mod.rs:3162-3176`, `backend/src/api/v4/calls_plugin/mod.rs:3193-3213`.
- Verification test reference: `backend/tests/api_calls_signaling.rs:131-179` (`calls_start_in_direct_channel_auto_rings_other_participants`).
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/state.rs`, `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: Dismissed notifications persist and round-trip via call state.
- Upstream source reference: `../mattermost-mobile/app/products/calls/actions/calls.ts:167-168`, `../mattermost-mobile/app/products/calls/state/actions.ts:95-98`.
- Rustchat implementation reference: `backend/src/api/v4/calls_plugin/state.rs:17-27`, `backend/src/api/v4/calls_plugin/state.rs:273-280`, `backend/src/api/v4/calls_plugin/mod.rs:671-717`, `backend/src/api/v4/calls_plugin/mod.rs:2086-2122`.
- Verification test reference: `backend/tests/api_calls_signaling.rs:609-654` (dismissed websocket + GET call-state assertions).
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: Config endpoint includes mobile-used fields.
- Upstream source reference: `../mattermost-mobile/app/products/calls/types/calls.ts:161-184`.
- Rustchat implementation reference: `backend/src/api/v4/calls_plugin/mod.rs:203-241`, `backend/src/api/v4/calls_plugin/mod.rs:484-502`.
- Verification test reference: `backend/tests/api_calls_signaling.rs:301-310` (`calls_mobile_channel_state_and_end_route_are_compatible` config assertions).
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: Emit `custom_com.mattermost.calls_call_state` full-state websocket event on key state mutations.
- Upstream source reference: `../mattermost-mobile/app/constants/websocket.ts:87`, `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:198-210`.
- Rustchat implementation reference: `backend/src/api/v4/calls_plugin/mod.rs:966-968`, `backend/src/api/v4/calls_plugin/mod.rs:1129-1131`, `backend/src/api/v4/calls_plugin/mod.rs:1210-1211`, `backend/src/api/v4/calls_plugin/mod.rs:2062-2063`, `backend/src/api/v4/calls_plugin/mod.rs:3215-3263`.
- Verification test reference: `backend/tests/api_calls_signaling.rs:438-449` and `backend/tests/api_calls_signaling.rs:595-608` (parseable call-state payload and dismissed state).
- Status: completed

## Compatibility checklist status

- API Contract: completed for changed routes/fields (`/config`, `/dismiss-notification`, `/calls/{channel_id}` response shape).
- Realtime Contract: completed for `calls_ringing` and `calls_call_state` names + payload keys.
- Data Semantics: completed for `dismissed_notification` map persistence and empty/default config field semantics.
- Auth and Permissions: no new bypass introduced; existing permission checks preserved on start/join/leave/host-change flows.
- Client Expectations: incoming-call ring gating and call-state sync behavior now align with mobile reducers.
- Verification: integration coverage added in `backend/tests/api_calls_signaling.rs`; suite passed.

## Remaining risks

- `calls_call_job_state` and `calls_caption` are still not emitted by RustChat (not in this scope).
- Recording/caption execution remains unsupported (existing explicit error behavior unchanged).
- No manual on-device mobile smoke pass was run in this turn.

## Test evidence

- Command: `cargo test --manifest-path backend/Cargo.toml --test api_calls_signaling`
- Result: passed (`5 passed; 0 failed`)
- Notes: non-fatal runtime warnings were observed in test logs (UDP port in use fallback, Redis stale payload warning), but assertions succeeded.
