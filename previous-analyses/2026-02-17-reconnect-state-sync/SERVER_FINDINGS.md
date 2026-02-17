# Server Findings (Mattermost)

## Conservative Offline

- Endpoint/component: websocket hub disconnect/offline transition
- Source path: `../mattermost/server/channels/app/platform/web_hub.go`
- Source lines: 605-630
- Observed behavior:
  - Checks whether all local connections are inactive.
  - In cluster mode, fetches remote web connection count.
  - If cluster count lookup errors, it does **not** force offline (conservative behavior).
  - Sets offline only when remote count is zero.

## Manual Status Precedence

- Endpoint/component: status transitions
- Source path: `../mattermost/server/channels/app/platform/status.go`
- Source lines: 314-315, 355-366, 387-389
- Observed behavior:
  - Manual status blocks non-manual overwrite (`status.Manual && !manual` short-circuit).
  - Applies to both direct offline and queued offline updates.

## Websocket Auth/Online on Connect

- Endpoint/component: websocket router
- Source path: `../mattermost/server/channels/app/platform/websocket_router.go`
- Source lines: 67-70
- Observed behavior:
  - On authenticated websocket connect, marks user online and updates activity.

## No Dedicated WS "Reconnect Snapshot" Event

- Endpoint/component: websocket API/user actions
- Source path: `../mattermost/server/channels/wsapi/user.go`
- Source lines: 44-62
- Observed behavior:
  - Supports typing and active-status updates.
  - No explicit websocket `initial_load` snapshot event.
