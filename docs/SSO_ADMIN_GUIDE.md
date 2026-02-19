# SSO/OIDC Administration Guide

This guide covers the configuration and management of Single Sign-On (SSO) and OpenID Connect (OIDC) authentication in RustChat.

## Table of Contents

- [Overview](#overview)
- [Supported Providers](#supported-providers)
- [Global SSO Settings](#global-sso-settings)
- [Provider Configuration](#provider-configuration)
  - [GitHub OAuth2](#github-oauth2)
  - [Google OIDC](#google-oidc)
  - [Generic OIDC](#generic-oidc)
- [User Provisioning](#user-provisioning)
- [Role Mappings](#role-mappings)
- [Testing Configuration](#testing-configuration)
- [Troubleshooting](#troubleshooting)
- [API Reference](#api-reference)

## Overview

RustChat supports three SSO provider types:

1. **GitHub** - OAuth2 authentication with optional organization/team restrictions
2. **Google** - OIDC authentication with Google Workspace domain restrictions
3. **Generic OIDC** - Any OIDC-compliant provider (Keycloak, ZITADEL, Authentik, Okta, etc.)

## Supported Providers

### Security Features

- **State parameter** - CSRF protection via random state stored in Redis (5-minute TTL)
- **PKCE** - Proof Key for Code Exchange (S256) for OIDC providers
- **Nonce validation** - Replay attack protection for OIDC
- **Encrypted secrets** - Client secrets encrypted at rest using AES-GCM
- **ID token validation** - Signature verification using JWKS

### Provider Comparison

| Feature | GitHub | Google | Generic OIDC |
|---------|--------|--------|--------------|
| Discovery | No | Yes | Yes |
| PKCE | No | Yes | Yes |
| Domain Restrictions | No | Yes | No |
| Org/Team Restrictions | Yes | No | No |
| Group Mappings | No | No | Yes |

## Global SSO Settings

Navigate to **Admin Console > SSO / OAuth** to manage global settings.

### Enable SSO

When enabled, users can sign in using configured SSO providers. The login page will display SSO buttons.

### Require SSO

When enabled:
- Password login is disabled on the web interface
- Only SSO authentication is allowed
- API endpoints remain available (for integrations)

**Warning:** Ensure at least one SSO provider is active and tested before enabling this setting.

## Provider Configuration

### Provider Key

The `provider_key` is a unique identifier used in URLs:
- Must be lowercase alphanumeric with hyphens only
- Used in callback URLs: `/api/v1/oauth2/{provider_key}/callback`
- Cannot be changed after creation

Examples: `github`, `google-workspace`, `oidc-keycloak`

### GitHub OAuth2

#### Setup Instructions

1. Go to GitHub > Settings > Developer settings > OAuth Apps
2. Click "New OAuth App"
3. Configure:
   - **Application name**: RustChat
   - **Homepage URL**: `https://your-rustchat-domain.com`
   - **Authorization callback URL**: `https://your-rustchat-domain.com/api/v1/oauth2/{provider_key}/callback`
4. Generate a client secret
5. Copy Client ID and Client Secret to RustChat

#### Required Fields

| Field | Description |
|-------|-------------|
| Provider Key | Unique identifier (e.g., `github`) |
| Display Name | User-facing name (e.g., "GitHub") |
| Client ID | From GitHub OAuth app |
| Client Secret | From GitHub OAuth app |

#### Optional Restrictions

| Field | Description |
|-------|-------------|
| GitHub Organization | Require membership in this org |
| GitHub Team | Require membership in this team (within org) |

#### Default Scopes

- `read:user` - Read user profile
- `user:email` - Read user email addresses

### Google OIDC

#### Setup Instructions

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. APIs & Services > Credentials
3. Create OAuth 2.0 Client ID (Web application)
4. Configure:
   - **Authorized redirect URIs**: `https://your-rustchat-domain.com/api/v1/oauth2/{provider_key}/callback`
5. Enable "People API" for user info access

#### Required Fields

| Field | Description |
|-------|-------------|
| Provider Key | Unique identifier (e.g., `google`) |
| Display Name | User-facing name (e.g., "Google") |
| Issuer URL | `https://accounts.google.com` |
| Client ID | From Google Cloud Console |
| Client Secret | From Google Cloud Console |

#### Optional Restrictions

| Field | Description |
|-------|-------------|
| Allowed Domains | Restrict to specific email domains (e.g., `company.com`) |

#### Default Scopes

- `openid` - OIDC authentication
- `profile` - User profile info
- `email` - Email address

### Generic OIDC

Compatible with Keycloak, ZITADEL, Authentik, Okta, Auth0, and any OIDC-compliant provider.

#### Setup Instructions

1. Create an OIDC client in your provider
2. Set redirect URI: `https://your-rustchat-domain.com/api/v1/oauth2/{provider_key}/callback`
3. Enable "Authorization Code" grant type
4. Enable PKCE (S256)

#### Required Fields

| Field | Description |
|-------|-------------|
| Provider Key | Unique identifier (e.g., `keycloak`) |
| Display Name | User-facing name (e.g., "Company SSO") |
| Issuer URL | Your OIDC issuer URL (e.g., `https://keycloak.company.com/realms/main`) |
| Client ID | From your OIDC provider |
| Client Secret | From your OIDC provider |

#### Optional Fields

| Field | Description |
|-------|-------------|
| Groups Claim | Claim name containing user groups (default: `groups`) |
| Role Mappings | JSON mapping of provider groups to RustChat roles |

#### Default Scopes

- `openid` - Required for OIDC
- `profile` - User profile info
- `email` - Email address

## User Provisioning

### Auto-Provision

When enabled, new users are automatically created on first SSO login.

**Default Role**: Users are created with the specified default role (typically `member`).

### Username Generation

Usernames are generated from:
1. `preferred_username` from claims (if available)
2. `name` from claims (normalized)
3. Local part of email address

If a username is taken, a numeric suffix is added.

### Existing Users

If a user with the same email exists:
- Their account is linked to the SSO provider
- Last login timestamp is updated
- No password change required

## Role Mappings

For OIDC providers, you can map provider groups to RustChat roles.

### Example Configuration

```json
{
  "admins": "system_admin",
  "moderators": "team_admin",
  "users": "member"
}
```

### How It Works

1. User authenticates via OIDC
2. RustChat extracts groups from the `groups_claim` (default: `groups`)
3. First matching group determines the role
4. If no match, uses default role

### Available Roles

| Role | Description |
|------|-------------|
| `system_admin` | Full system administration |
| `org_admin` | Organization administration |
| `team_admin` | Team administration |
| `member` | Standard user |
| `guest` | Limited access user |

## Testing Configuration

Each provider has a "Test" button that verifies:

### GitHub
- GitHub API reachability
- Configuration completeness

### OIDC (Google/Generic)
- OIDC discovery endpoint accessibility
- JWKS endpoint accessibility
- Endpoint response validation

### Interpreting Test Results

**Success**:
```json
{
  "success": true,
  "message": "OIDC discovery and JWKS fetch successful",
  "details": {
    "issuer": "https://accounts.google.com",
    "authorization_endpoint": "https://accounts.google.com/o/oauth2/v2/auth",
    "token_endpoint": "https://oauth2.googleapis.com/token",
    "userinfo_endpoint": "https://openidconnect.googleapis.com/v1/userinfo",
    "jwks_keys_count": 2
  }
}
```

**Failure**:
```json
{
  "success": false,
  "message": "OIDC discovery failed: 404 Not Found",
  "details": {
    "issuer_url": "https://invalid-issuer.com",
    "discovery_url": "https://invalid-issuer.com/.well-known/openid-configuration"
  }
}
```

## Troubleshooting

### "Invalid or expired OAuth state"

- State parameter expired (5-minute limit)
- User took too long to complete authentication
- Redis connectivity issues

### "OIDC discovery failed"

- Incorrect issuer URL
- Network connectivity issues
- Provider not OIDC-compliant
- Missing `.well-known/openid-configuration` endpoint

### "Email not verified"

- User's email is not verified with the provider
- Some providers (GitHub) may return unverified emails

### "No verified email found"

- GitHub user has no verified primary email
- User needs to verify email on GitHub

### "User is not a member of required GitHub organization"

- User is not a member of the configured organization
- Organization membership is not public (user needs to make it public or grant access)

### "Email domain not allowed"

- User's email domain is not in the allowed domains list
- Check the `allow_domains` configuration

### Redirect URI Mismatch

- Callback URL in provider doesn't match RustChat
- Format: `https://your-domain/api/v1/oauth2/{provider_key}/callback`

## API Reference

### Public Endpoints

#### List Providers
```http
GET /api/v1/oauth2/providers
```

Returns active SSO providers for login page.

**Response**:
```json
[
  {
    "id": "uuid",
    "provider_key": "github",
    "provider_type": "github",
    "display_name": "GitHub",
    "login_url": "https://rustchat.com/api/v1/oauth2/github/login"
  }
]
```

#### Initiate Login
```http
GET /api/v1/oauth2/{provider_key}/login?redirect_uri=/path
```

Redirects to provider's authorization endpoint.

#### Callback
```http
GET /api/v1/oauth2/{provider_key}/callback?code=...&state=...
```

Handles provider callback, creates/updates user, redirects with JWT.

### Admin Endpoints (Require Admin Role)

#### List Configs
```http
GET /api/v1/admin/sso
Authorization: Bearer {token}
```

#### Create Config
```http
POST /api/v1/admin/sso
Authorization: Bearer {token}
Content-Type: application/json

{
  "provider_key": "my-oidc",
  "provider_type": "oidc",
  "display_name": "My SSO",
  "issuer_url": "https://auth.example.com",
  "client_id": "client-id",
  "client_secret": "client-secret",
  "is_active": true,
  "auto_provision": true,
  "default_role": "member"
}
```

#### Update Config
```http
PUT /api/v1/admin/sso/{id}
Authorization: Bearer {token}
```

#### Delete Config
```http
DELETE /api/v1/admin/sso/{id}
Authorization: Bearer {token}
```

#### Test Config
```http
POST /api/v1/admin/sso/{id}/test
Authorization: Bearer {token}
```

## Migration from Legacy SSO

If you have existing SSO configurations:

1. The migration automatically converts legacy configs
2. Review converted configurations in Admin Console
3. Update provider keys if needed (cannot be changed after creation)
4. Test each provider before enabling

## Security Best Practices

1. **Use HTTPS** - Never use HTTP in production
2. **Enable auto-provision carefully** - Consider reviewing users before granting access
3. **Use role mappings** - Automatically assign appropriate roles based on groups
4. **Test before requiring SSO** - Ensure at least one working provider before enabling "Require SSO"
5. **Regular secret rotation** - Update client secrets periodically
6. **Monitor audit logs** - Track SSO login events

## Support

For issues or questions:
- Check the [troubleshooting section](#troubleshooting)
- Review provider-specific documentation
- Check RustChat logs for detailed error messages
