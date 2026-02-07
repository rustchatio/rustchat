# Fix: Calls Plugin Not Enabled Error

## Problem
After saving the configuration via admin console, the `/call start` command still returns "Calls are not enabled" error.

## Root Cause
The `/call` slash command was only checking the environment variable configuration (`state.config.calls.enabled`) instead of checking the database configuration that was saved via the admin console.

## Solution
Modified the `/call` command handler in `src/api/integrations.rs` to:
1. First check the database for the enabled setting
2. Fall back to environment variable if not found in database
3. Use `COALESCE` to handle null values properly

## Code Change
```rust
// Before: Only checked env config
if !state.config.calls.enabled {
    return Ok(CommandResponse {
        response_type: "ephemeral".to_string(),
        text: "Calls are not enabled".to_string(),
        ...
    });
}

// After: Checks database first, then env
let calls_enabled = sqlx::query_scalar::<_, bool>(
    "SELECT COALESCE(plugins->'calls'->>'enabled', $1)::boolean FROM server_config WHERE id = 'default'"
)
.bind(state.config.calls.enabled.to_string())
.fetch_optional(&state.db)
.await?
.unwrap_or(state.config.calls.enabled);

if !calls_enabled {
    return Ok(CommandResponse {
        response_type: "ephemeral".to_string(),
        text: "Calls are not enabled".to_string(),
        ...
    });
}
```

## Deployment

```bash
# Rebuild and restart backend
docker-compose build backend
docker-compose up -d backend

# Verify it's working
docker logs rustchat-backend | grep -i "call\|plugin"
```

## Testing
1. Save configuration in admin console (enable plugin)
2. Type `/call start` in a channel
3. Should now start the call without "not enabled" error

## Status
✅ Fix applied and compiled successfully
⏳ Ready for deployment
