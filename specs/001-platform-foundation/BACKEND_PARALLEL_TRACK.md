# Backend Parallel Track

**Approach**: Option B - Frontend-First with Backend in Parallel  
**Duration**: Weeks 1-13 (overlaps with frontend migration)

While frontend team works on Solid.js migration (13 weeks), backend team addresses critical gaps in parallel.

---

## Parallel Track Philosophy

1. **Frontend team**: Owns Solid.js migration (F0-F9 phases)
2. **Backend team**: Addresses critical gaps (B1-B5 tracks)
3. **Integration points**: Weekly sync, shared API contracts
4. **Goal**: Backend ready when frontend needs new capabilities

---

## Track B1: Security & Compliance (Weeks 1-4)

**Priority**: Critical  
**Goal**: Close Constitution XVII (Privacy) and XIX (DevSecOps) gaps

### B1.1 DevSecOps Hardening

- [ ] **B101** Add `cargo-audit` to CI pipeline
  - File: `.github/workflows/backend-ci.yml`
  - Block build on high/critical CVEs
  - Weekly automated scans

- [ ] **B102** Add `cargo-deny` to CI pipeline
  - File: `backend/deny.toml`
  - License whitelist (OSI-approved)
  - Ban copyleft where prohibited

- [ ] **B103** Container image signing (cosign)
  - File: `.github/workflows/docker-publish.yml`
  - Sign images on release
  - Verify signatures in deploy

- [ ] **B104** Container vulnerability scanning
  - Tool: Trivy or Grype
  - Scan on every build
  - Block on critical vulnerabilities

- [ ] **B105** SBOM generation
  - Tool: `cargo-cyclonedx` or `syft`
  - Generate SPDX on release
  - Store with artifacts

### B1.2 GDPR Hard Delete Implementation

- [ ] **B106** Audit current soft delete usage
  - Search all `deleted_at` columns
  - Identify tables requiring hard delete

- [ ] **B107** Implement hard delete for posts
  - File: `backend/src/services/posts.rs`
  - Delete from `posts` table
  - Cascade to reactions, files

- [ ] **B108** Implement hard delete for users (Right to Erasure)
  - File: `backend/src/api/users.rs`
  - Endpoint: `DELETE /api/v4/users/{user_id}`
  - Cryptographic wipe for files
  - PostgreSQL VACUUM after deletion

- [ ] **B109** File cryptographic wipe
  - File: `backend/src/storage/mod.rs`
  - Overwrite file content before S3 delete
  - Certificate of destruction

- [ ] **B110** Data export (GDPR Article 20)
  - File: `backend/src/api/users.rs`
  - Endpoint: `GET /api/v4/users/{user_id}/export`
  - JSON format, all user data

- [ ] **B111** Update retention job
  - File: `backend/src/jobs/retention.rs`
  - Use `DELETE` not `UPDATE deleted_at`

### B1.3 SAML Security Hardening

- [ ] **B112** XML signature validation hardening
  - File: `backend/src/api/v4/saml.rs`
  - Prevent XML signature wrapping attacks
  - Strict schema validation

**Checkpoint**: `cargo audit` clean, GDPR hard delete tested

---

## Track B2: Scalability Foundation (Weeks 5-8)

**Priority**: High  
**Goal**: Prepare for 200k users, p99 < 150ms

### B2.1 Kafka Integration

- [ ] **B201** Add `rdkafka` dependency
  - File: `backend/Cargo.toml`
  - Features: `tokio`, `ssl`

- [ ] **B202** Create Kafka configuration
  - File: `backend/src/config/kafka.rs`
  - Bootstrap servers, topics, consumer groups

- [ ] **B203** Implement Kafka producer
  - File: `backend/src/services/kafka_producer.rs`
  - Topic: `rustchat.posts`
  - Async delivery, retry logic

- [ ] **B204** Implement Kafka consumer
  - File: `backend/src/services/kafka_consumer.rs`
  - Consumer group: `websocket-fanout`
  - Offset management

- [ ] **B205** Massive channel fan-out
  - File: `backend/src/realtime/fanout.rs`
  - Use Kafka for >1000 member channels
  - Fallback to Redis for smaller channels

- [ ] **B206** Kafka monitoring
  - Metrics: lag, throughput, errors
  - Alerts: consumer lag > 10,000

### B2.2 OpenSearch Integration

- [ ] **B207** Add OpenSearch client
  - Crate: `opensearch` or `elasticsearch`
  - AWS SigV4 authentication

- [ ] **B208** Create indexing pipeline
  - File: `backend/src/search/indexer.rs`
  - Index posts on creation/update
  - Bulk indexing for backfill

- [ ] **B209** Search API with OpenSearch fallback
  - File: `backend/src/api/search.rs`
  - Use OpenSearch if available
  - Fallback to PostgreSQL FTS

- [ ] **B210** Index management
  - Create index mappings
  - Index rollover strategy
  - Reindexing capability

### B2.3 High Availability Setup

- [ ] **B211** PostgreSQL primary-replica
  - Docker Compose for local testing
  - Read replica routing

- [ ] **B212** Redis Cluster
  - Cluster mode configuration
  - Slot migration testing

- [ ] **B213** Kubernetes manifests
  - File: `infrastructure/k8s/`
  - Deployment, Service, HPA
  - ConfigMap, Secret management

**Checkpoint**: Kafka/OpenSearch local stack working

---

## Track B3: AI Integration - MCP (Weeks 9-12)

**Priority**: Critical (Constitution XIII)  
**Goal**: Model Context Protocol for secure AI agent access

### B3.1 MCP Server Foundation

- [ ] **B301** Create MCP module
  - Directory: `backend/src/mcp/`
  - JSON-RPC 2.0 protocol implementation

- [ ] **B302** MCP endpoint
  - File: `backend/src/api/mcp.rs`
  - Route: `POST /mcp`
  - WebSocket upgrade for streaming

- [ ] **B303** Capability negotiation
  - File: `backend/src/mcp/capabilities.rs`
  - Advertise available tools
  - Version negotiation

- [ ] **B304** Tool schema registry
  - File: `backend/src/mcp/tools.rs`
  - Define tool schemas (JSON Schema)
  - Tool discovery endpoint

### B3.2 MCP Tools Implementation

- [ ] **B305** Channel query tool
  - List channels
  - Get channel history
  - Search messages

- [ ] **B306** User query tool
  - Get user profile
  - List team members
  - Check user status

- [ ] **B307** Message creation tool
  - Post message to channel
  - Create thread reply
  - Add reaction

- [ ] **B308** File access tool
  - List files in channel
  - Get file metadata
  - Generate temporary download URL

### B3.3 MCP Security

- [ ] **B309** User approval workflow
  - File: `backend/src/mcp/approval.rs`
  - OAuth-style consent screen
  - Scoped permissions per tool

- [ ] **B310** Token-based auth
  - MCP connection tokens
  - Expiry and refresh
  - Revocation

- [ ] **B311** Rate limiting
  - Per-user limits
  - Per-tool limits
  - Burst handling

- [ ] **B312** Audit logging
  - Log all MCP invocations
  - User, tool, parameters, result

**Checkpoint**: MCP server passes protocol compliance test

---

## Track B4: AI Integration - A2A (Weeks 11-13)

**Priority**: Critical (Constitution XIII)  
**Goal**: Agent-to-Agent protocol for multi-agent collaboration

### B4.1 A2A Message Bus

- [ ] **B401** Create A2A module
  - Directory: `backend/src/a2a/`
  - Redis-based message bus

- [ ] **B402** Agent registry
  - File: `backend/src/a2a/registry.rs`
  - Agent discovery
  - Capability advertisement
  - Heartbeat monitoring

- [ ] **B403** Message types
  - TaskRequest
  - TaskResponse
  - StatusUpdate
  - Negotiation

- [ ] **B404** Message routing
  - Direct agent-to-agent
  - Broadcast to capability group
  - Pub/sub patterns

### B4.2 A2A Collaboration

- [ ] **B405** Task delegation
  - File: `backend/src/a2a/tasks.rs`
  - Create task
  - Assign to agent
  - Track status

- [ ] **B406** Negotiation protocol
  - Capability matching
  - Bid/ask workflow
  - Agreement formation

- [ ] **B407** State synchronization
  - Shared state store (Redis)
  - Conflict resolution
  - Consensus for critical decisions

### B4.3 A2A Security

- [ ] **B408** Agent authentication
  - X.509 certificates
  - JWT tokens
  - Mutual TLS

- [ ] **B409** Message signing
  - Sign all A2A messages
  - Verify signatures
  - Nonce replay protection

- [ ] **B410** Permission scoping
  - Agent capabilities bound to user
  - No privilege escalation
  - Audit all agent actions

**Checkpoint**: Two agents can discover and collaborate

---

## Track B5: API Ecosystem (Weeks 10-13)

**Priority**: High  
**Goal**: OpenAPI 3.1.1, Standard Webhooks

### B5.1 OpenAPI 3.1.1

- [ ] **B501** Add `utoipa` dependency
  - File: `backend/Cargo.toml`
  - Derive macros for OpenAPI

- [ ] **B502** Annotate all API endpoints
  - Add `#[utoipa::path]` to handlers
  - Document request/response schemas

- [ ] **B503** Generate OpenAPI spec
  - File: `backend/openapi.yaml`
  - Validate against 3.1.1 schema
  - CI check for spec freshness

- [ ] **B504** Swagger UI endpoint
  - Route: `/api/docs`
  - Interactive documentation

### B5.2 SDK Generation

- [ ] **B505** TypeScript SDK
  - Tool: `openapi-generator-cli`
  - Publish to npm

- [ ] **B506** Python SDK
  - Tool: `openapi-generator-cli`
  - Publish to PyPI

- [ ] **B507** Rust SDK
  - Tool: `openapi-generator-cli`
  - Publish to crates.io

### B5.3 Standard Webhooks

- [ ] **B508** Migrate to Standard Webhooks
  - File: `backend/src/services/webhooks.rs`
  - HMAC-SHA256 signing
  - Standard payload format

- [ ] **B509** Webhook verification library
  - Extract to separate crate
  - Publish for community use

**Checkpoint**: OpenAPI spec generates working SDKs

---

## Integration Schedule

### Week 1-4: Security Track
| Week | Frontend | Backend |
|------|----------|---------|
| 1 | F0: Setup | B1.1: DevSecOps |
| 2 | F1: Infrastructure | B1.1: DevSecOps |
| 3 | F1: Infrastructure | B1.2: GDPR hard delete |
| 4 | F2: Auth | B1.3: SAML hardening |

### Week 5-8: Scalability Track
| Week | Frontend | Backend |
|------|----------|---------|
| 5 | F3: Layout | B2.1: Kafka |
| 6 | F4: Messaging | B2.1: Kafka, B2.2: OpenSearch |
| 7 | F4: Messaging | B2.2: OpenSearch, B2.3: K8s |
| 8 | F4: Messaging | B2.3: K8s |

### Week 9-13: AI & API Track
| Week | Frontend | Backend |
|------|----------|---------|
| 9 | F5: Channels | B3.1: MCP Foundation |
| 10 | F6: Settings | B3.2: MCP Tools, B5: OpenAPI |
| 11 | F7: Accessibility | B3.3: MCP Security, B4.1: A2A |
| 12 | F8: Polish | B4.2-4.3: A2A Collaboration |
| 13 | F9: Deploy | B5: SDKs, Integration testing |

---

## API Contract Stability

During frontend migration, backend API must remain stable:

### Stable Contracts (Don't Change)
- All `/api/v4/*` Mattermost-compatible endpoints
- WebSocket event format
- Authentication flows (OIDC/SAML)

### New Capabilities (Additive Only)
- MCP endpoint (`/mcp`) - NEW
- A2A endpoints - NEW
- Enhanced search - BACKWARD COMPATIBLE
- Webhook improvements - BACKWARD COMPATIBLE

---

## Communication Plan

### Weekly Sync
- Frontend progress demo
- Backend API readiness
- Integration blockers

### API Versioning
- Current: v4 (stable)
- New features: v4 with feature flags
- Breaking changes: v5 (post-migration)

### Documentation
- API changes documented immediately
- Frontend team notified of new endpoints
- Shared Postman collection

---

## Success Criteria by Track

### B1: Security & Compliance
- [ ] `cargo audit` passes in CI
- [ ] `cargo deny` passes in CI
- [ ] GDPR hard delete functional
- [ ] Data export works
- [ ] SAML hardened against attacks

### B2: Scalability
- [ ] Kafka local stack working
- [ ] OpenSearch indexing functional
- [ ] K8s manifests deployable
- [ ] Load tests pass (target: 50k users)

### B3: MCP
- [ ] MCP server passes protocol test
- [ ] All tools functional
- [ ] User approval workflow works
- [ ] Audit logging complete

### B4: A2A
- [ ] Agent discovery works
- [ ] Task delegation functional
- [ ] Two agents can collaborate
- [ ] Security audit passes

### B5: API Ecosystem
- [ ] OpenAPI 3.1.1 spec valid
- [ ] TypeScript SDK generates
- [ ] Standard Webhooks compliant

---

## Resource Allocation

### Team Structure (Recommended)

**Frontend Team (2-3 devs)**
- 1 Senior: Architecture, performance
- 1-2 Mid: Components, features

**Backend Team (2-3 devs)**
- 1 Senior: MCP/A2A protocols
- 1 Mid: Kafka/OpenSearch
- 1 Mid: Security, compliance

**Shared**
- DevOps: CI/CD, K8s
- QA: Testing both tracks

---

*End of Backend Parallel Track*
