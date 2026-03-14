# rustchat Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Web / SPA Client                       │
│               (Solid.js + TypeScript + Vite)                │
└─────────────────────────┬───────────────────────────────────┘
                          │ REST / WebSocket
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    rustchat API Server                      │
│                    (Axum + Tokio)                           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │  Auth   │  │ Channels│  │  Posts  │  │   Real-time     │ │
│  │ Module  │  │ Module  │  │ Module  │  │   WebSocket Hub │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
│       │            │            │                │          │
│  ┌────▼────────────▼────────────▼────────────────▼────────┐ │
│  │                    Service Layer                       │ │
│  └────────────────────────┬───────────────────────────────┘ │
└───────────────────────────┼─────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   ┌────▼────┐         ┌────▼────┐         ┌────▼────┐
   │Postgres │         │  Redis  │         │   S3    │
   │ (Data)  │         │ (Cache) │         │ (Files) │
   └─────────┘         └─────────┘         └─────────┘
```

---

## Backend Architecture

### API Layer (`src/api/`)

HTTP request handling using Axum:
- Route definitions
- Request validation
- Response formatting
- Middleware (auth, logging, CORS)

### Service Layer

Business logic modules:
- **auth** — Authentication, JWT tokens, password hashing
- **orgs** — Organization management
- **teams** — Team membership and permissions
- **channels** — Channel CRUD and access control
- **posts** — Message handling, threads, reactions
- **files** — Upload/download, S3 integration
- **realtime** — WebSocket connections, event fan-out

### Data Layer (`src/db/`)

PostgreSQL with SQLx:
- Connection pool management
- Compile-time checked queries
- Migration runner

### Configuration (`src/config/`)

Environment-based configuration using the `config` crate.

### Error Handling (`src/error/`)

Structured error types with HTTP status mapping.

### Telemetry (`src/telemetry/`)

Structured JSON logging with `tracing`.

---

## Frontend Architecture (Solid)

The active web client lives in `frontend-solid/` and is implemented with Solid.js + TypeScript.

### Directory Structure

```
frontend-solid/src/
├── api/                           # API clients
├── auth/                          # OIDC/SAML/auth helpers
├── components/                    # Solid UI components
├── hooks/                         # WebSocket/media query/toast hooks
├── realtime/                      # Real-time websocket layer
├── routes/                        # Route components
├── stores/                        # Solid state stores
├── styles/                        # Tailwind/CSS tokens and globals
├── types/                         # TypeScript types
└── utils/                         # Shared utilities
```

### Key Principles

1. **Route-first composition**: Route components orchestrate stores and API calls.
2. **Thin API layer**: `src/api/*` wraps HTTP contracts and response typing.
3. **State-centric UI**: `src/stores/*` is the primary client state boundary.
4. **Realtime integration**: websocket events update stores via dedicated hooks/handlers.
5. **Type safety**: strict TypeScript typing across DTOs and store state.

### Usage Example

```typescript
import { authStore } from '@/stores/auth';
import { channelStore } from '@/stores/channels';
import { useWebSocket } from '@/hooks/useWebSocket';

useWebSocket();
if (authStore.isAuthenticated) {
  // e.g. load channels/messages after auth
}
```

---

## Data Flow

### HTTP Request (Backend)

```
Request → Middleware → Router → Handler → Service → Repository → Database
                                   ↓
Response ← JSON Serialization ← Result
```

### WebSocket Event (Backend)

```
Client ← WebSocket Hub ← Event Publisher ← Service ← Database Change
```

Detailed adapter boundaries and v1/v4 contract mapping:
`docs/websocket_architecture.md`.

### Frontend Data Flow

```
Route/Component → Store → API Client → Backend
                    ↑
            WebSocket Hook/Event
```

---

## Database Schema

Key tables:
- `organizations` — Multi-tenant organizations
- `users` — User accounts (humans and bots)
- `teams` — Teams within organizations
- `channels` — Communication channels
- `posts` — Messages and threads
- `files` — File metadata

---

## Scalability

- **Stateless API servers** — Horizontal scaling behind load balancer
- **Redis pub/sub** — Cross-instance event propagation
- **Redis-backed calls state** — Shared call control-plane state (`docs/calls_deployment_modes.md`)
- **Connection pooling** — Efficient database connections
- **Async I/O** — Non-blocking operations throughout

---

## Documentation

- `docs/websocket_architecture.md` — WebSocket protocol details
- `docs/calls_deployment_modes.md` — Call service deployment
- `frontend-solid/README.md` — Frontend setup and scripts
- `frontend-solid/PHASE_F3_SUMMARY.md` — Migration phase summary
- `frontend-solid/PHASE_F4_SUMMARY.md` — Migration phase summary
- `FRONTEND_MIGRATION_SUMMARY.md` — Repository-level migration status
