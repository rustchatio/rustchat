# Rate Limit Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace stub rate limiting with a Redis-backed `RateLimitService` that enforces IP-based and per-user login limits, is configurable from the admin console, and hot-reloads without process restart.

**Architecture:** Extend the existing `RateLimitService` in `services/rate_limit.rs` with a new `check_key()` method and `IpRateLimitConfig`/`RateLimitLimits` structs backed by a `tokio::sync::RwLock`. The service holds a `db: sqlx::PgPool` for `reload()`. Initial limits are loaded in `main.rs` before `api::router()` is called, then stored in `AppState::rate_limit: Arc<RateLimitService>`. IP middleware stubs are replaced with actual checks; auth login handlers call both IP and per-user checks.

**Tech Stack:** Rust, Axum, deadpool-redis, SQLx (PostgreSQL), `tokio::sync::RwLock`, Redis Lua scripting.

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `backend/migrations/20260319000001_add_rate_limits_table.sql` | **Create** | DB schema and seed data |
| `backend/src/services/rate_limit.rs` | **Modify** | Add `IpRateLimitConfig`, `RateLimitLimits`, `db` field, `check_key()`, `reload()`, convenience methods; update Lua script |
| `backend/src/api/mod.rs` | **Modify** | Add `rate_limit: Arc<RateLimitService>` to `AppState`; update `router()` signature |
| `backend/src/main.rs` | **Modify** | Construct and reload `RateLimitService` before calling `api::router()` |
| `backend/src/middleware/rate_limit.rs` | **Modify** | Replace pass-through stubs with real `check_*` calls; remove legacy stub (after Tasks 6+7) |
| `backend/src/api/auth.rs` | **Modify** | Replace legacy stub call with real IP + user rate limit checks |
| `backend/src/api/v4/users.rs` | **Modify** | Replace legacy stub call with real IP + user rate limit checks |
| `backend/src/api/v4/admin.rs` | **Modify** | Add `GET /admin/rate-limits` and `PUT /admin/rate-limits` handlers |
| `backend/src/api/v4/mod.rs` | **Modify** | Ensure admin router is registered |

---

## Task 1: Database Migration

**Files:**
- Create: `backend/migrations/20260319000001_add_rate_limits_table.sql`

- [ ] **Step 1: Write the migration file**

```sql
-- Rate limit configuration table.
-- Two row types share this table:
--   Limit rows:  window_secs > 0, limit_value = max requests per window
--   Flag rows:   window_secs = 0, limit_value = 1 (enabled) or 0 (disabled)
CREATE TABLE rate_limits (
    key TEXT PRIMARY KEY,
    limit_value INTEGER NOT NULL,
    window_secs INTEGER NOT NULL DEFAULT 60,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE rate_limits IS
  'Rate limit config. Rows with window_secs=0 are enabled flags (limit_value 1=on, 0=off). '
  'Rows with window_secs>0 are request limits (limit_value = max requests per window_secs).';

-- Limit rows
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_per_minute',           20, 60),
    ('auth_user_per_minute',         10, 60),
    ('register_ip_per_minute',       10, 60),
    ('password_reset_ip_per_minute',  5, 60),
    ('websocket_ip_per_minute',      30, 60);

-- Enabled flag rows (window_secs = 0 marks these as flags)
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_enabled',           1, 0),
    ('auth_user_enabled',         1, 0),
    ('register_ip_enabled',       1, 0),
    ('password_reset_ip_enabled', 1, 0),
    ('websocket_ip_enabled',      1, 0);
```

- [ ] **Step 2: Verify migration runs**

```bash
cd backend && cargo sqlx migrate run
```
Expected: Migration applies without error. Run `psql $DATABASE_URL -c "SELECT * FROM rate_limits;"` and confirm 10 rows.

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/20260319000001_add_rate_limits_table.sql
git commit -m "feat(db): add rate_limits configuration table"
```

---

## Task 2: New types — `IpRateLimitConfig` and `RateLimitLimits`

**Files:**
- Modify: `backend/src/services/rate_limit.rs`

Add the new types for IP/key-based rate limiting. The existing `RateLimitConfig` (entity-tier struct) and `RateLimitService` struct are not changed yet.

- [ ] **Step 1: Add types after the closing `}` of `impl RateLimitConfig`**

In `backend/src/services/rate_limit.rs`, find the end of `impl RateLimitConfig` block and add immediately after:

```rust
/// Rate limit config for IP/key-based limits (admin-configurable, hot-reloadable)
#[derive(Debug, Clone, Copy)]
pub struct IpRateLimitConfig {
    /// Max requests allowed per window
    pub limit: u64,
    /// Window duration in seconds
    pub window_secs: u64,
    /// If false, this limit is skipped entirely
    pub enabled: bool,
}

/// Full set of hot-reloadable IP rate limit configs, populated from the `rate_limits` DB table
#[derive(Debug, Clone)]
pub struct RateLimitLimits {
    pub auth_ip: IpRateLimitConfig,
    pub auth_user: IpRateLimitConfig,
    pub register_ip: IpRateLimitConfig,
    pub password_reset_ip: IpRateLimitConfig,
    pub websocket_ip: IpRateLimitConfig,
}

impl Default for RateLimitLimits {
    fn default() -> Self {
        Self {
            auth_ip:            IpRateLimitConfig { limit: 20, window_secs: 60, enabled: true },
            auth_user:          IpRateLimitConfig { limit: 10, window_secs: 60, enabled: true },
            register_ip:        IpRateLimitConfig { limit: 10, window_secs: 60, enabled: true },
            password_reset_ip:  IpRateLimitConfig { limit:  5, window_secs: 60, enabled: true },
            websocket_ip:       IpRateLimitConfig { limit: 30, window_secs: 60, enabled: true },
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd backend && cargo check
```
Expected: Clean.

- [ ] **Step 3: Commit**

```bash
git add backend/src/services/rate_limit.rs
git commit -m "feat(rate-limit): add IpRateLimitConfig and RateLimitLimits types"
```

---

## Task 3: Extend `RateLimitService` — Lua script, new fields, new methods

**Files:**
- Modify: `backend/src/services/rate_limit.rs`

This is the core task. We update the Lua script, add `db` and `ip_limits` fields, update both constructors, fix `check_rate_limit` for the new script return type, and add `check_key`, `reload`, and convenience methods.

- [ ] **Step 1: Add missing imports at the top of the file**

Ensure these are present (add any that are missing):

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
```

- [ ] **Step 2: Replace the `RATE_LIMIT_SCRIPT` constant**

Find `const RATE_LIMIT_SCRIPT` and replace the entire constant:

```rust
/// Lua script for atomic INCR+EXPIRE.
/// Returns a two-element array: [current_count, ttl_remaining_secs]
///
/// KEYS[1]: Redis key
/// ARGV[1]: Window TTL in seconds
const RATE_LIMIT_SCRIPT: &str = r#"
local key = KEYS[1]
local ttl = tonumber(ARGV[1])
local count = redis.call('INCR', key)
if count == 1 then
    redis.call('EXPIRE', key, ttl)
end
local remaining_ttl = redis.call('TTL', key)
return {count, remaining_ttl}
"#;
```

- [ ] **Step 3: Add `db` and `ip_limits` fields to `RateLimitService`**

Find the `pub struct RateLimitService` definition and add the two new fields:

```rust
pub struct RateLimitService {
    redis: Pool,
    script: Arc<Script>,
    /// PostgreSQL pool used by reload() to read the rate_limits table
    db: sqlx::PgPool,
    /// Hot-reloadable IP/key rate limit configuration
    ip_limits: Arc<RwLock<RateLimitLimits>>,
}
```

- [ ] **Step 4: Rewrite `RateLimitService::new` and add `new_with_limits`**

Replace the existing `new` method and add `new_with_limits`. Both take `redis` and `db`:

```rust
impl RateLimitService {
    /// Create service with default IP limits (call `reload()` to populate from DB).
    pub fn new(redis: Pool, db: sqlx::PgPool) -> Self {
        Self {
            redis,
            script: Arc::new(Script::new(RATE_LIMIT_SCRIPT)),
            db,
            ip_limits: Arc::new(RwLock::new(RateLimitLimits::default())),
        }
    }

    /// Create service with pre-loaded IP limits.
    pub fn new_with_limits(redis: Pool, db: sqlx::PgPool, initial_limits: RateLimitLimits) -> Self {
        Self {
            redis,
            script: Arc::new(Script::new(RATE_LIMIT_SCRIPT)),
            db,
            ip_limits: Arc::new(RwLock::new(initial_limits)),
        }
    }

    /// Returns a clone of the current IP rate limit configuration.
    pub async fn ip_limits(&self) -> RateLimitLimits {
        self.ip_limits.read().await.clone()
    }
```

- [ ] **Step 5: Update `check_rate_limit` for the new Lua return type**

The script now returns `[count, ttl]` instead of just `count`. Find `check_rate_limit` and update the `invoke_async` call and downstream usage:

```rust
    pub async fn check_rate_limit(
        &self,
        entity_id: &uuid::Uuid,
        tier: RateLimitTier,
    ) -> ApiResult<()> {
        let config = RateLimitConfig::for_tier(tier);

        if config.is_unlimited() {
            debug!(entity_id = %entity_id, tier = ?tier, "Rate limit bypassed for unlimited tier");
            return Ok(());
        }

        let key = self.build_key(entity_id, tier);

        let mut conn = self.redis.get().await.map_err(|e| {
            warn!(error = %e, entity_id = %entity_id, "Redis connection failed for entity rate limit");
            AppError::Internal(format!("Redis connection error: {}", e))
        })?;

        // Script now returns [count, ttl]; take only count for entity-level check
        let result: Vec<u64> = self
            .script
            .key(&key)
            .arg(config.window_secs)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                warn!(error = %e, entity_id = %entity_id, "Redis error during entity rate limit");
                AppError::Redis(e)
            })?;

        let count = result[0];

        if count > config.limit {
            warn!(entity_id = %entity_id, tier = ?tier, count, limit = config.limit, "Entity rate limit exceeded");
            return Err(AppError::RateLimitExceeded(format!(
                "Rate limit exceeded: {} requests in {}s window (limit: {})",
                count, config.window_secs, config.limit
            )));
        }

        debug!(entity_id = %entity_id, tier = ?tier, count, limit = config.limit, "Entity rate limit passed");
        Ok(())
    }
```

- [ ] **Step 6: Add `check_key` method**

```rust
    /// Check rate limit for an arbitrary string key.
    /// Fails open on Redis errors (allows request) to prevent cascading outages.
    ///
    /// Returns `Err(AppError::RateLimitExceeded)` when the limit is breached.
    pub async fn check_key(&self, key: &str, config: &IpRateLimitConfig) -> ApiResult<()> {
        if !config.enabled {
            return Ok(());
        }

        let redis_key = format!("{}:ip_limit:{}", RATE_LIMIT_KEY_PREFIX, key);

        let mut conn = match self.redis.get().await {
            Ok(c) => c,
            Err(e) => {
                warn!(error = %e, key, "Redis unavailable for IP rate limit; failing open");
                return Ok(());
            }
        };

        let result: Vec<i64> = match self
            .script
            .key(&redis_key)
            .arg(config.window_secs)
            .invoke_async(&mut *conn)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, key, "Redis Lua error during IP rate limit; failing open");
                return Ok(());
            }
        };

        let count = result[0] as u64;
        let ttl = result[1]; // seconds until window resets

        if count > config.limit {
            warn!(key, count, limit = config.limit, ttl, "IP rate limit exceeded");
            return Err(AppError::RateLimitExceeded(format!(
                "Too many requests. Retry after {}s.",
                ttl.max(0)
            )));
        }

        debug!(key, count, limit = config.limit, "IP rate limit passed");
        Ok(())
    }
```

- [ ] **Step 7: Add convenience methods**

```rust
    pub async fn check_auth_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.auth_ip;
        self.check_key(&format!("auth:ip:{}", ip), &config).await
    }

    pub async fn check_auth_user(&self, user_id: uuid::Uuid) -> ApiResult<()> {
        let config = self.ip_limits.read().await.auth_user;
        self.check_key(&format!("auth:user:{}", user_id), &config).await
    }

    pub async fn check_register_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.register_ip;
        self.check_key(&format!("register:ip:{}", ip), &config).await
    }

    pub async fn check_password_reset_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.password_reset_ip;
        self.check_key(&format!("password_reset:ip:{}", ip), &config).await
    }

    pub async fn check_websocket_ip(&self, ip: &str) -> ApiResult<()> {
        let config = self.ip_limits.read().await.websocket_ip;
        self.check_key(&format!("websocket:ip:{}", ip), &config).await
    }
```

- [ ] **Step 8: Add `reload` method**

The reload query reads all rows, applies limit rows first, then flag rows (order-independent single pass with flag rows overriding the `enabled` field):

```rust
    /// Hot-reload IP rate limits from the `rate_limits` database table.
    /// Applies a 5-second query timeout to avoid blocking callers.
    pub async fn reload(&self) -> ApiResult<()> {
        #[derive(sqlx::FromRow)]
        struct Row {
            key: String,
            limit_value: i32,
            window_secs: i32,
        }

        let rows: Vec<Row> = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            sqlx::query_as::<_, Row>(
                "SELECT key, limit_value, window_secs FROM rate_limits"
            )
            .fetch_all(&self.db),
        )
        .await
        .map_err(|_| AppError::Internal("Rate limit DB reload timed out".into()))?
        .map_err(|e| AppError::Internal(format!("Rate limit DB reload failed: {}", e)))?;

        let mut limits = RateLimitLimits::default();

        // First pass: apply limit rows (window_secs > 0)
        for row in &rows {
            if row.window_secs > 0 {
                let limit = row.limit_value as u64;
                let window_secs = row.window_secs as u64;
                match row.key.as_str() {
                    "auth_ip_per_minute" =>
                        limits.auth_ip = IpRateLimitConfig { limit, window_secs, ..limits.auth_ip },
                    "auth_user_per_minute" =>
                        limits.auth_user = IpRateLimitConfig { limit, window_secs, ..limits.auth_user },
                    "register_ip_per_minute" =>
                        limits.register_ip = IpRateLimitConfig { limit, window_secs, ..limits.register_ip },
                    "password_reset_ip_per_minute" =>
                        limits.password_reset_ip = IpRateLimitConfig { limit, window_secs, ..limits.password_reset_ip },
                    "websocket_ip_per_minute" =>
                        limits.websocket_ip = IpRateLimitConfig { limit, window_secs, ..limits.websocket_ip },
                    _ => {}
                }
            }
        }

        // Second pass: apply enabled flag rows (window_secs == 0), overriding the enabled field
        for row in &rows {
            if row.window_secs == 0 {
                let enabled = row.limit_value != 0;
                match row.key.as_str() {
                    "auth_ip_enabled"           => limits.auth_ip.enabled = enabled,
                    "auth_user_enabled"         => limits.auth_user.enabled = enabled,
                    "register_ip_enabled"       => limits.register_ip.enabled = enabled,
                    "password_reset_ip_enabled" => limits.password_reset_ip.enabled = enabled,
                    "websocket_ip_enabled"      => limits.websocket_ip.enabled = enabled,
                    _ => {}
                }
            }
        }

        *self.ip_limits.write().await = limits;
        tracing::info!("IP rate limits reloaded from database");
        Ok(())
    }
```

- [ ] **Step 9: Verify it compiles**

```bash
cd backend && cargo check
```
Expected: Clean. If you see errors about `AppError::Internal` taking a `String` vs `&str`, adjust `.into()` / `format!()` to match the variant's type. Check how other callers wrap sqlx errors (e.g. `AppError::Sqlx(e)` or `e.into()`).

- [ ] **Step 10: Add unit tests inside the existing `#[cfg(test)]` block**

```rust
    #[test]
    fn test_ip_rate_limit_config_defaults() {
        let limits = RateLimitLimits::default();
        assert_eq!(limits.auth_ip.limit, 20);
        assert_eq!(limits.auth_ip.window_secs, 60);
        assert!(limits.auth_ip.enabled);
        assert_eq!(limits.password_reset_ip.limit, 5);
        assert!(limits.password_reset_ip.enabled);
    }

    #[tokio::test]
    async fn test_check_key_disabled_bypasses() {
        // When enabled=false, check_key must return Ok without contacting Redis.
        // We pass a deliberately invalid Redis URL — if Redis were contacted it would error.
        let cfg_disabled = deadpool_redis::Config::from_url("redis://127.0.0.1:1"); // unreachable
        let pool = cfg_disabled
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("pool creation");
        // Use a fake PgPool — reload() is not called here so DB is irrelevant
        // (PgPool::connect panics if called; we don't call it)
        let svc = RateLimitService::new(pool, sqlx::PgPool::connect_lazy("postgres://x").unwrap());
        let config = IpRateLimitConfig { limit: 1, window_secs: 60, enabled: false };
        // Must succeed without touching Redis
        let result = svc.check_key("test_key", &config).await;
        assert!(result.is_ok(), "disabled limit should always pass: {:?}", result);
    }
```

- [ ] **Step 11: Run unit tests**

```bash
cd backend && cargo test -p rustchat services::rate_limit
```
Expected: All tests pass including the two new ones.

- [ ] **Step 12: Commit**

```bash
git add backend/src/services/rate_limit.rs
git commit -m "feat(rate-limit): extend RateLimitService with check_key, reload, and convenience methods"
```

---

## Task 4: Wire `RateLimitService` into `AppState`

**Files:**
- Modify: `backend/src/api/mod.rs`
- Modify: `backend/src/main.rs`

`router()` is synchronous so `reload()` cannot be called inside it. Instead, construct and reload the service in `main.rs` and pass it in.

- [ ] **Step 1: Add import to `backend/src/api/mod.rs`**

```rust
use crate::services::rate_limit::RateLimitService;
```

- [ ] **Step 2: Add `rate_limit` field to `AppState`**

In the `AppState` struct (around line where `circuit_breakers` is), add:

```rust
pub rate_limit: Arc<RateLimitService>,
```

- [ ] **Step 3: Add `rate_limit` parameter to `router()`**

Change the signature of `pub fn router(...)` to accept a new parameter:

```rust
pub fn router(
    db: PgPool,
    redis: deadpool_redis::Pool,
    jwt_secret: String,
    jwt_expiry_hours: u64,
    ws_hub: Arc<WsHub>,
    s3_client: S3Client,
    config: Config,
    rate_limit: Arc<RateLimitService>,   // <-- new parameter
) -> Router {
```

- [ ] **Step 4: Thread `rate_limit` into both `AppState` constructions inside `router()`**

Both the `temp_state` and the final `state` need the field. For `temp_state`, construct a throwaway service (it won't be reloaded):

```rust
// temp_state (reconciliation worker only — rate_limit not used here)
rate_limit: Arc::new(RateLimitService::new(redis.clone(), db.clone())),
```

For the final `state`:

```rust
rate_limit,  // use the Arc<RateLimitService> passed in from main.rs
```

- [ ] **Step 5: Update `main.rs` — construct and reload before `api::router()`**

In `main.rs`, before the `let app = api::router(...)` call, add:

```rust
use rustchat::services::rate_limit::RateLimitService;
use std::sync::Arc;

let rate_limit_service = {
    let svc = RateLimitService::new(redis_pool.clone(), db_pool.clone());
    if let Err(e) = svc.reload().await {
        warn!(error = %e, "Rate limit DB load failed at startup; using defaults");
    } else {
        info!("Rate limits loaded from database");
    }
    Arc::new(svc)
};
```

Then update the `api::router(...)` call to pass `rate_limit_service`:

```rust
let app = api::router(
    db_pool.clone(),
    redis_pool,
    config.jwt_secret.clone(),
    config.jwt_expiry_hours,
    ws_hub,
    s3_client,
    config.clone(),
    rate_limit_service,  // <-- new argument
);
```

- [ ] **Step 6: Verify it compiles**

```bash
cd backend && cargo check
```
Expected: Clean compile.

- [ ] **Step 7: Commit**

```bash
git add backend/src/api/mod.rs backend/src/main.rs
git commit -m "feat(app-state): add RateLimitService to AppState, load from DB at startup"
```

---

## Task 5: Implement IP middleware (P1-2)

**Files:**
- Modify: `backend/src/middleware/rate_limit.rs`

Replace the four pass-through stubs with actual calls. **Do not remove the legacy stub yet** — `auth.rs` and `v4/users.rs` still reference it; that removal happens in Tasks 6 and 7.

- [ ] **Step 1: Add helper imports and `extract_client_ip`**

At the top of `backend/src/middleware/rate_limit.rs`, replace or extend the existing imports:

```rust
use crate::error::AppError;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Extract the originating client IP.
/// When the `TRUST_PROXY` environment variable is `"true"`, reads the first
/// value from `X-Forwarded-For` (the client IP). Otherwise uses the socket address.
fn extract_client_ip(addr: &SocketAddr, headers: &HeaderMap) -> String {
    let trust_proxy = std::env::var("TRUST_PROXY")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if trust_proxy {
        if let Some(xff) = headers.get("x-forwarded-for") {
            if let Ok(s) = xff.to_str() {
                if let Some(ip) = s.split(',').next() {
                    return ip.trim().to_string();
                }
            }
        }
    }

    addr.ip().to_string()
}
```

- [ ] **Step 2: Replace `register_ip_rate_limit`**

```rust
pub async fn register_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_register_ip(&ip).await?;
    Ok(next.run(request).await)
}
```

- [ ] **Step 3: Replace `auth_ip_rate_limit`**

```rust
pub async fn auth_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_auth_ip(&ip).await?;
    Ok(next.run(request).await)
}
```

- [ ] **Step 4: Replace `password_reset_ip_rate_limit`**

```rust
pub async fn password_reset_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_password_reset_ip(&ip).await?;
    Ok(next.run(request).await)
}
```

- [ ] **Step 5: Replace `websocket_ip_rate_limit`**

```rust
pub async fn websocket_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_websocket_ip(&ip).await?;
    Ok(next.run(request).await)
}
```

- [ ] **Step 6: Verify it compiles (legacy stub still present)**

```bash
cd backend && cargo check
```
Expected: Clean — legacy stub is still in the file so `auth.rs` and `v4/users.rs` still compile.

- [ ] **Step 7: Commit**

```bash
git add backend/src/middleware/rate_limit.rs
git commit -m "feat(middleware): implement IP rate limiting — replace pass-through stubs (P1-2)"
```

---

## Task 6: Update `api/auth.rs` login handler (P1-1)

**Files:**
- Modify: `backend/src/api/auth.rs`

- [ ] **Step 1: Remove the legacy rate-limit import**

Find and delete this line near the top of the file:

```rust
use crate::middleware::rate_limit::{self, RateLimitConfig};
```

- [ ] **Step 2: Add `ConnectInfo` and `SocketAddr` to the `login()` signature and imports**

The `login()` function currently does **not** take `ConnectInfo`. Add the extractor:

```rust
async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,   // <-- add this
    Json(input): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
```

Add imports at the top if not present:

```rust
use axum::extract::ConnectInfo;
use std::net::SocketAddr;
```

- [ ] **Step 3: Replace the rate-limit block in `login()`**

Find the block (look for `if state.config.security.rate_limit_enabled`):

```rust
if state.config.security.rate_limit_enabled {
    let config =
        RateLimitConfig::auth_per_minute(state.config.security.rate_limit_auth_per_minute);
    let user_key = format!("user:{}", user.id);
    let user_result = rate_limit::check_rate_limit(&state.redis, &config, &user_key).await?;

    if !user_result.allowed {
        tracing::warn!(user_id = %user.id, "Rate limit exceeded for user login");
        return Err(AppError::TooManyRequests(
            "Too many login attempts. Please try again later.".to_string(),
        ));
    }
}
```

Replace with:

```rust
// IP-based limit (all attempts from this IP address)
state.rate_limit.check_auth_ip(&addr.ip().to_string()).await?;
// Per-account limit (attempts against this specific user account)
state.rate_limit.check_auth_user(user.id).await?;
```

- [ ] **Step 4: Verify it compiles**

```bash
cd backend && cargo check 2>&1 | grep "auth.rs"
```
Expected: No errors from `auth.rs`.

- [ ] **Step 5: Commit**

```bash
git add backend/src/api/auth.rs
git commit -m "fix(auth): replace stub rate limit with real IP+user-account checks (P1-1)"
```

---

## Task 7: Update `api/v4/users.rs` login handler (P1-1)

**Files:**
- Modify: `backend/src/api/v4/users.rs`

- [ ] **Step 1: Remove the legacy rate-limit import**

Find and delete:

```rust
use crate::middleware::rate_limit::{self, RateLimitConfig};
```

- [ ] **Step 2: Add `ConnectInfo` + `SocketAddr` to the v4 `login()` parameters if not present**

The v4 login function signature should include:

```rust
async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,   // add if missing
    // ... other extractors
) -> ApiResult<...>
```

Add `use axum::extract::ConnectInfo;` and `use std::net::SocketAddr;` at the top if not present.

- [ ] **Step 3: Replace the rate-limit block in the v4 `login()` function**

Find the block starting with `if state.config.security.rate_limit_enabled`:

```rust
if state.config.security.rate_limit_enabled {
    let config =
        RateLimitConfig::auth_per_minute(state.config.security.rate_limit_auth_per_minute);
    let user_key = format!("user:{}", user.id);
    let user_result = rate_limit::check_rate_limit(&state.redis, &config, &user_key).await?;
    if !user_result.allowed {
        tracing::warn!(
            user_id = %user.id,
            "Rate limit exceeded for v4 user login"
        );
        return Err(AppError::TooManyRequests(
            "Too many login attempts. Please try again later.".to_string(),
        ));
    }
}
```

Replace with:

```rust
state.rate_limit.check_auth_ip(&addr.ip().to_string()).await?;
state.rate_limit.check_auth_user(user.id).await?;
```

- [ ] **Step 4: Remove the legacy stub from `middleware/rate_limit.rs`**

Now that both callers are updated, open `backend/src/middleware/rate_limit.rs` and delete the entire legacy section:

```
// ============================================================================
// Legacy API - Kept for backward compatibility
// ============================================================================
```

...through to the end of the file (the `RateLimitConfig`, `RateLimitResult`, and `check_rate_limit` stub function).

- [ ] **Step 5: Full compile check**

```bash
cd backend && cargo check
```
Expected: Clean compile with no references to the deleted legacy types.

- [ ] **Step 6: Commit**

```bash
git add backend/src/api/v4/users.rs backend/src/middleware/rate_limit.rs
git commit -m "fix(v4/users): replace stub rate limit with real checks; remove legacy stub (P1-1)"
```

---

## Task 8: 429 response headers

**Files:**
- Modify: `backend/src/services/rate_limit.rs` (or wherever `AppError::RateLimitExceeded` is converted to an HTTP response)

The spec requires `Retry-After`, `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and `X-RateLimit-Reset` headers on 429 responses.

- [ ] **Step 1: Find where `AppError::RateLimitExceeded` is converted to a response**

Search for `RateLimitExceeded` in the error handling code:

```bash
grep -rn "RateLimitExceeded" backend/src/
```

Find the `IntoResponse` implementation for `AppError` (likely in `backend/src/error.rs` or similar). Identify the arm that handles `RateLimitExceeded`.

- [ ] **Step 2: Add TTL info to `RateLimitExceeded` error variant**

The current `AppError::RateLimitExceeded(String)` only carries a message. Update it to also carry the TTL so headers can be set. Change the variant to:

```rust
RateLimitExceeded { message: String, retry_after_secs: i64 }
```

Update the `check_key` method in `services/rate_limit.rs` to use the new form:

```rust
return Err(AppError::RateLimitExceeded {
    message: format!("Too many requests. Retry after {}s.", ttl.max(0)),
    retry_after_secs: ttl.max(0),
});
```

Also update the entity-tier `check_rate_limit` return — it doesn't have TTL info from Redis, so use `window_secs` as a conservative estimate:

```rust
return Err(AppError::RateLimitExceeded {
    message: format!(
        "Rate limit exceeded: {} requests in {}s window (limit: {})",
        count, config.window_secs, config.limit
    ),
    retry_after_secs: config.window_secs as i64,
});
```

- [ ] **Step 3: Add response headers in the `IntoResponse` impl**

In the `IntoResponse` for `AppError`, update the `RateLimitExceeded` arm to add headers:

```rust
AppError::RateLimitExceeded { message, retry_after_secs } => {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let reset_at = now + retry_after_secs.max(0) as u64;

    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({
            "error": "rate_limit_exceeded",
            "message": message,
        })),
    ).into_response();

    let headers = response.headers_mut();
    headers.insert("Retry-After", retry_after_secs.max(0).to_string().parse().unwrap());
    headers.insert("X-RateLimit-Reset", reset_at.to_string().parse().unwrap());
    response
}
```

Note: `X-RateLimit-Limit` and `X-RateLimit-Remaining` require the limit value which is not available in the error variant. Either:
- **Option A (simpler):** Only add `Retry-After` and `X-RateLimit-Reset` (sufficient for RFC 6585 compliance).
- **Option B:** Add `limit` and `remaining` fields to `RateLimitExceeded` and pass them from `check_key`.

Use Option A unless the team specifically requires the limit/remaining headers.

- [ ] **Step 4: Fix all compilation errors from the variant change**

```bash
cd backend && cargo check 2>&1 | grep "RateLimitExceeded"
```

Fix any pattern matches on `RateLimitExceeded(msg)` to use `RateLimitExceeded { message: msg, .. }`.

- [ ] **Step 5: Verify clean compile**

```bash
cd backend && cargo check
```
Expected: Clean.

- [ ] **Step 6: Commit**

```bash
git add backend/src/
git commit -m "feat(rate-limit): add Retry-After and X-RateLimit-Reset headers on 429 responses"
```

---

## Task 9: Admin API — `GET` and `PUT` rate-limits endpoints

**Files:**
- Modify: `backend/src/api/v4/admin.rs`
- Modify: `backend/src/api/v4/mod.rs` (if admin routes not yet registered)

- [ ] **Step 1: Add types and helper function to `admin.rs`**

At the top of `backend/src/api/v4/admin.rs`, add:

```rust
use crate::services::rate_limit::{IpRateLimitConfig, RateLimitLimits};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LimitEntry {
    pub limit: u32,
    pub window_secs: u32,
    pub enabled: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct RateLimitsResponse {
    pub auth_ip: LimitEntry,
    pub auth_user: LimitEntry,
    pub register_ip: LimitEntry,
    pub password_reset_ip: LimitEntry,
    pub websocket_ip: LimitEntry,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateRateLimitsRequest {
    pub auth_ip: Option<LimitEntry>,
    pub auth_user: Option<LimitEntry>,
    pub register_ip: Option<LimitEntry>,
    pub password_reset_ip: Option<LimitEntry>,
    pub websocket_ip: Option<LimitEntry>,
}

fn limits_to_response(limits: &RateLimitLimits) -> RateLimitsResponse {
    let e = |c: IpRateLimitConfig| LimitEntry {
        limit: c.limit as u32,
        window_secs: c.window_secs as u32,
        enabled: c.enabled,
    };
    RateLimitsResponse {
        auth_ip: e(limits.auth_ip),
        auth_user: e(limits.auth_user),
        register_ip: e(limits.register_ip),
        password_reset_ip: e(limits.password_reset_ip),
        websocket_ip: e(limits.websocket_ip),
    }
}
```

- [ ] **Step 2: Add `get_rate_limits` handler**

```rust
pub async fn get_rate_limits(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<RateLimitsResponse>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }
    let limits = state.rate_limit.ip_limits().await;
    Ok(Json(limits_to_response(&limits)))
}
```

- [ ] **Step 3: Add `update_rate_limits` handler**

The flag key for a limit key is derived by trimming `_per_minute` and appending `_enabled`
(e.g., `"auth_ip_per_minute"` → `"auth_ip_enabled"`). This mapping is explicit in the match
arms of `reload()`, so there is no ambiguity for the 5 known keys.

```rust
pub async fn update_rate_limits(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(payload): Json<UpdateRateLimitsRequest>,
) -> ApiResult<Json<RateLimitsResponse>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden("Insufficient permissions".into()));
    }

    async fn upsert(db: &sqlx::PgPool, limit_key: &str, flag_key: &str, entry: &LimitEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO rate_limits (key, limit_value, window_secs, updated_at)
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (key) DO UPDATE
             SET limit_value = EXCLUDED.limit_value,
                 window_secs = EXCLUDED.window_secs,
                 updated_at = NOW()"
        )
        .bind(limit_key)
        .bind(entry.limit as i32)
        .bind(entry.window_secs as i32)
        .execute(db)
        .await?;

        sqlx::query(
            "INSERT INTO rate_limits (key, limit_value, window_secs, updated_at)
             VALUES ($1, $2, 0, NOW())
             ON CONFLICT (key) DO UPDATE
             SET limit_value = EXCLUDED.limit_value,
                 updated_at = NOW()"
        )
        .bind(flag_key)
        .bind(if entry.enabled { 1i32 } else { 0i32 })
        .execute(db)
        .await?;

        Ok(())
    }

    if let Some(ref e) = payload.auth_ip {
        upsert(&state.db, "auth_ip_per_minute", "auth_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.auth_user {
        upsert(&state.db, "auth_user_per_minute", "auth_user_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.register_ip {
        upsert(&state.db, "register_ip_per_minute", "register_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.password_reset_ip {
        upsert(&state.db, "password_reset_ip_per_minute", "password_reset_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    if let Some(ref e) = payload.websocket_ip {
        upsert(&state.db, "websocket_ip_per_minute", "websocket_ip_enabled", e).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }

    // Hot-reload — new limits take effect immediately for subsequent requests
    if let Err(e) = state.rate_limit.reload().await {
        tracing::warn!(error = %e, "Rate limit hot-reload failed after admin update");
    }

    let limits = state.rate_limit.ip_limits().await;
    Ok(Json(limits_to_response(&limits)))
}
```

- [ ] **Step 4: Register the new routes in `pub fn router()` in `admin.rs`**

In the existing `Router::new()` chain in `admin.rs::router()`, add:

```rust
.route("/admin/rate-limits", get(get_rate_limits).put(update_rate_limits))
```

Add `use axum::routing::get;` if it's not already imported.

- [ ] **Step 5: Check if admin router is registered in `v4/mod.rs`**

Search for `admin::router()` in `backend/src/api/v4/mod.rs`. If already merged, skip this step. If not, add:

```rust
.merge(admin::router().layer(DefaultBodyLimit::max(small_limit)))
```

- [ ] **Step 6: Verify it compiles**

```bash
cd backend && cargo check
```
Expected: Clean.

- [ ] **Step 7: Commit**

```bash
git add backend/src/api/v4/admin.rs backend/src/api/v4/mod.rs
git commit -m "feat(admin): add GET/PUT /admin/rate-limits with hot-reload"
```

---

## Task 10: Final verification

- [ ] **Step 1: Full clean build**

```bash
cd backend && cargo build 2>&1 | grep -E "^error"
```
Expected: No errors.

- [ ] **Step 2: Run all tests**

```bash
cd backend && cargo test 2>&1 | tail -20
```
Expected: All tests pass. If any fail, investigate and fix before proceeding.

- [ ] **Step 3: Manual smoke test (if a running stack is available)**

```bash
# Confirm 429 is returned after exceeding auth IP limit (default: 20/min)
for i in {1..25}; do
  code=$(curl -s -o /dev/null -w "%{http_code}" \
    -X POST http://localhost:3000/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"nobody@test.com","password":"wrong"}')
  echo "Request $i: $code"
done
# Expected: first ~20 return 401, remaining return 429

# Confirm Retry-After header is present on 429
curl -v -X POST http://localhost:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"nobody@test.com","password":"wrong"}' 2>&1 | grep -i "retry-after"
# Expected: Retry-After: <seconds>

# Confirm admin GET works
curl -H "Authorization: Bearer <admin_token>" \
  http://localhost:3000/api/v4/admin/rate-limits
# Expected: JSON with 5 limit entries

# Confirm admin PUT updates and hot-reloads
curl -X PUT -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{"auth_ip": {"limit": 3, "window_secs": 60, "enabled": true}}' \
  http://localhost:3000/api/v4/admin/rate-limits
# Expected: returns updated config; subsequent >3 login attempts 429 quickly
```

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat(rate-limit): P1-1 and P1-2 complete — real rate limiting with admin hot-reload"
```

---

## Troubleshooting

**Lua script returns `Vec<i64>` not `Vec<u64>`:** deadpool-redis deserializes Redis integer arrays as signed. Use `Vec<i64>` and cast to `u64` after.

**`AppError::RateLimitExceeded` variant mismatch:** When changing from `RateLimitExceeded(String)` to a struct variant, search the whole codebase with `grep -rn "RateLimitExceeded"` and update every match/construct site.

**`AppError::Internal` type:** Check `src/error.rs` — it may be `AppError::Internal(String)` or `AppError::InternalError`. Use `.to_string()` to convert from other error types.

**`router()` is sync but we need async `reload()`:** Do NOT call `reload()` inside `router()`. Always call it in `main.rs` before passing the `Arc<RateLimitService>` into `router()`.

**`ConnectInfo` not in scope:** Add `use axum::extract::ConnectInfo;` and `use std::net::SocketAddr;`.

**Admin routes not showing up:** Verify `admin::router()` is merged inside `router_with_body_limits` in `v4/mod.rs`. Check there's no route prefix mismatch — routes defined as `/admin/rate-limits` will be served at `/api/v4/admin/rate-limits`.
