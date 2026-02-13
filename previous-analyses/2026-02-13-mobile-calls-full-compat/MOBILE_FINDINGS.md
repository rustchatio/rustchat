# Mobile Findings

- Screen, store, or service: Calls REST client contract.
  - Source path: `../mattermost-mobile/app/products/calls/client/rest.ts`
  - Source lines: `41-170`
  - Observed behavior: Mobile depends on calls plugin endpoints for list/state/config/version, turn credentials, dismiss, host controls, and recording start/stop.
  - Notes: RustChat routes must keep path/method compatibility even when some features are disabled.

- Screen, store, or service: Calls websocket event routing.
  - Source path: `../mattermost-mobile/app/constants/websocket.ts`
  - Source lines: `65-87`
  - Observed behavior: Mobile recognizes specific event names, including `custom_com.mattermost.calls_call_state`.
  - Notes: Event-name drift breaks handlers immediately.

- Screen, store, or service: Calls websocket handlers.
  - Source path: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
  - Source lines: `54-71`, `165-170`, `198-210`
  - Observed behavior: `session_id` is consumed as-is for join/mute/unmute and host controls; call-state JSON is parsed and applied.
  - Notes: `host_mute` logic compares `msg.data.session_id` to `currentCall.mySessionId`; mismatched encoding prevents expected mute behavior.

- Screen, store, or service: Calls state reducer behavior.
  - Source path: `../mattermost-mobile/app/products/calls/state/actions.ts`
  - Source lines: `327-342`, `531-535`
  - Observed behavior: sessions map is keyed by `sessionId`; mute updates no-op when key is absent.
  - Notes: If one event uses encoded IDs and another uses raw UUIDs, participant state diverges and mute indicators stop updating.

- Screen, store, or service: API to in-app call mapping.
  - Source path: `../mattermost-mobile/app/products/calls/actions/calls.ts`
  - Source lines: `143-168`
  - Observed behavior: Mobile maps `call.sessions[*].session_id`, `unmuted`, and `dismissed_notification` into internal call state.
  - Notes: Backend session identity and field naming must be consistent across WS and REST.

- Screen, store, or service: Calls config defaults expected by UI.
  - Source path: `../mattermost-mobile/app/products/calls/types/calls.ts`
  - Source lines: `161-184`
  - Observed behavior: UI references advanced config flags and limits beyond the minimal calls fields.
  - Notes: Missing fields can disable features or alter behavior guards.
