# Foundation Docs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create 7 structured foundation docs that consolidate rustchat's ~35 ad-hoc flat files into the format required by the LLM Development Operating Model v2.0, so LLM agents have reliable navigable reference material.

**Architecture:** Each task creates one structured doc (or a small group of related files) by writing complete content derived from existing flat files. 4 superseded flat files are replaced with one-line redirect stubs. No production code is touched.

**Tech Stack:** Markdown only. Verification is `ls` and `grep` to confirm files and key headings exist.

**Spec:** `docs/superpowers/specs/2026-03-22-foundation-docs-design.md`

---

## File Map

| File | Action |
|---|---|
| `docs/architecture/architecture-overview.md` | Create (new directory) |
| `docs/agent-operating-model.md` | Create |
| `docs/compatibility-scope.md` | Create |
| `docs/testing-model.md` | Create |
| `docs/ownership-map.md` | Create |
| `docs/repo-current-state.md` | Create |
| `docs/target-operating-model.md` | Create |
| `docs/adr/README.md` | Create (new directory + stub) |
| `docs/decision-notes/README.md` | Create (new directory + stub) |
| `docs/architecture.md` | Modify → redirect stub |
| `docs/websocket_architecture.md` | Modify → redirect stub |
| `docs/MATTERMOST_CLIENTS.md` | Modify → redirect stub |
| `docs/mobile-compatibility-matrix.md` | Modify → redirect stub |
| `README.md` | Modify → add documentation table |

---

## Task 1: Architecture overview

**Files:**
- Create: `docs/architecture/architecture-overview.md`
- Modify: `docs/architecture.md` (redirect stub)
- Modify: `docs/websocket_architecture.md` (redirect stub)

- [ ] **Step 1: Create `docs/architecture/` directory and write `architecture-overview.md`**

```bash
mkdir -p docs/architecture
```

Write `docs/architecture/architecture-overview.md` with the following complete content:

````markdown
# Architecture Overview

**Last updated:** 2026-03-22
**Source docs consolidated:** `docs/architecture.md`, `docs/websocket_architecture.md`, `docs/calls_deployment_modes.md`

---

## 1. System Overview

rustchat is a self-hosted team collaboration platform composed of 4 services:

| Service | Language | Purpose |
|---|---|---|
| `backend` | Rust (Axum 0.8 + Tokio) | HTTP API, WebSocket hub, business logic, DB |
| `frontend` | Vue 3.5 + TypeScript + Pinia | Single-page web application |
| `push-proxy` | Rust | Mobile push notification gateway (FCM/APNS) |
| `tools/mm-compat` | Python | Offline Mattermost compatibility analysis tooling |

**External dependencies:**

| Dependency | Purpose | Required |
|---|---|---|
| PostgreSQL 16+ | Primary data store | Yes |
| Redis 7+ | Pub/sub for cross-instance events, rate limiting, sessions | Yes |
| S3-compatible (RustFS/MinIO) | File storage | Yes |
| FCM / APNS | Mobile push notifications (via push-proxy) | Optional |
| SMTP | Email notifications, password reset | Optional |
| OAuth providers | SSO login (configurable) | Optional |

```
┌────────────────────────────────────────────────────────────────────┐
│                        Web / SPA Client                            │
│                  (Vue 3.5 + TypeScript + Pinia)                    │
└──────────────────────────────┬─────────────────────────────────────┘
                               │ REST + WebSocket
                               ▼
┌────────────────────────────────────────────────────────────────────┐
│                      rustchat API Server                           │
│                      (Axum 0.8 + Tokio)                            │
│                                                                    │
│  /api/v1/*  ──── native API (internal clients)                     │
│  /api/v4/*  ──── Mattermost-compatible API (mobile/desktop clients)│
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                     Service Layer                            │  │
│  │  auth · channels · posts · files · realtime · jobs · a2a    │  │
│  └──────────────────────┬───────────────────────────────────────┘  │
└─────────────────────────┼──────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        ▼                 ▼                 ▼
   PostgreSQL           Redis           S3-compatible
   (primary data)    (events/cache)     (file storage)

┌──────────────────────────────┐
│       push-proxy             │
│  (Rust, separate service)    │
│  Receives events from        │
│  backend → delivers to       │
│  FCM (Android) / APNS (iOS)  │
└──────────────────────────────┘
```

---

## 2. Backend

**Stack:** Rust, Axum 0.8, Tokio, SQLx (compile-time checked queries), Tower middleware

**Top-level module structure** (`backend/src/`):

| Module | Responsibility |
|---|---|
| `api/` | HTTP handlers: v1 native API + v4 Mattermost-compatible API |
| `auth/` | Authentication: JWT generation/validation, password hashing (Argon2id) |
| `config/` | Environment-based configuration (the `config` crate) |
| `db/` | PostgreSQL connection pool, SQLx macros, migration runner |
| `error/` | Structured error types with HTTP status mapping |
| `jobs/` | Background job workers (async task queue) |
| `mattermost_compat/` | Mattermost-specific response transformation utilities |
| `middleware/` | Axum middleware: auth extraction, rate limiting, logging, CORS |
| `models/` | Data models: User, Channel, Post, Team, File, Entity, etc. |
| `realtime/` | WebSocket hub: connection management, event fan-out, cluster broadcast via Redis |
| `services/` | Business logic: one service per domain area (channels, posts, files, …) |
| `storage/` | S3-compatible file upload/download |
| `telemetry/` | Structured JSON logging with `tracing` |

Note: `channels` and `posts` are sub-modules under `api/v1/` and `services/`, not top-level modules.

**Request lifecycle:**
```
Request → Middleware (auth, rate-limit, CORS) → Router → Handler → Service → DB/Storage
                                                                           ↓
Response ← JSON serialization ← Result<T, AppError>
```

**Migrations:** `backend/migrations/` — SQLx numbered migrations, run automatically at startup. Irreversible — see `.governance/pr-size-limits.yml` for migration PR size constraints.

---

## 3. WebSocket Architecture

Two WebSocket endpoints share a common core (`websocket_core.rs`) but present different wire formats:

| Endpoint | Clients | Wire format |
|---|---|---|
| `/api/v1/ws` | rustchat web app, internal clients | Internal envelope (`type`, `event`, `data`, `channel_id`) |
| `/api/v4/websocket` | Mattermost mobile/desktop clients | Mattermost framing (`event`, `data`, `broadcast`, `seq`) |

**Shared core handles:**
- Auth token normalization (header + `Sec-WebSocket-Protocol` fallback)
- Connection limit enforcement
- Default team/channel subscription bootstrap
- Presence lifecycle: `online` on connect, `offline` when last connection drops
- Shared commands: `subscribe_channel`, `unsubscribe_channel`, `typing`, `presence`, `ping`→`pong`

**v4-specific behavior:**
- Optional auth challenge exchange (`action=authentication_challenge`)
- Session resumption (`connection_id`, `sequence_number`)
- Mattermost event name mapping: `posted`, `typing`, `post_edited`, `status_change`, etc.

**Event fan-out:**
```
Service writes event → realtime::hub → broadcast to subscribed connections
                                    → Redis pub/sub → other backend instances → their hubs
```

---

## 4. Frontend

**Stack:** Vue 3.5, TypeScript, Pinia (state), Vue Router, Vite (build)

**Directory structure:**
```
frontend/src/
├── core/          # Shared primitives: entities, errors, websocket infrastructure
├── features/      # 13 domain feature modules (auth, calls, channels, messages, …)
├── api/           # API client functions
├── components/    # Vue components
├── composables/   # Vue composables
└── stores/        # Legacy Pinia stores (deprecated, being migrated to features/)
```

**Feature module pattern** — every feature follows the same layers:
```
features/[feature]/
├── repositories/    # Data access (API calls)
├── services/        # Business logic
├── stores/          # Pinia state (no business logic)
├── handlers/        # WebSocket event handlers
└── index.ts         # Public API
```

**E2E tests:** Playwright snapshot tests in `frontend/e2e/`. Run with `cd frontend && npx playwright test`.

---

## 5. Push Proxy

Separate Rust service (`push-proxy/`). Receives push notification events from the backend via an internal HTTP call and forwards them to FCM (Android) or APNS (iOS). Deployed separately from the main backend to isolate credential scope.

---

## 6. Mattermost Compatibility Surface

The compatibility surface is the set of paths that external Mattermost clients depend on. Changes here require compat-reviewer co-approval (see `CODEOWNERS`):

| Path | Purpose |
|---|---|
| `backend/src/api/v4/` | Mattermost HTTP API v4 handlers |
| `backend/src/mattermost_compat/` | Response transformation, field mapping utilities |
| `backend/compat/` | Contract JSON schemas + contract validation tests |
| `backend/src/realtime/` | WebSocket hub (v4 event contracts) |

For coverage details see `docs/compatibility-scope.md`.

---

## 7. Key Design Decisions

- **Axum over Actix-web:** Tower middleware ecosystem, async-first, ergonomic extractors.
- **Separate push-proxy service:** Isolates FCM/APNS credentials; can be scaled/deployed independently.
- **Redis for cross-instance fan-out:** Enables horizontal scaling of API servers without sticky sessions.
- **SQLx compile-time query checks:** Prevents schema/query drift at the cost of requiring a live DB at compile time (see `SQLX_OFFLINE` flag for CI).
- **Two WebSocket endpoints (v1 + v4):** Avoids breaking the native web app while maintaining Mattermost client compatibility. Shared core prevents logic drift between them.
- **Feature-based frontend structure:** Avoids the 960-line store antipattern; enforces single responsibility (avg 105 lines/file post-refactor).
````

- [ ] **Step 2: Verify the file was created with key sections**

Run:
```bash
grep -c "^## " docs/architecture/architecture-overview.md
```

Expected: `7` (one line per `## N.` section heading).

- [ ] **Step 3: Replace `docs/architecture.md` with a redirect stub**

Overwrite `docs/architecture.md` entirely:

```markdown
> **Moved:** This document has been consolidated into [`docs/architecture/architecture-overview.md`](architecture/architecture-overview.md).
```

- [ ] **Step 4: Replace `docs/websocket_architecture.md` with a redirect stub**

Overwrite `docs/websocket_architecture.md` entirely:

```markdown
> **Moved:** WebSocket architecture details have been consolidated into [`docs/architecture/architecture-overview.md`](architecture/architecture-overview.md) (Section 3: WebSocket Architecture).
```

- [ ] **Step 5: Commit**

```bash
git add docs/architecture/ docs/architecture.md docs/websocket_architecture.md
git commit -m "docs: add architecture-overview, stub architecture.md and websocket_architecture.md"
```

---

## Task 2: Agent operating model

**Files:**
- Create: `docs/agent-operating-model.md`

Note: `AGENTS.md` at the root stays as-is — it remains the canonical agent guide. This doc provides the structured model-layer view.

- [ ] **Step 1: Write `docs/agent-operating-model.md`**

````markdown
# Agent Operating Model

**Last updated:** 2026-03-22
**Canonical agent guide:** [`AGENTS.md`](../AGENTS.md) (root) — read that first for full workflow details

---

## Overview

This document describes the LLM agent operating model for rustchat: which agents are authorized, what skills they use, and what scope boundaries apply. It structures the information per the LLM Development Operating Model v2.0.

The machine-readable contracts live in `.governance/agent-contracts.yml`. This doc is the human-readable summary.

---

## 1. Agent Roles

Three agents are defined for rustchat:

| Agent | Mode | Role |
|---|---|---|
| `backend-agent` | bounded-write | Implements backend features and fixes |
| `frontend-agent` | bounded-write | Implements frontend features and fixes |
| `compat-agent` | read-only | Performs Mattermost compatibility analysis, produces design artifacts |

### backend-agent

**Allowed paths:** `backend/src/**`, `backend/tests/**`, `backend/migrations/**`, `push-proxy/**`

**Prohibited without explicit approval:**
- `backend/src/auth/**` — requires explicit human approval
- `backend/src/api/v4/**` — requires compat-reviewer co-approval (`@zoorpha`)
- `backend/src/mattermost_compat/**` — requires compat-reviewer co-approval
- `backend/src/a2a/**` — requires senior review
- `.governance/**`, `frontend/**` — always prohibited

**Limits:** ≤10 files, ≤300 lines per PR. Tests required. Human review required.

**ADR required for:** auth changes, permission changes, API contract changes.

### frontend-agent

**Allowed paths:** `frontend/src/**`, `frontend/e2e/**`

**Prohibited:** `backend/**`, `.governance/**`

**Limits:** ≤10 files, ≤300 lines per PR. Human review required.

### compat-agent

**Mode:** Read-only — cannot write production code, cannot approve PRs.

**Can read:** `backend/compat/**`, `backend/src/api/v4/**`, `backend/src/mattermost_compat/**`, `tools/mm-compat/**`

**Can write (analysis output only):** `previous-analyses/**`, `docs/superpowers/specs/**`, `docs/superpowers/plans/**`

**Applicable skills:** `mattermost-api-parity`, `mm-endpoint-contract-parity`, `mm-websocket-calls-parity`, `mm-mobile-journey-parity`

---

## 2. Skills System

7 capability skills are defined in `.agents/skills/`:

| Skill | Domain |
|---|---|
| `mattermost-api-parity` | Compat: Mattermost HTTP API analysis |
| `mm-endpoint-contract-parity` | Compat: endpoint contract comparison |
| `mm-websocket-calls-parity` | Compat: WebSocket event contract analysis |
| `mm-mobile-journey-parity` | Compat: mobile user journey coverage |
| `production-readiness-gate` | Quality: pre-release checklist |
| `user-validation` | Quality: user-facing behavior verification |
| `ai-summarization-rag` | Feature: AI summarization / RAG pipeline |

**Relationship to agent contracts:** Skills define *how* (capability, method, output format). Contracts define *where* (allowed paths, scope boundaries). A skill tells the agent what to do; the contract tells it where it is allowed to do it.

For compat-agent: only the 4 compat skills apply. For backend-agent and frontend-agent: any non-compat skill.

---

## 3. Workflow

All agent work follows the **plan-first** workflow:

```
1. Brainstorm → spec (docs/superpowers/specs/YYYY-MM-DD-<topic>-design.md)
2. Write plan (docs/superpowers/plans/YYYY-MM-DD-<topic>.md)
3. Execute plan task-by-task (subagent-driven-development or executing-plans)
4. Two-stage review per task: spec compliance → code quality
5. finishing-a-development-branch
```

No ad-hoc changes to protected paths. No implementation without a prior spec.

---

## 4. Compatibility-First Rule

Any change to the Mattermost compatibility surface (`backend/src/api/v4/**`, `backend/src/mattermost_compat/**`, `backend/compat/**`) requires:

1. Compat reviewer co-approval (`@zoorpha` per `CODEOWNERS`)
2. Risk tier elevated (automatic per `protected-paths.yml`)
3. Compat contract tests pass

When in doubt whether a change touches the compat surface, treat it as if it does.

---

## 5. Boundaries Summary

| Area | backend-agent | frontend-agent | compat-agent |
|---|---|---|---|
| `backend/src/` (non-protected) | ✅ write | ❌ | ✅ read |
| `backend/src/api/v4/` | ⚠️ co-approval | ❌ | ✅ read |
| `backend/src/auth/` | ⚠️ explicit approval | ❌ | ❌ |
| `backend/src/mattermost_compat/` | ⚠️ co-approval | ❌ | ✅ read |
| `backend/src/a2a/` | ⚠️ senior review | ❌ | ❌ |
| `backend/migrations/` | ✅ write | ❌ | ❌ |
| `backend/tests/` | ✅ write | ❌ | ❌ |
| `frontend/src/` | ❌ | ✅ write | ❌ |
| `frontend/e2e/` | ❌ | ✅ write | ❌ |
| `.governance/` | ❌ | ❌ | ❌ |
| `docs/superpowers/specs/` | ❌ | ❌ | ✅ write |
| `docs/superpowers/plans/` | ❌ | ❌ | ✅ write |

---

## 6. Cross-References

- Full agent workflow guide: [`AGENTS.md`](../AGENTS.md)
- Machine-readable contracts: [`.governance/agent-contracts.yml`](../.governance/agent-contracts.yml)
- Risk tier definitions: [`.governance/risk-tiers.yml`](../.governance/risk-tiers.yml)
- Protected paths: [`.governance/protected-paths.yml`](../.governance/protected-paths.yml)
- Code ownership: [`.github/CODEOWNERS`](../.github/CODEOWNERS)
````

- [ ] **Step 2: Verify**

```bash
grep -c "^## " docs/agent-operating-model.md
```

Expected: `6`

- [ ] **Step 3: Commit**

```bash
git add docs/agent-operating-model.md
git commit -m "docs: add agent-operating-model"
```

---

## Task 3: Compatibility scope

**Files:**
- Create: `docs/compatibility-scope.md`
- Modify: `docs/MATTERMOST_CLIENTS.md` (redirect stub)
- Modify: `docs/mobile-compatibility-matrix.md` (redirect stub)

Note: `docs/V4_API_GAP_ANALYSIS.md` is NOT stubbed — it covers a different scope (full v4 surface ~60%) and stays as a historical analysis artifact.

- [ ] **Step 1: Write `docs/compatibility-scope.md`**

````markdown
# Compatibility Scope

**Last updated:** 2026-03-22
**Source docs consolidated:** `docs/MATTERMOST_CLIENTS.md`, `docs/mobile-compatibility-matrix.md`

---

## 1. Compatibility Commitment

rustchat commits to supporting Mattermost mobile (iOS/Android) and desktop clients as first-class external clients. Specifically:

- Mattermost Mobile v2.x clients can authenticate and use core functionality
- Mattermost Desktop clients can connect via the `/api/v4/` HTTP surface
- The Mattermost WebSocket protocol (`/api/v4/websocket`) is supported including auth challenge, session resumption, and standard event types

This is not a full Mattermost clone — some advanced features (plugins, advanced search, custom emoji upload) are not implemented.

---

## 2. Current Coverage

### Mobile-critical endpoints (primary metric)

**39/41 endpoints working (95.1%)** — as of 2026-03-17

| Category | Coverage |
|---|---|
| Authentication & Session | 6/6 ✅ |
| User Management | 5/5 ✅ |
| Teams | 4/4 ✅ |
| Channels | 6/6 ✅ |
| Posts & Messaging | 7/7 ✅ |
| Threads | 3/3 ✅ |
| Files | 3/3 ✅ |
| Preferences | 2/2 ✅ |
| WebSocket | 1/1 ✅ |
| System | 2/2 ✅ |
| **Total** | **39/41** |

**Known gaps (2/41):**

| Endpoint | Status | Impact | Planned |
|---|---|---|---|
| `POST /api/v4/emoji` | ❌ Not implemented | Medium — custom emoji upload unavailable | Phase 2 |
| `POST /api/v4/posts/search` | ❌ Not implemented | Medium — advanced search unavailable | Phase 2 |

For the full endpoint-by-endpoint table see `docs/mobile-compatibility-matrix.md`.

### Full v4 API surface (secondary metric)

The `docs/V4_API_GAP_ANALYSIS.md` measures coverage of the *entire* Mattermost v4 API surface — not just mobile-critical endpoints. That analysis reports approximately **60% coverage** of the full v4 surface. The gap is expected: rustchat does not aim to implement the full Mattermost v4 API, only the subset required for mobile and desktop client operation.

Do not conflate these two numbers. **39/41 (95.1%)** is the mobile compatibility headline. **~60%** is the broader full-API coverage figure.

---

## 3. Protected Surface

The following paths form the Mattermost compatibility surface. Changes here require compat-reviewer co-approval (`@zoorpha`) and are automatically elevated to the `elevated` risk tier per `.governance/protected-paths.yml`:

| Path | Purpose |
|---|---|
| `backend/src/api/v4/` | HTTP API v4 handlers (the compat surface clients call) |
| `backend/src/mattermost_compat/` | Response transformation, field mapping utilities |
| `backend/compat/` | Contract JSON schemas + contract validation tests |
| `backend/src/realtime/` | WebSocket hub (v4 event contracts) |

No changes to these paths without compat review. If unsure whether your change touches this surface, treat it as if it does.

---

## 4. Contract Tests

Contract tests live in `backend/compat/tests/`. They validate that rustchat's API responses match the expected Mattermost v4 JSON schemas.

```bash
# Run contract tests (requires live Postgres + Redis)
cd backend
RUSTCHAT_TEST_DATABASE_URL=postgres://... cargo test --test contract
```

Contract JSON schemas: `backend/compat/` — one schema per major entity type (user, channel, post, error).

When the compat surface changes, update the schema and regenerate the contract test. Do not merge compat surface changes without passing contract tests.

---

## 5. Gap Handling

New gaps discovered during development:
1. Open a GitHub issue using the `Bug Report` template, set `Compatibility impact: Yes — affects Mattermost clients`
2. Label will be applied as `area/compat` during triage
3. Implementation planned via the standard spec → plan → execution workflow with compat-reviewer co-approval

---

## 6. Connecting Mattermost Clients

**Server URL:** `http://your-host:8080` (Nginx proxy, recommended) or `http://your-host:3000` (direct backend)

| Port | Service | Notes |
|---|---|---|
| 8080 | Nginx proxy | Recommended — handles web UI and `/api/v4` |
| 3000 | Backend direct | Bypass Nginx if needed |

**Steps:**
1. In Mattermost Mobile or Desktop: Settings → Add Server
2. Enter the Server URL (port 8080)
3. Use email/password login (SSO configurable separately)
4. WebSocket connects automatically to `/api/v4/websocket`

**Known client limitations:**
- Push notifications require the push-proxy service to be running with valid FCM/APNS credentials
- Some advanced features (search, custom emoji, plugins) are not available
````

- [ ] **Step 2: Verify**

```bash
grep -c "^## " docs/compatibility-scope.md
```

Expected: `6`

- [ ] **Step 3: Replace `docs/MATTERMOST_CLIENTS.md` with a redirect stub**

Overwrite `docs/MATTERMOST_CLIENTS.md` entirely:

```markdown
> **Moved:** This document has been consolidated into [`docs/compatibility-scope.md`](compatibility-scope.md).
```

- [ ] **Step 4: Replace `docs/mobile-compatibility-matrix.md` with a redirect stub**

Overwrite `docs/mobile-compatibility-matrix.md` entirely:

```markdown
> **Moved:** The mobile compatibility matrix has been consolidated into [`docs/compatibility-scope.md`](compatibility-scope.md). The full endpoint table is in Section 2 of that document.
```

- [ ] **Step 5: Commit**

```bash
git add docs/compatibility-scope.md docs/MATTERMOST_CLIENTS.md docs/mobile-compatibility-matrix.md
git commit -m "docs: add compatibility-scope, stub MATTERMOST_CLIENTS.md and mobile-compatibility-matrix.md"
```

---

## Task 4: Testing model

**Files:**
- Create: `docs/testing-model.md`

- [ ] **Step 1: Write `docs/testing-model.md`**

````markdown
# Testing Model

**Last updated:** 2026-03-22

---

## Overview

rustchat uses four test layers. There are no formal unit tests — the backend test suite is entirely integration tests that require live infrastructure. The frontend has E2E tests only (no unit/component test framework is configured).

| Layer | Location | Scope | Infrastructure required |
|---|---|---|---|
| Backend integration | `backend/tests/` | HTTP API, services, DB queries | PostgreSQL + Redis + S3 |
| Compat contract | `backend/compat/tests/` | Mattermost v4 JSON schema compliance | PostgreSQL + Redis |
| Frontend E2E | `frontend/e2e/` | Playwright browser automation | Running full stack |
| Frontend build | CI only | TypeScript compilation + Vite build | None |

---

## 1. Backend Integration Tests

**Location:** `backend/tests/`
**Count:** ~170 tests (as of 2026-03-22)
**Framework:** Rust `tokio::test`

These tests start real HTTP servers against a real database and assert on full request/response cycles. They are not mocked.

**Required environment:**
```bash
export RUSTCHAT_TEST_DATABASE_URL=postgres://user:pass@localhost/rustchat_test
export REDIS_URL=redis://localhost:6379
export S3_ENDPOINT=http://localhost:9000
export S3_BUCKET=rustchat-test
```

**Run all integration tests:**
```bash
cd backend
cargo test --no-fail-fast
```

**Run a single test file:**
```bash
cd backend
cargo test --test channels_test
```

**Run with output:**
```bash
cd backend
cargo test -- --nocapture 2>&1 | head -100
```

**Note:** `cargo test --lib` runs only library unit tests (~125 as of README). The integration test suite under `backend/tests/` is separate and requires infrastructure.

---

## 2. Compat Contract Tests

**Location:** `backend/compat/tests/`
**Purpose:** Validate that rustchat's API responses conform to Mattermost v4 JSON schemas

Contract schemas: `backend/compat/` (JSON schema files per entity: user, channel, post, error, etc.)

**Run:**
```bash
cd backend
cargo test --test contract
```

Contract tests are **mandatory** before merging any change to the compat surface (`backend/src/api/v4/`, `backend/src/mattermost_compat/`, `backend/compat/`). See `docs/compatibility-scope.md` for details.

---

## 3. Frontend E2E Tests

**Location:** `frontend/e2e/`
**Framework:** Playwright (snapshot-based)

Playwright tests capture visual snapshots and assert against them. They run against a full running stack.

**Run:**
```bash
cd frontend
npx playwright test
```

**Update snapshots** (when intentional UI changes are made):
```bash
cd frontend
npx playwright test --update-snapshots
```

After updating snapshots, commit the new baseline files in `frontend/e2e/`.

---

## 4. Frontend Build Check

The frontend CI (`frontend-ci.yml`) runs `npm run build` as a build-time type check. There is no separate unit/component test framework configured.

```bash
cd frontend
npm run build
```

Expected: exits 0 with no TypeScript errors.

---

## 5. CI Gates

| Workflow | What it runs | Required to merge |
|---|---|---|
| `backend-ci.yml` | `cargo check`, `cargo clippy`, `cargo test --lib`, `cargo build` | Yes |
| `ci.yml` | `cargo fmt`, `cargo clippy --all-targets`, `cargo test`, `cargo build` | Yes |
| `compat.yml` | OpenAPI diff report against Mattermost v4 spec | Informational (not blocking) |
| `frontend-ci.yml` | `npm run build`, Playwright snapshot tests | Yes |
| `docker-publish.yml` | Multi-arch Docker build + push to GHCR | On tag push only |
| `release.yml` | Changelog generation + GitHub Release | On `v*` tag push only |

---

## 6. Test Requirements by Risk Tier

Per `.governance/risk-tiers.yml`:

| Tier | Tests required | Notes |
|---|---|---|
| `standard` | Recommended | Typos, UI polish, CI improvements |
| `elevated` | **Mandatory** | Auth, permissions, API behavior, schema, compat paths |
| `architectural` | **Mandatory** | Architecture changes, storage model, protocol changes |

For elevated changes to the compat surface: compat contract tests must pass. For auth changes: backend integration tests covering the auth paths must pass.

---

## 7. Running the Full Test Suite Locally

```bash
# 1. Start infrastructure (requires Docker)
docker compose up -d postgres redis minio

# 2. Backend integration tests
cd backend
RUSTCHAT_TEST_DATABASE_URL=postgres://rustchat:rustchat@localhost/rustchat_test \
REDIS_URL=redis://localhost:6379 \
cargo test --no-fail-fast

# 3. Frontend build check
cd frontend
npm run build

# 4. E2E tests (requires full stack running)
cd frontend
npx playwright test
```

For environment setup details see `docs/running_environment.md`.
````

- [ ] **Step 2: Verify**

```bash
grep -c "^## " docs/testing-model.md
```

Expected: `7`

- [ ] **Step 3: Commit**

```bash
git add docs/testing-model.md
git commit -m "docs: add testing-model"
```

---

## Task 5: Ownership map

**Files:**
- Create: `docs/ownership-map.md`

- [ ] **Step 1: Write `docs/ownership-map.md`**

````markdown
# Ownership Map

**Last updated:** 2026-03-22
**Source of truth:** [`.github/CODEOWNERS`](../.github/CODEOWNERS)

---

## Overview

Ownership determines who must review a PR before it can be merged. GitHub enforces this automatically when branch protection is enabled.

All paths not listed below are owned by `@senolcolak` (catchall).

---

## 1. Area Map

| Path | Human owner(s) | Agent authorized | Risk tier |
|---|---|---|---|
| `backend/src/` (non-protected) | `@senolcolak` | `backend-agent` | standard |
| `backend/src/auth/**` | `@senolcolak` | `backend-agent` ⚠️ explicit approval | elevated |
| `backend/src/api/v4/**` | `@senolcolak` + `@zoorpha` | `backend-agent` ⚠️ co-approval | elevated |
| `backend/src/mattermost_compat/**` | `@senolcolak` + `@zoorpha` | `backend-agent` ⚠️ co-approval | elevated |
| `backend/src/a2a/**` | `@senolcolak` | `backend-agent` ⚠️ senior review | elevated |
| `backend/src/realtime/**` | `@senolcolak` | `backend-agent` | elevated |
| `backend/migrations/**` | `@senolcolak` | `backend-agent` | elevated |
| `backend/tests/**` | `@senolcolak` | `backend-agent` | standard |
| `backend/compat/**` | `@senolcolak` + `@zoorpha` | `compat-agent` (read) | elevated |
| `push-proxy/**` | `@senolcolak` | `backend-agent` | standard |
| `frontend/**` | `@senolcolak` | `frontend-agent` | standard |
| `tools/mm-compat/**` | `@senolcolak` + `@zoorpha` | `compat-agent` (read) | elevated |
| `.github/**` | `@senolcolak` | none | standard |
| `.github/CODEOWNERS` | `@senolcolak` + `@zoorpha` | none | elevated |
| `.governance/**` | `@senolcolak` + `@zoorpha` | none | architectural |
| `docker/**` | `@senolcolak` | none | standard |
| `scripts/**` | `@senolcolak` | none | standard |
| `docs/**` | `@senolcolak` | `compat-agent` (specs/plans only) | standard |
| `AGENTS.md` | `@senolcolak` | none | standard |
| `*` (everything else) | `@senolcolak` | — | standard |

---

## 2. Dual-Owner Areas

The following areas require **both** owners to approve:

| Area | Owners | Why |
|---|---|---|
| `backend/compat/**` | `@senolcolak` + `@zoorpha` | Compat contracts must have compat-reviewer sign-off |
| `backend/src/api/v4/**` | `@senolcolak` + `@zoorpha` | Mattermost HTTP compat surface |
| `backend/src/mattermost_compat/**` | `@senolcolak` + `@zoorpha` | Compat utilities |
| `tools/mm-compat/**` | `@senolcolak` + `@zoorpha` | Compat analysis tooling |
| `.github/CODEOWNERS` | `@senolcolak` + `@zoorpha` | Ownership changes affect review routing |
| `.governance/**` | `@senolcolak` + `@zoorpha` | Architectural tier — 2 reviewers required |

---

## 3. Risk Tier Auto-Elevation

Changes to any path in `.governance/protected-paths.yml` are automatically elevated to the minimum tier listed. A PR touching `backend/src/api/v4/` is always `elevated` regardless of how trivial the change appears.

See `.governance/protected-paths.yml` for the full list.

---

## 4. Agent Boundaries

| Agent | Authorized areas | Prohibited |
|---|---|---|
| `backend-agent` | `backend/src/`, `backend/tests/`, `backend/migrations/`, `push-proxy/` | `frontend/`, `.governance/`, and protected sub-paths without approval |
| `frontend-agent` | `frontend/src/`, `frontend/e2e/` | `backend/`, `.governance/` |
| `compat-agent` | Read: compat surface. Write: `docs/superpowers/`, `previous-analyses/` | All production code paths |

For full contract details see `.governance/agent-contracts.yml` and `docs/agent-operating-model.md`.

---

## 5. How to Update Ownership

Changing CODEOWNERS is an `elevated` risk tier change (automatic per `protected-paths.yml`). It requires:

1. Edit `.github/CODEOWNERS`
2. Both `@senolcolak` and `@zoorpha` must approve the PR
3. Verify branch protection is still correct after the change

CODEOWNERS uses **last-match wins** semantics — the catchall `*` rule is placed first so specific rules below it take precedence.
````

- [ ] **Step 2: Verify**

```bash
grep -c "^## " docs/ownership-map.md
```

Expected: `5`

- [ ] **Step 3: Commit**

```bash
git add docs/ownership-map.md
git commit -m "docs: add ownership-map"
```

---

## Task 6: Repo current state

**Files:**
- Create: `docs/repo-current-state.md`

- [ ] **Step 1: Write `docs/repo-current-state.md`**

````markdown
# Repo Current State

**Last updated:** 2026-03-22
**Version:** v0.3.5

> This document describes the state of the repository as of its last update. For live issue tracking see GitHub Issues.

---

## 1. Version

**Current:** v0.3.5

Version is synchronized across two files:
- `backend/Cargo.toml` → `[package] version`
- `frontend/package.json` → `"version"`

To check: `grep '^version' backend/Cargo.toml` or `jq .version frontend/package.json`

Versioning follows semver. Releases are cut by pushing a `v*` tag — the `release.yml` workflow auto-generates a GitHub Release with changelog.

---

## 2. Services

| Service | Path | Port (default) | Status |
|---|---|---|---|
| Backend API | `backend/` | 3000 | ✅ Active |
| Frontend SPA | `frontend/` | 5173 (dev) / 8080 (via Nginx) | ✅ Active |
| Push Proxy | `push-proxy/` | 8065 | ✅ Active |
| Nginx proxy | `docker/nginx.conf` | 8080 | ✅ Active (Docker) |

**External services required:**
- PostgreSQL 16+ (port 5432)
- Redis 7+ (port 6379)
- S3-compatible storage (port 9000 for MinIO/RustFS)

For local setup see `docs/running_environment.md`.

---

## 3. Compatibility Status

- **Mobile-critical endpoints:** 39/41 (95.1%) — compatible with Mattermost Mobile v2.x
- **Last audited:** 2026-03-17
- **Gaps:** `POST /api/v4/emoji` (custom emoji upload), `POST /api/v4/posts/search` (advanced search) — both planned for Phase 2

For details see `docs/compatibility-scope.md`.

---

## 4. Known Gaps

| Gap | Area | Priority |
|---|---|---|
| Custom emoji upload (`POST /api/v4/emoji`) | compat | Phase 2 |
| Advanced search (`POST /api/v4/posts/search`) | compat | Phase 2 |
| No unit/component test framework for frontend | testing | Low |
| No Dependabot configured | dependencies | Low |
| Approach C (CI enforcement) not implemented | governance | Deferred |

---

## 5. Active Work Streams

For live in-flight work see [GitHub Issues](https://github.com/rustchatio/rustchat/issues).

**Recently completed:**

| Phase | Description | Date |
|---|---|---|
| Phase 1: Entity Foundation | Entity registration, API keys, rate limiting, WebSocket JWT expiry, mobile compat audit | 2026-03-17 |
| Governance Layer | `.governance/` policy files, CODEOWNERS, PR template, issue forms, branch protection, GitHub labels | 2026-03-22 |
| Foundation Docs | 7 structured docs consolidating existing flat docs | 2026-03-22 |

---

## 6. Quick Start

```bash
# Clone
git clone https://github.com/rustchatio/rustchat
cd rustchat

# Start infrastructure
docker compose up -d postgres redis minio

# Backend
cd backend
cargo run

# Frontend (separate terminal)
cd frontend
npm install
npm run dev
```

For full environment setup (env vars, config) see `docs/running_environment.md`.
````

- [ ] **Step 2: Verify**

```bash
grep -c "^## " docs/repo-current-state.md
```

Expected: `6`

- [ ] **Step 3: Commit**

```bash
git add docs/repo-current-state.md
git commit -m "docs: add repo-current-state"
```

---

## Task 7: Target operating model + ADR/decision-notes stubs + README update

**Files:**
- Create: `docs/target-operating-model.md`
- Create: `docs/adr/README.md`
- Create: `docs/decision-notes/README.md`
- Modify: `README.md`

- [ ] **Step 1: Write `docs/target-operating-model.md`**

````markdown
# Target Operating Model

**Last updated:** 2026-03-22
**Reference:** LLM Development Operating Model v2.0

---

## 1. Goal

rustchat aims to be a **production-ready, self-hosted team collaboration server** that:

1. Is **fully Mattermost-compatible** for mobile and desktop clients (41/41 mobile endpoints)
2. Is **safe to develop with LLM agents** — every area has clear ownership, risk classification, and scope boundaries
3. Has **high test confidence** — elevated and architectural changes require mandatory test coverage
4. Supports **horizontal scaling** — stateless API servers, Redis pub/sub, connection pooling

---

## 2. Operating Model Applied

The LLM Development Operating Model v2.0 is being applied to rustchat in phases:

| Phase | Description | Status |
|---|---|---|
| Governance layer | `.governance/` policy files, CODEOWNERS, PR template, issue forms, branch protection, GitHub labels | ✅ Complete (2026-03-22) |
| Foundation docs | 7 structured docs: architecture, agent model, compat scope, testing, ownership, repo state, target model | ✅ Complete (2026-03-22) |
| CI enforcement (Approach C) | Automated PR policy checks, agent boundary enforcement, label validation | 🔲 Deferred |
| Incident readiness | Revert policy, hotfix process, post-mortem template, dependency vulnerability SLA | 🔲 Deferred |
| Remaining foundation docs | `migration-roadmap.md`, `release-model.md`, `dependency-policy.md` | 🔲 Deferred |

---

## 3. Compatibility Target

**Current:** 39/41 mobile-critical endpoints (95.1%)

**Target:** 41/41 mobile endpoints + verified desktop client support

**Gap to close:**
- `POST /api/v4/emoji` — custom emoji upload (Phase 2)
- `POST /api/v4/posts/search` — advanced search (Phase 2)

---

## 4. Agent Workflow Target

All agent-driven development follows:
- Spec → plan → subagent execution → two-stage review (spec compliance + code quality) → PR
- No direct commits to protected paths without prior spec
- compat-agent produces analysis artifacts only; production code written by backend-agent with compat-reviewer co-approval

This is enforced by convention today (`.governance/agent-contracts.yml`). Approach C will add automated CI enforcement.

---

## 5. Deferred Items

The following are explicitly out of scope until a future phase:

| Item | Reason deferred |
|---|---|
| Approach C (CI enforcement workflows) | Requires stable policy files first; enforcement added after contracts prove out |
| `docs/migration-roadmap.md` | Requires Phase 2 scope to be defined first |
| `docs/release-model.md` | Current release process is documented in `bump-version.sh`; formal doc deferred |
| `docs/dependency-policy.md` | No Dependabot yet; doc deferred until automation is in place |
| Incident readiness docs | Low priority until production deployment |
````

- [ ] **Step 2: Create `docs/adr/README.md`**

```bash
mkdir -p docs/adr
```

Write `docs/adr/README.md`:

```markdown
# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records for rustchat.

An ADR documents a significant architectural decision: the context, the options considered, the decision made, and the consequences.

## When to create an ADR

Per `.governance/risk-tiers.yml`, an ADR is required for `architectural` tier changes, and for specific elevated changes involving:
- Auth or permission model changes
- API contract changes
- Storage model changes

## Format

```
# ADR-NNN: Title

**Date:** YYYY-MM-DD
**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-NNN
**Risk tier:** architectural

## Context
[What is the problem and why does it matter?]

## Decision
[What was decided?]

## Consequences
[What are the trade-offs and implications?]
```

## Index

_(No ADRs yet. Add entries here as ADRs are created.)_
```

- [ ] **Step 3: Create `docs/decision-notes/README.md`**

```bash
mkdir -p docs/decision-notes
```

Write `docs/decision-notes/README.md`:

```markdown
# Decision Notes

This directory contains decision notes for elevated-tier changes.

A decision note is a short explanation (3–5 lines) of what was decided and why — required for any PR marked as `elevated` or `architectural` risk tier. Unlike ADRs (which are for architectural-scale decisions), decision notes cover routine elevated changes such as auth tweaks, migration strategies, or compat surface changes.

## When to create a decision note

Required when a PR's risk tier is `elevated` or `architectural` and the change is not covered by an existing ADR.

## Format

File naming: `YYYY-MM-DD-<short-description>.md`

```
# Decision Note: [Short title]

**Date:** YYYY-MM-DD
**PR:** #NNN
**Risk tier:** elevated | architectural

## What was decided

[2–4 sentences: what the decision is]

## Why

[2–4 sentences: context, constraints, alternatives considered]
```

## Index

_(No decision notes yet. Add entries here as notes are created.)_
```

- [ ] **Step 4: Update `README.md` with documentation table**

README.md has an existing `## Key Documentation` section (near the bottom) that links to flat files being stubbed. Replace that entire section with the new structured `## Documentation` table.

Find this section in README.md:
```
## Key Documentation

- [Architecture](docs/architecture.md)
- [WebSocket Architecture](docs/websocket_architecture.md)
...
```

Replace the entire `## Key Documentation` section (from `## Key Documentation` through the last bullet before `## License`) with:

```markdown
## Documentation

| Document | Purpose |
|---|---|
| [Architecture Overview](docs/architecture/architecture-overview.md) | System components, data flow, key design decisions |
| [Agent Operating Model](docs/agent-operating-model.md) | LLM agent roles, skills, workflow, scope boundaries |
| [Compatibility Scope](docs/compatibility-scope.md) | Mattermost client compatibility commitments and coverage |
| [Testing Model](docs/testing-model.md) | Test layers, CI gates, requirements by risk tier |
| [Ownership Map](docs/ownership-map.md) | Code ownership, review routing, agent boundaries |
| [Repo Current State](docs/repo-current-state.md) | Current version, services, known gaps |
| [Target Operating Model](docs/target-operating-model.md) | Goals, applied operating model, deferred items |
```

- [ ] **Step 5: Verify all new files exist**

```bash
ls docs/architecture/architecture-overview.md \
   docs/agent-operating-model.md \
   docs/compatibility-scope.md \
   docs/testing-model.md \
   docs/ownership-map.md \
   docs/repo-current-state.md \
   docs/target-operating-model.md \
   docs/adr/README.md \
   docs/decision-notes/README.md
```

Expected: 9 lines, all file paths printed without error.

- [ ] **Step 6: Verify redirect stubs exist**

```bash
head -1 docs/architecture.md docs/websocket_architecture.md docs/MATTERMOST_CLIENTS.md docs/mobile-compatibility-matrix.md
```

Expected: each file starts with `> **Moved:**`

- [ ] **Step 7: Verify README documentation table was added**

```bash
grep "Architecture Overview" README.md
```

Expected: one line matching.

- [ ] **Step 8: Commit**

```bash
git add docs/target-operating-model.md docs/adr/ docs/decision-notes/ README.md
git commit -m "docs: add target-operating-model, ADR/decision-notes stubs, documentation table in README"
```

---

## Post-implementation verification

- [ ] **Verify all 9 new structured files exist:** `ls docs/architecture/architecture-overview.md docs/agent-operating-model.md docs/compatibility-scope.md docs/testing-model.md docs/ownership-map.md docs/repo-current-state.md docs/target-operating-model.md docs/adr/README.md docs/decision-notes/README.md`
- [ ] **Verify 4 redirect stubs:** `head -1 docs/architecture.md docs/websocket_architecture.md docs/MATTERMOST_CLIENTS.md docs/mobile-compatibility-matrix.md` — each should start with `> **Moved:**`
- [ ] **Verify README table:** `grep "Architecture Overview" README.md` — should match
- [ ] **Verify no production code was modified:** `git diff HEAD~7 --name-only -- backend/ frontend/ push-proxy/` — expected: empty
- [ ] **Verify no CI workflows were modified:** `git diff HEAD~7 --name-only -- .github/workflows/` — expected: empty
- [ ] **Final commit log:** `git log --oneline -8`
