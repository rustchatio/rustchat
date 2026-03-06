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

### Top resource gaps by volume

| Resource | Baseline | Matched | Missing | Coverage |
| :--- | ---: | ---: | ---: | ---: |
| `plugins` | 56 | 10 | 46 | 17.9% |
| `users` | 114 | 102 | 12 | 89.5% |
| `groups` | 21 | 11 | 10 | 52.4% |
| `access_control_policies` | 15 | 9 | 6 | 60.0% |
| `custom_profile_attributes` | 6 | 2 | 4 | 33.3% |
| `license` | 5 | 2 | 3 | 40.0% |

### Contract mismatches confirmed by source

1. Missing `PUT /api/v4/posts/{post_id}` in RustChat.
- Upstream contract exists: `../mattermost/api/v4/source/posts.yaml:206-263`.
- RustChat defines only `GET/DELETE` on `/posts/{post_id}` and uses `/posts/{post_id}/patch` for update: `backend/src/api/v4/posts.rs:40-45`.

2. Method mismatch for reveal endpoint.
- Upstream: `GET /api/v4/posts/{post_id}/reveal`: `../mattermost/api/v4/source/posts.yaml:1165-1214`.
- RustChat: `POST /posts/{post_id}/reveal`: `backend/src/api/v4/posts.rs:54`.

3. Method mismatch for burn endpoint.
- Upstream: `DELETE /api/v4/posts/{post_id}/burn`: `../mattermost/api/v4/source/posts.yaml:1216-1255`.
- RustChat: `POST /posts/{post_id}/burn`: `backend/src/api/v4/posts.rs:55`.

4. `GET /api/v4/channels` (system listing) exists upstream but not implemented in RustChat router.
- Upstream: `../mattermost/api/v4/source/channels.yaml:1-69`.
- RustChat routes include `POST /channels` only for root collection: `backend/src/api/v4/channels.rs:109-113`.

## Operational verification findings

- `cargo check` passes.
- Frontend production build passes (`npm run build`).
- Backend integration test suite fails in this environment due Postgres auth (`password authentication failed for user "rustchat"`), so compatibility regression confidence is incomplete.
- Local smoke scripts against `http://localhost:3000` fail because that port is currently serving a different stack and proxying to unresolved host `rafineri-api`.

## Server-side severity view

- P1:
  - `G-001` Missing `PUT /api/v4/posts/{post_id}`.
- P2:
  - `G-002` Reveal method mismatch (`GET` vs `POST`).
  - `G-003` Burn method mismatch (`DELETE` vs `POST`).
  - `G-004` Missing `GET /api/v4/channels`.
  - `G-005` Broad plugin/admin/enterprise endpoint delta.
