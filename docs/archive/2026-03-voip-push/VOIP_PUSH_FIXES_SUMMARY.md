# VoIP Push Notification Fixes Summary

## Problem
After the initial implementation, calls were showing notifications but not ringing (no native incoming call UI was displayed on mobile devices).

## Root Causes and Fixes

### 1. Android: Data-Only Messages (FIXED)

**Problem:**
FCM messages included a `notification` field, which caused Android to automatically show a system tray notification. The app never woke up to display the full-screen incoming call UI.

**Fix:**
In `push-proxy/src/fcm.rs`, modified `build_fcm_message()` to send **data-only messages** for calls:

```rust
let message = if is_call {
    serde_json::json!({
        "token": payload.token,
        // NO "notification" field - this is critical for VoIP ringing!
        "data": {
            "type": "call",
            "channel_id": payload.data.channel_id,
            "post_id": payload.data.post_id,
            "sender_name": payload.data.sender_name.unwrap_or_default(),
            "server_url": payload.data.server_url.unwrap_or_default(),
            "title": payload.title,
            "body": payload.body,
        },
        "android": android_config,
        "apns": apns_config
    })
}
```

**Why This Works:**
- Data-only messages are delivered to `FirebaseMessagingService.onMessageReceived()` even when the app is in background or killed
- The app can then show a full-screen incoming call UI with `startActivity()` using `FLAG_ACTIVITY_NEW_TASK | FLAG_ACTIVITY_CLEAR_TOP`
- System notification tray is not shown automatically

### 2. iOS: APNS Authentication (FIXED)

**Problem:**
APNS client wasn't using any authentication. The HTTP/2 APNS API requires JWT-based authentication using a .p8 auth key.

**Fix:**
Rewrote `push-proxy/src/apns.rs` to use JWT-based authentication:

```rust
pub struct ApnsClient {
    http_client: reqwest::Client,
    pub config: ApnsConfig,
    auth_token: String,
    token_expires_at: chrono::DateTime<chrono::Utc>,
}

// JWT token generation
async fn generate_jwt_token(config: &ApnsConfig) -> Result<(String, chrono::DateTime<chrono::Utc>), ApnsError> {
    let claims = ApnsJwtClaims {
        iss: config.team_id.clone(),
        iat: now.timestamp(),
    };
    
    let token = encode(&header, &claims, &key)?;
    // ...
}
```

**New Configuration:**
Instead of certificate-based auth, now uses JWT auth which is the modern recommended approach:

```bash
APNS_KEY_PATH=./secrets/AuthKey_KEYID.p8
APNS_KEY_ID=YOUR_KEY_ID
APNS_TEAM_ID=YOUR_TEAM_ID
APNS_BUNDLE_ID=com.rustchat.app
```

**Why This Works:**
- JWT authentication is the modern standard for APNS HTTP/2 API
- Single auth key works for all apps in your team
- No certificate expiration issues
- Required headers: `authorization: bearer <jwt>`, `apns-topic: <bundle>.voip`, `apns-push-type: voip`

### 3. Updated Documentation and Configuration

**Files Updated:**
1. `push-proxy/.env.example` - New JWT-based APNS variables
2. `docker-compose.yml` - Updated volume mounts and env vars
3. `.env.example` - New APNS configuration format
4. `push-proxy/DOCKER_DEPLOYMENT.md` - Updated setup instructions
5. `push-proxy/Cargo.toml` - Added `jsonwebtoken` and `chrono` dependencies

## How to Set Up

### 1. Android (FCM)

Already configured. Just ensure `FIREBASE_PROJECT_ID` and `FIREBASE_KEY_PATH` are set.

### 2. iOS (APNS) - NEW SETUP REQUIRED

The old certificate-based setup no longer works. Follow these steps:

1. **Generate APNS Auth Key:**
   - Go to [Apple Developer Portal](https://developer.apple.com/)
   - Keys → Add Key
   - Name: "Push Notifications Key"
   - Enable: "Apple Push Notifications service (APNs)"
   - Download `.p8` file

2. **Get Required IDs:**
   - Key ID: Shown next to your key in the portal
   - Team ID: Membership → Team ID
   - Bundle ID: Your app's bundle identifier (e.g., `com.rustchat.app`)

3. **Configure Environment:**
   ```bash
   APNS_KEY_PATH=./secrets/AuthKey_ABCD123456.p8
   APNS_KEY_ID=ABCD123456
   APNS_TEAM_ID=TEAM123456
   APNS_BUNDLE_ID=com.rustchat.app
   APNS_USE_PRODUCTION=false
   ```

## Testing

### Test Android (Data-Only Message)

```bash
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "fcm-device-token",
    "title": "Incoming call from John",
    "body": "Tap to answer",
    "platform": "android",
    "type": "call",
    "data": {
      "channel_id": "test-channel",
      "post_id": "test-post",
      "type": "message",
      "sub_type": "calls",
      "sender_name": "John Doe",
      "server_url": "https://rustchat.com"
    }
  }'
```

**Expected:** App should receive message in `onMessageReceived()` and can show full-screen call UI.

### Test iOS (VoIP Push)

```bash
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "apns-device-token",
    "title": "Incoming call from John",
    "body": "Tap to answer",
    "platform": "ios",
    "type": "call",
    "data": {
      "channel_id": "test-channel",
      "post_id": "test-post",
      "type": "message",
      "sub_type": "calls",
      "sender_name": "John Doe",
      "server_url": "https://rustchat.com",
      "call_uuid": "550e8400-e29b-41d4-a716-446655440000"
    }
  }'
```

**Expected:** CallKit should display native incoming call UI immediately.

## Mobile Integration Requirements

### Android

Your `FirebaseMessagingService` must handle data messages:

```java
@Override
public void onMessageReceived(RemoteMessage remoteMessage) {
    Map<String, String> data = remoteMessage.getData();
    if ("call".equals(data.get("type"))) {
        // Show full-screen incoming call UI
        Intent intent = new Intent(this, IncomingCallActivity.class);
        intent.putExtra("call_uuid", data.get("call_uuid"));
        intent.putExtra("caller_name", data.get("sender_name"));
        intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
        startActivity(intent);
    }
}
```

### iOS

Your `PKPushRegistryDelegate` must handle VoIP pushes:

```objc
- (void)pushRegistry:(PKPushRegistry *)registry 
didReceiveIncomingPushWithPayload:(PKPushPayload *)payload 
             forType:(PKPushType)type 
 withCompletionHandler:(void (^)(void))completion {
    
    NSDictionary *data = payload.dictionaryPayload;
    NSString *callUUID = data[@"data"][@"call_uuid"];
    NSString *callerName = data[@"data"][@"caller_name"];
    
    // Report to CallKit
    CXCallUpdate *update = [[CXCallUpdate alloc] init];
    update.localizedCallerName = callerName;
    
    [self.callProvider reportNewIncomingCallWithUUID:uuid 
                                              update:update 
                                          completion:^(NSError *error) {
        completion();
    }];
}
```

## Verification Checklist

- [ ] Push proxy starts without errors
- [ ] Health check returns `{"status":"ok"}`
- [ ] Android: Data-only FCM message received in `onMessageReceived()`
- [ ] Android: Full-screen call UI appears when app is backgrounded
- [ ] iOS: VoIP push received in `didReceiveIncomingPushWithPayload`
- [ ] iOS: CallKit shows native incoming call UI
- [ ] Call connects successfully when answered

## Troubleshooting

### Android: Still showing notification instead of ringing

1. Check FCM payload has NO `notification` field
2. Verify `android.priority` is set to `high`
3. Ensure `FirebaseMessagingService` is properly declared in manifest
4. Check app has `POST_NOTIFICATIONS` permission

### iOS: Push not received

1. Verify APNS_KEY_ID, APNS_TEAM_ID, APNS_BUNDLE_ID are correct
2. Check .p8 file is valid and readable
3. Ensure app has PushKit and CallKit entitlements
4. Verify device token is registered for VoIP pushes
5. Check APNS_USE_PRODUCTION matches your provisioning profile

## Summary

The key fixes were:
1. **Android**: Use data-only FCM messages (no `notification` field)
2. **iOS**: Implement JWT-based APNS authentication with proper headers

Both platforms now properly wake up the app to show the native incoming call UI instead of just displaying a notification.
