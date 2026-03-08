# Mobile Findings

## Upstream evidence (`../mattermost-mobile`)

1. Mobile ping call:
- `../mattermost-mobile/app/client/rest/general.ts:32-41` calls `${urlVersion}/system/ping`

2. Mobile config old format call:
- `../mattermost-mobile/app/client/rest/general.ts:60-64` calls `${urlVersion}/config/client?format=old`

3. Mobile license old format call:
- `../mattermost-mobile/app/client/rest/general.ts:67-71` calls `${urlVersion}/license/client?format=old`

4. Mobile websocket endpoint construction:
- `../mattermost-mobile/app/client/websocket/index.ts:85-91` uses `WebsocketURL || serverUrl` + `/api/v4/websocket`
- `../mattermost-mobile/app/products/calls/connection/websocket_client.ts:45-48` does same pattern for calls websocket path.

## Impact assessment

If `/system/ping` and related bootstrap endpoints are absent (or returning fallback/non-contract responses), mobile fails early during server validation and realtime initialization.
