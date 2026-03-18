# Design: WebSocket Auth Expiry Enforcement

**Status**: ✅ IMPLEMENTED (2026-03-17)
**Spec**: WebSocket Auth Expiry Enforcement
**Created**: 2026-03-13

---

## Problem Statement

Current behavior allows an already-established WebSocket connection to keep receiving realtime events after the JWT access token has expired. Observed by user: without refreshing the page, the web UI continues receiving messages after token expiry.

This is a security and session-integrity bug. Expired credentials must not continue to authorize realtime delivery. Additionally, when JWT expiry is reached, the web UI must not keep showing stale authenticated screens; it must clear session-scoped UI data and navigate to login.

---

## Goals

1. Ensure active WebSocket sessions stop receiving events as soon as JWT `exp` is reached
2. Apply consistent behavior to both websocket entrypoints:
   - `/api/v4/websocket`
   - `/ws` (legacy endpoint)
3. Force web UI logout UX when JWT expires:
   - Clear authenticated token/session
   - Clear session-scoped messaging state from stores
   - Navigate to login screen immediately
4. Keep behavior backward-compatible for valid, unexpired tokens

---

## Non-Goals

1. No refresh-token or silent re-auth feature in this task
2. No JWT revocation list implementation in this task
3. No changes to unrelated HTTP auth flows

---

## Upstream Compatibility Evidence

Server-side reference (`../mattermost`):
- `../mattermost/server/channels/app/platform/web_conn.go:880` (`ShouldSendEvent`) checks `IsAuthenticated()` before sending each event
- `../mattermost/server/channels/app/platform/web_conn.go:778` (`IsBasicAuthenticated`) reloads session when expiry passes; invalid/expired session results in unauthenticated state

Implication: upstream does not continue delivering websocket events to expired sessions.

Current Rustchat behavior:
- `backend/src/api/websocket_core.rs:363` validates token only when extracting initial user id
- `backend/src/api/v4/websocket.rs:203` and `backend/src/api/ws.rs:87` run the socket loop without any token-expiry deadline enforcement afterward
- `frontend/src/composables/useWebSocket.ts` reconnects on close but does not treat auth-expiry close as a forced logout flow

---

## Architecture

### Backend Changes

1. **Extend websocket auth parsing** to capture JWT claims (`sub`, `exp`) instead of only user id
2. **Thread token expiration timestamp** through websocket connection setup for both `/api/v4/websocket` and `/ws`
3. **Add expiry deadline branch** in the websocket run loop:
   - When reached, close connection with policy/auth close code
   - Perform normal cleanup (hub unregister, presence/offline handling, actor disconnect)

### Frontend Changes

1. **JWT-expiry timer** in auth lifecycle (login + rehydrate paths)
   - On timer expiration, execute centralized logout path
   - Centralized logout clears token/cookie/user and resets session-scoped stores before redirect
2. **WebSocket close handling**:
   - If socket closes with auth/policy violation for expired token, trigger centralized logout immediately
   - Do not continue reconnect loop in expired-token scenario

---

## Implementation Outline

1. Extend websocket auth parsing to capture JWT claims (`sub`, `exp`) instead of only user id
2. Thread token expiration timestamp through websocket connection setup for:
   - `/api/v4/websocket`
   - `/ws`
3. In the websocket run loop, add an expiry deadline branch:
   - When reached, close connection with policy/auth close code
   - Perform normal cleanup (hub unregister, presence/offline handling, actor disconnect)
4. Add frontend session-expiry enforcement:
   - Add JWT-expiry timer in auth lifecycle (login + rehydrate paths)
   - On timer expiration, execute one centralized logout path
   - Centralized logout must clear token/cookie/user and reset session-scoped stores before redirect
5. Add websocket close handling in frontend:
   - If socket closes with auth/policy violation for expired token, trigger centralized logout immediately
   - Do not continue reconnect loop in expired-token scenario
6. Add regression tests for claim extraction / expiry-aware auth plumbing where feasible

---

## Verification Plan

### Automated
```bash
cd backend && cargo test --no-fail-fast -- --nocapture
cd backend && cargo check
cd frontend && npm run build
```

### Manual
1. Start backend with short token lifetime: `cd backend && RUSTCHAT_JWT_EXPIRY_HOURS=1 cargo run`
2. Login in web UI, keep channel open, wait until token expiration boundary
3. Verify websocket disconnect occurs at/after expiry and no new realtime messages appear
4. Verify web UI immediately transitions to `/login` and previously visible channel/message content is cleared from authenticated view
5. Verify no reconnect storm continues with expired token (socket remains disconnected until new login)
6. Optional wire check: monitor websocket close in browser devtools Network tab

---

## Contract Impact

### In Scope
- WebSocket auth validation lifecycle
- WebSocket connection close behavior when auth expires
- Frontend auth-expiry detection and UX enforcement (logout + state clear + login redirect)

### Contract Changes
- Realtime auth semantics become stricter: expiry now applies during an already-open socket session, not only at handshake
- Expected close reason/code for expiry will be a policy/auth violation close from server side
- On JWT expiry, user-visible contract changes from "stale authenticated screen persists" to "forced transition to login with cleared session state"

### Out of Scope
- API response body shape changes
- Refresh-token rotation/re-issue

---

## Success Criteria

- ✅ Both `/api/v4/websocket` and `/ws` endpoints enforce token expiry
- ✅ Frontend detects auth expiry and triggers logout flow
- ✅ Close code 1008 (POLICY_VIOLATION) sent on auth expiry
