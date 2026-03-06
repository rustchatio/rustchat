# Task Plan

## Task
Restore mobile compatibility regression introduced by last build where API v4 router modules were replaced by empty stubs.

## Implementation Status
- [x] Root cause identified with code evidence.
- [x] Restored affected files from archived pre-regression snapshot:
  - `backend/src/api/v4/system.rs`
  - `backend/src/api/v4/plugins.rs`
  - `backend/src/api/v4/groups.rs`
  - `backend/src/api/v4/access_control.rs`
  - `backend/src/api/v4/custom_profile.rs`
  - `backend/src/api/v4/reports.rs`
  - `backend/src/api/v4/calls_plugin/mod.rs`
- [x] Confirmed restored routes are present in active source.

## Verification Status

### Automated
1. `cd backend && cargo check`
- Result: PASS

2. `cd backend && cargo clippy --all-targets --all-features -- -D warnings`
- Result: FAIL (pre-existing repository-wide clippy violations unrelated to this fix and predating this change)
- Representative failure surfaces include `src/api/admin.rs`, `src/api/oauth.rs`, `src/services/*`, `src/models/*`, etc.

3. `cd backend && cargo test --no-fail-fast -- --nocapture`
- Result: PARTIAL
  - Unit tests: PASS (`125 passed, 0 failed` for `src/lib.rs` test suite)
  - Integration tests: FAIL due environment prerequisites not available (`RUSTCHAT_TEST_DATABASE_URL` bootstrap fails; postgres/redis test services unavailable/auth mismatch).

### Mobile compatibility smoke
- Not executed automatically because target server endpoint was not available in local environment context.
- Manual acceptance command:
  - `BASE=<your-server-endpoint> ./scripts/mm_mobile_smoke.sh`

## Readiness
- Code-level regression fix is applied.
- Environment-dependent validation (integration DB-backed tests and remote endpoint smoke) must be run in your deployment/test environment.
