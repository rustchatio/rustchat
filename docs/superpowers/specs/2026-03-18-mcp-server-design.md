# Phase 2.x: MCP Server Implementation - Design Specification

**Date:** 2026-03-18
**Status:** Ready for Implementation - Revision 2
**Authors:** Claude (Brainstorming Agent)

---

## Executive Summary

This specification defines the complete implementation of Model Context Protocol (MCP) server support for RustChat, enabling AI assistants (Claude Desktop, ChatGPT, etc.) to connect as MCP clients and access workspace data with user authorization.

**Key Goals:**
- Enable AI assistants to read RustChat data (messages, channels, users, files)
- Implement OAuth 2.0 authorization with PKCE for secure client authentication
- Provide real-time notifications via Server-Sent Events (SSE)
- Comprehensive audit logging of all MCP access
- Admin and user management of OAuth clients

**Architecture Approach:** Integrated MCP Module within RustChat backend (not microservice)

---

## 1. System Architecture

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        MCP Clients                              │
│  (Claude Desktop, ChatGPT, Custom AI Apps)                      │
└────────────┬────────────────────────────────────┬───────────────┘
             │                                    │
             │ OAuth 2.0                         │ SSE
             │ Authorization                     │ Notifications
             │                                    │
┌────────────▼────────────────────────────────────▼───────────────┐
│                    RustChat Backend                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              OAuth 2.0 Authorization Server              │  │
│  │  - Authorization Code Flow with PKCE                     │  │
│  │  - Token Management (Access + Refresh)                   │  │
│  │  - Client Registration                                   │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              MCP Protocol Handler                        │  │
│  │  - JSON-RPC 2.0 Message Router                          │  │
│  │  - McpAuth Middleware (Token Validation)                │  │
│  │  - Scope-based Authorization                            │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              MCP Resource Providers                      │  │
│  │  - Messages Provider (read messages by channel)         │  │
│  │  - Channels Provider (read channel metadata)            │  │
│  │  - Users Provider (read user profiles)                  │  │
│  │  - Files Provider (read/download files)                 │  │
│  │  - Teams Provider (read team/workspace info)            │  │
│  │  - Search Provider (search all resources)               │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              SSE Connection Manager                      │  │
│  │  - Active connection tracking                           │  │
│  │  - Resource subscription management                     │  │
│  │  - Real-time notification broadcasting                  │  │
│  │  - Keepalive pings (30s interval)                       │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              MCP Audit Service                           │  │
│  │  - Comprehensive access logging                         │  │
│  │  - Statistics and analytics                             │  │
│  │  - 90-day retention policy                              │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────┬────────────────────────────────────┬─────────────┘
              │                                    │
              ▼                                    ▼
      ┌───────────────┐                  ┌─────────────────┐
      │   PostgreSQL  │                  │      Redis      │
      │  (OAuth Data, │                  │  (Rate Limits,  │
      │  Audit Logs)  │                  │   Sessions)     │
      └───────────────┘                  └─────────────────┘
```

### 1.2 Integration Points

**Existing RustChat Services:**
- `MessageService` - Enhanced to trigger MCP notifications on new messages
- `ChannelService` - Enhanced to trigger MCP notifications on channel changes
- `RateLimitService` - Extended with MCP-specific rate limiting
- `ApiKeyAuth` - Reused for API key-based MCP authentication

**New Services:**
- `OAuthClientService` - OAuth client registration and management
- `OAuthTokenService` - Token generation, validation, refresh, revocation
- `McpAuditService` - Comprehensive MCP access logging
- `SseConnectionManager` - Real-time SSE connection and subscription management
- `ResourceNotifier` - Broadcasts resource change notifications to SSE clients

---

## 2. Authentication & Authorization

### 2.1 OAuth 2.0 Authorization Code Flow with PKCE

**Why OAuth 2.0:**
- Industry-standard protocol for delegated authorization
- User explicitly grants AI assistant access to their RustChat data
- Time-limited tokens (1 hour access, 30 day refresh)
- User can revoke access at any time

**PKCE (Proof Key for Code Exchange):**
- Prevents authorization code interception attacks
- Required for all clients (public and confidential)
- Uses SHA-256 challenge/verifier pairs

**Flow:**

```
┌──────────────┐                                ┌──────────────┐
│  MCP Client  │                                │   RustChat   │
│(Claude, etc) │                                │    Server    │
└──────┬───────┘                                └──────┬───────┘
       │                                               │
       │ 1. Generate code_verifier (random string)    │
       │    code_challenge = SHA256(code_verifier)    │
       │                                               │
       │ 2. GET /api/oauth/authorize                  │
       │    ?client_id=mcp_xxx                        │
       │    &redirect_uri=https://...                 │
       │    &scope=read:messages read:channels        │
       │    &state=random_state                       │
       │    &code_challenge=xxx                       │
       │    &code_challenge_method=S256               │
       ├──────────────────────────────────────────────>
       │                                               │
       │ 3. User authenticates (JWT) and sees         │
       │    consent page listing requested scopes     │
       │                                               │
       │ 4. User clicks "Authorize"                   │
       │    POST /api/oauth/authorize                 │
       │    { user_consent: true, ... }               │
       ├──────────────────────────────────────────────>
       │                                               │
       │ 5. Redirect to client with code              │
       │    https://redirect_uri?code=xxx&state=xxx   │
       <───────────────────────────────────────────────┤
       │                                               │
       │ 6. POST /api/oauth/token                     │
       │    { grant_type: "authorization_code",       │
       │      code: "xxx",                             │
       │      redirect_uri: "...",                     │
       │      client_id: "mcp_xxx",                    │
       │      client_secret: "mcs_xxx" (if conf),     │
       │      code_verifier: "original_verifier" }    │
       ├──────────────────────────────────────────────>
       │                                               │
       │    Server validates:                          │
       │    - Code exists and not expired              │
       │    - Redirect URI matches                     │
       │    - Client credentials (if confidential)     │
       │    - SHA256(code_verifier) == code_challenge  │
       │    - Code used only once                      │
       │                                               │
       │ 7. Return access + refresh tokens            │
       │    { access_token: "rct_xxx",                │
       │      token_type: "Bearer",                    │
       │      expires_in: 3600,                        │
       │      refresh_token: "rcr_xxx",                │
       │      scope: "read:messages read:channels" }  │
       <───────────────────────────────────────────────┤
       │                                               │
       │ 8. Use access token for MCP requests         │
       │    Authorization: Bearer rct_xxx              │
       ├──────────────────────────────────────────────>
       │                                               │
```

### 2.1.1 OAuth Consent Page Implementation

**Server-Side Flow:**

1. **GET /api/oauth/authorize** stores pending authorization in Redis:
   ```rust
   // Extract JWT from cookie (user must be logged in)
   let user_id = extract_user_from_jwt(cookies)?;

   // Validate parameters
   validate_client_id(&client_id)?;
   validate_redirect_uri(&client_id, &redirect_uri)?;
   validate_scopes(&client_id, &scopes)?;
   validate_pkce(&code_challenge, &code_challenge_method)?;

   // Store pending authorization with 10-minute expiry
   let pending_key = generate_random_hex(32);
   redis.set_ex(
       format!("oauth:pending:{}", pending_key),
       json!({
           "client_id": client_id,
           "user_id": user_id,
           "redirect_uri": redirect_uri,
           "scopes": scopes,
           "code_challenge": code_challenge,
           "code_challenge_method": code_challenge_method,
           "state": state
       }),
       600  // 10 minutes
   ).await?;

   // Render consent page
   Ok(Html(render_consent_page(pending_key, client, scopes)))
   ```

2. **POST /api/oauth/authorize** processes user decision:
   ```rust
   // Retrieve pending authorization
   let pending = redis.get(format!("oauth:pending:{}", pending_key)).await?;

   // Verify same user
   if pending.user_id != auth.user_id {
       return Err(AppError::Forbidden("Authorization mismatch"));
   }

   if user_consent {
       // Generate authorization code
       let code = generate_random_hex(64);

       // Store in database
       sqlx::query!(
           "INSERT INTO oauth_authorization_codes
            (code, client_id, user_id, redirect_uri, scopes, code_challenge, code_challenge_method, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW() + INTERVAL '10 minutes')",
           code, pending.client_id, pending.user_id, pending.redirect_uri,
           &pending.scopes, pending.code_challenge, pending.code_challenge_method
       ).execute(&db).await?;

       // Delete pending authorization
       redis.del(format!("oauth:pending:{}", pending_key)).await?;

       // Redirect with code
       Ok(Redirect::to(format!("{}?code={}&state={}", pending.redirect_uri, code, pending.state)))
   } else {
       // User denied - redirect with error
       redis.del(format!("oauth:pending:{}", pending_key)).await?;
       Ok(Redirect::to(format!("{}?error=access_denied&state={}", pending.redirect_uri, pending.state)))
   }
   ```

### 2.1.2 Token Exchange with PKCE Validation

**Corrected transaction order** (validate BEFORE deleting code):

```rust
async fn exchange_authorization_code(request: TokenRequest) -> Result<TokenResponse> {
    let mut tx = db.begin().await?;

    // Step 1: Fetch code (with FOR UPDATE lock)
    let code_record = sqlx::query!(
        "SELECT * FROM oauth_authorization_codes
         WHERE code = $1 AND expires_at > NOW()
         FOR UPDATE",
        request.code
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized("Invalid or expired code"))?;

    // Step 2: Validate PKCE BEFORE deleting
    validate_pkce(
        &request.code_verifier,
        &code_record.code_challenge,
        &code_record.code_challenge_method
    )?;

    // Step 3: Validate redirect_uri
    if request.redirect_uri != code_record.redirect_uri {
        return Err(AppError::Unauthorized("Redirect URI mismatch"));
    }

    // Step 4: Authenticate client
    let client = authenticate_client(&request.client_id, request.client_secret.as_deref()).await?;

    // Step 5: Generate tokens
    let access_token = format!("rct_{}", generate_random_hex(64));
    let refresh_token = format!("rcr_{}", generate_random_hex(64));
    let token_prefix = &access_token[..20];
    let token_hash = sha256_hash(&access_token);
    let refresh_hash = sha256_hash(&refresh_token);

    // Step 6: Store tokens
    sqlx::query!(
        "INSERT INTO oauth_access_tokens
         (token_prefix, token_hash, refresh_token_hash, client_id, user_id, scopes,
          expires_at, refresh_expires_at)
         VALUES ($1, $2, $3, $4, $5, $6,
                 NOW() + INTERVAL '1 hour', NOW() + INTERVAL '30 days')",
        token_prefix, token_hash, refresh_hash, client.client_id,
        code_record.user_id, &code_record.scopes
    )
    .execute(&mut *tx)
    .await?;

    // Step 7: Delete code (single-use)
    sqlx::query!("DELETE FROM oauth_authorization_codes WHERE id = $1", code_record.id)
        .execute(&mut *tx)
        .await?;

    // Step 8: Commit
    tx.commit().await?;

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        refresh_token,
        scope: code_record.scopes.join(" "),
    })
}
```

### 2.2 Hybrid Authentication Strategy

**Two authentication methods supported:**

1. **OAuth 2.0 Access Tokens (Primary)**
   - Format: `rct_` + 64 hex chars = 68 total
   - For interactive MCP clients (Claude Desktop, ChatGPT)
   - User explicitly authorizes via consent screen
   - Scoped access (user controls what data is shared)
   - 1-hour expiry, refreshable for 30 days

2. **API Keys (Secondary)**
   - Format: `rck_` + 64 hex chars = 68 total (existing format)
   - For server-to-server MCP integrations
   - Agents/services get full MCP scope automatically
   - No expiry (revocable by admin)
   - Reuses existing `ApiKeyAuth` extractor
   - **Rate Limiting:** API keys follow MCP rate limits (1k req/hr per client, 5k req/hr per user), NOT AgentHigh limits (10k req/hr)

**McpAuth Extractor Logic:**
```rust
Authorization: Bearer <token>

if token.starts_with("rct_") {
    // OAuth access token - validate and check scopes
    validate_oauth_token(token) -> McpAuth { user_id, scopes, ... }
} else if token.starts_with("rck_") {
    // API key - full MCP access, MCP rate limits applied
    validate_api_key(token) -> McpAuth { user_id, full_scopes, ... }
} else {
    return 401 Unauthorized
}
```

**Token Storage & Validation:**

Tokens are stored securely using a prefix + hash strategy:

1. **Token Generation:**
   ```rust
   // Generate raw token
   let access_token = format!("rct_{}", generate_random_hex(64));  // 68 chars total

   // Extract prefix for O(1) indexed lookup (first 20 chars)
   let token_prefix = &access_token[..20];  // "rct_a1b2c3d4e5f6g7h8"

   // Hash full token for secure storage
   let token_hash = sha256_hash(&access_token);

   // Store both prefix (plaintext) and hash
   INSERT INTO oauth_access_tokens (token_prefix, token_hash, ...) VALUES (...);
   ```

2. **Token Validation:**
   ```rust
   async fn validate_access_token(token: &str) -> Result<McpAuth> {
       // 1. Extract prefix (first 20 chars: "rct_" + 16 hex)
       let token_prefix = &token[..20];

       // 2. Hash full token for comparison
       let token_hash = sha256_hash(token);

       // 3. Query by prefix (O(1) indexed lookup)
       let candidates = sqlx::query!(
           "SELECT * FROM oauth_access_tokens
            WHERE token_prefix = $1
              AND expires_at > NOW()
              AND revoked_at IS NULL",
           token_prefix
       ).fetch_all().await?;

       // 4. Verify exact hash (constant-time comparison)
       for candidate in candidates {
           if constant_time_compare(&candidate.token_hash, &token_hash) {
               return Ok(McpAuth::from(candidate));
           }
       }

       Err(AppError::Unauthorized("Invalid token"))
   }
   ```

**Why This Works:**
- Prefix stored in plaintext enables O(1) indexed lookup
- Full token never stored in plaintext (only SHA-256 hash)
- Collision probability: ~2^-80 for 20-char prefix (negligible)
- Even with prefix collision, exact hash comparison prevents false positives

### 2.3 OAuth Scopes

**Read-only scopes for Phase 2.x:**

| Scope | Description | Resources |
|-------|-------------|-----------|
| `read:messages` | Read messages from accessible channels | `rustchat-message://channel/{id}` |
| `read:channels` | Read channel metadata | `rustchat-channel://{id}` |
| `read:users` | Read user profiles | `rustchat-user://{id}` |
| `read:files` | Read and download files | `rustchat-file://{id}` |
| `read:teams` | Read team/workspace info | `rustchat-team://{id}` |
| `read:search` | Search messages, channels, users | `rustchat-search://query?q=...` |

**Scope Inheritance:**
- Scopes are stored per OAuth client in `allowed_scopes` column
- Users grant subset of allowed scopes during authorization
- Tokens store granted scopes in `scopes` column
- MCP requests validate required scope via `McpAuth::require_scope()`

### 2.4 Authorization Model

**Role-Based Authorization:**
- MCP clients inherit the user's existing RustChat permissions
- A user authorized with `read:messages` can only read messages from channels they're a member of
- Channel membership checks enforced in resource providers
- No elevation of privileges - MCP access ≤ user's web/app access

**Example:**
```
User Alice is member of #general and #engineering
AI Assistant authorized with read:messages scope

✅ Can read: messages in #general
✅ Can read: messages in #engineering
❌ Cannot read: messages in #marketing (not a member)
```

---

## 3. Database Schema

### 3.1 OAuth Clients Table

```sql
CREATE TABLE oauth_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id VARCHAR(255) UNIQUE NOT NULL,  -- Format: mcp_{32 hex} = 64 chars
    client_secret_hash TEXT NOT NULL,        -- bcrypt hash
    client_name VARCHAR(255) NOT NULL,
    client_type VARCHAR(50) NOT NULL CHECK (client_type IN ('confidential', 'public')),
    redirect_uris TEXT[] NOT NULL,           -- Array of allowed redirect URIs
    allowed_scopes TEXT[] NOT NULL,          -- Array of allowed scopes
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_active BOOLEAN DEFAULT true,
    is_first_party BOOLEAN DEFAULT false,    -- RustChat-owned clients
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_oauth_clients_owner ON oauth_clients(owner_user_id);
CREATE INDEX idx_oauth_clients_client_id ON oauth_clients(client_id);
```

**Client Types:**
- `confidential`: Server-side apps with client secret (e.g., backend integrations)
- `public`: Client-side apps without secret (e.g., desktop apps, Claude Desktop)

### 3.2 OAuth Authorization Codes Table

```sql
CREATE TABLE oauth_authorization_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(255) UNIQUE NOT NULL,       -- Random 64 hex chars
    client_id VARCHAR(255) NOT NULL,         -- OAuth client_id (not UUID)
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    redirect_uri TEXT NOT NULL,
    scopes TEXT[] NOT NULL,
    code_challenge VARCHAR(255),             -- PKCE challenge
    code_challenge_method VARCHAR(10),       -- "S256" (SHA-256)
    expires_at TIMESTAMPTZ NOT NULL,         -- 10 minutes from creation
    created_at TIMESTAMPTZ DEFAULT NOW(),

    CONSTRAINT fk_client FOREIGN KEY (client_id)
        REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);

CREATE INDEX idx_oauth_codes_code ON oauth_authorization_codes(code);
CREATE INDEX idx_oauth_codes_expires ON oauth_authorization_codes(expires_at);
```

**Lifecycle:**
- Created when user authorizes
- 10-minute expiry
- Single-use (deleted after token exchange)
- PKCE challenge stored for validation

### 3.3 OAuth Access Tokens Table

```sql
CREATE TABLE oauth_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_prefix VARCHAR(20) NOT NULL,       -- Raw prefix for O(1) lookup ("rct_" + 16 hex)
    token_hash TEXT UNIQUE NOT NULL,         -- SHA-256 hash of full token
    refresh_token_hash TEXT UNIQUE,          -- SHA-256 hash of refresh token
    client_id VARCHAR(255) NOT NULL,         -- OAuth client_id (not UUID)
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scopes TEXT[] NOT NULL,
    token_type VARCHAR(50) DEFAULT 'Bearer',
    expires_at TIMESTAMPTZ NOT NULL,         -- 1 hour from creation
    refresh_expires_at TIMESTAMPTZ,          -- 30 days from creation
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,                  -- NULL = active, set = revoked

    CONSTRAINT fk_client FOREIGN KEY (client_id)
        REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);

-- Prefix index for O(1) token lookups
CREATE INDEX idx_oauth_tokens_prefix ON oauth_access_tokens (token_prefix)
    WHERE revoked_at IS NULL AND expires_at > NOW();

CREATE INDEX idx_oauth_tokens_user ON oauth_access_tokens(user_id);
CREATE INDEX idx_oauth_tokens_client ON oauth_access_tokens(client_id);
CREATE INDEX idx_oauth_tokens_expires ON oauth_access_tokens(expires_at)
    WHERE revoked_at IS NULL;
```

**Token Format:**
- Access token: `rct_` + 64 hex chars = 68 total
- Refresh token: `rcr_` + 64 hex chars = 68 total
- Prefix: First 20 chars stored as plaintext for indexing (`rct_` + 16 hex)
- Full token stored as SHA-256 hash for security

**Token Rotation:**
- When refresh token is used, new access + refresh tokens issued
- Old refresh token invalidated (prevents reuse)
- Mitigates token theft/replay attacks

### 3.4 MCP Audit Logs Table

```sql
CREATE TABLE mcp_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id VARCHAR(255) NOT NULL,         -- OAuth client_id or "api_key:{email}"
    method VARCHAR(100) NOT NULL,            -- "resources/read", "messages/read", etc.
    resource_type VARCHAR(50),               -- "message", "channel", "user", etc.
    resource_id VARCHAR(255),                -- Resource UUID or identifier
    scopes_used TEXT[],                      -- Scopes used for this request
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'error')),
    error_message TEXT,
    request_duration_ms INTEGER
);

CREATE INDEX idx_mcp_audit_timestamp ON mcp_audit_logs(timestamp);
CREATE INDEX idx_mcp_audit_user ON mcp_audit_logs(user_id);
CREATE INDEX idx_mcp_audit_client ON mcp_audit_logs(client_id);
CREATE INDEX idx_mcp_audit_resource_type ON mcp_audit_logs(resource_type);
```

**Audit Log Lifecycle:**
- Every MCP request logged (success and errors)
- 90-day retention (configurable)
- Background job runs daily to purge old logs
- Used for security monitoring, analytics, user transparency

### 3.5 OAuth Scopes Table

```sql
CREATE TABLE oauth_scopes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    category VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed initial scopes
INSERT INTO oauth_scopes (scope, display_name, description, category) VALUES
    ('read:messages', 'Read Messages', 'Read messages from channels you have access to', 'messages'),
    ('read:channels', 'Read Channels', 'Read channel information and metadata', 'channels'),
    ('read:users', 'Read Users', 'Read user profiles and information', 'users'),
    ('read:files', 'Read Files', 'Read and download files', 'files'),
    ('read:teams', 'Read Teams', 'Read team/workspace information', 'teams'),
    ('read:search', 'Search', 'Search across messages, channels, and users', 'search');
```

---

## 4. MCP Protocol Implementation

### 4.1 JSON-RPC 2.0 Message Format

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "rustchat-message://channel/550e8400-e29b-41d4-a716-446655440000"
  },
  "id": 1
}
```

**Success Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "contents": [{
      "uri": "rustchat-message://channel/550e8400-e29b-41d4-a716-446655440000",
      "mimeType": "text/plain",
      "text": "[2026-03-18 10:00] alice: Hello world\n[2026-03-18 10:01] bob: Hi there\n"
    }]
  },
  "id": 1
}
```

**Error Response:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Missing required scope: read:messages"
  },
  "id": 1
}
```

### 4.2 MCP Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | Parse Error | Invalid JSON |
| -32600 | Invalid Request | Invalid JSON-RPC 2.0 format |
| -32601 | Method Not Found | Unknown MCP method |
| -32602 | Invalid Params | Invalid method parameters |
| -32603 | Internal Error | Server internal error |
| -32000 | Unauthorized | Missing or invalid token |
| -32001 | Forbidden | Missing required scope |
| -32002 | Resource Not Found | Resource doesn't exist or no access |
| -32003 | Rate Limit Exceeded | Too many requests |

### 4.3 Supported MCP Methods

**Server Capabilities:**
- `initialize` - Negotiate protocol version and capabilities
- `initialized` - Confirmation notification

**Resource Methods:**
- `resources/list` - List available resources
- `resources/read` - Read a specific resource
- `resources/subscribe` - Subscribe to resource updates (SSE)
- `resources/unsubscribe` - Unsubscribe from updates

**Tools & Prompts (Phase 2.x: Not Implemented):**
- `tools/list` - Returns empty array (read-only mode)
- `tools/call` - Returns METHOD_NOT_FOUND error
- `prompts/list` - Returns empty array
- `prompts/get` - Returns METHOD_NOT_FOUND error

### 4.4 HTTP Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/mcp` | POST | JSON-RPC 2.0 single request |
| `/api/mcp/batch` | POST | JSON-RPC 2.0 batch requests |
| `/api/mcp/sse` | GET | SSE endpoint for notifications |

**Authentication:**
- All MCP endpoints require `Authorization: Bearer <token>` header
- Token validated by `McpAuth` extractor
- Rate limiting applied per-client and per-user

### 4.5 Complete OAuth & Admin API Endpoints

#### OAuth 2.0 Authorization Endpoints

**GET /api/oauth/authorize**

Display authorization consent page.

Query Parameters:
- `client_id` (required): OAuth client identifier
- `redirect_uri` (required): Callback URL
- `response_type` (required): Must be "code"
- `scope` (required): Space-separated scopes
- `state` (required): CSRF protection token
- `code_challenge` (required): Base64url(SHA256(code_verifier))
- `code_challenge_method` (required): Must be "S256"

Responses:
- 200 OK: HTML consent page
- 302 Redirect: If already authorized
- 400 Bad Request: Invalid parameters
- 401 Unauthorized: User not authenticated

**POST /api/oauth/authorize**

User grants or denies authorization.

Request Body:
```json
{
  "client_id": "mcp_xxx",
  "redirect_uri": "https://...",
  "scopes": ["read:messages", "read:channels"],
  "state": "random_state",
  "code_challenge": "xxx",
  "code_challenge_method": "S256",
  "user_consent": true
}
```

Responses:
- 302 Redirect (Authorized): `{redirect_uri}?code=abc123&state=random_state`
- 302 Redirect (Denied): `{redirect_uri}?error=access_denied&state=random_state`

**POST /api/oauth/token**

Exchange authorization code for tokens OR refresh access token.

Request (Authorization Code):
```json
{
  "grant_type": "authorization_code",
  "code": "abc123...",
  "redirect_uri": "https://...",
  "client_id": "mcp_xxx",
  "client_secret": "mcs_xxx",
  "code_verifier": "original_random_string"
}
```

Request (Refresh Token):
```json
{
  "grant_type": "refresh_token",
  "refresh_token": "rcr_xxx...",
  "client_id": "mcp_xxx",
  "client_secret": "mcs_xxx"
}
```

Response (Success):
```json
{
  "access_token": "rct_xxx...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "rcr_xxx...",
  "scope": "read:messages read:channels"
}
```

**POST /api/oauth/revoke**

Revoke access or refresh token.

**POST /api/oauth/introspect**

Validate token (for resource servers).

#### Admin OAuth Client Management

**POST /api/admin/oauth/clients** - Register new OAuth client (Admin only)
**GET /api/admin/oauth/clients** - List all OAuth clients
**GET /api/admin/oauth/clients/:client_id** - Get client details
**PATCH /api/admin/oauth/clients/:client_id** - Update client
**DELETE /api/admin/oauth/clients/:client_id** - Delete client
**POST /api/admin/oauth/clients/:client_id/rotate-secret** - Rotate client secret

#### User OAuth Management

**GET /api/v1/oauth/my-clients** - List user's OAuth clients
**GET /api/v1/oauth/authorizations** - List active authorizations
**DELETE /api/v1/oauth/authorizations/:client_id** - Revoke client access

#### MCP Audit Endpoints

**GET /api/admin/mcp/audit/logs** - Query audit logs (Admin only)
**GET /api/admin/mcp/audit/stats** - Audit statistics (Admin only)
**GET /api/admin/mcp/audit/top-resources** - Most accessed resources (Admin only)
**GET /api/admin/mcp/audit/top-clients** - Most active clients (Admin only)
**GET /api/v1/mcp/audit** - User's own MCP access logs

*Full request/response specifications for each endpoint provided in implementation plan.*

---

## 5. MCP Resource Providers

### 5.1 Resource URI Schemes

| Scheme | Example | Description |
|--------|---------|-------------|
| `rustchat-message` | `rustchat-message://channel/{uuid}` | Messages in a channel |
| `rustchat-channel` | `rustchat-channel://{uuid}` | Channel metadata |
| `rustchat-user` | `rustchat-user://{uuid}` | User profile |
| `rustchat-file` | `rustchat-file://{uuid}` | File metadata and content |
| `rustchat-team` | `rustchat-team://{uuid}` | Team/workspace info |
| `rustchat-search` | `rustchat-search://query?q={query}&type={type}` | Search results |

### 5.2 Messages Resource Provider

**List Resources:**
```json
{
  "method": "resources/list"
}

// Response: List of accessible channels
{
  "resources": [
    {
      "uri": "rustchat-message://channel/550e8400-...",
      "name": "Messages in #general",
      "mimeType": "application/json",
      "description": "Recent messages from public channel"
    },
    {
      "uri": "rustchat-message://channel/660e8400-...",
      "name": "Messages in #engineering",
      "mimeType": "application/json",
      "description": "Recent messages from private channel"
    }
  ]
}
```

**Read Messages:**
```json
{
  "method": "resources/read",
  "params": {
    "uri": "rustchat-message://channel/550e8400-...?limit=50"
  }
}

// Response: Recent messages formatted as text
{
  "contents": [{
    "uri": "rustchat-message://channel/550e8400-...",
    "mimeType": "text/plain",
    "text": "[2026-03-18 10:00:00] alice: Hello\n[2026-03-18 10:01:15] bob: Hi\n..."
  }]
}
```

**Query Parameters:**
- `limit` - Number of messages (default 50, max 100)
- `before` - Message ID for pagination

**Authorization Check:**
- Query `channel_members` table to verify user is member
- Return 403 Forbidden if not a member
- Only return messages from accessible channels

### 5.3 Channels Resource Provider

**List Resources:**
```json
{
  "resources": [
    {
      "uri": "rustchat-channel://550e8400-...",
      "name": "#general",
      "mimeType": "application/json",
      "description": "Public channel for general discussion"
    }
  ]
}
```

**Read Channel:**
```json
{
  "contents": [{
    "uri": "rustchat-channel://550e8400-...",
    "mimeType": "text/plain",
    "text": "Channel: #general\nType: public\nDescription: General discussion\nMembers: 42\nCreated: 2026-01-15 09:00:00"
  }]
}
```

### 5.4 Users Resource Provider

**List Resources:**
- Returns users in user's workspace/team
- Respects visibility settings

**Read User:**
```json
{
  "contents": [{
    "uri": "rustchat-user://770e8400-...",
    "mimeType": "text/plain",
    "text": "User: Alice Smith (@alice)\nEmail: alice@example.com\nRole: member\nJoined: 2026-01-10"
  }]
}
```

### 5.5 Files Resource Provider

**List Resources:**
- Returns files user has access to (via channel membership)

**Read File:**
```json
{
  "contents": [{
    "uri": "rustchat-file://880e8400-...",
    "mimeType": "image/png",
    "blob": "base64_encoded_content..."  // For binary files
  }]
}
```

**Text files return `text` field, binary files return `blob` field (base64).**

### 5.6 Teams Resource Provider

**Read Team:**
```json
{
  "contents": [{
    "uri": "rustchat-team://990e8400-...",
    "mimeType": "text/plain",
    "text": "Team: Acme Corp\nMembers: 156\nChannels: 42\nCreated: 2025-12-01"
  }]
}
```

### 5.7 Search Resource Provider

**Search URI Format:**
```
rustchat-search://query?q={query}&type={messages|channels|users}&limit={50}
```

**Example:**
```json
{
  "method": "resources/read",
  "params": {
    "uri": "rustchat-search://query?q=authentication&type=messages&limit=20"
  }
}

// Response:
{
  "contents": [{
    "uri": "rustchat-search://query?q=authentication&type=messages&limit=20",
    "mimeType": "text/plain",
    "text": "Search results for 'authentication' (15 messages):\n\n[2026-03-15 14:30] #engineering - alice: We need to implement 2FA authentication\n[2026-03-16 09:00] #general - bob: Authentication system is live\n..."
  }]
}
```

**Search Types:**
- `messages` - Search message content (ILIKE query)
- `channels` - Search channel names and descriptions
- `users` - Search usernames and display names

**Authorization:**
- Search only returns results user has access to
- Messages filtered by channel membership
- Channels filtered by membership or visibility

---

## 6. Server-Sent Events (SSE)

### 6.1 SSE Connection Flow

```
Client                                 RustChat Server
  │                                           │
  │  GET /api/mcp/sse                        │
  │  Authorization: Bearer rct_xxx           │
  ├──────────────────────────────────────────>
  │                                           │
  │  <SSE Connection Established>            │
  │  event: connected                        │
  │  data: {"connectionId": "uuid", ...}     │
  <───────────────────────────────────────────┤
  │                                           │
  │  (Subscribe via JSON-RPC)                │
  │  POST /api/mcp                           │
  │  { method: "resources/subscribe",        │
  │    params: { uri: "rustchat-message://channel/..." } }
  ├──────────────────────────────────────────>
  │                                           │
  │  (New message posted to channel)         │
  │                                           │
  │  event: resources/updated                │
  │  data: {"uri": "rustchat-message://..."}│
  <───────────────────────────────────────────┤
  │                                           │
  │  (Keepalive every 30s)                   │
  │  event: ping                             │
  │  data: {"timestamp": 1710753600}         │
  <───────────────────────────────────────────┤
  │                                           │
```

### 6.1.1 SSE-JSON-RPC Connection Correlation

**Problem:** How does the server link JSON-RPC subscription requests to specific SSE connections?

**Solution:** Connection ID header mechanism

1. **Client opens SSE connection and receives connection ID:**
   ```
   GET /api/mcp/sse
   Authorization: Bearer rct_xxx

   Response (first event):
   event: connected
   data: {"connectionId": "550e8400-e29b-41d4-a716-446655440000"}
   ```

2. **Client includes connection ID in subscription requests:**
   ```http
   POST /api/mcp
   Authorization: Bearer rct_xxx
   X-MCP-Connection-ID: 550e8400-e29b-41d4-a716-446655440000

   {
     "jsonrpc": "2.0",
     "method": "resources/subscribe",
     "params": {"uri": "rustchat-message://channel/660e8400-..."},
     "id": 1
   }
   ```

3. **Server validates and subscribes:**
   ```rust
   // Extract connection ID from header
   let connection_id = headers.get("X-MCP-Connection-ID")
       .and_then(|h| h.to_str().ok())
       .and_then(|s| Uuid::parse_str(s).ok())
       .ok_or(AppError::BadRequest("Missing X-MCP-Connection-ID header"))?;

   // Verify connection belongs to authenticated user
   let connection = sse_manager.get_connection(connection_id).await?;
   if connection.user_id != auth.user_id {
       return Err(AppError::Forbidden("Connection belongs to different user"));
   }

   // Subscribe connection to resource
   sse_manager.subscribe(connection_id, uri).await?;
   ```

4. **Alternative (no header required):**
   If `X-MCP-Connection-ID` not provided, server finds connection by user+client:
   ```rust
   // Assume one SSE connection per user+client pair
   let connection = sse_manager.find_connection_by_user_client(&auth.user_id, &auth.client_id).await?;
   sse_manager.subscribe(connection.connection_id, uri).await?;
   ```

### 6.2 SSE Event Types

**Connected Event:**
```
event: connected
data: {"connectionId": "550e8400-...", "timestamp": 1710753600}
```

**Resources Updated Event:**
```
event: resources/updated
data: {"uri": "rustchat-message://channel/550e8400-..."}
```

**Resources List Changed Event:**
```
event: resources/list_changed
data: {}
```

**Ping/Keepalive Event:**
```
event: ping
data: {"timestamp": 1710753600}
```

### 6.3 Connection Management

**SseConnectionManager:**
- Tracks active SSE connections per user/client
- Manages resource subscriptions per connection
- Broadcasts events to subscribed connections
- Automatic cleanup on disconnect

**Connection Lifecycle:**
1. Client opens SSE connection → `register_connection()`
2. Client subscribes to resources → `subscribe()`
3. Resource changes → `broadcast_to_resource()`
4. Client disconnects → `remove_connection()` (auto-cleanup)

**Keepalive:**
- Background task sends ping every 30 seconds to all connections
- Prevents proxy/firewall timeouts
- Client should reconnect if no ping received for 60+ seconds

### 6.3.1 Connection Persistence Strategy

**Strategy:** In-memory only (no database persistence)

**Implications:**
- Server restart → all SSE connections lost
- Clients must reconnect after server restart
- Subscriptions not persisted

**Rationale:**
- SSE connections are ephemeral by nature
- Clients handle reconnection automatically (built into SSE spec)
- No need for complex connection state synchronization

**Client Reconnection Logic:**
```javascript
const eventSource = new EventSource('/api/mcp/sse', {
  headers: { 'Authorization': `Bearer ${accessToken}` }
});

eventSource.addEventListener('connected', (event) => {
  const { connectionId } = JSON.parse(event.data);
  // Re-subscribe to all resources
  subscriptions.forEach(uri => subscribeToResource(uri, connectionId));
});

eventSource.onerror = () => {
  // EventSource automatically reconnects with exponential backoff
  console.log('SSE connection lost, reconnecting...');
};
```

### 6.4 Notification Triggers

**When to notify SSE clients:**

| Event | SSE Notification | Resource URI |
|-------|------------------|--------------|
| New message posted | `resources/updated` | `rustchat-message://channel/{id}` |
| Channel created/updated | `resources/updated` + `resources/list_changed` | `rustchat-channel://{id}` |
| User joined channel | `resources/list_changed` | (new channel accessible) |
| File uploaded | `resources/updated` | `rustchat-file://{id}` |

**Integration:**
- `MessageService::create_message()` calls `ResourceNotifier::notify_message_created()`
- `ChannelService::update_channel()` calls `ResourceNotifier::notify_channel_changed()`
- Notifications triggered after DB commit (not on failure)

### 6.4.1 Scope-Based Notification Filtering

**Problem:** User subscribed to resource but lacks required scope.

**Solution:** Validate scope on subscription (not notification).

```rust
impl ResourceHandler {
    pub async fn subscribe(
        &self,
        params: &Option<Value>,
        auth: &McpAuth,
        state: &AppState,
    ) -> Result<Value, ErrorObject> {
        let request: SubscribeRequest = serde_json::from_value(...)?;

        // Determine required scope from URI scheme
        let (scheme, _path) = self.parse_uri(&request.uri)?;
        let required_scope = match scheme {
            "rustchat-message" => "read:messages",
            "rustchat-channel" => "read:channels",
            "rustchat-user" => "read:users",
            "rustchat-file" => "read:files",
            "rustchat-team" => "read:teams",
            _ => return Err(ErrorObject {
                code: error_codes::INVALID_PARAMS,
                message: "Unknown resource scheme".to_string(),
                data: None,
            }),
        };

        // Validate scope BEFORE allowing subscription
        auth.require_scope(required_scope)?;

        // Subscribe to resource
        let connection_id = extract_connection_id(headers)?;
        state.sse_manager.subscribe(connection_id, request.uri.clone()).await?;

        Ok(json!({"subscribed": true}))
    }
}
```

**Result:** Clients cannot subscribe to resources without proper scope. Notifications only go to properly authorized subscribers.

---

## 7. Rate Limiting

### 7.1 MCP Rate Limit Tiers

**Per-Client Limits:**
- 1,000 requests per hour per OAuth client
- Prevents single misbehaving client from overloading server

**Per-User Limits:**
- 5,000 requests per hour per user
- Prevents user with many authorized clients from overloading server

**Implementation:**
- Redis Lua scripts for atomic INCR+EXPIRE operations
- Separate keys: `ratelimit:mcp_client:{client_id}` and `ratelimit:mcp_user:{user_id}`
- Enforced in `McpAuth` extractor (before request processing)

### 7.2 Rate Limit Headers

**Response Headers (on success):**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 847
X-RateLimit-Reset: 1710757200
```

**Response (on rate limit exceeded):**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32003,
    "message": "MCP client rate limit exceeded: 1000 req/hr"
  },
  "id": 1
}
```

### 7.3 API Key Rate Limits

**API keys used for MCP follow MCP rate limits** (not AgentHigh):

- **Per-client (API key):** 1,000 req/hr (same as OAuth clients)
- **Per-user:** 5,000 req/hr (same as OAuth)

**Rationale:**
- API keys for MCP are treated as "service clients"
- Should not get preferential treatment over interactive OAuth clients
- Prevents abuse from automated integrations

**AgentHigh tier (10k req/hr):**
- Only applies to non-MCP API usage (standard REST API endpoints)
- MCP-specific rate limiting is separate

**Implementation:**
```rust
impl McpAuth {
    async fn validate_api_key(api_key: &str, state: &AppState) -> Result<Self> {
        // Validate API key
        let api_key_auth = ApiKeyAuth::from_api_key(api_key, state).await?;

        // Apply MCP rate limits (not AgentHigh)
        let client_id = format!("api_key:{}", api_key_auth.email);
        state.rate_limit_service
            .check_mcp_rate_limit(&api_key_auth.user_id, &client_id)
            .await?;

        Ok(McpAuth {
            user_id: api_key_auth.user_id,
            client_id,
            scopes: ALL_MCP_SCOPES.to_vec(),
            token_type: TokenType::ApiKey,
        })
    }
}
```

---

## 8. Audit Logging

### 8.1 What Gets Logged

**Every MCP request logged with:**
- Timestamp
- User ID
- Client ID (OAuth client_id or "api_key:{email}")
- Method (JSON-RPC method name)
- Resource type (message, channel, user, etc.)
- Resource ID (specific UUID accessed)
- Scopes used
- Status (success or error)
- Error message (if error)
- Request duration (milliseconds)

### 8.2 Audit API Endpoints

**Admin Endpoints:**
- `GET /api/admin/mcp/audit/logs` - Query audit logs (filter by user or client)
- `GET /api/admin/mcp/audit/stats` - Statistics (total requests, success rate, avg latency)
- `GET /api/admin/mcp/audit/top-resources` - Most accessed resource types
- `GET /api/admin/mcp/audit/top-clients` - Most active clients

**User Endpoints:**
- `GET /api/v1/mcp/audit` - User's own MCP access logs (transparency)

### 8.3 Retention Policy

**90-day retention:**
- Background job runs daily at 2 AM
- Deletes audit logs older than 90 days
- Configurable via `MCP_AUDIT_RETENTION_DAYS` env var

**Why 90 days:**
- Balance between security monitoring and storage costs
- Sufficient for security incident investigation
- Compliance with typical audit log retention policies

---

## 9. OAuth Client Management

### 9.1 Client Registration

**Admin API:**
```http
POST /api/admin/oauth/clients
Authorization: Bearer <admin_jwt>

{
  "owner_user_id": "550e8400-...",
  "client_name": "Claude Desktop Integration",
  "client_type": "public",
  "redirect_uris": ["http://localhost:8080/callback"],
  "allowed_scopes": ["read:messages", "read:channels"],
  "is_first_party": false
}

Response:
{
  "id": "660e8400-...",
  "client_id": "mcp_a1b2c3d4...",
  "client_secret": "mcs_x9y8z7...",  // Only shown once
  "client_name": "Claude Desktop Integration",
  "client_type": "public",
  "redirect_uris": ["http://localhost:8080/callback"],
  "allowed_scopes": ["read:messages", "read:channels"],
  "is_first_party": false,
  "created_at": "2026-03-18T10:00:00Z"
}
```

**Security:**
- Client secret only returned once during registration
- Client secret stored as bcrypt hash
- Client secret required for confidential clients (not public)

### 9.2 Client Management APIs

**Admin APIs:**
- `POST /api/admin/oauth/clients` - Register new client
- `GET /api/admin/oauth/clients` - List all clients
- `GET /api/admin/oauth/clients/:client_id` - Get client details
- `PATCH /api/admin/oauth/clients/:client_id` - Update client (URIs, scopes, active status)
- `DELETE /api/admin/oauth/clients/:client_id` - Delete client (revokes all tokens)
- `POST /api/admin/oauth/clients/:client_id/rotate-secret` - Rotate client secret

**User APIs:**
- `GET /api/v1/oauth/my-clients` - List user's registered clients (if they own any)
- `GET /api/v1/oauth/authorizations` - List active OAuth authorizations
- `DELETE /api/v1/oauth/authorizations/:client_id` - Revoke access (revokes all tokens)

### 9.3 Redirect URI Validation

**Security Requirements:**
- Redirect URIs must be HTTPS (production)
- Exception: `http://localhost` and `http://127.0.0.1` allowed (development)
- Exact match required (no wildcards, no subdomain matching)
- Client can register multiple redirect URIs
- Authorization code flow validates redirect_uri against registered list

**Example Valid URIs:**
- `https://claude-desktop.anthropic.com/callback`
- `http://localhost:8080/oauth/callback`
- `http://127.0.0.1:3000/callback`

**Example Invalid URIs:**
- `http://example.com/callback` (not HTTPS, not localhost)
- `https://*.example.com/callback` (wildcard not allowed)

---

## 10. Security Considerations

### 10.1 Token Security

**Token Storage:**
- Access tokens stored as SHA-256 hashes (not plaintext)
- Refresh tokens stored as SHA-256 hashes
- Client secrets stored as bcrypt hashes
- Tokens never logged in plaintext

**Token Transmission:**
- Tokens transmitted over HTTPS only
- Authorization header: `Authorization: Bearer <token>`
- Never passed in URL query parameters

**Token Expiry:**
- Access tokens: 1 hour (short-lived)
- Refresh tokens: 30 days (long-lived, but rotated on use)
- Authorization codes: 10 minutes (very short-lived)

### 10.2 PKCE Security

**Why PKCE is required:**
- Prevents authorization code interception attacks
- Protects public clients (desktop apps) without client secret
- Industry best practice for OAuth 2.0

**PKCE Implementation:**
- Only SHA-256 method supported (reject "plain" method)
- Code verifier: 43-128 character random string
- Code challenge: `base64url(SHA256(code_verifier))`
- Server validates: `SHA256(provided_verifier) == stored_challenge`

### 10.3 Rate Limiting

**Layered Rate Limiting:**
1. IP-based (delegated to reverse proxy): 10 req/min for `/api/oauth/*`
2. Per-client: 1,000 req/hr for MCP requests
3. Per-user: 5,000 req/hr for MCP requests

**Prevents:**
- Brute force attacks on OAuth endpoints
- DDoS attacks from single misbehaving client
- Resource exhaustion from single user with many clients

### 10.4 Audit Logging

**Security Monitoring:**
- All MCP access logged (success and failure)
- Failed authorization attempts logged
- Anomalous access patterns detectable via audit stats
- Admin can investigate suspicious activity

**User Transparency:**
- Users can view their own MCP audit logs
- Users can see which clients accessed their data
- Users can revoke client access at any time

### 10.5 Scope-Based Access Control

**Principle of Least Privilege:**
- OAuth clients request only scopes they need
- Users grant subset of requested scopes
- Tokens enforce granted scopes
- Resource providers validate required scope

**Example:**
```
Client requests: read:messages, read:channels, read:files
User grants: read:messages, read:channels
Token scopes: ["read:messages", "read:channels"]

✅ Can call: resources/read for messages and channels
❌ Cannot call: resources/read for files (403 Forbidden)
```

---

## 11. Error Handling Strategy

### 11.1 Error Response Format

**All errors follow JSON-RPC 2.0 error format:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Missing required scope: read:messages",
    "data": {
      "required_scope": "read:messages",
      "granted_scopes": ["read:channels"]
    }
  },
  "id": 1
}
```

### 11.2 Error Code Mapping

**AppError → JSON-RPC Error Code:**

| AppError | JSON-RPC Code | HTTP Status |
|----------|---------------|-------------|
| `AppError::Unauthorized` | -32000 | 401 |
| `AppError::Forbidden` | -32001 | 403 |
| `AppError::NotFound` | -32002 | 404 |
| `AppError::RateLimitExceeded` | -32003 | 429 |
| `AppError::BadRequest` | -32602 | 400 |
| `AppError::Internal` | -32603 | 500 |

### 11.3 OAuth Error Responses

**OAuth 2.0 errors follow RFC 6749:**

**Authorization Errors (redirect to client):**
```
https://redirect_uri?error=access_denied&error_description=User+denied+consent&state=xxx
```

**Token Errors (JSON response):**
```json
{
  "error": "invalid_grant",
  "error_description": "Authorization code expired or already used"
}
```

**Common OAuth Error Codes:**
- `invalid_request` - Missing/invalid parameters
- `unauthorized_client` - Client not authorized for this grant type
- `access_denied` - User denied consent
- `invalid_grant` - Invalid authorization code or refresh token
- `invalid_client` - Invalid client credentials

---

## 12. Testing Strategy

### 12.1 Unit Tests

**Components to test:**
- `McpAuth` extractor (token validation, scope checking)
- `OAuthTokenService` (token generation, validation, refresh)
- `McpRouter` (method routing, error handling)
- `ResourceHandler` (resource listing, reading, authorization)
- `SseConnectionManager` (connection lifecycle, subscriptions)
- `McpAuditService` (log creation, querying, statistics)

### 12.2 Integration Tests

**OAuth Flow:**
1. Register OAuth client
2. Request authorization code with PKCE
3. Exchange code for tokens
4. Use access token for MCP request
5. Refresh token before expiry
6. Revoke token

**MCP Protocol:**
1. Initialize MCP connection
2. List resources (verify only accessible resources returned)
3. Read resource (verify authorization checks)
4. Subscribe to resource via SSE
5. Trigger resource update (verify SSE notification)

**Error Cases:**
1. Invalid token → 401 Unauthorized
2. Missing scope → 403 Forbidden
3. Invalid resource URI → 400 Bad Request
4. Resource not found → 404 Not Found
5. Rate limit exceeded → 429 Too Many Requests

### 12.3 Performance Tests

**Goals:**
- OAuth token validation: < 50ms avg, < 100ms P95
- MCP resource read: < 200ms avg, < 500ms P95
- SSE notification latency: < 100ms from trigger to client

**Load Test Scenarios:**
1. 1,000 concurrent SSE connections
2. 100 MCP requests/sec sustained load
3. 10,000 OAuth tokens active (validate O(1) lookup)

---

## 13. Deployment Checklist

### 13.1 Database Migrations

- [ ] Run migration: `sqlx migrate run`
- [ ] Verify tables created: `oauth_clients`, `oauth_authorization_codes`, `oauth_access_tokens`, `mcp_audit_logs`, `oauth_scopes`
- [ ] Verify seed data: OAuth scopes populated
- [ ] Verify indexes: Token prefix index, audit log indexes

### 13.2 Configuration

- [ ] Set environment variables:
  - `OAUTH_CODE_TTL=600` (10 minutes)
  - `OAUTH_ACCESS_TOKEN_TTL=3600` (1 hour)
  - `OAUTH_REFRESH_TOKEN_TTL=2592000` (30 days)
  - `MCP_AUDIT_RETENTION_DAYS=90`
  - `MCP_SSE_KEEPALIVE_INTERVAL=30`

### 13.3 Reverse Proxy

- [ ] Configure IP rate limiting for OAuth endpoints:
  - `/api/oauth/authorize`: 10 req/min per IP
  - `/api/oauth/token`: 10 req/min per IP

### 13.4 Monitoring

- [ ] Set up alerts:
  - OAuth token validation latency > 100ms
  - MCP rate limit exceeded (per client)
  - MCP error rate > 5%
  - SSE connection count > 5,000
- [ ] Set up dashboards:
  - Active OAuth clients
  - MCP requests per minute
  - Top MCP resources accessed
  - Audit log statistics

### 13.5 Documentation

- [ ] Create OAuth client registration guide for admins
- [ ] Create MCP integration guide for AI assistant developers
- [ ] Document MCP resource URIs and schemas
- [ ] Document OAuth scopes and permissions
- [ ] Create user guide for managing OAuth authorizations

### 13.6 Background Jobs

**OAuth Authorization Code Cleanup:**

Expired codes must be cleaned up to prevent database bloat.

```rust
pub async fn cleanup_expired_codes(db: &PgPool) -> Result<u64> {
    let result = sqlx::query!(
        "DELETE FROM oauth_authorization_codes WHERE expires_at < NOW()"
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

pub fn schedule_code_cleanup(db: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(900)); // 15 minutes

        loop {
            interval.tick().await;

            match cleanup_expired_codes(&db).await {
                Ok(deleted) if deleted > 0 => {
                    tracing::debug!(deleted_codes = deleted, "OAuth code cleanup completed");
                }
                Err(e) => {
                    tracing::error!(error = %e, "OAuth code cleanup failed");
                }
                _ => {}
            }
        }
    });
}
```

**MCP Audit Log Cleanup:**

90-day retention policy (already specified in Section 8.3).

**SSE Keepalive:**

Ping all connections every 30 seconds (already specified in Section 6.3).

---

## 14. Future Enhancements (Post-Phase 2.x)

### 14.1 Write Operations

**Phase 3.x: MCP Write Support**
- New scopes: `write:messages`, `write:channels`, `write:files`
- MCP tools: `send_message`, `create_channel`, `upload_file`
- Two-phase commit: AI proposes action, user confirms via UI
- Audit logging extended for write operations

### 14.2 Advanced Search

**Enhanced Search Provider:**
- Full-text search with PostgreSQL `tsvector`
- Fuzzy matching for typos
- Date range filtering
- Relevance scoring

### 14.3 Resource Pagination

**Cursor-Based Pagination:**
- Add `cursor` and `limit` to all resource reads
- Return `nextCursor` in response for pagination
- Support for very large channels (10k+ messages)

### 14.4 Webhooks

**Outbound Webhooks:**
- Alternative to SSE for server-side integrations
- POST resource updates to client webhook URL
- HMAC signature verification
- Retry logic with exponential backoff

### 14.5 OAuth Refresh Token Expiry Extension

**Sliding Window Refresh:**
- Extend refresh token expiry on each use (e.g., +7 days, max 90 days)
- Reduces need for re-authorization for active users

---

## 15. Success Criteria

### 15.1 Functional Requirements

- [ ] AI assistants can authorize via OAuth 2.0 with PKCE
- [ ] MCP clients can read all 6 resource types (messages, channels, users, files, teams, search)
- [ ] SSE notifications delivered within 100ms of resource change
- [ ] Scope-based authorization enforced (users can only access their data)
- [ ] Rate limiting prevents abuse (1k req/hr per client, 5k req/hr per user)
- [ ] Comprehensive audit logging (90-day retention)
- [ ] Admin can manage OAuth clients (create, update, delete, rotate secrets)
- [ ] Users can view and revoke OAuth authorizations

### 15.2 Performance Requirements

- [ ] OAuth token validation: < 50ms avg, < 100ms P95
- [ ] MCP resource read: < 200ms avg, < 500ms P95
- [ ] SSE notification latency: < 100ms from trigger to client
- [ ] System handles 1,000 concurrent SSE connections
- [ ] System handles 100 MCP requests/sec sustained load

### 15.3 Security Requirements

- [ ] Tokens stored as hashes (never plaintext)
- [ ] PKCE required for all OAuth flows (no "plain" method)
- [ ] Redirect URI validation enforced (HTTPS only, except localhost)
- [ ] Rate limiting prevents brute force and DDoS
- [ ] Audit logs capture all MCP access (success and failure)
- [ ] Users can revoke client access at any time

### 15.4 Documentation Requirements

- [ ] OAuth integration guide for developers
- [ ] MCP resource URI reference documentation
- [ ] Admin guide for client management
- [ ] User guide for managing authorizations
- [ ] API reference for all OAuth and MCP endpoints

---

## 16. Appendices

### Appendix A: Example OAuth Authorization Flow

**1. Client initiates authorization:**
```
https://rustchat.example.com/api/oauth/authorize
  ?client_id=mcp_a1b2c3d4e5f6...
  &redirect_uri=http://localhost:8080/callback
  &response_type=code
  &scope=read:messages read:channels
  &state=random_state_value
  &code_challenge=qjrzSW9gMiUgpUvqgEPE4_-8swvyCtfOVvg55o5S_es
  &code_challenge_method=S256
```

**2. User sees consent page:**
```
Claude Desktop wants to access your RustChat data:

☑ Read messages from channels you have access to
☑ Read channel information and metadata

[Authorize] [Deny]
```

**3. User clicks Authorize, redirected:**
```
http://localhost:8080/callback
  ?code=abc123def456...
  &state=random_state_value
```

**4. Client exchanges code for tokens:**
```http
POST https://rustchat.example.com/api/oauth/token
Content-Type: application/json

{
  "grant_type": "authorization_code",
  "code": "abc123def456...",
  "redirect_uri": "http://localhost:8080/callback",
  "client_id": "mcp_a1b2c3d4e5f6...",
  "code_verifier": "original_random_string_from_step_1"
}

Response:
{
  "access_token": "rct_x1y2z3...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "rcr_a9b8c7...",
  "scope": "read:messages read:channels"
}
```

**5. Client uses access token for MCP:**
```http
POST https://rustchat.example.com/api/mcp
Authorization: Bearer rct_x1y2z3...
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "resources/list",
  "id": 1
}
```

### Appendix B: Example MCP Session

**Initialize:**
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "Claude Desktop",
      "version": "1.0.0"
    }
  },
  "id": 1
}

Response:
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "resources": {
        "subscribe": true,
        "listChanged": true
      },
      "tools": {},
      "prompts": {}
    },
    "serverInfo": {
      "name": "RustChat MCP Server",
      "version": "2.0.0"
    }
  },
  "id": 1
}
```

**List Resources:**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/list",
  "id": 2
}

Response:
{
  "jsonrpc": "2.0",
  "result": {
    "resources": [
      {
        "uri": "rustchat-message://channel/550e8400-...",
        "name": "Messages in #general",
        "mimeType": "application/json",
        "description": "Recent messages from public channel"
      },
      {
        "uri": "rustchat-channel://550e8400-...",
        "name": "#general",
        "mimeType": "application/json",
        "description": "Public channel for general discussion"
      }
    ]
  },
  "id": 2
}
```

**Read Resource:**
```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "rustchat-message://channel/550e8400-...?limit=10"
  },
  "id": 3
}

Response:
{
  "jsonrpc": "2.0",
  "result": {
    "contents": [{
      "uri": "rustchat-message://channel/550e8400-...",
      "mimeType": "text/plain",
      "text": "[2026-03-18 10:00] alice: Hello team\n[2026-03-18 10:01] bob: Hi Alice\n..."
    }]
  },
  "id": 3
}
```

### Appendix C: Resource URI Examples

**Messages:**
- `rustchat-message://channel/550e8400-e29b-41d4-a716-446655440000`
- `rustchat-message://channel/550e8400-e29b-41d4-a716-446655440000?limit=50`
- `rustchat-message://channel/550e8400-e29b-41d4-a716-446655440000?limit=20&before=msg_123`

**Channels:**
- `rustchat-channel://550e8400-e29b-41d4-a716-446655440000`

**Users:**
- `rustchat-user://660e8400-e29b-41d4-a716-446655440000`

**Files:**
- `rustchat-file://770e8400-e29b-41d4-a716-446655440000`

**Teams:**
- `rustchat-team://880e8400-e29b-41d4-a716-446655440000`

**Search:**
- `rustchat-search://query?q=authentication&type=messages&limit=50`
- `rustchat-search://query?q=john&type=users&limit=20`
- `rustchat-search://query?q=engineering&type=channels&limit=10`

---

## End of Specification

This design specification is ready for implementation. Next step: Create implementation plan using the `writing-plans` skill.
