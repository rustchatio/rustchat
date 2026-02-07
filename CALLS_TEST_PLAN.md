# Mattermost Mobile Calls - Test Plan & Validation Checklist

## Overview

This document provides the test plan and validation checklist for verifying that RustChat's Mattermost Calls implementation works correctly with Mattermost Mobile clients.

## API Endpoints Implemented

### Plugin Information Endpoints
- ✅ `GET /plugins/com.mattermost.calls/version` - Returns plugin version and rtcd status
- ✅ `GET /plugins/com.mattermost.calls/config` - Returns ICE servers with TURN credentials

### Call Management Endpoints
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/start` - Start a new call
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/join` - Join an existing call
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/leave` - Leave a call
- ✅ `GET /plugins/com.mattermost.calls/calls/{channel_id}` - Get call state

### Call Feature Endpoints
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/react` - Send emoji reaction
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/screen-share` - Toggle screen sharing
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/mute` - Mute self
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/unmute` - Unmute self
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/raise-hand` - Raise hand
- ✅ `POST /plugins/com.mattermost.calls/calls/{channel_id}/lower-hand` - Lower hand

### WebSocket Events Implemented
- ✅ `custom_com.mattermost.calls_call_start` - Call started
- ✅ `custom_com.mattermost.calls_call_end` - Call ended
- ✅ `custom_com.mattermost.calls_user_joined` - User joined call
- ✅ `custom_com.mattermost.calls_user_left` - User left call
- ✅ `custom_com.mattermost.calls_user_muted` - User muted
- ✅ `custom_com.mattermost.calls_user_unmuted` - User unmuted
- ✅ `custom_com.mattermost.calls_screen_on` - Screen sharing started
- ✅ `custom_com.mattermost.calls_screen_off` - Screen sharing stopped
- ✅ `custom_com.mattermost.calls_raise_hand` - Hand raised
- ✅ `custom_com.mattermost.calls_lower_hand` - Hand lowered
- ✅ `custom_com.mattermost.calls_user_reacted` - Emoji reaction sent

## Contract Tests

### Test 1: Version Endpoint
```bash
curl -X GET http://localhost:8080/api/v4/plugins/com.mattermost.calls/version \
  -H "Authorization: Bearer <token>"
```

**Expected Response:**
```json
{
  "version": "0.28.0",
  "rtcd": false
}
```

### Test 2: Config Endpoint (with TURN)
```bash
curl -X GET http://localhost:8080/api/v4/plugins/com.mattermost.calls/config \
  -H "Authorization: Bearer <token>"
```

**Expected Response:**
```json
{
  "iceServers": [
    {
      "urls": ["stun:stun.l.google.com:19302"]
    },
    {
      "urls": ["turn:your.turn.server:3478"],
      "username": "{expiration}:{user_id}",
      "credential": "{base64_hmac}"
    }
  ]
}
```

### Test 3: Start Call
```bash
curl -X POST http://localhost:8080/api/v4/plugins/com.mattermost.calls/calls/{channel_id}/start \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json"
```

**Expected Response:**
```json
{
  "id": "call-uuid",
  "channel_id": "channel-uuid",
  "start_at": 1704067200000,
  "owner_id": "user-uuid"
}
```

**Expected WebSocket Event (broadcast to channel):**
```json
{
  "type": "event",
  "event": "custom_com.mattermost.calls_call_start",
  "data": {
    "channel_id": "channel-uuid",
    "user_id": "user-uuid",
    "call_id": "call-uuid",
    "start_at": "1704067200000",
    "owner_id": "user-uuid"
  }
}
```

### Test 4: Join Call
```bash
curl -X POST http://localhost:8080/api/v4/plugins/com.mattermost.calls/calls/{channel_id}/join \
  -H "Authorization: Bearer <token>"
```

**Expected Response:**
```json
{
  "status": "OK"
}
```

**Expected WebSocket Event:**
```json
{
  "type": "event",
  "event": "custom_com.mattermost.calls_user_joined",
  "data": {
    "channel_id": "channel-uuid",
    "user_id": "user-uuid",
    "session_id": "session-uuid",
    "muted": true,
    "raised_hand": false
  }
}
```

### Test 5: Get Call State
```bash
curl -X GET http://localhost:8080/api/v4/plugins/com.mattermost.calls/calls/{channel_id} \
  -H "Authorization: Bearer <token>"
```

**Expected Response:**
```json
{
  "id": "call-uuid",
  "channel_id": "channel-uuid",
  "start_at": 1704067200000,
  "owner_id": "user-uuid",
  "participants": ["user1-uuid", "user2-uuid"],
  "screen_sharing_id": null,
  "thread_id": null
}
```

## Mobile QA Checklist

### Pre-Flight Checks
- [ ] Server is running with Calls enabled (`RUSTCHAT_CALLS_ENABLED=true`)
- [ ] TURN server is configured and accessible
- [ ] RTC ports (UDP 8443) are open in firewall
- [ ] SSL certificate is valid (required for WebRTC)

### iOS Testing

#### Basic Functionality
- [ ] Call button appears in channel header
- [ ] Can start a call
- [ ] Call shows as active in channel
- [ ] Second user can join call
- [ ] Both users can hear each other (WiFi to WiFi)
- [ ] Both users can hear each other (LTE to WiFi)
- [ ] Mute/unmute works
- [ ] Leave call works
- [ ] Call ends when last participant leaves

#### Advanced Features
- [ ] Screen sharing works (iOS 15+)
- [ ] Hand raise/lower works
- [ ] Emoji reactions work
- [ ] Participant list shows correctly
- [ ] Speaking indicators work

#### Network Conditions
- [ ] Call works on WiFi
- [ ] Call works on LTE/5G
- [ ] Call works behind corporate firewall (TURN relay)
- [ ] Call reconnects after network switch (WiFi → LTE)
- [ ] Call survives brief network interruption

### Android Testing

#### Basic Functionality
- [ ] Call button appears in channel header
- [ ] Can start a call
- [ ] Call shows as active in channel
- [ ] Second user can join call
- [ ] Both users can hear each other (WiFi to WiFi)
- [ ] Both users can hear each other (LTE to WiFi)
- [ ] Mute/unmute works
- [ ] Leave call works
- [ ] Call ends when last participant leaves

#### Advanced Features
- [ ] Screen sharing works (Android 10+)
- [ ] Hand raise/lower works
- [ ] Emoji reactions work
- [ ] Participant list shows correctly
- [ ] Speaking indicators work

#### Network Conditions
- [ ] Call works on WiFi
- [ ] Call works on LTE/5G
- [ ] Call works behind corporate firewall (TURN relay)
- [ ] Call reconnects after network switch (WiFi → LTE)
- [ ] Call survives brief network interruption

### Cross-Platform Testing
- [ ] iOS can call Android
- [ ] Android can call iOS
- [ ] Desktop client can join mobile call
- [ ] Mobile client can join desktop call

## Debugging Guide

### Mobile Shows "Connecting..." Forever

**Check server logs:**
```bash
curl -X GET http://localhost:8080/api/v4/plugins/com.mattermost.calls/version
```
- Should return 200 with version info
- If 404, plugin routes not registered
- If 401, auth token invalid

**Check ICE config:**
```bash
curl -X GET http://localhost:8080/api/v4/plugins/com.mattermost.calls/config \
  -H "Authorization: Bearer <token>"
```
- Should return valid JSON with iceServers array
- If empty, check TURN server configuration

**Check WebSocket events:**
- Use browser dev tools or proxy to see WS messages
- Look for `custom_com.mattermost.calls_*` events
- If no events, check WS subscription to channel

### No Audio Heard

**Check TURN server:**
- Verify TURN credentials are being generated
- Test TURN server connectivity: `turnutils_uclient -u username -w credential turn.server:3478`

**Check firewall:**
- UDP 8443 must be open
- TURN ports (3478, 5349) must be open

**Check ICE gathering:**
- Look for ICE candidate pairs in mobile logs
- Should see relay candidates if behind NAT

### Call State Not Updating

**Check permissions:**
- User must be channel member
- Check `check_channel_permission` in logs

**Check WebSocket:**
- Verify mobile subscribed to channel via WebSocket
- Look for `channel_subscribed` event

## Configuration Reference

### Environment Variables

```bash
# Enable/disable calls
RUSTCHAT_CALLS_ENABLED=true

# RTC ports
RUSTCHAT_CALLS_UDP_PORT=8443
RUSTCHAT_CALLS_TCP_PORT=8443

# ICE configuration
RUSTCHAT_CALLS_ICE_HOST_OVERRIDE=your.public.ip

# TURN server (for NAT traversal)
RUSTCHAT_CALLS_TURN_SECRET=your-shared-secret
RUSTCHAT_CALLS_TURN_TTL_MINUTES=1440

# STUN/TURN servers (JSON array)
RUSTCHAT_CALLS_STUN_SERVERS='["stun:stun.l.google.com:19302"]'
RUSTCHAT_CALLS_TURN_SERVERS='["turn:your.turn.server:3478"]'
```

## Known Limitations

1. **Phase 1 Implementation**: Current implementation is signaling-only with no actual SFU/media routing. Audio/video will not work until SFU is implemented.

2. **Single Node**: Call state is stored in-memory. Multi-node deployments need Redis backing (Phase 2).

3. **No Recording**: Call recording not implemented.

4. **No Transcription**: Live transcription not implemented.

## Success Criteria

For this implementation to be considered successful:

1. **All contract tests pass** - API returns expected responses
2. **Mobile app recognizes server** - Call button appears in channel
3. **Call lifecycle works** - Start, join, leave, end all function
4. **WebSocket events broadcast** - Events sent to all channel members
5. **TURN credentials work** - Mobile can gather ICE candidates
6. **Cross-platform compatible** - Works with official Mattermost mobile apps

## Next Steps

1. Implement SFU for actual audio/video routing
2. Add Redis backing for multi-node support
3. Add call recording capabilities
4. Implement bandwidth estimation and adaptive bitrate
5. Add noise suppression and echo cancellation
