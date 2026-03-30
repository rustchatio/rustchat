# Production Readiness Scorecard

- Topic: Presence/status contract closure on `ui-improvements-3`
- Date: 2026-03-29
- Branch head reviewed: `d7490ba1`
- Decision: `NO-GO`

## Severity Totals

- Source of truth status: blocked
- Latest full parity bundle in-repo is stale:
  - [GAP_REGISTER.md](/Users/scolak/Projects/rustchat/previous-analyses/2026-03-06-mobile-endpoint-regression/GAP_REGISTER.md)
  - [GAP_PLAN.md](/Users/scolak/Projects/rustchat/previous-analyses/2026-03-06-mobile-endpoint-regression/GAP_PLAN.md)
  - [ENDPOINT_MATRIX.md](/Users/scolak/Projects/rustchat/previous-analyses/2026-03-06-mobile-endpoint-regression/ENDPOINT_MATRIX.md)
  - [UX_JOURNEYS.md](/Users/scolak/Projects/rustchat/previous-analyses/2026-03-06-mobile-endpoint-regression/UX_JOURNEYS.md)
- Latest branch-adjacent gap register is partial and not a full release artifact:
  - [GAP_REGISTER.md](/Users/scolak/Projects/rustchat/previous-analyses/2026-03-13-ws-token-expiry-auth-enforcement/GAP_REGISTER.md)
- Because there is no fresh branch-specific parity bundle, `P0/P1` totals cannot be certified for this release candidate.

## Gate Results

### Backend Compile
- Result: `PASS`
- Evidence:
  - `cd /Users/scolak/Projects/rustchat/backend && cargo check`

### Backend Lint
- Result: `PASS`
- Evidence:
  - `cd /Users/scolak/Projects/rustchat/backend && cargo clippy --all-targets --all-features -- -D warnings`

### Backend Tests
- Result: `PARTIAL`
- Evidence:
  - `cd /Users/scolak/Projects/rustchat/backend && cargo test --test api_v4_mobile_presence -- --nocapture` passed (`9 passed`)
  - `cd /Users/scolak/Projects/rustchat/backend && cargo test --no-fail-fast -- --nocapture` exposed at least one environment-blocked failure:
    - `tests/api_mattermost.rs::mm_files_upload`
    - runtime error chain includes `InvalidAccessKeyId` from S3 upload setup
- Assessment:
  - This is an infra/storage readiness blocker for release validation, not evidence that the presence/status branch regressed its target behavior.

### Frontend Build
- Result: `PASS`
- Evidence:
  - `cd /Users/scolak/Projects/rustchat/frontend && npm run build`
- Note:
  - Existing Vite warning remains for `frontend/src/stores/calls.ts` mixed dynamic/static import chunking.

### Frontend Verification
- Result: `PASS`
- Evidence already available from branch work:
  - `cd /Users/scolak/Projects/rustchat/frontend && npm run test:unit`
  - `cd /Users/scolak/Projects/rustchat/frontend && npm run test:e2e`
  - Result: `60 passed`

### Compatibility Smoke
- Result: `BLOCKED`
- Evidence:
  - `./scripts/mm_compat_smoke.sh` failed immediately because `BASE` was not set
  - `./scripts/mm_mobile_smoke.sh` failed immediately because `BASE` was not set
  - `curl http://127.0.0.1:3000/api/v1/health` failed and nothing was listening on port `3000`

## Decision Logic

`NO-GO` because the required release thresholds are not satisfied:

- mandatory verification is not fully green
- compatibility smoke checks were not run against a live target
- the branch does not have a fresh parity-analysis bundle to certify `P0 == 0` and `P1 == 0`

## Remediation List

1. Create a fresh branch-specific parity bundle in `previous-analyses/` with `GAP_REGISTER.md`, `GAP_PLAN.md`, `ENDPOINT_MATRIX.md`, `UX_JOURNEYS.md`, and `SUMMARY.md`.
2. Stand up a real target environment and rerun:
   - `BASE=<running-rustchat-url> ./scripts/mm_compat_smoke.sh`
   - `BASE=<running-rustchat-url> ./scripts/mm_mobile_smoke.sh`
3. Fix or provision the S3 test environment so `cargo test --no-fail-fast -- --nocapture` is green, including the failing upload path in `api_mattermost::mm_files_upload`.
4. After those are green, rerun this production gate and update the decision.
