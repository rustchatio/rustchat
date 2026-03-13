# Mobile Findings

- Component: Calls websocket client reconnect behavior
- Source path: `../mattermost-mobile/app/products/calls/connection/websocket_client.ts`
- Source lines: `45`, `67`, `145`
- Observed behavior: Client reconnects automatically on close and reuses the same auth token until refreshed.
- Notes: Mobile-side behavior assumes server enforces token validity; no local token-expiry hard stop is applied in this module.

- Component: Hello and session continuity handling
- Source path: `../mattermost-mobile/app/products/calls/connection/websocket_client.ts`
- Source lines: `81`, `97`, `117`
- Observed behavior: Client updates connection/session ids from server `hello`, but does not perform auth renewal in websocket layer.
- Notes: Server-driven disconnect on expiry is required for secure parity.
