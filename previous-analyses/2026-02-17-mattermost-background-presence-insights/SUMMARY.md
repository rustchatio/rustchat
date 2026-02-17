# Mattermost Background Presence Insights

- Topic: Mobile background websocket/presence behavior.
- Date: 2026-02-17
- Scope: Mattermost server websocket/presence pipeline and Mattermost mobile websocket lifecycle.

## Compatibility contract (observed behavior)

1. Mattermost mobile intentionally closes websocket connections after app goes to background for 15 seconds.
   - Evidence: `../mattermost-mobile/app/managers/websocket_manager.ts:23`, `../mattermost-mobile/app/managers/websocket_manager.ts:281`, `../mattermost-mobile/app/managers/websocket_manager.ts:287`, `../mattermost-mobile/app/managers/websocket_manager.test.ts:170`.
2. Background close is explicit stop mode (`close(true)`), so reconnect is disabled until foreground/open logic runs.
   - Evidence: `../mattermost-mobile/app/managers/websocket_manager.ts:106`, `../mattermost-mobile/app/client/websocket/index.ts:380`, `../mattermost-mobile/app/client/websocket/index.ts:381`, `../mattermost-mobile/app/client/websocket/index.ts:250`.
3. Returning to active app state re-opens websocket clients (`openAll('WebSocket Reconnect')`).
   - Evidence: `../mattermost-mobile/app/managers/websocket_manager.ts:269`, `../mattermost-mobile/app/managers/websocket_manager.ts:275`.
4. Server marks user online on websocket authentication/connect flow.
   - Evidence: `../mattermost/server/channels/app/platform/websocket_router.go:67`, `../mattermost/server/channels/app/platform/websocket_router.go:68`, `../mattermost/server/channels/app/platform/web_conn.go:205`.
5. Server disconnect path marks user offline only when all local connections are inactive and cluster-wide connection count is zero.
   - Evidence: `../mattermost/server/channels/app/platform/web_conn.go:423`, `../mattermost/server/channels/app/platform/web_hub.go:608`, `../mattermost/server/channels/app/platform/web_hub.go:616`, `../mattermost/server/channels/app/platform/web_hub.go:628`, `../mattermost/server/channels/app/platform/web_hub.go:629`.
6. If cluster count lookup fails, Mattermost intentionally does not set user offline (conservative behavior).
   - Evidence: `../mattermost/server/channels/app/platform/web_hub.go:619`, `../mattermost/server/channels/app/platform/web_hub.go:621`, `../mattermost/server/channels/app/platform/web_hub.go:624`.
7. Non-manual offline transitions do not override manual status (busy/dnd/etc.).
   - Evidence: `../mattermost/server/channels/app/platform/status.go:363`, `../mattermost/server/channels/app/platform/status.go:387`, `../mattermost/server/channels/app/platform/status_test.go:179`, `../mattermost/server/channels/app/platform/status_test.go:181`.
8. When not all connections are inactive, server does not force offline and may move toward away based on activity timeout.
   - Evidence: `../mattermost/server/channels/app/platform/web_hub.go:633`, `../mattermost/server/channels/app/platform/web_hub.go:644`, `../mattermost/server/channels/app/platform/web_hub.go:647`.
9. Mobile "Connection restored" UI is shown on reconnect after disconnect, not on first connect.
   - Evidence: `../mattermost-mobile/app/components/connection_banner/use_connection_banner.ts:101`, `../mattermost-mobile/app/components/connection_banner/use_connection_banner.ts:103`, `../mattermost-mobile/app/components/connection_banner/use_connection_banner.ts:105`, `../mattermost-mobile/app/components/connection_banner/use_connection_banner.test.ts:652`.

## Key insight for Rustchat

Mattermost does not keep the general mobile websocket permanently alive in background. The intended design is:
- mobile app closes websocket shortly after backgrounding,
- server computes presence from active websocket count (plus cluster count),
- manual statuses are protected from non-manual offline updates.

Any Rustchat behavior that immediately overwrites manual busy/dnd to offline, or that sets offline without connection-count/cluster checks, diverges from Mattermost.

## Open questions

- Mattermost mobile can appear "online" in some user scenarios due to another active client (web/desktop) or quick foreground return before 15s timer fires. Validate this against your exact reproduction timeline.
