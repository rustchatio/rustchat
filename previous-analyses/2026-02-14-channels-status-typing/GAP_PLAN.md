# Gap Plan - REVISED

## Analysis Summary

After thorough analysis of both codebases:

### What's Already Working ✅

1. **Status API Endpoints** - Already return full `mm::Status` objects with all required fields:
   - `get_status` (line 1220) returns `mm::Status` with user_id, status, manual, last_activity_at
   - `get_my_status` (line 1244) returns full Status object
   - `get_statuses_by_ids` (line 1128) returns array of full Status objects

2. **Typing Events** - Already include proper broadcast with channel_id:
   - `websocket_core.rs` lines 260-265 and 284-289 create typing events with `WsBroadcast` including `channel_id`
   - `map_envelope_to_mm` correctly maps broadcast fields

3. **Channel APIs** - All required endpoints exist:
   - `GET /users/me/teams/{team_id}/channels` - `my_team_channels` (line 597)
   - Returns proper `mm::Channel` objects with hydrated DM display names

### Issues Identified

1. **WebSocket Status Event Data** - `persist_presence_and_broadcast` (websocket_core.rs:154) creates events with `PresenceEvent` which only has `user_id` and `status`. The mapping in `websocket.rs:966-983` expects these fields in the data.

2. **last_activity_at Source** - Currently using `last_login_at` from database, which may not reflect recent activity

3. **Initialization Timing** - Mobile app might fetch channels before WebSocket connection is fully established

## Implementation Tasks

### Task 1: Verify Status WebSocket Event Format ✅
**Status: VERIFIED WORKING**

The status change events ARE being sent correctly:
- `PresenceEvent` serializes to `{"user_id": "...", "status": "online"}`
- `map_envelope_to_mm` extracts these fields correctly
- Event is broadcast to all connections

### Task 2: Enhance Status Event Data
**Priority: MEDIUM**

Files to modify:
- `backend/src/api/websocket_core.rs` - Update `persist_presence_and_broadcast`

Changes:
1. Include `manual` field in the event data (currently always false for auto-updates)
2. Include proper `last_activity_at` timestamp
3. Consider creating a more detailed status event struct

### Task 3: Add Debug Logging
**Priority: HIGH**

Add tracing/debugging to help diagnose mobile connectivity issues:
1. Log when channels API is called
2. Log WebSocket connection/disconnection events
3. Log status broadcast events
4. Log typing events

### Task 4: Test with Real Mobile App
**Priority: HIGH**

Test scenarios:
1. Fresh app install/login
2. Background/foreground app switch
3. Network disconnect/reconnect
4. Multiple users and typing indicators

## Potential Root Causes of Reported Issues

1. **Empty Channels on Open**: 
   - Mobile app might make API calls before auth is complete
   - Timing issue between WebSocket connect and initial data fetch
   - Possible: Add retry logic or ensure proper sequencing

2. **Status Not Showing**:
   - Status API returns correct data
   - WebSocket events are being sent
   - Possible: Mobile app UI not updating correctly, or events not reaching the right clients
   - Possible: Status events need `broadcast.user_id` set

3. **Typing Not Showing**:
   - Events ARE being sent with channel_id in broadcast
   - Possible: Mobile app expects different event format
   - Possible: Events not reaching channel subscribers

## Immediate Actions Needed

1. Add comprehensive logging to trace the flow
2. Verify the WebSocket subscription mechanism is working
3. Test with actual mobile app to see exact API calls and responses
4. Check if there are any CORS or connection issues

## Code Review Notes

Looking at the actual implementation:
- ✅ Status API: `get_status` returns `Json<mm::Status>` - CORRECT
- ✅ Status API: `get_statuses_by_ids` returns `Json<Vec<mm::Status>>` - CORRECT
- ✅ Typing: Events created with `.with_broadcast(WsBroadcast { channel_id: Some(...), ... })` - CORRECT
- ✅ Typing: `map_envelope_to_mm` calls `map_broadcast(env.broadcast.as_ref())` - CORRECT
- ✅ Channels: `my_team_channels` queries and returns channels - CORRECT

The code looks correct. The issues might be:
1. Runtime/connection issues
2. Mobile app expecting slightly different format
3. Timing issues between connection and data fetch

## Changes Made

### 1. Enhanced WebSocket Status Events (backend/src/api/websocket_core.rs)
- Updated `persist_presence_and_broadcast` to include complete status data:
  - Added `manual` field (false for automatic updates)
  - Added `last_activity_at` timestamp (current time)
  - Added proper `WsBroadcast` with `user_id` set to identify whose status changed
- Added `chrono::Utc` import for timestamp handling
- Added debug logging to trace status broadcasts

### 2. Enhanced Status Event Mapping (backend/src/api/v4/websocket.rs)
- Updated `map_envelope_to_mm` for `status_change` events to:
  - Extract and include `manual` field from event data
  - Extract and include `last_activity_at` from event data
  - Pass through to Mattermost-compatible message format

### 3. Added Debug Logging (backend/src/api/v4/users.rs)
- Added logging to `my_team_channels` to trace:
  - When channels are requested
  - How many channels are found
  - User and team IDs for debugging

### 4. Added Debug Logging (backend/src/realtime/hub.rs)
- Added logging to `broadcast` function for important events:
  - typing/stop_typing events
  - status_change events
  - Shows whether broadcast info is present

## Verification Checklist

- [x] GET /users/{id}/status returns full Status object
- [x] POST /users/status/ids returns array of full Status objects
- [x] Typing WebSocket events include broadcast.channel_id
- [x] Status WebSocket events are being broadcast
- [x] Debug logging added to trace issues
- [ ] Tested with actual mattermost-mobile app
- [ ] User status visible in mobile app
- [ ] Typing indicators visible in mobile app
- [ ] Channels load on app open without refresh
