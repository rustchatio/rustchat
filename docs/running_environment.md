# Running Environment

This page is the compatibility entry point for local and production runtime setup.

## Start Here

- Local development setup: [Development Local Setup](./development/local-setup.md)
- Deployment and first install: [Admin Installation](./admin/installation.md)
- Environment variables: [Admin Configuration](./admin/configuration.md)
- Production operations: [Operations Runbook](./operations/runbook.md)

## Quick Local Commands

```bash
# Start core dependencies
docker compose up -d postgres redis rustfs

# Backend (terminal 1)
cd backend
cargo run

# Frontend (terminal 2)
cd frontend
npm run dev
```

## Quick Full-Stack Docker Command

```bash
cp .env.example .env
docker compose up -d --build
```
