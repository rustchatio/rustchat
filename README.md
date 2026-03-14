# rustchat

Self-hosted team collaboration platform with a Rust backend and a Solid.js web client.

rustchat targets two audiences:
- **Contributing developers** who want to build a Mattermost-compatible server in Rust.
- **Self-hosting operators** who want to run their own collaboration stack.

## Scope and Honesty Policy

This README is intentionally explicit about:
- what is implemented,
- what is partial,
- what is not implemented,
- what was actually verified in this workspace.

If a capability is uncertain, it is marked as partial or unverified.

## Current Status (as of 2026-03-07)

Project maturity:
- **Active development / pre-release**.
- **No production-ready claim** is made here.

Verification snapshot executed in this workspace:
- `cd backend && cargo check` -> **PASS**
- `cd frontend-solid && npm run build` -> **PASS**
- `cd backend && cargo test --no-fail-fast -- --nocapture` -> **PARTIAL/FAIL** (unit tests passed; many integration tests failed due missing DB bootstrap/test env)
- `BASE=http://127.0.0.1:3000 ./scripts/mm_compat_smoke.sh` -> **FAIL** (target not running)
- `BASE=http://127.0.0.1:3000 ./scripts/mm_mobile_smoke.sh` -> **FAIL** (target not running / no compatibility preflight)

## What rustchat Does

### Core platform
- Rust backend (`Axum + Tokio + SQLx`) under [`backend/`](backend/).
- Web app (`Solid.js 1.9 + TypeScript`) under [`frontend-solid/`](frontend-solid/).
- Push notification proxy service under [`push-proxy/`](push-proxy/).
- PostgreSQL + Redis + S3-compatible object storage integration.

### API surfaces
- Native API surface under `/api/v1` for first-party web features.
- Mattermost compatibility surface under `/api/v4` with broad route coverage.
- v4 compatibility behavior includes:
  - `X-MM-COMPAT: 1` response header on v4 routes.
  - Explicit `501 Not Implemented` fallback for unsupported v4 routes.

Evidence:
- [`backend/src/api/v4/mod.rs`](backend/src/api/v4/mod.rs)
- [`scripts/mm_compat_smoke.sh`](scripts/mm_compat_smoke.sh)
- [`scripts/mm_mobile_smoke.sh`](scripts/mm_mobile_smoke.sh)

### Real-time and calls
- WebSocket endpoint for Mattermost-style clients at `/api/v4/websocket`.
- Separate legacy/first-party websocket surface exists (`/api/v1/ws`).
- Calls plugin route surface under `/api/v4/plugins/com.mattermost.calls/*`.
- Calls state backends:
  - `memory` (single-node)
  - `redis` (shared control-plane state)
  - `auto` (Redis-first with fallback)

Evidence:
- [`backend/src/api/v4/websocket.rs`](backend/src/api/v4/websocket.rs)
- [`backend/src/api/ws.rs`](backend/src/api/ws.rs)
- [`backend/src/api/v4/calls_plugin/mod.rs`](backend/src/api/v4/calls_plugin/mod.rs)
- [`docs/calls_deployment_modes.md`](docs/calls_deployment_modes.md)

### Operations and security posture
- Production-mode validation enforces stricter security constraints (JWT issuer/audience, HTTPS requirements, restricted legacy token transport).
- Environment-based CORS behavior (development vs production).

Evidence:
- [`backend/src/config/mod.rs`](backend/src/config/mod.rs)
- [`backend/src/api/mod.rs`](backend/src/api/mod.rs)

## What rustchat Cannot (or Does Not Yet) Do Completely

### Not fully implemented v4 areas (explicit or effective)
- Some v4 modules intentionally return `501` for selected endpoints (examples: parts of plugins, dialogs, custom profile, selected system/calls plugin-management paths).
- Several v4 domains expose placeholder-style responses for now (notably parts of OAuth app/outgoing connection management and some command/bot mutation paths).

Evidence:
- [`backend/src/api/v4/plugins.rs`](backend/src/api/v4/plugins.rs)
- [`backend/src/api/v4/dialogs.rs`](backend/src/api/v4/dialogs.rs)
- [`backend/src/api/v4/custom_profile.rs`](backend/src/api/v4/custom_profile.rs)
- [`backend/src/api/v4/oauth.rs`](backend/src/api/v4/oauth.rs)
- [`backend/src/api/v4/commands.rs`](backend/src/api/v4/commands.rs)
- [`backend/src/api/v4/bots.rs`](backend/src/api/v4/bots.rs)

### Calls architecture limits
- Multi-node control-plane call state is available in Redis mode.
- SFU media plane is still instance-local (no fully distributed media fabric claim).

Evidence:
- [`docs/calls_deployment_modes.md`](docs/calls_deployment_modes.md)

### Verification limits right now
- Full integration test confidence requires a correctly bootstrapped test DB/Redis/S3 test environment.
- Compatibility smoke checks require a live running backend exposing `/api/v4`.

## What rustchat Does Differently

Compared with typical Mattermost-compatible deployments, rustchat explicitly differs in these areas:

1. **Rust-first server implementation**
- The backend is implemented in Rust rather than Go.

2. **Dual API strategy**
- Maintains native `/api/v1` plus compatibility `/api/v4` in the same server.

3. **Compatibility signaling discipline**
- v4 explicitly advertises compatibility with `X-MM-COMPAT: 1` and uses explicit `501` fallback for unsupported routes.

4. **Command invocation policy in product UX**
- Primary command invocation is keyboard-first (`Ctrl/Cmd+K` on desktop, `^k` token in composer/mobile-typed input).
- Slash-command-first UX is intentionally not the primary entry path.

Evidence:
- [`backend/src/api/v4/mod.rs`](backend/src/api/v4/mod.rs)
- [`frontend-solid/src/components/layout/Header.tsx`](frontend-solid/src/components/layout/Header.tsx)
- [`AGENTS.md`](AGENTS.md)

## Target Audience Guidance

### For self-host operators
Use rustchat if you want:
- self-hosted collaboration infrastructure,
- Rust backend stack,
- gradual Mattermost client compatibility.

Do not treat this repository as production-ready by default unless your own deployment gates are green (tests, smoke checks, security hardening, operational monitoring).

### For contributing developers
You will work on:
- strict API contract behavior for compatibility-sensitive endpoints,
- websocket and calls parity details,
- incremental replacement of partial/stubbed routes,
- CI/test hardening for confidence.

## Quick Start (Operator)

### Prerequisites
- Docker + Docker Compose
- `.env` file with required secrets

### 1) Configure
```bash
cp .env.example .env
```
Set at minimum:
- `RUSTCHAT_JWT_SECRET`
- `RUSTCHAT_JWT_ISSUER`
- `RUSTCHAT_JWT_AUDIENCE`
- `RUSTCHAT_ENCRYPTION_KEY`
- `RUSTCHAT_S3_ACCESS_KEY`
- `RUSTCHAT_S3_SECRET_KEY`
- `RUSTFS_ACCESS_KEY`
- `RUSTFS_SECRET_KEY`

### 2) Run stack
```bash
docker compose up -d --build
```

Default endpoints:
- Web UI: `http://localhost:8080`
- Backend: `http://localhost:3000`
- Postgres: `localhost:5432`
- Redis: `localhost:6379`
- S3 API (RustFS): `localhost:9000`

### 3) Compatibility smoke checks
```bash
BASE=http://127.0.0.1:3000 ./scripts/mm_compat_smoke.sh
BASE=http://127.0.0.1:3000 ./scripts/mm_mobile_smoke.sh
```

## Local Development (Contributor)

### Backend
```bash
cd backend
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test --no-fail-fast -- --nocapture
```

If integration tests need dependencies:
```bash
docker compose up -d postgres redis rustfs
```

Or deterministic integration profile:
```bash
docker compose -f docker-compose.integration.yml up -d
export RUSTCHAT_TEST_DATABASE_URL=postgres://rustchat:rustchat@127.0.0.1:55432/rustchat
export RUSTCHAT_TEST_REDIS_URL=redis://127.0.0.1:56379/
export RUSTCHAT_TEST_S3_ENDPOINT=http://127.0.0.1:59000
export RUSTCHAT_TEST_S3_ACCESS_KEY=minioadmin
export RUSTCHAT_TEST_S3_SECRET_KEY=minioadmin
```

### Frontend
```bash
cd frontend-solid
npm ci
npm run build
```

Optional E2E:
```bash
npm run test:e2e
```

## Repository Map

```text
rustchat/
├── backend/            Rust API server (v1 + v4 + websocket + calls)
├── frontend-solid/     Solid.js web client
├── push-proxy/         Push notification proxy
├── scripts/            Compatibility and operational smoke scripts
├── tools/mm-compat/    Compatibility extraction/report tooling
├── docs/               Operator/developer architecture and runbooks
└── previous-analyses/  Historical compatibility analysis artifacts
```

## Key Documentation

- [Architecture](docs/architecture.md)
- [WebSocket Architecture](docs/websocket_architecture.md)
- [Calls Deployment Modes](docs/calls_deployment_modes.md)
- [Running Environment](docs/running_environment.md)
- [Operations Runbook](docs/operations-runbook.md)
- [Security Deployment Guide](docs/security-deployment-guide.md)
- [Admin Guide](docs/admin_guide.md)
- [User Guide](docs/user_guide.md)
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

## License

MIT - see [LICENSE](LICENSE).
