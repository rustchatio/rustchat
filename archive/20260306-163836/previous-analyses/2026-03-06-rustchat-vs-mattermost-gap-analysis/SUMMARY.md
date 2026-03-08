# Summary

- Topic: RustChat vs Mattermost parity and production-readiness gap analysis
- Date: 2026-03-06
- Scope: Server architecture, API v4 endpoint coverage, websocket/calls behavior, and Mattermost mobile core user journeys
- Upstream baseline:
  - `../mattermost` commit: `56f51d7df254ef0dea727976ca3b4b2175ba2fab`
  - `../mattermost-mobile` commit: `0a3e701ba5d536174a7e5b1be0adcfc1b8959165`
  - `rustchat` commit: `47cf28137992fb543285e8009294503760931dfc`

## Compatibility contract (current state)

- API v4 method+path coverage against upstream OpenAPI baseline: `438 / 570` (`76.8%`), `132` missing, `117` RustChat-only extras.
- Core mobile journey route coverage (selected 74 high-traffic routes): direct route gap closed for `PUT /api/v4/posts/{post_id}` in this iteration; sampled set now covered at route level.
- Calls plugin surface required by mobile app is implemented under `/api/v4/plugins/com.mattermost.calls/*` and corresponding websocket custom events are handled.

## Headline findings

1. Core chat + team/channel + calls mobile paths are mostly present, but full Mattermost parity is not yet met.
2. A high volume of uncovered endpoints still sits in plugin/enterprise/admin surfaces (`plugins`, `groups`, and remaining deep `license`/`reports`/other sub-contracts) even after closing the current high-impact slices.
3. Post route contract mismatches identified for mobile/core flows in this iteration are closed (`PUT /posts/{post_id}`, `GET /posts/{post_id}/reveal`, `DELETE /posts/{post_id}/burn`), with temporary POST shims retained for burn-on-read routes.
4. Plugin marketplace admin-status contract now has parity for `GET/POST /api/v4/plugins/marketplace/first_admin_visit` (permission gate + persisted system status + websocket event mapping), and missing `POST /api/v4/plugins/marketplace` route is now present with explicit 501 semantics.
5. Plugin management permission/status parity is now aligned on high-impact surfaces: non-admin access is forbidden where upstream expects sysconsole permissions, including overlap paths under `/plugins/com.mattermost.calls/enable|disable`, while admin unsupported mutations still return explicit 501.
6. Plugin marketplace query semantics are now aligned for `remote_only` and conflicting `local_only+remote_only` filter handling.
7. Reports method parity slice is closed for canonical report routes (`POST /reports/users/export`, `POST /reports/posts`) with permission and bad-request validation guards aligned to upstream intent.
8. License method parity slice is closed for canonical admin license routes (`POST/DELETE /license`, `GET /license/renewal`) with stronger permission enforcement and MM-compatible response envelopes/shapes.
9. Access-control method and permission parity slice is closed for high-impact routes (`PUT /access_control_policies`, `DELETE /{policy_id}`, `GET /{policy_id}/activate`, `DELETE /{policy_id}/unassign`, `POST /cel/visual_ast`, `PUT /activate`) with temporary legacy shims retained.
10. Custom profile attributes route-coverage slice is closed for missing high-impact routes (`POST /fields`, `PATCH/DELETE /fields/{field_id}`, `GET /group`) with strict permission gates and explicit 501 mutation stubs.
11. Duplicate group patch implementation was removed by dropping non-upstream `PUT /groups/{group_id}` and keeping canonical `PUT /groups/{group_id}/patch` only.
12. Groups syncable routes now expose explicit canonical team/channel paths for link/get/list/patch surfaces, replacing generic syncable-path declarations.
13. Custom profile values parity slice is now aligned for mobile/admin clients: `PATCH /custom_profile_attributes/values` now returns Mattermost-style map payloads with JSON value type preservation, `PATCH /users/{user_id}/custom_profile_attributes` is now available, and `GET /users/{user_id}/custom_profile_attributes` now preserves stored value types.
14. Verification environment now has deterministic integration bootstrap (`docker-compose.integration.yml` + `RUSTCHAT_TEST_*`), and smoke scripts now require explicit `BASE` with compatibility preflight.

## Production decision

- Status: **Not ready for production parity claim**.
- Reason: parity backlog (remaining P2 contract/coverage gaps) and incomplete full-stack smoke evidence against a dedicated live RustChat target in this iteration.

## Open questions

- Which parity target profile is release-blocking: "mobile core only" or "full upstream v4 including enterprise/plugin surfaces"?
- Should RustChat explicitly defer enterprise-only routes in a published compatibility profile, or implement stubs with exact upstream semantics?
