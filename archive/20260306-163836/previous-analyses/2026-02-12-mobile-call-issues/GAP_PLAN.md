# GAP_PLAN.md

## Fix 1: Add session_id to mute/unmute HTTP API events

### Current State
The HTTP API handlers for mute and unmute only send:
- `channel_id`
- `user_id`
- `muted`

### Required State
Must also include:
- `session_id` (the connection ID for the user)

### Implementation
Modify `mute_user` and `unmute_user` functions in `mod.rs`:
1. Get the user's session_id from the call participants
2. Include it in the broadcast_call_event data

### Changes Made
**File**: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`

1. **mute_user** (lines 1514-1562): Added session_id retrieval and inclusion in broadcast event
2. **unmute_user** (lines 1564-1613): Added session_id retrieval and inclusion in broadcast event
3. **host_mute** (lines 1749-1761): Added session_id to user_muted broadcast event
4. **host_mute_others** (lines 1813-1824): Added session_id to user_muted broadcast event

### Code Pattern
```rust
// Get user's session_id from participants
let participants = call_manager.get_participants(call.call_id).await;
let session_id = participants
    .iter()
    .find(|p| p.user_id == auth.user_id)
    .map(|p| p.session_id.to_string())
    .unwrap_or_default();

// Broadcast with session_id
broadcast_call_event(
    &state,
    "custom_com.mattermost.calls_user_muted",
    &channel_uuid,
    serde_json::json!({
        "channel_id": channel_id,
        "user_id": encode_mm_id(auth.user_id),
        "session_id": session_id,  // NOW INCLUDED
        "muted": true,
    }),
    None,
)
.await;
```

### Test Verification
- [x] Code compiles successfully
- [x] All existing tests pass
- [ ] Verify mobile receives session_id in user_muted events
- [ ] Verify mute button works correctly on host

## Fix 2: Verify Thread Handling

### Current State
Thread ID is included in responses but may be missing during call creation.

### Required State
Thread ID should be properly set and returned in all call state responses.

### Implementation
Review call creation flow to ensure thread_id is properly handled.

## Fix 3: Investigate Call Screen Visibility

### Current State
Receiver sees only minimal popup that disappears when enlarged.

### Investigation Needed
Check if call state is being properly broadcast to all participants on join.

### Potential Causes
1. Missing call_state event on user join
2. Incorrect channel_id in broadcast
3. Mobile not receiving or parsing call state correctly

## Compatibility Checklist

- [x] user_muted event includes session_id
- [x] user_unmuted event includes session_id
- [ ] Call state response includes all required fields
- [ ] Thread ID properly formatted and included
- [ ] Call screen visible to all participants

## Implementation Status

**Completed:**
- All mute/unmute endpoints now include `session_id` in broadcast events
- Code compiles without errors
- All existing tests pass

**Remaining Issues:**
1. Thread button exception - needs investigation into thread_id handling
2. Receiver call screen visibility - may require additional call state broadcast on join
