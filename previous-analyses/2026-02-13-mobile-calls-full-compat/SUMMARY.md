# Summary

- Topic: Mattermost Mobile calls compatibility hardening (session/mute/state contract).
- Date: 2026-02-13.
- Scope: Calls plugin REST + websocket compatibility for `mattermost-mobile` clients, with emphasis on participant session identity and mute/host-control behavior.

## Compatibility Contract

- REST surface expected by mobile:
  - `../mattermost-mobile/app/products/calls/client/rest.ts:41-170` defines `/channels?mobilev2=true`, `/{channel_id}?mobilev2=true`, `/config`, `/version`, `/turn-credentials`, `/calls/{id}/end`, `/dismiss-notification`, host control routes, and recording routes.
- WS events expected by mobile:
  - `../mattermost-mobile/app/constants/websocket.ts:65-87` enumerates calls events (`user_joined`, `user_muted`, `call_state`, `host_mute`, etc.).
- Session identity must be stable and raw:
  - `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:54-71` consumes `msg.data.session_id` directly.
  - `../mattermost-mobile/app/products/calls/state/actions.ts:327-342` indexes sessions by `sessionId`.
  - `../mattermost-mobile/app/products/calls/state/actions.ts:531-535` mute updates are ignored if that `sessionId` key does not exist.
- Call-state payload requirements used by mobile conversion:
  - `../mattermost-mobile/app/products/calls/actions/calls.ts:143-168` expects `sessions[].session_id`, `unmuted`, `dismissed_notification`, `screen_sharing_session_id`, `owner_id`, `host_id`.
- Config fields mobile expects:
  - `../mattermost-mobile/app/products/calls/types/calls.ts:161-184` defaults include `MaxCallParticipants`, `AllowScreenSharing`, `EnableSimulcast`, `EnableAV1`, `MaxRecordingDuration`, `TranscribeAPI`, `sku_short_name`, `EnableDCSignaling`, `EnableTranscriptions`, `EnableLiveCaptions`.

## Open Questions

- Upstream Mattermost monorepo does not contain the calls plugin server implementation in this checkout, so mobile source and RustChat integration tests are the primary compatibility oracle for runtime payload shapes.

## Implemented In This Iteration

- Ensured `calls_user_joined` uses raw `session_id` (REST start/join paths).
- Added channel membership enforcement to `/calls/{channel_id}/ring`.
- Hardened WS calls action session resolution across reconnect/connection-id mismatch paths (`mute`, `unmute`, `sdp`, `ice`, `leave`, `raise_hand`, `react`).
- Fixed frontend calls normalization to prefer raw `owner_id`/`host_id` for host gating.
