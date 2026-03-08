<!--
Sync Impact Report
- Version change: 1.0.0 -> 1.0.1
- Modified principles:
  - I. Contract Compatibility First -> I. Contract Compatibility First (wording/style clarification)
  - II. Plan-First User Validation -> II. Plan-First User Validation (wording/style clarification)
  - III. Evidence-Backed Analysis -> III. Evidence-Backed Analysis (wording/style clarification)
  - IV. Test and Verification Gates -> IV. Test and Verification Gates (wording/style clarification)
  - V. Security and Operational Discipline -> V. Security and Operational Discipline (wording/style clarification)
- Added sections:
  - None
- Removed sections:
  - None
- Templates requiring updates:
  - ✅ no update required: .specify/templates/plan-template.md
  - ✅ no update required: .specify/templates/spec-template.md
  - ✅ no update required: .specify/templates/tasks-template.md
  - ✅ not applicable (directory absent): .specify/templates/commands/*.md
- Follow-up TODOs:
  - None
-->

# rustchat Constitution

## Core Principles

### I. Contract Compatibility First
Compatibility-facing behavior MUST match Mattermost wire contracts for status
codes, response schema, error envelope semantics, pagination defaults, headers,
websocket event names, and calls payload fields.

- Every compatibility claim MUST include concrete evidence paths with line
  numbers.
- "Close enough" behavior is not acceptable where clients depend on exact
  contracts.

Rationale: External clients and mobile flows are contract-sensitive and regress
with semantic drift.

### II. Plan-First User Validation
Feature and behavior changes MUST follow a plan-first flow.

- Work MUST start in `SPEC.md` with scope, contract impact, and verification.
- Explicit user approval MUST occur before implementation starts.
- Implementation and verification status MUST be tracked in `task_plan.md`.

Rationale: This enforces scope control and prevents misaligned implementation.

### III. Evidence-Backed Analysis
Compatibility-sensitive work MUST start with upstream analysis and explicit gap
capture.

- Create a dated analysis folder under `previous-analyses/`.
- Include required artifacts:
  `ENDPOINT_MATRIX.md`, `ARCHITECTURE_GAPS.md`, `UX_JOURNEYS.md`,
  `PRODUCTION_READINESS_SCORECARD.md`, `GAP_REGISTER.md`.
- Every implementation task MUST trace to a concrete, testable gap entry.

Rationale: Analysis artifacts make parity work auditable and repeatable.

### IV. Test and Verification Gates
Verification is mandatory for completion.

- Backend changes MUST run `cargo clippy` and `cargo test`.
- Compatibility-sensitive changes MUST verify status/error schema, websocket
  behavior, and pagination/ordering.
- If automated coverage is not feasible, manual verification commands MUST be
  explicit and reproducible.

Rationale: Reliability and compatibility are release gates, not optional tasks.

### V. Security and Operational Discipline
Security and readiness controls are non-negotiable.

- Secrets MUST NOT be committed.
- Production-oriented claims MUST include hardening checks and required green
  gates, including compatibility smoke checks where applicable.
- If mandatory gates fail, status MUST be reported as `not production ready`.

Rationale: Operational safety is a first-order quality requirement.

## Engineering Constraints

- Backend implementation MUST remain idiomatic async Rust (Tokio + Axum).
- PostgreSQL MUST remain the primary persistent store.
- Meilisearch SHOULD be used for dedicated retrieval/summarization workflows.
- Public Rust functions MUST return `Result` or `Option`.
- Compatibility surfaces MUST preserve established response signatures unless a
  contract change is explicitly approved and documented.

## Delivery Workflow and Review

- Context loading MUST follow progressive disclosure: `AGENTS.md` first,
  relevant skills next, task-local files as needed.
- Changes MUST stay scoped to the request; unrelated refactors are out of
  scope.
- Compatibility-sensitive PRs MUST include:
  - what changed
  - compatibility impact with gap IDs
  - exact verification commands and results
  - residual risks and follow-ups
- Each fix SHOULD include a regression test when feasible.

## Governance

This constitution supersedes conflicting local workflow preferences.

Amendment process:
1. Document rationale.
2. Update this file.
3. Synchronize affected templates/guidance.
4. Update the Sync Impact Report at the top of this file.

Versioning policy:
- MAJOR: backward-incompatible governance changes, principle removals, or
  principle redefinitions.
- MINOR: new principle/section or materially expanded guidance.
- PATCH: wording clarifications, typo fixes, and non-semantic refinements.

Compliance review expectations:
- Implementation plans MUST pass constitution checks before and after design.
- Tasks MUST include required evidence, verification, and compatibility checks.
- Reviewers MUST reject changes that violate non-negotiable principles.

**Version**: 1.0.1 | **Ratified**: 2026-03-08 | **Last Amended**: 2026-03-08
