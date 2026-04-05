# Secure Deployment Guide

This guide outlines the security hardening measures required for production deployments of RustChat.

## Table of Contents

1. [Critical Security Configuration](#critical-security-configuration)
2. [Token Transport Security](#token-transport-security)
3. [Rate Limiting](#rate-limiting)
4. [Secret Management](#secret-management)
5. [TLS and Network Security](#tls-and-network-security)
6. [Monitoring and Alerting](#monitoring-and-alerting)

---

## Critical Security Configuration

### Environment Variables

The following environment variables MUST be set for secure production deployments:

```bash
# Required: Strong random secrets (min 32 characters, high entropy)
RUSTCHAT_JWT_SECRET="your-256-bit-secret-min-32-chars-long-random"
RUSTCHAT_ENCRYPTION_KEY="another-256-bit-secret-for-encryption-ops"

# Required: Set to production
RUSTCHAT_ENVIRONMENT=production

# Required: Secure OAuth token delivery (one-time codes only)
RUSTCHAT_SECURITY_OAUTH_TOKEN_DELIVERY=cookie

# Recommended: Enable rate limiting (enabled by default)
RUSTCHAT_SECURITY_RATE_LIMIT_ENABLED=true
```

### Secret Generation

Generate cryptographically secure secrets using:

```bash
# Using OpenSSL (recommended)
RUSTCHAT_JWT_SECRET=$(openssl rand -base64 48)
RUSTCHAT_ENCRYPTION_KEY=$(openssl rand -base64 48)

# Using /dev/urandom
RUSTCHAT_JWT_SECRET=$(head -c 48 /dev/urandom | base64)
```

**IMPORTANT:** Secrets must:
- Be at least 32 characters long
- Have high entropy (no dictionary words, sequential characters)
- Be different from each other (JWT_SECRET ≠ ENCRYPTION_KEY)
- Be rotated periodically

---

## Token Transport Security

### WebSocket Authentication

RustChat accepts authentication tokens via:
1. Authorization header: `Authorization: Bearer xyz` (✅ **Secure**)
2. Sec-WebSocket-Protocol header (✅ **Secure**)

Query-token authentication is removed and rejected at startup if
`RUSTCHAT_SECURITY_WS_ALLOW_QUERY_TOKEN=true`.

WebSocket connections MUST use secure headers:
```javascript
const ws = new WebSocket('wss://chat.example.com/api/v1/ws');
// Token is sent in the handshake headers, not visible in URL
```

### OAuth Token Delivery

Only secure mode is supported: `RUSTCHAT_SECURITY_OAUTH_TOKEN_DELIVERY=cookie`.

- One-time exchange code in URL: `/login?code=xyz`
- Client exchanges code for token via `POST /api/v1/oauth2/exchange`
- Token never appears in redirect URLs

---

## Rate Limiting

Rate limiting is enabled by default and protects against brute-force attacks.

### Configuration

```bash
# Enable/disable rate limiting (default: true)
RUSTCHAT_SECURITY_RATE_LIMIT_ENABLED=true

# Auth endpoints (login, register) - per IP
RUSTCHAT_SECURITY_RATE_LIMIT_AUTH_PER_MINUTE=10

# WebSocket connections - per IP
RUSTCHAT_SECURITY_RATE_LIMIT_WS_PER_MINUTE=30
```

### Rate Limit Responses

When rate limited, the API returns:
```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1704067200
Retry-After: 45

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests. Please try again later.",
    "retry_after": 45
  }
}
```

---

## Secret Management

### Startup Validation

RustChat validates secrets on startup:

**In Production (`RUSTCHAT_ENVIRONMENT=production`):**
- Fails to start if secrets are weak
- Checks minimum length (32 chars)
- Checks entropy (no repeated patterns)
- Checks for common weak patterns

**In Development:**
- Logs warnings for weak secrets
- Allows startup with weak secrets

### Secret Rotation

To rotate secrets:

1. **JWT Secret Rotation:**
   ```bash
   # 1. Generate new secret
   NEW_JWT_SECRET=$(openssl rand -base64 48)
   
   # 2. Deploy with new secret (existing tokens become invalid)
   # 3. Users must re-authenticate
   ```

2. **Encryption Key Rotation:**
   ```bash
   # Requires data re-encryption - contact support
   ```

---

## TLS and Network Security

### Required TLS Configuration

```nginx
# nginx example
server {
    listen 443 ssl http2;
    server_name chat.example.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    
    location / {
        proxy_pass http://rustchat_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Forward client IP for rate limiting
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### CORS Configuration

In production, explicitly set allowed origins:

```bash
RUSTCHAT_CORS_ALLOWED_ORIGINS="https://chat.example.com,https://admin.example.com"
```

Never use wildcard (`*`) in production.

---

## Monitoring and Alerting

### Security Events to Monitor

| Event | Severity | Action |
|-------|----------|--------|
| Rate limit exceeded | Warning | Monitor for patterns |
| Invalid OAuth state | Warning | Possible CSRF attempt |
| Weak secret detected | Critical | Rotate immediately |
| WebSocket auth failure | Info | Normal for expired tokens |
| Exchange code reuse | Warning | Possible token theft |

### Log Redaction

Configure your reverse proxy to redact sensitive headers:

```nginx
# nginx - don't log tokens
log_format security '$remote_addr - $remote_user [$time_local] '
                    '"$request" $status $body_bytes_sent '
                    '"$http_referer" "$http_user_agent"';

access_log /var/log/nginx/access.log security;
```

---

## Deployment Checklist

Before going to production:

- [ ] Set `RUSTCHAT_ENVIRONMENT=production`
- [ ] Generate strong JWT_SECRET (≥32 chars, random)
- [ ] Generate strong ENCRYPTION_KEY (different from JWT_SECRET)
- [ ] Set `RUSTCHAT_SECURITY_OAUTH_TOKEN_DELIVERY=cookie`
- [ ] Configure CORS with specific origins
- [ ] Enable TLS 1.2+ only
- [ ] Configure rate limiting
- [ ] Set up log redaction
- [ ] Configure security headers
- [ ] Test OAuth flow end-to-end
- [ ] Verify WebSocket connections work with headers

---

## Migration from Insecure Defaults

If currently running with insecure defaults:

1. **Immediate (P0):** Rotate to strong secrets
2. **Week 1:** Update frontend to use header-based WebSocket auth
3. **Week 2:** Update frontend to support OAuth exchange codes
4. **Week 3:** Deploy with `OAUTH_TOKEN_DELIVERY=cookie`

Contact your security team if you need assistance with the migration.
