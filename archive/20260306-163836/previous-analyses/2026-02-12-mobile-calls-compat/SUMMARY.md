- Topic: mattermost-mobile <-> mattermost-mobile calls compatibility (Rustchat backend)
- Date: 2026-02-12
- Scope: server API + websocket contracts for Calls plugin paths consumed by mattermost-mobile

## Compatibility contract (implemented)

1. REST routes used by mobile exist and are non-404:
   - `GET /api/v4/plugins/com.mattermost.calls/channels?mobilev2=true`
   - `GET /api/v4/plugins/com.mattermost.calls/{channel_id}?mobilev2=true`
   - `POST /api/v4/plugins/com.mattermost.calls/{channel_id}` (enable/disable)
   - `POST /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/end`
   - `POST /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/host/screen-off`
   - `POST /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/recording/start`
   - `POST /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/recording/stop`

2. `/channels` and `/{channel_id}` now return mobile-compatible envelope shape:
   - `channel_id`, `enabled`, optional `call`
   - `call` payload includes sessions and key fields used by mobile state reducers
   - `/{channel_id}` returns `call: null` when no active call (instead of hard 404)

3. Calls websocket names/payloads now include mobile expected forms:
   - `custom_com.mattermost.calls_user_screen_on/off` + `session_id`
   - `custom_com.mattermost.calls_user_raise_hand/unraise_hand` + `session_id`, `raised_hand`
   - `custom_com.mattermost.calls_call_host_changed` + `hostID`
   - `custom_com.mattermost.calls_user_dismissed_notification` + `userID`, `callID`
   - Legacy aliases kept for existing rustchat consumers.

4. Calls config now includes mobile gating flags:
   - `EnableRinging`, `HostControlsAllowed`, `DefaultEnabled`, `AllowEnableCalls`, `GroupCallsAllowed`, `EnableRecordings`

## Open questions / risks

- Upstream calls server logic lives in external plugin package (`mattermost-plugin-calls`), not in `../mattermost/server` source tree, so plugin-internal semantics are inferred from mobile client contracts and available server/plugin wiring.
- Recording/captioning job execution remains unimplemented (compatibility routes now return explicit error).
- Channel enable/disable override persistence is in-memory only in current implementation.
