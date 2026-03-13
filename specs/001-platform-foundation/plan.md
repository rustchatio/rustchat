# Implementation Plan: RustChat Platform Foundation

**Branch**: `001-platform-foundation` | **Date**: 2026-03-13 | **Spec**: [specs/001-platform-foundation/spec.md](spec.md)
**Input**: Feature specification from `/specs/001-platform-foundation/spec.md`

---

## Summary

Build the foundational RustChat platform supporting 50,000-200,000 concurrent users with p99 latency under 150ms. Implement enterprise-grade security (SAML/OIDC), AI agent integration (MCP/A2A), GDPR compliance, and WCAG 2.1 AA accessibility. Deliver via 4-stage SaaS maturity model (Ad-Hoc → Reactive → Proactive → Strategic).

**Primary Requirements**:
- Rust/Axum/Tokio backend with horizontal scalability
- Solid.js frontend with high reactivity
- Redis for ephemeral state, PostgreSQL for persistence
- Kafka for high-throughput message fan-out
- MCP/A2A protocols for AI agent integration
- OpenAPI 3.1.1 + Standard Webhooks for API ecosystem

---

## Technical Context

**Language/Version**: Rust 1.75+ (Edition 2021)
**Primary Dependencies**: 
- Axum 0.8 (web framework)
- Tokio 1.35+ (async runtime)
- SQLx 0.8 (compile-time checked SQL)
- Redis 0.24 (ephemeral state)
- rdkafka 0.36 (message streaming)
- Solid.js 1.8+ (frontend)

**Storage**: 
- PostgreSQL 16+ (primary, primary-replica)
- Redis 7+ (ephemeral sessions, presence, WebSocket routing)
- AWS OpenSearch (optional, >5M posts)
- S3-compatible (file attachments)

**Testing**: 
- `cargo test` (unit/integration)
- `cargo nextest` (parallel test runner)
- Playwright (frontend E2E)
- k6 (load testing)

**Target Platform**: Linux x86_64, ARM64 (containerized)
**Project Type**: Web service (backend + SPA frontend)
**Performance Goals**: 
- p99 latency < 150ms (messaging)
- 30 auth/sec sustained, 150k/90min burst
- <5ms Kafka fan-out latency

**Constraints**:
- Stateless application tier (Constitution XI)
- Memory-safe Rust (`#![forbid(unsafe_code)]` where feasible)
- Air-gapped deployment capable (Constitution X)

**Scale/Scope**: 
- 50,000-200,000 concurrent users
- 10,000+ member channels
- 5M+ posts (OpenSearch threshold)

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle Compliance

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Contract Compatibility First | ✅ | Mattermost v4 API compatibility required for mobile |
| II. Plan-First User Validation | ✅ | Spec approved 2026-03-13 |
| III. Evidence-Backed Analysis | ⏳ | Mattermost upstream analysis in `previous-analyses/` |
| IV. Test and Verification Gates | ✅ | Test strategy defined per story |
| V. Security and Operational Discipline | ✅ | Security architecture defined |
| VI. Mobile-First Alignment Priority | ✅ | Mobile journey verification required |
| VII. Product Contract Immutability | ✅ | Additive changes only, new events documented |
| VIII. Feature Safety Requirements | ✅ | Permission/migration/observability/rollback defined |
| IX. Comprehensive Test Coverage | ✅ | Protocol/permission/failure/compat tests specified |
| **X. Absolute Data Sovereignty** | ✅ | Air-gapped support, no external deps for core |
| **XI. Stateless Application Tier** | ✅ | Redis for ephemeral, PostgreSQL for persistent |
| **XII. Memory Safety First** | ✅ | Rust with unsafe minimization |
| **XIII. Zero-Trust Extensibility** | ✅ | Scoped MCP/A2A, webhook HMAC |
| **XIV. Federated Identity** | ✅ | OIDC PKCE, SAML 2.0 required |
| **XV. Strict RBAC** | ✅ | Global/Team/Channel role dimensions |
| **XVI. Immutable Audit Logging** | ✅ | Async SIEM-formatted logs |
| **XVII. Privacy by Design** | ✅ | GDPR retention, hard delete, crypto wipe |
| **XVIII. Universal Accessibility** | ✅ | WCAG 2.1 AA, BITV 2.0 |
| **XIX. DevSecOps Automation** | ✅ | cargo-audit, cargo-deny, cosign |

### Architecture Constraints

- [x] Backend remains idiomatic async Rust (Tokio + Axum)
- [x] PostgreSQL remains primary persistent store
- [x] Redis for ephemeral state (WebSocket, sessions)
- [x] No external dependencies for core functionality (Constitution X)
- [x] Stateless application tier (Constitution XI)
- [x] Memory-safe Rust implementation (Constitution XII)
- [x] No unapproved dependencies or framework additions
- [x] Minimal architecture, clear state transitions, explicit contracts

---

## Implementation Phases

### Phase 0: Foundation & Tooling (Stage 1 - Ad-Hoc)

**Goal**: Containerize application, establish PostgreSQL persistence, CI/CD pipelines

**Key Deliverables**:
- Docker multi-stage builds (backend/frontend)
- PostgreSQL schema with migrations (SQLx)
- GitHub Actions CI/CD pipeline
- Basic Axum routing structure
- Health check endpoints

**Constitution Gates**:
- [XIX] CI pipeline with `cargo clippy`, `cargo audit`, `cargo deny`
- [XII] Rust `#![forbid(unsafe_code)]` in new modules
- [IV] Unit test framework established

---

### Phase 1: State Decoupling & Authentication (Stage 2 - Reactive)

**Goal**: Decouple state to Redis, implement OIDC/SAML, add telemetry

**Key Deliverables**:
- Redis connection pooling and session storage
- OIDC flow with PKCE (SPA)
- SAML 2.0 integration with XML signature validation
- WebSocket state externalization (Redis pub/sub)
- Prometheus metrics export
- Grafana dashboards

**Constitution Gates**:
- [XI] Verify no local session state remains
- [XIV] PKCE enforcement validated
- [XVI] Audit logging infrastructure
- [XIX] Prometheus metrics exposed

---

### Phase 2: Security & Compliance (Stage 3 - Proactive)

**Goal**: Strict RBAC, search clustering, GDPR retention, immutable audit logs

**Key Deliverables**:
- Multi-dimensional RBAC (Global/Team/Channel)
- AWS OpenSearch integration (>5M posts)
- GDPR data retention automation
- Immutable audit logging (async, SIEM format)
- Data export (GDPR Article 20)
- Right to Erasure (cryptographic wipe)

**Constitution Gates**:
- [XV] RBAC enforcement on all endpoints
- [XVII] Retention policies automated
- [XVI] Audit logs immutable and tamper-evident
- [VIII] Rollback procedures tested

---

### Phase 3: AI Integration & APIs (Stage 4 - Strategic)

**Goal**: MCP/A2A protocols, OpenAPI, Standard Webhooks, dynamic scaling

**Key Deliverables**:
- MCP server implementation (JSON-RPC)
- A2A protocol message bus
- OpenAPI 3.1.1 spec generation (Utoipa)
- SDK generation pipeline
- Standard Webhooks with HMAC
- Kubernetes HPA for dynamic scaling

**Constitution Gates**:
- [XIII] Zero-trust agent access verified
- [VIII] Webhook delivery resilience
- [IX] OpenAPI contract tests pass
- [X] AI features opt-in only

---

### Phase 4: Frontend & Accessibility

**Goal**: Solid.js SPA, WCAG 2.1 AA, BITV 2.0 compliance

**Key Deliverables**:
- Solid.js component architecture
- Reactive state management (Stores)
- Keyboard navigation
- Screen reader support (ARIA)
- Color contrast compliance (4.5:1)
- Focus management
- German BITV 2.0 verification

**Constitution Gates**:
- [XVIII] Automated a11y tests in CI
- [VI] Mobile compatibility maintained
- [IV] Playwright E2E tests pass

---

## Safety Requirements Implementation

### Permission Boundaries Implementation
- [ ] RBAC middleware (`backend/src/middleware/rbac.rs`)
- [ ] Permission check functions per resource type
- [ ] Role hierarchy enforcement (System → Team → Channel)
- [ ] Edge case: Role elevation workflow

### Migration Implementation
- [ ] SQLx migrations in `backend/migrations/`
- [ ] Migration testing on 1M+ row datasets
- [ ] Rollback scripts verified per migration
- [ ] Zero-downtime migration strategy (expand/contract)

### Observability Implementation
- [ ] Prometheus metrics middleware
- [ ] Structured logging (JSON) with `tracing`
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Alertmanager rules for SLOs

### Rollback Procedure
- [ ] Blue-green deployment configuration
- [ ] Database migration reversibility
- [ ] Feature flags for gradual rollout
- [ ] Runbook documentation in `docs/operations/`

---

## Mobile Alignment Implementation

### Mobile Compatibility Verification
- [ ] API v4 endpoint parity (Mattermost 9.4.0)
- [ ] WebSocket event compatibility
- [ ] Calls plugin WebRTC integration
- [ ] Mobile journey tests (login → channel → post)

### Test Coverage for Mobile
- [ ] Mattermost mobile client smoke tests
- [ ] API contract regression tests
- [ ] Backward compatibility validation

---

## Security & Identity Implementation

### Zero-Trust Extensibility (Constitution XIII)
- [ ] MCP connection scoping (user-approved only)
- [ ] Webhook HMAC-SHA256 signature verification
- [ ] AI agent API token delegation (no DB access)

### Federated Identity (Constitution XIV)
- [ ] OIDC provider integration (generic)
- [ ] PKCE flow enforcement for SPA
- [ ] SAML 2.0 assertion validation
- [ ] Session binding (MFA context)

### RBAC Enforcement (Constitution XV)
- [ ] Permission matrix implementation
- [ ] Authorization middleware (all endpoints)
- [ ] Role elevation approval workflow

### Audit Logging (Constitution XVI)
- [ ] Async log delivery (channel-based)
- [ ] SIEM-compatible JSON format (CEF)
- [ ] Tamper-evident log hashing
- [ ] Critical event coverage (auth, RBAC, data access)

---

## Privacy & Compliance Implementation

### Data Retention
- [ ] Retention policy configuration (workspace-level)
- [ ] Automated deletion jobs (background worker)
- [ ] Legal hold override mechanism

### Right to Erasure
- [ ] Hard delete from PostgreSQL (no soft delete)
- [ ] Cryptographic wipe for S3 files
- [ ] Propagation to OpenSearch index
- [ ] Deletion certificate generation

### PII Handling
- [ ] Encryption at rest (PostgreSQL TDE)
- [ ] PII access logging
- [ ] Data minimization in logs

---

## Accessibility Implementation

### WCAG 2.1 Level AA
- [ ] Keyboard navigation (TabIndex, key handlers)
- [ ] ARIA labels and live regions
- [ ] Color contrast verification ( automated)
- [ ] Focus visible indicators
- [ ] 200% zoom support

### BITV 2.0 Compliance
- [ ] Screen reader testing (NVDA, VoiceOver)
- [ ] German accessibility standard checklist
- [ ] Automated a11y testing (axe-core in CI)

---

## DevSecOps Implementation

### CI/CD Security Gates
- [ ] `cargo clippy -- -D warnings` (zero tolerance)
- [ ] `cargo audit` (block on high/critical CVEs)
- [ ] `cargo deny` (license whitelist enforcement)
- [ ] Container scanning (Trivy)

### Supply Chain Security
- [ ] Vendored dependencies for reproducibility
- [ ] Crate checksum verification
- [ ] SBOM generation (SPDX)

### Deployment Security
- [ ] Container signing (cosign)
- [ ] Immutable infrastructure (no SSH)
- [ ] Canary deployments with automated analysis

---

## Project Structure

### Documentation (this feature)

```text
specs/001-platform-foundation/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Upstream Mattermost analysis
├── data-model.md        # Entity relationship diagrams
├── quickstart.md        # Local development setup
├── contracts/
│   ├── openapi.yaml     # OpenAPI 3.1.1 specification
│   ├── webhooks.md      # Standard Webhooks spec
│   └── mcp/             # MCP protocol documentation
└── tasks.md             # Generated task list
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── main.rs              # Application entry
│   ├── config/              # Environment configuration
│   ├── api/                 # HTTP handlers (v1 native + v4 compat)
│   │   ├── websocket_core.rs
│   │   ├── v4/              # Mattermost API compatibility
│   │   └── v1/              # Native API
│   ├── auth/                # OIDC/SAML authentication
│   ├── models/              # SQLx data models
│   ├── services/            # Business logic
│   ├── middleware/          # RBAC, logging, rate limiting
│   ├── realtime/            # WebSocket hub (Redis pub/sub)
│   ├── mcp/                 # Model Context Protocol
│   ├── a2a/                 # Agent-to-Agent protocol
│   └── telemetry/           # Metrics, tracing, logs
├── migrations/              # SQLx database migrations
├── tests/
│   ├── unit/                # Unit tests
│   ├── integration/         # API/integration tests
│   └── contract/            # OpenAPI contract tests
├── Cargo.toml
└── Dockerfile

frontend/
├── src/
│   ├── index.tsx            # Solid.js entry
│   ├── components/          # UI components
│   ├── stores/              # Reactive state management
│   ├── api/                 # API client (OpenAPI-generated)
│   ├── features/            # Feature modules
│   └── accessibility/       # ARIA utilities, focus management
├── e2e/                     # Playwright tests
├── package.json
└── Dockerfile

infrastructure/
├── docker-compose.yml       # Local development
├── kubernetes/              # K8s manifests
│   ├── backend/
│   ├── frontend/
│   ├── redis/
│   ├── postgres/
│   └── kafka/
├── terraform/               # AWS infrastructure (OpenSearch)
└── monitoring/              # Prometheus/Grafana configs
```

---

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Kafka (message broker) | Constitution XI statelessness requires external queue for fan-out | Redis pub/sub insufficient for 10k-member channel broadcast guarantees |
| AWS OpenSearch | >5M post search performance requirement | PostgreSQL full-text search degrades beyond 5M rows |
| MCP/A2A protocols | Strategic AI integration requirement | Direct API integration lacks standardized agent interoperability |

---

## Verification Milestones

### Stage 1 (Ad-Hoc) Gates
- [ ] `docker compose up` starts all services
- [ ] Health check endpoints return 200
- [ ] `cargo test` passes (unit tests)
- [ ] CI pipeline green

### Stage 2 (Reactive) Gates
- [ ] Redis session storage functional
- [ ] OIDC login with PKCE verified
- [ ] p99 latency < 150ms in k6 test
- [ ] Prometheus metrics flowing

### Stage 3 (Proactive) Gates
- [ ] RBAC enforcement passes penetration test
- [ ] GDPR erasure completes in <30 days
- [ ] Audit logs immutable verified
- [ ] OpenSearch indexing >5M posts

### Stage 4 (Strategic) Gates
- [ ] MCP server passes protocol compliance test
- [ ] A2A agent collaboration verified
- [ ] OpenAPI spec generates working SDK
- [ ] Webhook HMAC verification tested
- [ ] Auto-scaling handles 3x traffic spike

### Accessibility Gates
- [ ] axe-core passes with 0 violations
- [ ] Keyboard navigation complete
- [ ] Screen reader testing passed
- [ ] BITV 2.0 checklist complete

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Kafka operational complexity | Medium | High | Start with Redis pub/sub, migrate to Kafka at scale threshold |
| SAML integration edge cases | Medium | Medium | Extensive testing with major IdPs (Okta, Azure AD, Auth0) |
| OpenSearch cost | Medium | Medium | Make optional, provide PostgreSQL fallback |
| Solid.js ecosystem maturity | Low | Medium | Fallback to React if critical gaps emerge |
| MCP/A2A protocol churn | Medium | Medium | Abstract protocol layer, versioned implementations |

---

## Next Steps

1. **Generate tasks**: Run `/speckit.tasks` to create executable task list
2. **Begin implementation**: Start with Phase 0 (Foundation)
3. **Checkpoint reviews**: Validate each phase before proceeding
4. **Constitution compliance**: Re-check principles at each gate
