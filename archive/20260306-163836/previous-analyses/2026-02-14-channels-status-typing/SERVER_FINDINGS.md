# Server Findings

## Rustchat API Analysis

### Status Endpoints

#### GET /users/{user_id}/status
File: `backend/src/api/v4/users.rs` lines 1220-1242

Current implementation:
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

**ISSUE**: Returns only `{ "status": "online" }` but Mattermost expects full Status object:
```json
{
  "user_id": "string",
  "status": "string",
  "manual": false,
  "last_activity_at": 0
}
```

#### POST /users/status/ids
File: `backend/src/api/v4/users.rs` lines 1128-1161

Current implementation returns array of objects with `user_id` and `status`.

**ISSUE**: Missing `manual` and `last_activity_at` fields.

### WebSocket Events

#### Status Change Event
File: `backend/src/api/v4/websocket.rs` lines 966-983

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

**ISSUE**: Missing `manual` and `last_activity_at` in data.

#### Typing Event
File: `backend/src/api/v4/websocket.rs` lines 841-859

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
            broadcast: map_broadcast(env.broadcast.as_ref()),
        })
    }
}
```

**ISSUE**: When typing events are created in `websocket_core.rs`, they don't have broadcast info with channel_id.

Looking at typing event creation in `websocket_core.rs` lines 245-267:
```rust
"typing" | "typing_start" => {
    if let Some(channel_id) = envelope.channel_id {
        let thread_root_id = serde_json::from_value::<TypingCommandData>(envelope.data)
            .ok()
            .and_then(|v| v.thread_root_id);
        
        let evt = WsEnvelope::event(
            EventType::UserTyping,
            TypingEvent {
                user_id,
                display_name: username.to_string(),
                thread_root_id,
            },
            Some(channel_id),
        );
        // Broadcast with channel scope
        state.ws_hub.broadcast_to_channel(channel_id, evt).await;
    }
}
```

The `WsEnvelope::event()` sets `channel_id` in the envelope but doesn't set `broadcast` field.

Looking at `map_envelope_to_mm` (line 1040-1056):
```rust
fn map_broadcast(b_opt: Option<&crate::realtime::WsBroadcast>) -> mm::Broadcast {
    if let Some(b) = b_opt {
        mm::Broadcast {
            omit_users: None,
            user_id: b.user_id.map(encode_mm_id).unwrap_or_default(),
            channel_id: b.channel_id.map(encode_mm_id).unwrap_or_default(),
            team_id: b.team_id.map(encode_mm_id).unwrap_or_default(),
        }
    } else {
        mm::Broadcast {
            omit_users: None,
            user_id: String::new(),
            channel_id: String::new(),
            team_id: String::new(),
        }
    }
}
```

The issue is that `WsEnvelope.event()` doesn't set the `broadcast` field - it only sets `channel_id`. So when `map_envelope_to_mm` is called, `env.broadcast` is `None`, resulting in empty broadcast fields.

### Hello Message
File: `backend/src/api/v4/websocket.rs` lines 289-303

```rust
let hello = mm::WebSocketMessage {
    seq: Some(hello_seq),
    event: "hello".to_string(),
    data: json!({
        "connection_id": actor_connection_id.clone(),
        "server_version": format!("rustchat-{}", env!("CARGO_PKG_VERSION")),
        "protocol_version": "1.0"
    }),
    broadcast: mm::Broadcast {
        omit_users: None,
        user_id: String::new(),
        channel_id: String::new(),
        team_id: String::new(),
    },
};
```

This looks correct for basic hello, but might be missing fields Mattermost mobile expects.

## Database Schema

### Users Table
File: Various migration files

Users table has `presence` field for status.

**ISSUE**: No `last_activity_at` field for tracking when user was last active.
**ISSUE**: No `manual` field to track if status was manually set.

## Required Changes

### 1. Add missing fields to users table
- `last_activity_at` - timestamp of last activity
- `manual` - boolean for manual status

### 2. Fix Status API endpoints
- Return full Status object with all fields
- Implement proper `last_activity_at` tracking

### 3. Fix WebSocket event formats
- Add broadcast info to typing events
- Add `manual` and `last_activity_at` to status_change events
- Ensure channel_id is in broadcast for all channel-related events

### 4. Update status change logic
- Track last activity when user interacts
- Set `manual` flag when user explicitly sets status
