# Debug Logging Added for 422 Error

## Changes Made

Added debug logging to `backend/src/api/admin.rs` in the `update_calls_plugin_config` function:

```rust
// Now accepts raw JSON first
Json(payload): Json<serde_json::Value>

// Logs incoming payload
tracing::info!("Received Calls Plugin config update: {}", payload);

// Manual deserialization with error logging
let payload: UpdateCallsPluginConfig = serde_json::from_value(payload)
    .map_err(|e| {
        tracing::error!("Failed to deserialize Calls Plugin config: {}", e);
        AppError::BadRequest(format!("Invalid configuration data: {}", e))
    })?;
```

## Next Steps

1. **Rebuild the backend Docker container:**
   ```bash
   docker-compose build backend
   docker-compose up -d backend
   ```

2. **Try saving the configuration again** in the admin console

3. **Check the backend logs** for the exact error:
   ```bash
   docker logs rustchat-backend
   ```

You'll see either:
- `Received Calls Plugin config update: {...}` followed by success
- `Failed to deserialize Calls Plugin config: <specific error>`

This will tell us exactly which field is causing the 422 error!
