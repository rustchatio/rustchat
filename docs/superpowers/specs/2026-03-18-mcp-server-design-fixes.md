# MCP Server Design Specification - Revision Fixes

## Critical Issues Fixed

### 1. Database Foreign Key Integrity (Section 3.3)

**Issue:** `oauth_access_tokens.client_id` references wrong field type

**Fix:**
```sql
-- CORRECTED: oauth_access_tokens table
CREATE TABLE oauth_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash TEXT UNIQUE NOT NULL,
    refresh_token_hash TEXT UNIQUE,
    client_id VARCHAR(255) NOT NULL,  -- Changed from UUID
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scopes TEXT[] NOT NULL,
    token_type VARCHAR(50) DEFAULT 'Bearer',
    expires_at TIMESTAMPTZ NOT NULL,
    refresh_expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,

    -- Foreign key to oauth_clients using client_id (VARCHAR), not id (UUID)
    CONSTRAINT fk_client FOREIGN KEY (client_id)
        REFERENCES oauth_clients(client_id) ON DELETE CASCADE
);
```

**Rationale:** OAuth tokens reference clients by `client_id` (the public identifier like `mcp_xxx`), not the internal UUID `id`.

### 2. Token Validation Logic Gap

**Issue:** Tokens stored as SHA-256 hashes, but validation logic unclear

**Fix - Token Storage & Lookup:**

Tokens are **NOT** stored hashed in full. Instead:

1. **Token Generation:**
   ```rust
   // Generate raw token
   let access_token = format!("rct_{}", generate_random_hex(64));  // 68 chars total

   // Extract prefix for indexing (first 20 chars: "rct_" + 16 hex)
   let token_prefix = &access_token[..20];

   // Hash full token for secure storage
   let token_hash = sha256_hash(&access_token);

   // Store both prefix and hash
   INSERT INTO oauth_access_tokens (token_hash, ...) VALUES ($token_hash, ...);
   ```

2. **Token Validation:**
   ```rust
   async fn validate_access_token(token: &str) -> Result<McpAuth> {
       // 1. Extract prefix (first 20 chars)
       let prefix = &token[..20];  // "rct_a1b2c3d4e5f6g7h8"

       // 2. Hash full token
       let token_hash = sha256_hash(token);

       // 3. Query by prefix (indexed) + exact hash match
       let token_record = sqlx::query!(
           "SELECT * FROM oauth_access_tokens
            WHERE LEFT(token_hash, 20) = $1  -- Prefix scan
              AND token_hash = $2             -- Exact match
              AND expires_at > NOW()
              AND revoked_at IS NULL",
           prefix,
           token_hash
       ).fetch_one().await?;

       Ok(McpAuth { ... })
   }
   ```

3. **Index Strategy:**
   ```sql
   -- Prefix index on token_hash for O(1) lookup
   CREATE INDEX idx_oauth_tokens_prefix
   ON oauth_access_tokens (LEFT(token_hash, 20))
   WHERE revoked_at IS NULL AND expires_at > NOW();
   ```

**Wait, this doesn't work!** The hash destroys the prefix. **Revised approach:**

**CORRECTED Token Storage Strategy:**

Store **token prefix separately** (not hashed):

```sql
CREATE TABLE oauth_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_prefix VARCHAR(20) NOT NULL,    -- NEW: Raw prefix for O(1) lookup
    token_hash TEXT UNIQUE NOT NULL,       -- Full token SHA-256 hash
    refresh_token_hash TEXT UNIQUE,
    client_id VARCHAR(255) NOT NULL,
    ...
);

-- Index on raw prefix
CREATE INDEX idx_oauth_tokens_prefix
ON oauth_access_tokens (token_prefix)
WHERE revoked_at IS NULL AND expires_at > NOW();
```

**Token Validation (Corrected):**
```rust
async fn validate_access_token(token: &str) -> Result<McpAuth> {
    // 1. Extract raw prefix (first 20 chars)
    let token_prefix = &token[..20];  // "rct_a1b2c3d4e5f6g7h8"

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

    // 4. Verify exact token hash (constant-time comparison)
    for candidate in candidates {
        if constant_time_compare(&candidate.token_hash, &token_hash) {
            return Ok(McpAuth::from(candidate));
        }
    }

    Err(AppError::Unauthorized("Invalid token"))
}
```

**Why this works:**
- Prefix stored in plaintext allows O(1) indexed lookup
- Full token never stored in plaintext (only hash)
- Collision probability: 2^-80 for 20-char prefix (16 hex = 64 bits + 4-char prefix)
- Even with collision, exact hash comparison prevents false positives

### 3. Missing API Endpoint Specifications

**Added Section 4.5: Complete API Endpoint Reference**

```markdown
## 4.5 Complete API Endpoint Reference

### OAuth 2.0 Endpoints

#### GET /api/oauth/authorize
**Purpose:** Display authorization consent page

**Query Parameters:**
- `client_id` (required): OAuth client identifier
- `redirect_uri` (required): Callback URL (must match registered URI)
- `response_type` (required): Must be "code"
- `scope` (required): Space-separated scopes (e.g., "read:messages read:channels")
- `state` (required): Client-generated random string (CSRF protection)
- `code_challenge` (required): Base64url(SHA256(code_verifier))
- `code_challenge_method` (required): Must be "S256"

**Response:**
- 200 OK: HTML consent page showing requested scopes
- 302 Redirect: If user already authenticated and consented
- 400 Bad Request: Invalid parameters
- 401 Unauthorized: User not authenticated (redirect to login)

#### POST /api/oauth/authorize
**Purpose:** User grants or denies authorization

**Request Body:**
```json
{
  "client_id": "mcp_xxx",
  "redirect_uri": "https://...",
  "scopes": ["read:messages", "read:channels"],
  "state": "random_state",
  "code_challenge": "xxx",
  "code_challenge_method": "S256",
  "user_consent": true  // true = authorize, false = deny
}
```

**Response:**
- 302 Redirect (Authorized):
  ```
  {redirect_uri}?code=abc123&state=random_state
  ```
- 302 Redirect (Denied):
  ```
  {redirect_uri}?error=access_denied&error_description=User+denied&state=random_state
  ```

#### POST /api/oauth/token
**Purpose:** Exchange authorization code for tokens OR refresh access token

**Request Body (Authorization Code):**
```json
{
  "grant_type": "authorization_code",
  "code": "abc123...",
  "redirect_uri": "https://...",
  "client_id": "mcp_xxx",
  "client_secret": "mcs_xxx",  // Optional for public clients
  "code_verifier": "original_random_string"
}
```

**Request Body (Refresh Token):**
```json
{
  "grant_type": "refresh_token",
  "refresh_token": "rcr_xxx...",
  "client_id": "mcp_xxx",
  "client_secret": "mcs_xxx"  // Optional for public clients
}
```

**Response (Success):**
```json
{
  "access_token": "rct_xxx...",
  "token_type": "Bearer",
  "expires_in": 3600,
  "refresh_token": "rcr_xxx...",
  "scope": "read:messages read:channels"
}
```

**Response (Error):**
```json
{
  "error": "invalid_grant",
  "error_description": "Authorization code expired or invalid"
}
```

#### POST /api/oauth/revoke
**Purpose:** Revoke access or refresh token

**Request Body:**
```json
{
  "token": "rct_xxx or rcr_xxx",
  "token_type_hint": "access_token"  // or "refresh_token"
}
```

**Response:**
- 200 OK: Token revoked (or already invalid)

#### POST /api/oauth/introspect
**Purpose:** Validate token (for resource servers)

**Request Body:**
```json
{
  "token": "rct_xxx..."
}
```

**Response:**
```json
{
  "active": true,
  "scope": "read:messages read:channels",
  "client_id": "mcp_xxx",
  "user_id": "550e8400-...",
  "exp": 1710757200
}
```

### Admin OAuth Client Management Endpoints

#### POST /api/admin/oauth/clients
**Auth:** Admin JWT required

**Request:**
```json
{
  "owner_user_id": "550e8400-...",
  "client_name": "Claude Desktop",
  "client_type": "public",
  "redirect_uris": ["http://localhost:8080/callback"],
  "allowed_scopes": ["read:messages", "read:channels"],
  "is_first_party": false
}
```

**Response:** (201 Created)
```json
{
  "id": "660e8400-...",
  "client_id": "mcp_a1b2c3d4...",
  "client_secret": "mcs_x9y8z7...",  // Only shown once
  "client_name": "Claude Desktop",
  ...
}
```

#### GET /api/admin/oauth/clients
**Auth:** Admin JWT required

**Query Parameters:**
- `owner_user_id` (optional): Filter by owner
- `first_party` (optional): Filter first-party clients

**Response:** (200 OK)
```json
{
  "clients": [
    {
      "id": "660e8400-...",
      "client_id": "mcp_xxx",
      "client_name": "Claude Desktop",
      "client_type": "public",
      "redirect_uris": [...],
      "allowed_scopes": [...],
      "is_active": true,
      ...
    }
  ]
}
```

#### GET /api/admin/oauth/clients/:client_id
#### PATCH /api/admin/oauth/clients/:client_id
#### DELETE /api/admin/oauth/clients/:client_id
#### POST /api/admin/oauth/clients/:client_id/rotate-secret

(Detailed specs provided in main document Section 9.2)

### User OAuth Management Endpoints

#### GET /api/v1/oauth/my-clients
**Auth:** User JWT required
**Response:** List of OAuth clients owned by user

#### GET /api/v1/oauth/authorizations
**Auth:** User JWT required
**Response:** List of active OAuth authorizations (which clients have access)

#### DELETE /api/v1/oauth/authorizations/:client_id
**Auth:** User JWT required
**Purpose:** Revoke all tokens for a specific client

### MCP Audit Endpoints

#### GET /api/admin/mcp/audit/logs
**Auth:** Admin JWT required

**Query Parameters:**
- `user_id` or `client_id` (required): Filter logs
- `limit` (optional, default 100, max 500)
- `offset` (optional, default 0)

**Response:**
```json
{
  "logs": [
    {
      "id": "...",
      "timestamp": "2026-03-18T10:00:00Z",
      "user_id": "...",
      "client_id": "mcp_xxx",
      "method": "resources/read",
      "resource_type": "message",
      "resource_id": "...",
      "scopes_used": ["read:messages"],
      "status": "success",
      "request_duration_ms": 45
    }
  ],
  "total": 1
}
```

#### GET /api/admin/mcp/audit/stats
**Query Parameters:**
- `start_time` (optional, ISO 8601)
- `end_time` (optional, ISO 8601)

#### GET /api/admin/mcp/audit/top-resources
#### GET /api/admin/mcp/audit/top-clients

#### GET /api/v1/mcp/audit
**Auth:** User JWT required
**Purpose:** User views their own MCP access logs
```

### 4. OAuth Authorization Flow Implementation

**Added Section 2.1.1: Consent Page Implementation**

```markdown
### 2.1.1 OAuth Consent Page Implementation

**GET /api/oauth/authorize** renders an HTML consent page:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Authorize Application</title>
</head>
<body>
    <h1>Authorize Access</h1>
    <p><strong>{{ client_name }}</strong> wants to access your RustChat data:</p>

    <ul>
        {% for scope in requested_scopes %}
        <li>
            <strong>{{ scope.display_name }}</strong><br>
            <small>{{ scope.description }}</small>
        </li>
        {% endfor %}
    </ul>

    <form method="POST" action="/api/oauth/authorize">
        <input type="hidden" name="client_id" value="{{ client_id }}">
        <input type="hidden" name="redirect_uri" value="{{ redirect_uri }}">
        <input type="hidden" name="state" value="{{ state }}">
        <input type="hidden" name="code_challenge" value="{{ code_challenge }}">
        <input type="hidden" name="code_challenge_method" value="{{ code_challenge_method }}">
        <input type="hidden" name="scopes" value="{{ requested_scopes_json }}">

        <button type="submit" name="user_consent" value="true">Authorize</button>
        <button type="submit" name="user_consent" value="false">Deny</button>
    </form>
</body>
</html>
```

**Server-Side Flow:**

1. **GET /api/oauth/authorize** (auth/oauth/authorize.rs):
   - Extract JWT from cookie (user must be logged in)
   - Validate `client_id`, `redirect_uri`, `scopes`, PKCE parameters
   - Store pending authorization in Redis with 10-minute expiry:
     ```redis
     SET oauth:pending:{random_key} {
         "client_id": "mcp_xxx",
         "user_id": "550e8400-...",
         "redirect_uri": "...",
         "scopes": [...],
         "code_challenge": "...",
         "code_challenge_method": "S256",
         "state": "..."
     } EX 600
     ```
   - Render consent page with `pending_key` in hidden form field

2. **POST /api/oauth/authorize** (auth/oauth/authorize.rs):
   - Extract JWT from cookie (verify same user)
   - Retrieve pending authorization from Redis using `pending_key`
   - If `user_consent == true`:
     - Generate authorization code (64 hex chars)
     - Store code in `oauth_authorization_codes` table (10 min expiry)
     - Delete pending authorization from Redis
     - Redirect: `{redirect_uri}?code={code}&state={state}`
   - If `user_consent == false`:
     - Delete pending authorization from Redis
     - Redirect: `{redirect_uri}?error=access_denied&state={state}`
```

### 5. PKCE Validation Transaction Order

**Issue:** Circular dependency - codes deleted before validation

**Fix - Corrected Token Exchange Flow:**

```rust
// POST /api/oauth/token (grant_type=authorization_code)

async fn exchange_authorization_code(request: TokenRequest) -> Result<TokenResponse> {
    let mut tx = db.begin().await?;

    // Step 1: Fetch authorization code (with SELECT FOR UPDATE to prevent race conditions)
    let code_record = sqlx::query!(
        "SELECT * FROM oauth_authorization_codes
         WHERE code = $1
           AND expires_at > NOW()
         FOR UPDATE",
        request.code
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::Unauthorized("Invalid or expired code"))?;

    // Step 2: Validate PKCE BEFORE deleting code
    validate_pkce(
        &request.code_verifier,
        &code_record.code_challenge,
        &code_record.code_challenge_method
    )?;

    // Step 3: Validate redirect_uri matches
    if request.redirect_uri != code_record.redirect_uri {
        return Err(AppError::Unauthorized("Redirect URI mismatch"));
    }

    // Step 4: Validate client credentials
    let client = authenticate_client(&request.client_id, request.client_secret.as_deref()).await?;

    // Step 5: Generate access + refresh tokens
    let access_token = generate_access_token();
    let refresh_token = generate_refresh_token();
    let token_prefix = &access_token[..20];
    let token_hash = sha256_hash(&access_token);
    let refresh_hash = sha256_hash(&refresh_token);

    // Step 6: Store tokens
    sqlx::query!(
        "INSERT INTO oauth_access_tokens
         (token_prefix, token_hash, refresh_token_hash, client_id, user_id, scopes, expires_at, refresh_expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW() + INTERVAL '1 hour', NOW() + INTERVAL '30 days')",
        token_prefix,
        token_hash,
        refresh_hash,
        client.client_id,  // VARCHAR, not UUID
        code_record.user_id,
        &code_record.scopes
    )
    .execute(&mut *tx)
    .await?;

    // Step 7: Delete authorization code (single-use)
    sqlx::query!(
        "DELETE FROM oauth_authorization_codes WHERE id = $1",
        code_record.id
    )
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
```

## Important Issues Fixed

### 6. Missing Token Refresh Endpoint Specification

**Added to Section 4.5:**

Already included in "POST /api/oauth/token" with `grant_type=refresh_token`.

### 7. SSE Subscription Mechanism

**Added Section 6.1.1: SSE-JSON-RPC Correlation**

```markdown
### 6.1.1 SSE Connection and Subscription Correlation

**Problem:** How does the server link JSON-RPC subscription requests to specific SSE connections?

**Solution:** Connection ID tracking

1. **Client opens SSE connection:**
   ```
   GET /api/mcp/sse
   Authorization: Bearer rct_xxx
   ```

   Server response (first event):
   ```
   event: connected
   data: {"connectionId": "550e8400-e29b-41d4-a716-446655440000"}
   ```

2. **Client stores connectionId and uses it for subscriptions:**
   ```http
   POST /api/mcp
   Authorization: Bearer rct_xxx
   X-MCP-Connection-ID: 550e8400-e29b-41d4-a716-446655440000

   {
     "jsonrpc": "2.0",
     "method": "resources/subscribe",
     "params": {
       "uri": "rustchat-message://channel/660e8400-..."
     },
     "id": 1
   }
   ```

3. **Server validates connection:**
   ```rust
   let connection_id = headers.get("X-MCP-Connection-ID")
       .and_then(|h| h.to_str().ok())
       .and_then(|s| Uuid::parse_str(s).ok())
       .ok_or(AppError::BadRequest("Missing connection ID"))?;

   // Verify connection exists and belongs to this user
   let connection = sse_manager.get_connection(connection_id).await?;
   if connection.user_id != auth.user_id {
       return Err(AppError::Forbidden("Connection belongs to different user"));
   }

   // Subscribe
   sse_manager.subscribe(connection_id, uri).await?;
   ```

4. **Alternative (No Header Required):**

   If `X-MCP-Connection-ID` header not provided, server finds connection by token:

   ```rust
   // Find active SSE connection for this user+token
   let connection = sse_manager.find_connection_by_token(&auth.user_id, &auth.client_id).await?;
   sse_manager.subscribe(connection.connection_id, uri).await?;
   ```

   **Note:** This assumes one SSE connection per user+client pair. If multiple concurrent connections allowed, header is required.
```

### 8. Rate Limiting Inconsistency

**Clarified in Section 7.3:**

```markdown
### 7.3 API Key Rate Limits (Revised)

API keys used for MCP follow **MCP rate limits**, not AgentHigh limits:

- **Per-client (API key):** 1,000 req/hr (same as OAuth clients)
- **Per-user:** 5,000 req/hr (same as OAuth)

**Rationale:**
- API keys for MCP are treated as "service clients"
- Should not get preferential treatment over interactive OAuth clients
- Prevents abuse from automated integrations

**AgentHigh tier (10k req/hr):**
- Only applies to non-MCP API usage (standard REST API)
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
```

### 9. SSE Connection Persistence

**Added Section 6.3.1: Connection Persistence Strategy**

```markdown
### 6.3.1 SSE Connection Persistence

**Strategy:** In-memory only (no database persistence)

**Implications:**
- Server restart → all SSE connections lost
- Clients must reconnect after server restart
- Subscriptions not persisted

**Rationale:**
- SSE connections are ephemeral by nature
- Clients handle reconnection automatically (built into SSE spec)
- No need for complex connection state synchronization across restarts

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

**Server Restart Behavior:**
- Existing SSE connections receive connection close event
- Clients detect closure and reconnect
- New `connectionId` issued on reconnect
- Clients re-establish subscriptions
```

### 10. Authorization Code Cleanup

**Added Section 13.6: Background Jobs**

```markdown
### 13.6 Background Jobs

#### OAuth Authorization Code Cleanup

**Job:** Clean up expired authorization codes

**Schedule:** Every 15 minutes

**Implementation:**
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
```

### 11. Notification Filtering by Scopes

**Added Section 6.4.1: Scope-Based Notification Filtering**

```markdown
### 6.4.1 Scope-Based Notification Filtering

**Problem:** User subscribed to `rustchat-message://channel/xxx` but only has `read:channels` scope (not `read:messages`).

**Solution:** Validate scope on subscription, not notification.

```rust
impl ResourceHandler {
    pub async fn subscribe(
        &self,
        params: &Option<Value>,
        auth: &McpAuth,
        state: &AppState,
    ) -> Result<Value, ErrorObject> {
        let request: SubscribeRequest = serde_json::from_value(...)?;

        // Parse URI to determine required scope
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

        // Validate scope before allowing subscription
        auth.require_scope(required_scope)?;

        // Subscribe to resource
        let connection_id = extract_connection_id(headers)?;
        state.sse_manager.subscribe(connection_id, request.uri.clone()).await
            .map_err(|e| ErrorObject {
                code: error_codes::INTERNAL_ERROR,
                message: e,
                data: None,
            })?;

        Ok(json!({"subscribed": true}))
    }
}
```

**Result:** Clients cannot subscribe to resources they don't have scope for. Notifications only go to properly authorized subscribers.
```

## Summary of Changes

1. ✅ Fixed database foreign key (client_id VARCHAR, not UUID)
2. ✅ Clarified token storage (added `token_prefix` column for O(1) lookup)
3. ✅ Added complete API endpoint specifications (OAuth, admin, user, audit)
4. ✅ Specified OAuth consent page implementation (HTML + Redis pending state)
5. ✅ Fixed PKCE validation order (validate before deleting code)
6. ✅ Clarified token refresh endpoint (grant_type=refresh_token)
7. ✅ Added SSE-JSON-RPC correlation mechanism (connection ID header)
8. ✅ Clarified rate limiting (API keys follow MCP limits, not AgentHigh)
9. ✅ Specified SSE connection persistence (in-memory only, auto-reconnect)
10. ✅ Added authorization code cleanup background job
11. ✅ Added scope-based notification filtering (validate on subscribe)

All critical and important issues addressed. Ready for re-review.
