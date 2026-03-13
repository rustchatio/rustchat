# Gap Register

| Gap ID | Severity | Area | Current Behavior | Expected Behavior | Proposed Fix | Status |
|---|---|---|---|---|---|---|
| GAP-WS-AUTH-001 | High | WebSocket auth lifecycle | Active websocket continues to receive events after JWT expiry. | Realtime delivery stops immediately at token expiry. | Capture JWT `exp`, schedule runtime expiry close, cleanup connection. | Open |
| GAP-WS-AUTH-002 | Medium | Endpoint consistency (`/api/v4/websocket`, `/ws`) | Runtime auth enforcement missing in both loops and can diverge. | Same expiry enforcement on all websocket entrypoints. | Reuse shared claim extraction + pass expiry deadline to both handlers. | Open |
