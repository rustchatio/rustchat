# Summary

- Topic: Calls ringing + call-state mobile compatibility fixes
- Date: 2026-02-12
- Scope: Mattermost-mobile consumed Calls REST + websocket behavior in RustChat backend

## Compatibility contract

1. `custom_com.mattermost.calls_call_start` must carry call identifiers and trigger mobile incoming-call processing.
   Evidence: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:81-93`, `../mattermost-mobile/app/products/calls/state/actions.ts:476-483`.

2. Incoming ringing is only shown when the call is DM/GM, ringing is enabled, caller is not self, and call is not dismissed for the user.
   Evidence: `../mattermost-mobile/app/products/calls/state/actions.ts:78-132`.

3. Dismissed state must be persisted in server call state and returned in `dismissed_notification` so mobile can suppress repeated ringing.
   Evidence: `../mattermost-mobile/app/products/calls/actions/calls.ts:167`, `../mattermost-mobile/app/products/calls/state/actions.ts:95-98`, `../mattermost-mobile/app/products/calls/state/actions.ts:185-190`.

4. Calls websocket stream should include `custom_com.mattermost.calls_call_state` with serialized full call payload in `data.call`.
   Evidence: `../mattermost-mobile/app/constants/websocket.ts:87`, `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:198-210`.

5. Calls config payload must include mobile-used gating fields:
   `sku_short_name`, `MaxCallParticipants`, `AllowScreenSharing`, `EnableSimulcast`, `EnableRinging`, `EnableTranscriptions`, `EnableLiveCaptions`, `EnableAV1`, `TranscribeAPI`, `EnableDCSignaling`, `MaxRecordingDuration`.
   Evidence: `../mattermost-mobile/app/products/calls/types/calls.ts:161-184`.

## Open questions

- Upstream calls server internals are packaged as external plugin and are not present in `../mattermost/server`; server-side contract is inferred from mobile consumer behavior and existing RustChat behavior.
