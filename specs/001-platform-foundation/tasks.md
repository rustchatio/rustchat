# Tasks: RustChat Platform Foundation

**Input**: Design documents from `/specs/001-platform-foundation/`
**Prerequisites**: plan.md (required), spec.md (required for user stories)

---

## Phase 0: Foundation & Tooling (Stage 1 - Ad-Hoc)

**Purpose**: Containerize application, establish PostgreSQL persistence, CI/CD pipelines

### Setup Tasks
- [ ] T001 Create `backend/Cargo.toml` with Axum, Tokio, SQLx, Redis dependencies
- [ ] T002 Create `frontend/package.json` with Solid.js, TypeScript, Vite setup
- [ ] T003 [P] Create backend Dockerfile (multi-stage build)
- [ ] T004 [P] Create frontend Dockerfile (multi-stage build)
- [ ] T005 Create `docker-compose.yml` for local development (Postgres, Redis)

### Database Foundation
- [ ] T006 Create initial PostgreSQL schema migration (`migrations/001_initial_schema.sql`)
- [ ] T007 Setup SQLx compile-time query checking (`.sqlx/` query metadata)
- [ ] T008 Create database connection pool configuration

### CI/CD Pipeline
- [ ] T009 Create `.github/workflows/ci.yml` (lint, test, build)
- [ ] T010 Add `cargo clippy -- -D warnings` to CI
- [ ] T011 Add `cargo audit` security scan to CI
- [ ] T012 Add `cargo deny` license check to CI
- [ ] T013 Add frontend build and test to CI

### Basic API Structure
- [ ] T014 Create Axum app structure (`main.rs`, router module)
- [ ] T015 Implement health check endpoint (`GET /health`)
- [ ] T016 Implement readiness probe (`GET /ready`)
- [ ] T017 Add request logging middleware (`tracing`)

**Checkpoint**: `docker compose up` starts all services, health checks pass

---

## Phase 1: State Decoupling & Authentication (Stage 2 - Reactive)

**Purpose**: Decouple state to Redis, implement OIDC/SAML, add telemetry

### Redis Integration
- [ ] T018 [P] Add Redis connection pool (`redis` crate)
- [ ] T019 Implement session storage in Redis
- [ ] T020 Implement WebSocket presence in Redis
- [ ] T021 Implement Redis pub/sub for cross-server WebSocket broadcast
- [ ] T022 Add Redis connection health checks

### OIDC Authentication
- [ ] T023 Add `openidconnect` crate dependency
- [ ] T024 Implement OIDC discovery endpoint client
- [ ] T025 Implement authorization endpoint handler (PKCE)
- [ ] T026 Implement token endpoint handler
- [ ] T027 Enforce PKCE for all OIDC flows (Constitution XIV)
- [ ] T028 Add OIDC session binding (MFA context)

### SAML Authentication
- [ ] T029 Add `samael` or `samling` crate for SAML
- [ ] T030 Implement SAML SP metadata endpoint
- [ ] T031 Implement SAML ACS (Assertion Consumer Service)
- [ ] T032 Add XML signature validation (Constitution XIV)
- [ ] T033 Add XML signature wrapping attack prevention

### Session Management
- [ ] T034 Implement JWT token generation (access + refresh)
- [ ] T035 Implement token refresh endpoint
- [ ] T036 Implement logout/session revocation
- [ ] T037 Add session cleanup background job

### Observability
- [ ] T038 Add Prometheus metrics middleware (`metrics` crate)
- [ ] T039 Implement key metrics (requests, latency, active users)
- [ ] T040 Add structured JSON logging (`tracing-subscriber`)
- [ ] T041 Implement distributed tracing (OpenTelemetry)
- [ ] T042 Create Grafana dashboard JSON

**Checkpoint**: OIDC login with PKCE works, Redis sessions functional, metrics flowing

---

## Phase 2: Core Messaging & Mattermost Compatibility

**Purpose**: Implement messaging with Mattermost API v4 compatibility

### User & Team Management
- [ ] T043 Create User model (SQLx)
- [ ] T044 Create Team model
- [ ] T045 Create TeamMembership model
- [ ] T046 Implement user registration (if local auth enabled)
- [ ] T047 Implement Mattermost v4 user endpoints (`/api/v4/users/*`)

### Channel Management
- [ ] T048 Create Channel model
- [ ] T049 Create ChannelMembership model
- [ ] T050 Implement channel CRUD operations
- [ ] T051 Implement Mattermost v4 channel endpoints (`/api/v4/channels/*`)

### Messaging
- [ ] T052 Create Post model
- [ ] T053 Implement post creation with validation
- [ ] T054 Implement post edit (with time limit policy)
- [ ] T055 Implement post deletion (hard delete per Constitution XVII)
- [ ] T056 Implement Mattermost v4 post endpoints (`/api/v4/posts/*`)
- [ ] T057 Implement thread/reply functionality

### WebSocket Real-time
- [ ] T058 Implement WebSocket upgrade handler
- [ ] T059 Implement WebSocket message framing (Mattermost protocol)
- [ ] T060 Implement event broadcasting via Redis pub/sub
- [ ] T061 Add presence/typing indicators
- [ ] T062 Implement notification events

**Checkpoint**: Mobile client can connect, send/receive messages via WebSocket

---

## Phase 3: Security & RBAC (Stage 3 - Proactive)

**Purpose**: Strict RBAC, audit logging, security hardening

### RBAC Implementation
- [ ] T063 Define permission constants (global, team, channel levels)
- [ ] T064 Create Role model and default roles
- [ ] T065 Implement RBAC middleware (all endpoints)
- [ ] T066 Implement permission check functions
- [ ] T067 Add role assignment endpoints (System/Team/Channel Admin)
- [ ] T068 Implement role elevation workflow

### Audit Logging
- [ ] T069 Create audit log table (immutable, append-only)
- [ ] T070 Implement async audit log delivery (channel-based)
- [ ] T071 Add authentication event logging
- [ ] T072 Add RBAC mutation logging
- [ ] T073 Add data access logging (bulk exports)
- [ ] T074 Add SIEM-compatible JSON format (CEF)
- [ ] T075 Implement log tamper-evident hashing

### API Security
- [ ] T076 Implement rate limiting (per user, per endpoint)
- [ ] T077 Add request size limits
- [ ] T078 Implement CORS configuration
- [ ] T079 Add security headers (HSTS, CSP, etc.)
- [ ] T080 Implement API versioning strategy

**Checkpoint**: RBAC enforcement passes penetration test, audit logs immutable

---

## Phase 4: GDPR & Data Lifecycle (Stage 3 - Proactive)

**Purpose**: GDPR compliance, data retention, Right to Erasure

### Data Retention
- [ ] T081 Create retention policy configuration model
- [ ] T082 Implement automated data retention jobs
- [ ] T083 Add legal hold override mechanism
- [ ] T084 Create retention audit dashboard

### Right to Erasure
- [ ] T085 Implement user data export (GDPR Article 20)
- [ ] T086 Implement hard delete for user data (no soft delete)
- [ ] T087 Implement cryptographic wipe for file attachments
- [ ] T088 Add deletion certificate generation
- [ ] T089 Implement data propagation to OpenSearch (deletion)

### Privacy Controls
- [ ] T090 Add consent management for AI features
- [ ] T091 Implement PII detection and classification
- [ ] T092 Add data minimization in logs (PII masking)
- [ ] T093 Create privacy dashboard for users

**Checkpoint**: GDPR erasure completes in <30 days, export functional

---

## Phase 5: Search & Scalability (Stage 3 - Proactive)

**Purpose**: AWS OpenSearch integration, Kafka for high-throughput

### PostgreSQL Search (Baseline)
- [ ] T094 Implement PostgreSQL full-text search for posts
- [ ] T095 Add search index maintenance
- [ ] T096 Implement search API endpoints

### AWS OpenSearch (Scale Threshold)
- [ ] T097 Add OpenSearch client configuration
- [ ] T098 Implement post indexing pipeline
- [ ] T099 Add OpenSearch search fallback
- [ ] T100 Implement index rollover strategy
- [ ] T101 Add OpenSearch monitoring

### Kafka Integration
- [ ] T102 Add `rdkafka` crate dependency
- [ ] T103 Implement Kafka producer for post events
- [ ] T104 Implement Kafka consumer for fan-out
- [ ] T105 Add massive channel support (>10k members)
- [ ] T106 Implement fallback to PostgreSQL (Kafka unavailable)

### High Availability
- [ ] T107 Configure PostgreSQL primary-replica
- [ ] T108 Implement read replica routing
- [ ] T109 Add connection pooling (deadpool/sqlx)
- [ ] T110 Implement circuit breakers for external services

**Checkpoint**: OpenSearch handles >5M posts, Kafka fan-out <5ms latency

---

## Phase 6: AI Integration - MCP (Stage 4 - Strategic)

**Purpose**: Model Context Protocol for secure AI agent data access

### MCP Server
- [ ] T111 Implement MCP server endpoint (`/mcp`)
- [ ] T112 Add JSON-RPC message handling
- [ ] T113 Implement MCP capability negotiation
- [ ] T114 Add MCP connection lifecycle management

### MCP Tools
- [ ] T115 Define MCP tool schemas
- [ ] T116 Implement channel query tool
- [ ] T117 Implement user query tool
- [ ] T118 Implement message search tool
- [ ] T119 Add user approval workflow for tool access

### MCP Security
- [ ] T120 Implement scoped access (channel/user level)
- [ ] T121 Add MCP audit logging
- [ ] T122 Implement rate limiting per MCP connection
- [ ] T123 Add connection timeout and cleanup

**Checkpoint**: MCP server passes protocol compliance test

---

## Phase 7: AI Integration - A2A (Stage 4 - Strategic)

**Purpose**: Agent-to-Agent protocol for multi-agent collaboration

### A2A Protocol
- [ ] T124 Implement A2A message bus (Redis-based)
- [ ] T125 Add agent discovery mechanism
- [ ] T126 Implement capability advertisement
- [ ] T127 Add agent negotiation protocol

### A2A Collaboration
- [ ] T128 Implement task delegation messages
- [ ] T129 Add progress update messages
- [ ] T130 Implement result delivery
- [ ] T131 Add conflict resolution

### A2A Security
- [ ] T132 Implement agent authentication
- [ ] T133 Add message signing/verification
- [ ] T134 Implement permission scoping per agent

**Checkpoint**: Two agents on different frameworks can collaborate

---

## Phase 8: API Ecosystem (Stage 4 - Strategic)

**Purpose**: OpenAPI 3.1.1, Standard Webhooks, SDK generation

### OpenAPI
- [ ] T135 Add `utoipa` for OpenAPI generation
- [ ] T136 Document all API endpoints
- [ ] T137 Generate OpenAPI 3.1.1 spec
- [ ] T138 Add spec validation in CI
- [ ] T139 Implement contract tests

### Webhooks
- [ ] T140 Create WebhookSubscription model
- [ ] T141 Implement webhook delivery queue
- [ ] T142 Add HMAC-SHA256 signature generation
- [ ] T143 Implement exponential backoff retry
- [ ] T144 Add circuit breaker for failing endpoints
- [ ] T145 Implement Standard Webhooks spec compliance

### SDK Generation
- [ ] T146 Add OpenAPI generator to CI
- [ ] T147 Generate TypeScript SDK
- [ ] T148 Generate Python SDK
- [ ] T149 Generate Rust SDK
- [ ] T150 Publish SDKs to registries

**Checkpoint**: OpenAPI spec generates working SDKs

---

## Phase 9: Frontend (Solid.js) & Accessibility

**Purpose**: Solid.js SPA, WCAG 2.1 AA, BITV 2.0

### Solid.js Foundation
- [ ] T151 Setup Solid.js project structure
- [ ] T152 Implement router (solid-router)
- [ ] T153 Create reactive stores (auth, channels, messages)
- [ ] T154 Implement API client (OpenAPI-generated)

### Core UI
- [ ] T155 Implement login page (OIDC/SAML)
- [ ] T156 Implement channel list sidebar
- [ ] T157 Implement message list component
- [ ] T158 Implement message input/editor
- [ ] T159 Implement thread view

### Accessibility
- [ ] T160 Add keyboard navigation (TabIndex handlers)
- [ ] T161 Implement ARIA labels and live regions
- [ ] T162 Add focus management (trap in modals)
- [ ] T163 Implement skip links
- [ ] T164 Add high contrast mode support
- [ ] T165 Implement 200% zoom support
- [ ] T166 Add axe-core automated testing

### Mobile Compatibility
- [ ] T167 Implement responsive design
- [ ] T168 Add touch gesture support
- [ ] T169 Test with Mattermost mobile app

**Checkpoint**: axe-core passes with 0 violations, keyboard nav complete

---

## Phase 10: DevSecOps & Deployment

**Purpose**: Production hardening, observability, zero-downtime deploys

### Security Hardening
- [ ] T170 Run `cargo audit` and fix all findings
- [ ] T171 Run `cargo deny` and validate licenses
- [ ] T172 Implement container image signing (cosign)
- [ ] T173 Add container scanning (Trivy) to CI
- [ ] T174 Implement secret management (Vault integration)

### Kubernetes
- [ ] T175 Create backend deployment manifests
- [ ] T176 Create frontend deployment manifests
- [ ] T177 Add HPA for auto-scaling
- [ ] T178 Configure pod disruption budgets
- [ ] T179 Implement blue-green deployment strategy

### Monitoring
- [ ] T180 Configure Prometheus ServiceMonitor
- [ ] T181 Add Grafana dashboards
- [ ] T182 Configure Alertmanager rules
- [ ] T183 Implement SLO tracking
- [ ] T184 Add distributed tracing visualization

**Checkpoint**: `cargo audit` clean, zero-downtime deploys working

---

## Phase 11: Load Testing & Performance Validation

**Purpose**: Verify scalability requirements

### Load Testing
- [ ] T185 Create k6 load test scripts
- [ ] T186 Test 50,000 concurrent WebSocket connections
- [ ] T187 Test 200,000 concurrent users (target)
- [ ] T188 Verify p99 latency < 150ms
- [ ] T189 Test 30 auth/sec sustained
- [ ] T190 Test 150k auth in 90min window

### Kafka Performance
- [ ] T191 Test 10,000-member channel fan-out
- [ ] T192 Verify <5ms fan-out latency
- [ ] T193 Test Kafka failover to PostgreSQL

### Chaos Testing
- [ ] T194 Test Redis partition handling
- [ ] T195 Test PostgreSQL replica failover
- [ ] T196 Test Kafka broker loss
- [ ] T197 Verify graceful degradation

**Checkpoint**: All SC-001 through SC-010 success criteria met

---

## Dependencies & Execution Order

### Phase Dependencies
```
Phase 0 (Foundation)
  → Phase 1 (Auth/State)
    → Phase 2 (Messaging)
      → Phase 3 (RBAC) → Phase 4 (GDPR)
      → Phase 5 (Search/Kafka)
        → Phase 6 (MCP) → Phase 7 (A2A)
        → Phase 8 (APIs/Webhooks)
          → Phase 9 (Frontend)
            → Phase 10 (DevSecOps)
              → Phase 11 (Load Testing)
```

### Parallel Opportunities
- Phases 3-4 (RBAC/GDPR) can run in parallel with Phase 5 (Search)
- Phases 6-7 (MCP/A2A) can run in parallel with Phase 8 (APIs)
- Phase 9 (Frontend) can start after Phase 2 (Messaging API stable)

---

## Constitution Compliance Checklist

### Per Phase Verification

**Phase 0**: [X] XIX DevSecOps gates in CI
**Phase 1**: [X] XIV PKCE, XI Stateless, XVI Audit infra
**Phase 2**: [X] VI Mobile compatibility, I Contract parity
**Phase 3**: [X] XV RBAC, XVI Audit logs
**Phase 4**: [X] XVII GDPR compliance
**Phase 5**: [X] XI Scalability, IX Performance tests
**Phase 6**: [X] XIII Zero-trust agents
**Phase 7**: [X] XIII A2A security
**Phase 8**: [X] I OpenAPI contracts, VIII Webhook safety
**Phase 9**: [X] XVIII Accessibility
**Phase 10**: [X] XIX cargo-audit, deny, cosign
**Phase 11**: [X] IX All success criteria validated

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- Each phase has a checkpoint before proceeding
- Constitution compliance verified at each checkpoint
- All tests must fail before implementation (TDD)
- Security audit required before Phase 10 completion
