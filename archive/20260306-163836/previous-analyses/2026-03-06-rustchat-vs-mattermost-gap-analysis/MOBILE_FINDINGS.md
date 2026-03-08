# Mobile Findings

## Mobile architecture behavior (upstream)

- Login bootstrap initializes config/license, registers security settings, and starts websocket client: `../mattermost-mobile/app/actions/remote/entry/login.ts:30-42`.
- Reconnect flow performs full entry sync, batches models to local DB, reloads calls config/state, then fetches posts/read state: `../mattermost-mobile/app/actions/websocket/index.ts:52-115`, `../mattermost-mobile/app/actions/websocket/index.ts:117-157`.
- Channel sync path expects pagination and category hydration across teams with batched local persistence: `../mattermost-mobile/app/actions/remote/channel.ts:420-474`.
- Post sync path is optimized for incremental fetching and local write batching: `../mattermost-mobile/app/actions/remote/post.ts:300-338`, `../mattermost-mobile/app/actions/remote/post.ts:408-445`.

## Mobile endpoint dependencies (sampled)

- Auth/session: `../mattermost-mobile/app/client/rest/users.ts:122-220`
- Team/channel discovery and membership: `../mattermost-mobile/app/client/rest/teams.ts:64-125`, `../mattermost-mobile/app/client/rest/channels.ts:196-272`
- Posts/reactions/ack/thread operations: `../mattermost-mobile/app/client/rest/posts.ts:100-246`
- Files upload/preview/link/search: `../mattermost-mobile/app/client/rest/files.ts:55-103`
- Calls REST surface: `../mattermost-mobile/app/products/calls/client/rest.ts:41-170`
- Calls websocket custom events and handlers: `../mattermost-mobile/app/products/calls/connection/websocket_client.ts:112-143`, `../mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts:54-211`

## Mobile compatibility findings

1. Core mobile journey route set is broadly implemented in RustChat (73/74 sampled routes).
2. The single direct sampled route gap is `PUT /api/v4/posts/{post_id}`.
3. Calls plugin route surface expected by mobile is present in RustChat (`version`, `config`, `channels`, `turn-credentials`, host controls, recording, dismiss).
4. Websocket custom calls action/event handling exists in RustChat (`custom_com.mattermost.calls_*`) and includes plain-text mobile action fallback for mute/unmute/raise-hand/leave.
5. Custom profile values contract expected by mobile is now aligned for high-impact flows: map-shaped `PATCH /custom_profile_attributes/values` response and canonical `PATCH /users/{user_id}/custom_profile_attributes` route are present, with JSON value type preservation on reads.

## Ease-of-use implications

- Positive:
  - Core sign-in, channel browsing, posting, read/unread updates, file flows, and calls primitives are present.
- Risk:
  - Method-level contract drifts can break feature flags or edge paths silently (example: reveal/burn verbs).
  - Local verification confidence is currently reduced because integration tests and smoke checks are not fully green in this environment.

## Mobile severity view

- P1:
  - None open in current register.
- P2:
  - `G-005` broad plugin/admin/enterprise endpoint backlog still blocks "almost all Mattermost features" target.
- P2/P3:
  - Non-core enterprise/plugin parity backlog does not block basic mobile chat/calls, but remains incompatible with full upstream parity claims.
