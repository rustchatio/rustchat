# MCP Server Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Model Context Protocol (MCP) server support enabling AI assistants to securely access RustChat data via OAuth 2.0 authorization.

**Architecture:** Integrated MCP module within RustChat backend using OAuth 2.0 + PKCE for authorization, JSON-RPC 2.0 for protocol, SSE for real-time notifications, and comprehensive audit logging.

**Tech Stack:** Rust, Axum, PostgreSQL, Redis, SQLx, OAuth 2.0, JSON-RPC 2.0, SSE, bcrypt, SHA-256

---

## File Structure

### New Files to Create

**Database Migrations:**
- `backend/migrations/20260318000001_add_mcp_oauth_tables.sql` - OAuth clients, codes, tokens, scopes
- `backend/migrations/20260318000002_add_mcp_audit_tables.sql` - MCP audit logs

**OAuth Module (`backend/src/oauth/`):**
- `backend/src/oauth/mod.rs` - Module exports
- `backend/src/oauth/types.rs` - OAuth data types (ClientType, GrantType, etc.)
- `backend/src/oauth/authorization.rs` - Authorization endpoints (GET/POST /authorize)
- `backend/src/oauth/token.rs` - Token endpoint (POST /token for code exchange & refresh)
- `backend/src/oauth/client.rs` - OAuth client service (registration, management)
- `backend/src/oauth/utils.rs` - Helper functions (generate tokens, hash, validate PKCE)

**MCP Module (`backend/src/mcp/`):**
- `backend/src/mcp/mod.rs` - Module exports
- `backend/src/mcp/protocol.rs` - JSON-RPC 2.0 types and error codes
- `backend/src/mcp/router.rs` - MCP method dispatcher
- `backend/src/mcp/audit.rs` - MCP audit logging service
- `backend/src/mcp/resources/mod.rs` - Resource providers module
- `backend/src/mcp/resources/types.rs` - Resource types (McpResource, McpResourceContents, etc.)
- `backend/src/mcp/resources/handler.rs` - Main resource handler (list, read, subscribe)
- `backend/src/mcp/resources/messages.rs` - Messages resource provider
- `backend/src/mcp/resources/channels.rs` - Channels resource provider
- `backend/src/mcp/resources/users.rs` - Users resource provider
- `backend/src/mcp/resources/files.rs` - Files resource provider
- `backend/src/mcp/resources/teams.rs` - Teams resource provider
- `backend/src/mcp/resources/search.rs` - Search resource provider
- `backend/src/mcp/sse/mod.rs` - SSE module exports
- `backend/src/mcp/sse/types.rs` - SSE event types and connection metadata
- `backend/src/mcp/sse/manager.rs` - SSE connection manager
- `backend/src/mcp/sse/notifier.rs` - Resource update notifier
- `backend/src/mcp/sse/keepalive.rs` - Keepalive background task

**API Endpoints (`backend/src/api/`):**
- `backend/src/api/oauth.rs` - OAuth endpoint routes
- `backend/src/api/mcp.rs` - MCP endpoint routes (JSON-RPC, SSE)
- `backend/src/api/admin/oauth_clients.rs` - Admin OAuth client management
- `backend/src/api/admin/mcp_audit.rs` - Admin MCP audit endpoints
- `backend/src/api/v1/oauth.rs` - User OAuth management endpoints

### Files to Modify

- `backend/src/auth/extractors.rs` - Add McpAuth extractor (replace stub)
- `backend/src/services/rate_limit.rs` - Add check_mcp_rate_limit method
- `backend/src/services/message.rs` - Add ResourceNotifier integration
- `backend/src/services/channel.rs` - Add ResourceNotifier integration
- `backend/src/api/mod.rs` - Add AppState fields for MCP services
- `backend/src/api/routes.rs` - Register OAuth and MCP routes
- `backend/src/config.rs` - Add OAuth and MCP configuration
- `backend/src/main.rs` - Initialize MCP services and background jobs
- `backend/src/lib.rs` - Export oauth and mcp modules
- `backend/src/jobs/mod.rs` - Add OAuth code cleanup and audit cleanup jobs

### Tests to Create

- `backend/tests/test_oauth_flow.rs` - OAuth authorization code flow tests
- `backend/tests/test_mcp_protocol.rs` - MCP JSON-RPC protocol tests
- `backend/tests/test_mcp_resources.rs` - MCP resource provider tests
- `backend/tests/test_mcp_sse.rs` - SSE connection and notification tests

---

## Task 1: Database Schema - OAuth Tables

**Files:**
- Create: `backend/migrations/20260318000001_add_mcp_oauth_tables.sql`

- [ ] **Step 1: Write migration for OAuth tables**

Create `backend/migrations/20260318000001_add_mcp_oauth_tables.sql`:

```sql
-- OAuth Clients table
CREATE TABLE oauth_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) UNIQUE NOT NULL,
    client_secret_hash TEXT NOT NULL,
    client_name VARCHAR(255) NOT NULL,
    client_type VARCHAR(50) NOT NULL CHECK (client_type IN ('confidential', 'public')),
    redirect_uris TEXT[] NOT NULL,
    allowed_scopes TEXT[] NOT NULL,
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    is_first_party BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_oauth_clients_owner ON oauth_clients(owner_user_id);
CREATE INDEX idx_oauth_clients_client_id ON oauth_clients(client_id);

-- OAuth Authorization Codes table
CREATE TABLE oauth_authorization_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(255) UNIQUE NOT NULL,
    client_id VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    scopes TEXT[] NOT NULL,
    code_challenge VARCHAR(255),
    code_challenge_method VARCHAR(10),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT fk_client FOREIGN KEY (client_id)
        REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);

CREATE INDEX idx_oauth_codes_code ON oauth_authorization_codes(code);
CREATE INDEX idx_oauth_codes_expires ON oauth_authorization_codes(expires_at);

-- OAuth Access Tokens table
CREATE TABLE oauth_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_prefix VARCHAR(20) NOT NULL,
    token_hash TEXT UNIQUE NOT NULL,
    refresh_token_hash TEXT UNIQUE,
    client_id VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scopes TEXT[] NOT NULL,
    token_type VARCHAR(50) DEFAULT 'Bearer',
    expires_at TIMESTAMPTZ NOT NULL,
    refresh_expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,

    CONSTRAINT fk_client FOREIGN KEY (client_id)
        REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);

CREATE INDEX idx_oauth_tokens_prefix ON oauth_access_tokens (token_prefix)
    WHERE revoked_at IS NULL AND expires_at > NOW();
CREATE INDEX idx_oauth_tokens_user ON oauth_access_tokens(user_id);
CREATE INDEX idx_oauth_tokens_client ON oauth_access_tokens(client_id);
CREATE INDEX idx_oauth_tokens_expires ON oauth_access_tokens(expires_at)
    WHERE revoked_at IS NULL;

-- OAuth Scopes table
CREATE TABLE oauth_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed initial OAuth scopes
INSERT INTO oauth_scopes (scope, display_name, description, category) VALUES
    ('read:messages', 'Read Messages', 'Read messages from channels you have access to', 'messages'),
    ('read:channels', 'Read Channels', 'Read channel information and metadata', 'channels'),
    ('read:users', 'Read Users', 'Read user profiles and information', 'users'),
    ('read:files', 'Read Files', 'Read and download files', 'files'),
    ('read:teams', 'Read Teams', 'Read team/workspace information', 'teams'),
    ('read:search', 'Search', 'Search across messages, channels, and users', 'search');
```

- [ ] **Step 2: Verify migration syntax**

Run: `cd backend && sqlx migrate add oauth_tables --source migrations`
Expected: Migration file created

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/20260318000001_add_mcp_oauth_tables.sql
git commit -m "feat(mcp): add OAuth database schema

- OAuth clients table with bcrypt secret hashing
- Authorization codes table with PKCE support
- Access tokens table with prefix indexing
- OAuth scopes table with seed data"
```

---

## Task 2: Database Schema - MCP Audit Tables

**Files:**
- Create: `backend/migrations/20260318000002_add_mcp_audit_tables.sql`

- [ ] **Step 1: Write migration for MCP audit table**

Create `backend/migrations/20260318000002_add_mcp_audit_tables.sql`:

```sql
-- MCP Audit Logs table
CREATE TABLE mcp_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id VARCHAR(255) NOT NULL,
    method VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id VARCHAR(255),
    scopes_used TEXT[],
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'error')),
    error_message TEXT,
    request_duration_ms INTEGER
);

CREATE INDEX idx_mcp_audit_timestamp ON mcp_audit_logs(timestamp);
CREATE INDEX idx_mcp_audit_user ON mcp_audit_logs(user_id);
CREATE INDEX idx_mcp_audit_client ON mcp_audit_logs(client_id);
CREATE INDEX idx_mcp_audit_resource_type ON mcp_audit_logs(resource_type);
```

- [ ] **Step 2: Verify migration syntax**

Run: `cd backend && sqlx migrate add mcp_audit --source migrations`
Expected: Migration file created

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/20260318000002_add_mcp_audit_tables.sql
git commit -m "feat(mcp): add MCP audit logging schema

- Comprehensive audit logs for all MCP access
- Indexed by timestamp, user, client, resource type
- Supports 90-day retention policy"
```

---

## Task 3: OAuth Types and Utilities

**Files:**
- Create: `backend/src/oauth/mod.rs`
- Create: `backend/src/oauth/types.rs`
- Create: `backend/src/oauth/utils.rs`

- [ ] **Step 1: Create OAuth module structure**

Create `backend/src/oauth/mod.rs`:

```rust
pub mod types;
pub mod authorization;
pub mod token;
pub mod client;
pub mod utils;

pub use types::*;
pub use authorization::{authorize, authorize_post};
pub use token::{token, revoke, introspect};
pub use client::OAuthClientService;
pub use utils::*;
```

- [ ] **Step 2: Define OAuth types**

Create `backend/src/oauth/types.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "client_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Confidential,
    Public,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "grant_type", rename_all = "snake_case")]
pub enum TokenRequest {
    AuthorizationCode {
        code: String,
        redirect_uri: String,
        client_id: String,
        client_secret: Option<String>,
        code_verifier: String,
    },
    RefreshToken {
        refresh_token: String,
        client_id: String,
        client_secret: Option<String>,
    },
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Debug, Serialize)]
pub struct TokenErrorResponse {
    pub error: String,
    pub error_description: String,
}

#[derive(Debug)]
pub struct OAuthClient {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret_hash: String,
    pub client_name: String,
    pub client_type: ClientType,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub owner_user_id: Uuid,
    pub is_active: bool,
    pub is_first_party: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AuthorizationCode {
    pub id: Uuid,
    pub code: String,
    pub client_id: String,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AccessToken {
    pub id: Uuid,
    pub token_prefix: String,
    pub token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub client_id: String,
    pub user_id: Uuid,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub refresh_expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}
```

- [ ] **Step 3: Implement OAuth utility functions**

Create `backend/src/oauth/utils.rs`:

```rust
use crate::error::{ApiResult, AppError};
use rand::Rng;
use sha2::{Digest, Sha256};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generate random hex string
pub fn generate_random_hex(length: usize) -> String {
    let bytes: Vec<u8> = (0..length).map(|_| rand::thread_rng().gen()).collect();
    hex::encode(bytes)
}

/// Generate OAuth access token (rct_ + 64 hex = 68 chars)
pub fn generate_access_token() -> String {
    format!("rct_{}", generate_random_hex(32))
}

/// Generate OAuth refresh token (rcr_ + 64 hex = 68 chars)
pub fn generate_refresh_token() -> String {
    format!("rcr_{}", generate_random_hex(32))
}

/// Generate authorization code (64 hex chars)
pub fn generate_authorization_code() -> String {
    generate_random_hex(32)
}

/// Generate OAuth client ID (mcp_ + 32 hex = 36 chars)
pub fn generate_client_id() -> String {
    format!("mcp_{}", generate_random_hex(16))
}

/// Generate OAuth client secret (mcs_ + 48 hex = 52 chars)
pub fn generate_client_secret() -> String {
    format!("mcs_{}", generate_random_hex(24))
}

/// Extract token prefix (first 20 chars)
pub fn extract_token_prefix(token: &str) -> ApiResult<String> {
    if token.len() < 20 {
        return Err(AppError::BadRequest("Invalid token format".to_string()));
    }
    Ok(token[..20].to_string())
}

/// Hash token with SHA-256
pub fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validate PKCE code verifier against challenge
pub fn validate_pkce(
    code_verifier: &str,
    code_challenge: &str,
    method: &str,
) -> ApiResult<()> {
    if method != "S256" {
        return Err(AppError::BadRequest(
            "Only S256 code challenge method is supported".to_string(),
        ));
    }

    // Compute SHA-256 of verifier
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let computed = URL_SAFE_NO_PAD.encode(hasher.finalize());

    // Compare with challenge
    if computed != code_challenge {
        return Err(AppError::Unauthorized("Invalid code verifier".to_string()));
    }

    Ok(())
}

/// Validate redirect URI
pub fn is_valid_redirect_uri(uri: &str) -> bool {
    uri.starts_with("https://")
        || uri.starts_with("http://localhost")
        || uri.starts_with("http://127.0.0.1")
}

/// Constant-time comparison (prevents timing attacks)
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_tokens() {
        let access = generate_access_token();
        assert_eq!(access.len(), 68);
        assert!(access.starts_with("rct_"));

        let refresh = generate_refresh_token();
        assert_eq!(refresh.len(), 68);
        assert!(refresh.starts_with("rcr_"));
    }

    #[test]
    fn test_extract_prefix() {
        let token = "rct_a1b2c3d4e5f6g7h890123456789abcdef";
        let prefix = extract_token_prefix(token).unwrap();
        assert_eq!(prefix, "rct_a1b2c3d4e5f6g7h8");
    }

    #[test]
    fn test_validate_pkce() {
        let verifier = "test_verifier_12345";
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        assert!(validate_pkce(verifier, &challenge, "S256").is_ok());
        assert!(validate_pkce("wrong_verifier", &challenge, "S256").is_err());
        assert!(validate_pkce(verifier, &challenge, "plain").is_err());
    }

    #[test]
    fn test_validate_redirect_uri() {
        assert!(is_valid_redirect_uri("https://example.com/callback"));
        assert!(is_valid_redirect_uri("http://localhost:8080/callback"));
        assert!(is_valid_redirect_uri("http://127.0.0.1:3000/callback"));
        assert!(!is_valid_redirect_uri("http://example.com/callback"));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test oauth::utils`
Expected: All 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add backend/src/oauth/
git commit -m "feat(mcp): add OAuth types and utility functions

- OAuth data types (ClientType, TokenRequest, etc.)
- Token generation (access, refresh, codes)
- PKCE validation with SHA-256
- Redirect URI validation
- Unit tests for all utilities"
```

---

## Task 4: OAuth Client Service

**Files:**
- Create: `backend/src/oauth/client.rs`

- [ ] **Step 1: Implement OAuth client service**

Create `backend/src/oauth/client.rs`:

```rust
use crate::error::{ApiResult, AppError};
use crate::oauth::types::{ClientType, OAuthClient};
use crate::oauth::utils::{generate_client_id, generate_client_secret, is_valid_redirect_uri};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::PgPool;
use uuid::Uuid;

pub struct OAuthClientService {
    db: PgPool,
}

impl OAuthClientService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Register a new OAuth client
    pub async fn register_client(
        &self,
        owner_user_id: Uuid,
        client_name: String,
        client_type: ClientType,
        redirect_uris: Vec<String>,
        allowed_scopes: Vec<String>,
        is_first_party: bool,
    ) -> ApiResult<RegisteredClient> {
        // Validate redirect URIs
        for uri in &redirect_uris {
            if !is_valid_redirect_uri(uri) {
                return Err(AppError::BadRequest(format!("Invalid redirect URI: {}", uri)));
            }
        }

        // Validate scopes
        self.validate_scopes(&allowed_scopes).await?;

        // Generate credentials
        let client_id = generate_client_id();
        let client_secret = generate_client_secret();
        let client_secret_hash = hash(&client_secret, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("Hash error: {}", e)))?;

        // Insert into database
        let client = sqlx::query_as!(
            OAuthClient,
            r#"
            INSERT INTO oauth_clients (
                client_id, client_secret_hash, client_name, client_type,
                redirect_uris, allowed_scopes, owner_user_id, is_first_party
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, client_id, client_secret_hash, client_name,
                      client_type as "client_type: ClientType",
                      redirect_uris, allowed_scopes, owner_user_id,
                      is_active, is_first_party, created_at, updated_at
            "#,
            client_id,
            client_secret_hash,
            client_name,
            client_type as ClientType,
            &redirect_uris,
            &allowed_scopes,
            owner_user_id,
            is_first_party
        )
        .fetch_one(&self.db)
        .await?;

        Ok(RegisteredClient {
            id: client.id,
            client_id,
            client_secret, // Only returned once
            client_name,
            client_type,
            redirect_uris,
            allowed_scopes,
            is_first_party,
            created_at: client.created_at,
        })
    }

    /// Get OAuth client by client_id
    pub async fn get_client(&self, client_id: &str) -> ApiResult<OAuthClient> {
        sqlx::query_as!(
            OAuthClient,
            r#"
            SELECT id, client_id, client_secret_hash, client_name,
                   client_type as "client_type: ClientType",
                   redirect_uris, allowed_scopes, owner_user_id,
                   is_active, is_first_party, created_at, updated_at
            FROM oauth_clients
            WHERE client_id = $1
            "#,
            client_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("OAuth client not found".to_string()))
    }

    /// Validate scopes exist
    async fn validate_scopes(&self, scopes: &[String]) -> ApiResult<()> {
        let valid_scopes = sqlx::query_scalar!(
            "SELECT scope FROM oauth_scopes"
        )
        .fetch_all(&self.db)
        .await?;

        for scope in scopes {
            if !valid_scopes.contains(scope) {
                return Err(AppError::BadRequest(format!("Invalid scope: {}", scope)));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct RegisteredClient {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret: String,
    pub client_name: String,
    pub client_type: ClientType,
    pub redirect_uris: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub is_first_party: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

- [ ] **Step 2: Commit**

```bash
git add backend/src/oauth/client.rs
git commit -m "feat(mcp): add OAuth client service

- Client registration with validation
- Redirect URI and scope validation
- Bcrypt secret hashing
- Client lookup by client_id"
```

---

## Task 5: OAuth Authorization Endpoints

**Files:**
- Create: `backend/src/oauth/authorization.rs`

- [ ] **Step 1: Implement authorization endpoints**

Create `backend/src/oauth/authorization.rs`:

```rust
use crate::auth::extractors::JwtAuth;
use crate::error::{ApiResult, AppError};
use crate::oauth::client::OAuthClientService;
use crate::oauth::utils::{generate_authorization_code, is_valid_redirect_uri, validate_pkce};
use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    Json,
};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
}

#[derive(Deserialize)]
pub struct AuthorizeRequest {
    pub pending_key: String,
    pub user_consent: bool,
}

#[derive(Serialize)]
struct PendingAuthorization {
    client_id: String,
    user_id: String,
    redirect_uri: String,
    scopes: Vec<String>,
    code_challenge: String,
    code_challenge_method: String,
    state: String,
}

/// GET /api/oauth/authorize - Display consent page
pub async fn authorize(
    auth: JwtAuth,
    Query(params): Query<AuthorizeQuery>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Html<String>> {
    // Validate response_type
    if params.response_type != "code" {
        return Err(AppError::BadRequest("Only 'code' response_type is supported".to_string()));
    }

    // Validate PKCE method
    if params.code_challenge_method != "S256" {
        return Err(AppError::BadRequest("Only 'S256' code_challenge_method is supported".to_string()));
    }

    // Get and validate client
    let client = state.oauth_client_service.get_client(&params.client_id).await?;

    if !client.is_active {
        return Err(AppError::BadRequest("Client is not active".to_string()));
    }

    // Validate redirect URI
    if !client.redirect_uris.contains(&params.redirect_uri) {
        return Err(AppError::BadRequest("Invalid redirect_uri".to_string()));
    }

    // Parse and validate scopes
    let requested_scopes: Vec<String> = params.scope.split_whitespace().map(String::from).collect();
    for scope in &requested_scopes {
        if !client.allowed_scopes.contains(scope) {
            return Err(AppError::BadRequest(format!("Scope '{}' not allowed for this client", scope)));
        }
    }

    // Store pending authorization in Redis (10 minute expiry)
    let pending_key = generate_random_hex(16);
    let pending = PendingAuthorization {
        client_id: params.client_id.clone(),
        user_id: auth.user_id.to_string(),
        redirect_uri: params.redirect_uri.clone(),
        scopes: requested_scopes.clone(),
        code_challenge: params.code_challenge.clone(),
        code_challenge_method: params.code_challenge_method.clone(),
        state: params.state.clone(),
    };

    let mut redis_conn = state.redis.get().await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    redis_conn
        .set_ex(
            format!("oauth:pending:{}", pending_key),
            serde_json::to_string(&pending)?,
            600, // 10 minutes
        )
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    // Get scope descriptions
    let scope_descriptions = fetch_scope_descriptions(&state.db, &requested_scopes).await?;

    // Render consent page
    let html = render_consent_page(&pending_key, &client.client_name, &scope_descriptions);

    Ok(Html(html))
}

/// POST /api/oauth/authorize - Process user consent
pub async fn authorize_post(
    auth: JwtAuth,
    State(state): State<Arc<AppState>>,
    Json(req): Json<AuthorizeRequest>,
) -> ApiResult<Redirect> {
    // Retrieve pending authorization
    let mut redis_conn = state.redis.get().await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    let pending_json: Option<String> = redis_conn
        .get(format!("oauth:pending:{}", req.pending_key))
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    let pending: PendingAuthorization = serde_json::from_str(
        &pending_json.ok_or_else(|| AppError::BadRequest("Invalid or expired pending authorization".to_string()))?
    )?;

    // Verify same user
    if pending.user_id != auth.user_id.to_string() {
        return Err(AppError::Forbidden("Authorization mismatch".to_string()));
    }

    // Delete pending authorization
    let _: () = redis_conn
        .del(format!("oauth:pending:{}", req.pending_key))
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    if req.user_consent {
        // User granted consent - generate authorization code
        let code = generate_authorization_code();

        // Store code in database (10 minute expiry)
        sqlx::query!(
            r#"
            INSERT INTO oauth_authorization_codes
            (code, client_id, user_id, redirect_uri, scopes, code_challenge, code_challenge_method, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW() + INTERVAL '10 minutes')
            "#,
            code,
            pending.client_id,
            uuid::Uuid::parse_str(&pending.user_id)?,
            pending.redirect_uri,
            &pending.scopes,
            pending.code_challenge,
            pending.code_challenge_method
        )
        .execute(&state.db)
        .await?;

        // Redirect with code
        Ok(Redirect::to(&format!(
            "{}?code={}&state={}",
            pending.redirect_uri, code, pending.state
        )))
    } else {
        // User denied consent
        Ok(Redirect::to(&format!(
            "{}?error=access_denied&error_description=User+denied+consent&state={}",
            pending.redirect_uri, pending.state
        )))
    }
}

fn render_consent_page(pending_key: &str, client_name: &str, scopes: &[(String, String)]) -> String {
    let scope_list = scopes
        .iter()
        .map(|(name, desc)| format!("<li><strong>{}</strong><br><small>{}</small></li>", name, desc))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorize Application</title>
    <style>
        body {{ font-family: sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
        h1 {{ color: #333; }}
        ul {{ list-style: none; padding: 0; }}
        li {{ margin: 10px 0; padding: 10px; background: #f5f5f5; border-radius: 5px; }}
        button {{ padding: 10px 20px; margin: 5px; font-size: 16px; cursor: pointer; }}
        .authorize {{ background: #4CAF50; color: white; border: none; }}
        .deny {{ background: #f44336; color: white; border: none; }}
    </style>
</head>
<body>
    <h1>Authorize Access</h1>
    <p><strong>{}</strong> wants to access your RustChat data:</p>
    <ul>
        {}
    </ul>
    <form method="POST" action="/api/oauth/authorize">
        <input type="hidden" name="pending_key" value="{}">
        <button type="submit" name="user_consent" value="true" class="authorize">Authorize</button>
        <button type="submit" name="user_consent" value="false" class="deny">Deny</button>
    </form>
</body>
</html>"#,
        client_name, scope_list, pending_key
    )
}

async fn fetch_scope_descriptions(db: &PgPool, scopes: &[String]) -> ApiResult<Vec<(String, String)>> {
    let mut descriptions = Vec::new();
    for scope in scopes {
        let row = sqlx::query!(
            "SELECT display_name, description FROM oauth_scopes WHERE scope = $1",
            scope
        )
        .fetch_one(db)
        .await?;
        descriptions.push((row.display_name, row.description));
    }
    Ok(descriptions)
}

use crate::oauth::utils::generate_random_hex;
use sqlx::PgPool;

struct AppState {
    db: PgPool,
    redis: deadpool_redis::Pool,
    oauth_client_service: Arc<OAuthClientService>,
}
```

- [ ] **Step 2: Commit**

```bash
git add backend/src/oauth/authorization.rs
git commit -m "feat(mcp): add OAuth authorization endpoints

- GET /authorize displays consent page
- POST /authorize processes user decision
- Redis-based pending authorization (10 min expiry)
- PKCE parameter validation
- Redirect URI validation"
```

---

*Due to length constraints, I'll continue with the remaining tasks in a structured format. The plan continues with:*

- Task 6: OAuth Token Endpoint
- Task 7: MCP Protocol Types
- Task 8: MCP Auth Extractor
- Task 9-14: MCP Resource Providers
- Task 15: SSE Connection Manager
- Task 16: MCP Audit Service
- Task 17: Admin API Endpoints
- Task 18: Integration & Routes
- Task 19: Background Jobs
- Task 20: Integration Tests

Would you like me to continue with the complete detailed plan for all remaining tasks?
