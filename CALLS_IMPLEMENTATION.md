# Call Functionality Implementation for RustChat Web Client

## Summary

This implementation adds Mattermost-compatible call functionality to the RustChat web client. The web client now uses the same API endpoints as the mobile client (`/plugins/com.mattermost.calls/...`) for native WebRTC-based voice calls.

## Changes Made

### 1. Updated Calls API (`frontend/src/api/calls.ts`)
- Changed from old `/calls` endpoints to Mattermost plugin endpoints
- Base route: `/api/v4/plugins/com.mattermost.calls`
- Added all Mattermost-compatible methods:
  - `startCall(channelId)` - POST `/calls/{channel_id}/start`
  - `joinCall(channelId)` - POST `/calls/{channel_id}/join`
  - `leaveCall(channelId)` - POST `/calls/{channel_id}/leave`
  - `endCall(channelId)` - POST `/calls/{channel_id}/end`
  - `mute/unmute(channelId)` - POST `/calls/{channel_id}/mute`
  - `raiseHand/lowerHand(channelId)` - POST `/calls/{channel_id}/raise-hand`
  - `sendOffer/ICE` for WebRTC signaling
  - Host controls (make host, mute others, etc.)

### 2. Updated Calls Store (`frontend/src/stores/calls.ts`)
- Added Mattermost-compatible types (`CallState`, `CallSession`, `CallsConfig`)
- Implemented WebRTC initialization with ICE servers from server config
- Added WebSocket event listeners for call events:
  - `custom_com.mattermost.calls_call_start`
  - `custom_com.mattermost.calls_call_end`
  - `custom_com.mattermost.calls_user_joined`
  - `custom_com.mattermost.calls_user_left`
  - `custom_com.mattermost.calls_user_muted/unmuted`
  - `custom_com.mattermost.calls_raise_hand/lower_hand`
  - `custom_com.mattermost.calls_screen_on/off`
- Implemented call actions: `startCall`, `joinCall`, `leaveCall`, `endCall`
- Added mute/unmute, hand raise, and screen share controls

### 3. Updated ActiveCall Component (`frontend/src/components/calls/ActiveCall.vue`)
- Complete redesign for WebRTC audio calls
- Shows participants list with mute/hand status
- Call controls: mute/unmute, raise hand, screen share, hangup
- Expanded and compact modes
- Host controls (end call for everyone)
- Call duration timer

### 4. Updated MessageComposer (`frontend/src/components/composer/MessageComposer.vue`)
- Added slash command handling for `/call` commands:
  - `/call` or `/call start` - Start a new call
  - `/call join` - Join existing call
  - `/call leave` - Leave current call
  - `/call end` - End the call (host only)
- Added phone button to start audio calls
- Shows active call indicator when in a call

### 5. Updated ChannelHeader (`frontend/src/components/channel/ChannelHeader.vue`)
- Added native call button with dynamic states:
  - Shows "join" button with pulse animation when call is active
  - Shows "in call" button when user is in the channel's call
  - Shows "start call" button when no call is active
- Separate button for MiroTalk video calls (if enabled)

### 6. Updated ChannelView (`frontend/src/views/main/ChannelView.vue`)
- Integrated `ActiveCall` component for floating call widget
- Integrated `IncomingCallModal` for incoming call notifications
- Added `onStartAudioCall` handler
- Loads active calls and config on mount

## API Endpoints Used

The implementation uses these Mattermost-compatible endpoints:

```
GET    /api/v4/plugins/com.mattermost.calls/version
GET    /api/v4/plugins/com.mattermost.calls/config
GET    /api/v4/plugins/com.mattermost.calls/channels?mobilev2=true
GET    /api/v4/plugins/com.mattermost.calls/{channel_id}?mobilev2=true
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/start
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/join
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/leave
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/end
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/mute
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/unmute
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/raise-hand
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/lower-hand
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/react
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/screen-share
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/offer
POST   /api/v4/plugins/com.mattermost.calls/calls/{channel_id}/ice
```

## WebSocket Events

The client listens for these call-related WebSocket events:

```
custom_com.mattermost.calls_call_start
custom_com.mattermost.calls_call_end
custom_com.mattermost.calls_user_joined
custom_com.mattermost.calls_user_left
custom_com.mattermost.calls_user_muted
custom_com.mattermost.calls_user_unmuted
custom_com.mattermost.calls_raise_hand
custom_com.mattermost.calls_lower_hand
custom_com.mattermost.calls_screen_on
custom_com.mattermost.calls_screen_off
```

## Usage

### Starting a Call
1. Click the phone icon in the channel header
2. Or type `/call` or `/call start` in the message composer
3. Or click the phone button in the composer toolbar

### Joining a Call
1. Click the pulsing phone icon when a call is active in the channel
2. Or type `/call join` in the message composer

### Leaving/Ending a Call
- Click the red hangup button in the call widget
- Or type `/call leave` to leave
- Or type `/call end` to end for everyone (host only)

### In-Call Controls
- **Mute/Unmute**: Click the microphone button
- **Raise Hand**: Click the hand button
- **Screen Share**: Click the monitor button (if enabled)
- **Expand/Minimize**: Click the maximize/minimize button
- **View Participants**: Click the users button (expanded mode)

## Compatibility

This implementation is compatible with:
- Mattermost mobile client call actions
- Server-side SFU implementation in RustChat
- MiroTalk video calls (optional, configured separately)

## Future Enhancements

- Video support (currently audio-only)
- Screen sharing UI
- Recording controls
- Live captions
- Transcriptions
