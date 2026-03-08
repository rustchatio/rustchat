- Component: Web hub disconnect flow
- Source path: `../mattermost/server/channels/app/platform/web_hub.go`
- Observed behavior:
  - When all user connections are inactive and no other cluster node has active conns, server queues non-manual offline (`QueueSetStatusOffline(userID, false)`) (`:626-630`).

- Component: Connection bootstrap
- Source path: `../mattermost/server/channels/app/platform/web_conn.go`
- Observed behavior:
  - New authenticated websocket connection triggers `SetStatusOnline(userID, false)` and activity refresh (`:203-207`).

- Component: Auth-challenge websocket route
- Source path: `../mattermost/server/channels/app/platform/websocket_router.go`
- Observed behavior:
  - After websocket auth challenge succeeds, server registers connection and sets online (`:60-70`).

- Component: Status state rules
- Source path: `../mattermost/server/channels/app/platform/status.go`
- Observed behavior:
  - `SetStatusOnline` and `SetStatusOffline` contain manual-status guard logic (`:314-316`, `:363-365`).
  - `QueueSetStatusOffline` also respects manual status guard (`:387-390`).

- Relevance to Rustchat:
  - Rustchat already had connect/disconnect online/offline transitions, but lacked persisted manual-state parity in DB status reads.
