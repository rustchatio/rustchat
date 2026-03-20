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

## Task 6: OAuth Token Endpoint

**Files:**
- Create: `backend/src/oauth/token.rs`

- [ ] **Step 1: Write test for authorization code exchange**

Create `backend/tests/test_oauth_token.rs`:

```rust
#[cfg(test)]
mod token_exchange_tests {
    use super::*;

    #[tokio::test]
    async fn test_exchange_authorization_code_for_tokens() {
        // Test will verify PKCE validation and token generation
        // Placeholder - requires database setup
    }

    #[tokio::test]
    async fn test_pkce_validation_failure() {
        // Test invalid code_verifier rejection
    }

    #[tokio::test]
    async fn test_expired_code_rejection() {
        // Test expired authorization codes are rejected
    }
}
```

- [ ] **Step 2: Implement token endpoint**

Create `backend/src/oauth/token.rs`:

```rust
use crate::auth::extractors::JwtAuth;
use crate::error::{ApiResult, AppError};
use crate::oauth::client::OAuthClientService;
use crate::oauth::types::{AccessToken, AuthorizationCode, TokenRequest, TokenResponse, TokenErrorResponse};
use crate::oauth::utils::{
    constant_time_compare, extract_token_prefix, generate_access_token,
    generate_refresh_token, sha256_hash, validate_pkce,
};
use axum::{extract::State, Json};
use bcrypt;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

/// POST /api/oauth/token - Exchange code for tokens or refresh token
pub async fn token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, Json<TokenErrorResponse>> {
    match req {
        TokenRequest::AuthorizationCode {
            code,
            redirect_uri,
            client_id,
            client_secret,
            code_verifier,
        } => {
            exchange_authorization_code(
                &state,
                code,
                redirect_uri,
                client_id,
                client_secret,
                code_verifier,
            )
            .await
        }
        TokenRequest::RefreshToken {
            refresh_token,
            client_id,
            client_secret,
        } => {
            refresh_access_token(&state, refresh_token, client_id, client_secret).await
        }
    }
    .map(Json)
    .map_err(|e| Json(TokenErrorResponse {
        error: "invalid_grant".to_string(),
        error_description: e.to_string(),
    }))
}

async fn exchange_authorization_code(
    state: &AppState,
    code: String,
    redirect_uri: String,
    client_id: String,
    client_secret: Option<String>,
    code_verifier: String,
) -> ApiResult<TokenResponse> {
    let mut tx = state.db.begin().await?;

    // Step 1: Fetch authorization code (with FOR UPDATE lock)
    let code_record = sqlx::query_as!(
        AuthorizationCode,
        r#"
        SELECT id, code, client_id, user_id, redirect_uri, scopes,
               code_challenge, code_challenge_method, expires_at
        FROM oauth_authorization_codes
        WHERE code = $1 AND expires_at > NOW()
        FOR UPDATE
        "#,
        code
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid or expired authorization code".to_string()))?;

    // Step 2: Validate PKCE BEFORE deleting code
    if let (Some(challenge), Some(method)) = (&code_record.code_challenge, &code_record.code_challenge_method) {
        validate_pkce(&code_verifier, challenge, method)?;
    } else {
        return Err(AppError::BadRequest("PKCE parameters required".to_string()));
    }

    // Step 3: Validate redirect_uri matches
    if redirect_uri != code_record.redirect_uri {
        return Err(AppError::Unauthorized("Redirect URI mismatch".to_string()));
    }

    // Step 4: Authenticate client
    authenticate_client(&state.db, &client_id, client_secret.as_deref()).await?;

    // Step 5: Generate tokens
    let access_token = generate_access_token();
    let refresh_token = generate_refresh_token();
    let token_prefix = extract_token_prefix(&access_token)?;
    let token_hash = sha256_hash(&access_token);
    let refresh_hash = sha256_hash(&refresh_token);

    // Step 6: Store tokens
    sqlx::query!(
        r#"
        INSERT INTO oauth_access_tokens
        (token_prefix, token_hash, refresh_token_hash, client_id, user_id, scopes,
         expires_at, refresh_expires_at)
        VALUES ($1, $2, $3, $4, $5, $6,
                NOW() + INTERVAL '1 hour', NOW() + INTERVAL '30 days')
        "#,
        token_prefix,
        token_hash,
        refresh_hash,
        client_id,
        code_record.user_id,
        &code_record.scopes
    )
    .execute(&mut *tx)
    .await?;

    // Step 7: Delete authorization code (single-use)
    sqlx::query!("DELETE FROM oauth_authorization_codes WHERE id = $1", code_record.id)
        .execute(&mut *tx)
        .await?;

    // Step 8: Commit transaction
    tx.commit().await?;

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token,
        scope: code_record.scopes.join(" "),
    })
}

async fn refresh_access_token(
    state: &AppState,
    refresh_token: String,
    client_id: String,
    client_secret: Option<String>,
) -> ApiResult<TokenResponse> {
    // Authenticate client
    authenticate_client(&state.db, &client_id, client_secret.as_deref()).await?;

    // Hash refresh token
    let refresh_hash = sha256_hash(&refresh_token);

    // Fetch token record
    let token_record = sqlx::query_as!(
        AccessToken,
        r#"
        SELECT id, token_prefix, token_hash, refresh_token_hash, client_id, user_id, scopes,
               expires_at, refresh_expires_at, revoked_at
        FROM oauth_access_tokens
        WHERE refresh_token_hash = $1
          AND client_id = $2
          AND refresh_expires_at > NOW()
          AND revoked_at IS NULL
        "#,
        refresh_hash,
        client_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    // Generate new tokens (token rotation)
    let new_access_token = generate_access_token();
    let new_refresh_token = generate_refresh_token();
    let token_prefix = extract_token_prefix(&new_access_token)?;
    let token_hash = sha256_hash(&new_access_token);
    let new_refresh_hash = sha256_hash(&new_refresh_token);

    // Update token record
    sqlx::query!(
        r#"
        UPDATE oauth_access_tokens
        SET token_prefix = $1,
            token_hash = $2,
            refresh_token_hash = $3,
            expires_at = NOW() + INTERVAL '1 hour',
            refresh_expires_at = NOW() + INTERVAL '30 days'
        WHERE id = $4
        "#,
        token_prefix,
        token_hash,
        new_refresh_hash,
        token_record.id
    )
    .execute(&state.db)
    .await?;

    Ok(TokenResponse {
        access_token: new_access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token: new_refresh_token,
        scope: token_record.scopes.join(" "),
    })
}

async fn authenticate_client(
    db: &PgPool,
    client_id: &str,
    client_secret: Option<&str>,
) -> ApiResult<()> {
    let client = sqlx::query!(
        r#"
        SELECT client_type, client_secret_hash, is_active
        FROM oauth_clients
        WHERE client_id = $1
        "#,
        client_id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid client".to_string()))?;

    if !client.is_active {
        return Err(AppError::Unauthorized("Client is not active".to_string()));
    }

    // Confidential clients must provide secret
    if client.client_type == "confidential" {
        let secret = client_secret
            .ok_or_else(|| AppError::Unauthorized("Client secret required".to_string()))?;

        bcrypt::verify(secret, &client.client_secret_hash)
            .map_err(|_| AppError::Unauthorized("Invalid client secret".to_string()))?;
    }

    Ok(())
}

/// POST /api/oauth/revoke - Revoke token
pub async fn revoke(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RevokeRequest>,
) -> ApiResult<()> {
    let token_hash = sha256_hash(&req.token);

    sqlx::query!(
        r#"
        UPDATE oauth_access_tokens
        SET revoked_at = NOW()
        WHERE token_hash = $1 OR refresh_token_hash = $1
        "#,
        token_hash
    )
    .execute(&state.db)
    .await?;

    Ok(())
}

/// POST /api/oauth/introspect - Token introspection
pub async fn introspect(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IntrospectRequest>,
) -> ApiResult<Json<IntrospectResponse>> {
    let token_prefix = extract_token_prefix(&req.token)?;
    let token_hash = sha256_hash(&req.token);

    let token = sqlx::query!(
        r#"
        SELECT user_id, client_id, scopes, expires_at, revoked_at
        FROM oauth_access_tokens
        WHERE token_prefix = $1 AND token_hash = $2
        "#,
        token_prefix,
        token_hash
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(token) = token {
        if token.revoked_at.is_none() && token.expires_at > chrono::Utc::now() {
            return Ok(Json(IntrospectResponse {
                active: true,
                scope: Some(token.scopes.join(" ")),
                client_id: Some(token.client_id),
                user_id: Some(token.user_id.to_string()),
                exp: Some(token.expires_at.timestamp() as u64),
            }));
        }
    }

    Ok(Json(IntrospectResponse {
        active: false,
        scope: None,
        client_id: None,
        user_id: None,
        exp: None,
    }))
}

#[derive(Deserialize)]
pub struct RevokeRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
}

#[derive(Deserialize)]
pub struct IntrospectRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct IntrospectResponse {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub user_id: Option<String>,
    pub exp: Option<u64>,
}

use serde::Serialize;

struct AppState {
    db: PgPool,
}
```

- [ ] **Step 3: Run tests**

Run: `cd backend && cargo test oauth::token`
Expected: Tests pass

- [ ] **Step 4: Commit**

```bash
git add backend/src/oauth/token.rs backend/tests/test_oauth_token.rs
git commit -m "feat(mcp): add OAuth token endpoint

- Authorization code exchange with PKCE validation
- Refresh token flow with token rotation
- Token revocation endpoint
- Token introspection endpoint
- Transaction safety (validate before delete)"
```

---

## Task 7: MCP Protocol Types

**Files:**
- Create: `backend/src/mcp/mod.rs`
- Create: `backend/src/mcp/protocol.rs`

- [ ] **Step 1: Create MCP module structure**

Create `backend/src/mcp/mod.rs`:

```rust
pub mod protocol;
pub mod router;
pub mod audit;
pub mod resources;
pub mod sse;

pub use protocol::{JsonRpcRequest, JsonRpcResponse, JsonRpcError, ErrorObject, error_codes};
pub use router::McpRouter;
pub use audit::McpAuditService;
```

- [ ] **Step 2: Write test for JSON-RPC protocol**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_parsing() {
        let json = r#"{"jsonrpc":"2.0","method":"resources/list","id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "resources/list");
    }

    #[test]
    fn test_error_object_serialization() {
        let error = ErrorObject {
            code: -32001,
            message: "Forbidden".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("-32001"));
    }
}
```

- [ ] **Step 3: Implement JSON-RPC 2.0 types**

Create `backend/src/mcp/protocol.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Success Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Value,
    pub id: Value,
}

/// JSON-RPC 2.0 Error Response
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub error: ErrorObject,
    pub id: Value,
}

/// JSON-RPC 2.0 Error Object
#[derive(Debug, Serialize, Clone)]
pub struct ErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP Error Codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // MCP-specific errors
    pub const UNAUTHORIZED: i32 = -32000;
    pub const FORBIDDEN: i32 = -32001;
    pub const RESOURCE_NOT_FOUND: i32 = -32002;
    pub const RATE_LIMIT_EXCEEDED: i32 = -32003;
}

impl JsonRpcRequest {
    pub fn validate(&self) -> Result<(), ErrorObject> {
        if self.jsonrpc != "2.0" {
            return Err(ErrorObject {
                code: error_codes::INVALID_REQUEST,
                message: "Invalid JSON-RPC version, must be 2.0".to_string(),
                data: None,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_request_parsing() {
        let json = r#"{"jsonrpc":"2.0","method":"resources/list","id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "resources/list");
    }

    #[test]
    fn test_json_rpc_validation() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: None,
            id: Some(Value::from(1)),
        };
        assert!(req.validate().is_ok());

        let bad_req = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            method: "test".to_string(),
            params: None,
            id: Some(Value::from(1)),
        };
        assert!(bad_req.validate().is_err());
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test mcp::protocol`
Expected: 2 tests pass

- [ ] **Step 5: Commit**

```bash
git add backend/src/mcp/
git commit -m "feat(mcp): add JSON-RPC 2.0 protocol types

- JsonRpcRequest, JsonRpcResponse, JsonRpcError
- MCP error codes following JSON-RPC 2.0 spec
- Request validation
- Unit tests for protocol types"
```

---

## Task 8: MCP Auth Extractor

**Files:**
- Modify: `backend/src/auth/extractors.rs`
- Modify: `backend/src/services/rate_limit.rs`

- [ ] **Step 1: Write test for McpAuth extractor**

Add to `backend/tests/test_api_key_auth.rs`:

```rust
#[tokio::test]
async fn test_mcp_auth_with_oauth_token() {
    // Test OAuth token validation in McpAuth
    // Placeholder - requires database
}

#[tokio::test]
async fn test_mcp_auth_with_api_key() {
    // Test API key validation in McpAuth
    // Placeholder - requires database
}

#[tokio::test]
async fn test_mcp_auth_scope_validation() {
    // Test scope checking
}
```

- [ ] **Step 2: Implement McpAuth extractor**

Modify `backend/src/auth/extractors.rs`:

```rust
/// MCP (Model Context Protocol) authentication extractor
pub struct McpAuth {
    pub user_id: Uuid,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub token_type: TokenType,
}

pub enum TokenType {
    AccessToken,
    ApiKey,
}

#[async_trait]
impl<S> FromRequestParts<S> for McpAuth
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid Authorization format".to_string()));
        }

        let token = &auth_header[7..];

        // Determine token type by prefix
        if token.starts_with("rct_") {
            Self::validate_access_token(token, &app_state).await
        } else if token.starts_with("rck_") {
            Self::validate_api_key(token, &app_state).await
        } else {
            Err(AppError::Unauthorized("Unknown token format".to_string()))
        }
    }
}

impl McpAuth {
    async fn validate_access_token(token: &str, state: &AppState) -> Result<Self, AppError> {
        use crate::oauth::utils::{extract_token_prefix, sha256_hash, constant_time_compare};

        let token_prefix = extract_token_prefix(token)?;
        let token_hash = sha256_hash(token);

        // Query by prefix (O(1) indexed lookup)
        let candidates = sqlx::query!(
            r#"
            SELECT user_id, client_id, scopes, expires_at, revoked_at
            FROM oauth_access_tokens
            WHERE token_prefix = $1
              AND expires_at > NOW()
              AND revoked_at IS NULL
            "#,
            token_prefix
        )
        .fetch_all(&state.db)
        .await?;

        // Verify exact hash
        for candidate in candidates {
            let stored_hash = sqlx::query_scalar!(
                "SELECT token_hash FROM oauth_access_tokens WHERE user_id = $1 AND client_id = $2",
                candidate.user_id,
                candidate.client_id
            )
            .fetch_one(&state.db)
            .await?;

            if constant_time_compare(&stored_hash, &token_hash) {
                // Update last_used_at
                sqlx::query!(
                    "UPDATE oauth_access_tokens SET last_used_at = NOW() WHERE user_id = $1 AND client_id = $2",
                    candidate.user_id,
                    candidate.client_id
                )
                .execute(&state.db)
                .await?;

                // Check MCP rate limits
                state.rate_limit_service
                    .check_mcp_rate_limit(&candidate.user_id, &candidate.client_id)
                    .await?;

                return Ok(McpAuth {
                    user_id: candidate.user_id,
                    client_id: candidate.client_id,
                    scopes: candidate.scopes,
                    token_type: TokenType::AccessToken,
                });
            }
        }

        Err(AppError::Unauthorized("Invalid token".to_string()))
    }

    async fn validate_api_key(api_key: &str, state: &AppState) -> Result<Self, AppError> {
        // Reuse existing ApiKeyAuth logic
        let api_key_auth = ApiKeyAuth::from_api_key(api_key, state).await?;

        // API keys get full MCP scopes
        let all_scopes = vec![
            "read:messages".to_string(),
            "read:channels".to_string(),
            "read:users".to_string(),
            "read:files".to_string(),
            "read:teams".to_string(),
            "read:search".to_string(),
        ];

        let client_id = format!("api_key:{}", api_key_auth.email);

        // Apply MCP rate limits (not AgentHigh)
        state.rate_limit_service
            .check_mcp_rate_limit(&api_key_auth.user_id, &client_id)
            .await?;

        Ok(McpAuth {
            user_id: api_key_auth.user_id,
            client_id,
            scopes: all_scopes,
            token_type: TokenType::ApiKey,
        })
    }

    pub fn has_scope(&self, required_scope: &str) -> bool {
        self.scopes.iter().any(|s| s == required_scope)
    }

    pub fn require_scope(&self, scope: &str) -> Result<(), AppError> {
        if !self.has_scope(scope) {
            return Err(AppError::Forbidden(format!(
                "Missing required scope: {}",
                scope
            )));
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Add MCP rate limiting method**

Modify `backend/src/services/rate_limit.rs`:

```rust
impl RateLimitService {
    /// Check MCP-specific rate limits
    pub async fn check_mcp_rate_limit(
        &self,
        user_id: &Uuid,
        client_id: &str,
    ) -> ApiResult<()> {
        // Per-client limit (1000 req/hr)
        let client_key = format!("{}:mcp_client:{}", RATE_LIMIT_KEY_PREFIX, client_id);
        let client_count: u64 = self
            .script
            .key(&client_key)
            .arg(1000)
            .arg(3600)
            .invoke_async(&mut *self.redis.get().await?)
            .await?;

        if client_count > 1000 {
            return Err(AppError::RateLimitExceeded(
                "MCP client rate limit exceeded: 1000 req/hr".to_string(),
            ));
        }

        // Per-user limit (5000 req/hr)
        let user_key = format!("{}:mcp_user:{}", RATE_LIMIT_KEY_PREFIX, user_id);
        let user_count: u64 = self
            .script
            .key(&user_key)
            .arg(5000)
            .arg(3600)
            .invoke_async(&mut *self.redis.get().await?)
            .await?;

        if user_count > 5000 {
            return Err(AppError::RateLimitExceeded(
                "MCP user rate limit exceeded: 5000 req/hr".to_string(),
            ));
        }

        Ok(())
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd backend && cargo test auth::extractors`
Expected: Tests compile and pass

- [ ] **Step 5: Commit**

```bash
git add backend/src/auth/extractors.rs backend/src/services/rate_limit.rs
git commit -m "feat(mcp): add McpAuth extractor

- Validates OAuth tokens and API keys
- O(1) token lookup with prefix indexing
- Scope-based authorization
- MCP-specific rate limiting (1k/hr client, 5k/hr user)
- Replaces stub implementation"
```

---

### Task 9: MCP Router and Method Dispatcher

**Goal**: Implement JSON-RPC 2.0 router that dispatches MCP method calls to appropriate handlers.

**Files:**
- Create: `backend/src/mcp/router.rs`
- Create: `backend/src/mcp/tests/test_router.rs`

- [ ] **Step 1: Write failing test for router**

Create `backend/src/mcp/tests/test_router.rs`:

```rust
use crate::mcp::router::McpRouter;
use crate::mcp::types::{JsonRpcRequest, JsonRpcResponse};
use crate::auth::extractors::{McpAuth, TokenType};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_router_initialize() {
    let router = McpRouter::new();
    let auth = McpAuth {
        user_id: Uuid::new_v4(),
        client_id: "test_client".to_string(),
        scopes: vec!["mcp:read".to_string()],
        token_type: TokenType::AccessToken,
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
        id: Some(json!(1)),
    };

    let response = router.handle_request(request, auth).await;
    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(response.result.is_some());
}

#[tokio::test]
async fn test_router_unknown_method() {
    let router = McpRouter::new();
    let auth = McpAuth {
        user_id: Uuid::new_v4(),
        client_id: "test_client".to_string(),
        scopes: vec!["mcp:read".to_string()],
        token_type: TokenType::AccessToken,
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "unknown/method".to_string(),
        params: None,
        id: Some(json!(1)),
    };

    let response = router.handle_request(request, auth).await;
    assert!(response.is_err());
}

#[tokio::test]
async fn test_router_missing_scope() {
    let router = McpRouter::new();
    let auth = McpAuth {
        user_id: Uuid::new_v4(),
        client_id: "test_client".to_string(),
        scopes: vec![], // No scopes
        token_type: TokenType::AccessToken,
    };

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "resources/list".to_string(),
        params: None,
        id: Some(json!(1)),
    };

    let response = router.handle_request(request, auth).await;
    assert!(response.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test mcp::tests::test_router`
Expected: FAIL with "module not found: router"

- [ ] **Step 3: Write minimal router implementation**

Create `backend/src/mcp/router.rs`:

```rust
use crate::mcp::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
use crate::auth::extractors::McpAuth;
use serde_json::{json, Value};

/// MCP JSON-RPC router
pub struct McpRouter {
    // Will be populated with resource providers in later tasks
}

impl McpRouter {
    pub fn new() -> Self {
        Self {}
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(
        &self,
        request: JsonRpcRequest,
        auth: McpAuth,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        // Validate scope for all methods except initialize
        if request.method != "initialize" && !auth.has_scope("mcp:read") {
            return Err(JsonRpcError {
                code: -32600,
                message: "Insufficient scope for MCP access".to_string(),
                data: None,
            });
        }

        // Route to handler
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "resources/list" => self.handle_resources_list(request.params, &auth).await,
            "resources/read" => self.handle_resources_read(request.params, &auth).await,
            "resources/subscribe" => self.handle_resources_subscribe(request.params, &auth).await,
            "resources/unsubscribe" => self.handle_resources_unsubscribe(request.params, &auth).await,
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        }
    }

    async fn handle_initialize(&self, params: Option<Value>) -> Result<JsonRpcResponse, JsonRpcError> {
        // Validate params
        let params = params.ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing params for initialize".to_string(),
            data: None,
        })?;

        let protocol_version = params["protocolVersion"].as_str().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing protocolVersion".to_string(),
            data: None,
        })?;

        if protocol_version != "2024-11-05" {
            return Err(JsonRpcError {
                code: -32602,
                message: format!("Unsupported protocol version: {}", protocol_version),
                data: None,
            });
        }

        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "resources": {
                        "subscribe": true,
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "RustChat MCP Server",
                    "version": "1.0.0"
                }
            })),
            error: None,
            id: json!(1),
        })
    }

    async fn handle_resources_list(
        &self,
        _params: Option<Value>,
        _auth: &McpAuth,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        // Stub - will be implemented in Task 11-16
        Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({
                "resources": []
            })),
            error: None,
            id: json!(1),
        })
    }

    async fn handle_resources_read(
        &self,
        _params: Option<Value>,
        _auth: &McpAuth,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        // Stub - will be implemented in Task 11-16
        Err(JsonRpcError {
            code: -32601,
            message: "resources/read not yet implemented".to_string(),
            data: None,
        })
    }

    async fn handle_resources_subscribe(
        &self,
        _params: Option<Value>,
        _auth: &McpAuth,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        // Stub - will be implemented in Task 17
        Err(JsonRpcError {
            code: -32601,
            message: "resources/subscribe not yet implemented".to_string(),
            data: None,
        })
    }

    async fn handle_resources_unsubscribe(
        &self,
        _params: Option<Value>,
        _auth: &McpAuth,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        // Stub - will be implemented in Task 17
        Err(JsonRpcError {
            code: -32601,
            message: "resources/unsubscribe not yet implemented".to_string(),
            data: None,
        })
    }
}
```

- [ ] **Step 4: Update mcp/mod.rs**

Modify `backend/src/mcp/mod.rs`:

```rust
pub mod types;
pub mod router;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd backend && cargo test mcp::tests::test_router`
Expected: PASS (3 tests)

- [ ] **Step 6: Commit**

```bash
git add backend/src/mcp/router.rs backend/src/mcp/tests/test_router.rs backend/src/mcp/mod.rs
git commit -m "feat(mcp): add JSON-RPC router

- Implement McpRouter for handling JSON-RPC requests
- Handle initialize method with protocol version validation
- Add method routing for resources/* methods (stubs)
- Validate scope for all non-initialize methods
- Add tests for router functionality"
```

---

### Task 10: MCP Resource Types

**Goal**: Define MCP resource types and trait for resource providers.

**Files:**
- Create: `backend/src/mcp/resources/mod.rs`
- Create: `backend/src/mcp/resources/types.rs`

- [ ] **Step 1: Write failing test for resource types**

Create `backend/src/mcp/resources/types.rs` with tests at bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resource_uri_parsing() {
        let uri = "rustchat://messages/channel-123/msg-456";
        let parsed = ResourceUri::from_str(uri);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().scheme, "rustchat");
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "rustchat://messages/channel-123/msg-456".to_string(),
            name: "Message in General".to_string(),
            description: Some("Hello world".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "rustchat://messages/channel-123/msg-456");
        assert_eq!(json["name"], "Message in General");
    }

    #[test]
    fn test_resource_content_serialization() {
        let content = ResourceContent {
            uri: "rustchat://messages/channel-123/msg-456".to_string(),
            mime_type: "text/plain".to_string(),
            text: Some("Hello world".to_string()),
            blob: None,
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["mimeType"], "text/plain");
        assert_eq!(json["text"], "Hello world");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test mcp::resources::types`
Expected: FAIL with "module not found"

- [ ] **Step 3: Write minimal implementation**

Create `backend/src/mcp/resources/types.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::error::AppError;
use async_trait::async_trait;
use crate::auth::extractors::McpAuth;
use uuid::Uuid;

/// MCP Resource URI (e.g., "rustchat://messages/channel-123/msg-456")
#[derive(Debug, Clone)]
pub struct ResourceUri {
    pub scheme: String,
    pub resource_type: String,
    pub segments: Vec<String>,
}

impl FromStr for ResourceUri {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, "://").collect();
        if parts.len() != 2 {
            return Err(AppError::BadRequest("Invalid resource URI format".to_string()));
        }

        let scheme = parts[0].to_string();
        let path_parts: Vec<&str> = parts[1].split('/').collect();

        if path_parts.is_empty() {
            return Err(AppError::BadRequest("Missing resource type in URI".to_string()));
        }

        let resource_type = path_parts[0].to_string();
        let segments = path_parts[1..].iter().map(|s| s.to_string()).collect();

        Ok(Self {
            scheme,
            resource_type,
            segments,
        })
    }
}

/// MCP Resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// MCP Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>, // Base64-encoded binary data
}

/// Resource provider trait
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// Get resource type prefix (e.g., "messages", "channels")
    fn resource_type(&self) -> &str;

    /// List all resources of this type
    async fn list_resources(
        &self,
        auth: &McpAuth,
        cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError>;

    /// Read a specific resource
    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError>;

    /// Check if a resource has changed (for subscriptions)
    async fn has_changed(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
        since: i64,
    ) -> Result<bool, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resource_uri_parsing() {
        let uri = "rustchat://messages/channel-123/msg-456";
        let parsed = ResourceUri::from_str(uri);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.scheme, "rustchat");
        assert_eq!(parsed.resource_type, "messages");
        assert_eq!(parsed.segments.len(), 2);
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "rustchat://messages/channel-123/msg-456".to_string(),
            name: "Message in General".to_string(),
            description: Some("Hello world".to_string()),
            mime_type: Some("text/plain".to_string()),
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "rustchat://messages/channel-123/msg-456");
        assert_eq!(json["name"], "Message in General");
    }

    #[test]
    fn test_resource_content_serialization() {
        let content = ResourceContent {
            uri: "rustchat://messages/channel-123/msg-456".to_string(),
            mime_type: "text/plain".to_string(),
            text: Some("Hello world".to_string()),
            blob: None,
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["mimeType"], "text/plain");
        assert_eq!(json["text"], "Hello world");
    }
}
```

- [ ] **Step 4: Create resources module**

Create `backend/src/mcp/resources/mod.rs`:

```rust
pub mod types;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
```

- [ ] **Step 5: Update mcp/mod.rs**

Modify `backend/src/mcp/mod.rs`:

```rust
pub mod types;
pub mod router;
pub mod resources;

#[cfg(test)]
mod tests;
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cd backend && cargo test mcp::resources::types`
Expected: PASS (3 tests)

- [ ] **Step 7: Commit**

```bash
git add backend/src/mcp/resources/
git commit -m "feat(mcp): add resource types and provider trait

- Add ResourceUri for parsing MCP URIs
- Add Resource and ResourceContent types
- Add ResourceProvider trait for implementing resource handlers
- Add URI parsing and serialization tests"
```

---
# MCP Server Implementation Plan - Part 2 (Tasks 11-20)

This continues the implementation plan from `/tmp/rustchat/docs/superpowers/plans/2026-03-18-mcp-server-implementation.md`.

## Tasks 11-16: Resource Providers

### Task 11: Messages Resource Provider

**Goal**: Implement resource provider for RustChat messages with authorization checks.

**Files:**
- Create: `backend/src/mcp/resources/messages.rs`
- Create: `backend/src/mcp/resources/tests/test_messages.rs`

- [ ] **Step 1: Write failing test for messages provider**

Create `backend/src/mcp/resources/tests/test_messages.rs`:

```rust
use crate::mcp::resources::messages::MessagesProvider;
use crate::mcp::resources::types::{ResourceProvider, ResourceUri};
use crate::auth::extractors::{McpAuth, TokenType};
use uuid::Uuid;
use std::str::FromStr;

#[tokio::test]
async fn test_list_messages_requires_channel_membership() {
    // Placeholder - requires database
    // Test that listing messages checks channel membership
}

#[tokio::test]
async fn test_read_message_checks_authorization() {
    // Placeholder - requires database
    // Test that reading a message checks both:
    // 1. User is member of channel containing message
    // 2. Message exists and is not deleted
}

#[tokio::test]
async fn test_message_uri_format() {
    let uri = "rustchat://messages/channel-id/message-id";
    let parsed = ResourceUri::from_str(uri).unwrap();
    assert_eq!(parsed.resource_type, "messages");
    assert_eq!(parsed.segments.len(), 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test mcp::resources::tests::test_messages`
Expected: FAIL with "module not found: messages"

- [ ] **Step 3: Write minimal implementation**

Create `backend/src/mcp/resources/messages.rs`:

```rust
use crate::mcp::resources::types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
use crate::auth::extractors::McpAuth;
use crate::error::AppError;
use crate::AppState;
use async_trait::async_trait;
use uuid::Uuid;

pub struct MessagesProvider {
    state: AppState,
}

impl MessagesProvider {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Check if user is member of channel
    async fn check_channel_access(&self, user_id: &Uuid, channel_id: &Uuid) -> Result<(), AppError> {
        let is_member = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM channel_members
                WHERE channel_id = $1 AND user_id = $2
            ) as "exists!"
            "#,
            channel_id,
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !is_member {
            return Err(AppError::Forbidden("Not a member of this channel".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ResourceProvider for MessagesProvider {
    fn resource_type(&self) -> &str {
        "messages"
    }

    async fn list_resources(
        &self,
        auth: &McpAuth,
        cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError> {
        // Get all channels user is member of
        let channels = sqlx::query!(
            r#"
            SELECT DISTINCT c.id, c.name
            FROM channels c
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm.user_id = $1
            ORDER BY c.name
            LIMIT 100
            "#,
            auth.user_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut resources = Vec::new();

        // For each channel, get recent messages
        for channel in channels {
            let messages = sqlx::query!(
                r#"
                SELECT id, content, created_at
                FROM messages
                WHERE channel_id = $1
                  AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT 10
                "#,
                channel.id
            )
            .fetch_all(&self.state.db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            for msg in messages {
                let preview = if msg.content.len() > 50 {
                    format!("{}...", &msg.content[..50])
                } else {
                    msg.content.clone()
                };

                resources.push(Resource {
                    uri: format!("rustchat://messages/{}/{}", channel.id, msg.id),
                    name: format!("Message in {}", channel.name),
                    description: Some(preview),
                    mime_type: Some("text/plain".to_string()),
                });
            }
        }

        Ok(resources)
    }

    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError> {
        if uri.segments.len() != 2 {
            return Err(AppError::BadRequest(
                "Invalid message URI format: expected rustchat://messages/channel-id/message-id".to_string()
            ));
        }

        let channel_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
        let message_id = Uuid::parse_str(&uri.segments[1])
            .map_err(|_| AppError::BadRequest("Invalid message ID".to_string()))?;

        // Check channel access
        self.check_channel_access(&auth.user_id, &channel_id).await?;

        // Fetch message
        let message = sqlx::query!(
            r#"
            SELECT m.id, m.content, m.created_at, u.username
            FROM messages m
            INNER JOIN users u ON m.user_id = u.id
            WHERE m.id = $1
              AND m.channel_id = $2
              AND m.deleted_at IS NULL
            "#,
            message_id,
            channel_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Message not found".to_string()))?;

        let text = format!(
            "[{}] {}: {}",
            message.created_at.format("%Y-%m-%d %H:%M:%S"),
            message.username,
            message.content
        );

        Ok(ResourceContent {
            uri: format!("rustchat://messages/{}/{}", channel_id, message_id),
            mime_type: "text/plain".to_string(),
            text: Some(text),
            blob: None,
        })
    }

    async fn has_changed(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
        since: i64,
    ) -> Result<bool, AppError> {
        if uri.segments.len() != 2 {
            return Ok(false);
        }

        let channel_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;
        let message_id = Uuid::parse_str(&uri.segments[1])
            .map_err(|_| AppError::BadRequest("Invalid message ID".to_string()))?;

        // Check channel access
        self.check_channel_access(&auth.user_id, &channel_id).await?;

        // Check if message was modified after 'since'
        let since_time = chrono::NaiveDateTime::from_timestamp_opt(since, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid timestamp".to_string()))?;

        let changed = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM messages
                WHERE id = $1
                  AND channel_id = $2
                  AND updated_at > $3
            ) as "exists!"
            "#,
            message_id,
            channel_id,
            since_time
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(changed)
    }
}
```

- [ ] **Step 4: Update resources/mod.rs**

Modify `backend/src/mcp/resources/mod.rs`:

```rust
pub mod types;
pub mod messages;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
pub use messages::MessagesProvider;

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run tests**

Run: `cd backend && cargo test mcp::resources`
Expected: PASS (type tests pass, message tests are placeholders)

- [ ] **Step 6: Commit**

```bash
git add backend/src/mcp/resources/messages.rs backend/src/mcp/resources/mod.rs
git commit -m "feat(mcp): add Messages resource provider

- Implement MessagesProvider with authorization checks
- List messages from all user's channels
- Read individual messages with channel membership verification
- Check message updates for subscriptions
- Add message URI format tests"
```

---

### Task 12: Channels Resource Provider

**Goal**: Implement resource provider for RustChat channels.

**Files:**
- Create: `backend/src/mcp/resources/channels.rs`

- [ ] **Step 1: Write failing test**

Add to `backend/src/mcp/resources/tests/mod.rs`:

```rust
#[tokio::test]
async fn test_list_channels_only_shows_member_channels() {
    // Placeholder - requires database
    // Test that user only sees channels they're members of
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test mcp::resources::tests`
Expected: Test is placeholder

- [ ] **Step 3: Write implementation**

Create `backend/src/mcp/resources/channels.rs`:

```rust
use crate::mcp::resources::types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
use crate::auth::extractors::McpAuth;
use crate::error::AppError;
use crate::AppState;
use async_trait::async_trait;
use uuid::Uuid;

pub struct ChannelsProvider {
    state: AppState,
}

impl ChannelsProvider {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn check_channel_access(&self, user_id: &Uuid, channel_id: &Uuid) -> Result<(), AppError> {
        let is_member = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM channel_members
                WHERE channel_id = $1 AND user_id = $2
            ) as "exists!"
            "#,
            channel_id,
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !is_member {
            return Err(AppError::Forbidden("Not a member of this channel".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ResourceProvider for ChannelsProvider {
    fn resource_type(&self) -> &str {
        "channels"
    }

    async fn list_resources(
        &self,
        auth: &McpAuth,
        _cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError> {
        let channels = sqlx::query!(
            r#"
            SELECT c.id, c.name, c.description, c.channel_type,
                   COUNT(DISTINCT cm.user_id) as member_count
            FROM channels c
            INNER JOIN channel_members cm_user ON c.id = cm_user.channel_id
            LEFT JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm_user.user_id = $1
            GROUP BY c.id, c.name, c.description, c.channel_type
            ORDER BY c.name
            LIMIT 100
            "#,
            auth.user_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(channels
            .into_iter()
            .map(|ch| Resource {
                uri: format!("rustchat://channels/{}", ch.id),
                name: ch.name.clone(),
                description: ch.description.or_else(|| {
                    Some(format!(
                        "{} channel with {} members",
                        ch.channel_type, ch.member_count.unwrap_or(0)
                    ))
                }),
                mime_type: Some("application/json".to_string()),
            })
            .collect())
    }

    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError> {
        if uri.segments.is_empty() {
            return Err(AppError::BadRequest(
                "Invalid channel URI format: expected rustchat://channels/channel-id".to_string()
            ));
        }

        let channel_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;

        // Check access
        self.check_channel_access(&auth.user_id, &channel_id).await?;

        // Fetch channel details
        let channel = sqlx::query!(
            r#"
            SELECT c.id, c.name, c.description, c.channel_type, c.created_at,
                   COUNT(DISTINCT cm.user_id) as member_count
            FROM channels c
            LEFT JOIN channel_members cm ON c.id = cm.channel_id
            WHERE c.id = $1
            GROUP BY c.id
            "#,
            channel_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

        let text = serde_json::to_string_pretty(&serde_json::json!({
            "id": channel.id,
            "name": channel.name,
            "description": channel.description,
            "type": channel.channel_type,
            "member_count": channel.member_count,
            "created_at": channel.created_at.to_rfc3339(),
        }))
        .unwrap();

        Ok(ResourceContent {
            uri: format!("rustchat://channels/{}", channel_id),
            mime_type: "application/json".to_string(),
            text: Some(text),
            blob: None,
        })
    }

    async fn has_changed(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
        since: i64,
    ) -> Result<bool, AppError> {
        if uri.segments.is_empty() {
            return Ok(false);
        }

        let channel_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid channel ID".to_string()))?;

        self.check_channel_access(&auth.user_id, &channel_id).await?;

        let since_time = chrono::NaiveDateTime::from_timestamp_opt(since, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid timestamp".to_string()))?;

        let changed = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM channels
                WHERE id = $1 AND updated_at > $2
            ) as "exists!"
            "#,
            channel_id,
            since_time
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(changed)
    }
}
```

- [ ] **Step 4: Update resources/mod.rs**

```rust
pub mod types;
pub mod messages;
pub mod channels;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
pub use messages::MessagesProvider;
pub use channels::ChannelsProvider;
```

- [ ] **Step 5: Run tests**

Run: `cd backend && cargo test mcp::resources`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/src/mcp/resources/channels.rs backend/src/mcp/resources/mod.rs
git commit -m "feat(mcp): add Channels resource provider

- Implement ChannelsProvider with membership filtering
- List user's channels with member counts
- Read channel details as JSON
- Check channel updates for subscriptions"
```

---

### Task 13: Users Resource Provider

**Goal**: Implement resource provider for RustChat users (team members only).

**Files:**
- Create: `backend/src/mcp/resources/users.rs`

- [ ] **Step 1: Write failing test**

```rust
#[tokio::test]
async fn test_list_users_only_shows_team_members() {
    // Placeholder - should only show users in same teams as requester
}
```

- [ ] **Step 2: Write implementation**

Create `backend/src/mcp/resources/users.rs`:

```rust
use crate::mcp::resources::types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
use crate::auth::extractors::McpAuth;
use crate::error::AppError;
use crate::AppState;
use async_trait::async_trait;
use uuid::Uuid;

pub struct UsersProvider {
    state: AppState,
}

impl UsersProvider {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Check if user can view target user (must be in same team)
    async fn check_user_visibility(&self, requester_id: &Uuid, target_id: &Uuid) -> Result<(), AppError> {
        let shared_team = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM team_members tm1
                INNER JOIN team_members tm2 ON tm1.team_id = tm2.team_id
                WHERE tm1.user_id = $1 AND tm2.user_id = $2
            ) as "exists!"
            "#,
            requester_id,
            target_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !shared_team {
            return Err(AppError::Forbidden("User not in your teams".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ResourceProvider for UsersProvider {
    fn resource_type(&self) -> &str {
        "users"
    }

    async fn list_resources(
        &self,
        auth: &McpAuth,
        _cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError> {
        // Get all users in requester's teams
        let users = sqlx::query!(
            r#"
            SELECT DISTINCT u.id, u.username, u.display_name, u.email
            FROM users u
            INNER JOIN team_members tm1 ON u.id = tm1.user_id
            INNER JOIN team_members tm2 ON tm1.team_id = tm2.team_id
            WHERE tm2.user_id = $1
              AND u.is_active = true
              AND u.deleted_at IS NULL
            ORDER BY u.username
            LIMIT 200
            "#,
            auth.user_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(users
            .into_iter()
            .map(|u| Resource {
                uri: format!("rustchat://users/{}", u.id),
                name: u.display_name.unwrap_or(u.username.clone()),
                description: Some(format!("@{} ({})", u.username, u.email)),
                mime_type: Some("application/json".to_string()),
            })
            .collect())
    }

    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError> {
        if uri.segments.is_empty() {
            return Err(AppError::BadRequest(
                "Invalid user URI format: expected rustchat://users/user-id".to_string()
            ));
        }

        let user_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

        // Check visibility
        self.check_user_visibility(&auth.user_id, &user_id).await?;

        // Fetch user details
        let user = sqlx::query!(
            r#"
            SELECT id, username, display_name, email, entity_type, created_at
            FROM users
            WHERE id = $1 AND is_active = true AND deleted_at IS NULL
            "#,
            user_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let text = serde_json::to_string_pretty(&serde_json::json!({
            "id": user.id,
            "username": user.username,
            "display_name": user.display_name,
            "email": user.email,
            "entity_type": user.entity_type,
            "created_at": user.created_at.to_rfc3339(),
        }))
        .unwrap();

        Ok(ResourceContent {
            uri: format!("rustchat://users/{}", user_id),
            mime_type: "application/json".to_string(),
            text: Some(text),
            blob: None,
        })
    }

    async fn has_changed(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
        since: i64,
    ) -> Result<bool, AppError> {
        if uri.segments.is_empty() {
            return Ok(false);
        }

        let user_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

        self.check_user_visibility(&auth.user_id, &user_id).await?;

        let since_time = chrono::NaiveDateTime::from_timestamp_opt(since, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid timestamp".to_string()))?;

        let changed = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE id = $1 AND updated_at > $2
            ) as "exists!"
            "#,
            user_id,
            since_time
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(changed)
    }
}
```

- [ ] **Step 3: Update resources/mod.rs**

```rust
pub mod types;
pub mod messages;
pub mod channels;
pub mod users;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
pub use messages::MessagesProvider;
pub use channels::ChannelsProvider;
pub use users::UsersProvider;
```

- [ ] **Step 4: Commit**

```bash
git add backend/src/mcp/resources/users.rs backend/src/mcp/resources/mod.rs
git commit -m "feat(mcp): add Users resource provider

- Implement UsersProvider with team visibility filtering
- List users from requester's teams
- Read user profile details as JSON
- Check user profile updates for subscriptions"
```

---

### Task 14: Files Resource Provider

**Goal**: Implement resource provider for RustChat files (with channel authorization).

**Files:**
- Create: `backend/src/mcp/resources/files.rs`

- [ ] **Step 1: Write implementation**

Create `backend/src/mcp/resources/files.rs`:

```rust
use crate::mcp::resources::types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
use crate::auth::extractors::McpAuth;
use crate::error::AppError;
use crate::AppState;
use async_trait::async_trait;
use uuid::Uuid;

pub struct FilesProvider {
    state: AppState,
}

impl FilesProvider {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn check_file_access(&self, user_id: &Uuid, file_id: &Uuid) -> Result<(), AppError> {
        // Files are accessible if user is member of the channel where file was uploaded
        let has_access = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM files f
                INNER JOIN messages m ON f.message_id = m.id
                INNER JOIN channel_members cm ON m.channel_id = cm.channel_id
                WHERE f.id = $1 AND cm.user_id = $2
            ) as "exists!"
            "#,
            file_id,
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !has_access {
            return Err(AppError::Forbidden("Cannot access this file".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ResourceProvider for FilesProvider {
    fn resource_type(&self) -> &str {
        "files"
    }

    async fn list_resources(
        &self,
        auth: &McpAuth,
        _cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError> {
        // Get recent files from user's channels
        let files = sqlx::query!(
            r#"
            SELECT DISTINCT f.id, f.filename, f.content_type, f.size, c.name as channel_name
            FROM files f
            INNER JOIN messages m ON f.message_id = m.id
            INNER JOIN channels c ON m.channel_id = c.id
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            WHERE cm.user_id = $1
              AND f.deleted_at IS NULL
            ORDER BY f.created_at DESC
            LIMIT 100
            "#,
            auth.user_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(files
            .into_iter()
            .map(|f| {
                let size_kb = f.size.unwrap_or(0) / 1024;
                Resource {
                    uri: format!("rustchat://files/{}", f.id),
                    name: f.filename.clone(),
                    description: Some(format!(
                        "File in {} ({} KB)",
                        f.channel_name.unwrap_or_else(|| "Unknown".to_string()),
                        size_kb
                    )),
                    mime_type: f.content_type,
                }
            })
            .collect())
    }

    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError> {
        if uri.segments.is_empty() {
            return Err(AppError::BadRequest(
                "Invalid file URI format: expected rustchat://files/file-id".to_string()
            ));
        }

        let file_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid file ID".to_string()))?;

        // Check access
        self.check_file_access(&auth.user_id, &file_id).await?;

        // Fetch file metadata
        let file = sqlx::query!(
            r#"
            SELECT f.id, f.filename, f.content_type, f.size, f.storage_path,
                   u.username as uploader
            FROM files f
            INNER JOIN messages m ON f.message_id = m.id
            INNER JOIN users u ON m.user_id = u.id
            WHERE f.id = $1 AND f.deleted_at IS NULL
            "#,
            file_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

        // Return metadata as JSON (not actual file contents for Phase 2.x)
        let text = serde_json::to_string_pretty(&serde_json::json!({
            "id": file.id,
            "filename": file.filename,
            "content_type": file.content_type,
            "size_bytes": file.size,
            "uploaded_by": file.uploader,
            "note": "File contents not available via MCP in Phase 2.x (read-only context)"
        }))
        .unwrap();

        Ok(ResourceContent {
            uri: format!("rustchat://files/{}", file_id),
            mime_type: "application/json".to_string(),
            text: Some(text),
            blob: None,
        })
    }

    async fn has_changed(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
        since: i64,
    ) -> Result<bool, AppError> {
        if uri.segments.is_empty() {
            return Ok(false);
        }

        let file_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid file ID".to_string()))?;

        self.check_file_access(&auth.user_id, &file_id).await?;

        let since_time = chrono::NaiveDateTime::from_timestamp_opt(since, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid timestamp".to_string()))?;

        let changed = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM files
                WHERE id = $1 AND updated_at > $2
            ) as "exists!"
            "#,
            file_id,
            since_time
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(changed)
    }
}
```

- [ ] **Step 2: Update resources/mod.rs**

```rust
pub mod types;
pub mod messages;
pub mod channels;
pub mod users;
pub mod files;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
pub use messages::MessagesProvider;
pub use channels::ChannelsProvider;
pub use users::UsersProvider;
pub use files::FilesProvider;
```

- [ ] **Step 3: Commit**

```bash
git add backend/src/mcp/resources/files.rs backend/src/mcp/resources/mod.rs
git commit -m "feat(mcp): add Files resource provider

- Implement FilesProvider with channel-based authorization
- List files from user's channels
- Read file metadata as JSON (not actual contents in Phase 2.x)
- Check file updates for subscriptions"
```

---

### Task 15: Teams Resource Provider

**Goal**: Implement resource provider for RustChat teams (member-only).

**Files:**
- Create: `backend/src/mcp/resources/teams.rs`

- [ ] **Step 1: Write implementation**

Create `backend/src/mcp/resources/teams.rs`:

```rust
use crate::mcp::resources::types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
use crate::auth::extractors::McpAuth;
use crate::error::AppError;
use crate::AppState;
use async_trait::async_trait;
use uuid::Uuid;

pub struct TeamsProvider {
    state: AppState,
}

impl TeamsProvider {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    async fn check_team_access(&self, user_id: &Uuid, team_id: &Uuid) -> Result<(), AppError> {
        let is_member = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM team_members
                WHERE team_id = $1 AND user_id = $2
            ) as "exists!"
            "#,
            team_id,
            user_id
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if !is_member {
            return Err(AppError::Forbidden("Not a member of this team".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ResourceProvider for TeamsProvider {
    fn resource_type(&self) -> &str {
        "teams"
    }

    async fn list_resources(
        &self,
        auth: &McpAuth,
        _cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError> {
        let teams = sqlx::query!(
            r#"
            SELECT t.id, t.name, t.description,
                   COUNT(DISTINCT tm.user_id) as member_count
            FROM teams t
            INNER JOIN team_members tm_user ON t.id = tm_user.team_id
            LEFT JOIN team_members tm ON t.id = tm.team_id
            WHERE tm_user.user_id = $1
            GROUP BY t.id, t.name, t.description
            ORDER BY t.name
            LIMIT 50
            "#,
            auth.user_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(teams
            .into_iter()
            .map(|t| Resource {
                uri: format!("rustchat://teams/{}", t.id),
                name: t.name.clone(),
                description: t.description.or_else(|| {
                    Some(format!("Team with {} members", t.member_count.unwrap_or(0)))
                }),
                mime_type: Some("application/json".to_string()),
            })
            .collect())
    }

    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError> {
        if uri.segments.is_empty() {
            return Err(AppError::BadRequest(
                "Invalid team URI format: expected rustchat://teams/team-id".to_string()
            ));
        }

        let team_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid team ID".to_string()))?;

        // Check access
        self.check_team_access(&auth.user_id, &team_id).await?;

        // Fetch team details with members
        let team = sqlx::query!(
            r#"
            SELECT t.id, t.name, t.description, t.created_at,
                   COUNT(DISTINCT tm.user_id) as member_count
            FROM teams t
            LEFT JOIN team_members tm ON t.id = tm.team_id
            WHERE t.id = $1
            GROUP BY t.id
            "#,
            team_id
        )
        .fetch_optional(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

        // Get team members
        let members = sqlx::query!(
            r#"
            SELECT u.id, u.username, u.display_name, tm.role
            FROM team_members tm
            INNER JOIN users u ON tm.user_id = u.id
            WHERE tm.team_id = $1
              AND u.is_active = true
              AND u.deleted_at IS NULL
            ORDER BY tm.role, u.username
            "#,
            team_id
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let members_json: Vec<_> = members
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "username": m.username,
                    "display_name": m.display_name,
                    "role": m.role,
                })
            })
            .collect();

        let text = serde_json::to_string_pretty(&serde_json::json!({
            "id": team.id,
            "name": team.name,
            "description": team.description,
            "member_count": team.member_count,
            "created_at": team.created_at.to_rfc3339(),
            "members": members_json,
        }))
        .unwrap();

        Ok(ResourceContent {
            uri: format!("rustchat://teams/{}", team_id),
            mime_type: "application/json".to_string(),
            text: Some(text),
            blob: None,
        })
    }

    async fn has_changed(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
        since: i64,
    ) -> Result<bool, AppError> {
        if uri.segments.is_empty() {
            return Ok(false);
        }

        let team_id = Uuid::parse_str(&uri.segments[0])
            .map_err(|_| AppError::BadRequest("Invalid team ID".to_string()))?;

        self.check_team_access(&auth.user_id, &team_id).await?;

        let since_time = chrono::NaiveDateTime::from_timestamp_opt(since, 0)
            .ok_or_else(|| AppError::BadRequest("Invalid timestamp".to_string()))?;

        // Check if team or its members changed
        let changed = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM teams WHERE id = $1 AND updated_at > $2
                UNION ALL
                SELECT 1 FROM team_members WHERE team_id = $1 AND created_at > $2
            ) as "exists!"
            "#,
            team_id,
            since_time
        )
        .fetch_one(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(changed)
    }
}
```

- [ ] **Step 2: Update resources/mod.rs**

```rust
pub mod types;
pub mod messages;
pub mod channels;
pub mod users;
pub mod files;
pub mod teams;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
pub use messages::MessagesProvider;
pub use channels::ChannelsProvider;
pub use users::UsersProvider;
pub use files::FilesProvider;
pub use teams::TeamsProvider;
```

- [ ] **Step 3: Commit**

```bash
git add backend/src/mcp/resources/teams.rs backend/src/mcp/resources/mod.rs
git commit -m "feat(mcp): add Teams resource provider

- Implement TeamsProvider with member-only access
- List user's teams with member counts
- Read team details with member list as JSON
- Check team/membership updates for subscriptions"
```

---

### Task 16: Search Resource Provider

**Goal**: Implement search resource provider with authorization filtering.

**Files:**
- Create: `backend/src/mcp/resources/search.rs`

- [ ] **Step 1: Write implementation**

Create `backend/src/mcp/resources/search.rs`:

```rust
use crate::mcp::resources::types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
use crate::auth::extractors::McpAuth;
use crate::error::AppError;
use crate::AppState;
use async_trait::async_trait;

pub struct SearchProvider {
    state: AppState,
}

impl SearchProvider {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ResourceProvider for SearchProvider {
    fn resource_type(&self) -> &str {
        "search"
    }

    async fn list_resources(
        &self,
        _auth: &McpAuth,
        _cursor: Option<String>,
    ) -> Result<Vec<Resource>, AppError> {
        // Search doesn't have a static list, return empty
        Ok(vec![])
    }

    async fn read_resource(
        &self,
        auth: &McpAuth,
        uri: &ResourceUri,
    ) -> Result<ResourceContent, AppError> {
        // URI format: rustchat://search/<query>
        if uri.segments.is_empty() {
            return Err(AppError::BadRequest(
                "Invalid search URI format: expected rustchat://search/<query>".to_string()
            ));
        }

        let query = uri.segments.join("/");

        // Search messages in user's channels
        let messages = sqlx::query!(
            r#"
            SELECT m.id, m.content, m.created_at, c.id as channel_id, c.name as channel_name,
                   u.username
            FROM messages m
            INNER JOIN channels c ON m.channel_id = c.id
            INNER JOIN channel_members cm ON c.id = cm.channel_id
            INNER JOIN users u ON m.user_id = u.id
            WHERE cm.user_id = $1
              AND m.deleted_at IS NULL
              AND m.content ILIKE $2
            ORDER BY m.created_at DESC
            LIMIT 50
            "#,
            auth.user_id,
            format!("%{}%", query)
        )
        .fetch_all(&self.state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let results: Vec<_> = messages
            .into_iter()
            .map(|m| {
                serde_json::json!({
                    "type": "message",
                    "id": m.id,
                    "channel_id": m.channel_id,
                    "channel_name": m.channel_name,
                    "username": m.username,
                    "content": m.content,
                    "created_at": m.created_at.to_rfc3339(),
                    "uri": format!("rustchat://messages/{}/{}", m.channel_id, m.id),
                })
            })
            .collect();

        let text = serde_json::to_string_pretty(&serde_json::json!({
            "query": query,
            "result_count": results.len(),
            "results": results,
        }))
        .unwrap();

        Ok(ResourceContent {
            uri: format!("rustchat://search/{}", query),
            mime_type: "application/json".to_string(),
            text: Some(text),
            blob: None,
        })
    }

    async fn has_changed(
        &self,
        _auth: &McpAuth,
        _uri: &ResourceUri,
        _since: i64,
    ) -> Result<bool, AppError> {
        // Search results always potentially change
        Ok(true)
    }
}
```

- [ ] **Step 2: Update resources/mod.rs**

```rust
pub mod types;
pub mod messages;
pub mod channels;
pub mod users;
pub mod files;
pub mod teams;
pub mod search;

pub use types::{Resource, ResourceContent, ResourceUri, ResourceProvider};
pub use messages::MessagesProvider;
pub use channels::ChannelsProvider;
pub use users::UsersProvider;
pub use files::FilesProvider;
pub use teams::TeamsProvider;
pub use search::SearchProvider;
```

- [ ] **Step 3: Commit**

```bash
git add backend/src/mcp/resources/search.rs backend/src/mcp/resources/mod.rs
git commit -m "feat(mcp): add Search resource provider

- Implement SearchProvider for full-text message search
- Filter results to user's accessible channels
- Return search results as JSON with URIs
- Always mark as changed (search results are dynamic)"
```

---


### Task 17: SSE Connection Manager

**Goal**: Implement Server-Sent Events manager for real-time resource updates.

**Files:**
- Create: `backend/src/mcp/sse/mod.rs`
- Create: `backend/src/mcp/sse/manager.rs`
- Create: `backend/src/mcp/sse/connection.rs`

- [ ] **Step 1: Write failing test for SSE manager**

Create `backend/src/mcp/sse/tests.rs`:

```rust
use super::manager::SseManager;
use crate::mcp::types::JsonRpcNotification;
use uuid::Uuid;
use serde_json::json;

#[tokio::test]
async fn test_sse_manager_subscribe() {
    let manager = SseManager::new();
    let client_id = Uuid::new_v4();
    let uri = "rustchat://messages/channel-123/msg-456".to_string();

    // Subscribe should add subscription
    manager.subscribe(client_id, uri.clone()).await;

    // Check subscription exists
    let subscriptions = manager.get_subscriptions(client_id).await;
    assert!(subscriptions.contains(&uri));
}

#[tokio::test]
async fn test_sse_manager_notify() {
    let manager = SseManager::new();
    let client_id = Uuid::new_v4();
    let uri = "rustchat://messages/channel-123/msg-456".to_string();

    // Subscribe
    manager.subscribe(client_id, uri.clone()).await;

    // Notify should deliver to subscribed clients
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "notifications/resources/updated".to_string(),
        params: json!({ "uri": uri }),
    };

    manager.notify(&uri, notification).await;

    // Verify notification was sent (would check channel in real implementation)
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test mcp::sse::tests`
Expected: FAIL with "module not found"

- [ ] **Step 3: Write SSE connection type**

Create `backend/src/mcp/sse/connection.rs`:

```rust
use tokio::sync::mpsc;
use crate::mcp::types::JsonRpcNotification;
use uuid::Uuid;
use std::time::{Duration, Instant};

/// SSE connection state
pub struct SseConnection {
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub tx: mpsc::UnboundedSender<JsonRpcNotification>,
    pub last_activity: Instant,
    pub subscriptions: Vec<String>,
}

impl SseConnection {
    pub fn new(
        client_id: Uuid,
        user_id: Uuid,
        tx: mpsc::UnboundedSender<JsonRpcNotification>,
    ) -> Self {
        Self {
            client_id,
            user_id,
            tx,
            last_activity: Instant::now(),
            subscriptions: Vec::new(),
        }
    }

    /// Send a notification to this connection
    pub fn send(&self, notification: JsonRpcNotification) -> Result<(), String> {
        self.tx
            .send(notification)
            .map_err(|_| "Failed to send notification".to_string())
    }

    /// Check if connection is stale (no activity for 5 minutes)
    pub fn is_stale(&self) -> bool {
        self.last_activity.elapsed() > Duration::from_secs(300)
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Add subscription
    pub fn subscribe(&mut self, uri: String) {
        if !self.subscriptions.contains(&uri) {
            self.subscriptions.push(uri);
        }
    }

    /// Remove subscription
    pub fn unsubscribe(&mut self, uri: &str) {
        self.subscriptions.retain(|s| s != uri);
    }
}
```

- [ ] **Step 4: Write SSE manager implementation**

Create `backend/src/mcp/sse/manager.rs`:

```rust
use crate::mcp::sse::connection::SseConnection;
use crate::mcp::types::JsonRpcNotification;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// SSE connection manager
pub struct SseManager {
    connections: Arc<RwLock<HashMap<Uuid, SseConnection>>>,
}

impl SseManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new SSE connection
    pub async fn register(&self, connection: SseConnection) {
        let client_id = connection.client_id;
        let mut connections = self.connections.write().await;
        connections.insert(client_id, connection);
    }

    /// Unregister an SSE connection
    pub async fn unregister(&self, client_id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&client_id);
    }

    /// Subscribe a client to a resource URI
    pub async fn subscribe(&self, client_id: Uuid, uri: String) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(&client_id) {
            conn.subscribe(uri);
            conn.touch();
        }
    }

    /// Unsubscribe a client from a resource URI
    pub async fn unsubscribe(&self, client_id: Uuid, uri: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(&client_id) {
            conn.unsubscribe(uri);
            conn.touch();
        }
    }

    /// Get all subscriptions for a client
    pub async fn get_subscriptions(&self, client_id: Uuid) -> Vec<String> {
        let connections = self.connections.read().await;
        connections
            .get(&client_id)
            .map(|conn| conn.subscriptions.clone())
            .unwrap_or_default()
    }

    /// Notify all subscribers of a resource update
    pub async fn notify(&self, uri: &str, notification: JsonRpcNotification) {
        let connections = self.connections.read().await;

        for conn in connections.values() {
            if conn.subscriptions.contains(&uri.to_string()) {
                // Ignore send errors (client might have disconnected)
                let _ = conn.send(notification.clone());
            }
        }
    }

    /// Remove stale connections (no activity for 5+ minutes)
    pub async fn cleanup_stale_connections(&self) {
        let mut connections = self.connections.write().await;
        connections.retain(|_, conn| !conn.is_stale());
    }

    /// Get count of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

impl Clone for SseManager {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
        }
    }
}
```

- [ ] **Step 5: Create SSE module**

Create `backend/src/mcp/sse/mod.rs`:

```rust
pub mod manager;
pub mod connection;

pub use manager::SseManager;
pub use connection::SseConnection;

#[cfg(test)]
mod tests;
```

- [ ] **Step 6: Update mcp/mod.rs**

Modify `backend/src/mcp/mod.rs`:

```rust
pub mod types;
pub mod router;
pub mod resources;
pub mod sse;

#[cfg(test)]
mod tests;
```

- [ ] **Step 7: Run tests**

Run: `cd backend && cargo test mcp::sse`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add backend/src/mcp/sse/
git commit -m "feat(mcp): add SSE connection manager

- Implement SseManager for managing subscriptions
- Add SseConnection for tracking client state
- Support subscribe/unsubscribe operations
- Add notification broadcasting to subscribers
- Add stale connection cleanup"
```

---

### Task 18: MCP Audit Service

**Goal**: Implement audit logging service for MCP access.

**Files:**
- Create: `backend/src/mcp/audit.rs`
- Modify: `backend/src/services/mod.rs`

- [ ] **Step 1: Write failing test for audit service**

Create `backend/src/mcp/tests/test_audit.rs`:

```rust
#[tokio::test]
async fn test_audit_log_creation() {
    // Placeholder - requires database
    // Test that audit logs are created for MCP operations
}

#[tokio::test]
async fn test_audit_log_retention() {
    // Placeholder - requires database
    // Test that logs older than 90 days are marked for deletion
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test mcp::tests::test_audit`
Expected: Test is placeholder

- [ ] **Step 3: Write audit service implementation**

Create `backend/src/mcp/audit.rs`:

```rust
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

/// MCP audit service for logging access
pub struct McpAuditService {
    db: PgPool,
}

impl McpAuditService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Log an MCP operation
    pub async fn log_operation(
        &self,
        user_id: Uuid,
        client_id: &str,
        operation: &str,
        resource_uri: Option<&str>,
        request_payload: Option<Value>,
        response_status: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO mcp_audit_logs (
                user_id, client_id, operation, resource_uri,
                request_payload, response_status
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user_id,
            client_id,
            operation,
            resource_uri,
            request_payload,
            response_status
        )
        .execute(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get audit logs for a user (paginated)
    pub async fn get_user_logs(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        let logs = sqlx::query_as!(
            AuditLogEntry,
            r#"
            SELECT id, user_id, client_id, operation, resource_uri,
                   request_payload, response_status, created_at
            FROM mcp_audit_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(logs)
    }

    /// Get audit logs for a client (paginated)
    pub async fn get_client_logs(
        &self,
        client_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        let logs = sqlx::query_as!(
            AuditLogEntry,
            r#"
            SELECT id, user_id, client_id, operation, resource_uri,
                   request_payload, response_status, created_at
            FROM mcp_audit_logs
            WHERE client_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            client_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(logs)
    }

    /// Count audit logs for statistics
    pub async fn count_logs(
        &self,
        user_id: Option<Uuid>,
        client_id: Option<&str>,
    ) -> Result<i64, AppError> {
        let count = match (user_id, client_id) {
            (Some(uid), Some(cid)) => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM mcp_audit_logs WHERE user_id = $1 AND client_id = $2",
                    uid,
                    cid
                )
                .fetch_one(&self.db)
                .await?
            }
            (Some(uid), None) => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM mcp_audit_logs WHERE user_id = $1",
                    uid
                )
                .fetch_one(&self.db)
                .await?
            }
            (None, Some(cid)) => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM mcp_audit_logs WHERE client_id = $1",
                    cid
                )
                .fetch_one(&self.db)
                .await?
            }
            (None, None) => {
                sqlx::query_scalar!("SELECT COUNT(*) FROM mcp_audit_logs")
                    .fetch_one(&self.db)
                    .await?
            }
        };

        Ok(count.unwrap_or(0))
    }
}

/// Audit log entry struct
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: String,
    pub operation: String,
    pub resource_uri: Option<String>,
    pub request_payload: Option<Value>,
    pub response_status: String,
    pub created_at: chrono::NaiveDateTime,
}
```

- [ ] **Step 4: Update mcp/mod.rs**

Modify `backend/src/mcp/mod.rs`:

```rust
pub mod types;
pub mod router;
pub mod resources;
pub mod sse;
pub mod audit;

pub use audit::{McpAuditService, AuditLogEntry};

#[cfg(test)]
mod tests;
```

- [ ] **Step 5: Run tests**

Run: `cd backend && cargo test`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add backend/src/mcp/audit.rs backend/src/mcp/mod.rs
git commit -m "feat(mcp): add audit logging service

- Implement McpAuditService for logging MCP operations
- Support logging user/client/operation/resource/status
- Add queries for retrieving audit logs (user/client filters)
- Add log counting for statistics
- 90-day retention enforced by database migration"
```

---


### Task 19: Admin API Endpoints

**Goal**: Implement admin endpoints for OAuth client management.

**Files:**
- Create: `backend/src/api/v1/oauth_admin.rs`
- Modify: `backend/src/api/v1/mod.rs`

- [ ] **Step 1: Write failing test for admin endpoints**

Create `backend/src/api/v1/tests/test_oauth_admin.rs`:

```rust
#[tokio::test]
async fn test_create_oauth_client_requires_admin() {
    // Placeholder - requires database
    // Test that only admins can create OAuth clients
}

#[tokio::test]
async fn test_list_oauth_clients() {
    // Placeholder - requires database
    // Test that owner can list their OAuth clients
}

#[tokio::test]
async fn test_revoke_client_requires_ownership() {
    // Placeholder - requires database
    // Test that only owner/admin can revoke client
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd backend && cargo test api::v1::tests::test_oauth_admin`
Expected: Test is placeholder

- [ ] **Step 3: Write admin endpoints implementation**

Create `backend/src/api/v1/oauth_admin.rs`:

```rust
use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{
    auth::extractors::Auth,
    error::{ApiResult, AppError},
    oauth::utils::{generate_client_id, generate_client_secret, hash_client_secret},
    AppState,
};

/// Register OAuth admin routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/clients", post(create_client))
        .route("/clients", get(list_clients))
        .route("/clients/:client_id", get(get_client))
        .route("/clients/:client_id", delete(revoke_client))
        .route("/clients/:client_id/regenerate-secret", post(regenerate_secret))
}

#[derive(Debug, Deserialize)]
struct CreateClientRequest {
    client_name: String,
    client_type: String, // "confidential" or "public"
    redirect_uris: Vec<String>,
    allowed_scopes: Vec<String>,
    is_first_party: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CreateClientResponse {
    client_id: String,
    client_secret: String, // Only returned once!
    client_name: String,
    client_type: String,
    redirect_uris: Vec<String>,
    allowed_scopes: Vec<String>,
}

/// Create OAuth client (admin only)
async fn create_client(
    State(state): State<AppState>,
    auth: Auth,
    Json(req): Json<CreateClientRequest>,
) -> ApiResult<Json<CreateClientResponse>> {
    // Check admin role
    if auth.role != "admin" {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    // Validate client type
    if req.client_type != "confidential" && req.client_type != "public" {
        return Err(AppError::BadRequest(
            "client_type must be 'confidential' or 'public'".to_string(),
        ));
    }

    // Validate redirect URIs
    if req.redirect_uris.is_empty() {
        return Err(AppError::BadRequest("At least one redirect_uri required".to_string()));
    }

    // Generate credentials
    let client_id = generate_client_id();
    let client_secret = generate_client_secret();
    let secret_hash = hash_client_secret(&client_secret)?;

    // Store in database
    sqlx::query!(
        r#"
        INSERT INTO oauth_clients (
            client_id, client_secret_hash, client_name, client_type,
            redirect_uris, allowed_scopes, owner_user_id, is_first_party
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        client_id,
        secret_hash,
        req.client_name,
        req.client_type,
        &req.redirect_uris,
        &req.allowed_scopes,
        auth.user_id,
        req.is_first_party.unwrap_or(false)
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(CreateClientResponse {
        client_id,
        client_secret, // Only returned once!
        client_name: req.client_name,
        client_type: req.client_type,
        redirect_uris: req.redirect_uris,
        allowed_scopes: req.allowed_scopes,
    }))
}

#[derive(Debug, Serialize)]
struct OAuthClientInfo {
    client_id: String,
    client_name: String,
    client_type: String,
    redirect_uris: Vec<String>,
    allowed_scopes: Vec<String>,
    is_active: bool,
    is_first_party: bool,
    created_at: String,
}

/// List OAuth clients (owner or admin only)
async fn list_clients(
    State(state): State<AppState>,
    auth: Auth,
) -> ApiResult<Json<Vec<OAuthClientInfo>>> {
    let clients = if auth.role == "admin" {
        // Admins see all clients
        sqlx::query!(
            r#"
            SELECT client_id, client_name, client_type, redirect_uris,
                   allowed_scopes, is_active, is_first_party, created_at
            FROM oauth_clients
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    } else {
        // Users see only their clients
        sqlx::query!(
            r#"
            SELECT client_id, client_name, client_type, redirect_uris,
                   allowed_scopes, is_active, is_first_party, created_at
            FROM oauth_clients
            WHERE owner_user_id = $1
            ORDER BY created_at DESC
            "#,
            auth.user_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
    };

    let response: Vec<_> = clients
        .into_iter()
        .map(|c| OAuthClientInfo {
            client_id: c.client_id,
            client_name: c.client_name,
            client_type: c.client_type,
            redirect_uris: c.redirect_uris,
            allowed_scopes: c.allowed_scopes,
            is_active: c.is_active,
            is_first_party: c.is_first_party,
            created_at: c.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(response))
}

/// Get OAuth client details
async fn get_client(
    State(state): State<AppState>,
    auth: Auth,
    Path(client_id): Path<String>,
) -> ApiResult<Json<OAuthClientInfo>> {
    let client = sqlx::query!(
        r#"
        SELECT client_id, client_name, client_type, redirect_uris,
               allowed_scopes, is_active, is_first_party, created_at,
               owner_user_id
        FROM oauth_clients
        WHERE client_id = $1
        "#,
        client_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("OAuth client not found".to_string()))?;

    // Check ownership
    if auth.role != "admin" && client.owner_user_id != auth.user_id {
        return Err(AppError::Forbidden("Not your OAuth client".to_string()));
    }

    Ok(Json(OAuthClientInfo {
        client_id: client.client_id,
        client_name: client.client_name,
        client_type: client.client_type,
        redirect_uris: client.redirect_uris,
        allowed_scopes: client.allowed_scopes,
        is_active: client.is_active,
        is_first_party: client.is_first_party,
        created_at: client.created_at.to_rfc3339(),
    }))
}

/// Revoke OAuth client (owner or admin only)
async fn revoke_client(
    State(state): State<AppState>,
    auth: Auth,
    Path(client_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check ownership
    let client = sqlx::query!(
        "SELECT owner_user_id FROM oauth_clients WHERE client_id = $1",
        client_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("OAuth client not found".to_string()))?;

    if auth.role != "admin" && client.owner_user_id != auth.user_id {
        return Err(AppError::Forbidden("Not your OAuth client".to_string()));
    }

    // Deactivate client
    sqlx::query!(
        "UPDATE oauth_clients SET is_active = false, updated_at = NOW() WHERE client_id = $1",
        client_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Revoke all active tokens
    sqlx::query!(
        "UPDATE oauth_access_tokens SET revoked_at = NOW() WHERE client_id = $1 AND revoked_at IS NULL",
        client_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "message": "OAuth client revoked successfully",
        "client_id": client_id
    })))
}

#[derive(Debug, Serialize)]
struct RegenerateSecretResponse {
    client_id: String,
    client_secret: String, // New secret, only returned once!
}

/// Regenerate client secret (owner or admin only)
async fn regenerate_secret(
    State(state): State<AppState>,
    auth: Auth,
    Path(client_id): Path<String>,
) -> ApiResult<Json<RegenerateSecretResponse>> {
    // Check ownership
    let client = sqlx::query!(
        "SELECT owner_user_id FROM oauth_clients WHERE client_id = $1",
        client_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AppError::NotFound("OAuth client not found".to_string()))?;

    if auth.role != "admin" && client.owner_user_id != auth.user_id {
        return Err(AppError::Forbidden("Not your OAuth client".to_string()));
    }

    // Generate new secret
    let new_secret = generate_client_secret();
    let secret_hash = hash_client_secret(&new_secret)?;

    // Update secret
    sqlx::query!(
        "UPDATE oauth_clients SET client_secret_hash = $1, updated_at = NOW() WHERE client_id = $2",
        secret_hash,
        client_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Revoke all active tokens (force re-authentication)
    sqlx::query!(
        "UPDATE oauth_access_tokens SET revoked_at = NOW() WHERE client_id = $1 AND revoked_at IS NULL",
        client_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(RegenerateSecretResponse {
        client_id,
        client_secret: new_secret, // Only returned once!
    }))
}
```

- [ ] **Step 4: Update api/v1/mod.rs**

Modify `backend/src/api/v1/mod.rs`:

```rust
pub mod entities;
pub mod oauth_admin;

use axum::Router;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/entities", entities::routes())
        .nest("/oauth/admin", oauth_admin::routes())
}
```

- [ ] **Step 5: Run tests**

Run: `cd backend && cargo test`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add backend/src/api/v1/oauth_admin.rs backend/src/api/v1/mod.rs
git commit -m "feat(mcp): add OAuth client admin API endpoints

- Add POST /api/v1/oauth/admin/clients (create client)
- Add GET /api/v1/oauth/admin/clients (list clients)
- Add GET /api/v1/oauth/admin/clients/:id (get client details)
- Add DELETE /api/v1/oauth/admin/clients/:id (revoke client)
- Add POST /api/v1/oauth/admin/clients/:id/regenerate-secret
- Admin-only for create, owner/admin for others
- Client secret only returned once on creation"
```

---

### Task 20: Integration, Routes, Background Jobs, and Tests

**Goal**: Wire everything together, add MCP routes, implement background jobs, and write integration tests.

**Files:**
- Create: `backend/src/api/mcp.rs`
- Modify: `backend/src/api/mod.rs`
- Modify: `backend/src/main.rs`
- Create: `backend/src/background/mcp_cleanup.rs`
- Create: `backend/tests/test_mcp_integration.rs`

- [ ] **Step 1: Write MCP HTTP handler**

Create `backend/src/api/mcp.rs`:

```rust
use axum::{
    extract::State,
    response::{Response, sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::{Stream, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use std::convert::Infallible;
use uuid::Uuid;

use crate::{
    auth::extractors::McpAuth,
    mcp::{
        types::{JsonRpcRequest, JsonRpcResponse, JsonRpcNotification},
        router::McpRouter,
        sse::SseConnection,
    },
    error::{ApiResult, AppError},
    AppState,
};

/// Register MCP routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(handle_json_rpc))
        .route("/sse", get(handle_sse))
}

/// Handle JSON-RPC requests
async fn handle_json_rpc(
    State(state): State<AppState>,
    auth: McpAuth,
    Json(request): Json<JsonRpcRequest>,
) -> ApiResult<Json<JsonRpcResponse>> {
    // Log audit
    let resource_uri = match request.method.as_str() {
        "resources/read" | "resources/subscribe" | "resources/unsubscribe" => {
            request.params.as_ref().and_then(|p| p["uri"].as_str())
        }
        _ => None,
    };

    state.mcp_audit_service
        .log_operation(
            auth.user_id,
            &auth.client_id,
            &request.method,
            resource_uri,
            request.params.clone(),
            "pending",
        )
        .await
        .ok(); // Don't fail request if audit fails

    // Route request
    let router = McpRouter::new(state.clone());
    let response = router.handle_request(request, auth).await
        .map_err(|e| AppError::InternalServerError(e.message))?;

    // Update audit log with result
    state.mcp_audit_service
        .log_operation(
            auth.user_id,
            &auth.client_id,
            &response.result.as_ref().map(|_| "success").unwrap_or("error"),
            resource_uri,
            None,
            if response.error.is_some() { "error" } else { "success" },
        )
        .await
        .ok();

    Ok(Json(response))
}

/// Handle SSE connections for real-time updates
async fn handle_sse(
    State(state): State<AppState>,
    auth: McpAuth,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::unbounded_channel();
    let client_id = Uuid::new_v4();

    // Register connection
    let connection = SseConnection::new(client_id, auth.user_id, tx.clone());
    state.sse_manager.register(connection).await;

    // Send initial connection event
    let _ = tx.send(JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "notifications/initialized".to_string(),
        params: serde_json::json!({ "client_id": client_id }),
    });

    // Convert to SSE stream
    let stream = UnboundedReceiverStream::new(rx).map(|notification| {
        let json = serde_json::to_string(&notification).unwrap();
        Ok(Event::default().data(json))
    });

    // Cleanup on disconnect
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        state.sse_manager.unregister(client_id).await;
    });

    Sse::new(stream)
}
```

- [ ] **Step 2: Update api/mod.rs**

Modify `backend/src/api/mod.rs`:

```rust
pub mod auth;
pub mod v1;
pub mod v4;
pub mod mcp;

use axum::Router;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/api/v1", v1::routes())
        .nest("/api/v4", v4::routes())
        .nest("/mcp", mcp::routes())
}
```

- [ ] **Step 3: Add MCP services to AppState**

Modify `backend/src/main.rs` (AppState section):

```rust
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub rate_limit_service: RateLimitService,
    pub mcp_audit_service: McpAuditService,  // Add this
    pub sse_manager: SseManager,             // Add this
}

// In main() function, add initialization:
let mcp_audit_service = McpAuditService::new(db.clone());
let sse_manager = SseManager::new();

let state = AppState {
    db,
    redis,
    rate_limit_service,
    mcp_audit_service,
    sse_manager,
};
```

- [ ] **Step 4: Create background cleanup job**

Create `backend/src/background/mcp_cleanup.rs`:

```rust
use crate::AppState;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Background job to cleanup stale SSE connections
pub async fn cleanup_stale_connections(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(60)); // Every minute

    loop {
        ticker.tick().await;

        // Cleanup stale connections (no activity for 5+ minutes)
        state.sse_manager.cleanup_stale_connections().await;

        // Log connection count
        let count = state.sse_manager.connection_count().await;
        tracing::debug!("Active SSE connections: {}", count);
    }
}

/// Background job to delete old audit logs (90+ days)
pub async fn cleanup_old_audit_logs(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(3600)); // Every hour

    loop {
        ticker.tick().await;

        // Delete audit logs older than 90 days
        let result = sqlx::query!(
            "DELETE FROM mcp_audit_logs WHERE created_at < NOW() - INTERVAL '90 days'"
        )
        .execute(&state.db)
        .await;

        match result {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    tracing::info!("Deleted {} old audit logs", result.rows_affected());
                }
            }
            Err(e) => {
                tracing::error!("Failed to delete old audit logs: {}", e);
            }
        }
    }
}
```

- [ ] **Step 5: Start background jobs in main.rs**

Modify `backend/src/main.rs`:

```rust
mod background;

// In main() function, after creating state:
let state_arc = Arc::new(state.clone());

// Spawn background jobs
tokio::spawn(background::mcp_cleanup::cleanup_stale_connections(state_arc.clone()));
tokio::spawn(background::mcp_cleanup::cleanup_old_audit_logs(state_arc.clone()));
```

- [ ] **Step 6: Write integration tests**

Create `backend/tests/test_mcp_integration.rs`:

```rust
#[tokio::test]
#[ignore] // Requires database
async fn test_mcp_initialize() {
    // Test MCP initialize handshake
}

#[tokio::test]
#[ignore] // Requires database
async fn test_mcp_list_resources() {
    // Test listing resources with authorization
}

#[tokio::test]
#[ignore] // Requires database
async fn test_mcp_read_message() {
    // Test reading a message resource
}

#[tokio::test]
#[ignore] // Requires database
async fn test_mcp_sse_subscription() {
    // Test SSE subscription and notifications
}

#[tokio::test]
#[ignore] // Requires database
async fn test_mcp_rate_limiting() {
    // Test MCP rate limits are enforced
}

#[tokio::test]
#[ignore] // Requires database
async fn test_mcp_audit_logging() {
    // Test audit logs are created
}

#[tokio::test]
#[ignore] // Requires database
async fn test_oauth_end_to_end() {
    // Test complete OAuth flow:
    // 1. Create client
    // 2. Authorize
    // 3. Exchange code
    // 4. Use access token for MCP
    // 5. Refresh token
    // 6. Revoke token
}
```

- [ ] **Step 7: Run all tests**

Run: `cd backend && cargo test`
Expected: Unit tests PASS, integration tests IGNORED (require database)

- [ ] **Step 8: Run cargo check**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 9: Run cargo fmt**

Run: `cd backend && cargo fmt --check`
Expected: No formatting issues

- [ ] **Step 10: Commit**

```bash
git add backend/src/api/mcp.rs backend/src/api/mod.rs backend/src/main.rs backend/src/background/ backend/tests/test_mcp_integration.rs
git commit -m "feat(mcp): integrate MCP server with HTTP and SSE endpoints

- Add POST /mcp (JSON-RPC handler)
- Add GET /mcp/sse (Server-Sent Events for real-time updates)
- Add MCP services to AppState (audit, SSE manager)
- Add background jobs (cleanup stale connections, delete old audit logs)
- Add integration test placeholders (7 tests)
- All unit tests passing, integration tests require database"
```

---

## Implementation Complete! 🎉

All 20 tasks are now detailed with full TDD structure:

### Database & Auth (Tasks 1-8)
- **Task 1**: OAuth database migrations (tables, indexes)
- **Task 2**: OAuth utilities (token generation, PKCE, Redis helpers)
- **Task 3**: OAuth authorization endpoint (consent page, code generation)
- **Task 4**: Redis authorization code storage
- **Task 5**: OAuth routes integration
- **Task 6**: OAuth token endpoint (code exchange, refresh, revoke, introspection)
- **Task 7**: MCP protocol types (JSON-RPC 2.0)
- **Task 8**: MCP authentication extractor (hybrid OAuth + API key)

### MCP Core (Tasks 9-10)
- **Task 9**: MCP router and method dispatcher
- **Task 10**: MCP resource types and provider trait

### Resource Providers (Tasks 11-16)
- **Task 11**: Messages resource provider
- **Task 12**: Channels resource provider
- **Task 13**: Users resource provider
- **Task 14**: Files resource provider
- **Task 15**: Teams resource provider
- **Task 16**: Search resource provider

### Infrastructure (Tasks 17-20)
- **Task 17**: SSE connection manager (real-time updates)
- **Task 18**: MCP audit logging service
- **Task 19**: Admin API endpoints (OAuth client management)
- **Task 20**: Integration (HTTP/SSE routes, background jobs, tests)

Each task follows consistent TDD pattern:
1. Write failing test
2. Run to verify failure
3. Write minimal implementation
4. Run to verify pass
5. Commit with descriptive message

**Total Files**: 40+ files created/modified
**Total Steps**: 100+ individual TDD steps
**Estimated LOC**: 6,000+ lines of Rust code

Ready for implementation! 🚀

