# Testing Model

**Last updated:** 2026-03-22

---

## Overview

rustchat uses four test layers. There are no formal unit tests — the backend test suite is entirely integration tests that require live infrastructure. The frontend has E2E tests only (no unit/component test framework is configured).

| Layer | Location | Scope | Infrastructure required |
|---|---|---|---|
| Backend integration | `backend/tests/` | HTTP API, services, DB queries | PostgreSQL + Redis + S3 |
| Compat contract | `backend/compat/tests/` | Mattermost v4 response shape validation utilities | Called from integration tests |
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

**Note:** `cargo test --lib` runs only library unit tests. The integration test suite under `backend/tests/` is separate and requires infrastructure.

---

## 2. Compat Contract Validation

**Location:** `backend/compat/tests/contract_validation.rs`
**Purpose:** Validation utility functions that check Mattermost v4 JSON response shapes

This file contains helper functions (e.g., `validate_user_response`, `validate_channel_response`) that validate response field presence and format. They are **not** runnable Cargo test binaries — they are utility functions intended to be called from integration tests.

**The actual compat CI check is the `compat.yml` workflow**, which runs an OpenAPI diff between rustchat's API spec and the Mattermost v4 spec:

```bash
# compat.yml runs this automatically on PRs
# To run the OpenAPI diff locally, see tools/mm-compat/
```

Contract schemas: `backend/compat/contracts/` — JSON/YAML schema definitions per entity type.

When the compat surface changes, check whether the OpenAPI diff in `compat.yml` still passes. Do not merge compat surface changes without reviewing the compat.yml output. See `docs/compatibility-scope.md` for the full compat surface definition.

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
