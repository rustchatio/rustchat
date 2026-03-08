# Gap Plan

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs:947-960`, `backend/src/api/v4/calls_plugin/mod.rs:1091-1104`
  - Required behavior: `calls_user_joined` events must emit `session_id` in the same raw session format used by mobile websocket/session state.
  - Current gap: REST-driven start/join paths emitted encoded session IDs, while websocket mute/host control and mobile state indexing use raw UUID session IDs.
  - Planned change: Emit `participant.session_id.to_string()` (raw UUID) in both start-call and join-call `calls_user_joined` payloads.
  - Verification test: `backend/tests/api_calls_signaling.rs:57-67`, `backend/tests/api_calls_signaling.rs:80-92`, `backend/tests/api_calls_signaling.rs:425-457`, `backend/tests/api_calls_signaling.rs:482-491`.
  - Status: Completed.

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs:2064-2077`
  - Required behavior: Ringing endpoint must enforce channel membership like other call endpoints.
  - Current gap: `/calls/{channel_id}/ring` resolved channel and active call but did not enforce member permission.
  - Planned change: Add `check_channel_permission` before ringing broadcast.
  - Verification test: `backend/tests/api_calls_signaling.rs:184-228` (`ring_endpoint_requires_channel_membership`).
  - Status: Completed.

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs:2524-3017`
  - Required behavior: WS call actions should continue to work when websocket connection IDs churn or reconnect paths provide `originalConnID`.
  - Current gap: Action handlers were keyed strictly by current `connection_id`, causing `No active call found for connection` on mute/unmute/ICE/SDP in reconnect edge cases.
  - Planned change: Add `resolve_ws_session_uuid` (honor `originalConnID` when provided) and `resolve_call_for_ws_connection` fallback to existing participant session for the same user when unique.
  - Verification test: `cargo test --test api_calls_signaling -- --nocapture` (6 passed after patch; existing signaling/calls tests remain green).
  - Status: Completed.

- Rustchat target path: `frontend/src/api/calls.ts:209-215`, `frontend/src/api/calls.ts:238-244`
  - Required behavior: Normalized `owner_id`/`host_id` used in UI permission checks must be raw UUIDs.
  - Current gap: Normalization preferred encoded wire IDs, breaking host/owner comparisons against `authStore.user.id`.
  - Planned change: Prefer `*_raw` values for normalized `owner_id` and `host_id`.
  - Verification test: Manual code-path verification in `frontend/src/components/calls/ActiveCall.vue:73-76` (comparison uses raw `authStore.user.id`).
  - Status: Completed.

- Rustchat target path: `backend/tests/api_calls_signaling.rs`
  - Required behavior: Prevent regressions where session IDs are emitted in mixed encodings across events.
  - Current gap: Tests validated event names but did not enforce raw UUID `session_id` for `calls_user_joined`.
  - Planned change: Add assertions that `session_id` parses as UUID and matches call-state session IDs.
  - Verification test: `cargo test --test api_calls_signaling -- --nocapture` (5 passed).
  - Status: Completed.

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs` and `backend/src/api/v4/calls_plugin/state.rs`
  - Required behavior: Preserve mobile-compatible call state/config contracts.
  - Current gap: None discovered for currently enabled features during this iteration; config and dismissed tracking already present.
  - Planned change: No code change required in this step.
  - Verification test: `cargo test --test api_calls_signaling -- --nocapture` (config and dismissed-notification checks pass).
  - Status: Verified.

- Remaining risks:
  - Recording/captioning remain disabled by design, so `custom_com.mattermost.calls_call_job_state` and `custom_com.mattermost.calls_caption` are not actively emitted in this implementation path.
  - Mixed old Redis call-state payloads can log decode warnings (`missing field host_id`) until stale keys are replaced.
