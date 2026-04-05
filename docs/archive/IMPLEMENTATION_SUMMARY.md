# VoIP Push Notification Implementation Summary

## Overview
This document summarizes the implementation of push notifications for call ringing in RustChat.

## Architecture Flow

```
Backend (push_notifications.rs)
    ↓ HTTP POST
Push Proxy (/send endpoint)
    ↓ Routes based on platform
FCM (Android) or APNS (iOS)
    ↓ Push delivery
Mobile Device
    ↓ Wake app + deliver data
Mobile App (React Native)
    ↓ Process notification
Call Notification UI
```

## Backend Implementation

### 1. Backend Service (`backend/src/services/push_notifications.rs`)
- `send_call_ringing_notification()` - Sends call notifications with `sub_type: "calls"`
- Generates `call_uuid` for VoIP identification
- Sends to push proxy or directly to FCM

### 2. Push Proxy (`push-proxy/src/`)
- **Main**: Axum server with `/send` and `/health` endpoints
- **FCM Client** (`fcm.rs`): OAuth2 authentication, data-only messages for calls
- **APNS Client** (`apns.rs`): JWT authentication, VoIP pushes (currently disabled)

### 3. FCM Payload Structure
```json
{
  "message": {
    "token": "<device_token>",
    "data": {
      "type": "call",
      "call_uuid": "<uuid>",
      "channel_id": "<channel_id>",
      "sender_name": "<caller_name>",
      "title": "Incoming Call",
      "body": "<caller> is calling"
    },
    "android": {
      "priority": "high",
      "direct_boot_ok": true
    }
  }
}
```

**Key Points:**
- Data-only message (no `notification` field) - required for custom handling
- `priority: high` - wakes device from Doze mode
- `direct_boot_ok: true` - delivers before user unlocks

## Mobile App Requirements

### Current Status
The backend and push proxy are complete and working. The mobile app needs handlers to process call push notifications.

### Android Implementation Needed

#### Option 1: Standard Push (Current - App in Foreground/Background)
The app uses `react-native-notifications` which handles standard push notifications.
- When app is **foreground**: JS handler processes notification
- When app is **background**: Notification appears in tray, tapping opens app
- When app is **killed**: May not wake up reliably for data-only messages

#### Option 2: Full-Screen Call UI (Recommended for Production)
For WhatsApp/Signal-style incoming call screen:

1. **Create a FirebaseMessagingService**:
```kotlin
class CallMessagingService : FirebaseMessagingService() {
    override fun onMessageReceived(remoteMessage: RemoteMessage) {
        val data = remoteMessage.data
        if (data["type"] == "call") {
            // Show full-screen incoming call activity
            showIncomingCallUI(data)
        }
    }
    
    private fun showIncomingCallUI(data: Map<String, String>) {
        val intent = Intent(this, IncomingCallActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or 
                    Intent.FLAG_ACTIVITY_CLEAR_TOP or
                    Intent.FLAG_ACTIVITY_EXCLUDE_FROM_RECENTS
            putExtra("call_uuid", data["call_uuid"])
            putExtra("channel_id", data["channel_id"])
            putExtra("sender_name", data["sender_name"])
        }
        startActivity(intent)
    }
}
```

2. **Register in AndroidManifest.xml**:
```xml
<service
    android:name=".CallMessagingService"
    android:exported="false">
    <intent-filter>
        <action android:name="com.google.firebase.MESSAGING_EVENT" />
    </intent-filter>
</service>
```

3. **Add to build.gradle**:
```gradle
dependencies {
    implementation 'com.google.firebase:firebase-messaging:23.4.0'
}
```

### iOS Implementation Needed

#### PushKit + CallKit (Required for True VoIP)
1. **Enable Background Modes**:
   - Voice over IP
   - Remote notifications

2. **Register for VoIP pushes**:
```swift
import PushKit
import CallKit

class CallProvider: NSObject, PKPushRegistryDelegate {
    let pushRegistry = PKPushRegistry(queue: .main)
    let callController = CXCallController()
    
    override init() {
        super.init()
        pushRegistry.delegate = self
        pushRegistry.desiredPushTypes = [.voIP]
    }
    
    func pushRegistry(_ registry: PKPushRegistry, 
                      didReceiveIncomingPushWith payload: PKPushPayload,
                      for type: PKPushType) {
        // Extract call data from payload
        let uuid = UUID(uuidString: payload.dictionaryPayload["call_uuid"] as! String)!
        let callerName = payload.dictionaryPayload["sender_name"] as! String
        
        // Report to CallKit
        let update = CXCallUpdate()
        update.remoteHandle = CXHandle(type: .generic, value: callerName)
        update.hasVideo = false
        
        let provider = CXProvider(configuration: CXProviderConfiguration())
        provider.reportNewIncomingCall(with: uuid, update: update) { error in
            // Handle error
        }
    }
}
```

## Testing

### Backend Test Script
```bash
# Send test call notification
curl -X POST http://localhost:3000/api/v4/calls/test-push \
  -H "Authorization: Bearer <token>" \
  -d '{"user_id": "<user_id>"}'
```

### Push Proxy Test
```bash
# Test push proxy directly
curl -X POST http://localhost:3001/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "<fcm_token>",
    "title": "Test Call",
    "body": "Someone is calling",
    "platform": "android",
    "type": "call",
    "data": {
      "channel_id": "test123",
      "post_id": "call456",
      "type": "call",
      "sub_type": "calls",
      "sender_name": "Test User"
    }
  }'
```

## Environment Variables

### Push Proxy
```bash
# Required for FCM
FIREBASE_PROJECT_ID=your-project-id
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json

# Required for APNS (optional)
APNS_KEY_PATH=/path/to/apns-key.p8
APNS_KEY_ID=your-key-id
APNS_TEAM_ID=your-team-id

# Optional
RUSTCHAT_PUSH_PORT=3001
RUSTCHAT_PUSH_TIMEOUT=10
```

### Backend
```bash
# Push proxy URL (if using proxy)
RUSTCHAT_PUSH_PROXY_URL=http://push-proxy:3001

# Or direct FCM
FIREBASE_PROJECT_ID=your-project-id
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

## Current Limitations

1. **Android**: App must be running (foreground/background) to ring immediately. For killed-state ringing, a custom `FirebaseMessagingService` is needed.

2. **iOS**: VoIP pushes require PushKit + CallKit implementation in the mobile app. Currently, standard push notifications are used.

3. **No Fallback**: If push fails, there's no retry mechanism or fallback notification.

## Next Steps for Full Implementation

### Phase 1: Android Full-Screen Calls
- [ ] Create `CallMessagingService` in Android native code
- [ ] Implement `IncomingCallActivity` with answer/decline buttons
- [ ] Test with app in killed state

### Phase 2: iOS CallKit Integration
- [ ] Add PushKit framework
- [ ] Implement `PKPushRegistryDelegate`
- [ ] Add CallKit provider
- [ ] Test background and killed states

### Phase 3: Enhanced Features
- [ ] Add call duration tracking
- [ ] Missed call notifications
- [ ] Call history sync

## References

- [FCM Data Messages](https://firebase.google.com/docs/cloud-messaging/concept-options#data_messages)
- [Android Full-Screen Intents](https://developer.android.com/reference/android/app/Notification.Builder#setFullScreenIntent(android.app.PendingIntent,%20boolean))
- [iOS PushKit Guide](https://developer.apple.com/documentation/pushkit)
- [iOS CallKit Guide](https://developer.apple.com/documentation/callkit)

## Notes

- Mattermost Mobile v2 uses in-app call notifications (React Native component) rather than native full-screen intents
- The `sub_type: "calls"` field is used by the mobile app to identify call notifications
- Data-only FCM messages are required for the app to handle the notification in `onMessageReceived()`
