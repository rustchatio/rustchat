- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: expose all mobile-used calls routes
- Current gap: no routes for `/{channel_id}` POST (enable/disable), `/calls/{channel_id}/end`, `/calls/{channel_id}/host/screen-off`, `/calls/{channel_id}/recording/start`, `/calls/{channel_id}/recording/stop`
- Planned change: add handlers and route registrations with MM-compatible status/JSON behavior
- Verification test: `backend/tests/api_calls_signaling.rs` (`calls_mobile_channel_state_and_end_route_are_compatible`, `calls_mobile_event_names_and_payloads_are_compatible`)
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: `/channels` and `/{channel_id}?mobilev2=true` return `CallChannelState` compatible shape
- Current gap: `/channels` returned custom metadata (`call_id`, `has_call`) and `/{channel_id}` returned direct call object/404
- Planned change: return envelope with `channel_id`, `enabled`, optional `call`; mobile path returns `call: null` when idle
- Verification test: `backend/tests/api_calls_signaling.rs` (`calls_mobile_channel_state_and_end_route_are_compatible`)
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: websocket event names and payload keys match mobile listeners
- Current gap: mismatched names (`calls_screen_on/off`, `calls_raise_hand/lower_hand`, `calls_host_changed`) and missing dismissed-notification event
- Planned change: emit mobile-expected event names/keys while keeping legacy aliases
- Verification test: `backend/tests/api_calls_signaling.rs` (`calls_mobile_event_names_and_payloads_are_compatible`)
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: calls config includes mobile gating flags
- Current gap: config response only included `ICEServersConfigs` and `NeedsTURNCredentials`
- Planned change: include `EnableRinging`, `HostControlsAllowed`, `DefaultEnabled`, `AllowEnableCalls`, `GroupCallsAllowed`, `EnableRecordings`
- Verification test: `backend/tests/api_calls_signaling.rs` (`calls_mobile_channel_state_and_end_route_are_compatible`)
- Status: completed

- Rustchat target path: `backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: voice activity websocket events target channel IDs (not call IDs)
- Current gap: `calls_user_voice_on/off` broadcasts were keyed by `call_id`
- Planned change: resolve call state and broadcast to `call.channel_id`
- Verification test: covered by existing lifecycle/signaling suite passing (`backend/tests/api_calls_signaling.rs`)
- Status: completed

## Remaining risks

- Recording/captioning jobs are still unsupported (routes now exist and return explicit error rather than 404).
- Channel enable/disable overrides are process-memory only (`DashMap`), not persisted in DB.
- One unrelated existing integration test suite still reports a non-calls failure in this workspace run:
  - `cargo test --manifest-path backend/Cargo.toml --test api_v4_plugins_dialogs`
  - failure observed: expected `501`, received `415` in `plugin_mutations_return_explicit_mm_501`.

## Test evidence

- Passed: `cargo test --manifest-path backend/Cargo.toml --test api_calls_signaling`
  - `calls_lifecycle_events_are_delivered_over_websocket`
  - `offer_generates_server_signaling_event_over_websocket`
  - `calls_mobile_channel_state_and_end_route_are_compatible`
  - `calls_mobile_event_names_and_payloads_are_compatible`
