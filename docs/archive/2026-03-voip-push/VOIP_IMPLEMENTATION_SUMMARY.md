# VoIP Push Notifications Implementation Summary

## Overview

This implementation adds comprehensive VoIP push notification support to RustChat, enabling native call ringing on both iOS and Android when the app is in the background.

## Components Implemented

### 1. Push Proxy (`push-proxy/`)

**New Files:**
- `src/apns.rs` - Apple Push Notification Service for VoIP pushes
- `src/fcm.rs` - Firebase Cloud Messaging (enhanced)
- `src/main.rs` - Updated to support both FCM and APNS
- `DOCKER_DEPLOYMENT.md` - Deployment guide

**Features:**
- Dual platform support (FCM for Android, APNS for iOS VoIP)
- Platform-specific payload generation
- Call UUID generation for CallKit integration
- Certificate-based APNS authentication
- Health check endpoint
- Comprehensive error handling

### 2. Backend Push Service (`backend/src/services/push_notifications.rs`)

**Enhancements:**
- Added `NotificationType` enum (Message/Call)
- Enhanced `PushNotification` struct with notification type
- Updated `PushProxyPayload` with platform and call_uuid fields
- Platform-specific payload generation
- Call UUID generation for VoIP pushes

### 3. Documentation (`docs/`)

**New Files:**
- `VOIP_PUSH_NOTIFICATIONS.md` - Complete integration guide for mobile developers
- `VOIP_IMPLEMENTATION_SUMMARY.md` - This document

## How It Works

### Call Flow

1. **Call Initiated**
   ```
   User A starts call → POST /plugins/com.mattermost.calls/calls/{id}/start
   ```

2. **Backend Processing**
   ```
   - Create call state
   - Get channel members
   - Send WebSocket events (for online users)
   - Send push notifications (for offline/background users)
   ```

3. **Push Notification Flow**
   ```
   RustChat Backend → Push Proxy → FCM/APNS → Mobile Device
   ```

4. **Mobile Handling**
   
   **iOS:**
   ```
   APNS VoIP Push → PKPushRegistry → CallKit → Native Call UI
   ```
   
   **Android:**
   ```
   FCM Data Message → FirebaseMessagingService → Full-screen Intent → Call UI
   ```

## Configuration

### Required Environment Variables

**Push Proxy:**
```bash
# Firebase (Android)
FIREBASE_PROJECT_ID=your-project-id
GOOGLE_APPLICATION_CREDENTIALS=/secrets/firebase-key.json

# APNS (iOS VoIP)
APNS_CERT_PATH=/secrets/voip-cert.pem
APNS_KEY_PATH=/secrets/voip-key.pem
APNS_BUNDLE_ID=com.rustchat.app
APNS_USE_PRODUCTION=false
```

**RustChat Backend:**
```bash
RUSTCHAT_PUSH_PROXY_URL=http://push-proxy:3000
```

## Payload Format

### Push Proxy Request

```json
{
  "token": "device-token",
  "title": "Incoming call from John Doe",
  "body": "Tap to answer",
  "platform": "ios",
  "type": "call",
  "data": {
    "channel_id": "channel-uuid",
    "post_id": "post-uuid",
    "type": "message",
    "sub_type": "calls",
    "version": "2",
    "sender_id": "sender-uuid",
    "sender_name": "John Doe",
    "server_url": "https://rustchat.com",
    "call_uuid": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### iOS VoIP Push Payload

```json
{
  "aps": {
    "alert": {
      "title": "John Doe",
      "body": "Incoming call"
    },
    "sound": "calls_ringtone.caf",
    "badge": 1,
    "content-available": 1,
    "mutable-content": 1
  },
  "data": {
    "type": "call",
    "call_uuid": "550e8400-e29b-41d4-a716-446655440000",
    "caller_name": "John Doe",
    "channel_id": "channel-uuid",
    "server_url": "https://rustchat.com",
    "is_voip": true
  }
}
```

### Android FCM Payload (Data-Only)

**CRITICAL:** For VoIP ringing to work on Android, we use **data-only messages** (no `notification` field). This ensures:
- The app receives the message in `FirebaseMessagingService.onMessageReceived()`
- The app can wake up and show a full-screen incoming call UI
- System notification tray is NOT shown automatically

```json
{
  "message": {
    "token": "fcm-token",
    "data": {
      "type": "call",
      "channel_id": "channel-uuid",
      "post_id": "post-uuid",
      "sender_name": "John Doe",
      "server_url": "https://rustchat.com",
      "title": "Incoming call from John Doe",
      "body": "Tap to answer"
    },
    "android": {
      "priority": "high",
      "ttl": "0s",
      "direct_boot_ok": true
    }
  }
}
```

**Note:** The absence of the `notification` field is intentional and required for proper VoIP behavior.

## Mobile Integration Requirements

### iOS

**Capabilities:**
- Push Notifications
- Background Modes → Voice over IP
- Background Modes → Remote Notifications

**Frameworks:**
- PushKit
- CallKit

**Implementation:**
1. Register for VoIP pushes (`PKPushRegistry`)
2. Implement `PKPushRegistryDelegate`
3. Configure CallKit provider
4. Handle incoming calls via `reportNewIncomingCall`

### Android

**Permissions:**
```xml
<uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
<uses-permission android:name="android.permission.USE_FULL_SCREEN_INTENT" />
<uses-permission android:name="android.permission.MANAGE_OWN_CALLS" />
```

**Implementation:**
1. Extend `FirebaseMessagingService`
2. Create notification channel for calls
3. Build full-screen notification
4. Handle notification actions (answer/decline)

## Testing

### Backend Testing

```bash
# Start push proxy
cd push-proxy && docker-compose up -d

# Test health endpoint
curl http://localhost:3000/health

# Send test push
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "test-token",
    "title": "Test Call",
    "body": "Test",
    "platform": "ios",
    "type": "call",
    "data": {"channel_id": "test", "post_id": "test", "type": "call", "sub_type": "calls"}
  }'
```

### Mobile Testing

**iOS:**
- Requires physical device (simulator doesn't support push)
- Use Apple Push Notification Console
- Test CallKit integration

**Android:**
- Can use emulator with Google Play Services
- Use Firebase Console to send test messages
- Test full-screen intent on locked device

## Deployment Checklist

- [ ] Generate Firebase service account key
- [ ] Generate APNS VoIP certificate
- [ ] Configure environment variables
- [ ] Deploy push proxy container
- [ ] Configure RustChat backend with push proxy URL
- [ ] Test push notifications on both platforms
- [ ] Monitor logs for errors
- [ ] Set up health check monitoring

## Migration Guide

### From Direct FCM to Push Proxy

1. Deploy push proxy service
2. Set `RUSTCHAT_PUSH_PROXY_URL` environment variable
3. Backend will automatically use push proxy
4. Direct FCM becomes fallback if push proxy is unavailable

### Mobile App Updates

No changes required to RustChat backend API. Mobile apps need to:

1. **iOS:** Add CallKit and PushKit integration
2. **Android:** Add FCM message handler for call type

## Known Limitations

1. **iOS:** 
   - VoIP pushes must be handled within milliseconds or app is terminated
   - Certificate must be renewed annually

2. **Android:**
   - Doze mode may delay notifications
   - Full-screen intent requires special permission

3. **General:**
   - Requires device token registration
   - Invalid tokens must be cleaned up periodically

## Future Enhancements

1. **Push Proxy:**
   - Add metrics endpoint for Prometheus
   - Implement notification queuing for reliability
   - Add support for APNS token-based authentication

2. **Backend:**
   - Batch push notifications for multiple recipients
   - Add push notification analytics
   - Implement retry logic with exponential backoff

3. **Mobile:**
   - Add support for call waiting
   - Implement call forwarding
   - Add picture-in-picture support

## References

- [Apple PushKit Documentation](https://developer.apple.com/documentation/pushkit)
- [Apple CallKit Documentation](https://developer.apple.com/documentation/callkit)
- [Firebase Cloud Messaging](https://firebase.google.com/docs/cloud-messaging)
- [Android Telecom Framework](https://developer.android.com/reference/android/telecom/package-summary)

## Support

For issues or questions:
1. Check troubleshooting section in `VOIP_PUSH_NOTIFICATIONS.md`
2. Review push proxy logs: `docker-compose logs push-proxy`
3. Verify certificate validity
4. Test with cURL commands provided above
