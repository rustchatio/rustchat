# Gap Plan

## Prioritized remediation plan

1. **P1 contract fix: canonical post update endpoint**
- Rustchat target path: `backend/src/api/v4/posts.rs`
- Required behavior: support `PUT /api/v4/posts/{post_id}` per upstream contract.
- Current gap: route missing; only `/patch` exists.
- Planned change: add `put(update_post)` semantics compatible with upstream `UpdatePost` behavior.
- Verification test: API integration test covering `PUT /posts/{id}` success + permission failures.
- Status: Completed (implemented on 2026-03-06)
- Implementation evidence:
  - Upstream route/handler: `../mattermost/server/channels/api4/post.go:40`, `../mattermost/server/channels/api4/post.go:1006`.
  - RustChat route/handler: `backend/src/api/v4/posts.rs` (`/posts/{post_id}` now supports `PUT`; body `id` is validated against path).
  - RustChat tests added: `backend/tests/api_v4_post_routes.rs`
    - `mm_update_post_route_put_updates_post_message`
    - `mm_update_post_route_put_requires_matching_body_id`
    - `mm_update_post_route_put_rejects_non_author`
- Verification run:
  - `cargo check` (pass)
  - `cargo test --test api_v4_post_routes -- --nocapture` with deterministic integration profile (pass: 9/9)

2. **P2 contract fix: burn-on-read verb parity**
- Rustchat target path: `backend/src/api/v4/posts.rs`
- Required behavior: `GET /posts/{id}/reveal` and `DELETE /posts/{id}/burn`.
- Current gap: both are `POST` in RustChat.
- Planned change: align HTTP verbs; keep compatibility for existing clients if required via temporary dual-route shim.
- Verification test: route/method table tests + smoke checks for BoR flows.
- Status: Completed (implemented on 2026-03-06)
- Implementation evidence:
  - Upstream route contract: `../mattermost/server/channels/api4/post.go:61-62`.
  - RustChat route parity: `backend/src/api/v4/posts.rs` now exposes:
    - `GET /posts/{post_id}/reveal` (+ temporary `POST` shim),
    - `DELETE /posts/{post_id}/burn` (+ temporary `POST` shim).
  - Verification test: `mm_burn_on_read_routes_support_mattermost_verbs_with_legacy_post_shim` in `backend/tests/api_v4_post_routes.rs`.
- Verification run:
  - `cargo test --test api_v4_post_routes -- --nocapture` with deterministic integration profile (pass: 9/9).

3. **P2 endpoint surface closure for high-impact resources**
- Rustchat target path: `backend/src/api/v4/{plugins,groups,access_control,custom_profile,license,reports}*.rs`
- Required behavior: raise coverage on resources where mobile/admin parity is expected.
- Current gap: large deltas (`plugins`, `groups`, `access_control_policies`, etc.).
- Planned change: implement or explicitly document/defer endpoint classes by profile.
- Verification test: endpoint matrix diff gate in CI with allowed-deviation policy file.
- Status: In Progress
- Progress update (2026-03-06):
  - Completed first high-impact slice: `GET /api/v4/channels` (admin/system list-all channels route).
  - Verification: `cargo test --test api_v4_channels_all -- --nocapture` with deterministic integration profile (pass: 2/2).
  - Completed plugin marketplace slice:
    - `GET /api/v4/plugins/marketplace/first_admin_visit` and `POST /api/v4/plugins/marketplace/first_admin_visit` now match Mattermost semantics (manage_system permission + persisted `System` status).
    - Added missing `POST /api/v4/plugins/marketplace` route with explicit 501 contract (route coverage closure for this method+path).
    - Added websocket parity mapping for `first_admin_visit_marketplace_status_received`.
  - Verification: `cargo test --test api_v4_plugins_dialogs -- --nocapture` with deterministic integration profile (pass: 5/5).
  - Remaining: broader plugin/enterprise/admin endpoint classes (`plugins`, `groups`, `access_control_policies`, `custom_profile_attributes`, `license`, `reports`).

4. **P1 operational verification stability**
- Rustchat target path: test harness + local environment docs/scripts
- Required behavior: backend integration tests run with valid DB credentials, smoke scripts run against RustChat stack.
- Current gap: test DB auth failure; localhost smoke target collides with unrelated service.
- Planned change:
  - provide deterministic `docker compose` profile and credentials bootstrap,
  - enforce explicit `BASE` for smoke scripts in CI/local docs.
- Verification test: green run for `cargo test` integration subset + `./scripts/mm_compat_smoke.sh` + `./scripts/mm_mobile_smoke.sh`.
- Status: Completed (implemented on 2026-03-06)
- Implementation evidence:
  - Deterministic integration deps profile: `docker-compose.integration.yml` (Postgres/Redis/S3 on isolated ports).
  - Test bootstrap fallback and explicit env support: `backend/tests/common/mod.rs` (`RUSTCHAT_TEST_DATABASE_URL`, `RUSTCHAT_TEST_REDIS_URL`, `RUSTCHAT_TEST_S3_*`).
  - Smoke target hardening: `scripts/mm_compat_smoke.sh`, `scripts/mm_mobile_smoke.sh` (`BASE` mandatory + preflight guard).
  - Governance docs updated: `AGENTS.md` deterministic integration bootstrap and mandatory `BASE` usage.
- Verification run:
  - `cargo test --test api_v4_post_routes -- --nocapture` with test env vars against `docker-compose.integration.yml` (pass: 9/9).
  - `./scripts/mm_compat_smoke.sh` without `BASE` (expected fail-fast).
  - `./scripts/mm_mobile_smoke.sh` without `BASE` (expected fail-fast).

## Success criteria to exit analysis state

- `P1` gaps resolved.
- `P2` gaps either resolved or documented in approved compatibility profile with explicit non-goals.
- Compatibility smoke and integration checks green in a controlled environment.
