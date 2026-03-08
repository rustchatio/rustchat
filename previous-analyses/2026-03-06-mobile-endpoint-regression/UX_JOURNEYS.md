# UX Journeys

## Journey: Add server in Mattermost mobile

1. User enters server endpoint.
2. Mobile calls `GET /api/v4/system/ping` (`general.ts:32-41`).
3. Mobile requests `config/client?format=old` and `license/client?format=old`.
4. Mobile initializes websocket URL from config/site URL and connects to `/api/v4/websocket`.

## Current failure point

- Step 2 fails when `system::router()` is empty in active backend build.
- User sees connection/setup failure and cannot proceed to authenticated usage.
