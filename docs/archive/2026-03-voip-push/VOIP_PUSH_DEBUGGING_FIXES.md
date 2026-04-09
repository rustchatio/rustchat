# VoIP Push Debugging & Additional Fixes

## Problem Reported
"After deploy if I use the android curl nothing happens"

## Root Causes Found and Fixed

### 1. Missing Debug Logging (FIXED)

**Problem:** No visibility into what the push proxy was doing.

**Fix:** Added comprehensive logging throughout the request flow:

```rust
// In send_notification:
info!(platform = %platform, is_call = is_call, "Received push notification request");

// In send_fcm_push:
info!("FCM client is available, building payload");
info!("FCM payload built, sending to FCM client");

// In fcm::send:
info!("OAuth token obtained, building FCM message");
info!(status = %status, "Received FCM API response");
```

**How to use:**
```bash
# Enable debug logging
docker-compose restart push-proxy

# Watch logs
docker-compose logs push-proxy -f
```

### 2. Android Config Had `notification` Field (FIXED)

**Problem:** Even though we removed the top-level `notification` field for calls, the `android_config` still had:
```json
"notification": {
    "channel_id": "channel_01",
    "sound": "default",
    "click_action": "TOP_STORY_ACTIVITY"
}
```

This could cause Android to still show a system notification instead of delivering to `onMessageReceived()`.

**Fix:** Removed `notification` from `android_config` for calls:
```rust
let android_config = if is_call {
    serde_json::json!({
        "priority": "high",
        "ttl": "0s",
        // NO "notification" field here either!
        "direct_boot_ok": true
    })
}
```

### 3. Missing `call_uuid` in FCM Data (FIXED)

**Problem:** The `call_uuid` field was being extracted from the request but never added to the FCM data payload.

**Fix:** 
1. Added `call_uuid` to `fcm::PushData` struct
2. Passed it through from main.rs to FCM
3. Included it in the data payload:
```rust
"data": {
    "type": "call",
    "channel_id": payload.data.channel_id,
    // ...
    "call_uuid": payload.data.call_uuid.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
}
```

## New Test Script

Created `push-proxy/test-push.sh` for easy testing:

```bash
# Make it executable
chmod +x push-proxy/test-push.sh

# Test Android
./push-proxy/test-push.sh android YOUR_DEVICE_TOKEN

# Test iOS
./push-proxy/test-push.sh ios YOUR_DEVICE_TOKEN
```

This script will:
1. Check health endpoint
2. Send test push
3. Show detailed output
4. Explain expected behavior

## Debugging Checklist

If "nothing happens", check these in order:

### Step 1: Is Push Proxy Running?
```bash
curl http://localhost:3000/health
# Should return: {"status":"ok",...}
```

### Step 2: Check Logs
```bash
docker-compose logs push-proxy --tail=50
```

Look for:
- "Received push notification request" - Request received
- "Routing to Android/FCM handler" - Platform detection working
- "FCM client is available" - FCM initialized
- "OAuth token obtained" - Authentication working
- "Received FCM API response status=200" - FCM accepted the message

### Step 3: Verify Request Format
The request MUST have:
```json
{
  "platform": "android",
  "type": "call",
  "data": {
    "sub_type": "calls"
  }
}
```

Without `sub_type: "calls"`, it won't be treated as a call notification.

### Step 4: Check FCM Configuration
```bash
# In container
docker-compose exec push-proxy env | grep -E "(FIREBASE|FCM)"
```

Should show:
- `FIREBASE_PROJECT_ID=your-project-id`
- `GOOGLE_APPLICATION_CREDENTIALS=/secrets/firebase-key.json`

### Step 5: Verify Device Token
Test with a fresh token from your Android app. Tokens can become invalid if:
- App was uninstalled and reinstalled
- Token wasn't refreshed
- Wrong Firebase project

## Expected Behavior After Fixes

### Android Data-Only Message Flow:

1. **Request received:**
   ```
   INFO Received push notification request platform=android is_call=true
   ```

2. **FCM processes:**
   ```
   INFO Routing to Android/FCM handler
   INFO FCM client is available, building payload
   INFO Building FCM message is_call=true
   INFO FCM message built, sending to FCM client
   INFO OAuth token obtained, building FCM message
   INFO Received FCM API response status=200
   INFO Successfully sent notification to FCM
   ```

3. **FCM delivers to device:**
   - Message delivered to `FirebaseMessagingService.onMessageReceived()`
   - Even if app is in background or killed
   - System notification is NOT shown

4. **App handles message:**
   ```java
   @Override
   public void onMessageReceived(RemoteMessage remoteMessage) {
       Map<String, String> data = remoteMessage.getData();
       Log.d("FCM", "Data: " + data); // Should show type=call
       
       if ("call".equals(data.get("type"))) {
           // Show full-screen call UI
       }
   }
   ```

## Common "Nothing Happens" Causes

| Symptom | Cause | Fix |
|---------|-------|-----|
| No connection | Push proxy not running | `docker-compose up -d push-proxy` |
| 503 error | FCM not configured | Set `FIREBASE_PROJECT_ID` and key path |
| 410 error | Invalid device token | Get fresh token from device |
| 401 error | Wrong Firebase project | Verify key file matches project |
| Message not received | `sub_type` not "calls" | Add `"sub_type": "calls"` to request |
| Notification shown | App not handling data msg | Implement `onMessageReceived()` |
| Silent failure | No logging enabled | Set `RUST_LOG=push_proxy=debug` |

## Files Modified

1. `push-proxy/src/main.rs` - Added logging, fixed call_uuid passing
2. `push-proxy/src/fcm.rs` - Fixed android_config, added call_uuid to data
3. `push-proxy/test-push.sh` - New test script
4. `push-proxy/TROUBLESHOOTING.md` - New troubleshooting guide

## Testing After Deploy

```bash
# 1. Rebuild and restart
docker-compose up -d --build push-proxy

# 2. Check logs
docker-compose logs push-proxy -f

# 3. Run test (in another terminal)
./push-proxy/test-push.sh android YOUR_DEVICE_TOKEN

# 4. Verify in logs that you see:
#    - "Received push notification request"
#    - "Successfully sent notification to FCM"
```

## Next Steps

If it still doesn't work after these fixes:

1. Run the test script and capture output
2. Check push proxy logs
3. Verify Android app has:
   - `FirebaseMessagingService` implementation
   - Proper manifest declarations
   - Logging in `onMessageReceived()`
4. Share the logs for further analysis
