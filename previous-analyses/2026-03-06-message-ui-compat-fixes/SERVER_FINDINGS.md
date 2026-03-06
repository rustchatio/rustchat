# Server Findings

## Upstream evidence (`../mattermost`)

1. Post edit time-limit is a first-class service config:
- `../mattermost/server/public/model/config.go:404` (`PostEditTimeLimit`)
- default value `-1`: `../mattermost/server/public/model/config.go:832-834`

2. Edit API enforces the configured time limit and returns a specific error ID:
- `../mattermost/server/channels/api4/post.go:1074-1076`
- `../mattermost/server/channels/api4/post.go:1186-1188`
- error ID used by webapp handling: `../mattermost/webapp/channels/src/actions/views/posts.js:27-30`

3. Web post utilities use `edit_at > 0` and `PostEditTimeLimit`:
- `../mattermost/webapp/channels/src/packages/mattermost-redux/src/utils/post_utils.ts:51-53`
- `../mattermost/webapp/channels/src/packages/mattermost-redux/src/utils/post_utils.ts:69-74`

## Rustchat current behavior evidence

1. V4 edit endpoints update message unconditionally (owner-only) with no global time-limit gate:
- `backend/src/api/v4/posts.rs:1249-1301` (no limit check before `UPDATE posts SET message = $1, edited_at = NOW()`)

2. V1 edit endpoint also has no global time-limit gate:
- `backend/src/api/posts.rs:242-301`

3. Rustchat config/client compatibility payload currently does not expose `PostEditTimeLimit` or `AllowEditPost`:
- `backend/src/api/v4/config_client.rs:133-260` (legacy map population)
- `backend/src/api/v4/config_client.rs:260-420`

4. `GET /api/v4/system/config` ServiceSettings payload currently omits `PostEditTimeLimit`:
- `backend/src/api/v4/system.rs:310-316`

5. Websocket mapping does expose post-edited event name compatibility:
- `backend/src/realtime/events.rs:85-87`

## Contract capture notes

- Contract-sensitive edit surfaces:
  - `PUT /api/v4/posts/{post_id}`
  - `PUT /api/v4/posts/{post_id}/patch`
  - `GET /api/v4/config/client?format=old` (must surface edit-limit config values used by clients)
- Expected error semantic when over limit:
  - `id = api.post.update_post.permissions_time_limit.app_error`, HTTP `400`
