## 🔍 Debug Logging Added!

I've added detailed logging to diagnose the "Calls are not enabled" error.

### Changes Made

**File**: `backend/src/api/integrations.rs`

Added logging that will show:
1. Raw database value
2. Environment variable value  
3. Final parsed result
4. Detailed error message with both values

### Deploy & Check Logs

```bash
# 1. Build and deploy
docker-compose build backend
docker-compose up -d backend

# 2. Try the /call command
# Type: /call start

# 3. Check logs immediately after
docker logs rustchat-backend | grep -A 2 "Calls enabled"
```

### Expected Output

If working correctly:
```
Calls enabled - DB value: Some("true"), Env value: true
Calls enabled - Final result: true
```

If still failing:
```
Calls enabled - DB value: None, Env value: false
Calls enabled - Final result: false
```

### The Error Message Will Show

The client will now see:
```
Calls are not enabled (db: Some("false"), env: true)
```

This will tell us exactly what's wrong!
