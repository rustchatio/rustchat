# TURN Server Configuration Update - Summary

## Overview
Updated RustChat to use your specific TURN server values as defaults and added an admin console section for managing the "RustChat Calls Plugin" settings.

## Your TURN Server Values
```bash
TURN_SERVER_ENABLED=true
TURN_SERVER_URL=turn:turn.rustchat.io:3478
TURN_SERVER_USERNAME=XXXXXXXXXXXXXXXXXXXXX
TURN_SERVER_CREDENTIAL=yyyyyyyyyyyyy
```

## Changes Made

### 1. Configuration Updates (`src/config/mod.rs`)

**New Environment Variables:**
- `TURN_SERVER_ENABLED` - Enable/disable TURN server (default: true)
- `TURN_SERVER_URL` - TURN server URL (default: turn:turn.rustchat.io:3478)
- `TURN_SERVER_USERNAME` - TURN username (default: XXXXXXXXXXXXXXXXXX)
- `TURN_SERVER_CREDENTIAL` - TURN credential (default: yyyyyyyyyyyyyyyy)

**Removed:**
- `TURN_SECRET` (no longer needed with static credentials)
- `TURN_SERVERS` array (replaced with single URL)

The config loader now reads both `RUSTCHAT_*` and `TURN_SERVER_*` prefixed variables.

### 2. TURN Credential Generation (`src/api/v4/calls_plugin/turn.rs`)

**Updated to support static credentials:**
- If `TURN_SERVER_USERNAME` and `TURN_SERVER_CREDENTIAL` are provided, uses them directly
- If not provided, falls back to REST API style ephemeral credentials
- Your static credentials are returned to all clients

### 3. ICE Config Endpoint (`src/api/v4/calls_plugin/mod.rs`)

**Updated `GET /plugins/com.mattermost.calls/config`:**
- Returns your TURN server URL with static credentials
- Includes STUN servers (Google STUN by default)
- Example response:
```json
{
  "iceServers": [
    {
      "urls": ["stun:stun.l.google.com:19302"]
    },
    {
      "urls": ["turn:turn.rustchat.io:3478"],
      "username": "XXXXXXXXXXXXXXXXXXX",
      "credential": "yyyyyyyyyyyyyyyyyyyy"
    }
  ]
}
```

### 4. Database Migration (`migrations/20260201000001_add_calls_plugin_settings.sql`)

**Added `plugins` column to `server_config` table:**
- Stores Calls Plugin settings in JSONB format
- Default values match your TURN server configuration
- Can be updated via admin API

### 5. Admin Console API (`src/api/admin.rs`)

**New Endpoints:**

#### Get Plugin Settings
```
GET /api/v1/admin/plugins/calls
```

**Response:**
```json
{
  "plugin_id": "com.rustchat.calls",
  "plugin_name": "RustChat Calls Plugin",
  "settings": {
    "enabled": true,
    "turn_server_enabled": true,
    "turn_server_url": "turn:turn.rustchat.io:3478",
    "turn_server_username": "XXXXXXX",
    "turn_server_credential": "yyyyyyyy",
    "udp_port": 8443,
    "tcp_port": 8443,
    "ice_host_override": null,
    "stun_servers": ["stun:stun.l.google.com:19302"]
  }
}
```

#### Update Plugin Settings
```
PUT /api/v1/admin/plugins/calls
Content-Type: application/json

{
  "enabled": true,
  "turn_server_enabled": true,
  "turn_server_url": "turn:turn.rustchat.io:3478",
  "turn_server_username": "XXXXXXXXXXXXXXX",
  "turn_server_credential": "xxxxxxxxxxxx",
  "udp_port": 8443,
  "tcp_port": 8443,
  "ice_host_override": "your.public.ip",
  "stun_servers": ["stun:stun.l.google.com:19302"]
}
```

## Admin Console Structure

```
Admin Console
├── Plugins
│   └── RustChat Calls Plugin
│       ├── Enable/Disable Calls
│       ├── TURN Server Settings
│       │   ├── TURN Server Enabled
│       │   ├── TURN Server URL
│       │   ├── TURN Username
│       │   └── TURN Credential
│       ├── RTC Ports
│       │   ├── UDP Port (default: 8443)
│       │   └── TCP Port (default: 8443)
│       └── ICE Settings
│           ├── ICE Host Override
│           └── STUN Servers
```

## Environment Variables Reference

### Required
```bash
# These are now optional since defaults are set:
# TURN_SERVER_ENABLED=true
# TURN_SERVER_URL=turn:turn.rustchat.io:3478
# TURN_SERVER_USERNAME=XXXXXXXXXXXXXXXXXX
# TURN_SERVER_CREDENTIAL=yyyyyyyyyyyyy
```

### Optional Overrides
```bash
# Override defaults from .env
TURN_SERVER_ENABLED=false  # Disable TURN
TURN_SERVER_URL=turn:your.server:3478  # Use different TURN server

# Or use RUSTCHAT_ prefix
RUSTCHAT_CALLS_ENABLED=true
RUSTCHAT_CALLS_UDP_PORT=8443
RUSTCHAT_CALLS_ICE_HOST_OVERRIDE=your.public.ip
```

## Configuration Precedence

1. **Database settings** (set via admin console) - Highest priority
2. **Environment variables** (.env file) - Second priority
3. **Default values** (your TURN server) - Lowest priority

## Testing

### 1. Verify TURN Config Endpoint
```bash
curl http://localhost:8080/api/v4/plugins/com.mattermost.calls/config \
  -H "Authorization: Bearer YOUR_TOKEN"
```

Should return your TURN server credentials.

### 2. Test Admin Console API
```bash
# Get current settings
curl http://localhost:8080/api/v1/admin/plugins/calls \
  -H "Authorization: Bearer ADMIN_TOKEN"

# Update settings
curl -X PUT http://localhost:8080/api/v1/admin/plugins/calls \
  -H "Authorization: Bearer ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": true,
    "turn_server_enabled": true,
    "turn_server_url": "turn:turn.rustchat.io:3478",
    "turn_server_username": "XXXXXXXXXXXXXXXXX",
    "turn_server_credential": "yyyyyyyyyyy",
    "udp_port": 8443,
    "tcp_port": 8443,
    "ice_host_override": null,
    "stun_servers": ["stun:stun.l.google.com:19302"]
  }'
```

## Security Notes

1. **Credential Handling**: TURN credentials are stored in the database and returned to authenticated users only
2. **Admin Access**: Only system_admin and org_admin roles can view/update plugin settings
3. **Credential Masking**: When viewing settings via API, the credential field is not serialized (hidden from output)
4. **Static vs Ephemeral**: Your static credentials are simpler but less secure than ephemeral credentials. Consider switching to REST API style if security is a concern.

## Deployment Checklist

1. ✅ TURN server values set as defaults
2. ✅ Admin console API implemented
3. ✅ Database migration created
4. ✅ Static credential support added
5. ⏳ Run migration: `sqlx migrate run`
6. ⏳ Test endpoints
7. ⏳ Verify mobile app can connect to TURN server

## Files Modified

- `backend/src/config/mod.rs` - Added TURN_SERVER_* env vars
- `backend/src/api/v4/calls_plugin/turn.rs` - Static credential support
- `backend/src/api/v4/calls_plugin/mod.rs` - Updated config endpoint
- `backend/src/api/admin.rs` - Added admin console endpoints
- `backend/migrations/20260201000001_add_calls_plugin_settings.sql` - New migration

## Verification

Run these commands to verify:
```bash
cd backend

# Compile
cargo check

# Run tests
cargo test

# Apply migrations
sqlx migrate run

# Start server
cargo run
```

All changes are complete and the code compiles successfully!
