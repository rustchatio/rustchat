# Mobile Findings

- Screen, store, or service: Calls websocket event names consumed by mobile
- Source path: `../mattermost-mobile/app/constants/websocket.ts`
- Source lines: `73-87`
- Observed behavior: Mobile listens for exact names including `...calls_call_state`, `...calls_user_dismissed_notification`, `...calls_call_job_state`, `...calls_caption`.
- Notes: Name mismatch or missing events silently break reducers.

- Screen, store, or service: Incoming-call/ringing conditions
- Source path: `../mattermost-mobile/app/products/calls/state/actions.ts`
- Source lines: `78-132`, `476-483`
- Observed behavior: `callStarted` triggers `processIncomingCalls`; calls ring only when `EnableRinging`, not self-owned, not dismissed, and channel is DM/GM.
- Notes: Server must provide complete call state (including dismissal data) and DM/GM lifecycle events.

- Screen, store, or service: Dismissed notification wiring
- Source path: `../mattermost-mobile/app/products/calls/actions/calls.ts`
- Source lines: `167-168`
- Observed behavior: Mobile maps `dismissed_notification` from server to local `dismissed` dictionary.
- Notes: Empty/absent server field causes repeated incoming notifications.

- Screen, store, or service: Dismissed notification websocket handling
- Source path: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: `146-159`
- Observed behavior: On dismissed event for current user, mobile calls `removeIncomingCall`.
- Notes: Event alone updates transient state; persistence still depends on server call-state payload.

- Screen, store, or service: Full call-state sync websocket handling
- Source path: `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: `198-210`
- Observed behavior: `calls_call_state` expects `data.call` as serialized JSON string and applies full state refresh for channel.
- Notes: Required for robust sync during websocket lifecycle churn.

- Screen, store, or service: Calls config defaults / expected fields
- Source path: `../mattermost-mobile/app/products/calls/types/calls.ts`
- Source lines: `161-184`
- Observed behavior: Mobile references many config fields beyond current RustChat response.
- Notes: Missing fields can disable UI pathways or enable incorrect defaults.
