# Agentic Skills Report: Mobile Alignment and Development Throughput

## Executive Summary

Rustchat already has a strong compatibility governance base (`AGENTS.md` and parity skills), but mobile alignment is still under-instrumented in day-to-day execution.
The main opportunity is to add skills that convert existing tooling (trace replay, contracts, smoke scripts, push pipeline) into mandatory and repeatable agent workflows.

## Repository Signals Reviewed

- Existing governance and skills:
  - `AGENTS.md`
  - `.agents/skills/*/SKILL.md`
- Mobile compatibility tooling:
  - `backend/compat/README.md`
  - `backend/compat/replay_traces.py`
  - `backend/compat/tests/contract_validation.rs`
  - `backend/compat/inventory_endpoints.csv`
  - `scripts/mm_mobile_smoke.sh`
  - `scripts/mm_compat_smoke.sh`
- Mobile push/calls integration docs and services:
  - `docs/PUSH_NOTIFICATIONS.md`
  - `docs/VOIP_PUSH_IMPLEMENTATION_COMPLETE.md`
  - `docs/VOIP_PUSH_FIXES_SUMMARY.md`
  - `backend/src/services/push_notifications.rs`
  - `push-proxy/src/main.rs`
  - `push-proxy/src/apns.rs`
  - `push-proxy/src/fcm.rs`

## Current Strengths

- Compatibility-first workflow already exists and is explicit.
- Mobile-focused parity skills already exist:
  - `mattermost-mobile-compatibility`
  - `mm-mobile-journey-parity`
  - `mm-websocket-calls-parity`
- Smoke scripts enforce `X-MM-COMPAT: 1` preflight and baseline API checks.
- Compatibility inventory and replay tooling are present in `backend/compat/`.

## Gaps Limiting Mobile Alignment

1. Mobile smoke checks are shallow
- `scripts/mm_mobile_smoke.sh` validates ping/config/license/version and optional `/users/me`, but not full journeys (threads, reactions, files, reconnect, push).

2. Trace replay and contracts are not operationalized as a required gate
- `backend/compat/replay_traces.py` and contract tests exist, but there is no dedicated skill that makes trace replay and schema diffing a standard release step.

3. Endpoint inventory evidence is incomplete
- `backend/compat/inventory_endpoints.csv` contains many rows where `tested_trace` and `tested_contract` are `no`, reducing confidence for mobile parity claims.

4. Push/VoIP path is complex and high-risk without a dedicated workflow skill
- Mobile ringing behavior spans backend + push-proxy + platform payload specifics.
- Current docs are rich, but agents need an explicit checklist skill to avoid regressions.

5. No dedicated workflow for websocket resilience outside calls-specific checks
- Existing websocket skill is calls-focused; reconnect/order/presence/session resync across general mobile flows still needs systematic validation.

## Recommended New Skills (Priority Order)

## 1) `mobile-trace-replay-gate` (P0)

Purpose:
- Make trace replay and response diff validation mandatory for mobile-sensitive changes.

Why now:
- Tooling already exists in `backend/compat/`, but workflow discipline is missing.

Core workflow:
1. Choose trace set (`backend/compat/traces/*`).
2. Run replay against target backend.
3. Produce diff report and compatibility summary.
4. Fail gate if status/schema mismatches exceed threshold.

Expected outputs:
- `previous-analyses/<iteration>/TRACE_REPLAY_REPORT.md`
- machine-readable replay output artifact path

## 2) `mobile-contract-schema-gate` (P0)

Purpose:
- Enforce response/error schema parity for mobile-consumed endpoints.

Why now:
- `backend/compat/tests/contract_validation.rs` exists, but not framed as a required skill gate.

Core workflow:
1. Map changed endpoints to contract schemas.
2. Run contract tests.
3. Require explicit pass/fail evidence per endpoint family.

Expected outputs:
- contract pass/fail matrix
- endpoint-to-schema mapping evidence

## 3) `mobile-push-voip-parity` (P0)

Purpose:
- Protect call ringing and mobile push behavior (Android/iOS) from regressions.

Why now:
- Push behavior spans `backend/src/services/push_notifications.rs` and `push-proxy/src/*` with platform-specific payload rules.

Core workflow:
1. Validate payload shape per platform (`sub_type`, `call_uuid`, channel/post IDs, sound/topic headers).
2. Validate fallback behavior (APNS unavailable, token mismatch/unregistered).
3. Validate manual smoke command paths.

Expected outputs:
- push parity checklist with platform-specific evidence
- known-risk registry for call notification regressions

## 4) `mobile-websocket-resilience` (P1)

Purpose:
- Cover reconnect, sequence, missed-event recovery, and presence/session continuity for mobile.

Why now:
- Current calls-focused websocket parity is necessary but not sufficient for broader mobile state sync.

Core workflow:
1. Define reconnect interruption scenarios.
2. Validate event ordering and recovery behavior.
3. Capture server + client expectations with evidence.

Expected outputs:
- reconnect scenario matrix
- websocket behavior gap list

## 5) `mobile-e2e-journey-smoke` (P1)

Purpose:
- Expand smoke from handshake-level to real user journeys.

Why now:
- Existing script checks do not reflect real mobile usage depth.

Core workflow:
1. Login/session
2. Team/channel discovery
3. Post/thread/reaction flow
4. File upload/download path
5. Call join/leave baseline

Expected outputs:
- journey-level pass/fail report with commands

## 6) `endpoint-inventory-evidence-maintainer` (P2)

Purpose:
- Keep `backend/compat/inventory_endpoints.csv` trustworthy.

Why now:
- Large inventory exists but evidence columns are frequently incomplete.

Core workflow:
1. Detect changed endpoint surfaces.
2. Update inventory rows and evidence flags.
3. Block parity claims if evidence flags are stale.

Expected outputs:
- updated inventory with `tested_trace/tested_contract` evidence

## 7) `mobile-release-readiness-pack` (P2)

Purpose:
- Aggregate all mobile gates into one release decision artifact.

Why now:
- Existing production readiness gate is broad; mobile-specific go/no-go should be explicit.

Core workflow:
1. Collect outputs from trace, contracts, journey smoke, websocket, and push checks.
2. Compute decision thresholds.
3. Emit `GO/NO-GO` with blocker list.

Expected outputs:
- `MOBILE_RELEASE_SCORECARD.md`

## Skill Adoption Sequence

Phase 1 (immediate):
- `mobile-trace-replay-gate`
- `mobile-contract-schema-gate`
- `mobile-push-voip-parity`

Phase 2:
- `mobile-websocket-resilience`
- `mobile-e2e-journey-smoke`

Phase 3:
- `endpoint-inventory-evidence-maintainer`
- `mobile-release-readiness-pack`

## Success Metrics

- Reduction of mobile regressions escaping to QA.
- Increase in `tested_trace` and `tested_contract` coverage in `inventory_endpoints.csv`.
- Consistent mobile go/no-go artifacts per release candidate.
- Lower time-to-diagnose for push and websocket issues.

## Implementation Note for Agentic Workflow

Each new skill should remain concise and operational:
- required inputs
- strict workflow steps
- explicit outputs
- failure handling
- definition of done

This keeps token usage low while giving agents deterministic execution behavior.
