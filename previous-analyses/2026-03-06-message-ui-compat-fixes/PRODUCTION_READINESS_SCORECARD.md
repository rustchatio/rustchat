# Production Readiness Scorecard

- P0 gaps: 0
- P1 gaps: 3
  - Missing global edit-limit enforcement in v4 edit endpoints
  - Missing edit-limit keys in config/client compatibility payload
  - Unread bell indicator store desync in WebUI
- P2 gaps: 3
  - Missing date separators in message timeline
  - Missing edited indicator rendering in WebUI
  - Reaction toggle parity needs explicit verification after wiring

Status: **NOT PRODUCTION READY**

Gate rationale:
- `P1` gaps are non-zero and touch compatibility-sensitive behavior used by mobile/web clients.
