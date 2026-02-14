# Mattermost-Mobile Compatibility Analysis

## Date: 2026-02-14
## Topic: Channels Not Loading, Status Not Showing, Typing Not Working

## Issues Summary

Based on the screenshots and codebase analysis, there are three main issues:

1. **Empty channels list on app open** - Channels don't appear until refresh
2. **User status not displayed** - Online/away/offline status indicators missing
3. **Typing indicators not shown** - "User is typing..." not appearing

## Mobile App Behavior

### Entry Flow (Initial Load)
From `app/actions/remote/entry/common.ts`:
- App calls `entry()` function on connect/reconnect
- Fetches: config, preferences, teams, user (me), channels
- Key API calls:
  - `fetchConfigAndLicense()` - Server config
  - `fetchMyPreferences()` - User preferences
  - `fetchMyTeams()` - User's teams
  - `fetchMe()` - Current user info
  - `fetchMyChannelsForTeam()` - **CRITICAL** - Gets channels for team

### WebSocket Reconnect Flow
From `app/actions/websocket/index.ts`:
- On reconnect: `handleReconnect()` → `doReconnect()` → `entry()`
- After reconnect, also calls:
  - `startPeriodicStatusUpdates()` - Fetches status every 5 minutes
  - `fetchPostsForChannel()` - Gets posts for current channel

### Status Updates
From `app/managers/websocket_manager.ts`:
- `startPeriodicStatusUpdates()` - Fetches all user statuses every `STATUS_INTERVAL` (5 min)
- Calls `fetchStatusByIds(serverUrl, userIds)` - Gets status for all users
- WebSocket event: `STATUS_CHANGED` handled by `handleStatusChangedEvent()`

### Typing Events
From `app/actions/websocket/users.ts`:
- Client sends: `user_typing` action with `channel_id` and `parent_id`
- Server should broadcast: `typing` event
- Mobile expects: `user_typing` event with `user_id`, `parent_id`

## Rustchat Implementation Analysis

### What's Working
1. ✅ WebSocket connection with session resumption
2. ✅ Hello message sent on connect
3. ✅ Channel API endpoints exist
4. ✅ Status update API endpoints exist
5. ✅ Typing event handlers exist

### Issues Found

#### Issue 1: Status Event Data Format Mismatch

In `backend/src/api/v4/websocket.rs` lines 966-983:
```rust
"user_updated" | "status_change" => {
    if let Some(status_str) = env.data.get("status").and_then(|v| v.as_str()) {
        let user_id = env
            .data
            .get("user_id")
            .and_then(|v| v.as_str())
            .and_then(parse_mm_or_uuid)
            .map(encode_mm_id)
            .unwrap_or_default();
        Some(mm::WebSocketMessage {
            seq,
            event: "status_change".to_string(),
            data: json!({ "user_id": user_id, "status": status_str }),
            broadcast: map_broadcast(env.broadcast.as_ref()),
        })
    } else {
        None
    }
}
```

The `PresenceEvent` struct is serialized directly, creating data like:
```json
{"user_id": "uuid", "status": "online"}
```

This should work, BUT the issue is that when `user_id` is serialized as a UUID, it's not being parsed correctly by `parse_mm_or_uuid` because the data comes from JSON serialization of `PresenceEvent` which serializes UUID as a string.

Wait - actually looking more carefully, `serde_json::to_value(PresenceEvent { user_id: Uuid, status: String })` would serialize `user_id` as a string (UUID to string), so `env.data.get("user_id")` returns a string, which should parse fine.

#### Issue 2: Missing `channel_id` in Typing Events

Looking at Mattermost mobile's `handleUserTypingEvent()`:
```typescript
const data = {
    channelId: msg.broadcast.channel_id,  // <-- Uses broadcast.channel_id
    rootId: msg.data.parent_id,
    userId: msg.data.user_id,
    username,
    now: Date.now(),
};
```

The mobile app expects `channel_id` in the broadcast field. Looking at `map_envelope_to_mm` for typing (lines 841-859):
```rust
"typing" | "typing_start" => {
    if let Ok(typing) = serde_json::from_value::<crate::realtime::TypingEvent>(env.data.clone())
    {
        let parent_id = typing.thread_root_id.map(encode_mm_id).unwrap_or_default();
        Some(mm::WebSocketMessage {
            seq,
            event: "user_typing".to_string(),
            data: json!({
                "parent_id": parent_id,
                "user_id": encode_mm_id(typing.user_id),
                "display_name": typing.display_name,
                "thread_root_id": parent_id,
            }),
            broadcast: map_broadcast(env.broadcast.as_ref()),  // <-- This may be None
        })
    }
}
```

The problem: When typing events are created in `websocket_core.rs:245-267`, they don't include broadcast info with `channel_id`. The `WsEnvelope` is created without broadcast data.

#### Issue 3: Initial Channel Load Timing

The mobile app expects channels to be available immediately after the entry flow completes. The issue might be:

1. The `hello` message format might not match what mobile expects
2. Channel data might not be properly synced on initial connect
3. The mobile app might be making API calls before WebSocket is ready

Looking at the hello message (lines 289-303):
```rust
let hello = mm::WebSocketMessage {
    seq: Some(hello_seq),
    event: "hello".to_string(),
    data: json!({
        "connection_id": actor_connection_id.clone(),
        "server_version": format!("rustchat-{}", env!("CARGO_PKG_VERSION")),
        "protocol_version": "1.0"
    }),
    broadcast: mm::Broadcast { ... },
};
```

Mattermost server sends additional fields in hello that might be expected by mobile.

#### Issue 4: Status API Response Format

From `backend/src/api/v4/users.rs` lines 1220-1242 (get user status):
```rust
async fn get_user_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = parse_mm_or_uuid(&user_id).ok_or(ApiError::not_found("user"))?;
    let status: Option<String> = sqlx::query_scalar("SELECT presence FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ApiError::internal_error(format!("DB error: {}", e)))?;

    let status = status.unwrap_or_else(|| "offline".to_string());
    Ok(Json(json!({ "status": status })))
}
```

The response is `{ "status": "online" }` but Mattermost expects a full Status object:
```json
{
  "user_id": "string",
  "status": "string",
  "manual": false,
  "last_activity_at": 0
}
```

This is likely the main issue with status not showing!

## Compatibility Contract

### API Endpoints Required

| Endpoint | Mattermost Format | Rustchat Status |
|----------|------------------|-----------------|
| GET /users/{id}/status | Full Status object | ❌ Only returns `{status}` |
| POST /users/status/ids | Array of Status | ❌ Unknown format |
| GET /users/me | User with status | ❌ Status may not be included |

### WebSocket Events Required

| Event | Data Fields | Broadcast Fields |
|-------|-------------|------------------|
| hello | connection_id, server_version, protocol_version | - |
| status_change | user_id, status | user_id, channel_id, team_id |
| user_typing | parent_id, user_id | channel_id |
| posted | post (JSON string) | channel_id |

## Gaps to Fix

### High Priority

1. **Fix Status API Response Format**
   - Change `/users/{id}/status` to return full Status object
   - Change `/users/status/ids` to return array of Status objects
   - Include status in user objects where applicable

2. **Fix Typing Event Broadcast**
   - Include channel_id in typing event broadcast
   - Ensure parent_id/thread_root_id is correctly mapped

3. **Verify Channel Loading**
   - Check if channels API returns correct format
   - Ensure channel memberships are included

### Medium Priority

4. **Add Missing WebSocket Events**
   - channel_updated
   - channel_deleted
   - team_updated
   - user_added/removed

5. **Improve Hello Message**
   - Add fields that mobile might expect

## Implementation Plan

1. Fix status API endpoints to return full Status objects
2. Fix typing events to include channel_id in broadcast
3. Test with mattermost-mobile to verify fixes
4. Add any additional missing events as needed
