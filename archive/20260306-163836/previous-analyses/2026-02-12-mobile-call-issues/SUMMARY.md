# Mobile Call Issues Analysis

Date: 2026-02-12

## Issues Reported

1. **Receiver can't see the call screen** - only minimal popup shown, disappears when enlarged
2. **Mute button not working on host**
3. **Thread button gives exception/error**

## Root Causes Identified

### Issue 1: Missing `session_id` in user_muted/user_unmuted Events

**Problem**: The HTTP API mute/unmute handlers don't include `session_id` in the broadcast event data.

**Evidence**:
- Mobile app expects `data.session_id` (websocket_event_handlers.test.ts:136,144)
- Rustchat WebSocket handler sends: `session_id: connection_id` (mod.rs:2857) ✓
- Rustchat HTTP handler sends: only `user_id` and `muted` (mod.rs:1542-1546) ✗

**Files**:
- `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs` lines 1542-1546, 1584-1588

### Issue 2: Thread ID Exception

**Problem**: Need to verify thread_id handling in call state responses.

**Evidence**:
- Mobile expects `threadId` in Call type (calls.ts:70)
- Rustchat returns `thread_id` in CallStateResponse (mod.rs:744)
- Mobile converts `thread_id` → `threadId` (calls.ts:163)

**Potential Issue**: Thread ID format or missing thread_id when call starts.

## Required Fixes

1. **Fix mute_user/unmute_user HTTP handlers** to include session_id in broadcast events
2. **Verify thread_id** is properly passed and formatted in all call state responses
3. **Verify dismissed_notification** field format matches mobile expectations

## Test Evidence

From mattermost-mobile tests (websocket_event_handlers.test.ts):
```typescript
// Expected data structure for user_muted/unmuted:
{
    broadcast: {channel_id: channelId},
    data: {session_id: sessionId},  // session_id is REQUIRED
}
```

## Files to Modify

1. `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
   - Lines 1542-1546: Add session_id to mute_user broadcast
   - Lines 1584-1588: Add session_id to unmute_user broadcast
