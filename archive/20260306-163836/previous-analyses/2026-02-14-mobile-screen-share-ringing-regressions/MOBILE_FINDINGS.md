- Screen, store, or service: Mattermost-mobile screen event handling
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: 111-117
- Observed behavior: `CALLS_SCREEN_ON/OFF` mutate call screen session state by `session_id`.
- Notes: Correct screen rendering still depends on receiving an actual remote screen stream URL.

- Screen, store, or service: Mattermost-mobile remote stream wiring
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/connection.ts`
- Source lines: 430-439
- Observed behavior: `peer.on('stream')` sets `screenShareURL` when a remote stream has video tracks.
- Notes: Stream identity/timing matters for screen rendering updates.

- Screen, store, or service: Mattermost-mobile ringing path
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/state/actions.ts`
- Source lines: 476-483, 78-118
- Observed behavior: Incoming ringing notifications are populated via `callStarted -> processIncomingCalls`.
- Notes: This upstream tree has no explicit websocket `CALLS_RINGING` event path.

- Screen, store, or service: Mattermost-mobile websocket call event map
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/event.ts`
- Source lines: 203-205
- Observed behavior: Handles `CALLS_CALL_START`; no `CALLS_RINGING` case in this upstream tree.
- Notes: `/ring` compatibility may require reusing call-start semantics.
