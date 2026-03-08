# Server Findings

## 1) Websocket connect marks user online

- Endpoint or component: websocket auth challenge / web connection creation.
- Source path: `../mattermost/server/channels/app/platform/websocket_router.go`.
- Source lines: 39-70.
- Observed behavior:
  - After successful `authentication_challenge`, server registers hub connection and calls `SetStatusOnline(userId, false)` and `UpdateLastActivityAtIfNeeded`.
- Notes:
  - Same online update is also triggered in `NewWebConn`.

## 2) New web connection also updates online + activity

- Endpoint or component: `NewWebConn`.
- Source path: `../mattermost/server/channels/app/platform/web_conn.go`.
- Source lines: 200-207.
- Observed behavior:
  - For authenticated users, server asynchronously calls `SetStatusOnline(userID, false)` and `UpdateLastActivityAtIfNeeded(session)`.
- Notes:
  - This keeps online state aligned with fresh/reused websocket setup paths.

## 3) Disconnect flow goes through hub unregister

- Endpoint or component: websocket pump lifecycle.
- Source path: `../mattermost/server/channels/app/platform/web_conn.go`.
- Source lines: 406-424.
- Observed behavior:
  - After read/write pumps end, `HubUnregister(wc)` is called.
- Notes:
  - Presence transitions are centralized in hub unregister logic.

## 4) Offline transition is connection-count and cluster-count gated

- Endpoint or component: hub unregister branch.
- Source path: `../mattermost/server/channels/app/platform/web_hub.go`.
- Source lines: 605-633.
- Observed behavior:
  - Server checks all user connections in this node (`areAllInactive`).
  - If all inactive, it queries cluster-wide connection count.
  - It queues offline status only when cluster count is exactly zero.
- Notes:
  - This avoids false offline when user is connected elsewhere.

## 5) Cluster lookup errors do not force offline

- Endpoint or component: hub unregister error handling.
- Source path: `../mattermost/server/channels/app/platform/web_hub.go`.
- Source lines: 618-624.
- Observed behavior:
  - If cluster webconn count retrieval fails, server returns without setting offline.
- Notes:
  - Explicit conservative safeguard.

## 6) If other active connection exists, server avoids offline and may set away

- Endpoint or component: hub unregister non-all-inactive path.
- Source path: `../mattermost/server/channels/app/platform/web_hub.go`.
- Source lines: 633-649.
- Observed behavior:
  - With remaining active connections, server computes latest activity and can call `SetStatusLastActivityAt` (away path), not offline.
- Notes:
  - Presence is per-user across all active connections.

## 7) Manual status protection for offline updates

- Endpoint or component: status transition logic.
- Source path: `../mattermost/server/channels/app/platform/status.go`.
- Source lines: 355-367 and 377-390.
- Observed behavior:
  - `SetStatusOffline` and `QueueSetStatusOffline` both skip non-manual offline if current status is manual.
- Notes:
  - This preserves busy/dnd/ooo unless caller uses force/manual semantics.

## 8) Tests verify manual override behavior

- Endpoint or component: status tests.
- Source path: `../mattermost/server/channels/app/platform/status_test.go`.
- Source lines: 165-186, 188-209, 234-255.
- Observed behavior:
  - Non-manual offline does not override manual status.
  - Force path can override manual.
  - Manual offline sets `Manual=true`.
- Notes:
  - These are direct compatibility assertions for server behavior.
