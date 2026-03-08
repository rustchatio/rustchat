# Summary

- Topic: Mobile bootstrap regression after last build (v4 system route stubs)
- Date: 2026-03-06
- Scope: Mattermost mobile bootstrap contract (`/api/v4/system/ping`, `/api/v4/system/version`, `/api/v4/config/client?format=old`, `/api/v4/license/client?format=old`, websocket URL discovery)
- Compatibility contract:
  - `GET /api/v4/system/ping` must return JSON status payload and remain publicly reachable.
  - `GET /api/v4/system/version` must return a version string used by clients.
  - `GET /api/v4/config/client?format=old` and `GET /api/v4/license/client?format=old` must remain available for mobile bootstrap.
  - Mobile websocket connection uses configured `WebsocketURL` or site URL and appends `/api/v4/websocket`.
- Open questions:
  - None blocking; root cause is code regression in current branch.
