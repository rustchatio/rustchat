# Implementation Plan (Advanced): [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

## Summary

[Primary requirement and implementation approach]

## Technical Context

**Language/Version**: [e.g., Rust 1.75 or NEEDS CLARIFICATION]
**Primary Dependencies**: [e.g., Axum, Tokio, SQLx]
**Storage**: [e.g., PostgreSQL, Redis, S3]
**Testing**: [e.g., cargo test, smoke scripts]
**Target Platform**: [e.g., Linux containerized services]
**Performance Goals**: [domain-specific target]
**Constraints**: [latency, compatibility, security, deployment]

## Constitution and Gate Checks

*Must pass before implementation starts.*

- [ ] Compatibility impact identified (API/websocket/calls/pagination/errors)
- [ ] If compatibility-sensitive: analysis folder and artifacts planned under `previous-analyses/YYYY-MM-DD-<topic>/`
- [ ] `SPEC.md` includes contract impact and verification approach
- [ ] User approval checkpoint exists before implementation
- [ ] Verification plan includes `cargo clippy`, `cargo test`, and smoke checks when applicable
- [ ] Security/operations impact reviewed (secrets, prod claims, hardening)

## Phase -1: Pre-Implementation Gates

- [ ] Simplicity gate: avoid speculative architecture and unnecessary indirection
- [ ] Integration-first gate: contract/integration checks defined before coding
- [ ] Risk gate: error semantics, rollback/recovery, and compatibility boundaries defined
- [ ] Any exception has explicit rationale and approval owner

## Execution Waves

### Wave 1 (Foundation)

[Blocking prerequisites, schema/contracts, compatibility analysis artifacts]

### Wave 2 (Core)

[Primary user-story implementation and contract-conformant behavior]

### Wave 3 (Hardening)

[Regression tests, compatibility smoke checks, release-readiness updates]

## Project Structure

```text
specs/[###-feature]/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
└── tasks.md
```

**Structure Decision**: [Describe selected source layout for backend/frontend/push-proxy]

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., extra abstraction] | [current need] | [reason] |
