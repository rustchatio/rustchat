# Feature Specification (Advanced): [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`
**Created**: [DATE]
**Status**: Draft
**Input**: User description: "$ARGUMENTS"

## Planning Context

- **Feature Scope Type**: [API / Websocket / Frontend UX / Cross-cutting]
- **Primary Risks**: [compatibility, security, migration, performance]
- **Out of Scope / Deferred**: [explicit deferrals]

## User Scenarios & Testing *(mandatory)*

### User Story 1 - [Brief Title] (Priority: P1)

[User journey in plain language]

**Why this priority**: [value and priority reason]
**Independent Test**: [standalone validation]

**Acceptance Scenarios**:

1. **Given** [state], **When** [action], **Then** [outcome]
2. **Given** [state], **When** [action], **Then** [outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Description]

**Why this priority**: [reason]
**Independent Test**: [validation]

**Acceptance Scenarios**:

1. **Given** [state], **When** [action], **Then** [outcome]

---

### User Story 3 - [Brief Title] (Priority: P3)

[Description]

**Why this priority**: [reason]
**Independent Test**: [validation]

**Acceptance Scenarios**:

1. **Given** [state], **When** [action], **Then** [outcome]

## Edge Cases *(mandatory)*

- What happens when [boundary condition]?
- How does system handle [error scenario]?
- What is the recovery expectation when [partial failure]?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST [capability]
- **FR-002**: System MUST [capability]
- **FR-003**: Users MUST be able to [interaction]

### Non-Functional Requirements

- **NFR-001 (Performance)**: System MUST [measurable threshold]
- **NFR-002 (Security)**: System MUST [security control]
- **NFR-003 (Reliability)**: System MUST [recovery/availability criteria]

### Constraints

- **CON-001**: [business/product constraint]
- **CON-002**: [technical/dependency constraint]
- **CON-003**: [compatibility/regulatory constraint]

## Constitution Alignment *(mandatory)*

- **Compatibility Impact**: [yes/no + details]
- **Upstream Evidence**: [paths with lines, or N/A with rationale]
- **Analysis Artifacts**: [planned `previous-analyses/YYYY-MM-DD-<topic>/` files]
- **User Validation Gate**: [explicit approval checkpoint]
- **Verification Obligations**: [required automated/manual checks]

## Key Entities *(if data involved)*

- **[Entity 1]**: [description]
- **[Entity 2]**: [description]

## Success Criteria *(mandatory)*

- **SC-001**: [measurable outcome]
- **SC-002**: [measurable outcome]
- **SC-003**: [measurable outcome]

## Author Checklist *(mandatory before approval)*

- [ ] All MUST statements are testable and unambiguous
- [ ] Exception and recovery scenarios are documented
- [ ] Acceptance criteria are measurable and mapped to requirements
- [ ] Compatibility impact and evidence are explicit
- [ ] Deferred scope is listed and justified
