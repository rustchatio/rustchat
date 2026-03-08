- Screen, store, or service: Calls REST client (mobile)
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/client/rest.ts`
- Source lines: 43-51
- Observed behavior: Calls list/state requests include `?mobilev2=true`.
- Notes: Confirms compatibility expectation around mobilev2 query semantics.

- Screen, store, or service: Calls websocket screen state handlers
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: 111-117
- Observed behavior: `handleCallScreenOn/Off` updates screen-sharing state via `session_id`.
- Notes: Correct rendering requires a media stream corresponding to that session.

- Screen, store, or service: Calls screen rendering
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/screens/call_screen/call_screen.tsx`
- Source lines: 647-656
- Observed behavior: Screen share view is rendered only when `currentCall.screenShareURL` is set and `screenShareOn` is true.
- Notes: State-only events are insufficient; media negotiation must remain stable.

- Screen, store, or service: Calls connection media lifecycle
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/connection.ts`
- Source lines: 167, 195
- Observed behavior: Audio state changes use `peer.replaceTrack(...)` (not repeated add/remove sender creation).
- Notes: This pattern informed the Rustchat desktop fix to reuse a persistent sender for screen-share toggles.
