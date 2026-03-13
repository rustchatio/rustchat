# Architecture Gaps

## GAP-WS-AUTH-001: One-time websocket token validation

- Current state:
  - Token is validated at websocket handshake/auth challenge time only.
  - Connection lifecycle does not track token expiry (`exp`) afterward.
- Risk:
  - Expired credentials keep receiving realtime events until socket reconnect/manual refresh.
- Upstream parity:
  - Mattermost checks websocket auth state during event delivery (`ShouldSendEvent` -> `IsAuthenticated`).
- Required change:
  - Carry claims (including `exp`) into websocket runtime and enforce close at expiry boundary.

## GAP-WS-AUTH-002: Inconsistent protection across websocket endpoints

- Current state:
  - `/api/v4/websocket` and `/ws` both validate auth only on entry, but have separate runtime loops.
- Risk:
  - Partial fixes can leave one endpoint vulnerable.
- Required change:
  - Implement shared expiry-aware auth plumbing and apply to both endpoints.
