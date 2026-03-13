<!--
Sync Impact Report
- Version change: 1.1.0 -> 1.2.0
- Modified principles:
  - All existing principles I-IX retained
- Added principles:
  - X. Absolute Data Sovereignty
  - XI. Stateless Application Tier
  - XII. Memory Safety First
  - XIII. Zero-Trust Extensibility
  - XIV. Federated Identity
  - XV. Strict RBAC
  - XVI. Immutable Audit Logging
  - XVII. Privacy by Design (GDPR/CCPA)
  - XVIII. Universal Accessibility
  - XIX. DevSecOps Automation
- Added sections:
  - Core Architectural Principles
  - Security and Identity Governance
  - Compliance and Accessibility Standards
- Templates requiring updates:
  - ⚠️  review recommended: .specify/templates/spec-template.md (add compliance/accessibility/DevSecOps sections)
  - ⚠️  review recommended: .specify/templates/plan-template.md (add architecture/security/compliance gates)
  - ⚠️  review recommended: .specify/templates/tasks-template.md (add security/compliance checklists)
- Follow-up TODOs:
  - Update spec template to include Data Sovereignty section
  - Update spec template to include Accessibility (WCAG/BITV) section
  - Update spec template to include Privacy/Retention Policy section
  - Update CI/CD templates to include cargo-audit and cargo-deny gates
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

### VI. Mobile-First Alignment Priority
Mobile alignment is priority one. RustChat mobile is a rebranded
mattermost-mobile; compatibility with upstream mobile behavior is paramount.

- All features MUST preserve mobile compatibility guarantees explicitly
  documented in the feature spec.
- Mobile-specific journeys (login, channels, posts, notifications, files, calls)
  MUST be verified against mattermost-mobile behavior.
- API v4 compatibility, websocket events, and calls plugin behavior MUST be
  tested with mobile client expectations.

Rationale: The mobile client is the primary user interface; breaking mobile
compatibility breaks the product.

### VII. Product Contract Immutability
Message, channel, membership, and event semantics are product contracts and
MUST NOT change implicitly.

- Core data model semantics (message structure, channel types, membership
  states, event payloads) are immutable without explicit contract revision.
- Any change to product contracts requires:
  - Explicit deprecation notice in spec
  - Migration path for existing data
  - Backward compatibility layer or coordinated client update
  - Version bump and changelog entry

Rationale: Implicit contract changes cause data corruption, client crashes, and
user trust erosion.

### VIII. Feature Safety Requirements
Every feature MUST define explicit safety boundaries before implementation.

**Required for every feature:**
- **Permission Boundaries**: Who can access/modify what under which conditions
- **Migration Impact**: Database changes, data transformations, rollback procedure
- **Observability Requirements**: Metrics, logs, traces, alerts
- **Rollback Safety**: How to revert without data loss or downtime

Features lacking any of these requirements MUST NOT proceed to implementation.

Rationale: Production incidents stem from undefined boundaries and untested
recovery paths.

### IX. Comprehensive Test Coverage
Tests MUST cover protocol behavior, permission checks, failure cases, and
backward compatibility where relevant.

**Required test categories:**
- **Protocol behavior**: API contracts, websocket events, payload formats
- **Permission checks**: All authorization paths, edge cases, escalation attempts
- **Failure cases**: Error handling, degradation modes, resource exhaustion
- **Backward compatibility**: Existing clients, data formats, API versions

Coverage gaps MUST be documented with risk assessment and acceptance criteria.

Rationale: Untested code is unreliable code; compatibility without tests is
unverified claims.

---

## Core Architectural Principles

### X. Absolute Data Sovereignty
The platform MUST support self-hosted, air-gapped, and sovereign cloud
deployments to ensure complete control over enterprise data and intellectual
property.

- No external dependencies for core functionality (no external APIs, no cloud
  services required for basic operation).
- All data MUST remain within the deployment boundary unless explicitly
  configured by the operator.
- Network egress MUST be opt-in and auditable.
- Multi-tenant isolation MUST prevent data leakage between organizations.

Rationale: Enterprise customers require full data control for compliance,
intellectual property protection, and operational security.

### XI. Stateless Application Tier
Application logic MUST be strictly decoupled from state. Ephemeral session data
and WebSocket routing MUST be offloaded to an in-memory data store (Redis) to
enable horizontal scaling.

- Application servers MUST NOT maintain local session state.
- All shared state MUST be externalized to Redis.
- WebSocket hub clustering MUST use Redis pub/sub for event broadcast.
- Any feature requiring shared state MUST use Redis or PostgreSQL, never local
  memory.

Rationale: Stateless architecture enables horizontal scaling, rolling
deployments, and fault tolerance.

### XII. Memory Safety First
The backend MUST be written in Rust, leveraging compile-time protections to
eliminate memory vulnerabilities (e.g., buffer overflows, use-after-free) common
in C/C++ messaging servers.

- All backend code MUST be idiomatic Rust with `#![forbid(unsafe_code)]` where
  feasible.
- `unsafe` blocks MUST be justified, documented, and minimized.
- External C dependencies MUST be wrapped in safe Rust abstractions.
- Fuzz testing SHOULD be employed for security-critical parsers.

Rationale: Memory safety eliminates entire classes of security vulnerabilities
at compile time.

### XIII. Zero-Trust Extensibility
All integrations, webhooks, and AI agents MUST operate under the principle of
least privilege, unable to access or mutate data beyond the invoking user's
authorization scope.

- Integrations MUST be scoped to specific channels/users, never global access.
- Webhooks MUST verify signatures and validate payload integrity.
- AI agents MUST NOT have direct database access; they MUST use the public API
  with user-delegated tokens.
- OAuth tokens MUST be scoped and time-limited.

Rationale: Third-party integrations are a common attack vector; least privilege
limits blast radius.

---

## Security and Identity Governance

### XIV. Federated Identity
Decentralized credential management is prohibited. Authentication MUST be
delegated to enterprise Identity Providers (IdPs) via SAML 2.0 or OpenID
Connect (OIDC). Single Page Applications (SPAs) using OIDC MUST enforce Proof
Key for Code Exchange (PKCE).

- Local password authentication MUST be disable-able via configuration.
- OIDC flows MUST validate `nonce`, `state`, and PKCE parameters.
- SAML assertions MUST be validated against XML signature wrapping attacks.
- Session tokens MUST be bound to the authentication context (MFA, device
  posture where available).

Rationale: Enterprise SSO reduces credential sprawl and enables centralized
access control, auditing, and revocation.

### XV. Strict RBAC
Authorization MUST be enforced via multi-dimensional Role-Based Access Control
(RBAC) at the global, workspace, and channel levels. Roles range from System
Administrator down to restricted External Guests.

**Role Hierarchy**:
- System Administrator: Global platform configuration, user management
- Team Administrator: Workspace-level settings, member management
- Channel Administrator: Channel settings, moderation
- Member: Standard participation
- Guest: Restricted access, limited channels

**Permission Enforcement**:
- Every API endpoint MUST check authorization before processing.
- Permission checks MUST be centralized and auditable.
- Role elevation MUST require explicit approval workflow.

Rationale: Multi-dimensional RBAC provides defense-in-depth and principle of
least privilege at every organizational level.

### XVI. Immutable Audit Logging
All critical security events, RBAC mutations, and data access operations MUST
be logged asynchronously to prevent database contention, formatted for direct
ingestion into SIEM platforms.

**Required Audit Events**:
- Authentication attempts (success and failure)
- Authorization decisions (denied access)
- RBAC changes (role assignments, permission grants)
- Data access (bulk exports, admin data views)
- Configuration changes (security settings, integrations)

**Log Format**:
- Structured JSON with standardized schema (CEF/LEEF compatible)
- Immutable append-only storage (WORM where supported)
- Asynchronous delivery to prevent request blocking
- Tamper-evident hashing for compliance

Rationale: Immutable audit logs are required for forensic investigation,
compliance attestation, and insider threat detection.

---

## Compliance and Accessibility Standards

### XVII. Privacy by Design (GDPR/CCPA)
The platform MUST execute automated, policy-driven data deletion. Right to
Erasure mandates cryptographic purging (e.g., PostgreSQL table vacuuming),
NOT soft deletes.

**Data Retention Policies** (examples, configurable per deployment):
- Applicant data: Purge after 6 months unless consent renewed
- Highly confidential records: Retain up to 10 years with legal hold
- Standard messages: Retain per workspace policy (30 days to indefinite)
- Audit logs: Retain 7 years (immutable)

**Right to Erasure**:
- Hard delete from database (no soft delete)
- Cryptographic wipe of file storage
- Propagation to all replicas and backups
- Certificate of destruction for compliance

**Data Subject Rights**:
- Export personal data in machine-readable format
- Rectify inaccurate data
- Object to automated processing
- Withdraw consent

Rationale: Privacy regulations require proactive data lifecycle management;
soft deletes violate Right to Erasure.

### XVIII. Universal Accessibility
The frontend MUST strictly adhere to WCAG 2.1 Level AA and the German BITV 2.0
legal standard to ensure operability for all users.

**WCAG 2.1 Level AA Requirements**:
- Perceivable: Text alternatives, adaptable content, distinguishable
- Operable: Keyboard accessible, sufficient time, navigable
- Understandable: Readable, predictable, input assistance
- Robust: Compatible with assistive technologies

**BITV 2.0 Compliance** (German federal accessibility law):
- Screen reader compatibility
- Keyboard-only navigation
- High contrast mode support
- Focus management
- ARIA landmarks and labels

**Accessibility Testing**:
- Automated a11y linting in CI
- Screen reader testing (NVDA, JAWS, VoiceOver)
- Keyboard navigation validation
- Color contrast verification

Rationale: Accessibility is a legal requirement (EU Accessibility Act, BITV 2.0)
and an ethical imperative.

### XIX. DevSecOps Automation
Software supply chain security is non-negotiable. CI/CD pipelines MUST enforce
`cargo-audit` for vulnerability scanning and `cargo-deny` to block
incompatible open-source licenses or untrusted registries.

**Security Gates**:
- `cargo audit`: Fail build on high/critical CVEs
- `cargo deny`: Enforce license whitelist, ban copyleft where prohibited
- `cargo clippy`: Zero warnings with `#![deny(warnings)]`
- Container scanning: Trivy/Grype for base image vulnerabilities

**Supply Chain Controls**:
- Vendored dependencies for reproducible builds
- Checksum verification for all crates
- Private crate registry for internal packages
- SBOM generation for compliance

**Deployment Gates**:
- Signed container images (cosign)
- Immutable infrastructure (no SSH to production)
- Automated rollback on health check failure
- Canary deployments with automated promotion

Rationale: Supply chain attacks are increasing; automated security gates catch
vulnerabilities before production.

---

## Engineering Constraints

- Backend implementation MUST remain idiomatic async Rust (Tokio + Axum).
- PostgreSQL MUST remain the primary persistent store.
- Meilisearch SHOULD be used for dedicated retrieval/summarization workflows.
- Public Rust functions MUST return `Result` or `Option`.
- Compatibility surfaces MUST preserve established response signatures unless a
  contract change is explicitly approved and documented.

## Architecture Preferences

Preferred engineering values in priority order:

1. **Minimal architecture** - Solve the problem, not every problem
2. **Clear state transitions** - Explicit over implicit, visible over hidden
3. **Explicit contracts** - Documented, versioned, validated interfaces
4. **No hidden magic** - Avoid framework magic, auto-wiring, implicit behavior

**Dependency policy:**
- No unapproved dependencies or framework additions
- New dependencies require:
  - Security audit justification
  - Maintenance status verification
  - License compatibility check
  - Alternatives considered documentation

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

**Version**: 1.2.0 | **Ratified**: 2026-03-08 | **Last Amended**: 2026-03-13
