# VoIP Push Notifications Implementation - Complete

## Summary

This implementation adds comprehensive VoIP push notification support to RustChat, enabling native call ringing on both iOS and Android when the app is in the background.

## What Was Implemented

### 1. Push Proxy Enhancement (`push-proxy/`)

**New Files:**
- `src/apns.rs` - Apple Push Notification Service for VoIP pushes
  - `ApnsClient` - HTTP/2 client for APNS
  - `ApnsConfig` - Certificate configuration
  - `ApnsVoipPayload` - VoIP-specific payload structure
  - `build_voip_topic()` - Helper to construct APNS topic

**Updated Files:**
- `src/fcm.rs` - Enhanced for call-specific payloads
  - Data-only messages for Android background handling
  - Call-specific notification channels
  - Custom ringtone support

- `src/main.rs` - Dual-platform support
  - FCM initialization (Android)
  - APNS initialization (iOS VoIP)
  - Platform-specific routing
  - Health check endpoint

- `Cargo.toml` - Added `uuid` dependency

**New Configuration Files:**
- `.env.example` - Environment variable template
- `docker-compose.yml` - Standalone deployment
- `DOCKER_DEPLOYMENT.md` - Comprehensive deployment guide

### 2. Backend Push Service (`backend/src/services/push_notifications.rs`)

**Added:**
- `NotificationType` enum (`Message` / `Call`)
- `notification_type` field to `PushNotification` struct
- `platform` field to `PushProxyPayload`
- `call_uuid` field to `PushProxyData` (for iOS CallKit)
- Automatic call UUID generation for VoIP pushes
- Platform detection for targeted delivery

### 3. Documentation (`docs/`)

**New Files:**
- `VOIP_PUSH_NOTIFICATIONS.md` - Complete mobile integration guide
  - iOS implementation with CallKit/PushKit
  - Android implementation with ConnectionService
  - Code samples in Objective-C, Swift, and Java
  - Testing procedures
  - Troubleshooting guide

- `VOIP_IMPLEMENTATION_SUMMARY.md` - Technical architecture overview

### 4. Docker Compose Integration

**Updated Files:**
- `docker-compose.yml` - Enhanced push-proxy service
  - APNS environment variables
  - Certificate volume mounts
  - Improved health check

- `.env.example` - Added APNS configuration variables

## How It Works

### Call Flow

```
1. User starts call
   ↓
2. Backend sends WebSocket event (online users)
   ↓
3. Backend sends Push Notification (offline/background users)
   ↓
4. Push Proxy routes to correct service
   ├─ Android → FCM (data message with type: call)
   └─ iOS → APNS (VoIP push with call_uuid)
   ↓
5. Mobile device receives push
   ├─ iOS: CallKit displays native call UI
   └─ Android: Full-screen notification with answer/decline
```

## Configuration Quick Start

### 1. Firebase Setup (Android)

```bash
# 1. Download service account key from Firebase Console
# 2. Save to secrets/firebase-key.json
# 3. Set environment variable
export FIREBASE_PROJECT_ID=your-project-id
export FIREBASE_KEY_PATH=./secrets/firebase-key.json
```

### 2. APNS Setup (iOS)

```bash
# 1. Generate VoIP certificate at Apple Developer Portal
# 2. Convert to PEM format
openssl x509 -in voip-cert.cer -inform DER -out voip-cert.pem

# 3. Set environment variables
export APNS_CERT_PATH=./secrets/voip-cert.pem
export APNS_KEY_PATH=./secrets/voip-key.pem
export APNS_BUNDLE_ID=com.rustchat.app
export APNS_USE_PRODUCTION=false
```

### 3. Deploy

```bash
# Start push proxy
docker-compose up -d push-proxy

# Verify health
curl http://localhost:3000/health
# Expected: {"status":"ok","service":"rustchat-push-proxy"}
```

## Payload Examples

### iOS VoIP Push (APNS)

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

### Android Push (FCM)

```json
{
  "message": {
    "token": "fcm-token",
    "notification": {
      "title": "Incoming call from John Doe",
      "body": "Tap to answer"
    },
    "data": {
      "type": "call",
      "channel_id": "channel-uuid",
      "sender_name": "John Doe",
      "server_url": "https://rustchat.com"
    },
    "android": {
      "priority": "high",
      "direct_boot_ok": true
    }
  }
}
```

## Mobile Integration Checklist

### iOS

- [ ] Add PushKit framework
- [ ] Add CallKit framework
- [ ] Enable VoIP background mode
- [ ] Register for VoIP pushes (`PKPushRegistry`)
- [ ] Implement `PKPushRegistryDelegate`
- [ ] Configure CallKit provider
- [ ] Handle incoming calls with `reportNewIncomingCall`
- [ ] Implement answer/decline handlers

### Android

- [ ] Add Firebase Messaging dependency
- [ ] Extend `FirebaseMessagingService`
- [ ] Create notification channel for calls
- [ ] Build full-screen notification
- [ ] Add answer/decline actions
- [ ] Handle notification tap
- [ ] Implement call connection logic

## Testing

### Backend Test

```bash
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "test-token",
    "title": "Test Call",
    "body": "Test",
    "platform": "ios",
    "type": "call",
    "data": {
      "channel_id": "test",
      "type": "call",
      "sub_type": "calls",
      "sender_name": "Test User",
      "call_uuid": "550e8400-e29b-41d4-a716-446655440000"
    }
  }'
```

### End-to-End Test

1. Start a call in a DM/GM channel
2. Verify push notification is sent
3. Check mobile device shows incoming call UI
4. Verify answer/decline actions work
5. Confirm call connects successfully

## Files Changed

```
push-proxy/
├── src/
│   ├── apns.rs (NEW)
│   ├── fcm.rs (UPDATED)
│   └── main.rs (UPDATED)
├── Cargo.toml (UPDATED)
├── .env.example (NEW)
├── docker-compose.yml (NEW)
├── DOCKER_DEPLOYMENT.md (NEW)

backend/
└── src/services/
    └── push_notifications.rs (UPDATED)

docs/
├── VOIP_PUSH_NOTIFICATIONS.md (NEW)
└── VOIP_IMPLEMENTATION_SUMMARY.md (NEW)

root/
├── docker-compose.yml (UPDATED)
└── .env.example (UPDATED)
```

## Next Steps

### For Mobile Developers

1. Review `docs/VOIP_PUSH_NOTIFICATIONS.md`
2. Implement iOS CallKit integration
3. Implement Android FCM handler
4. Test on physical devices

### For DevOps

1. Review `push-proxy/DOCKER_DEPLOYMENT.md`
2. Generate certificates and keys
3. Configure environment variables
4. Deploy push proxy service
5. Monitor health check endpoint

### For Backend Developers

No changes required - the backend automatically:
- Detects platform from device registration
- Generates call UUIDs for VoIP pushes
- Routes to appropriate push service

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Push proxy not starting | Check environment variables are set |
| FCM errors | Verify service account key file |
| APNS errors | Check certificate validity and bundle ID |
| No push received | Verify device token is registered |
| Call UI not showing | Check mobile app integration |

## References

- [Complete Mobile Integration Guide](docs/VOIP_PUSH_NOTIFICATIONS.md)
- [Deployment Guide](push-proxy/DOCKER_DEPLOYMENT.md)
- [Architecture Overview](docs/VOIP_IMPLEMENTATION_SUMMARY.md)
- [Apple PushKit Docs](https://developer.apple.com/documentation/pushkit)
- [Apple CallKit Docs](https://developer.apple.com/documentation/callkit)
- [Firebase FCM Docs](https://firebase.google.com/docs/cloud-messaging)

---

**Implementation Date:** 2026-02-17  
**Status:** Complete and Ready for Integration
