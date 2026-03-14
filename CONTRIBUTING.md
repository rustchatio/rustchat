# Contributing to rustchat

Thank you for contributing to rustchat.

## Prerequisites

- Rust `1.92+` (see `backend/Cargo.toml`)
- Node.js `24+` (see `frontend-solid/package.json` engines)
- Docker + Docker Compose

## Development Setup

1. Clone your fork and enter the repository.
2. Copy environment defaults:
   ```bash
   cp .env.example .env
   ```
3. Set required secrets in `.env`:
   - `RUSTCHAT_JWT_SECRET`
   - `RUSTCHAT_ENCRYPTION_KEY`
   - `RUSTCHAT_S3_ACCESS_KEY`
   - `RUSTCHAT_S3_SECRET_KEY`
   - `RUSTFS_ACCESS_KEY`
   - `RUSTFS_SECRET_KEY`
4. Start local dependencies:
   ```bash
   docker compose up -d postgres redis rustfs
   ```
5. Run backend and frontend locally (separate terminals):
   ```bash
   cd backend && cargo run
   cd frontend-solid && npm ci && npm run dev
   ```

For full containerized startup, use:

```bash
docker compose up -d --build
```

## Project Structure

- `backend/`: Rust API server and websocket layer
- `frontend-solid/`: Solid.js 1.9 + TypeScript client
- `push-proxy/`: push notification proxy service
- `scripts/`: smoke and utility scripts
- `tools/mm-compat/`: API compatibility tooling

## Code Quality Requirements

Before opening a PR, run these checks.

### Backend

```bash
cd backend
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test --no-fail-fast -- --nocapture
```

### Frontend

```bash
cd frontend-solid
npm ci
npm run build
```

### Compatibility Smoke Tests

Run these when touching v4 API/websocket/compatibility-sensitive behavior:

```bash
./scripts/mm_compat_smoke.sh
./scripts/mm_mobile_smoke.sh
```

## Coding Guidelines

- Keep code and documentation in English.
- Keep functions focused and avoid unrelated refactors in the same PR.
- Add or update tests for bug fixes and behavior changes whenever feasible.
- Preserve API and websocket contract behavior unless the change explicitly targets it.

## Command UX Standard (Permanent)

- Primary command invocation is `Ctrl/Cmd+K` on desktop.
- Mobile/typed equivalent is `^k`.
- Do not implement `/` as the primary command trigger in the UI.
- New command features must integrate with the command menu flow, not slash-triggered UX.

## Pull Request Process

1. Create a branch from `main` (for example: `feature/my-change` or `fix/my-change`).
2. Make focused changes with clear commits.
3. Run the required checks listed above.
4. Open a PR with:
   - concise summary
   - verification steps/commands run
   - compatibility impact (if any)
5. Address review feedback and keep history clean.

## Commit Messages

Use Conventional Commit style:

```text
feat: add user registration endpoint
fix: correct JWT expiry calculation
docs: update API documentation
test: add channel permission tests
```

## Compatibility-Sensitive Changes

If your change affects API v4 contracts, mobile/desktop client compatibility, websocket events, or calls behavior:

1. Analyze upstream behavior first in `../mattermost` and `../mattermost-mobile`.
2. Document findings in a new folder under `previous-analyses/YYYY-MM-DD-<topic>/`.
3. Use templates from `previous-analyses/_TEMPLATE/`.

## Security Notes

- Never commit secrets, credentials, or private keys.
- For production hardening guidance, see:
  - `docs/security-deployment-guide.md`
  - `docs/security-zero-trust-guide.md`

## Questions

Open an issue or start a discussion in the repository.
