# Summary

- Topic: WebUI parity + mobile compatibility gap closure (status, categories, notify props, settings/menu/composer alignment)
- Date: 2026-03-01
- Scope: `backend/src/api/v4/*`, websocket status broadcast semantics, and frontend parity-sensitive consumers (`frontend/src/components/*`, `frontend/src/api/*`, `frontend/src/stores/*`)
- Compatibility contract:
  - `POST /api/v4/users/status/ids` must accept raw array payload from mobile (`../rustchat-mobile/app/client/rest/users.ts:406`) and return status array semantics consumed by mobile reducers (`../rustchat-mobile/app/actions/remote/user.ts:370`); upstream parses sorted array from body (`../mattermost/server/channels/api4/status.go:51`).
  - Custom status routes must support `me` path usage from mobile (`../rustchat-mobile/app/client/rest/users.ts:429`, `:436`, `:443`).
  - `PUT /channels/{channel_id}/members/{user_id}/notify_props` should return `{status: "OK"}` semantics (`../mattermost/server/channels/api4/channel.go:1816-1846`).
  - Categories update endpoint must accept raw categories array (`../rustchat-mobile/app/client/rest/categories.ts:33`) and category order read endpoint must exist (`../rustchat-mobile/app/client/rest/categories.ts:20`; upstream `../mattermost/server/channels/api4/channel_category.go:97`).
  - Status websocket events must be broadly visible to connected listeners (not direct-user-only filtering) so presence indicators stay synchronized.
- Open questions:
  - Desktop settings/menu visuals are still not exact 1:1 with Mattermost in all rows/states; this iteration prioritizes contract and high-impact interaction parity.
