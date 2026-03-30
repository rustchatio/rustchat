# TODOS

## Review

### Align Team-Member Presence Contracts

**What:** Align presence and status contract behavior across `/api/v4/teams/{team_id}/members` and `/api/v4/users/.../teams/members`, and document whether custom status belongs in those payloads or is intentionally websocket/detail-only.

**Why:** Equivalent team-member endpoints currently drift in presence semantics, which creates cross-surface inconsistencies for compatible clients and frontend consumers.

**Context:** This work was explicitly split out of the `ui-improvements-3` Lake 1 plan so the current branch can focus on live status propagation, reconnect correctness, shared summary batching, and regression coverage. Pick this up as Lake 2 after the current user-visible fix lands, and verify both endpoint families tell the same story for the same user set.

**Effort:** M
**Priority:** P1
**Depends on:** Lake 1 presence/status implementation on `ui-improvements-3`

### Harden Server-Side Custom-Status Expiry Semantics

**What:** Formalize and harden server-side custom-status expiry semantics, including cleanup timing, broadcast behavior, and regression tests for expiry without a manual REST read.

**Why:** Custom-status expiry is a distributed-state problem, and frontend timers alone cannot keep multiple clients in sync.

**Context:** The current review deliberately moved expiry-worker and parity work out of Lake 1 to keep the branch reviewable. This follow-up should define the backend source of truth for when custom status expires, how connected clients hear about it, and what guarantees hold across websocket reconnects and passive clients.

**Effort:** M
**Priority:** P1
**Depends on:** Lake 1 presence/status implementation on `ui-improvements-3`

## Completed
