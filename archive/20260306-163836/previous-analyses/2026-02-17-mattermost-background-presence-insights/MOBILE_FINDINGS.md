# Mobile Findings

## 1) Background close timer is explicit and fixed to 15 seconds

- Screen, store, or service: websocket manager lifecycle.
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`.
- Source lines: 23, 281-287.
- Observed behavior:
  - `WAIT_TO_CLOSE` is 15 seconds.
  - On transition from active to background, app starts background timer and closes all websocket clients when timer fires.
- Notes:
  - This is intentional lifecycle behavior, not an error path.

## 2) Tests assert 15-second background close behavior

- Screen, store, or service: websocket manager tests.
- Source path: `../mattermost-mobile/app/managers/websocket_manager.test.ts`.
- Source lines: 160-171.
- Observed behavior:
  - Test expects `BackgroundTimer.setInterval(..., 15000)` after app state changes to background.
- Notes:
  - This acts as executable documentation of intended behavior.

## 3) closeAll uses stop-mode close and invalidate

- Screen, store, or service: websocket manager close path.
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`.
- Source lines: 103-109.
- Observed behavior:
  - Each client is closed with `close(true)` and then invalidated.
- Notes:
  - Stop mode affects reconnect behavior in websocket client.

## 4) stop-mode close suppresses auto-reconnect

- Screen, store, or service: websocket client close/reconnect policy.
- Source path: `../mattermost-mobile/app/client/websocket/index.ts`.
- Source lines: 250-252, 380-387.
- Observed behavior:
  - `close(true)` sets `stop=true`.
  - On socket close event, reconnect is skipped when `stop` is true.
- Notes:
  - Reconnect resumes only through manager `openAll` when app becomes active.

## 5) Foreground transition re-opens websocket clients

- Screen, store, or service: app state handling.
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`.
- Source lines: 269-276.
- Observed behavior:
  - When app becomes active and network is connected, manager calls `openAll('WebSocket Reconnect')`.
- Notes:
  - This matches observed "Connection restored" patterns.

## 6) Mobile marks local current user offline on close callback first fail

- Screen, store, or service: websocket close callback.
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`.
- Source lines: 192-197.
- Observed behavior:
  - On first close failure (`connectFailCount <= 1`), app sets local current user status to offline and stops periodic status polling.
- Notes:
  - This is local DB/UI state handling; server-side presence still depends on server websocket/session logic.

## 7) Connection banner semantics for reconnect

- Screen, store, or service: connection banner hook/tests.
- Source path: `../mattermost-mobile/app/components/connection_banner/use_connection_banner.ts`.
- Source lines: 101-116.
- Observed behavior:
  - "Connection restored" banner is shown only when websocket transitions to connected after prior disconnected state and not during initial app session.
- Notes:
  - Background-only app state changes do not automatically imply banner.

## 8) Test covers no "Connection restored" when websocket stayed connected

- Screen, store, or service: connection banner test.
- Source path: `../mattermost-mobile/app/components/connection_banner/use_connection_banner.test.ts`.
- Source lines: 476-533.
- Observed behavior:
  - Returning from background with websocket still connected should not show the restored banner.
- Notes:
  - Distinguishes reconnect from plain foregrounding.
