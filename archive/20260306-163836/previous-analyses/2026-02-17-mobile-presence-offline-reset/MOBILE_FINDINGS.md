- Screen/store/service: Websocket lifecycle manager
- Source path: `../mattermost-mobile/app/managers/websocket_manager.ts`
- Observed behavior:
  - On websocket close (first failure), client sets current user status to offline locally and stops periodic status updates (`:192-197`).
  - On app going background, manager schedules `closeAll()` after 15 seconds (`:281-287`).
  - On app becoming active with network, manager runs `openAll('WebSocket Reconnect')` (`:269-276`).

- Screen/store/service: Connection banner
- Source path: `../mattermost-mobile/app/components/connection_banner/use_connection_banner.ts`
- Observed behavior:
  - When websocket state transitions from non-connected to connected, client displays `Connection restored` (`:100-112`).
  - Banner logic is app-active only (`:140-164`, `:175-184`).

- Screen/store/service: Local status mutation helper
- Source path: `../mattermost-mobile/app/actions/local/user.ts`
- Observed behavior:
  - `setCurrentUserStatus` updates local DB user status; this is used by websocket close handling (`:16-31`).

- Inference:
  - Mobile UX semantics are websocket-lifecycle-driven: close => local offline; reconnect => restored/connected UX and follow-up sync.
