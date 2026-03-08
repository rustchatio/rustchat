# Repository Guidelines

This document defines how AI agents and contributors should work in `rustchat`.

## Project Overview

Rustchat is a self-hosted collaboration platform with:
- Rust backend (`backend/`) using Axum + Tokio + SQLx
- Vue 3 + TypeScript frontend (`frontend/`)
- Push notification proxy service (`push-proxy/`)
- Compatibility layers for external clients (`/api/v4`, websocket, calls plugin routes)

## Communication Rules

- Respond to users in the same language they use.
- Keep code, comments, docs, commit messages, and PR text in English.
- Keep changes scoped to the request; do not bundle unrelated refactors.

## Project Mission

We are building `rustchat`, a high-performance, security-hardened Rust backend compatible with the Mattermost API.

## Tech Stack Constraints

- Prefer idiomatic asynchronous Rust using Tokio + Axum patterns.
- Use PostgreSQL as the primary persistent data store.
- Treat Meilisearch as the preferred indexing/search engine when building dedicated retrieval workflows.

## Universal Coding Rules

- All public functions must return `Result` or `Option`.
- Preserve strict compatibility with Mattermost API response signatures for compatibility surfaces.
- Run `cargo clippy` and `cargo test` before concluding a backend task.

## Progressive Disclosure for Agents

Use a three-tier documentation model to reduce context waste and improve decision quality:

1. Tier 1 (`AGENTS.md`): global repository policy and non-negotiable rules.
2. Tier 2 (`.agents/skills/*/SKILL.md`): specialized, trigger-based workflows and constraints.
3. Tier 3 (task-local files): working memory artifacts such as `SPEC.md` and `task_plan.md`.

Agents must load the minimum tier needed first, and only pull deeper context when required by task complexity.

## Plan-First Workflow (Mandatory)

For feature or behavior changes, use this gate before implementation:

1. Requirement phase:
   - Research the current code and compatibility context.
   - Draft `SPEC.md` at repository root with scope, contract impact, and verification approach.
2. Validation gate:
   - Pause and ask: `Does this plan meet your expectations? Please approve or provide feedback.`
   - Do not write implementation code until explicit user approval is received.
3. Verification phase:
   - Update `task_plan.md` at repository root with implementation and test status.
   - Mark task readiness for testing.
   - Provide at least one concrete manual verification command (for example, `curl`) for user acceptance.

## Compatibility-First Workflow (Mandatory, Strict)

Use this workflow for any change that can affect external client compatibility (API v4 routes, websocket events, pagination, error semantics, calls behavior).

### Phase 1: Upstream analysis before coding

1. Analyze upstream behavior first (read-only):
   - `../mattermost`
   - `../mattermost-mobile`
2. Create a new analysis folder: `previous-analyses/YYYY-MM-DD-<topic>/`
3. Start from templates in `previous-analyses/_TEMPLATE/`:
   - `SUMMARY.md`
   - `SERVER_FINDINGS.md`
   - `MOBILE_FINDINGS.md`
   - `GAP_PLAN.md`
4. Add mandatory analysis artifacts:
   - `ENDPOINT_MATRIX.md`
   - `ARCHITECTURE_GAPS.md`
   - `UX_JOURNEYS.md`
   - `PRODUCTION_READINESS_SCORECARD.md`
   - `GAP_REGISTER.md`

### Phase 2: Contract capture (no implementation yet)

Document exact contract details before coding:
- HTTP status codes and response shape
- Required headers and error payload semantics
- Websocket event names and payload fields
- Pagination, ordering, and filtering defaults
- Auth and permission edge-case behavior

### Phase 3: Implementation only after explicit gaps

Implement only after every gap is explicit and testable in `GAP_REGISTER.md`.

Rules:
- Never edit upstream reference repositories.
- Do not treat "close enough" behavior as compatible when clients depend on exact wire contracts.
- Every compatibility claim must include concrete evidence path(s) with line numbers.

## Mandatory Evidence Schema

Every `GAP_REGISTER.md` row must include:
- `id`
- `domain`
- `surface`
- `severity` (`P0`/`P1`/`P2`/`P3`)
- `user_impact`
- `evidence`
- `current_behavior`
- `expected_behavior`
- `fix_strategy`
- `test_strategy`
- `owner_suggestion`

## Mobile-First Constraints

For any contract-affecting change:
- Treat Mattermost mobile journeys as first-class release gates.
- Validate at minimum: login/session, team/channel discovery, posting/threading/reactions, notifications/read state, file flows, calls entry/join/leave.
- Do not merge contract changes without mobile journey verification evidence.

## Production Readiness Policy

Use this policy when claiming production readiness or near-parity status.

Release gate thresholds:
- `P0` gaps: must be `0`.
- `P1` gaps: must be `0`.
- `P2` gaps: allowed only if explicitly documented in compatibility profile and approved.

Mandatory green checks before production-ready claim:
- backend checks/tests in intended deployment profile
- frontend production build
- compatibility smoke checks:
  - `./scripts/mm_compat_smoke.sh`
  - `./scripts/mm_mobile_smoke.sh`
- endpoint parity diff review from `ENDPOINT_MATRIX.md`

If any mandatory gate fails, status must be reported as "not production ready".

## Pull Request Acceptance Checklist (Compatibility Changes)

PR description must include all of the following sections.

### 1) What changed
- concise summary of behavior changes

### 2) Compatibility impact
- `Gap IDs`: list of affected `G-xxx` rows
- `Contract surfaces`: endpoints/events/pagination/errors touched
- `Upstream evidence`: file paths with line numbers

### 3) Verification
- exact commands run
- expected vs actual result
- links/paths to updated analysis artifacts

### 4) Risks and follow-ups
- residual incompatibilities
- explicit non-goals/deferred items

A compatibility-sensitive PR is not review-ready without all four sections.

## Repository Layout

- `backend/`: API server, websocket, persistence, compatibility endpoints
- `frontend/`: web client (feature-based architecture under `src/features`)
- `push-proxy/`: mobile push notification service
- `scripts/`: smoke tests and operational helpers
- `tools/mm-compat/`: compatibility extraction/report tooling
- `previous-analyses/`: compatibility research history
- `docker-compose.yml`: local multi-service development stack

## Build, Test, and Verification Commands

## Backend

```bash
cd backend
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test --no-fail-fast -- --nocapture
```

If tests require local dependencies, start them first:

```bash
docker compose up -d postgres redis rustfs
```

For deterministic integration tests that do not depend on local `5432/6379` state:

```bash
docker compose -f docker-compose.integration.yml up -d
export RUSTCHAT_TEST_DATABASE_URL=postgres://rustchat:rustchat@127.0.0.1:55432/rustchat
export RUSTCHAT_TEST_REDIS_URL=redis://127.0.0.1:56379/
export RUSTCHAT_TEST_S3_ENDPOINT=http://127.0.0.1:59000
export RUSTCHAT_TEST_S3_ACCESS_KEY=minioadmin
export RUSTCHAT_TEST_S3_SECRET_KEY=minioadmin
```

## Frontend

```bash
cd frontend
npm ci
npm run build
```

Optional end-to-end tests:

```bash
npm run test:e2e
```

## Compatibility smoke checks

```bash
BASE=http://127.0.0.1:3000 ./scripts/mm_compat_smoke.sh
BASE=http://127.0.0.1:3000 ./scripts/mm_mobile_smoke.sh
```

`BASE` is mandatory. Scripts must fail fast when target does not expose `X-MM-COMPAT: 1`.

## Full stack local run

```bash
cp .env.example .env
docker compose up -d --build
```

## Coding Standards

### Rust backend
- Prefer explicit `Result`-based error handling; avoid panics in production paths.
- Keep handlers thin; place logic in services/repositories.
- Preserve existing API contract shapes unless the task explicitly changes them.

### Frontend
- Keep feature boundaries intact (`core/` vs `features/`).
- Use typed API/client models; avoid introducing untyped payload handling.
- Maintain compatibility-sensitive event names and payload formats.

### Command Invocation Policy (Permanent)
- Rustchat uses command invocation via `Ctrl/Cmd+K` (desktop) and `^k` token (mobile/typed input).
- Do not introduce `/`-triggered slash command UX as the primary command entry path.
- Command discovery UX must be command-menu based and keyboard-first.
- If backward compatibility paths exist, keep them internal-only and never present `/` as the user-facing standard.

## Testing Expectations

- Every bug fix should include or update a regression test when feasible.
- For compatibility-sensitive changes, verify:
  - status codes
  - response/error schema
  - websocket events
  - pagination and ordering behavior
- If automated coverage is not possible, include explicit manual verification steps in the PR.

## Security and Configuration

- Never commit secrets or credential files.
- Required local secrets are enforced in compose:
  - `RUSTCHAT_JWT_SECRET`
  - `RUSTCHAT_ENCRYPTION_KEY`
- For production-like runs:
  - set `RUSTCHAT_ENVIRONMENT=production`
  - set `RUSTCHAT_CORS_ALLOWED_ORIGINS` explicitly

## Commit and Pull Request Guidelines

- Use Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, etc.).
- Keep commit scope focused and reviewable.
- Before opening PR, run backend + frontend verification commands listed above.
- In PR description, list:
  - what changed
  - how it was verified
  - compatibility impact (if any)

## Skill Usage Policy (Mandatory)

Use the following skills based on task category:

- `mattermost-analysis-first`:
  - required before any compatibility-sensitive implementation
- `mattermost-mobile-compatibility`:
  - required before merge decisions for mobile-consumed behavior
- `mm-endpoint-contract-parity`:
  - required for API route/status/schema contract work
- `mm-websocket-calls-parity`:
  - required for websocket/calls events, payloads, reconnect semantics
- `mm-mobile-journey-parity`:
  - required for end-to-end mobile journey validation
- `production-readiness-gate`:
  - required when evaluating release readiness or parity claims
- `mattermost-api-parity`:
  - required for API error envelope, error ID/message semantics, and parameter pattern parity
- `ai-summarization-rag`:
  - required for retrieval-augmented summarization features and thread-summary pipelines
- `user-validation`:
  - required for plan-first user-in-the-loop delivery (`SPEC.md` approval gate + `task_plan.md` verification tracking)
- `guidelines`:
  - required as a final quality/scope sanity check

Trigger composition guidance:
- `user-validation` governs delivery process and approval gates; it does not replace compatibility skills.
- For compatibility-sensitive API routes, combine `mattermost-analysis-first` + `mattermost-api-parity` + `mm-endpoint-contract-parity`.
- Use `ai-summarization-rag` only when the requested change includes summarization or retrieval behavior.

## Important Reminders

- Do exactly what was requested: no speculative features.
- Prefer editing existing files over creating new ones.
- If you see unrelated issues, note them separately; do not silently fix them in the same change.
