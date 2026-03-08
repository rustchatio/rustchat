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
- Note: these counts are from the initial matrix snapshot and were not recomputed after the incremental fixes implemented on 2026-03-06 (`G-001..G-004`, `G-008..G-017`).

### Top resource gaps by volume

| Resource | Baseline | Matched | Missing | Coverage |
| :--- | ---: | ---: | ---: | ---: |
| `plugins` | 56 | 10 | 46 | 17.9% |
| `users` | 114 | 102 | 12 | 89.5% |
| `groups` | 21 | 11 | 10 | 52.4% |
| `access_control_policies` | 15 | 9 | 6 | 60.0% |
| `custom_profile_attributes` | 6 | 2 | 4 | 33.3% |
| `license` | 5 | 2 | 3 | 40.0% |

Note: the `access_control_policies` method-parity slice (`G-011`), `custom_profile_attributes` route-coverage slice (`G-012`), groups route de-dup slice (`G-013`), groups syncable canonical-path slice (`G-014`), custom-profile values contract slice (`G-015`), plugin management permission slice (`G-016`), and plugin marketplace query-semantics slice (`G-017`) were implemented after this snapshot; the table above is intentionally left as historical baseline.

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

5. Reports canonical method parity implemented.
- Upstream route/handler: `../mattermost/api/v4/source/reports.yaml:154-284`, `../mattermost/server/channels/api4/report.go:19-23,77-110,162-252`.
- RustChat: `backend/src/api/v4/reports.rs` now provides canonical POST methods for:
  - `/reports/users/export`,
  - `/reports/posts`,
  with `manage_system` guard + validation checks.

6. License canonical method parity implemented.
- Upstream route/handler: `../mattermost/api/v4/source/system.yaml:532-689`, `../mattermost/server/channels/api4/license.go:20-26`.
- RustChat: `backend/src/api/v4/system.rs` now provides canonical:
  - `POST /license`,
  - `DELETE /license`,
  - `GET /license/renewal`,
  with `manage_system` enforcement and temporary legacy shims retained.

7. Access-control canonical method parity implemented for high-impact routes.
- Upstream route/handler: `../mattermost/api/v4/source/access_control.yaml:1-607`, `../mattermost/server/channels/api4/access_control.go:16-36`.
- RustChat: `backend/src/api/v4/access_control.rs` now provides canonical:
  - `PUT /access_control_policies`,
  - `DELETE /access_control_policies/{policy_id}`,
  - `GET /access_control_policies/{policy_id}/activate`,
  - `DELETE /access_control_policies/{policy_id}/unassign`,
  - `POST /access_control_policies/cel/visual_ast`,
  - `PUT /access_control_policies/activate`,
  with `manage_system` enforcement and temporary legacy shims retained.

8. Custom profile attributes route-coverage parity implemented for high-impact routes.
- Upstream route/handler: `../mattermost/api/v4/source/custom_profile_attributes.yaml:1-320`, `../mattermost/server/channels/api4/custom_profile_attributes.go:14-24`.
- RustChat: `backend/src/api/v4/custom_profile.rs` now provides:
  - `POST /custom_profile_attributes/fields`,
  - `PATCH /custom_profile_attributes/fields/{field_id}`,
  - `DELETE /custom_profile_attributes/fields/{field_id}`,
  - `GET /custom_profile_attributes/group`,
  with `manage_system` enforcement on field-management routes and explicit 501 mutation stubs.

9. Groups duplicate patch-route implementation removed.
- Upstream route/handler: `../mattermost/api/v4/source/groups.yaml:111-231`, `../mattermost/server/channels/api4/group.go:30-35`.
- RustChat: `backend/src/api/v4/groups.rs` now only exposes canonical patch at:
  - `PUT /groups/{group_id}/patch`,
  and no longer exposes non-upstream `PUT /groups/{group_id}`.

10. Groups syncable routes aligned to explicit canonical team/channel paths.
- Upstream route/handler: `../mattermost/api/v4/source/groups.yaml:285-549`, `../mattermost/server/channels/api4/group.go:38-63`.
- RustChat: `backend/src/api/v4/groups.rs` now explicitly exposes:
  - `POST/DELETE /groups/{group_id}/teams/{team_id}/link`,
  - `POST/DELETE /groups/{group_id}/channels/{channel_id}/link`,
  - `GET /groups/{group_id}/teams/{team_id}`,
  - `GET /groups/{group_id}/channels/{channel_id}`,
  - `GET /groups/{group_id}/teams`,
  - `GET /groups/{group_id}/channels`,
  - `PUT /groups/{group_id}/teams/{team_id}/patch`,
  - `PUT /groups/{group_id}/channels/{channel_id}/patch`.

11. Custom-profile values and user-target patch contract aligned to upstream map semantics.
- Upstream route/handler: `../mattermost/server/channels/api4/custom_profile_attributes.go:15-25,207-377`.
- Mobile client dependency: `../mattermost-mobile/app/client/rest/custom_profile_attributes.ts:21-35`.
- RustChat: `backend/src/api/v4/custom_profile.rs` now provides:
  - map-shaped `PATCH /custom_profile_attributes/values` responses with preserved JSON value types,
  - canonical `PATCH /users/{user_id}/custom_profile_attributes` route,
  - `GET /users/{user_id}/custom_profile_attributes` JSON type-preserving decode.

12. Plugin management permission/status semantics aligned on high-impact routes.
- Upstream route/handler: `../mattermost/server/channels/api4/plugin.go:16-40,46-239,319-421`.
- RustChat:
  - `backend/src/api/v4/plugins.rs` now enforces system-manage permission on plugin management/read/status surfaces.
  - `backend/src/api/v4/calls_plugin/mod.rs` now applies the same permission gate for overlap compatibility routes `/plugins/com.mattermost.calls/enable|disable`.
- Resulting behavior: non-admin callers now get `403` where expected; admin callers still receive explicit `501` on unimplemented mutation flows.

13. Plugin marketplace query semantics aligned on high-impact filters.
- Upstream route/handler: `../mattermost/server/channels/api4/plugin.go:278-317,348-365`.
- RustChat (`backend/src/api/v4/plugins.rs`) now aligns:
  - `remote_only=true` allows authenticated non-admin marketplace reads,
  - `local_only=true&remote_only=true` returns internal server error parity,
  - default (non-remote-only) path keeps system-manage permission gate.

## Operational verification findings

- `cargo check` passes.
- Frontend production build passes (`npm run build`).
- Router overlap guard test passes (`cargo test v4_router_builds_without_overlaps -- --nocapture`).
- Targeted backend parity integration suites pass with deterministic profile (`docker-compose.integration.yml` + `RUSTCHAT_TEST_*`):
  - `api_v4_post_routes` (9/9),
  - `api_v4_channels_all` (2/2),
  - `api_v4_plugins_dialogs` (6/6),
  - `api_v4_reports_routes` (3/3),
  - `api_v4_license_routes` (3/3),
  - `api_v4_access_control_routes` (2/2),
  - `api_v4_custom_profile_routes` (4/4),
  - `api_v4_groups_syncables` (7/7 full suite pass),
  - plus focused groups route de-dup guard (`v4_group_patch_requires_canonical_patch_route`, filtered pass 1/1).
- Smoke scripts now fail fast unless explicit `BASE` + compatibility preflight requirements are met (prevents false-positive target selection).

## Server-side severity view

- P1:
  - None open in current register.
- P2:
  - `G-005` Broad plugin/admin/enterprise endpoint delta.
