# Previous Analyses Index

Consolidated on: 2026-02-11

## Summary of Prior Analysis

### 0. Latest re-validation (recommended starting point)
Source set: `2026-02-11-compat-revalidation/`

Key conclusions:
- Commands and dialogs are still partial for strict Mattermost-mobile parity (`POST /commands` missing, `/actions/dialogs/submit` returns `501`).
- Scheduled-post routes exist and align on `/posts/schedule*`, but list response envelope differs from Mattermost.
- Core websocket event names are mapped, but payload fidelity and scheduled-post websocket events need follow-up checks.
- This re-validation supersedes conflicting command conclusions in older 2026-02-07 reports.

### 1. API v4 baseline (Mattermost compatibility posture)
Source set: `2026-02-07-api-v4-baseline/`

Key conclusions:
- Rustchat targets Mattermost compatibility version `10.11.10` with `X-MM-COMPAT: 1` on `/api/v4` responses.
- Core mobile-critical flows were reported as implemented: login/bootstrap, teams/channels basics, posts, websocket, and calls plugin surface.
- Router fallback strategy is explicit Mattermost-style `501` for unsupported `/api/v4/*` routes.
- A large set of enterprise/admin endpoints was flagged as partial/stubbed and not fully behavior-compatible.

### 2. Mobile client requirements and expectation docs
Source set: `2026-02-07-mobile-client-requirements/`

Key conclusions:
- Minimum startup and messaging endpoints were tracked as implemented in checklist form.
- WebSocket support was partially complete: `posted`/`typing` implemented, several events marked TODO (`status_change`, `post_edited`, `reaction_*`, etc.).
- Mobile integration notes emphasize strict behavior around file uploads (`localPath`, `/files/{id}/preview`, presigned URL host correctness).

### 3. Compatibility audits, scoring, and commands deep-dive
Source set: `2026-02-07-compat-audit-reports/`

Key conclusions:
- Reported compatibility score: `91.54%` (`119/130` endpoints implemented in inventory).
- 11 endpoints marked not implemented, with scheduled posts identified as the highest-impact missing feature set.
- Commands analysis includes both:
  - a full-gap matrix showing major parity gaps, and
  - a closure report claiming full coverage.
- These two command conclusions conflict and require re-validation against current code before planning implementation.

## Migration Map (Old Path -> New Path)

- `docs/api_v4_compatibility_report.md` -> `previous-analyses/2026-02-07-api-v4-baseline/api_v4_compatibility_report.md`
- `docs/mattermost-compat.md` -> `previous-analyses/2026-02-07-api-v4-baseline/mattermost-compat.md`
- `docs/mattermost-v4-comparison.md` -> `previous-analyses/2026-02-07-api-v4-baseline/mattermost-v4-comparison.md`
- `docs/mattermost_mobile_knowhow.md` -> `previous-analyses/2026-02-07-api-v4-baseline/mattermost_mobile_knowhow.md`
- `docs/mattermost_compat/README.md` -> `previous-analyses/2026-02-07-mobile-client-requirements/README.md`
- `docs/mattermost_compat/minimum_mobile_endpoints.md` -> `previous-analyses/2026-02-07-mobile-client-requirements/minimum_mobile_endpoints.md`
- `docs/mattermost_compat/websocket_events.md` -> `previous-analyses/2026-02-07-mobile-client-requirements/websocket_events.md`
- `docs/mattermost_compat/mobile_client_matrix.md` -> `previous-analyses/2026-02-07-mobile-client-requirements/mobile_client_matrix.md`
- `backend/compat/reports/commands_canonical_behavior.md` -> `previous-analyses/2026-02-07-compat-audit-reports/commands_canonical_behavior.md`
- `backend/compat/reports/commands_gap.md` -> `previous-analyses/2026-02-07-compat-audit-reports/commands_gap.md`
- `backend/compat/reports/commands_gap_matrix.md` -> `previous-analyses/2026-02-07-compat-audit-reports/commands_gap_matrix.md`
- `backend/compat/reports/commands_implementation_plan.md` -> `previous-analyses/2026-02-07-compat-audit-reports/commands_implementation_plan.md`
- `backend/compat/reports/commands_mobile_inventory.md` -> `previous-analyses/2026-02-07-compat-audit-reports/commands_mobile_inventory.md`
- `backend/compat/reports/commands_test_plan.md` -> `previous-analyses/2026-02-07-compat-audit-reports/commands_test_plan.md`
- `backend/compat/reports/compat_score.md` -> `previous-analyses/2026-02-07-compat-audit-reports/compat_score.md`
- `backend/compat/reports/failures.md` -> `previous-analyses/2026-02-07-compat-audit-reports/failures.md`
- `backend/compat/reports/optional_features.md` -> `previous-analyses/2026-02-07-compat-audit-reports/optional_features.md`

## Next Iteration Note

Use `previous-analyses/_TEMPLATE/` for all new analyses so future work stays centralized.
