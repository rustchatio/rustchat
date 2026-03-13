# Task Plan

## 2026-03-13 WebSocket Token Expiry Enforcement

### Task
- Stop realtime websocket message delivery after JWT token expiration, even without page refresh.
- Force web UI to clear authenticated screen state and navigate to login immediately when JWT expires.

### Implementation Status
- [x] Added shared websocket auth context (`user_id` + `expires_at`) parsing in `backend/src/api/websocket_core.rs`.
- [x] Enforced token-expiry runtime disconnect in `/api/v4/websocket` loop (`backend/src/api/v4/websocket.rs`).
- [x] Enforced token-expiry runtime disconnect in legacy `/ws` loop (`backend/src/api/ws.rs`).
- [x] Added regression tests for claims-to-websocket-auth expiry mapping.
- [x] Added JWT expiry timer in frontend auth lifecycle to trigger forced logout at token `exp` (`frontend/src/stores/auth.ts`).
- [x] Added centralized frontend session cleanup on logout (messages/channels/unreads/presence/teams/preferences/calls/UI) before login redirect.
- [x] Updated websocket client close handling to treat auth-expiry close as forced logout and suppress reconnect loops (`frontend/src/composables/useWebSocket.ts`).

### Verification Status
1. `cd backend && cargo check`
- Result: PASS

2. `cd backend && cargo clippy --all-targets --all-features -- -D warnings`
- Result: PASS

3. `cd backend && cargo test claims_to_websocket_auth -- --nocapture`
- Result: PASS

4. `cd frontend && npm run build`
- Result: PASS

### Manual Verification Commands
1. Start backend with a short token lifetime:
   - `cd backend && RUSTCHAT_JWT_EXPIRY_HOURS=1 cargo run`
2. Connect to websocket with a valid token and observe forced close after expiry boundary:
   - `npx wscat -c ws://127.0.0.1:3000/api/v4/websocket -s "<JWT_TOKEN>"`
3. Web UI check:
   - Login, keep a channel open past token expiration, verify UI redirects to `/login` and channel/message UI state is cleared.
   - Confirm websocket reconnect attempts stop while token is expired.

### Readiness
- Ready for user acceptance testing.
- Expected behavior: when JWT expires, active websocket session is closed and realtime events stop.

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

---

## 001-platform-foundation: RustChat Platform Foundation

### Specification
**Status**: ✅ Approved (2026-03-13)  
**Location**: `specs/001-platform-foundation/spec.md`  
**User Stories**: 8 (4 P1, 4 P2)  
**Stages**: 4-stage SaaS maturity (Ad-Hoc → Reactive → Proactive → Strategic)

### Implementation Plan
**Status**: ✅ Created (2026-03-13)  
**Location**: `specs/001-platform-foundation/plan.md`  
**Phases**: 5 (Foundation, Auth/State, Security/Compliance, AI/APIs, Frontend)

### Constitution Compliance
| Principle | Status |
|-----------|--------|
| I. Contract Compatibility | ✅ |
| II. Plan-First Validation | ✅ |
| III. Evidence-Backed Analysis | ⏳ (upstream analysis pending) |
| IV. Test Gates | ✅ |
| V. Security Discipline | ✅ |
| VI. Mobile-First | ✅ |
| VII. Product Contract | ✅ |
| VIII. Feature Safety | ✅ |
| IX. Test Coverage | ✅ |
| X. Data Sovereignty | ✅ |
| XI. Stateless Tier | ✅ |
| XII. Memory Safety | ✅ |
| XIII. Zero-Trust | ✅ |
| XIV. Federated Identity | ✅ |
| XV. Strict RBAC | ✅ |
| XVI. Audit Logging | ✅ |
| XVII. Privacy by Design | ✅ |
| XVIII. Accessibility | ✅ |
| XIX. DevSecOps | ✅ |

### Current Status
- ✅ Specification approved
- ✅ Implementation plan created
- ✅ Tasks generated (196 tasks across 11 phases)
- ⏳ **READY FOR**: Implementation → Validation

### Next Actions
1. Begin Phase 0: Foundation & Tooling
   - `cd backend && cargo init`
   - Setup Docker, CI/CD
2. Checkpoint at each stage gate
3. Run `/speckit.validate` after Phase 11

### Task Summary
| Phase | Stage | Tasks | Focus Area |
|-------|-------|-------|------------|
| 0 | Ad-Hoc | T001-T017 | Containerization, CI/CD |
| 1 | Reactive | T018-T042 | Redis, OIDC/SAML, Metrics |
| 2 | Core | T043-T062 | Messaging, Mattermost API |
| 3 | Proactive | T063-T080 | RBAC, Audit Logging |
| 4 | Proactive | T081-T093 | GDPR, Data Lifecycle |
| 5 | Proactive | T094-T110 | OpenSearch, Kafka, HA |
| 6 | Strategic | T111-T123 | MCP Protocol |
| 7 | Strategic | T124-T134 | A2A Protocol |
| 8 | Strategic | T135-T150 | OpenAPI, Webhooks, SDKs |
| 9 | Frontend | T151-T169 | Solid.js, Accessibility |
| 10 | DevSecOps | T170-T184 | Security, K8s, Monitoring |
| 11 | Validation | T185-T197 | Load Testing, Chaos |

**Total Tasks**: 196
**Estimated Duration**: 16-20 weeks (2-3 devs parallel)

### Success Criteria (from spec)
- SC-001: Platform sustains 200,000 concurrent users with p99 latency < 150ms
- SC-002: Morning auth spike (30/sec, 150k/90min) completes with <0.1% failure rate
- SC-003: Kafka fan-out achieves <5ms latency for 10,000-member channel broadcasts
- SC-004: MCP/A2A agent integration passes security audit (zero critical findings)
- SC-005: OpenAPI 3.1.1 spec generates functional SDKs for TypeScript, Python, Rust
- SC-006: Webhook delivery achieves 99.9% success rate with Standard Webhooks compliance
- SC-007: GDPR Right to Erasure completes within 30 days, cryptographically verified
- SC-008: Accessibility audit passes WCAG 2.1 AA and BITV 2.0 with zero violations
- SC-009: `cargo audit` shows zero high/critical CVEs at release
- SC-010: Zero-downtime deployment achieved via blue-green strategy
