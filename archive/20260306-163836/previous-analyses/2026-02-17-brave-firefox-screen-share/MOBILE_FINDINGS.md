- Screen, store, or service: Calls config defaults
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/types/calls.ts`
- Source lines: 161-176
- Observed behavior: Default calls config sets `EnableSimulcast: false`.
- Notes: Indicates compatibility baseline where calls can run without simulcast.

- Screen, store, or service: Calls connection config usage
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/connection.ts`
- Source lines: 99-101
- Observed behavior: Connection logic checks `EnableAV1` combined with `!EnableSimulcast`.
- Notes: Upstream client code is config-aware regarding simulcast mode.

- Screen, store, or service: Screen-share state handlers
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: 111-117
- Observed behavior: Screen on/off events update screen-sharing session state.
- Notes: Correct UX still depends on a valid corresponding remote stream.

- Screen, store, or service: Screen-share rendering
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/screens/call_screen/call_screen.tsx`
- Source lines: 647-656
- Observed behavior: Screen share view renders only when `screenShareURL` exists and screen state is on.
- Notes: One-way media failures appear as missing screen stream despite state changes.
