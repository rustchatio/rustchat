# Server Findings

- Endpoint or component: Status bulk endpoint input/output contract
- Source path: `../mattermost/server/channels/api4/status.go`
- Source lines: 51-88
- Observed behavior: Handler decodes body as array of user IDs, validates IDs, returns marshaled status array.
- Notes: Rustchat accepts both raw array and wrapped object for transition, and returns array.
- Resolution status: Closed

- Endpoint or component: Custom status route topology
- Source path: `../mattermost/server/channels/api4/status.go`
- Source lines: 15-24
- Observed behavior: Custom status routes are mounted on `BaseRoutes.User`, so `/users/me/status/custom*` is valid.
- Notes: Rustchat added explicit `"me"` resolution for custom-status path param handlers.
- Resolution status: Closed

- Endpoint or component: Channel member notify props update response
- Source path: `../mattermost/server/channels/api4/channel.go`
- Source lines: 1816-1846
- Observed behavior: Successful notify_props update writes status-only OK response.
- Notes: Rustchat returns `{ "status": "OK" }` from v4 notify_props update.
- Resolution status: Closed

- Endpoint or component: Categories order read endpoint
- Source path: `../mattermost/server/channels/api4/channel_category.go`
- Source lines: 97-140
- Observed behavior: `GET /users/{user}/teams/{team}/channels/categories/order` returns ordered category IDs.
- Notes: Rustchat supports GET and PUT for `/channels/categories/order`.
- Resolution status: Closed

- Endpoint or component: Categories update payload
- Source path: `../mattermost/server/channels/api4/channel_category.go`
- Source lines: 203-241
- Observed behavior: Update categories decodes request body as raw array of categories.
- Notes: Rustchat accepts raw array canonically and wrapped object for transition.
- Resolution status: Closed

## Closeout note

- Verified with `cargo test --test api_v4_mobile_presence --test api_v4_channel_member_routes --test api_categories`.
- Contract-level parity blockers from this findings set are closed.
