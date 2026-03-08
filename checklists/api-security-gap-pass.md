# API + Security Requirements Gap Pass

**Date**: 2026-03-08
**Checklist**: /Users/scolak/Projects/rustchat/checklists/api-security.md
**Spec Evaluated**: /Users/scolak/Projects/rustchat/SPEC.md
**Companion Context**: /Users/scolak/Projects/rustchat/task_plan.md

## Result Summary

- Total checklist items evaluated: 30
- Pass: 2
- Partial: 8
- Gap/Fail: 20
- Overall readiness of requirements quality: Not release-ready

## Priority Gaps (Actionable)

### P0 (Must fix before compatibility/security sign-off)

1. Missing explicit error contract for edit-policy denial.
- Checklist: CHK004, CHK012
- Current evidence: Spec names over-limit failure but does not define status code, error ID, or payload schema (`SPEC.md:81-82`).
- Required requirement text: Exact HTTP status, error ID, message semantics, and response schema for deny paths on both edit endpoints.

2. Missing authorization matrix per contract-sensitive surface.
- Checklist: CHK002, CHK018
- Current evidence: Endpoints/events listed without role/permission requirements (`SPEC.md:38-43`).
- Required requirement text: For each endpoint/event, define who can call/receive, and deny behavior.

3. Missing websocket payload contract detail.
- Checklist: CHK007
- Current evidence: Event names are listed, but required payload fields/order/compat constraints are absent (`SPEC.md:42`).
- Required requirement text: Required fields, optional fields, and compatibility constraints for each event payload.

4. Missing explicit recovery requirements for transient sync failures.
- Checklist: CHK019, CHK022
- Current evidence: Manual checks cover happy-path outcomes only (`SPEC.md:79-87`).
- Required requirement text: Expected retry/resync/refresh behavior after dropped, duplicated, or out-of-order events.

### P1 (High value; should be fixed for robust implementation and review)

5. Edit-policy semantics are underspecified.
- Checklist: CHK005, CHK006, CHK020
- Current evidence: Mentions disabled/enabled/time-limited but omits defaults, units, boundary values, and cutoff semantics (`SPEC.md:8,67-69`).
- Required requirement text: Canonical mode values, default mode, unit and range, equality behavior at exact cutoff, clock-skew assumptions.

6. No explicit non-functional security requirements.
- Checklist: CHK023, CHK024
- Current evidence: Security constraints are implicit, not declarative (`SPEC.md` overall).
- Required requirement text: Integrity and audit requirements for edits/reactions/read-state mutations (required audit fields and retention policy hook).

7. No explicit requirement IDs/traceability model.
- Checklist: CHK015
- Current evidence: Narrative sections only.
- Required requirement text: Introduce FR/AC IDs and map contract surfaces to acceptance checks.

8. Environment prerequisites are verification notes, not requirements.
- Checklist: CHK027, CHK030
- Current evidence: Dependencies appear in task status but are not normative requirements (`task_plan.md:27-39`).
- Required requirement text: Required test/smoke preconditions and release decision rule when prerequisites are unmet.

### P2 (Quality hardening)

9. Ambiguity around "immediately".
- Checklist: CHK028
- Current evidence: "update immediately" is unbounded (`SPEC.md:8,18,66`).
- Required requirement text: Deterministic latency/event-order expectation.

10. Potential scope boundary conflict not explicitly resolved.
- Checklist: CHK029
- Current evidence: "No large frontend architecture migration" may conflict with required event/store wiring (`SPEC.md:24,66,69`).
- Required requirement text: Explicitly allow minimal wiring changes while prohibiting broad refactors.

11. Performance expectations for websocket-heavy flows are absent.
- Checklist: CHK025
- Current evidence: No measurable NFR thresholds.
- Required requirement text: At least one measurable responsiveness target for unread/reaction/edit update propagation.

## Items Assessed as Pass

- CHK009: Problem/Goal/Implementation sections are directionally aligned.
- CHK010: Contract-sensitive surfaces and evidence references are largely consistent.

## Suggested Spec Patch Blocks

Add sections to `SPEC.md`:
- `## API Contract Requirements (Normative)`
- `## Security and Audit Requirements (Normative)`
- `## Error Semantics and Permission Matrix`
- `## Recovery and Event Ordering Requirements`
- `## Traceability Matrix (FR/AC IDs)`

