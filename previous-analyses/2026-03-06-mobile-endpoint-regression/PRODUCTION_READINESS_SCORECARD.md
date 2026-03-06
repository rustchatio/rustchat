# Production Readiness Scorecard

- P0 gaps: 1 (`/api/v4/system/ping` and system bootstrap routes unavailable)
- P1 gaps: 2 (`plugins`, `groups` routers empty)
- P2 gaps: 3 (`custom_profile`, `reports`, `access_control` routers empty)

Status: **NOT PRODUCTION READY**

Gate rationale:
- Fails required threshold (`P0` must be `0`).
- Mobile bootstrap journey is blocked.
