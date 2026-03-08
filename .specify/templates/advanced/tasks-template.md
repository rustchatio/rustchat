---
description: "Advanced wave-based task list template"
---

# Tasks (Advanced): [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: plan.md, spec.md, optional research/data-model/contracts

## Execution Metadata

- **Wave Plan**: Wave 1 Foundation / Wave 2 Core / Wave 3 Hardening
- **Parallel Strategy**: mark independent tasks `[P]`
- **Task Sizing**: prefer 15-30 minute tasks with one clear verification step

## Format: `[ID] [P?] [Story] Description`

- **[P]**: parallel-safe task
- **[Story]**: US1/US2/US3 mapping
- Include exact file paths

## Wave 1: Foundation (Blocking)

- [ ] T001 Setup project scaffolding per plan
- [ ] T002 Define/update contract artifacts and compatibility notes
- [ ] T003 [P] Setup validation/lint/test commands
- [ ] T004 Add explicit user approval checkpoint in task flow
- [ ] T005 If compatibility-sensitive: create and populate analysis artifacts (`previous-analyses/YYYY-MM-DD-<topic>/`)

## Wave 2: Core User Stories

### User Story 1 (P1)

- [ ] T006 [P] [US1] Write contract/integration tests first
- [ ] T007 [US1] Implement core behavior for story 1
- [ ] T008 [US1] Add validation/error semantics for story 1

### User Story 2 (P2)

- [ ] T009 [P] [US2] Write contract/integration tests first
- [ ] T010 [US2] Implement core behavior for story 2

### User Story 3 (P3)

- [ ] T011 [P] [US3] Write contract/integration tests first
- [ ] T012 [US3] Implement core behavior for story 3

## Wave 3: Hardening and Release Gates

- [ ] T013 Run required verification (`cargo clippy`, `cargo test`)
- [ ] T014 Run compatibility smoke checks when applicable
- [ ] T015 Resolve or document remaining gaps/risks
- [ ] T016 Update `task_plan.md` with executed commands and readiness

## Dependencies & Order

- Wave 1 blocks Wave 2
- Wave 2 blocks Wave 3
- `[P]` tasks can run concurrently when files/dependencies do not overlap
