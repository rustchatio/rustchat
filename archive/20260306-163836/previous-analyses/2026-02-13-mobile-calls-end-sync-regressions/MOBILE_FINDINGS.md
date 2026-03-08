# Mobile Findings

1. End call permissions
- Mobile allows ending a call when user is host OR system admin (`canEndCall`).
- Evidence: `../mattermost-mobile/app/products/calls/actions/calls.ts:358-375`

2. Calls control transport
- Unmute, raise-hand, unraise-hand, react are sent on calls websocket actions, not REST.
- Evidence: `../mattermost-mobile/app/products/calls/connection/connection.ts:205-229`

3. Event field dependencies
- Realtime call updates are keyed by `broadcast.channel_id` + `data.session_id` and consumed by reducers:
  - mute/unmute: `setUserMuted(...)`
  - raised hand: `setRaisedHand(...)`
  - reaction: `userReacted(...)`
- Evidence: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:70-129`

4. Reaction payload expectations
- `userReacted` logic uses `reaction.emoji.name`, `reaction.session_id`, and `reaction.timestamp` for stream ordering and timeout cleanup.
- Evidence: `../mattermost-mobile/app/products/calls/state/actions.ts:732-799`

5. Host changes
- Host-change websocket events feed `setHost(...)` with `data.hostID`.
- Evidence: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:142-144`
