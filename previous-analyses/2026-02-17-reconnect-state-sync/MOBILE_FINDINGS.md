# Mobile Findings (Mattermost Mobile)

## Reliable Reconnect Identity

- Screen/store/service: websocket client
- Source path: `../mattermost-mobile/app/client/websocket/index.ts`
- Source lines: 111-116, 300-325, 329-340
- Observed behavior:
  - Connects with `?connection_id=<id>&sequence_number=<seq>` when reliable websocket is enabled.
  - Uses `hello.connection_id` to determine whether this is a resumed stream or a new stream requiring resync.
  - Enforces strict sequence monotonicity for server event stream.

## Reconnect Sync Trigger

- Screen/store/service: reconnect action
- Source path: `../mattermost-mobile/app/actions/websocket/index.ts`
- Source lines: 48-53, 65-83
- Observed behavior:
  - `handleReconnect` executes `doReconnect`.
  - `doReconnect` performs `entry(...)`, then batches models into DB.
  - This is the full-state refresh path after reconnect.

## Background Websocket Close

- Screen/store/service: websocket manager app-state handling
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`
- Source lines: 23, 281-288
- Observed behavior:
  - Uses a 15s background timer (`WAIT_TO_CLOSE`) before closing sockets.

## Heartbeat/Pong Expectations

- Screen/store/service: websocket client ping/pong loop
- Source path: `../mattermost-mobile/app/client/websocket/index.ts`
- Source lines: 200-216, 296-299
- Observed behavior:
  - Sends ping every 30s.
  - Expects pong replies and closes/reconnects if pong is not received in time.
