- Screen, store, or service: Websocket screen state handlers
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: 120-126
- Observed behavior: `handleCallScreenOn/Off` updates current call screen session state from `msg.data.session_id`.
- Notes: State update alone does not guarantee visible media.

- Screen, store, or service: Remote stream handling
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/connection.ts`
- Source lines: 434-447
- Observed behavior: `setScreenShareURL(remoteStream.toURL())` is set when a remote stream has video tracks.
- Notes: If negotiated stream is non-decodable/empty for mobile, UI will reserve space but render empty.

- Screen, store, or service: Screen view rendering
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/screens/call_screen/call_screen.tsx`
- Source lines: 647-656
- Observed behavior: Screen view renders only with both `currentCall.screenShareURL` and `screenShareOn`.
- Notes: Matches reported symptom of empty screen area when media is unavailable.
