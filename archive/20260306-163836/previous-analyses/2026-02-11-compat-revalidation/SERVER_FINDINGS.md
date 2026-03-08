# Server Findings

- Endpoint or component: `/api/v4` router compatibility header + fallback
- Source path: `backend/src/api/v4/mod.rs`
- Source lines: 62-104, 106-125
- Observed behavior:
  - `/api/v4` includes a fallback handler returning Mattermost-style `501` payload (`id`, `message`, `detailed_error`, `request_id`, `status_code`).
  - `X-MM-COMPAT: 1` header is injected at the router layer.
- Notes: Baseline behavior is aligned with prior analysis for unsupported routes.

- Endpoint or component: Commands API route coverage
- Source path: `backend/src/api/v4/commands.rs`
- Source lines: 16-33
- Observed behavior:
  - Exposes `GET /commands`, `POST /commands/execute`, CRUD on `/commands/{id}`, `PUT /commands/{id}/move`, `POST /commands/{id}/regen_token`, and `GET /teams/{team_id}/commands/autocomplete_suggestions`.
  - Does not expose `POST /commands`.
- Notes: Mattermost server registers `POST /commands` and `PUT /commands/{id}/regen_token` (`../mattermost/server/channels/api4/command.go:17-29`). Rustchat method for regen token differs (`POST` vs `PUT`).

- Endpoint or component: `GET /commands` behavior
- Source path: `backend/src/api/v4/commands.rs`
- Source lines: 63-77
- Observed behavior:
  - Returns a hardcoded single `call` command object, not team/user scoped command inventory.
- Notes: Confirms stub/partial implementation.

- Endpoint or component: `GET /teams/{team_id}/commands/autocomplete_suggestions` behavior
- Source path: `backend/src/api/v4/commands.rs`
- Source lines: 137-159
- Observed behavior:
  - Returns object `{ suggestions: [...], did_succeed: true }`.
  - Suggestion fields are lowercase (`complete`, `suggestion`, `hint`, `description`).
  - Suggestion set is hardcoded to `/call` prefix.
- Notes:
  - Mattermost server returns a JSON array of suggestion objects (`../mattermost/server/channels/api4/command.go:454-503`), and mobile types expect `Complete/Suggestion/Hint/Description/IconData` (`../mattermost-mobile/types/api/integrations.d.ts:33-39`).

- Endpoint or component: `GET /teams/{team_id}/commands/autocomplete`
- Source path: `backend/src/api/v4/teams.rs`
- Source lines: 104-106, 903-909
- Observed behavior:
  - Route exists and handler returns `[]` unconditionally.
- Notes: Route exists but no functional autocomplete command data.

- Endpoint or component: `/actions/dialogs/*`
- Source path: `backend/src/api/v4/dialogs.rs`
- Source lines: 5-10, 23-42
- Observed behavior:
  - `open`, `submit`, `lookup` endpoints are explicit `501 Not Implemented`.
- Notes:
  - Mattermost server implements these (`../mattermost/server/channels/api4/integration_action.go:18-21,118-160`).

- Endpoint or component: Command execution internals
- Source path: `backend/src/api/integrations.rs`
- Source lines: 410-414, 985-1067
- Observed behavior:
  - `execute_command_internal` includes custom command lookup and outbound HTTP execution.
- Notes: Core execution pipeline exists, but v4 route-level contracts around list/create/autocomplete/dialog remain partial.

- Endpoint or component: Command response model
- Source path: `backend/src/models/integration.rs`
- Source lines: 166-173
- Observed behavior:
  - `CommandResponse` lacks fields commonly used by Mattermost (`trigger_id`, `type`, `props`, `skip_slack_parsing`, `extra_responses`).
- Notes: Missing `trigger_id` is significant for dialog/app command flows in mobile.

- Endpoint or component: Files preview support
- Source path: `backend/src/api/v4/files.rs`
- Source lines: 25-30, 454-527
- Observed behavior:
  - `/files/{file_id}/preview` and `/files/{file_id}/thumbnail` routes exist.
  - Preview attempts direct preview object first and then thumbnail fallback.
- Notes: Aligns with mobile expectations for image rendering paths.

- Endpoint or component: Scheduled posts route coverage
- Source path: `backend/src/api/v4/posts.rs`
- Source lines: 74-79
- Observed behavior:
  - Exposes `POST /posts/schedule`, `PUT|DELETE /posts/schedule/{id}`, `GET /posts/scheduled/team/{team_id}`.
- Notes:
  - This matches current Mattermost route family (`../mattermost/server/channels/api4/scheduled_post.go:19-23`).
  - Prior report claiming only `/scheduled_posts*` endpoints was stale for current client/server.

- Endpoint or component: Scheduled posts list contract
- Source path: `backend/src/api/v4/posts.rs`
- Source lines: 958-996
- Observed behavior:
  - Returns `Vec<mm::ScheduledPost>`.
- Notes:
  - Mattermost returns map keyed by team ID and optional `directChannels` (`../mattermost/server/channels/api4/scheduled_post.go:131-142`).
  - Contract mismatch likely affects mobile scheduled-post aggregation logic.

- Endpoint or component: Scheduled posts creation/update/delete details
- Source path: `backend/src/api/v4/posts.rs`
- Source lines: 998-1052, 1069-1149, 1151-1186
- Observed behavior:
  - Endpoints implemented with DB persistence and ownership checks.
  - Create handler returns default 200 response (no explicit 201 set).
  - No explicit `Connection-Id` header handling.
- Notes: Mattermost create path returns HTTP 201 and uses connection id in app layer (`../mattermost/server/channels/api4/scheduled_post.go:63,99-103`).

- Endpoint or component: WebSocket event mapping
- Source path: `backend/src/api/v4/websocket.rs`
- Source lines: 763-949
- Observed behavior:
  - Maps internal events to Mattermost event names: `posted`, `typing`, `post_edited`, `post_deleted`, `reaction_added`, `reaction_removed`, `status_change`, `channel_viewed`, `user_added`, `user_removed`.
  - `posted` payload currently sets `channel_display_name`, `channel_name`, `team_id` to empty placeholders.
- Notes: Event name coverage is good, but some payload fidelity remains partial.

- Endpoint or component: Missing websocket scheduled-post events
- Source path: `backend/src/realtime/events.rs` and repo-wide search
- Source lines: `backend/src/realtime/events.rs:36-103`
- Observed behavior:
  - No internal event enum/mapping found for `scheduled_post_created|updated|deleted`.
- Notes: Mobile websocket constants include scheduled-post events (`../mattermost-mobile/app/constants/websocket.ts:101-103`).
