# Server Findings

- Endpoint or component: Status bulk endpoint input/output contract
- Source path: `../mattermost/server/channels/api4/status.go`
- Source lines: 51-88
- Observed behavior: Handler decodes body as array of user IDs, validates IDs, returns marshaled status array.
- Notes: Rustchat now accepts both raw array and wrapped object for transition, and returns array.

- Endpoint or component: Custom status route topology
- Source path: `../mattermost/server/channels/api4/status.go`
- Source lines: 15-24
- Observed behavior: Custom status routes are mounted on `BaseRoutes.User`, so `/users/me/status/custom*` is valid.
- Notes: Rustchat added explicit `"me"` resolution for custom-status path param handlers.

- Endpoint or component: Channel member notify props update response
- Source path: `../mattermost/server/channels/api4/channel.go`
- Source lines: 1816-1846
- Observed behavior: Successful notify_props update writes status-only OK response.
- Notes: Rustchat now returns `{ "status": "OK" }` from v4 notify_props update.

- Endpoint or component: Categories order read endpoint
- Source path: `../mattermost/server/channels/api4/channel_category.go`
- Source lines: 97-140
- Observed behavior: `GET /users/{user}/teams/{team}/channels/categories/order` returns ordered category IDs.
- Notes: Rustchat route now supports GET and PUT for `/channels/categories/order`.

- Endpoint or component: Categories update payload
- Source path: `../mattermost/server/channels/api4/channel_category.go`
- Source lines: 203-241
- Observed behavior: Update categories decodes request body as raw array of categories.
- Notes: Rustchat now accepts raw array canonically and wrapped object for transition.
