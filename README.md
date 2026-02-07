# rustchat

Self-hosted team collaboration platform written in Rust, with a native web UI and a Mattermost-compatible API layer.

[![Rust](https://img.shields.io/badge/rust-1.92+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Project Status

Status: **active development / pre-release** (not production-hardened yet).

Snapshot date: **2026-02-07**

Verification signals from this repo:
- Backend compiles: `cargo check` passes (with many warnings).
- Frontend builds: `npm run build` passes.
- Test health is not green yet:
  - `cargo test` fails to compile several integration tests because `api::router` call sites were not updated after adding a new `Config` argument.
  - `cargo test --lib` runs, but 5 MiroTalk-related tests fail on `system-configuration` runtime panic; 20 lib tests pass.

## Implemented Functions

RustChat currently includes these major function groups.

### Core collaboration
- User registration/login and JWT auth.
- Teams and channels (public/private/direct/group), membership management.
- Posts/messages with edit/delete, reactions, pin/unpin, save/unsave.
- Threads and thread-following routes.
- Unread tracking and mark-read flows.
- Real-time messaging via WebSocket.
- File upload/download/preview and metadata endpoints.
- Message search (PostgreSQL full-text queries).

### Admin and operations
- Admin dashboard APIs for config, users, teams, permissions, health, and audit views.
- Server config persistence in DB (`server_config` table family).
- Retention background job.
- Health probes (`/api/v1/health/live`, `/api/v1/health/ready`).

### Integrations and automation
- Incoming/outgoing webhooks.
- Slash commands.
- Bot accounts and bot tokens.
- OAuth provider integration endpoints.
- Playbooks/checklists/runs APIs.
- Video meeting integration path for MiroTalk.

### Calls
- Mattermost Calls plugin route namespace under `/api/v4/plugins/com.mattermost.calls/*`.
- Call lifecycle APIs (start/join/leave/state, mute/unmute, raise/lower hand, react, screenshare flags).
- TURN/STUN config exposure for clients.
- SFU/WebRTC signaling path (`offer`, `ice`) with ICE candidate handling and websocket signaling events.
- Redis-backed call state backend with `memory|redis|auto` mode selection.

## Compatibility

Compatibility claims in this section were last verified on **2026-02-07** against:
- `backend/src/api/v4/mod.rs`
- `backend/src/api/v4/system.rs`
- `backend/src/api/v4/plugins.rs`
- `backend/src/api/v4/websocket.rs`
- `docs/mattermost-compat.md`
- `docs/mattermost-v4-comparison.md`

### Client compatibility matrix

| Client/API | Status | Notes |
|---|---|---|
| RustChat Web UI (Vue 3) | Working baseline | Main first-party client. |
| RustChat API v1 (`/api/v1`) | Working baseline | Native API used by web app and admin console. |
| Mattermost API v4 (`/api/v4`) | Partial compatibility | Broad endpoint surface exists; parity varies by endpoint depth. |
| Mattermost Mobile/Desktop clients | Partial compatibility | Login, teams/channels, posts, websocket, and calls plugin handshake paths are present; some features are stubbed or simplified. |
| Mattermost Calls plugin API | Partial compatibility | Signaling and distributed call state baseline are implemented; media forwarding/scaling is still evolving. |

### Compatibility behavior notes
- `/api/v4/*` responses include `X-MM-COMPAT: 1`.
- v4 router has a fallback that returns `501 Not Implemented` for unmatched routes.
- Mattermost compatibility version constant is currently `10.11.10`.
- Plugin mutation and interactive dialogs routes are explicit `501` (not silent stubs).
- Several enterprise-specific paths intentionally return `501` (e.g., many LDAP/SAML operations).

## Current Design Problems

The following are known technical/design gaps in the current codebase:

1. API surface vs semantic completeness gap
- Many v4 routes exist, but a subset return placeholder/hardcoded responses or stub success payloads (for example in cluster/compliance/terms/roles and other enterprise modules).

2. Calls architecture not fully production-ready
- SFU media forwarding/scaling remains a baseline implementation.

3. Dual websocket stacks increase complexity
- There is a v1 websocket endpoint and a separate Mattermost-style v4 websocket implementation with overlapping responsibilities.

4. Test suite drift
- Integration tests are out of sync with current router signature.
- Some lib tests are environment-sensitive and currently fail in this environment.

5. Security and operations defaults need tightening
- Global permissive CORS (`allow_origin(Any)` and broad methods/headers).
- Default TURN credentials in compose are development-friendly but should not be treated as secure defaults.

6. Maintainability debt
- Large monolithic API files (notably v4 users/channels/posts modules) and a high warning count make evolution harder.

## Improvement Priorities

1. Make tests green and enforced
- Fix integration test compile breaks.
- Stabilize or isolate environment-dependent tests.
- Add CI gates for `cargo check`, `cargo test`, frontend build, and compatibility smoke tests.

2. Harden Mattermost compatibility claims
- Convert placeholder endpoints either to real behavior or explicit documented `501`.
- Add an automated compatibility matrix generated from tests, not manual docs.

3. Complete calls media architecture
- Extend SFU media quality and scaling controls for high-participant calls.
- Harden distributed SFU orchestration across instances.

4. Tighten security posture
- Restrict CORS by environment.
- Remove insecure development defaults from production examples.
- Review auth/session handling paths (especially OAuth callback/token flow).

5. Improve structure and docs
- Split oversized route modules.
- Keep compatibility docs and env examples aligned with code and compose updates.

## Quick Start (Dev)

### Prerequisites
- Rust `1.92+`
- Node.js `24+` (frontend `package.json` engine)
- Docker and Docker Compose

### Run with Docker Compose
```bash
cp .env.example .env
docker compose up -d --build
```

Important:
- `docker-compose.yml` no longer ships fallback JWT/encryption secrets.
- Set `RUSTCHAT_JWT_SECRET` and `RUSTCHAT_ENCRYPTION_KEY` in `.env` before startup.
- For production, set `RUSTCHAT_ENVIRONMENT=production` and define `RUSTCHAT_CORS_ALLOWED_ORIGINS`.

Default services:
- Web UI / reverse proxy: `http://localhost:8080`
- Backend API: `http://localhost:3000`
- Postgres: `localhost:5432`
- Redis: `localhost:6379`
- S3-compatible storage (RustFS): `localhost:9000`

### Local backend + frontend
```bash
# Terminal 1
cd backend
cargo run

# Terminal 2
cd frontend
npm run dev
```

## Repository Layout

```text
rustchat/
├── backend/        # Rust backend (API, realtime, DB, storage, compat layer)
├── frontend/       # Vue 3 frontend
├── docs/           # Project docs
├── docker/         # Dockerfiles and Nginx config
├── scripts/        # Smoke/build helper scripts
└── tools/          # Compatibility tooling and reports
```

## Calls Deployment Modes

See `docs/calls_deployment_modes.md` for single-node vs multi-node behavior, fallback semantics, and current limits.

## Security Mode Notes

- `RUSTCHAT_ENVIRONMENT=development` keeps developer-friendly CORS defaults.
- `RUSTCHAT_ENVIRONMENT=production` makes CORS restricted by default unless `RUSTCHAT_CORS_ALLOWED_ORIGINS` is configured.
- OAuth login now enforces Redis-backed CSRF `state` validation and decrypts stored SSO client secrets.

## Documentation

- [docs/user_guide.md](docs/user_guide.md)
- [docs/admin_guide.md](docs/admin_guide.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/mattermost-compat.md](docs/mattermost-compat.md)
- [docs/mattermost-v4-comparison.md](docs/mattermost-v4-comparison.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT. See [LICENSE](LICENSE).
