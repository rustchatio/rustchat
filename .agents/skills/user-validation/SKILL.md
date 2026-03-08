---
name: user-validation
description: "Enforces a user-in-the-loop plan-first workflow for feature or behavior changes: draft SPEC.md, require explicit approval, then update task_plan.md and provide manual verification commands."
license: MIT
---

# User Validation

Use this skill for feature or behavior changes that require explicit user approval before implementation.

This skill governs delivery workflow only. It does not replace compatibility analysis skills when contract-sensitive surfaces are involved.

## Default Working Files

- `SPEC.md` at repository root.
- `task_plan.md` at repository root.

Use alternative locations only if the user explicitly requests it.

## Phase 1: Requirement

1. Research the current implementation and constraints.
2. Draft `SPEC.md` with:
   - problem statement
   - goals and non-goals
   - scope and contract impact
   - implementation outline
   - verification plan

No implementation edits are allowed before `SPEC.md` exists.

## Phase 2: Validation Gate

After drafting `SPEC.md`, pause and ask exactly:

`Does this plan meet your expectations? Please approve or provide feedback.`

No code should be written until the user gives explicit approval.

If approval is not granted, revise `SPEC.md` based on feedback and repeat this gate with the same approval prompt.

## Phase 3: Verification

After implementation:

1. Update `task_plan.md` with task status and mark it ready for testing.
2. Provide at least one manual verification command for user acceptance (for example a `curl` request).
3. Summarize expected vs actual behavior and list any known limitations.

## Completion Criteria

- `SPEC.md` approved by the user before coding.
- `task_plan.md` reflects final execution status.
- Manual verification command provided and aligned with implemented behavior.
