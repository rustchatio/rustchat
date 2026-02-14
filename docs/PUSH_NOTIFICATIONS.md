# Push Notification Setup Guide

## Overview

This guide explains how to configure push notifications for the rustchat backend to support:
- **Call ringing notifications** on mattermost-mobile (Android/iOS)
- **Message notifications** for mentions and direct messages

## Architecture

Push notifications are sent via:
1. **FCM (Firebase Cloud Messaging)** for Android devices
2. **APNS (Apple Push Notification Service)** for iOS devices

The backend stores device tokens when mobile apps register and sends push notifications in addition to WebSocket events.

## Setup Instructions

### 1. Firebase Cloud Messaging (Android)

#### Step 1: Create Firebase Project
1. Go to [Firebase Console](https://console.firebase.google.com/)
2. Create a new project or select existing
3. Add an Android app with your package name (e.g., `com.mattermost.rnbeta`)
4. Download `google-services.json` (you'll need this for the mobile app)

#### Step 2: Generate Service Account Key
1. In Firebase Console, go to Project Settings → Service Accounts
2. Click "Generate new private key"
3. Save the JSON file securely

#### Step 3: Get Access Token
The backend needs an OAuth2 access token to send push notifications. You have two options:

**Option A: Using the private key directly (Recommended for testing)**
```bash
# Install google-auth-library
pip install google-auth

# Generate access token
python3 -c "
import json
from google.oauth2 import service_account
from google.auth.transport.requests import Request

# Load your service account JSON
with open('service-account.json') as f:
    credentials = service_account.Credentials.from_service_account_info(
        json.load(f),
        scopes=['https://www.googleapis.com/auth/cloud-platform']
    )

# Get access token
credentials.refresh(Request())
print(credentials.token)
"
```

**Option B: Set up token refresh (Recommended for production)**
For production, implement automatic token refresh. The token expires every hour.

#### Step 4: Configure Backend

**Via Environment Variables:**
```bash
export FCM_PROJECT_ID="your-project-id"
export FCM_ACCESS_TOKEN="your-access-token"
```

**Via Database (for runtime configuration):**
```sql
UPDATE server_config 
SET 
    fcm_project_id = 'your-project-id',
    fcm_access_token = 'your-access-token'
WHERE id = 'default';
```

### 2. Apple Push Notification Service (iOS)

#### Step 1: Create APNS Key
1. Go to [Apple Developer Portal](https://developer.apple.com/)
2. Go to Certificates, Identifiers & Profiles → Keys
3. Create a new key with "Apple Push Notifications service (APNs)" enabled
4. Download the `.p8` key file and note the Key ID

#### Step 2: Get Required Information
- **Team ID**: From Apple Developer Portal → Membership
- **Bundle ID**: Your app's bundle identifier (e.g., `com.mattermost.rnbeta`)
- **Key ID**: From the key you just created

#### Step 3: Configure Backend

**Via Environment Variables:**
```bash
export APNS_KEY_ID="your-key-id"
export APNS_TEAM_ID="your-team-id"
export APNS_BUNDLE_ID="com.mattermost.rnbeta"
export APNS_PRIVATE_KEY="-----BEGIN EC PRIVATE KEY-----\n...\n-----END EC PRIVATE KEY-----"
```

**Via Database:**
```sql
UPDATE server_config 
SET 
    apns_key_id = 'your-key-id',
    apns_team_id = 'your-team-id',
    apns_bundle_id = 'com.mattermost.rnbeta',
    apns_private_key = '-----BEGIN EC PRIVATE KEY-----
...
-----END EC PRIVATE KEY-----'
WHERE id = 'default';
```

## Mobile App Configuration

### Mattermost-Mobile Changes Required

The mobile app needs to register the device token with the backend. This is typically done during login or when the app starts.

**API Endpoint:**
```
POST /api/v4/users/me/device
Content-Type: application/json

{
    "device_id": "unique-device-id",
    "token": "fcm-or-apns-device-token",
    "platform": "ios" | "android"
}
```

The mobile app should call this endpoint:
1. When the user logs in
2. When the device token changes
3. Periodically to keep the registration active

### Token Management

**Android (FCM):**
```kotlin
FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
    if (task.isSuccessful) {
        val token = task.result
        // Send token to rustchat backend
        registerDevice(token, "android")
    }
}
```

**iOS (APNS):**
```swift
// In AppDelegate.swift
func application(_ application: UIApplication, didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data) {
    let token = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
    // Send token to rustchat backend
    registerDevice(token: token, platform: "ios")
}
```

## Notification Types

### 1. Call Ringing Notifications

**Triggered when:**
- A call starts in a DM/GM channel (auto-ringing)
- Someone clicks the "Ring" button

**Notification content:**
- **Title**: "Incoming call from [Caller Name]"
- **Body**: "Tap to answer"
- **Priority**: High (wakes device, shows as heads-up notification)
- **Sound**: Default notification sound
- **Data payload**: Includes call_id, channel_id, caller_name

**Android channel**: `calls` (high priority)

### 2. Message Notifications

**Triggered when:**
- Direct message received
- Mentioned in a message (@username)

**Notification content:**
- **Title**: Sender name (for DMs) or "Sender in Channel"
- **Body**: Message preview (first 100 characters)
- **Priority**: Normal
- **Sound**: Default notification sound
- **Data payload**: Includes channel_id, sender_name

**Android channel**: `messages` (normal priority)

## Testing

### 1. Verify Device Registration

Check that devices are registered:
```sql
SELECT user_id, device_id, platform, last_seen_at 
FROM user_devices 
WHERE user_id = 'your-user-uuid';
```

### 2. Test Push Notification

**Using cURL:**
```bash
curl -X POST https://fcm.googleapis.com/v1/projects/YOUR_PROJECT_ID/messages:send \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "message": {
      "token": "DEVICE_TOKEN",
      "notification": {
        "title": "Test",
        "body": "Hello from rustchat!"
      }
    }
  }'
```

### 3. Check Backend Logs

Enable debug logging to see push notification activity:
```bash
RUST_LOG=debug ./rustchat
```

Look for log messages:
- `Sending push notification for call`
- `Sent push notification for incoming call`
- `Sending push notification for message`
- `Sent push notification for message`

## Troubleshooting

### Issue: Push notifications not received

**Checklist:**
1. ✅ FCM/APNS configured in backend (environment variables or database)
2. ✅ Device registered with backend (check `user_devices` table)
3. ✅ Mobile app has notification permission granted
4. ✅ Device token is valid and not expired
5. ✅ Backend can reach FCM/APNS servers (no firewall blocking)

**Debug steps:**
```bash
# Check backend logs
tail -f /var/log/rustchat/app.log | grep -i "push\|notification"

# Verify device registration
psql -d rustchat -c "SELECT * FROM user_devices WHERE user_id = '...';"

# Test FCM token validity
curl -H "Authorization: Bearer YOUR_TOKEN" \
  https://fcm.googleapis.com/v1/projects/YOUR_PROJECT_ID/messages:send \
  -d '{}'
```

### Issue: Call ringing not working

**Checklist:**
1. ✅ Call started in DM/GM channel (auto-ringing) or ring button clicked
2. ✅ Other user has registered device
3. ✅ Other user hasn't dismissed the notification
4. ✅ Push notification service is configured

**Debug steps:**
```bash
# Check if ringing event was sent
tail -f /var/log/rustchat/app.log | grep "broadcast_ringing_event"

# Check dismissed notifications
# Look for calls_user_dismissed_notification events in logs
```

### Issue: Message notifications not working

**Checklist:**
1. ✅ Message is a DM or contains @mention
2. ✅ Recipient has registered device
3. ✅ Push notification service is configured

**Debug steps:**
```bash
# Check if mentions are parsed
tail -f /var/log/rustchat/app.log | grep "mention"

# Verify message creation flow
tail -f /var/log/rustchat/app.log | grep "create_post"
```

## Security Considerations

1. **Token Storage**: Store FCM access tokens and APNS keys securely
   - Use environment variables or secret management
   - Never commit tokens to version control
   - Rotate tokens periodically

2. **HTTPS Only**: Always use HTTPS for device token registration

3. **Token Expiry**: FCM access tokens expire after 1 hour
   - Implement automatic token refresh for production
   - The backend currently requires manual token updates

4. **Device Token Validation**: Validate device tokens before storing

## Performance Considerations

1. **Async Processing**: Push notifications are sent asynchronously
   - Won't block WebSocket event broadcasting
   - Failed notifications don't affect message delivery

2. **Batch Processing**: For high-volume scenarios, consider batching push notifications

3. **Rate Limiting**: FCM and APNS have rate limits
   - FCM: 500 messages/second per project
   - APNS: No explicit limit, but be reasonable

## Future Improvements

1. **Automatic Token Refresh**: Implement OAuth2 token refresh for FCM
2. **APNS Support**: Full APNS HTTP/2 implementation (currently using FCM as proxy)
3. **Notification Preferences**: Per-user notification settings
4. **Rich Notifications**: Images, actions, and custom UI
5. **Notification History**: Store notification delivery status
6. **Badge Counts**: Update app badge counts for unread messages

## References

- [FCM HTTP v1 API Documentation](https://firebase.google.com/docs/cloud-messaging/send-message)
- [APNS Documentation](https://developer.apple.com/documentation/usernotifications)
- [Mattermost Mobile Documentation](https://developers.mattermost.com/contribute/more-info/mobile/)
