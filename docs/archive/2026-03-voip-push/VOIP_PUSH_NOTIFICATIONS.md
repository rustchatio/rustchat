# VoIP Push Notifications for Native Call Ringing

This document describes the complete implementation of VoIP push notifications for RustChat, enabling native call ringing on iOS and Android when the app is in the background.

## Overview

The VoIP push notification system consists of three main components:

1. **Push Proxy** - Relays notifications to FCM (Android) and APNS (iOS)
2. **RustChat Backend** - Generates and sends push notifications for calls
3. **Mobile Client** - Receives and handles push notifications to show native call UI

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  RustChat       │────▶│  Push Proxy      │────▶│  FCM (Android)  │
│  Backend        │     │  (rustchat-push) │     │                 │
│                 │     │                  │────▶│  APNS (iOS)     │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                                                          │
                              ┌───────────────────────────┘
                              ▼
                    ┌─────────────────┐
                    │  Mobile Client  │
                    │  - CallKit (iOS)│
                    │  - ConnectionService
                    │    (Android)    │
                    └─────────────────┘
```

## 1. Push Proxy Configuration

### Docker Compose

Add the push proxy service to your `docker-compose.yml`:

```yaml
services:
  push-proxy:
    build:
      context: ./push-proxy
      dockerfile: Dockerfile
    environment:
      # Firebase (Android)
      - FIREBASE_PROJECT_ID=your-project-id
      - GOOGLE_APPLICATION_CREDENTIALS=/secrets/firebase-key.json
      
      # APNS (iOS VoIP)
      - APNS_CERT_PATH=/secrets/voip-cert.pem
      - APNS_KEY_PATH=/secrets/voip-key.pem
      - APNS_BUNDLE_ID=com.rustchat.app
      - APNS_USE_PRODUCTION=false  # Set to true for production
      
      # General
      - RUSTCHAT_PUSH_PORT=3000
      - RUST_LOG=push_proxy=info
    volumes:
      - ./secrets/firebase-key.json:/secrets/firebase-key.json:ro
      - ./secrets/voip-cert.pem:/secrets/voip-cert.pem:ro
      - ./secrets/voip-key.pem:/secrets/voip-key.pem:ro
    networks:
      - rustchat-internal
    restart: unless-stopped

  rustchat-server:
    # ... existing configuration
    environment:
      - RUSTCHAT_PUSH_PROXY_URL=http://push-proxy:3000
```

### Firebase Setup (Android)

1. Go to [Firebase Console](https://console.firebase.google.com/)
2. Create a new project or select existing
3. Go to Project Settings > Service Accounts
4. Click "Generate new private key"
5. Save the JSON file and mount it to the push proxy container

### APNS Setup (iOS VoIP)

1. Go to [Apple Developer Portal](https://developer.apple.com/)
2. Navigate to Certificates, Identifiers & Profiles
3. Create a new VoIP Services Certificate:
   - Select your App ID
   - Download the certificate
   - Export as .pem format

```bash
# Convert .p12 to .pem (if needed)
openssl pkcs12 -in voip-cert.p12 -out voip-cert.pem -nodes -clcerts
```

## 2. Backend Configuration

### Environment Variables

Add to your RustChat backend environment:

```bash
# Push Proxy URL (required)
RUSTCHAT_PUSH_PROXY_URL=https://push.rustchat.com

# OR configure direct FCM (fallback)
FCM_PROJECT_ID=your-project-id
FCM_ACCESS_TOKEN=your-access-token
```

### Server Configuration

Ensure push notifications are enabled in your server config:

```json
{
  "site": {
    "send_push_notifications": true
  }
}
```

## 3. Mobile Client Implementation

### iOS Implementation

#### Required Capabilities

Add to your app's entitlements:

```xml
<key>com.apple.developer.pushkit.unified-voip</key>
<true/>
<key>com.apple.developer.networking.voip</key>
<true/>
```

#### AppDelegate.m / AppDelegate.swift

**Objective-C:**

```objc
#import <PushKit/PushKit.h>
#import <CallKit/CallKit.h>

@interface AppDelegate () <PKPushRegistryDelegate, CXProviderDelegate>
@property (nonatomic, strong) CXProvider *callProvider;
@property (nonatomic, strong) CXCallController *callController;
@end

@implementation AppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    // Register for VoIP pushes
    PKPushRegistry *pushRegistry = [[PKPushRegistry alloc] initWithQueue:dispatch_get_main_queue()];
    pushRegistry.delegate = self;
    pushRegistry.desiredPushTypes = [NSSet setWithObject:PKPushTypeVoIP];
    
    // Configure CallKit provider
    CXProviderConfiguration *config = [[CXProviderConfiguration alloc] initWithLocalizedName:@"RustChat"];
    config.supportsVideo = YES;
    config.supportedHandleTypes = [NSSet setWithObject:@(CXHandleTypeGeneric)];
    config.iconTemplateImageData = UIImagePNGRepresentation([UIImage imageNamed:@"AppIcon"])
    
    self.callProvider = [[CXProvider alloc] initWithConfiguration:config];
    [self.callProvider setDelegate:self queue:nil];
    self.callController = [[CXCallController alloc] init];
    
    return YES;
}

#pragma mark - PKPushRegistryDelegate

- (void)pushRegistry:(PKPushRegistry *)registry didUpdatePushCredentials:(PKPushCredentials *)credentials forType:(PKPushType)type {
    // Send device token to RustChat server
    NSString *deviceToken = [credentials.token.description stringByTrimmingCharactersInSet:[NSCharacterSet characterSetWithCharactersInString:@"<>"]];
    deviceToken = [deviceToken stringByReplacingOccurrencesOfString:@" " withString:@""];
    
    // Register token with RustChat server
    [self registerDeviceToken:deviceToken];
}

- (void)pushRegistry:(PKPushRegistry *)registry didReceiveIncomingPushWithPayload:(PKPushPayload *)payload forType:(PKPushType)type withCompletionHandler:(void (^)(void))completion {
    NSDictionary *data = payload.dictionaryPayload;
    
    // Extract call information
    NSString *callUUID = data[@"data"][@"call_uuid"];
    NSString *callerName = data[@"data"][@"caller_name"];
    NSString *channelId = data[@"data"][@"channel_id"];
    NSString *serverUrl = data[@"data"][@"server_url"];
    
    // Report incoming call to CallKit
    NSUUID *uuid = [[NSUUID alloc] initWithUUIDString:callUUID];
    CXHandle *handle = [[CXHandle alloc] initWithType:CXHandleTypeGeneric value:channelId];
    CXCallUpdate *update = [[CXCallUpdate alloc] init];
    update.remoteHandle = handle;
    update.localizedCallerName = callerName;
    update.supportsDTMF = NO;
    update.supportsHolding = NO;
    update.supportsGrouping = NO;
    update.supportsUngrouping = NO;
    update.hasVideo = NO;
    
    [self.callProvider reportNewIncomingCallWithUUID:uuid update:update completion:^(NSError *error) {
        if (error) {
            NSLog(@"Failed to report incoming call: %@", error);
        } else {
            // Store call info for later use
            [self storeIncomingCallInfo:uuid channelId:channelId serverUrl:serverUrl];
        }
        completion();
    }];
}

#pragma mark - CXProviderDelegate

- (void)provider:(CXProvider *)provider performAnswerCallAction:(CXAnswerCallAction *)action {
    // User answered the call
    NSUUID *callUUID = action.callUUID;
    
    // Connect to the call
    [self connectToCall:callUUID];
    
    [action fulfill];
}

- (void)provider:(CXProvider *)provider performEndCallAction:(CXEndCallAction *)action {
    // User ended/rejected the call
    NSUUID *callUUID = action.callUUID;
    
    // Disconnect from call
    [self disconnectFromCall:callUUID];
    
    [action fulfill];
}

@end
```

**Swift:**

```swift
import PushKit
import CallKit

@main
class AppDelegate: UIResponder, UIApplicationDelegate, PKPushRegistryDelegate, CXProviderDelegate {
    var window: UIWindow?
    var callProvider: CXProvider?
    var callController: CXCallController?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Register for VoIP pushes
        let pushRegistry = PKPushRegistry(queue: DispatchQueue.main)
        pushRegistry.delegate = self
        pushRegistry.desiredPushTypes = [.voIP]
        
        // Configure CallKit
        let config = CXProviderConfiguration(localizedName: "RustChat")
        config.supportsVideo = true
        config.supportedHandleTypes = [.generic]
        
        callProvider = CXProvider(configuration: config)
        callProvider?.setDelegate(self, queue: nil)
        callController = CXCallController()
        
        return true
    }
    
    // MARK: - PKPushRegistryDelegate
    
    func pushRegistry(_ registry: PKPushRegistry, didUpdate credentials: PKPushCredentials, for type: PKPushType) {
        let deviceToken = credentials.token.map { String(format: "%02x", $0) }.joined()
        registerDeviceToken(deviceToken)
    }
    
    func pushRegistry(_ registry: PKPushRegistry, didReceiveIncomingPushWith payload: PKPushPayload, for type: PKPushType, completion: @escaping () -> Void) {
        guard let data = payload.dictionaryPayload["data"] as? [String: Any],
              let callUUID = data["call_uuid"] as? String,
              let callerName = data["caller_name"] as? String,
              let channelId = data["channel_id"] as? String else {
            completion()
            return
        }
        
        let uuid = UUID(uuidString: callUUID)!
        let handle = CXHandle(type: .generic, value: channelId)
        let update = CXCallUpdate()
        update.remoteHandle = handle
        update.localizedCallerName = callerName
        
        callProvider?.reportNewIncomingCall(with: uuid, update: update) { error in
            if let error = error {
                print("Failed to report call: \(error)")
            }
            completion()
        }
    }
    
    // MARK: - CXProviderDelegate
    
    func provider(_ provider: CXProvider, perform action: CXAnswerCallAction) {
        connectToCall(action.callUUID)
        action.fulfill()
    }
    
    func provider(_ provider: CXProvider, perform action: CXEndCallAction) {
        disconnectFromCall(action.callUUID)
        action.fulfill()
    }
}
```

### Android Implementation

#### AndroidManifest.xml

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.rustchat.app">

    <!-- Permissions -->
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
    <uses-permission android:name="android.permission.RECORD_AUDIO" />
    <uses-permission android:name="android.permission.CAMERA" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_MICROPHONE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_CAMERA" />
    <uses-permission android:name="android.permission.USE_FULL_SCREEN_INTENT" />
    <uses-permission android:name="android.permission.WAKE_LOCK" />
    
    <!-- Telecom permissions for ConnectionService -->
    <uses-permission android:name="android.permission.MANAGE_OWN_CALLS" />
    <uses-permission android:name="android.permission.CALL_PHONE" />

    <application
        android:name=".MainApplication"
        android:label="@string/app_name"
        android:icon="@mipmap/ic_launcher">

        <!-- Main Activity -->
        <activity
            android:name=".MainActivity"
            android:launchMode="singleTop"
            android:showOnLockScreen="true"
            android:turnScreenOn="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>

        <!-- FCM Service -->
        <service
            android:name=".CallMessagingService"
            android:exported="false">
            <intent-filter>
                <action android:name="com.google.firebase.MESSAGING_EVENT" />
            </intent-filter>
        </service>

        <!-- ConnectionService for native call integration -->
        <service
            android:name=".CallConnectionService"
            android:permission="android.permission.BIND_TELECOM_CONNECTION_SERVICE"
            android:exported="true">
            <intent-filter>
                <action android:name="android.telecom.ConnectionService" />
            </intent-filter>
        </service>

    </application>
</manifest>
```

#### CallMessagingService.java (FCM Handler)

```java
package com.rustchat.app;

import com.google.firebase.messaging.FirebaseMessagingService;
import com.google.firebase.messaging.RemoteMessage;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Intent;
import android.content.Context;
import android.os.Build;
import android.media.AudioAttributes;
import android.media.RingtoneManager;
import android.net.Uri;
import androidx.core.app.NotificationCompat;
import androidx.core.app.NotificationManagerCompat;
import android.telecom.TelecomManager;

public class CallMessagingService extends FirebaseMessagingService {
    
    private static final String CHANNEL_ID_CALLS = "channel_calls";
    private static final int NOTIFICATION_ID_CALL = 1001;

    @Override
    public void onMessageReceived(RemoteMessage remoteMessage) {
        super.onMessageReceived(remoteMessage);
        
        // Check if this is a call notification
        Map<String, String> data = remoteMessage.getData();
        if ("call".equals(data.get("type"))) {
            handleIncomingCall(data);
        }
    }

    @Override
    public void onNewToken(String token) {
        super.onNewToken(token);
        // Send token to RustChat server
        sendTokenToServer(token);
    }

    private void handleIncomingCall(Map<String, String> data) {
        String callUuid = data.get("call_uuid");
        String callerName = data.get("sender_name");
        String channelId = data.get("channel_id");
        String serverUrl = data.get("server_url");
        
        // Create notification channel for calls
        createCallNotificationChannel();
        
        // Build full-screen intent for incoming call
        Intent fullScreenIntent = new Intent(this, IncomingCallActivity.class);
        fullScreenIntent.putExtra("call_uuid", callUuid);
        fullScreenIntent.putExtra("caller_name", callerName);
        fullScreenIntent.putExtra("channel_id", channelId);
        fullScreenIntent.putExtra("server_url", serverUrl);
        fullScreenIntent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
        
        PendingIntent fullScreenPendingIntent = PendingIntent.getActivity(
            this, 0, fullScreenIntent,
            PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE
        );

        // Build notification
        NotificationCompat.Builder builder = new NotificationCompat.Builder(this, CHANNEL_ID_CALLS)
            .setSmallIcon(R.drawable.ic_call)
            .setContentTitle("Incoming Call")
            .setContentText(callerName + " is calling")
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(NotificationCompat.CATEGORY_CALL)
            .setFullScreenIntent(fullScreenPendingIntent, true)
            .setAutoCancel(false)
            .setOngoing(true)
            .setSound(RingtoneManager.getDefaultUri(RingtoneManager.TYPE_RINGTONE))
            .setVibrate(new long[]{0, 1000, 1000, 1000, 1000})
            .addAction(R.drawable.ic_call_decline, "Decline", 
                createActionIntent(callUuid, "DECLINE"))
            .addAction(R.drawable.ic_call_answer, "Answer", 
                createActionIntent(callUuid, "ANSWER"));

        // Show notification
        NotificationManagerCompat notificationManager = NotificationManagerCompat.from(this);
        notificationManager.notify(NOTIFICATION_ID_CALL, builder.build());
        
        // Also trigger ConnectionService for native call integration
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            TelecomManager telecomManager = (TelecomManager) getSystemService(Context.TELECOM_SERVICE);
            // Register incoming call with system
        }
    }

    private void createCallNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Uri ringtoneUri = RingtoneManager.getDefaultUri(RingtoneManager.TYPE_RINGTONE);
            
            AudioAttributes audioAttributes = new AudioAttributes.Builder()
                .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
                .setUsage(AudioAttributes.USAGE_NOTIFICATION_RINGTONE)
                .build();

            NotificationChannel channel = new NotificationChannel(
                CHANNEL_ID_CALLS,
                "Incoming Calls",
                NotificationManager.IMPORTANCE_HIGH
            );
            channel.setDescription("Incoming voice and video calls");
            channel.setSound(ringtoneUri, audioAttributes);
            channel.enableVibration(true);
            channel.setVibrationPattern(new long[]{0, 1000, 1000, 1000, 1000});
            channel.setLockscreenVisibility(NotificationCompat.VISIBILITY_PUBLIC);

            NotificationManager notificationManager = getSystemService(NotificationManager.class);
            notificationManager.createNotificationChannel(channel);
        }
    }

    private PendingIntent createActionIntent(String callUuid, String action) {
        Intent intent = new Intent(this, CallActionReceiver.class);
        intent.putExtra("call_uuid", callUuid);
        intent.putExtra("action", action);
        return PendingIntent.getBroadcast(this, callUuid.hashCode(), intent,
            PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);
    }
}
```

#### IncomingCallActivity.java (Full Screen Call UI)

```java
package com.rustchat.app;

import android.app.Activity;
import android.os.Bundle;
import android.view.WindowManager;
import android.widget.Button;
import android.widget.TextView;
import com.facebook.react.ReactActivity;
import com.wix.reactnativecallkeep.CallKeepModule;

public class IncomingCallActivity extends ReactActivity {
    
    private String callUuid;
    private String channelId;
    private String serverUrl;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Show on lock screen
        getWindow().addFlags(
            WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON |
            WindowManager.LayoutParams.FLAG_DISMISS_KEYGUARD |
            WindowManager.LayoutParams.FLAG_SHOW_WHEN_LOCKED |
            WindowManager.LayoutParams.FLAG_TURN_SCREEN_ON
        );
        
        setContentView(R.layout.activity_incoming_call);
        
        // Get call info from intent
        callUuid = getIntent().getStringExtra("call_uuid");
        String callerName = getIntent().getStringExtra("caller_name");
        channelId = getIntent().getStringExtra("channel_id");
        serverUrl = getIntent().getStringExtra("server_url");
        
        // Set caller name
        TextView callerNameText = findViewById(R.id.caller_name);
        callerNameText.setText(callerName);
        
        // Answer button
        Button answerButton = findViewById(R.id.answer_button);
        answerButton.setOnClickListener(v -> answerCall());
        
        // Decline button
        Button declineButton = findViewById(R.id.decline_button);
        declineButton.setOnClickListener(v -> declineCall());
    }
    
    private void answerCall() {
        // Start ringtone
        InCallManagerModule.startRingtone("_DEFAULT_");
        
        // Answer via CallKeep
        CallKeepModule.answerCall(callUuid);
        
        // Launch main app with call info
        Intent intent = new Intent(this, MainActivity.class);
        intent.putExtra("action", "join_call");
        intent.putExtra("channel_id", channelId);
        intent.putExtra("server_url", serverUrl);
        intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TOP);
        startActivity(intent);
        
        finish();
    }
    
    private void declineCall() {
        // Decline via CallKeep
        CallKeepModule.endCall(callUuid);
        
        // Notify server
        dismissNotification();
        
        finish();
    }
}
```

## 4. Testing

### Backend Tests

```bash
# Test push notification configuration
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "test-device-token",
    "title": "Test Call",
    "body": "Incoming call from Test User",
    "platform": "ios",
    "type": "call",
    "data": {
      "channel_id": "test-channel",
      "post_id": "test-post",
      "type": "call",
      "sub_type": "calls",
      "sender_name": "Test User",
      "server_url": "https://rustchat.com"
    }
  }'
```

### Mobile Testing

1. **iOS:**
   - Test with device (simulator doesn't support push)
   - Use Apple Push Notification Console
   - Verify CallKit integration

2. **Android:**
   - Test FCM with Firebase Console
   - Verify full-screen intent on locked screen
   - Test with Doze mode

## 5. Troubleshooting

### iOS Issues

| Issue | Solution |
|-------|----------|
| `kPushNotificationError` | Verify VoIP certificate is valid |
| Call not showing on lock screen | Check `showOnLockScreen` in manifest |
| No ringtone | Ensure `aps.sound` is set in payload |

### Android Issues

| Issue | Solution |
|-------|----------|
| Notification not showing | Check notification channel importance |
| No sound in Doze mode | Use `setPriority(PRIORITY_HIGH)` |
| Full-screen intent not working | Add `USE_FULL_SCREEN_INTENT` permission |

## 6. Security Considerations

1. **Certificate Management:**
   - Store certificates in secure location
   - Use read-only mounts in Docker
   - Rotate certificates annually

2. **Token Validation:**
   - Validate device tokens before sending
   - Remove invalid tokens from database

3. **APNS/FCM Best Practices:**
   - Use production servers for production apps
   - Implement token refresh handling
   - Monitor for rejected tokens

## References

- [Apple VoIP Push Notifications](https://developer.apple.com/documentation/pushkit)
- [CallKit Framework](https://developer.apple.com/documentation/callkit)
- [FCM Documentation](https://firebase.google.com/docs/cloud-messaging)
- [Android Telecom Framework](https://developer.android.com/reference/android/telecom/package-summary)
