# API + Security Requirements Quality Checklist: Message Timeline/Edit/Reaction/Unread Parity

**Purpose**: Validate API and security requirement quality (completeness, clarity, consistency,
measurability, and scenario coverage) for the active parity-fix specification.
**Created**: 2026-03-08
**Feature**: /Users/scolak/Projects/rustchat/SPEC.md

**Note**: This checklist validates requirement quality only. It does not validate runtime behavior.

## Requirement Completeness

- [ ] CHK001 Are request/response requirements defined for all listed contract-sensitive surfaces (`PUT /api/v4/posts/{post_id}`, `PUT /api/v4/posts/{post_id}/patch`, `GET /api/v4/config/client?format=old`, websocket events)? [Completeness, Spec §Scope and Contract Impact]
- [ ] CHK002 Are authorization requirements specified for each API surface (who may edit posts, who may read config, who may mutate reactions/read state)? [Completeness, Gap]
- [ ] CHK003 Are security requirements for edit-policy configuration exposure explicitly documented (allowed fields, backward-compatibility keys, and disclosure boundaries)? [Completeness, Spec §Implementation Outline]
- [ ] CHK004 Are requirements defined for error payload structure when edit operations are denied by policy (error ID, message semantics, and status code expectations)? [Completeness, Gap]

## Requirement Clarity

- [ ] CHK005 Is the phrase "global post edit policy" fully specified with exact modes, defaults, and time-unit semantics to remove interpretation risk? [Clarity, Spec §Problem Statement]
- [ ] CHK006 Are "compatible semantics" requirements for `AllowEditPost` and `PostEditTimeLimit` quantified with exact expected values and type formats? [Clarity, Spec §Implementation Outline]
- [ ] CHK007 Are websocket requirements for `post_edited`, `reaction_added`, `reaction_removed`, and `unread_counts_updated` defined with explicit payload field expectations? [Clarity, Gap]
- [ ] CHK008 Are manual verification command requirements precise about preconditions (`BASE`, `TOKEN`, `POST_ID`) and expected outcomes to avoid ambiguous acceptance? [Clarity, Spec §Verification Plan]

## Requirement Consistency

- [ ] CHK009 Do API edit-policy requirements align between `Problem Statement`, `Goals`, and `Implementation Outline` without conflicting scope or behavior? [Consistency, Spec §Problem Statement, Spec §Goals, Spec §Implementation Outline]
- [ ] CHK010 Are compatibility requirements consistent between `Scope and Contract Impact` and `Evidence Summary` regarding the same endpoints/events? [Consistency, Spec §Scope and Contract Impact, Spec §Evidence Summary]
- [ ] CHK011 Do security-related expectations in the spec align with `task_plan.md` verification notes and unresolved failures without contradiction? [Consistency, Spec §Verification Plan, Task Plan §Verification Status]

## Acceptance Criteria Quality

- [ ] CHK012 Are acceptance requirements for edit-within-limit versus over-limit outcomes objectively measurable (status code, error envelope, policy threshold)? [Measurability, Gap]
- [ ] CHK013 Are reaction toggle requirements measurable as state-transition rules rather than narrative behavior only (first action, second action, multi-user counts)? [Measurability, Spec §Verification Plan]
- [ ] CHK014 Are unread-indicator clearing requirements measurable with explicit event/state criteria and completion conditions? [Measurability, Spec §Goals]
- [ ] CHK015 Is there an explicit requirement ID scheme (or equivalent traceability method) linking requirements to acceptance checks and task outcomes? [Traceability, Gap]

## Scenario Coverage

- [ ] CHK016 Are primary API success scenarios documented for all impacted surfaces (edit, config fetch, reaction toggle, unread update)? [Coverage, Spec §Scope and Contract Impact]
- [ ] CHK017 Are alternate scenarios specified, including partial compatibility support and explicit unsupported-route behavior boundaries? [Coverage, Gap]
- [ ] CHK018 Are exception scenarios defined for auth failure, permission denial, malformed payloads, and stale post edits? [Coverage, Gap]
- [ ] CHK019 Are recovery scenarios specified for transient failures (retry/refresh/resync expectations after failed edit/reaction/unread updates)? [Coverage, Recovery, Gap]

## Edge Case Coverage

- [ ] CHK020 Are boundary requirements defined for edit time windows (exact cutoff, clock skew assumptions, and zero/negative policy values)? [Edge Case, Gap]
- [ ] CHK021 Are requirements defined for concurrent reaction toggles by multiple users to prevent ambiguous count semantics? [Edge Case, Spec §Problem Statement]
- [ ] CHK022 Are requirements explicit for unread-dot behavior when websocket events are delayed, duplicated, or received out of order? [Edge Case, Gap]

## Non-Functional Requirements

- [ ] CHK023 Are security requirements specified for configuration and edit-policy data integrity (tamper resistance, trusted source of policy values)? [Non-Functional, Security, Gap]
- [ ] CHK024 Are auditability requirements specified for sensitive mutation paths (post edits/reaction mutations/read-state updates), including required log fields? [Non-Functional, Security, Gap]
- [ ] CHK025 Are performance-related requirement thresholds defined for high-frequency websocket update handling so compatibility changes do not degrade responsiveness? [Non-Functional, Gap]

## Dependencies & Assumptions

- [ ] CHK026 Are assumptions about upstream parity dependencies explicitly documented and validated with evidence paths for each compatibility claim? [Dependency, Spec §Evidence Summary]
- [ ] CHK027 Are environment dependencies for verification (live compatible server, DB bootstrap, smoke preconditions) captured as requirements rather than informal notes? [Dependency, Task Plan §Verification Status]

## Ambiguities & Conflicts

- [ ] CHK028 Is the term "immediately" in edited-message update requirements quantified with a deterministic latency or event-order expectation? [Ambiguity, Spec §Problem Statement]
- [ ] CHK029 Are there conflicting implications between "no large frontend architecture migration" and required store/event wiring changes, and is the boundary explicitly resolved? [Conflict, Spec §Non-Goals, Spec §Implementation Outline]
- [ ] CHK030 Is responsibility for unresolved verification failures (clippy/integration/smoke prerequisites) clearly assigned in requirements for release decisions? [Ambiguity, Task Plan §Verification Status]
