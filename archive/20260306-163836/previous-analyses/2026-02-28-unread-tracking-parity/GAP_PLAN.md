# Gap Plan

## Architectural decisions (target)

### AD-1: Source of truth and cache roles
- Rustchat target path: `backend/src/api/v4/*`, `backend/src/services/unreads.rs`, Postgres schema/migrations
- Required behavior: Unread/read state must be durable, replayable, and MM-compatible.
- Current gap: Redis unread service currently acts as partial source for UI counts but stores only basic counters.
- Planned change:
  - Keep Postgres as source of truth for unread/read cursor and per-channel counters.
  - Use Redis as derived cache + fan-out helper with deterministic recompute.
  - Do not accept Redis-only writes for user unread state.
- Verification test:
  - Kill Redis during runtime; unread APIs should still return correct values from DB fallback.
  - Warm Redis from DB and verify equality against DB-derived counts.
- Status: planned

### AD-2: Redis unread schema v2 (full unread tuple)
- Rustchat target path: `backend/src/services/unreads.rs` (new v2 keyspace), optional background worker module
- Required behavior: Cache all MM-relevant unread fields.
- Current gap: v1 keys only store `unread_count` and team totals.
- Planned change:
  - User-channel hash key: `rc:unread:v2:uc:{user_id}:{channel_id}`
    - fields: `msg_count`, `msg_count_root`, `mention_count`, `mention_count_root`, `urgent_mention_count`, `last_viewed_at`, `manually_unread`, `version`
  - User-team hash key: `rc:unread:v2:ut:{user_id}:{team_id}`
    - fields: same aggregate subset + `version`
  - Dirty marker set: `rc:unread:v2:dirty:{user_id}` contains channel IDs needing recompute
  - Last channel seq cache: `rc:unread:v2:chan_seq:{channel_id}`
  - Remove blocking `KEYS` usage; use indexed keys and `SCAN` only for maintenance jobs.
- Verification test:
  - Unit tests for serialize/deserialize + atomic update script.
  - Load test to confirm no `KEYS` in hot path.
- Status: planned

### AD-3: Write ordering and consistency
- Rustchat target path: create/update unread flows in `backend/src/services/posts.rs`, `backend/src/api/v4/posts.rs`, `backend/src/api/v4/channels/view.rs`, `backend/src/api/v4/channels.rs`
- Required behavior: No cache or websocket state ahead of DB commit.
- Current gap: multiple independent update paths; some SQL failures ignored.
- Planned change:
  - Perform DB mutation transaction first.
  - After commit, update Redis via atomic operation.
  - Emit websocket event only after successful DB commit.
  - On Redis failure, mark dirty for recompute and continue.
- Verification test:
  - Transaction rollback tests must produce no websocket unread event.
  - Inject Redis failure and verify dirty-set reconciliation path.
- Status: planned

## Compatibility gap items (implementation plan)

### GP-1: Replace channel-level unread contract with post-level unread behavior
- Rustchat target path:
  - `backend/src/api/v4/channels.rs` (deprecate usage of `/members/{user}/set_unread`)
  - `backend/src/api/v4/posts.rs` (`set_post_unread`)
  - `frontend/src/components/channel/MessageItem.vue`
  - `frontend/src/components/channels/ChannelContextMenu.vue`
  - `frontend/src/api/posts.ts`
- Required behavior:
  - Manual unread in channels is anchored to a post (`/users/{user}/posts/{post}/set_unread`).
  - Sidebar channel “mark unread” chooses most recent post and uses same post API.
- Current gap:
  - UI calls channel-level unread endpoint.
  - Channel-level endpoint sets oldest/epoch unread rather than post boundary.
- Planned change:
  - Add post unread API call in frontend API client and wire message menu to post ID.
  - Change channel menu to fetch most recent visible post and call post unread.
  - Keep channel-level route only as compatibility shim if needed, internally delegating to post-unread with explicit policy.
- Verification test:
  - Web E2E: mark specific message unread -> divider appears above that message.
  - API test: post unread updates unread counts relative to selected post.
- Status: planned

### GP-2: Implement `collapsed_threads_supported` parsing and full response shape
- Rustchat target path: `backend/src/api/v4/posts.rs`, compatibility models
- Required behavior: Parse request body and return full `ChannelUnreadAt` fields expected by clients.
- Current gap:
  - Body not parsed.
  - Response missing `user_id`, root counters, urgent counters.
- Planned change:
  - Parse body with content-type tolerant parser (json/form).
  - Extend compat model to include:
    - `msg_count_root`, `mention_count_root`, `urgent_mention_count`, `user_id`
  - Compute non-CRT/CRT behavior deterministically and populate fields.
- Verification test:
  - API contract tests for body accepted/rejected forms.
  - Snapshot test of response JSON keys and types.
- Status: planned

### GP-3: Emit and map `post_unread` websocket event
- Rustchat target path: `backend/src/realtime/events.rs`, `backend/src/api/v4/posts.rs`, `backend/src/api/v4/websocket.rs`, frontend websocket handler
- Required behavior: `post_unread` websocket event with MM payload fields.
- Current gap:
  - No event enum variant.
  - No mapper case.
  - No frontend handler for `post_unread`.
- Planned change:
  - Add `EventType::PostUnread -> "post_unread"`.
  - Emit from post unread handler after DB update.
  - Add `map_envelope_to_mm` mapping for `post_unread`.
  - Add frontend handling path (or reuse existing unread update reducer with full payload support).
- Verification test:
  - Websocket integration test: event arrives for target user and contains required keys.
  - Mobile smoke: mark post unread reflects without refresh.
- Status: planned

### GP-4: Fix `/channels/members/{user}/view` contract
- Rustchat target path: `backend/src/api/v4/channels/view.rs`
- Required behavior:
  - Strict invalid-body handling (HTTP 400).
  - Validate IDs.
  - Return `{"status":"OK","last_viewed_at_times":{...}}`.
  - Sync read cursor semantics.
- Current gap:
  - Invalid body returns `OK`.
  - No `last_viewed_at_times`.
  - Updates only `channel_members.last_viewed_at`.
- Planned change:
  - Reuse shared body parser with validation.
  - Return structured view response.
  - Update associated read cursor and Redis cache in same flow.
- Verification test:
  - API test: garbage body -> 400.
  - API test: valid request -> `last_viewed_at_times` populated.
- Status: planned

### GP-5: Align `channel_reads` column usage and remove swallowed DB errors
- Rustchat target path:
  - `backend/src/api/v4/channels.rs`
  - possibly migration if canonical column should differ
- Required behavior: SQL must target real columns and errors must surface.
- Current gap:
  - Queries reference `last_viewed_at` in `channel_reads`, but schema defines `last_read_at`.
  - Current code suppresses SQL failures (`let _ = ...`).
- Planned change:
  - Standardize on schema column names (`last_read_at`).
  - Remove silent error swallowing from read/unread endpoints.
  - Add explicit error mapping and regression tests.
- Verification test:
  - Endpoint tests ensure channel read writes `channel_reads` row successfully.
  - Negative test (forced SQL error) returns error response, not silent success.
- Status: planned

### GP-6: Team unread APIs parity
- Rustchat target path: `backend/src/api/v4/users.rs`, unread service/repository layer
- Required behavior:
  - `/users/me/teams/unread`, `/users/{id}/teams/unread`, `/users/{id}/teams/{team}/unread` must return computed aggregates.
  - Include root/thread fields where contract expects them.
- Current gap: routes return empty arrays or zeros.
- Planned change:
  - Implement DB-backed aggregation + Redis cache read-through.
  - Include collapsed-thread counts if enabled.
- Verification test:
  - API tests with seeded unread data across multiple teams/channels.
- Status: planned

### GP-7: Channel member count semantics parity
- Rustchat target path:
  - `backend/src/mattermost_compat/models.rs`
  - `backend/src/mattermost_compat/mappers.rs`
  - `backend/src/api/v4/channels.rs`
  - `backend/src/api/v4/websocket.rs` reconnect snapshot builder
- Required behavior:
  - `channel_members.msg_count` is read-position style count (MM semantics), not unread count.
  - Include root/urgent fields in model as needed.
- Current gap:
  - Most responses hardcode `0`.
  - reconnect snapshot maps unread count into `msg_count`.
- Planned change:
  - Compute and expose correct member counters from DB.
  - Update compat structs to include missing fields.
  - Correct snapshot calculations.
- Verification test:
  - Reconnect snapshot integration test comparing channel member counts before/after unread operations.
- Status: planned

### GP-8: Remove frontend double-accounting path for unread increments
- Rustchat target path:
  - `frontend/src/stores/messages.ts`
  - `frontend/src/composables/useWebSocket.ts`
  - unread store implementation convergence
- Required behavior:
  - One authoritative update path per event (prefer server unread event payloads).
- Current gap:
  - `handleNewMessage` locally increments unread for non-current channels.
  - websocket listener also applies `unread_counts_updated`.
- Planned change:
  - Use server unread events as primary.
  - Keep local optimistic increment only behind feature flag or for legacy mode.
- Verification test:
  - Simulate posted + unread event sequence and assert no double increments.
- Status: planned

## Verification matrix (must-pass before rollout)

- Rustchat target path: backend + frontend test suites
- Required behavior: Contract and UX parity for unread lifecycle.
- Current gap: Missing dedicated regression tests for unread parity.
- Planned change:
  - Backend:
    - `/users/{u}/posts/{p}/set_unread` response and ws contract tests
    - `/channels/members/{u}/view` invalid-body and response-shape tests
    - team unread aggregate tests
  - Frontend:
    - message menu mark unread anchors correct message
    - manual unread prevents auto-read on focus/scroll
    - new-message divider placement tests
  - Compatibility smoke:
    - update scripts to assert `post_unread` websocket event fields
- Verification test:
  - `cargo test` backend suites
  - frontend unit/e2e flows for unread marker behavior
  - mm compatibility smoke scripts where relevant
- Status: planned

## Implementation sequence (PR-sized chunks)

### PR-1: Contract-safe fixes with lowest blast radius
- Rustchat target path:
  - `backend/src/api/v4/channels.rs`
  - `backend/src/api/v4/channels/view.rs`
- Required behavior:
  - Correct `channel_reads` column usage (`last_read_at`), stop swallowing SQL failures.
  - Add strict parsing and 400 behavior for invalid view body.
- Current gap:
  - Invalid SQL column names and silent error handling.
  - view endpoint returns `OK` for invalid body.
- Planned change:
  - Fix column names and error propagation.
  - Add response shape with `last_viewed_at_times`.
- Verification test:
  - New API tests for read/view error semantics.
- Status: planned

### PR-2: Add missing websocket contract (`post_unread`)
- Rustchat target path:
  - `backend/src/realtime/events.rs`
  - `backend/src/api/v4/posts.rs`
  - `backend/src/api/v4/websocket.rs`
  - `frontend/src/composables/useWebSocket.ts`
- Required behavior:
  - Emit, map, and consume `post_unread` with MM payload keys.
- Current gap:
  - Event missing end-to-end.
- Planned change:
  - Introduce `PostUnread` event and mapper branch; wire frontend store updates from this event.
- Verification test:
  - websocket integration test + frontend reducer test.
- Status: planned

### PR-3: Post unread API parity
- Rustchat target path:
  - `backend/src/api/v4/posts.rs`
  - `backend/src/mattermost_compat/models.rs`
  - `backend/src/mattermost_compat/mappers.rs`
- Required behavior:
  - Parse `collapsed_threads_supported`, return full `ChannelUnreadAt` shape.
- Current gap:
  - Partial response fields and missing request-body contract.
- Planned change:
  - Extend compat structs and post unread response builder with root/urgent/user fields.
- Verification test:
  - API contract snapshots for both CRT and non-CRT requests.
- Status: planned

### PR-4: Frontend action parity (mark unread from post)
- Rustchat target path:
  - `frontend/src/api/posts.ts`
  - `frontend/src/components/channel/MessageItem.vue`
  - `frontend/src/components/channels/ChannelContextMenu.vue`
  - `frontend/src/stores/messages.ts`
- Required behavior:
  - Message and channel menu unread actions use post-based unread anchor.
- Current gap:
  - UI calls channel-level unread endpoint.
- Planned change:
  - Add post unread API call and use latest relevant post ID in channel menu.
  - Remove local double increment path once server unread event path is trusted.
- Verification test:
  - UI e2e: mark unread from message and from channel menu.
- Status: planned

### PR-5: Team unread API implementation
- Rustchat target path:
  - `backend/src/api/v4/users.rs`
  - unread aggregation service module
- Required behavior:
  - Return real team unread aggregates with root/thread fields.
- Current gap:
  - Stubbed endpoints.
- Planned change:
  - Implement DB-first aggregation with Redis read-through cache.
- Verification test:
  - API tests with multi-team, multi-channel fixture data.
- Status: planned

### PR-6: Channel member count semantics + reconnect parity
- Rustchat target path:
  - `backend/src/api/v4/channels.rs`
  - `backend/src/api/v4/websocket.rs`
  - compat models/mappers
- Required behavior:
  - `channel_members.msg_count` semantics align with MM read-position model.
- Current gap:
  - zeroed fields and unread-vs-read semantic mismatch in reconnect.
- Planned change:
  - Populate accurate counters and fix reconnect snapshot mapping.
- Verification test:
  - reconnect snapshot parity test.
- Status: planned

### PR-7: Redis unread v2 introduction (dual-write)
- Rustchat target path:
  - `backend/src/services/unreads.rs`
  - optional worker module for reconciliation
- Required behavior:
  - Full unread tuple cache with no blocking keyspace operations.
- Current gap:
  - minimal counters and `KEYS` usage.
- Planned change:
  - Add v2 keyspace, dual-write from authoritative DB changes, dirty-set reconciliation.
- Verification test:
  - cache mismatch metric assertions + failover tests.
- Status: planned

## Rollout strategy (low risk)

- Rustchat target path: feature flags + phased deployment
- Required behavior: Introduce parity behavior without breaking existing clients.
- Current gap: unread logic split across v1/v4 and frontend stores.
- Planned change:
  1. Add new unread v2 backend path and Redis v2 dual-write (no behavior switch yet).
  2. Add `post_unread` emission/mapping and frontend handler behind feature flag.
  3. Switch frontend mark-unread actions to post-based API.
  4. Enable strict view validation and team unread APIs.
  5. Remove legacy channel-level unread path from UI usage; keep compatibility shim for transition window.
- Verification test:
  - Compare dual-write counters between old/new paths for a canary set of users.
  - Monitor mismatch metrics before full cutover.
- Status: planned

## Implementation update (2026-02-28)

Completed checks:
- PR-1 implemented:
  - `/channels/members/me/view` and `/channels/members/{user_id}/view` now require valid body parsing and return `{status,last_viewed_at_times}`.
  - `channel_reads` write paths fixed to `last_read_at` and no longer swallow SQL errors in read/unread handlers.
- PR-2 implemented:
  - Added realtime `post_unread` event variant and websocket v4 mapping.
  - Added websocket mapper unit test for exact `post_unread` payload keys.
- PR-3 implemented (core):
  - `POST /users/{user_id}/posts/{post_id}/set_unread` now parses `collapsed_threads_supported`.
  - Returns full unread payload with `team_id`, `user_id`, root and urgent fields.
  - Emits `post_unread` websocket event (config-gated).
- PR-4 implemented (core wiring):
  - Web UI message menu and channel menu now use post-based mark-unread anchor where possible.
  - Websocket client handles `post_unread` to apply server unread state.
- PR-5 implemented:
  - `/users/{user_id}/teams/unread` and `/users/{user_id}/teams/{team_id}/unread` now aggregate real values.
  - Supports `exclude_team` and `include_collapsed_threads`.
  - Applies `notify_props.mark_unread=mention` semantics for team message aggregation.
- PR-6 implemented:
  - reconnect `channel_members/channel_unreads` snapshot now derives unread counts from `channel_reads` position and includes root fields.
  - reconnect snapshot now computes `urgent_mention_count` (config-gated by `post_priority_enabled`).
  - `/channels/*/members*` and `/users/*/channel_members` responses now return computed MM-compatible channel member counters instead of hardcoded zeros.
  - compatibility mapper `From<ChannelMember> for mm::ChannelMember` now maps unread/read fields from model values (no hardcoded zeros).
- PR-7 implemented (rollout foundation):
  - Added Redis unread v2 keyspace support in `backend/src/services/unreads.rs`:
    - `rc:unread:v2:uc:{user}:{channel}`
    - `rc:unread:v2:ut:{user}:{team}`
    - `rc:unread:v2:dirty:{user}`
  - Added dual-write behavior to unread update paths (DB authoritative; Redis best-effort with dirty fallback).
  - Added v2 mismatch detection (`v1` vs `v2` unread count) and dirty marking.
  - Hardened v2 reads: on cache version/count mismatch, unread overview now recomputes from DB immediately and refreshes cache (dirty fallback retained).
  - Added periodic reconciliation worker (`run_unread_v2_reconciler`) and startup wiring behind `unread_v2_enabled`.
  - Removed blocking `KEYS` usage from `mark_all_as_read` (explicit key deletion by known team/channel ids).
- Data model:
  - Added migration for unread parity columns in `channel_members` (msg/mention/root/urgent, manual unread, last_update_at).
- Config gates:
  - Added unread domain flags: `unread_v2_enabled`, `post_unread_ws_enabled`, `team_unread_v2_enabled`, `collapsed_threads_enabled`, `post_priority_enabled`, `thread_auto_follow`.

Test evidence:
- Backend compile: `cd backend && cargo check` (pass).
- Backend targeted tests:
  - `cargo test map_envelope_to_mm_maps_post_unread_payload -- --nocapture` (pass).
  - `cargo test parses_view_request_json -- --nocapture` (pass).
  - `cargo test team_unread_includes_thread_urgent_mentions_when_enabled -- --nocapture` (pass).
  - `cargo test mm_channel_member_routes_return_computed_counts -- --nocapture` (pass).
- Frontend compile: `cd frontend && npm run build` (pass).
- Frontend unread path:
  - removed local unread-count increment in `messages` store; websocket unread events remain authoritative for unread totals (mention hinting remains local).

Remaining risks / not yet completed:
- `thread_urgent_mention_count` now computed via unread-thread reply scan (`@here` + mention predicate) when `post_priority_enabled=true`; no separate persisted urgent-thread counter exists yet.
- CRT/reply-unread behavior is approximated by config gates but not yet parity-complete with Mattermost thread-update semantics across all matrices.
- V2 read cutover SLO is not yet enforced by explicit metric thresholds; mismatch safety is now immediate DB recompute + cache refresh (plus dirty-reconcile).
