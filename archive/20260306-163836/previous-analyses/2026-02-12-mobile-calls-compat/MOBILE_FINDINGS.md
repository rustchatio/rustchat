- Screen, store, or service: Calls REST contract (routes and return types)
- Source path: `../mattermost-mobile/app/products/calls/client/rest.ts`
- Source lines: `8-25`, `41-53`, `73-85`, `94-105`, `108-113`, `142-149`
- Observed behavior: mobile expects `CallChannelState` for `getCalls` and `getCallForChannel`, plus specific routes for `end`, recording start/stop, dismiss, and host screen-off.
- Notes: missing server routes produce hard failures in call controls.

- Screen, store, or service: Calls state loading/parsing
- Source path: `../mattermost-mobile/app/products/calls/actions/calls.ts`
- Source lines: `81-114`, `116-141`, `143-168`
- Observed behavior: mobile requires `channel.call` payload in `/channels`, and `resp.call` in `/{channel_id}`; call payload expects `sessions`, `screen_sharing_session_id`, `recording`, `dismissed_notification`.
- Notes: server response shape mismatch prevents local call-state hydration.

- Screen, store, or service: Websocket event name contract
- Source path: `../mattermost-mobile/app/constants/websocket.ts`
- Source lines: `65-87`
- Observed behavior: mobile listens for exact calls event names (`...user_screen_on`, `...user_raise_hand`, `...call_host_changed`, `...user_dismissed_notification`, etc.).
- Notes: near-match event names are not handled.

- Screen, store, or service: Websocket payload field contract
- Source path: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: `81-91`, `111-125`, `131-144`, `146-159`, `198-210`
- Observed behavior: handlers require specific keys (`channelID`, `hostID`, `session_id`, `raised_hand`, `userID`, `callID`, `call` string payload for call_state).
- Notes: payload key casing differences break downstream state updates.

- Screen, store, or service: Feature gating from config
- Source path: `../mattermost-mobile/app/products/calls/hooks.ts`
- Source lines: `166-187`
- Observed behavior: host controls are gated by `HostControlsAllowed`; incoming ringing behavior is gated by `EnableRinging` elsewhere in calls state/actions.
- Notes: missing config flags silently disable parts of call UX.
