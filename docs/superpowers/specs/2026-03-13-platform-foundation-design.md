# Design: RustChat Platform Foundation

**Status**: Draft
**Branch**: `001-platform-foundation`
**Created**: 2026-03-13
**Input**: Platform architecture specification covering technology stack, scalability, agent integration, APIs, and SaaS maturity lifecycle

---

## Problem Statement

Build the foundational RustChat platform supporting 50,000-200,000 concurrent users with p99 latency under 150ms. The platform must implement enterprise-grade security (SAML/OIDC), AI agent integration (MCP/A2A), GDPR compliance, and WCAG 2.1 AA accessibility. Deliver via 4-stage SaaS maturity model (Ad-Hoc → Reactive → Proactive → Strategic).

---

## Goals

1. Support 50,000-200,000 concurrent users with horizontal scaling
2. Achieve p99 latency < 150ms for real-time messaging
3. Sustain 30 authentication requests/second (150k in 90min window)
4. Implement Mattermost API v4 compatibility for mobile clients
5. Support SAML 2.0 and OIDC with PKCE for federated identity
6. Implement MCP for secure AI agent data access
7. Implement A2A protocol for agent-to-agent collaboration
8. Expose OpenAPI 3.1.1 specification for all REST endpoints
9. Deliver Standard Webhooks with HMAC-SHA256 signatures
10. Enforce GDPR-compliant data retention and Right to Erasure
11. Achieve WCAG 2.1 Level AA and BITV 2.0 accessibility compliance

---

## Non-Goals

1. Custom mobile apps (use rebranded Mattermost mobile)
2. Video conferencing beyond WebRTC SFU
3. Blockchain or distributed ledger features
4. Custom plugin architecture (use MCP/A2A instead)

---

## Architecture

### Technology Stack

**Backend**:
- Rust 1.75+ with Axum 0.8 web framework
- Tokio 1.35+ async runtime
- SQLx 0.8 for compile-time checked SQL
- PostgreSQL 16+ (primary, primary-replica)
- Redis 7+ (ephemeral sessions, presence, WebSocket routing)
- AWS OpenSearch (optional, >5M posts)
- rdkafka 0.36 for message streaming

**Frontend**:
- Solid.js 1.8+ SPA
- Reactive state management (Stores)
- TypeScript 5.9+

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Load Balancer                        │
└──────────────────────┬──────────────────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
   ┌────▼────┐    ┌────▼────┐    ┌────▼────┐
   │Backend 1│    │Backend 2│    │Backend N│  (Stateless)
   └────┬────┘    └────┬────┘    └────┬────┘
        │              │              │
        └──────────────┼──────────────┘
                       │
       ┌───────────────┼───────────────┐
       │               │               │
  ┌────▼────┐     ┌────▼────┐    ┌────▼────┐
  │PostgreSQL│     │  Redis  │    │  Kafka  │
  │Primary  │     │ Cluster │    │ Cluster │
  └─────────┘     └─────────┘    └─────────┘
```

---

## Key Entities

- **User**: Identity with federated credentials, roles (System/Team/Channel/Guest), session context
- **Channel**: Communication container with type (public/private/direct), membership, permissions
- **Post**: Message entity with content, metadata, thread relationship, retention policy
- **Session**: Ephemeral authentication state stored in Redis, bound to device/MFA context
- **Agent**: AI entity with MCP tool connections, A2A capabilities, user-delegated permissions
- **MCPConnection**: Secure bridge to external data source via Model Context Protocol
- **WebhookSubscription**: Outbound event delivery configuration with HMAC secrets

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

### Phase 1: State Decoupling & Authentication (Stage 2 - Reactive)

**Goal**: Decouple state to Redis, implement OIDC/SAML, add telemetry

**Key Deliverables**:
- Redis connection pooling and session storage
- OIDC flow with PKCE (SPA)
- SAML 2.0 integration with XML signature validation
- WebSocket state externalization (Redis pub/sub)
- Prometheus metrics export
- Grafana dashboards

### Phase 2: Security & Compliance (Stage 3 - Proactive)

**Goal**: Strict RBAC, search clustering, GDPR retention, immutable audit logs

**Key Deliverables**:
- Multi-dimensional RBAC (Global/Team/Channel)
- AWS OpenSearch integration (>5M posts)
- GDPR data retention automation
- Immutable audit logging (async, SIEM format)
- Data export (GDPR Article 20)
- Right to Erasure (cryptographic wipe)

### Phase 3: AI Integration & APIs (Stage 4 - Strategic)

**Goal**: MCP/A2A protocols, OpenAPI, Standard Webhooks, dynamic scaling

**Key Deliverables**:
- MCP server implementation (JSON-RPC)
- A2A protocol message bus
- OpenAPI 3.1.1 spec generation (Utoipa)
- SDK generation pipeline
- Standard Webhooks with HMAC
- Kubernetes HPA for dynamic scaling

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

---

## Data Sovereignty

**External Dependencies**: Required (opt-in configurable)

**Network Egress**:
- Core messaging: None (air-gapped capable)
- AI/Agent features: Opt-in (MCP/A2A external connections)
- Search: AWS OpenSearch (opt-in for >5M posts)
- Authentication: IdP-dependent (OIDC/SAML)

**Data Locality**:
- Data stored in: PostgreSQL (primary), Redis (ephemeral), S3-compatible (files)
- Cross-boundary data flow: AI agent context (user-approved), search index (if OpenSearch enabled)

**Multi-tenant Impact**: High
- Tenant isolation: Database-level (schema separation or instance isolation)
- Cross-tenant data: Strictly prohibited

---

## Mobile Alignment

**Mobile Compatibility Impact**: High

**Mobile-Specific Considerations**:
- API v4 endpoints affected: All (Mattermost compatibility layer)
- WebSocket events affected: Presence, messaging, notifications, calls
- Calls plugin impact: WebRTC SFU integration
- Mobile journey verification: Login, Channels, Posts, Notifications, Files, Calls

**Upstream Evidence**:
- Mattermost server version analyzed: 9.4.0
- Mobile client: mattermost-mobile (rebranded RustChat Mobile)
- Compatibility strategy: API v4 parity with native extensions

---

## Security & Identity

### Zero-Trust Extensibility

- Integration Scope: Channel-scoped for agents, User-scoped for MCP
- Webhook Security: HMAC-SHA256 signature verification mandatory
- AI Agent Access: API-only with user-delegated tokens, no direct DB access

### Federated Identity

- Authentication Impact: OIDC flow with PKCE for SPA, SAML for enterprise
- PKCE Requirement: Required for all OIDC flows
- IdP Integration: SSO login, user provisioning via SCIM (future)

### RBAC Enforcement

- New Permissions: agent:invoke, agent:deploy, mcp:connect, webhook:configure
- Role Assignment: System Admin for agent deployment, User for invocation
- Authorization: Middleware check on every API endpoint, resource-level for mutations

---

## Privacy & Compliance

**Data Classification**: Internal to Highly Confidential (configurable per workspace)

**Retention Policy**:
- Default retention: Workspace-configurable (30 days to indefinite)
- Auto-deletion: Yes, with configurable schedules
- Legal hold support: Yes (overrides auto-deletion)

**Right to Erasure Impact**:
- Feature data included in user export
- Feature data purged on account deletion
- Cryptographic wipe for file attachments

---

## Test Coverage Requirements

### Protocol Behavior Tests
- [ ] API contract compliance (Mattermost v4 parity)
- [ ] OpenAPI 3.1.1 spec validation
- [ ] WebSocket event format and delivery
- [ ] MCP JSON-RPC protocol compliance
- [ ] A2A protocol message exchange
- [ ] Standard Webhooks signature verification

### Permission Tests
- [ ] Happy path authorization (all RBAC levels)
- [ ] Edge case permission checks (expired sessions, revoked roles)
- [ ] Privilege escalation attempts (horizontal, vertical)
- [ ] Cross-tenant isolation (data leakage prevention)
- [ ] Agent permission scoping

### Failure Case Tests
- [ ] Kafka unavailable (PostgreSQL fallback)
- [ ] Redis partition (degraded presence)
- [ ] IdP unreachable (graceful session handling)
- [ ] MCP connection failure (user notification)
- [ ] Webhook delivery failure (exponential backoff)
- [ ] Load spike (auto-scaling, queue buffering)

---

## DevSecOps Requirements

**Build Requirements**:
- `cargo clippy` with zero warnings (`-D warnings`)
- `cargo audit` passes (no high/critical CVEs)
- `cargo deny` license check passes (whitelist OSI-approved)

**Dependency Impact**:
- New crates: `axum`, `tokio`, `sqlx`, `redis`, `rdkafka`, `openidconnect`, `samael`, `utoipa`
- License compatibility: All MIT/Apache-2.0
- Security audit: Required for auth crates

**Deployment Impact**:
- Container image signing (cosign)
- Database migrations in CI/CD
- Feature flags for gradual rollout

---

## Success Criteria

### Measurable Outcomes

- **SC-001**: Platform sustains 200,000 concurrent users with p99 latency < 150ms
- **SC-002**: Morning auth spike (30/sec, 150k/90min) completes with <0.1% failure rate
- **SC-003**: Kafka fan-out achieves <5ms latency for 10,000-member channel broadcasts
- **SC-004**: MCP/A2A agent integration passes security audit (zero critical findings)
- **SC-005**: OpenAPI 3.1.1 spec generates functional SDKs for TypeScript, Python, Rust
- **SC-006**: Webhook delivery achieves 99.9% success rate with Standard Webhooks compliance
- **SC-007**: GDPR Right to Erasure completes within 30 days, cryptographically verified
- **SC-008**: Accessibility audit passes WCAG 2.1 AA and BITV 2.0 with zero violations
- **SC-009**: `cargo audit` shows zero high/critical CVEs at release
- **SC-010**: Zero-downtime deployment achieved via blue-green strategy

---

## Related Documents

- Original spec: `specs/001-platform-foundation/spec.md`
- Original plan: `specs/001-platform-foundation/plan.md`
