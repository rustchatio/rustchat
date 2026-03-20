# Rate Limit Service Design

## Overview

Implement IP-based and user-account-based rate limiting with admin console configuration and hot-reload support. This addresses PR review issues P1-1 (legacy `check_rate_limit` stub always returns `allowed: true`) and P1-2 (IP middleware is pass-through).

## Goals

- Replace stub `check_rate_limit` with actual Redis-backed rate limiting
- Implement IP-based rate limiting in middleware for unauthenticated endpoints
- Support per-account rate limiting for authenticated endpoints
- Allow admin console to view and modify rate limits
- Hot-reload limits without process restart

## Non-Goals

- Distributed rate limiting beyond Redis-backed atomic counters
- Per-endpoint granular rate limits (just the 5 key endpoints)
- Historical rate limit analytics or reporting

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       HTTP Request                          │
└──────────────────────┬──────────────────────────────────────┘
                       │
           ┌───────────┴───────────┐
           │                       │
    ┌──────▼──────┐         ┌──────▼──────┐
    │   IP-based  │         │ User-based  │
    │  Middleware │         │  Auth Check │
    │  (P1-2)     │         │  (P1-1)     │
    └──────┬──────┘         └──────┬──────┘
           │                       │
           └───────────┬───────────┘
                       │
              ┌────────▼────────┐
              │ RateLimitService│
              │                 │
              │ ┌─────────────┐ │
              │ │RwLock<Limits>│ │  ← Hot-reload cache
              │ └─────────────┘ │
              │                 │
              │ ┌─────────────┐ │
              │ │ Redis Lua   │ │  ← Atomic INCR+EXPIRE
              │ │   Script    │ │
              │ └─────────────┘ │
              └────────┬────────┘
                       │
              ┌────────▼────────┐
              │     Redis       │
              └─────────────────┘
                       │
           ┌───────────┴───────────┐
           │                       │
    ┌──────▼──────┐         ┌──────▼──────┐
    │   Reject    │         │   Admin     │
    │   (429)     │         │  Console    │
    │             │         │  (PUT/GET)  │
    └─────────────┘         └──────┬──────┘
                                   │
                          ┌────────▼────────┐
                          │  PostgreSQL     │
                          │ rate_limits tbl │
                          └─────────────────┘
```

## Components

### 1. Database Schema

```sql
CREATE TABLE rate_limits (
    key TEXT PRIMARY KEY,
    limit_value INTEGER NOT NULL,  -- requests allowed (for limits) or 1/0 (for enabled flags)
    window_secs INTEGER NOT NULL DEFAULT 60,  -- 0 means this is an enabled flag row
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

COMMENT ON TABLE rate_limits IS 'Rate limit configuration with toggle flags. Rows with window_secs=0 are enabled flags.';

-- Initial seed data
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_per_minute', 20, 60),
    ('auth_user_per_minute', 10, 60),
    ('register_ip_per_minute', 10, 60),
    ('password_reset_ip_per_minute', 5, 60),
    ('websocket_ip_per_minute', 30, 60);

-- Enabled flags (window_secs=0 indicates this is a toggle, not a limit)
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_enabled', 1, 0),
    ('auth_user_enabled', 1, 0),
    ('register_ip_enabled', 1, 0),
    ('password_reset_ip_enabled', 1, 0),
    ('websocket_ip_enabled', 1, 0);
```

### 2. RateLimitService

Located in `backend/src/services/rate_limit.rs`.

```rust
use std::sync::RwLock;

/// Runtime rate limit configuration (hot-reloadable)
#[derive(Debug, Clone)]
pub struct RateLimitLimits {
    pub auth_ip: IpRateLimitConfig,
    pub auth_user: IpRateLimitConfig,
    pub register_ip: IpRateLimitConfig,
    pub password_reset_ip: IpRateLimitConfig,
    pub websocket_ip: IpRateLimitConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct IpRateLimitConfig {
    pub limit: u64,
    pub window_secs: u64,
    pub enabled: bool,
}

pub struct RateLimitService {
    redis: Pool,
    script: Arc<Script>,
    limits: Arc<RwLock<RateLimitLimits>>,
}

impl RateLimitService {
    /// Create service with initial limits loaded from DB
    pub fn new(redis: Pool, initial_limits: RateLimitLimits) -> Self;

    /// Hot-reload limits from database
    pub async fn reload(&self) -> ApiResult<()>;

    /// Get current limits (cloned)
    pub fn limits(&self) -> RateLimitLimits;

    /// Generic key-based check (used by middleware)
    pub async fn check_key(&self, key: &str, config: &IpRateLimitConfig) -> ApiResult<()>;

    /// Convenience methods for specific limit types
    pub async fn check_auth_ip(&self, ip: &str) -> ApiResult<()>;
    pub async fn check_auth_user(&self, user_id: Uuid) -> ApiResult<()>;
    pub async fn check_register_ip(&self, ip: &str) -> ApiResult<()>;
    pub async fn check_password_reset_ip(&self, ip: &str) -> ApiResult<()>;
    pub async fn check_websocket_ip(&self, ip: &str) -> ApiResult<()>;
}
```

### 3. IP Middleware Implementation

Replace pass-through stubs in `backend/src/middleware/rate_limit.rs`:

```rust
pub async fn auth_ip_rate_limit(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = addr.ip().to_string();
    state.rate_limit.check_auth_ip(&ip).await?;
    Ok(next.run(request).await)
}

pub async fn register_ip_rate_limit(...)
pub async fn password_reset_ip_rate_limit(...)
pub async fn websocket_ip_rate_limit(...)
```

### 4. Auth Handler Updates

In `backend/src/api/auth.rs` and `backend/src/api/v4/users.rs`, update login handlers:

```rust
async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(input): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // ... find user ...

    // Check IP-based rate limit
    let ip = addr.ip().to_string();
    state.rate_limit.check_auth_ip(&ip).await?;

    // Check user-account rate limit
    state.rate_limit.check_auth_user(user.id).await?;

    // ... verify password ...
}
```

Remove the legacy `check_rate_limit` stub function entirely.

### 5. Admin Console API

New endpoints in `backend/src/api/v4/admin.rs`:

```rust
// GET /api/v4/admin/rate-limits
pub async fn get_rate_limits(State(state): State<AppState>) -> ApiResult<Json<RateLimitsResponse>>;

// PUT /api/v4/admin/rate-limits
pub async fn update_rate_limits(
    State(state): State<AppState>,
    Json(payload): Json<UpdateRateLimitsRequest>,
) -> ApiResult<Json<RateLimitsResponse>>;
```

Request/response types:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitsResponse {
    pub auth_ip: LimitEntry,
    pub auth_user: LimitEntry,
    pub register_ip: LimitEntry,
    pub password_reset_ip: LimitEntry,
    pub websocket_ip: LimitEntry,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitEntry {
    pub limit: u32,
    pub window_secs: u32,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRateLimitsRequest {
    pub auth_ip: Option<LimitEntry>,
    pub auth_user: Option<LimitEntry>,
    pub register_ip: Option<LimitEntry>,
    pub password_reset_ip: Option<LimitEntry>,
    pub websocket_ip: Option<LimitEntry>,
}
```

Hot-reload is triggered automatically on successful PUT.

### 6. Error Responses

When rate limit exceeded:

```json
{
  "error": "Rate limit exceeded",
  "message": "Too many login attempts. Please try again later."
}
```

HTTP headers on 429:

```
HTTP/1.1 429 Too Many Requests
Retry-After: 45
X-RateLimit-Limit: 20
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1710864000
```

On successful requests, optionally include:

```
X-RateLimit-Remaining: 15
```

### Redis Lua Script

The Lua script atomically increments the counter, sets expiry on first increment, and returns both the new count and the remaining TTL:

```lua
local key = KEYS[1]
local ttl = tonumber(ARGV[1])
local count = redis.call('INCR', key)
if count == 1 then
    redis.call('EXPIRE', key, ttl)
end
local remaining_ttl = redis.call('TTL', key)
return {count, remaining_ttl}
```

Returns: `[count, ttl_seconds]` — count is the current request count, ttl_seconds is time until window resets (for `X-RateLimit-Reset` header).

## Data Flow

### Request Path (Rate Limit Check)

```
1. Request arrives at middleware/auth handler
2. Extract client IP or user ID
3. Call RateLimitService::check_*()
4. Build Redis key: "ratelimit:ip:{ip}" or "ratelimit:user:{user_id}"
5. Execute Lua script (atomic INCR+EXPIRE), returns [count, ttl]
6. Compare count against RwLock-cached limit
7. If exceeded → return 429 with Retry-After = ttl
8. If allowed → continue to handler
```

### Admin Update Path (Hot Reload)

```
1. Admin PUT /api/v4/admin/rate-limits
2. Validate and update rate_limits table rows
3. Call rate_limit_service.reload()
4. Query all rows from rate_limits table
5. Build new RateLimitLimits struct
6. Write to RwLock (replaces old config)
7. Return updated limits to admin
8. Subsequent requests use new limits immediately
```

## Migration

### SQL Migration

File: `backend/migrations/20260319000001_add_rate_limits_table.sql`

```sql
-- Create rate_limits table
CREATE TABLE rate_limits (
    key TEXT PRIMARY KEY,
    limit_value INTEGER NOT NULL,
    window_secs INTEGER NOT NULL DEFAULT 60,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed default values
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_per_minute', 20, 60),
    ('auth_user_per_minute', 10, 60),
    ('register_ip_per_minute', 10, 60),
    ('password_reset_ip_per_minute', 5, 60),
    ('websocket_ip_per_minute', 30, 60),
    ('auth_ip_enabled', 1, 0),
    ('auth_user_enabled', 1, 0),
    ('register_ip_enabled', 1, 0),
    ('password_reset_ip_enabled', 1, 0),
    ('websocket_ip_enabled', 1, 0);
```

## Testing

### Unit Tests (services/rate_limit.rs)

- `test_check_key_under_limit` → passes
- `test_check_key_at_limit` → passes
- `test_check_key_over_limit` → returns error
- `test_reload_updates_limits` → RwLock reflects new values
- `test_disabled_limit_bypasses` → always passes

### Integration Tests

- Login endpoint with rapid requests → 429 after limit
- Register endpoint with rapid requests → 429 after limit
- Admin GET rate-limits → returns current config
- Admin PUT rate-limits + immediate retry → new limits active

## Migration Path

1. Run SQL migration to create table and seed defaults
2. Deploy code with new RateLimitService
3. AppState construction loads initial limits from DB
4. Legacy `check_rate_limit` stub removed, callers updated
5. IP middleware stubs replaced with actual checks
6. Admin API endpoints exposed
7. Remove legacy config fields from SecurityConfig (optional cleanup)

## Security Considerations

- IP extraction: use `X-Forwarded-For` header when `TRUST_PROXY=true` env var is set, otherwise use `ConnectInfo` socket address. Take the first IP from X-Forwarded-For (the originating client IP).
- Admin API requires admin role (reuse existing auth middleware)
- Rate limit keys use consistent prefix to avoid collisions
- Redis Lua script ensures atomicity under concurrent load

## Operational Notes

- `reload()` has a 5-second database query timeout to prevent blocking admin responses indefinitely
- Rate limit errors are logged at WARN level with the key for monitoring
- Redis connection failures fail open (allow request) to prevent cascading outages
