- Screen, store, or service: Websocket event constants
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/constants/websocket.ts`
- Source lines: 65-88
- Observed behavior: Calls websocket constants include call start/end/state/caption/etc but no `CALLS_RINGING` constant.
- Notes: Add `CALLS_RINGING: custom_${Calls.PluginId}_ringing`.

- Screen, store, or service: Websocket dispatcher
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/event.ts`
- Source lines: 178-247
- Observed behavior: Calls-related switch cases dispatch many call events, but no routing for ringing.
- Notes: Add `case WebsocketEvents.CALLS_RINGING` routing to calls handler.

- Screen, store, or service: Calls websocket handlers
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: 54-212
- Observed behavior: Handler set includes call state, start/end, mute, captions, host controls; no ringing handler.
- Notes: Add `handleCallRinging` with guard for active call and existing incoming-call processing.

- Screen, store, or service: Incoming call pipeline
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/state/actions.ts`
- Source lines: 78-152
- Observed behavior: `processIncomingCalls` enforces ringing config, dismissal rules, ownership/current-call guards, channel lookup, and DM/GM gating.
- Notes: Reuse this path to preserve current ringing semantics.

- Screen, store, or service: Channel call fetch path
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/actions/calls.ts`
- Source lines: 116-141
- Observed behavior: `loadCallForChannel` fetches and stores call state for a channel and returns call data.
- Notes: Ringing handler can call this when local call state for channel is absent.
