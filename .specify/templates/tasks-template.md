---

description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The examples below include test tasks. Tests are OPTIONAL - only include them if explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume single project - adjust based on plan.md structure

<!-- 
  ============================================================================
  IMPORTANT: The tasks below are SAMPLE TASKS for illustration purposes only.
  
  The /speckit.tasks command MUST replace these with actual tasks based on:
  - User stories from spec.md (with their priorities P1, P2, P3...)
  - Feature requirements from plan.md
  - Entities from data-model.md
  - Endpoints from contracts/
  - Safety requirements (permission, migration, observability, rollback)
  - Mobile alignment requirements
  - Test coverage requirements (protocol, permission, failure, compat)
  - Security requirements (Constitution X-XVI)
  - Privacy requirements (Constitution XVII)
  - Accessibility requirements (Constitution XVIII)
  - DevSecOps requirements (Constitution XIX)
  
  Tasks MUST be organized by user story so each story can be:
  - Implemented independently
  - Tested independently
  - Delivered as an MVP increment
  
  DO NOT keep these sample tasks in the generated tasks.md file.
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize [language] project with [framework] dependencies
- [ ] T003 [P] Configure linting and formatting tools

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

Examples of foundational tasks (adjust based on your project):

- [ ] T004 Setup database schema and migrations framework
- [ ] T005 [P] Implement authentication/authorization framework
- [ ] T006 [P] Setup API routing and middleware structure
- [ ] T007 Create base models/entities that all stories depend on
- [ ] T008 Configure error handling and logging infrastructure
- [ ] T009 Setup environment configuration management

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 1 (OPTIONAL - only if tests requested) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

#### Constitution IX: Protocol Behavior Tests
- [ ] T010 [P] [US1] API contract test: status codes and response schema in tests/contract/test_[name]_protocol.py
- [ ] T011 [P] [US1] WebSocket event format test in tests/contract/test_[name]_websocket.py
- [ ] T012 [P] [US1] Pagination/ordering behavior test in tests/contract/test_[name]_pagination.py
- [ ] T013 [P] [US1] Error envelope semantics test in tests/contract/test_[name]_errors.py

#### Constitution IX: Permission Tests
- [ ] T014 [P] [US1] Happy path authorization test in tests/integration/test_[name]_auth_happy.py
- [ ] T015 [P] [US1] Edge case permission test in tests/integration/test_[name]_auth_edge.py
- [ ] T016 [P] [US1] Privilege escalation attempt test in tests/integration/test_[name]_auth_escalation.py

#### Constitution IX: Failure Case Tests
- [ ] T017 [P] [US1] Error handling path test in tests/integration/test_[name]_errors.py
- [ ] T018 [P] [US1] Degradation mode test in tests/integration/test_[name]_degradation.py
- [ ] T019 [P] [US1] Resource exhaustion test in tests/integration/test_[name]_resources.py

#### Constitution IX: Backward Compatibility Tests
- [ ] T020 [P] [US1] Existing client compatibility test in tests/integration/test_[name]_compat.py
- [ ] T021 [P] [US1] Mobile client compatibility test in tests/integration/test_[name]_mobile.py

### Implementation for User Story 1

- [ ] T022 [P] [US1] Create [Entity1] model in src/models/[entity1].py
- [ ] T023 [P] [US1] Create [Entity2] model in src/models/[entity2].py
- [ ] T024 [US1] Implement [Service] in src/services/[service].py (depends on T022, T023)
- [ ] T025 [US1] Implement [endpoint/feature] in src/[location]/[file].py
- [ ] T026 [US1] Add validation and error handling
- [ ] T027 [US1] Add logging for user story 1 operations
- [ ] T028 [US1] **Constitution VIII**: Implement permission boundaries

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - [Title] (Priority: P2)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 2 (OPTIONAL - only if tests requested) ⚠️

#### Constitution IX: Required Test Coverage
- [ ] T029 [P] [US2] Protocol behavior tests in tests/contract/test_[name]_protocol.py
- [ ] T030 [P] [US2] Permission tests in tests/integration/test_[name]_auth.py
- [ ] T031 [P] [US2] Failure case tests in tests/integration/test_[name]_errors.py
- [ ] T032 [P] [US2] Backward compatibility tests in tests/integration/test_[name]_compat.py

### Implementation for User Story 2

- [ ] T033 [P] [US2] Create [Entity] model in src/models/[entity].py
- [ ] T034 [US2] Implement [Service] in src/services/[service].py
- [ ] T035 [US2] Implement [endpoint/feature] in src/[location]/[file].py
- [ ] T036 [US2] Integrate with User Story 1 components (if needed)
- [ ] T037 [US2] **Constitution VIII**: Implement permission boundaries

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - [Title] (Priority: P3)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 3 (OPTIONAL - only if tests requested) ⚠️

#### Constitution IX: Required Test Coverage
- [ ] T038 [P] [US3] Protocol behavior tests in tests/contract/test_[name]_protocol.py
- [ ] T039 [P] [US3] Permission tests in tests/integration/test_[name]_auth.py
- [ ] T040 [P] [US3] Failure case tests in tests/integration/test_[name]_errors.py
- [ ] T041 [P] [US3] Backward compatibility tests in tests/integration/test_[name]_compat.py

### Implementation for User Story 3

- [ ] T042 [P] [US3] Create [Entity] model in src/models/[entity].py
- [ ] T043 [US3] Implement [Service] in src/services/[service].py
- [ ] T044 [US3] Implement [endpoint/feature] in src/[location]/[file].py
- [ ] T045 [US3] **Constitution VIII**: Implement permission boundaries

**Checkpoint**: All user stories should now be independently functional

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Safety Requirements & Cross-Cutting Concerns

**Purpose**: Constitution VIII requirements and improvements that affect multiple user stories

### Constitution VIII: Migration & Rollback
- [ ] TXXX Create database migration in migrations/[version]_[description].sql
- [ ] TXXX Test migration on representative data volume
- [ ] TXXX Verify rollback procedure works correctly
- [ ] TXXX Document rollback steps in operations/runbook.md

### Constitution VIII: Observability
- [ ] TXXX Add metrics instrumentation for feature usage
- [ ] TXXX Add structured logging at appropriate levels
- [ ] TXXX Configure alerts for error rate thresholds (if applicable)

### Constitution VI: Mobile Alignment Verification
- [ ] TXXX Run mobile journey smoke tests
- [ ] TXXX Verify API v4 contract compliance
- [ ] TXXX Verify WebSocket event compatibility

### General Polish
- [ ] TXXX [P] Documentation updates in docs/
- [ ] TXXX Code cleanup and refactoring
- [ ] TXXX Performance optimization across all stories
- [ ] TXXX [P] Additional unit tests in tests/unit/
- [ ] TXXX Security hardening
- [ ] TXXX Run quickstart.md validation

---

## Phase N+1: Security & Compliance Verification

**Purpose**: Constitution X-XIX verification gates

### Constitution X: Data Sovereignty
- [ ] TXXX Verify no external API dependencies for core functionality
- [ ] TXXX Document network egress requirements (if any)
- [ ] TXXX Verify multi-tenant isolation

### Constitution XI: Stateless Architecture
- [ ] TXXX Verify no local session state
- [ ] TXXX Confirm Redis usage for ephemeral data
- [ ] TXXX WebSocket clustering uses Redis pub/sub

### Constitution XII: Memory Safety
- [ ] TXXX Review all `unsafe` blocks for necessity
- [ ] TXXX Document any required `unsafe` with safety comments
- [ ] TXXX Run miri test if applicable

### Constitution XIII: Zero-Trust
- [ ] TXXX Verify integration scope limitations
- [ ] TXXX Webhook signature verification implemented
- [ ] TXXX AI agent access uses API tokens only

### Constitution XIV: Federated Identity
- [ ] TXXX OIDC PKCE enforced for SPA
- [ ] TXXX SAML assertion validation reviewed
- [ ] TXXX Session context binding implemented

### Constitution XV: RBAC
- [ ] TXXX Multi-dimensional permission checks verified
- [ ] TXXX Authorization tests pass
- [ ] TXXX Role elevation workflow documented

### Constitution XVI: Audit Logging
- [ ] TXXX Critical events logged asynchronously
- [ ] TXXX SIEM-compatible format verified
- [ ] TXXX Log delivery non-blocking

### Constitution XVII: Privacy/Compliance
- [ ] TXXX Data retention policy implemented
- [ ] TXXX Hard delete (no soft delete) for erasure
- [ ] TXXX PII handling compliant

### Constitution XVIII: Accessibility
- [ ] TXXX Keyboard navigation works
- [ ] TXXX ARIA labels present
- [ ] TXXX Color contrast verified
- [ ] TXXX Automated a11y tests pass

### Constitution XIX: DevSecOps
- [ ] TXXX `cargo audit` passes (no high/critical CVEs)
- [ ] TXXX `cargo deny` license check passes
- [ ] TXXX Container image signing configured
- [ ] TXXX SBOM generated

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Safety Requirements (Phase N)**: Depends on all desired user stories being complete
- **Security & Compliance (Phase N+1)**: Final verification gate before release

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable

### Within Each User Story

- Tests (if included) MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members
- Security & Compliance verification tasks can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all Constitution IX tests for User Story 1 together:
Task: "Protocol behavior test in tests/contract/test_[name]_protocol.py"
Task: "Permission test in tests/integration/test_[name]_auth.py"
Task: "Failure case test in tests/integration/test_[name]_errors.py"
Task: "Backward compatibility test in tests/integration/test_[name]_compat.py"

# Launch all models for User Story 1 together:
Task: "Create [Entity1] model in src/models/[entity1].py"
Task: "Create [Entity2] model in src/models/[entity2].py"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (including Constitution IX tests)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Complete Phase N: Safety requirements for User Story 1
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Safety requirements → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Safety requirements → Deploy/Demo
4. Add User Story 3 → Test independently → Safety requirements → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 + its safety requirements
   - Developer B: User Story 2 + its safety requirements
   - Developer C: User Story 3 + its safety requirements
3. Stories complete and integrate independently
4. Final phase: Security & Compliance verification (parallel tasks)

---

## Constitution Compliance Checklist

Before marking feature complete, verify:

### Workflow Principles (I-IX)
- [ ] **I. Contract Compatibility First**: All API/websocket contracts match Mattermost
- [ ] **II. Plan-First User Validation**: Spec approved, plan documented
- [ ] **III. Evidence-Backed Analysis**: Upstream analysis artifacts linked
- [ ] **IV. Test and Verification Gates**: All tests pass, `cargo clippy` clean
- [ ] **V. Security and Operational Discipline**: No secrets, security reviewed
- [ ] **VI. Mobile-First Alignment**: Mobile compatibility verified
- [ ] **VII. Product Contract Immutability**: No implicit contract changes
- [ ] **VIII. Feature Safety Requirements**:
  - [ ] Permission boundaries implemented
  - [ ] Migration impact addressed
  - [ ] Observability requirements met
  - [ ] Rollback safety verified
- [ ] **IX. Comprehensive Test Coverage**:
  - [ ] Protocol behavior tests
  - [ ] Permission tests
  - [ ] Failure case tests
  - [ ] Backward compatibility tests

### Architecture Principles (X-XIII)
- [ ] **X. Absolute Data Sovereignty**: Air-gapped deployment support, no external deps
- [ ] **XI. Stateless Application Tier**: Redis for ephemeral state, no local state
- [ ] **XII. Memory Safety First**: Rust `#![forbid(unsafe_code)]` or documented justification
- [ ] **XIII. Zero-Trust Extensibility**: Least privilege, scoped access, webhook verification

### Security & Identity (XIV-XVI)
- [ ] **XIV. Federated Identity**: OIDC PKCE, SAML validation
- [ ] **XV. Strict RBAC**: Multi-dimensional authorization
- [ ] **XVI. Immutable Audit Logging**: Async SIEM-formatted logs

### Compliance & DevSecOps (XVII-XIX)
- [ ] **XVII. Privacy by Design**: Retention, erasure, PII handling
- [ ] **XVIII. Universal Accessibility**: WCAG 2.1 AA, BITV 2.0
- [ ] **XIX. DevSecOps Automation**: `cargo audit`, `cargo deny`, signed containers

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- All Constitution VIII safety requirements must be addressed before feature is complete
- All Constitution IX test categories must be covered (or explicitly waived with justification)
- All Constitution X-XIX requirements must be verified in final Security & Compliance phase
