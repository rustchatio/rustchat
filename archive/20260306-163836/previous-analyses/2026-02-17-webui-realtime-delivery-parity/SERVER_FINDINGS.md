- Endpoint or component: Mattermost websocket connect + channel-scoped broadcast filter
- Source path: `../mattermost/server/channels/api4/websocket.go`
- Source lines: 75-84, 114-117
- Observed behavior:
  - Websocket connection builds `WebConnConfig` and registers the connection in hub (`HubRegister`) for authenticated users.
- Notes:
  - Connection setup is centralized and then event visibility is enforced by hub/webconn membership checks.

- Endpoint or component: Mattermost hub channel indexing on connect
- Source path: `../mattermost/server/channels/app/platform/web_hub.go`
- Source lines: 846-864
- Observed behavior:
  - On connection add, hub loads all channel memberships via `GetAllChannelMembersForUser` and indexes the connection by channel.
- Notes:
  - This enables immediate delivery for all channels the user belongs to.

- Endpoint or component: Mattermost per-event delivery filter
- Source path: `../mattermost/server/channels/app/platform/web_conn.go`
- Source lines: 958-1000
- Observed behavior:
  - For events with `broadcast.channel_id`, server checks membership (`allChannelMembers`) and only sends to members.
- Notes:
  - Delivery contract is membership-scoped, not active-channel-scoped.

- Endpoint or component: Rustchat websocket default scope wiring
- Source path: `backend/src/api/websocket_core.rs`
- Source lines: 139-145, 346-370
- Observed behavior:
  - `initialize_connection_state` conditionally subscribes channels via `subscribe_channels` flag.
  - If `subscribe_channels` is false, channel subscriptions are skipped.
- Notes:
  - This is the key switch controlling cross-channel realtime delivery.

- Endpoint or component: Rustchat v1 websocket path
- Source path: `backend/src/api/ws.rs`
- Source lines: 95-98
- Observed behavior:
  - v1 websocket initializes with `subscribe_channels=false`.
- Notes:
  - This diverges from member-channel-wide delivery semantics.

- Endpoint or component: Rustchat v4 websocket path
- Source path: `backend/src/api/v4/websocket.rs`
- Source lines: 264-267
- Observed behavior:
  - v4 websocket initializes with `subscribe_channels=true`.
- Notes:
  - v4 behavior already aligns with full channel subscription on connect.
