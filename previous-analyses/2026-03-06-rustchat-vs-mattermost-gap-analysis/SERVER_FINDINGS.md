# Server Findings

## Evidence snapshots

- Upstream API architecture is mux/subrouter based with explicit resource routers under `/api/v4` and handler init sequence: `../mattermost/server/channels/api4/api.go:177-385`.
- Upstream documents layered auth/permission handlers and contract stability in `api4` package docs: `../mattermost/server/channels/api4/doc.go:5-54`.
- Upstream websocket endpoint and reconnect semantics are implemented on `/api/v4/websocket`: `../mattermost/server/channels/api4/websocket.go:52-125`.
- RustChat builds v4 by merging many Axum routers plus fallback `501` compatibility envelope: `backend/src/api/v4/mod.rs:71-210`.
- RustChat websocket handler supports token auth, connection_id/sequence_number, and custom action handling: `backend/src/api/v4/websocket.rs:54-113`, `backend/src/api/v4/websocket.rs:595-699`.
- RustChat calls plugin surface is mounted under `/api/v4/plugins/com.mattermost.calls/*`: `backend/src/api/v4/calls_plugin/mod.rs:48-175`.

## Endpoint coverage findings

- Baseline extraction (`tools/mm-compat` workflow in temp copy):
  - Upstream OpenAPI endpoints: `570`
  - RustChat v4 extracted endpoints: `555`
  - Exact method+path matches: `438`
  - Missing from RustChat: `132`
  - RustChat extras: `117`
- Note: these counts are from the initial matrix snapshot and were not recomputed after the incremental fixes implemented on 2026-03-06 (`G-001..G-004`, `G-008`).

### Top resource gaps by volume

| Resource | Baseline | Matched | Missing | Coverage |
| :--- | ---: | ---: | ---: | ---: |
| `plugins` | 56 | 10 | 46 | 17.9% |
| `users` | 114 | 102 | 12 | 89.5% |
| `groups` | 21 | 11 | 10 | 52.4% |
| `access_control_policies` | 15 | 9 | 6 | 60.0% |
| `custom_profile_attributes` | 6 | 2 | 4 | 33.3% |
| `license` | 5 | 2 | 3 | 40.0% |

### Contract mismatches resolved in this iteration

1. `PUT /api/v4/posts/{post_id}` implemented with path/body `id` parity checks.
- Upstream contract: `../mattermost/api/v4/source/posts.yaml:206-263`.
- RustChat: `backend/src/api/v4/posts.rs` (`PUT /posts/{post_id}`).

2. Burn-on-read verb parity aligned.
- Upstream contracts: `GET /posts/{post_id}/reveal` and `DELETE /posts/{post_id}/burn` (`../mattermost/api/v4/source/posts.yaml:1165-1255`).
- RustChat: `backend/src/api/v4/posts.rs` now supports canonical verbs, with temporary `POST` shims retained.

3. `GET /api/v4/channels` implemented for system/admin listing semantics.
- Upstream contract/handler: `../mattermost/api/v4/source/channels.yaml:1-69`, `../mattermost/server/channels/api4/channel.go:getAllChannels`.
- RustChat: `backend/src/api/v4/channels.rs` (`get_all_channels`).

4. Plugin marketplace first-admin-visit contract implemented.
- Upstream route/handler: `../mattermost/api/v4/source/plugins.yaml:431-476`, `../mattermost/server/channels/api4/plugin.go:42-43,433-491`.
- RustChat: `backend/src/api/v4/plugins.rs` now provides:
  - `GET /plugins/marketplace/first_admin_visit` (`System` shape),
  - `POST /plugins/marketplace/first_admin_visit` (persist + websocket event),
  - `POST /plugins/marketplace` route with explicit 501 contract.

## Operational verification findings

- `cargo check` passes.
- Frontend production build passes (`npm run build`).
- Targeted backend parity integration suites pass with deterministic profile (`docker-compose.integration.yml` + `RUSTCHAT_TEST_*`):
  - `api_v4_post_routes` (9/9),
  - `api_v4_channels_all` (2/2),
  - `api_v4_plugins_dialogs` (5/5).
- Smoke scripts now fail fast unless explicit `BASE` + compatibility preflight requirements are met (prevents false-positive target selection).

## Server-side severity view

- P1:
  - None open in current register.
- P2:
  - `G-005` Broad plugin/admin/enterprise endpoint delta.
