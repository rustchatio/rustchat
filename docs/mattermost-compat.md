# Mattermost Compatibility

RustChat implements a partial Mattermost API v4 compatibility layer for Mattermost mobile/desktop clients.

## Last Verified

- Date: **2026-02-07**
- Verified against:
  - `backend/src/api/v4/mod.rs`
  - `backend/src/api/v4/system.rs`
  - `backend/src/api/v4/plugins.rs`
  - `backend/src/api/v4/calls_plugin/mod.rs`
  - `backend/src/api/v4/websocket.rs`
  - `docs/api_v4_compatibility_report.md`

## Compatibility Baseline

- Mattermost compatibility version constant: `10.11.10` (`backend/src/mattermost_compat/mod.rs`).
- All `/api/v4/*` responses include `X-MM-COMPAT: 1`.
- Unmatched `/api/v4/*` routes return Mattermost-style `501 Not Implemented` JSON via router fallback.

## Verified Endpoint Coverage (Core Client Flows)

### System and startup
- `GET /api/v4/system/ping`
- `GET /api/v4/system/version`
- `GET /api/v4/config/client`
- `GET /api/v4/license/client`

### Auth and user bootstrap
- `POST /api/v4/users/login`
- `GET /api/v4/users/me`
- `GET /api/v4/users/me/teams`
- `GET /api/v4/users/me/teams/{team_id}/channels`
- `POST /api/v4/users/status/ids`
- `GET|PUT /api/v4/users/{user_id}/status`
- `GET|PUT /api/v4/users/me/status`

### Channels, posts, files
- `GET /api/v4/channels/{channel_id}/posts`
- `POST /api/v4/posts`
- `GET /api/v4/posts/{post_id}`
- `GET /api/v4/files/{file_id}`
- `GET /api/v4/files/{file_id}/info`

### Threads
- `GET /api/v4/users/{user_id}/threads`
- `GET|PUT /api/v4/users/{user_id}/teams/{team_id}/threads`
- `PUT|DELETE /api/v4/users/{user_id}/teams/{team_id}/threads/{thread_id}/following`

### Calls plugin namespace
- Routes exist under `/api/v4/plugins/com.mattermost.calls/*` for version/config/channels and call lifecycle (`start`, `join`, `leave`, state, reactions, mute/unmute, raise/lower hand, `offer`, `ice`).
- TURN/STUN data is exposed from runtime configuration.

## WebSocket Compatibility

- Endpoint: `GET /api/v4/websocket`
- Supports authentication challenge flow.
- Internal events are mapped to Mattermost-style events including:
  - `posted`
  - `typing`
  - `post_edited`
  - `post_deleted`
  - `reaction_added`
  - `reaction_removed`
  - `status_change`
  - `channel_viewed`
  - `user_added`
  - `user_removed`

## Explicitly Unsupported (501) Examples

- `POST /api/v4/plugins`
- `POST /api/v4/plugins/install_from_url`
- `DELETE /api/v4/plugins/{plugin_id}`
- `POST /api/v4/plugins/{plugin_id}/enable`
- `POST /api/v4/plugins/{plugin_id}/disable`
- `POST /api/v4/actions/dialogs/open`
- `POST /api/v4/actions/dialogs/submit`
- `POST /api/v4/actions/dialogs/lookup`

## Notes

- Compatibility is endpoint-by-endpoint and behavior depth varies.
- For prioritized endpoint-level status and known stubbed areas, see `docs/api_v4_compatibility_report.md`.
