# Server Findings

- Endpoint or component: Webapp calls config consumption in upstream Mattermost monorepo.
  - Source path: `../mattermost/webapp/channels/src/selectors/calls.ts`
  - Source lines: `25-28`, `38-44`
  - Observed behavior: Calls plugin config is consumed via `callsConfig`, and `EnableRinging` is used as a first-class server capability flag.
  - Notes: Confirms server-provided calls config fields are part of client behavior gates.

- Endpoint or component: Upstream calls plugin backend source availability in this checkout.
  - Source path: `../mattermost/server` and `../mattermost/webapp/channels/src/plugins`
  - Source lines: N/A (search evidence)
  - Observed behavior: No calls plugin backend implementation files were found in this monorepo snapshot for direct endpoint/event payload confirmation.
  - Notes: Mobile repo and RustChat integration tests are used as authoritative runtime contract sources for this iteration.

- Endpoint or component: RustChat calls config response parity.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `204-240`, `486-505`
  - Observed behavior: RustChat now returns mobile-expected config fields (`MaxCallParticipants`, `AllowScreenSharing`, `EnableSimulcast`, `EnableAV1`, `TranscribeAPI`, `sku_short_name`, `EnableDCSignaling`, `EnableTranscriptions`, `EnableLiveCaptions`, etc.).
  - Notes: Matches expected field presence from mobile defaults.

- Endpoint or component: RustChat call-state and dismissed tracking.
  - Source path: `backend/src/api/v4/calls_plugin/state.rs`
  - Source lines: `17-27`, `273-277`
  - Observed behavior: `dismissed_users` is part of call state and persisted via the call state manager mutation path.
  - Notes: Supports `dismissed_notification` population in responses/events.

- Endpoint or component: RustChat call-state serialization and call_state event.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `690-747`, `3238-3285`
  - Observed behavior: Sessions and `dismissed_notification` are serialized, and `custom_com.mattermost.calls_call_state` is broadcast with serialized `call` JSON.
  - Notes: Provides full-state sync payload expected by mobile handlers.

- Endpoint or component: RustChat ring endpoint authorization.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `2064-2077`
  - Observed behavior: Ringing now enforces `check_channel_permission` before broadcasting.
  - Notes: Aligns ring endpoint access control with other calls routes.

- Endpoint or component: RustChat WS session resolution for calls actions.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `2524-3017`
  - Observed behavior: Calls WS actions now resolve session IDs via `originalConnID` when present and fall back to the existing participant session for the same user when unique.
  - Notes: Addresses reconnect/connection-id mismatch failures reported in runtime logs (`No active call found for connection`).
