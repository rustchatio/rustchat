---
name: production-readiness-gate
description: Release readiness gate for RustChat compatibility work, including severity thresholds, verification checklist, and go/no-go criteria.
license: MIT
---

# Production Readiness Gate

Use this skill when asked "are we production ready?" or before parity release decisions.

## Trigger Conditions

- Any request for production readiness assessment.
- Any release candidate touching compatibility-sensitive surfaces.
- Any claim of near-full feature parity.

## Required Inputs

- Latest `GAP_REGISTER.md` and `GAP_PLAN.md`.
- Latest `ENDPOINT_MATRIX.md` and `UX_JOURNEYS.md`.
- Build/test/smoke command outputs for the target environment.

## Workflow

1. Compute severity totals (`P0`..`P3`) from gap register.
2. Run/collect mandatory verification checks:
   - backend compile/tests
   - frontend production build
   - compatibility smoke scripts
3. Score each gate (`PASS`, `PARTIAL`, `FAIL`) with concrete evidence.
4. Decide `GO` or `NO-GO` using thresholds:
   - `P0 == 0`
   - `P1 == 0`
   - mandatory checks green
5. Produce remediation list for failed gates.

## Expected Outputs

- `previous-analyses/<iteration>/PRODUCTION_READINESS_SCORECARD.md`
- `previous-analyses/<iteration>/SUMMARY.md` decision update

## Command Checklist

```bash
cd backend && cargo check
cd backend && cargo test --no-fail-fast -- --nocapture
cd frontend && npm run build
./scripts/mm_compat_smoke.sh
./scripts/mm_mobile_smoke.sh
```

## Failure Handling

- If environment failures are infra-related (DB credentials, wrong stack target), mark as blocked and list exact blocker.
- Do not convert blocked/failed checks into a green decision.

## Definition of Done

- Scorecard includes gate results, blockers, and final decision.
- Decision is explicit (`GO`/`NO-GO`) with objective criteria.
- Next actions are ordered and tied to gap IDs.
