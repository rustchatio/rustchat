- Screen, store, or service: Mattermost Mobile websocket client URL
- Source path: `../mattermost-mobile/app/client/websocket/index.ts`
- Source lines: 85-90
- Observed behavior:
  - Mobile websocket client connects to `/api/v4/websocket`.
- Notes:
  - Mobile path uses v4 websocket contract.

- Screen, store, or service: Mattermost Mobile websocket manager dispatch
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`
- Source lines: 88-92
- Observed behavior:
  - Incoming websocket events are fed into `handleWebSocketEvent`.
- Notes:
  - Event pipeline assumes continuous posted-event delivery.

- Screen, store, or service: Mattermost Mobile websocket event routing
- Source path: `../mattermost-mobile/app/actions/websocket/event.ts`
- Source lines: 32-37
- Observed behavior:
  - `WebsocketEvents.POSTED` is routed to `posts.handleNewPostEvent`.
- Notes:
  - `posted` is first-class for immediate message ingestion.

- Screen, store, or service: Mattermost Mobile posted event handler
- Source path: `../mattermost-mobile/app/actions/websocket/posts.ts`
- Source lines: 39-55, 66-85
- Observed behavior:
  - Parses post payload immediately, ensures channel membership state, and updates local records.
- Notes:
  - Delivery timing depends on receiving websocket posted events promptly.

- Screen, store, or service: Rustchat WebUI websocket event handling
- Source path: `frontend/src/composables/useWebSocket.ts`
- Source lines: 112, 189-201
- Observed behavior:
  - WebUI connects to `/api/v1/ws` and handles `posted` by calling `messageStore.handleNewMessage`.
- Notes:
  - Client side already supports posted event ingestion; backend scope is the bottleneck.

- Screen, store, or service: Rustchat WebUI explicit per-channel subscribe
- Source path: `frontend/src/views/main/ChannelView.vue`
- Source lines: 69-76
- Observed behavior:
  - UI only subscribes active channel via `subscribe(newId)`/`unsubscribe(oldId)`.
- Notes:
  - Without server-side default channel subscriptions, off-channel immediate events can be missed.
