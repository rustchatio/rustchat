# Mobile Calls End + Sync Regressions

- Topic: Mattermost Mobile calls controls and live sync regressions in Rustchat.
- Date: 2026-02-13
- Scope: Calls plugin API + websocket compatibility for host/end-call, mute/unmute, raised hand, reactions, and live channel/post sync.

## Compatibility Contract

1. End-call permissions must support host and system admin paths used by mobile (`canEndCall`) and must not deadlock when host ownership/session is stale.
   - Mobile evidence: `../mattermost-mobile/app/products/calls/actions/calls.ts:358-375`
   - Rustchat endpoint: `backend/src/api/v4/calls_plugin/mod.rs:1345-1395`

2. Mute/unmute/raise-hand/reaction controls are sent over calls websocket actions (`unmute`, `raise_hand`, `unraise_hand`, `react`) and require realtime event payloads with fields mobile reducers consume.
   - Mobile outbound actions: `../mattermost-mobile/app/products/calls/connection/connection.ts:205-229`
   - Mobile websocket handlers: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:119-129`

3. Reaction events must carry `session_id`, `emoji`, and timing fields (`timestamp`) so mobile `userReacted` state updates and expiry logic work.
   - Mobile consumer: `../mattermost-mobile/app/products/calls/state/actions.ts:732-799`
   - Rustchat emitters: `backend/src/api/v4/calls_plugin/mod.rs:1558-1599`, `backend/src/api/v4/calls_plugin/mod.rs:3231-3275`

4. Calls must recover when websocket closes abruptly (common mobile behavior) so participant/host state does not remain stale.
   - Runtime symptom: repeated websocket reset logs + 403 on `/calls/{channel_id}/end`
   - Rustchat cleanup path: `backend/src/api/v4/calls_plugin/mod.rs:3079-3118`

## Open Questions

- Upstream Mattermost server calls-plugin implementation is not present in `../mattermost` tree; verification used Mattermost Mobile behavior as primary contract + Rustchat integration tests.
