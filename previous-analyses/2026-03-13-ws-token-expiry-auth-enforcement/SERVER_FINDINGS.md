# Server Findings

- Endpoint or component: Mattermost websocket event delivery auth guard
- Source path: `../mattermost/server/channels/app/platform/web_conn.go`
- Source lines: `778`, `820`, `880`
- Observed behavior: `ShouldSendEvent` requires `IsAuthenticated()`. When session is expired/invalid, auth fails and events are not delivered to that websocket connection.
- Notes: Upstream enforces auth validity continuously during outbound websocket delivery, not only during handshake.

- Endpoint or component: Rustchat websocket initial auth extraction
- Source path: `backend/src/api/websocket_core.rs`
- Source lines: `363`
- Observed behavior: `validate_user_id` only validates token during connection setup and returns `sub` only.
- Notes: No expiry timestamp is retained for runtime enforcement.

- Endpoint or component: Rustchat websocket run loop (`/api/v4/websocket`)
- Source path: `backend/src/api/v4/websocket.rs`
- Source lines: `203`, `409`
- Observed behavior: Main loop handles socket/hub events, but has no branch that closes on JWT expiration.
- Notes: Active connections continue receiving hub broadcasts after token lifetime boundary.

- Endpoint or component: Rustchat websocket run loop (`/ws`)
- Source path: `backend/src/api/ws.rs`
- Source lines: `47`, `87`
- Observed behavior: Token is validated once before upgrade and never revalidated afterward.
- Notes: Same post-expiry delivery risk exists on legacy endpoint.
