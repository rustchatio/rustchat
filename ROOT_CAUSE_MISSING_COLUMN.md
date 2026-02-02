# Root Cause Found: Missing Database Column! 🎯

## Problem Identified

The `plugins` column doesn't exist in the `server_config` table!

**Current Database State:**
- Table exists with: `id, site, authentication, integrations, compliance, email, experimental`
- **Missing:** `plugins` column (needed for Calls Plugin settings)
- 10 migrations are pending including the one that adds this column

## Solution

**Run the database migrations:**

```bash
# Option 1: Using docker-compose
docker-compose exec backend sqlx migrate run

# Option 2: Using cargo (if you have Rust installed)
cd backend
cargo sqlx migrate run

# Option 3: Manual SQL execution
docker-compose exec postgres psql -U rustchat -d rustchat -f /migrations/20260201000001_add_calls_plugin_settings.sql
```

## Alternative Quick Fix

If migrations fail, manually add the column:

```bash
docker-compose exec postgres psql -U rustchat -d rustchat -c "
ALTER TABLE server_config 
ADD COLUMN IF NOT EXISTS plugins JSONB NOT NULL DEFAULT '{
    \"calls\": {
        \"enabled\": true,
        \"turn_server_enabled\": true,
        \"turn_server_url\": \"turn:turn.kubedo.io:3478\",
        \"turn_server_username\": \"PtU7Uv7NdR2YcBJMC5n6EdfGoFhXLp\",
        \"turn_server_credential\": \"axY1ofBashEbJat9\",
        \"udp_port\": 8443,
        \"tcp_port\": 8443,
        \"ice_host_override\": null,
        \"stun_servers\": [\"stun:stun.l.google.com:19302\"]
    }
}'::jsonb;

UPDATE server_config 
SET plugins = '{
    \"calls\": {
        \"enabled\": true,
        \"turn_server_enabled\": true,
        \"turn_server_url\": \"turn:turn.kubedo.io:3478\",
        \"turn_server_username\": \"PtU7Uv7NdR2YcBJMC5n6EdfGoFhXLp\",
        \"turn_server_credential\": \"axY1ofBashEbJat9\",
        \"udp_port\": 8443,
        \"tcp_port\": 8443,
        \"ice_host_override\": null,
        \"stun_servers\": [\"stun:stun.l.google.com:19302\"]
    }
}'::jsonb
WHERE plugins IS NULL OR plugins = '{}'::jsonb;
"
```

## Verify Fix

After running migrations:
```bash
docker-compose exec postgres psql -U rustchat -d rustchat -c "SELECT plugins->'calls'->>'enabled' FROM server_config WHERE id='default';"
# Should return: true
```

Then test `/call start` again!

## Why This Happened

The backend Docker container starts and says "Database connected and migrations applied" but it only applies migrations up to 20260124. The newer migrations (including the one for Calls Plugin) weren't applied because:
1. The SQLX_OFFLINE=true mode might have cached old schema
2. Or migrations need to be run manually

## Summary

**Before:** Database missing `plugins` column → Query returns NULL → Calls appear disabled
**After:** Migration adds column with default `enabled: true` → Calls work!

Run the migration now! 🚀
