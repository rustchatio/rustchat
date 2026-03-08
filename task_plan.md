# Task Plan

## Task
Small compatibility-aligned messaging fixes:
- Show date separators for older messages (not only time) in WebUI message list.
- Enforce global message edit policy (`disabled`, `enabled`, or time-limited like 30 minutes) and keep edited UX consistent.
- Make reaction click behavior toggle correctly for the current user (add on first click, remove on second).
- Clear WebUI top-ring notification indicator when unseen messages are viewed/read.

## Implementation Status
- [x] Added timeline date separators in WebUI message list rendering (`frontend/src/components/channel/MessageList.vue`).
- [x] Exposed optional post edit timestamps in frontend post API typings (`frontend/src/api/posts.ts`) for edited-state handling.
- [x] Aligned unread indicator source in global header to shared unread store (`frontend/src/components/layout/GlobalHeader.vue`).
- [x] Added server setting form field for global post edit window (`frontend/src/views/admin/ServerSettings.vue`).
- [x] Included `post_edit_time_limit_seconds` in frontend config defaults (`frontend/src/features/config/stores/configStore.ts`).
- [x] Applied formatting-consistent updates in backend post update handlers (`backend/src/api/posts.rs`, `backend/src/api/v4/posts.rs`) after policy logic integration.

## Verification Status

### Automated
1. `cd backend && cargo check`
- Result: PASS

2. `cd frontend && npm run build`
- Result: PASS

3. `cd backend && cargo clippy --all-targets --all-features -- -D warnings`
- Result: PASS (all clippy warnings fixed across the codebase)

4. `cd backend && cargo test --no-fail-fast -- --nocapture`
- Result: PARTIAL
  - Unit tests in `src/lib.rs`: PASS (`125 passed, 0 failed`)
  - Many integration targets: FAIL due missing test DB bootstrap (`RUSTCHAT_TEST_DATABASE_URL` candidates unavailable/auth failure)

5. `BASE=http://localhost:3000 ./scripts/mm_compat_smoke.sh`
- Result: FAIL (target unavailable; preflight ping connection refused)

6. `BASE=http://localhost:3000 ./scripts/mm_mobile_smoke.sh`
- Result: FAIL (no `X-MM-COMPAT: 1` header observed because local target was unavailable)

## Manual Verification Commands
1. `./scripts/clippy_check.sh` - Run clippy validation
2. `BASE=<your-running-rustchat-url> ./scripts/mm_compat_smoke.sh`
3. `BASE=<your-running-rustchat-url> ./scripts/mm_mobile_smoke.sh`
4. `curl -si <your-running-rustchat-url>/api/v4/config/client | rg -n "AllowEditPost|PostEditTimeLimit"`

## Readiness
- Requested behavior changes are implemented in code.
- Final acceptance still depends on running smoke checks against a live compatible server endpoint and DB-backed integration test environment.
