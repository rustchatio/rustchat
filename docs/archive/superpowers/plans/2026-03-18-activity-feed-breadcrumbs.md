# Activity Feed and Breadcrumbs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Activity Feed panel for mentions/replies/reactions, Breadcrumbs navigation showing Team > Channel > Thread hierarchy, and Quick Switcher (Cmd+K) for fuzzy-search navigation.

**Architecture:** Backend adds `activities` table with trigger-based insertion on mentions/replies/reactions. Frontend uses feature-based architecture with Pinia stores, service layer for business logic, and slide-over panels similar to ThreadPanel.

**Tech Stack:** Rust/Axum/SQLx backend, Vue 3 + TypeScript + Pinia frontend, match-sorter for fuzzy search

---

## File Structure

### Backend Files
- `backend/migrations/20260318000001_create_activities.sql` - Database schema
- `backend/src/models/activity.rs` - Activity model and ActivityType enum
- `backend/src/services/activity.rs` - Activity creation and fetching service
- `backend/src/api/v4/users/activity.rs` - REST endpoints for activity feed
- `backend/src/realtime/events.rs` - WebSocket event types for activity

### Frontend Files
- `frontend/src/features/activity/index.ts` - Feature exports
- `frontend/src/features/activity/types.ts` - ActivityType enum and Activity interface
- `frontend/src/features/activity/repositories/activityRepository.ts` - API calls
- `frontend/src/features/activity/services/activityService.ts` - Business logic
- `frontend/src/features/activity/stores/activityStore.ts` - Pinia store
- `frontend/src/features/activity/handlers/activitySocketHandlers.ts` - WebSocket handlers
- `frontend/src/components/activity/ActivityFeed.vue` - Slide-over panel
- `frontend/src/components/activity/ActivityItem.vue` - Individual activity row
- `frontend/src/components/activity/ActivityIcon.vue` - Type icons
- `frontend/src/components/activity/ActivityFilters.vue` - Filter tabs
- `frontend/src/components/navigation/BreadcrumbBar.vue` - Breadcrumb navigation
- `frontend/src/components/navigation/QuickSwitcherModal.vue` - Cmd+K modal
- `frontend/src/composables/useQuickSwitcher.ts` - Keyboard shortcut composable

### Test Files
- `backend/tests/api_activity.rs` - Backend integration tests
- `frontend/src/components/activity/__tests__/ActivityFeed.test.ts` - Component tests

---

## Phase 2a: Activity Feed (5 days)

### Task 1: Create Database Migration

**Files:**
- Create: `backend/migrations/20260318000001_create_activities.sql`

- [ ] **Step 1: Write migration file**

```sql
-- Create activities table for user activity feed
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(20) NOT NULL CHECK (type IN ('mention', 'reply', 'reaction', 'dm', 'thread_reply')),
    actor_id UUID NOT NULL REFERENCES users(id),
    channel_id UUID NOT NULL REFERENCES channels(id),
    team_id UUID NOT NULL REFERENCES teams(id),
    post_id UUID NOT NULL REFERENCES posts(id),
    root_id UUID REFERENCES posts(id),
    message_text TEXT,
    reaction VARCHAR(50),
    read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_activities_user_created ON activities(user_id, created_at DESC);
CREATE INDEX idx_activities_user_read ON activities(user_id, read) WHERE read = FALSE;
CREATE INDEX idx_activities_post ON activities(post_id);
```

- [ ] **Step 2: Run migration to verify syntax**

Run: `cd backend && cargo sqlx migrate run`
Expected: Migration succeeds without errors

- [ ] **Step 3: Commit**

```bash
git add backend/migrations/20260318000001_create_activities.sql
git commit -m "feat: add activities table for activity feed"
```

---

### Task 2: Create Activity Model

**Files:**
- Create: `backend/src/models/activity.rs`
- Modify: `backend/src/models/mod.rs`

- [ ] **Step 1: Create activity.rs model file**

```rust
//! Activity (notification) model for user activity feed

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Types of activity that can appear in the feed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Mention,
    Reply,
    Reaction,
    Dm,
    ThreadReply,
}

/// Activity entity - represents a notification for a user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: ActivityType,
    pub actor_id: Uuid,
    pub channel_id: Uuid,
    pub team_id: Uuid,
    pub post_id: Uuid,
    pub root_id: Option<Uuid>,
    pub message_text: Option<String>,
    pub reaction: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Activity response with joined user/channel info for API
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ActivityResponse {
    pub id: Uuid,
    pub r#type: ActivityType,
    pub actor_id: Uuid,
    pub actor_username: String,
    pub actor_avatar_url: Option<String>,
    pub channel_id: Uuid,
    pub channel_name: String,
    pub team_id: Uuid,
    pub team_name: String,
    pub post_id: Uuid,
    pub root_id: Option<Uuid>,
    pub message_text: Option<String>,
    pub reaction: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for fetching activities
#[derive(Debug, Clone, Default)]
pub struct ActivityQuery {
    pub cursor: Option<Uuid>,
    pub limit: i64,
    pub activity_type: Option<String>, // Comma-separated types
    pub unread_only: bool,
}

/// Response for activity feed endpoint
#[derive(Debug, Clone, Serialize)]
pub struct ActivityFeedResponse {
    pub order: Vec<String>,
    pub activities: std::collections::HashMap<String, ActivityResponse>,
    pub unread_count: i64,
    pub next_cursor: Option<String>,
}

/// Request to mark activities as read
#[derive(Debug, Clone, Deserialize)]
pub struct MarkReadRequest {
    pub activity_ids: Vec<Uuid>,
}

impl ActivityType {
    /// Parse activity type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mention" => Some(ActivityType::Mention),
            "reply" => Some(ActivityType::Reply),
            "reaction" => Some(ActivityType::Reaction),
            "dm" => Some(ActivityType::Dm),
            "thread_reply" => Some(ActivityType::ThreadReply),
            _ => None,
        }
    }
}
```

- [ ] **Step 2: Update models/mod.rs to include activity module**

Add to `backend/src/models/mod.rs`:
```rust
pub mod activity;
pub use activity::*;
```

- [ ] **Step 3: Run cargo check to verify compilation**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add backend/src/models/activity.rs backend/src/models/mod.rs
git commit -m "feat: add Activity model and ActivityType enum"
```

---

### Task 3: Create Activity Service

**Files:**
- Create: `backend/src/services/activity.rs`
- Modify: `backend/src/services/mod.rs`

- [ ] **Step 1: Create activity service file**

```rust
//! Activity service - handles creation and retrieval of user activities

use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::models::{Activity, ActivityFeedResponse, ActivityQuery, ActivityResponse, ActivityType};
use crate::realtime::{EventType, WsBroadcast, WsEnvelope};
use uuid::Uuid;

/// Create a new activity entry
pub async fn create_activity(
    state: &AppState,
    user_id: Uuid,
    activity_type: ActivityType,
    actor_id: Uuid,
    channel_id: Uuid,
    team_id: Uuid,
    post_id: Uuid,
    root_id: Option<Uuid>,
    message_text: Option<String>,
    reaction: Option<String>,
) -> ApiResult<Activity> {
    let activity: Activity = sqlx::query_as(
        r#"
        INSERT INTO activities
            (user_id, type, actor_id, channel_id, team_id, post_id, root_id, message_text, reaction)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING
            id, user_id, type, actor_id, channel_id, team_id, post_id, root_id,
            message_text, reaction, read, created_at
        "#
    )
    .bind(user_id)
    .bind(activity_type)
    .bind(actor_id)
    .bind(channel_id)
    .bind(team_id)
    .bind(post_id)
    .bind(root_id)
    .bind(message_text.map(|m| truncate_text(&m, 200)))
    .bind(reaction)
    .fetch_one(&state.db)
    .await?;

    // Broadcast to the affected user via WebSocket
    let broadcast = WsEnvelope::event(
        EventType::ActivityCreated,
        serde_json::json!({
            "activity_id": activity.id,
            "user_id": activity.user_id,
            "type": activity.r#type
        }),
        None,
    )
    .with_broadcast(WsBroadcast {
        user_id: Some(user_id),
        channel_id: None,
        team_id: None,
        exclude_user_id: None,
    });

    state.ws_hub.broadcast(broadcast).await;

    Ok(activity)
}

/// Get activity feed for a user
pub async fn get_activities(
    state: &AppState,
    user_id: Uuid,
    query: ActivityQuery,
) -> ApiResult<ActivityFeedResponse> {
    let limit = query.limit.clamp(1, 100);

    // Build the query dynamically based on filters
    let mut sql = String::from(
        r#"
        SELECT
            a.id, a.type, a.actor_id, a.channel_id, a.team_id, a.post_id, a.root_id,
            a.message_text, a.reaction, a.read, a.created_at,
            u.username as actor_username, u.avatar_url as actor_avatar_url,
            c.name as channel_name, t.name as team_name
        FROM activities a
        JOIN users u ON a.actor_id = u.id
        JOIN channels c ON a.channel_id = c.id
        JOIN teams t ON a.team_id = t.id
        WHERE a.user_id = $1
        "#
    );

    let mut param_idx = 2;

    // Add cursor filter if provided
    if query.cursor.is_some() {
        sql.push_str(&format!(" AND a.id < ${} ", param_idx));
        param_idx += 1;
    }

    // Add type filter if provided
    if query.activity_type.is_some() {
        sql.push_str(&format!(" AND a.type::text = ANY(string_to_array(${}, ',')) ", param_idx));
        param_idx += 1;
    }

    // Add unread filter if requested
    if query.unread_only {
        sql.push_str(" AND a.read = FALSE ");
    }

    sql.push_str(" ORDER BY a.created_at DESC ");
    sql.push_str(&format!(" LIMIT ${} ", param_idx));

    // Execute query
    let activities: Vec<ActivityResponse> = if let Some(cursor) = query.cursor {
        let type_filter = query.activity_type.as_ref();

        if let Some(types) = type_filter {
            if query.unread_only {
                sqlx::query_as(&sql)
                    .bind(user_id)
                    .bind(cursor)
                    .bind(types)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            } else {
                sqlx::query_as(&sql)
                    .bind(user_id)
                    .bind(cursor)
                    .bind(types)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            }
        } else {
            if query.unread_only {
                sqlx::query_as(&sql.replace(&format!("AND a.type::text = ANY(string_to_array(${}, ','))", param_idx - 1), ""))
                    .bind(user_id)
                    .bind(cursor)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            } else {
                sqlx::query_as(&sql.replace(&format!("AND a.type::text = ANY(string_to_array(${}, ','))", param_idx - 1), ""))
                    .bind(user_id)
                    .bind(cursor)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            }
        }
    } else {
        let type_filter = query.activity_type.as_ref();
        let sql_no_cursor = sql.replace(&format!("AND a.id < ${}", param_idx - 2), "");

        if let Some(types) = type_filter {
            if query.unread_only {
                sqlx::query_as(&sql_no_cursor)
                    .bind(user_id)
                    .bind(types)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            } else {
                sqlx::query_as(&sql_no_cursor)
                    .bind(user_id)
                    .bind(types)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            }
        } else {
            let sql_clean = sql_no_cursor.replace(&format!("AND a.type::text = ANY(string_to_array(${}, ','))", param_idx - 1), "");
            if query.unread_only {
                sqlx::query_as(&sql_clean)
                    .bind(user_id)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            } else {
                sqlx::query_as(&sql_clean)
                    .bind(user_id)
                    .bind(limit + 1)
                    .fetch_all(&state.db)
                    .await?
            }
        }
    };

    // Determine pagination
    let has_more = activities.len() > limit as usize;
    let activities: Vec<ActivityResponse> = activities.into_iter().take(limit as usize).collect();

    let next_cursor = if has_more {
        activities.last().map(|a| a.id.to_string())
    } else {
        None
    };

    // Get unread count
    let unread_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activities WHERE user_id = $1 AND read = FALSE"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    // Build response
    let order: Vec<String> = activities.iter().map(|a| a.id.to_string()).collect();
    let activities_map: std::collections::HashMap<String, ActivityResponse> = activities
        .into_iter()
        .map(|a| (a.id.to_string(), a))
        .collect();

    Ok(ActivityFeedResponse {
        order,
        activities: activities_map,
        unread_count,
        next_cursor,
    })
}

/// Mark specific activities as read
pub async fn mark_activities_read(
    state: &AppState,
    user_id: Uuid,
    activity_ids: Vec<Uuid>,
) -> ApiResult<usize> {
    if activity_ids.is_empty() {
        return Ok(0);
    }

    let result = sqlx::query(
        "UPDATE activities SET read = TRUE WHERE user_id = $1 AND id = ANY($2)"
    )
    .bind(user_id)
    .bind(&activity_ids)
    .execute(&state.db)
    .await?;

    Ok(result.rows_affected() as usize)
}

/// Mark all activities as read for a user
pub async fn mark_all_read(
    state: &AppState,
    user_id: Uuid,
) -> ApiResult<usize> {
    let result = sqlx::query(
        "UPDATE activities SET read = TRUE WHERE user_id = $1 AND read = FALSE"
    )
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(result.rows_affected() as usize)
}

/// Create activity for a mention
pub async fn create_mention_activity(
    state: &AppState,
    mentioned_user_id: Uuid,
    actor_id: Uuid,
    channel_id: Uuid,
    team_id: Uuid,
    post_id: Uuid,
    message: &str,
) -> ApiResult<()> {
    create_activity(
        state,
        mentioned_user_id,
        ActivityType::Mention,
        actor_id,
        channel_id,
        team_id,
        post_id,
        None,
        Some(message.to_string()),
        None,
    ).await?;
    Ok(())
}

/// Create activity for a reply
pub async fn create_reply_activity(
    state: &AppState,
    parent_user_id: Uuid,
    actor_id: Uuid,
    channel_id: Uuid,
    team_id: Uuid,
    post_id: Uuid,
    reply_message: &str,
) -> ApiResult<()> {
    create_activity(
        state,
        parent_user_id,
        ActivityType::Reply,
        actor_id,
        channel_id,
        team_id,
        post_id,
        None,
        Some(reply_message.to_string()),
        None,
    ).await?;
    Ok(())
}

/// Create activity for a reaction
pub async fn create_reaction_activity(
    state: &AppState,
    post_user_id: Uuid,
    actor_id: Uuid,
    channel_id: Uuid,
    team_id: Uuid,
    post_id: Uuid,
    emoji: &str,
) -> ApiResult<()> {
    create_activity(
        state,
        post_user_id,
        ActivityType::Reaction,
        actor_id,
        channel_id,
        team_id,
        post_id,
        None,
        None,
        Some(emoji.to_string()),
    ).await?;
    Ok(())
}

/// Create activity for a thread reply
pub async fn create_thread_reply_activity(
    state: &AppState,
    parent_user_id: Uuid,
    actor_id: Uuid,
    channel_id: Uuid,
    team_id: Uuid,
    post_id: Uuid,
    root_id: Uuid,
    reply_message: &str,
) -> ApiResult<()> {
    create_activity(
        state,
        parent_user_id,
        ActivityType::ThreadReply,
        actor_id,
        channel_id,
        team_id,
        post_id,
        Some(root_id),
        Some(reply_message.to_string()),
        None,
    ).await?;
    Ok(())
}

/// Create activity for a DM
pub async fn create_dm_activity(
    state: &AppState,
    recipient_id: Uuid,
    actor_id: Uuid,
    channel_id: Uuid,
    team_id: Uuid,
    post_id: Uuid,
    message: &str,
) -> ApiResult<()> {
    create_activity(
        state,
        recipient_id,
        ActivityType::Dm,
        actor_id,
        channel_id,
        team_id,
        post_id,
        None,
        Some(message.to_string()),
        None,
    ).await?;
    Ok(())
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}
```

- [ ] **Step 2: Update services/mod.rs**

Add to `backend/src/services/mod.rs`:
```rust
pub mod activity;
```

- [ ] **Step 3: Run cargo check**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add backend/src/services/activity.rs backend/src/services/mod.rs
git commit -m "feat: add activity service with CRUD operations"
```

---

### Task 4: Create Activity API Endpoints

**Files:**
- Create: `backend/src/api/v4/users/activity.rs`
- Modify: `backend/src/api/v4/users/mod.rs` (or create it)
- Modify: `backend/src/api/v4/mod.rs`

- [ ] **Step 1: Check if users/mod.rs exists, create if needed**

Check: `ls backend/src/api/v4/users/`

If `mod.rs` doesn't exist, create `backend/src/api/v4/users/mod.rs`:
```rust
pub mod activity;
pub mod preferences;
pub mod sidebar_categories;

use axum::Router;
use crate::api::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(activity::router())
        .merge(preferences::router(state.clone()))
        .merge(sidebar_categories::router(state))
}
```

- [ ] **Step 2: Create activity.rs API file**

```rust
//! Activity feed API endpoints

use crate::api::AppState;
use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::{ActivityFeedResponse, ActivityQuery, MarkReadRequest};
use crate::services::activity;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for GET /users/{id}/activity
#[derive(Debug, Deserialize, Default)]
pub struct GetActivityParams {
    pub cursor: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub r#type: Option<String>,
    #[serde(default)]
    pub unread_only: bool,
}

fn default_limit() -> i64 {
    50
}

/// Routes for activity endpoints
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/{user_id}/activity", get(get_activity_feed))
        .route("/users/{user_id}/activity/read", post(mark_read))
        .route("/users/{user_id}/activity/read-all", post(mark_all_read))
}

/// GET /api/v4/users/{user_id}/activity
/// Get activity feed for a user
async fn get_activity_feed(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<GetActivityParams>,
    auth: AuthUser,
) -> Result<Json<ActivityFeedResponse>, AppError> {
    // Users can only view their own activity feed
    if auth.user_id != user_id {
        return Err(AppError::Forbidden(
            "Cannot view another user's activity feed".to_string()
        ));
    }

    let query = ActivityQuery {
        cursor: params.cursor,
        limit: params.limit,
        activity_type: params.r#type,
        unread_only: params.unread_only,
    };

    let response = activity::get_activities(&state, user_id, query).await?;
    Ok(Json(response))
}

/// POST /api/v4/users/{user_id}/activity/read
/// Mark specific activities as read
async fn mark_read(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<MarkReadRequest>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    if auth.user_id != user_id {
        return Err(AppError::Forbidden(
            "Cannot modify another user's activity".to_string()
        ));
    }

    let updated = activity::mark_activities_read(&state, user_id, payload.activity_ids).await?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}

/// POST /api/v4/users/{user_id}/activity/read-all
/// Mark all activities as read
async fn mark_all_read(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    if auth.user_id != user_id {
        return Err(AppError::Forbidden(
            "Cannot modify another user's activity".to_string()
        ));
    }

    let updated = activity::mark_all_read(&state, user_id).await?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}
```

- [ ] **Step 3: Update v4/mod.rs to include users router**

Add to `backend/src/api/v4/mod.rs` imports:
```rust
pub mod users;
```

In the `router_with_body_limits` function, add the users router:
```rust
// Users - medium limit for profiles
.merge(users::router(state.clone()).layer(DefaultBodyLimit::max(medium_limit)))
```

- [ ] **Step 4: Run cargo check**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add backend/src/api/v4/users/activity.rs backend/src/api/v4/users/mod.rs backend/src/api/v4/mod.rs
git commit -m "feat: add activity feed REST endpoints"
```

---

### Task 5: Integrate Activity Creation into Post Service

**Files:**
- Modify: `backend/src/services/posts.rs`

- [ ] **Step 1: Import activity service functions**

Add to imports in `backend/src/services/posts.rs`:
```rust
use crate::services::activity;
```

- [ ] **Step 2: Add mention activity creation after post creation**

In the `create_post` function, after parsing mentions, use the activity service helper functions:

```rust
// After: Parse mentions (simple parsing for now)
let mentions: Vec<String> = response
    .message
    .split_whitespace()
    .filter_map(|word| {
        if word.starts_with('@') && word.len() > 1 {
            Some(
                word[1..]
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string(),
            )
        } else {
            None
        }
    })
    .collect();

if !mentions.is_empty() {
    // Get team_id for the channel
    let team_info: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT team_id, name FROM channels WHERE id = $1"
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some((team_id, _channel_name)) = team_info {
        // Create mention activities for each mentioned user using service helper
        for username in &mentions {
            // Find the user by username
            if let Ok(Some(mentioned_user_id)) = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM users WHERE username = $1"
            )
            .bind(username)
            .fetch_optional(&state.db)
            .await
            {
                // Don't create activity for self-mentions
                if mentioned_user_id != user_id {
                    let _ = activity::create_mention_activity(
                        state,
                        mentioned_user_id,
                        user_id,
                        channel_id,
                        team_id,
                        post.id,
                        &response.message,
                    ).await;
                }
            }
        }
    }

    // ... rest of existing mention handling code ...
}
```

- [ ] **Step 3: Add reply/thread reply activity creation using service helpers**

For replies (when `root_post_id` is Some), create reply activity for the parent post author:

```rust
// In create_post function, after the mention handling section, add:

// If this is a reply, create activity for parent post author using service helper
if let Some(r_id) = root_post_id {
    // Get the parent post author
    if let Ok(Some(parent_user_id)) = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM posts WHERE id = $1"
    )
    .bind(r_id)
    .fetch_optional(&state.db)
    .await
    {
        // Don't notify if replying to own post
        if parent_user_id != user_id {
            // Get team_id
            if let Ok(Some(team_id)) = sqlx::query_scalar::<_, Uuid>(
                "SELECT team_id FROM channels WHERE id = $1"
            )
            .bind(channel_id)
            .fetch_optional(&state.db)
            .await
            {
                // Check if this is a thread reply (parent has root_post_id)
                let parent_root: Option<Uuid> = sqlx::query_scalar(
                    "SELECT root_post_id FROM posts WHERE id = $1"
                )
                .bind(r_id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();

                if parent_root.is_some() {
                    // Thread reply - notify parent author using helper
                    let _ = activity::create_thread_reply_activity(
                        state,
                        parent_user_id,
                        user_id,
                        channel_id,
                        team_id,
                        post.id,
                        r_id,
                        &response.message,
                    ).await;
                } else {
                    // Regular reply - notify parent author using helper
                    let _ = activity::create_reply_activity(
                        state,
                        parent_user_id,
                        user_id,
                        channel_id,
                        team_id,
                        post.id,
                        &response.message,
                    ).await;
                }
            }
        }
    }
}
```

- [ ] **Step 4: Run cargo check**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add backend/src/services/posts.rs
git commit -m "feat: integrate activity creation on mentions and replies"
```

---

### Task 6: Integrate Activity Creation into Reactions

**Files:**
- Modify: `backend/src/api/v4/posts/reactions.rs`

- [ ] **Step 1: Add activity creation in add_reaction function**

In `backend/src/api/v4/posts/reactions.rs`, in the `add_reaction` function after the reaction is created and before the WebSocket broadcast, add activity creation:

```rust
// After the reaction is inserted (around line 60), before the mm_reaction creation,
// add activity creation for post author:

// Get the post author to create activity
let post_info: Option<(Uuid, Uuid, Uuid)> = sqlx::query_as(
    "SELECT p.user_id, p.channel_id, c.team_id FROM posts p JOIN channels c ON p.channel_id = c.id WHERE p.id = $1"
)
.bind(post_id)
.fetch_optional(&state.db)
.await?;

if let Some((post_user_id, chan_id, team_id)) = post_info {
    // Don't create activity for self-reactions
    if post_user_id != auth.user_id {
        let _ = crate::services::activity::create_reaction_activity(
            &state,
            post_user_id,
            auth.user_id,
            chan_id,
            team_id,
            post_id,
            &emoji_name,
        ).await;
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add backend/src/api/v4/posts/reactions.rs
git commit -m "feat: create activity on reactions"
```

---

### Task 7: Add WebSocket Event Type for Activity

**Files:**
- Modify: `backend/src/realtime/events.rs`

- [ ] **Step 1: Add ActivityCreated event type to EventType enum**

In `backend/src/realtime/events.rs`, add to the EventType enum (around line 38):

```rust
pub enum EventType {
    // ... existing variants ...

    // Activity feed events
    ActivityCreated,
    ActivityRead,

    // ... rest of variants ...
}
```

- [ ] **Step 2: Add string mappings in as_str() method**

In the `as_str()` method (around line 80), add:

```rust
impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            // ... existing mappings ...
            Self::ActivityCreated => "activity_created",
            Self::ActivityRead => "activity_read",
            // ... rest of mappings ...
        }
    }
}
```

- [ ] **Step 3: Run cargo check**

Run: `cd backend && cargo check`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add backend/src/realtime/events.rs
git commit -m "feat: add ActivityCreated and ActivityRead WebSocket events"
```

---

### Task 8: Create Frontend Activity Types

**Files:**
- Create: `frontend/src/features/activity/types.ts`

- [ ] **Step 1: Create types file**

```typescript
/**
 * Activity Feed Types
 * Following the pattern from messages feature
 */

export enum ActivityType {
  MENTION = 'mention',
  REPLY = 'reply',
  REACTION = 'reaction',
  DM = 'dm',
  THREAD_REPLY = 'thread_reply'
}

export interface Activity {
  id: string;
  type: ActivityType;
  actorId: string;
  actorUsername: string;
  actorAvatarUrl?: string;
  channelId: string;
  channelName: string;
  teamId: string;
  teamName: string;
  postId: string;
  rootId?: string;
  message?: string;
  reaction?: string;
  read: boolean;
  createdAt: Date;
}

export interface ActivityFeedResponse {
  order: string[];
  activities: Record<string, Activity>;
  unreadCount: number;
  nextCursor?: string;
}

export interface ActivityQueryParams {
  cursor?: string;
  limit?: number;
  type?: ActivityType | string;
  unreadOnly?: boolean;
}
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/features/activity/types.ts
git commit -m "feat: add Activity types"
```

---

### Task 9: Create Activity Repository

**Files:**
- Create: `frontend/src/features/activity/repositories/activityRepository.ts`

- [ ] **Step 1: Create repository file**

```typescript
/**
 * Activity Repository - API layer for activity feed
 * Following the pattern from messageRepository
 */

import type { Activity, ActivityFeedResponse, ActivityQueryParams } from '../types'
import { apiClient } from '../../../api/client'

// Transform API response to domain model
function transformActivity(apiActivity: any): Activity {
  return {
    id: apiActivity.id,
    type: apiActivity.type,
    actorId: apiActivity.actor_id,
    actorUsername: apiActivity.actor_username,
    actorAvatarUrl: apiActivity.actor_avatar_url,
    channelId: apiActivity.channel_id,
    channelName: apiActivity.channel_name,
    teamId: apiActivity.team_id,
    teamName: apiActivity.team_name,
    postId: apiActivity.post_id,
    rootId: apiActivity.root_id,
    message: apiActivity.message_text,
    reaction: apiActivity.reaction,
    read: apiActivity.read,
    createdAt: new Date(apiActivity.created_at)
  }
}

export const activityRepository = {
  async getFeed(
    userId: string,
    params: ActivityQueryParams = {}
  ): Promise<ActivityFeedResponse> {
    const queryParams = new URLSearchParams()
    if (params.cursor) queryParams.set('cursor', params.cursor)
    if (params.limit) queryParams.set('limit', params.limit.toString())
    if (params.type) queryParams.set('type', params.type)
    if (params.unreadOnly) queryParams.set('unread_only', 'true')

    const response = await apiClient.get(`/users/${userId}/activity?${queryParams}`)

    const activities: Record<string, Activity> = {}
    for (const [id, activity] of Object.entries(response.data.activities || {})) {
      activities[id] = transformActivity(activity)
    }

    return {
      order: response.data.order || [],
      activities,
      unreadCount: response.data.unread_count || 0,
      nextCursor: response.data.next_cursor
    }
  },

  async markRead(userId: string, activityIds: string[]): Promise<number> {
    const response = await apiClient.post(`/users/${userId}/activity/read`, {
      activity_ids: activityIds
    })
    return response.data.updated
  },

  async markAllRead(userId: string): Promise<number> {
    const response = await apiClient.post(`/users/${userId}/activity/read-all`)
    return response.data.updated
  }
}
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/features/activity/repositories/activityRepository.ts
git commit -m "feat: add activity repository"
```

---

### Task 10: Create Activity Store

**Files:**
- Create: `frontend/src/features/activity/stores/activityStore.ts`

- [ ] **Step 1: Create store file**

```typescript
/**
 * Activity Store - Pure state management for activity feed
 * Following the pattern from messageStore
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Activity, ActivityType } from '../types'

export const useActivityStore = defineStore('activityStore', () => {
  // State
  const activities = ref<Map<string, Activity>>(new Map())
  const order = ref<string[]>([])
  const unreadCount = ref(0)
  const hasMore = ref(false)
  const cursor = ref<string | null>(null)
  const filter = ref<ActivityType | null>(null)
  const isLoading = ref(false)
  const isOpen = ref(false)

  // Getters
  const getActivities = computed(() => {
    return order.value
      .map(id => activities.value.get(id))
      .filter((a): a is Activity => a !== undefined)
  })

  const unreadActivities = computed(() => {
    return getActivities.value.filter(a => !a.read)
  })

  // Actions
  function setActivities(newActivities: Activity[], newOrder: string[]) {
    activities.value.clear()
    for (const activity of newActivities) {
      activities.value.set(activity.id, activity)
    }
    order.value = newOrder
  }

  function appendActivities(newActivities: Activity[], newOrder: string[]) {
    for (const activity of newActivities) {
      if (!activities.value.has(activity.id)) {
        activities.value.set(activity.id, activity)
      }
    }
    order.value = [...order.value, ...newOrder.filter(id => !order.value.includes(id))]
  }

  function addActivity(activity: Activity) {
    if (!activities.value.has(activity.id)) {
      activities.value.set(activity.id, activity)
      order.value.unshift(activity.id)
      if (!activity.read) {
        unreadCount.value++
      }
    }
  }

  function markActivityRead(activityId: string) {
    const activity = activities.value.get(activityId)
    if (activity && !activity.read) {
      activity.read = true
      unreadCount.value = Math.max(0, unreadCount.value - 1)
    }
  }

  function markAllRead() {
    for (const activity of activities.value.values()) {
      activity.read = true
    }
    unreadCount.value = 0
  }

  function setUnreadCount(count: number) {
    unreadCount.value = count
  }

  function setHasMore(value: boolean) {
    hasMore.value = value
  }

  function setCursor(value: string | null) {
    cursor.value = value
  }

  function setFilter(type: ActivityType | null) {
    filter.value = type
  }

  function setLoading(value: boolean) {
    isLoading.value = value
  }

  function openFeed() {
    isOpen.value = true
  }

  function closeFeed() {
    isOpen.value = false
  }

  function clearActivities() {
    activities.value.clear()
    order.value = []
  }

  return {
    // State
    activities,
    order,
    unreadCount,
    hasMore,
    cursor,
    filter,
    isLoading,
    isOpen,

    // Getters
    getActivities,
    unreadActivities,

    // Actions
    setActivities,
    appendActivities,
    addActivity,
    markActivityRead,
    markAllRead,
    setUnreadCount,
    setHasMore,
    setCursor,
    setFilter,
    setLoading,
    openFeed,
    closeFeed,
    clearActivities
  }
})
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/features/activity/stores/activityStore.ts
git commit -m "feat: add activity store"
```

---

### Task 11: Create Activity Service

**Files:**
- Create: `frontend/src/features/activity/services/activityService.ts`

- [ ] **Step 1: Create service file**

```typescript
/**
 * Activity Service - Business logic for activity feed
 * Following the pattern from messageService
 */

import { activityRepository } from '../repositories/activityRepository'
import { useActivityStore } from '../stores/activityStore'
import { useAuthStore } from '../../auth/stores/authStore'
import type { Activity, ActivityType } from '../types'

class ActivityService {
  private get store() {
    return useActivityStore()
  }

  private get userId() {
    const authStore = useAuthStore()
    return authStore.user?.id
  }

  async loadActivities(refresh = false) {
    const userId = this.userId
    if (!userId) return

    this.store.setLoading(true)

    try {
      const cursor = refresh ? undefined : this.store.cursor || undefined
      const filter = this.store.filter

      const response = await activityRepository.getFeed(userId, {
        cursor,
        limit: 50,
        type: filter || undefined,
        unreadOnly: false
      })

      if (refresh || !cursor) {
        this.store.setActivities(
          Object.values(response.activities),
          response.order
        )
      } else {
        this.store.appendActivities(
          Object.values(response.activities),
          response.order
        )
      }

      this.store.setUnreadCount(response.unreadCount)
      this.store.setHasMore(!!response.nextCursor)
      this.store.setCursor(response.nextCursor || null)
    } finally {
      this.store.setLoading(false)
    }
  }

  async loadMore() {
    if (!this.store.hasMore || this.store.isLoading) return
    await this.loadActivities()
  }

  async markRead(activityId: string) {
    const userId = this.userId
    if (!userId) return

    // Optimistic update
    this.store.markActivityRead(activityId)

    try {
      await activityRepository.markRead(userId, [activityId])
    } catch (error) {
      // Revert on error - reload activities
      await this.loadActivities(true)
      throw error
    }
  }

  async markAllRead() {
    const userId = this.userId
    if (!userId) return

    // Optimistic update
    this.store.markAllRead()

    try {
      await activityRepository.markAllRead(userId)
    } catch (error) {
      // Revert on error
      await this.loadActivities(true)
      throw error
    }
  }

  setFilter(type: ActivityType | null) {
    this.store.setFilter(type)
    this.store.clearActivities()
    this.store.setCursor(null)
    this.loadActivities(true)
  }

  openFeed() {
    this.store.openFeed()
    this.loadActivities(true)
  }

  closeFeed() {
    this.store.closeFeed()
  }

  // Handle incoming WebSocket activity
  handleNewActivity(activity: Activity) {
    this.store.addActivity(activity)
  }
}

export const activityService = new ActivityService()
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/features/activity/services/activityService.ts
git commit -m "feat: add activity service"
```

---

### Task 12: Create Activity Feature Index

**Files:**
- Create: `frontend/src/features/activity/index.ts`

- [ ] **Step 1: Create index file**

```typescript
/**
 * Activity Feature - Activity Feed for mentions, replies, and reactions
 */

export * from './types'
export { activityRepository } from './repositories/activityRepository'
export { activityService } from './services/activityService'
export { useActivityStore } from './stores/activityStore'
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/features/activity/index.ts
git commit -m "feat: export activity feature modules"
```

---

### Task 13: Install match-sorter Dependency

**Files:**
- Modify: `frontend/package.json`

- [ ] **Step 1: Install match-sorter**

Run: `cd frontend && npm install match-sorter`
Expected: Package installed successfully

- [ ] **Step 2: Commit**

```bash
git add frontend/package.json frontend/package-lock.json
git commit -m "chore: add match-sorter for fuzzy search"
```

---

### Task 14: Create ActivityFeed Component

**Files:**
- Create: `frontend/src/components/activity/ActivityFeed.vue`

- [ ] **Step 1: Create component file**

```vue
<template>
  <Transition name="slide">
    <div
      v-if="isOpen"
      class="fixed inset-y-0 right-0 w-[400px] bg-white dark:bg-gray-900 border-l border-gray-200 dark:border-gray-800 shadow-xl z-50 flex flex-col"
      role="dialog"
      aria-label="Activity feed"
      aria-modal="true"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-800">
        <div class="flex items-center gap-2">
          <Bell class="w-5 h-5" />
          <h2 class="text-lg font-semibold">
            Activity
            <span v-if="unreadCount > 0" class="ml-2 text-sm text-red-500">
              ({{ unreadCount }} unread)
            </span>
          </h2>
        </div>
        <div class="flex items-center gap-2">
          <button
            v-if="unreadCount > 0"
            class="text-sm text-blue-500 hover:text-blue-600 px-2 py-1"
            @click="handleMarkAllRead"
          >
            Mark all read
          </button>
          <button
            class="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded"
            aria-label="Close activity feed"
            @click="close"
          >
            <X class="w-5 h-5" />
          </button>
        </div>
      </div>

      <!-- Filters -->
      <ActivityFilters
        :model-value="filter"
        @update:model-value="handleFilterChange"
      />

      <!-- Activity List -->
      <div class="flex-1 overflow-y-auto p-4">
        <div v-if="isLoading && activities.length === 0" class="text-center py-8 text-gray-500">
          Loading...
        </div>

        <div v-else-if="activities.length === 0" class="text-center py-8 text-gray-500">
          <Inbox class="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>No activity yet</p>
          <p class="text-sm mt-1">Mentions, replies, and reactions will appear here</p>
        </div>

        <div v-else class="space-y-2">
          <ActivityItem
            v-for="activity in activities"
            :key="activity.id"
            :activity="activity"
            @click="handleActivityClick(activity)"
            @mark-read="handleMarkRead(activity.id)"
          />

          <!-- Load More -->
          <div v-if="hasMore" class="text-center py-4">
            <button
              class="text-blue-500 hover:text-blue-600 text-sm"
              :disabled="isLoading"
              @click="loadMore"
            >
              {{ isLoading ? 'Loading...' : 'Load more' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>

  <!-- Backdrop -->
  <Transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/20 z-40"
      @click="close"
    />
  </Transition>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { Bell, X, Inbox } from 'lucide-vue-next'
import { useActivityStore } from '../../features/activity/stores/activityStore'
import { activityService } from '../../features/activity/services/activityService'
import type { Activity, ActivityType } from '../../features/activity/types'
import ActivityItem from './ActivityItem.vue'
import ActivityFilters from './ActivityFilters.vue'

const store = useActivityStore()

const isOpen = computed(() => store.isOpen)
const activities = computed(() => store.getActivities)
const unreadCount = computed(() => store.unreadCount)
const hasMore = computed(() => store.hasMore)
const isLoading = computed(() => store.isLoading)
const filter = computed(() => store.filter)

// Close on escape key
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && isOpen.value) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

const close = () => {
  activityService.closeFeed()
}

const loadMore = () => {
  activityService.loadMore()
}

const handleFilterChange = (type: ActivityType | null) => {
  activityService.setFilter(type)
}

const handleMarkRead = async (activityId: string) => {
  try {
    await activityService.markRead(activityId)
  } catch (error) {
    console.error('Failed to mark activity as read:', error)
  }
}

const handleMarkAllRead = async () => {
  try {
    await activityService.markAllRead()
  } catch (error) {
    console.error('Failed to mark all as read:', error)
  }
}

const handleActivityClick = (activity: Activity) => {
  // Navigate to the relevant post
  if (activity.rootId) {
    // Thread reply - open thread
    window.location.href = `/channels/${activity.channelId}?thread=${activity.rootId}`
  } else {
    // Regular post
    window.location.href = `/channels/${activity.channelId}?post=${activity.postId}`
  }

  // Mark as read
  if (!activity.read) {
    handleMarkRead(activity.id)
  }
}
</script>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.2s ease-out;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease-out;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/activity/ActivityFeed.vue
git commit -m "feat: add ActivityFeed component"
```

---

### Task 15: Create ActivityItem Component

**Files:**
- Create: `frontend/src/components/activity/ActivityItem.vue`

- [ ] **Step 1: Create component file**

```vue
<template>
  <div
    class="flex items-start gap-3 p-3 rounded-lg cursor-pointer transition-colors"
    :class="{
      'bg-blue-50 dark:bg-blue-900/20': !activity.read,
      'hover:bg-gray-50 dark:hover:bg-gray-800': activity.read,
      'hover:bg-blue-100 dark:hover:bg-blue-900/30': !activity.read
    }"
    @click="$emit('click')"
  >
    <!-- Icon -->
    <ActivityIcon :type="activity.type" class="mt-0.5" />

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <div class="flex items-start justify-between gap-2">
        <p class="text-sm">
          <span class="font-medium">{{ activity.actorUsername }}</span>
          {{ actionText }}
        </p>
        <span class="text-xs text-gray-500 whitespace-nowrap">
          {{ formattedTime }}
        </span>
      </div>

      <!-- Message preview -->
      <p v-if="activity.message" class="text-sm text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">
        {{ activity.message }}
      </p>

      <!-- Reaction -->
      <p v-if="activity.reaction" class="text-sm mt-1">
        {{ activity.reaction }}
      </p>

      <!-- Location -->
      <p class="text-xs text-gray-500 mt-1">
        #{{ activity.channelName }} · {{ activity.teamName }}
      </p>
    </div>

    <!-- Unread indicator -->
    <div
      v-if="!activity.read"
      class="w-2 h-2 bg-blue-500 rounded-full mt-2"
      aria-label="Unread"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { formatDistanceToNow } from 'date-fns'
import type { Activity } from '../../features/activity/types'
import ActivityIcon from './ActivityIcon.vue'

const props = defineProps<{
  activity: Activity
}>()

defineEmits<{
  click: []
  markRead: []
}>()

const actionText = computed(() => {
  switch (props.activity.type) {
    case 'mention':
      return 'mentioned you'
    case 'reply':
      return 'replied to your message'
    case 'reaction':
      return `reacted ${props.activity.reaction || ''}`
    case 'dm':
      return 'sent you a message'
    case 'thread_reply':
      return 'replied in a thread'
    default:
      return 'interacted with you'
  }
})

const formattedTime = computed(() => {
  return formatDistanceToNow(props.activity.createdAt, { addSuffix: true })
})
</script>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/activity/ActivityItem.vue
git commit -m "feat: add ActivityItem component"
```

---

### Task 16: Create ActivityIcon Component

**Files:**
- Create: `frontend/src/components/activity/ActivityIcon.vue`

- [ ] **Step 1: Create component file**

```vue
<template>
  <div
    class="flex items-center justify-center w-8 h-8 rounded-full"
    :class="iconClasses"
  >
    <component :is="iconComponent" class="w-4 h-4" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  AtSign,
  MessageCircle,
  Heart,
  Mail,
  MessageSquare
} from 'lucide-vue-next'
import type { ActivityType } from '../../features/activity/types'

const props = defineProps<{
  type: ActivityType
}>()

const iconConfig: Record<ActivityType, { icon: any; bgClass: string; colorClass: string }> = {
  mention: {
    icon: AtSign,
    bgClass: 'bg-blue-100 dark:bg-blue-900/30',
    colorClass: 'text-blue-600 dark:text-blue-400'
  },
  reply: {
    icon: MessageCircle,
    bgClass: 'bg-green-100 dark:bg-green-900/30',
    colorClass: 'text-green-600 dark:text-green-400'
  },
  reaction: {
    icon: Heart,
    bgClass: 'bg-pink-100 dark:bg-pink-900/30',
    colorClass: 'text-pink-600 dark:text-pink-400'
  },
  dm: {
    icon: Mail,
    bgClass: 'bg-purple-100 dark:bg-purple-900/30',
    colorClass: 'text-purple-600 dark:text-purple-400'
  },
  thread_reply: {
    icon: MessageSquare,
    bgClass: 'bg-orange-100 dark:bg-orange-900/30',
    colorClass: 'text-orange-600 dark:text-orange-400'
  }
}

const config = computed(() => iconConfig[props.type])
const iconComponent = computed(() => config.value.icon)
const iconClasses = computed(() => `${config.value.bgClass} ${config.value.colorClass}`)
</script>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/activity/ActivityIcon.vue
git commit -m "feat: add ActivityIcon component"
```

---

### Task 17: Create ActivityFilters Component

**Files:**
- Create: `frontend/src/components/activity/ActivityFilters.vue`

- [ ] **Step 1: Create component file**

```vue
<template>
  <div class="flex gap-1 p-2 border-b border-gray-200 dark:border-gray-800 overflow-x-auto">
    <button
      v-for="filter in filters"
      :key="filter.value || 'all'"
      class="px-3 py-1 text-sm rounded-full whitespace-nowrap transition-colors"
      :class="{
        'bg-gray-900 text-white dark:bg-white dark:text-gray-900': modelValue === filter.value,
        'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700': modelValue !== filter.value
      }"
      @click="$emit('update:modelValue', filter.value)"
    >
      {{ filter.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
import type { ActivityType } from '../../features/activity/types'

interface Filter {
  label: string
  value: ActivityType | null
}

const filters: Filter[] = [
  { label: 'All', value: null },
  { label: 'Mentions', value: 'mention' },
  { label: 'Replies', value: 'reply' },
  { label: 'Reactions', value: 'reaction' },
  { label: 'DMs', value: 'dm' }
]

defineProps<{
  modelValue: ActivityType | null
}>()

defineEmits<{
  'update:modelValue': [value: ActivityType | null]
}>()
</script>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/activity/ActivityFilters.vue
git commit -m "feat: add ActivityFilters component"
```

---

### Task 18: Create Activity Socket Handlers

**Files:**
- Create: `frontend/src/features/activity/handlers/activitySocketHandlers.ts`

- [ ] **Step 1: Create handlers file**

```typescript
/**
 * Activity Socket Handlers - WebSocket event handlers for activity feed
 * Following the pattern from messageSocketHandlers
 */

import type { Activity } from '../types'
import { activityService } from '../services/activityService'

export function handleActivityCreated(data: any) {
  const activity: Activity = {
    id: data.id,
    type: data.type,
    actorId: data.actor_id,
    actorUsername: data.actor_username,
    actorAvatarUrl: data.actor_avatar_url,
    channelId: data.channel_id,
    channelName: data.channel_name,
    teamId: data.team_id,
    teamName: data.team_name,
    postId: data.post_id,
    rootId: data.root_id,
    message: data.message_text,
    reaction: data.reaction,
    read: false,
    createdAt: new Date()
  }

  activityService.handleNewActivity(activity)
}

export function handleActivityRead(data: any) {
  // Handle multi-device sync if needed
  // This could update the read status when another device marks activity as read
}
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/features/activity/handlers/activitySocketHandlers.ts
git commit -m "feat: add activity socket handlers"
```

---

### Task 19: Register Activity WebSocket Handler

**Files:**
- Modify: `frontend/src/core/websocket/registerHandlers.ts`

- [ ] **Step 1: Import activity handler**

Add to `frontend/src/core/websocket/registerHandlers.ts`:
```typescript
import { handleActivityCreated } from '../../features/activity/handlers/activitySocketHandlers'
```

- [ ] **Step 2: Register activity_created event handler**

Add to the `registerWebSocketHandlers` function:
```typescript
// Activity feed events
wsManager.on('activity_created', (event: WebSocketEvent) => {
  const data = JSON.parse((event as any).data)
  handleActivityCreated(data)
})
```

- [ ] **Step 3: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add frontend/src/core/websocket/registerHandlers.ts
git commit -m "feat: register activity WebSocket handler"
```

---

### Task 20: Integrate ActivityFeed into MainLayout

**Files:**
- Find and modify: Main layout file (likely `frontend/src/layouts/MainLayout.vue` or similar)

- [ ] **Step 1: Find the main layout**

Run: `find frontend/src -name "*.vue" | xargs grep -l "sidebar\|Sidebar" | head -5`

- [ ] **Step 2: Add ActivityFeed import and Activity button**

Add to the layout file:
```vue
<script setup>
import { Bell } from 'lucide-vue-next'
import ActivityFeed from '../components/activity/ActivityFeed.vue'
import { activityService } from '../features/activity/services/activityService'
import { useActivityStore } from '../features/activity/stores/activityStore'
import { computed } from 'vue'

const activityStore = useActivityStore()
const unreadCount = computed(() => activityStore.unreadCount)

const openActivityFeed = () => {
  activityService.openFeed()
}
</script>

<template>
  <!-- In the sidebar or header -->
  <button
    class="relative p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded"
    @click="openActivityFeed"
  >
    <Bell class="w-5 h-5" />
    <span
      v-if="unreadCount > 0"
      class="absolute top-0 right-0 bg-red-500 text-white text-xs rounded-full min-w-[18px] h-[18px] flex items-center justify-center px-1"
    >
      {{ unreadCount > 99 ? '99+' : unreadCount }}
    </span>
  </button>

  <!-- Activity Feed Panel -->
  <ActivityFeed />
</template>
```

- [ ] **Step 3: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add frontend/src/layouts/MainLayout.vue  # or relevant file
git commit -m "feat: integrate ActivityFeed into main layout"
```

---

## Phase 2b: Breadcrumbs and Quick Switcher (4 days)

### Task 21: Create BreadcrumbBar Component

**Files:**
- Create: `frontend/src/components/navigation/BreadcrumbBar.vue`

- [ ] **Step 1: Create component file**

```vue
<template>
  <nav aria-label="Breadcrumb" class="flex items-center gap-1 text-sm">
    <template v-for="(segment, index) in segments" :key="index">
      <!-- Separator (not for first item) -->
      <ChevronRight
        v-if="index > 0"
        class="w-4 h-4 text-gray-400"
      />

      <!-- Segment -->
      <component
        :is="segment.to ? 'router-link' : 'span'"
        v-bind="segment.to ? { to: segment.to } : {}"
        class="flex items-center gap-1 px-2 py-1 rounded transition-colors"
        :class="{
          'hover:bg-gray-100 dark:hover:bg-gray-800': segment.to,
          'text-gray-900 dark:text-gray-100 font-medium': index === segments.length - 1,
          'text-gray-600 dark:text-gray-400': index !== segments.length - 1
        }"
        :aria-current="index === segments.length - 1 ? 'location' : undefined"
      >
        <component
          :is="getIcon(segment.icon)"
          v-if="segment.icon"
          class="w-4 h-4"
        />
        <span class="truncate max-w-[150px]">{{ segment.label }}</span>
      </component>
    </template>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ChevronRight, Users, Hash, MessageSquare, User } from 'lucide-vue-next'
import type { RouteLocationRaw } from 'vue-router'

interface BreadcrumbSegment {
  label: string
  icon?: string
  to?: RouteLocationRaw
}

const props = defineProps<{
  segments: BreadcrumbSegment[]
}>()

const iconMap: Record<string, any> = {
  Users,
  Hash,
  MessageSquare,
  User
}

function getIcon(name?: string) {
  return name ? iconMap[name] : undefined
}
</script>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/navigation/BreadcrumbBar.vue
git commit -m "feat: add BreadcrumbBar component"
```

---

### Task 22: Add Breadcrumbs to ChannelView

**Files:**
- Find: `frontend/src/views/main/ChannelView.vue` or similar
- Modify: Add breadcrumb integration

- [ ] **Step 1: Add breadcrumbs to channel view**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import BreadcrumbBar from '../../components/navigation/BreadcrumbBar.vue'
import { useTeamStore } from '../../features/teams/stores/teamStore'
import { useChannelStore } from '../../features/channels/stores/channelStore'

const route = useRoute()
const teamStore = useTeamStore()
const channelStore = useChannelStore()

const channelId = computed(() => route.params.channelId as string)
const threadId = computed(() => route.query.thread as string | undefined)

const currentTeam = computed(() => {
  const channel = channelStore.getChannelById(channelId.value)
  if (!channel) return null
  return teamStore.allTeams.find(t => t.id === channel.teamId) || null
})

const currentChannel = computed(() => channelStore.getChannelById(channelId.value))

const breadcrumbs = computed(() => {
  const segments = []

  // Team segment
  if (currentTeam.value) {
    segments.push({
      label: currentTeam.value.name,
      icon: 'Users',
      to: `/teams/${currentTeam.value.id}`
    })
  }

  // Channel segment
  if (currentChannel.value) {
    const channelSegment = {
      label: `#${currentChannel.value.name}`,
      icon: 'Hash'
    }

    // If in thread view, channel is clickable
    if (threadId.value) {
      channelSegment.to = `/channels/${currentChannel.value.id}`
    }

    segments.push(channelSegment)
  }

  // Thread segment
  if (threadId.value) {
    segments.push({
      label: 'Thread',
      icon: 'MessageSquare'
    })
  }

  return segments
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Breadcrumb Header -->
    <div class="flex items-center px-4 py-2 border-b border-gray-200 dark:border-gray-800">
      <BreadcrumbBar :segments="breadcrumbs" />
    </div>

    <!-- Rest of channel view content -->
    ...
  </div>
</template>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/views/main/ChannelView.vue
git commit -m "feat: integrate breadcrumbs into ChannelView"
```

---

### Task 23: Create QuickSwitcherModal Component

**Files:**
- Create: `frontend/src/components/navigation/QuickSwitcherModal.vue`

- [ ] **Step 1: Create component file**

```vue
<template>
  <Transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 flex items-start justify-center pt-[20vh] bg-black/50"
      @click="close"
    >
      <div
        class="w-full max-w-lg bg-white dark:bg-gray-900 rounded-lg shadow-2xl overflow-hidden"
        @click.stop
      >
        <!-- Search Input -->
        <div class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-800">
          <Search class="w-5 h-5 text-gray-400" />
          <input
            ref="inputRef"
            v-model="query"
            type="text"
            placeholder="Jump to..."
            class="flex-1 bg-transparent outline-none text-lg"
            @keydown="handleKeydown"
          >
          <kbd class="px-2 py-1 text-xs bg-gray-100 dark:bg-gray-800 rounded">
            ESC
          </kbd>
        </div>

        <!-- Results -->
        <div class="max-h-[400px] overflow-y-auto py-2">
          <div v-if="filteredItems.length === 0" class="px-4 py-8 text-center text-gray-500">
            <p v-if="query">No results found</p>
            <p v-else>Start typing to search channels, teams, and recent items</p>
          </div>

          <template v-else>
            <!-- Recent section when no query -->
            <div v-if="!query && recentItems.length > 0" class="mb-2">
              <div class="px-4 py-1 text-xs font-medium text-gray-500 uppercase">
                Recent
              </div>
              <QuickSwitcherItem
                v-for="(item, index) in recentItems.slice(0, 5)"
                :key="item.id"
                :item="item"
                :selected="selectedIndex === index"
                @click="selectItem(item)"
              />
            </div>

            <!-- Search results -->
            <div v-if="query">
              <QuickSwitcherItem
                v-for="(item, index) in filteredItems"
                :key="item.id"
                :item="item"
                :selected="selectedIndex === index"
                @click="selectItem(item)"
              />
            </div>
          </template>
        </div>

        <!-- Footer -->
        <div class="flex items-center gap-4 px-4 py-2 text-xs text-gray-500 border-t border-gray-200 dark:border-gray-800">
          <div class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">↑↓</kbd>
            <span>Navigate</span>
          </div>
          <div class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">↵</kbd>
            <span>Select</span>
          </div>
          <div class="flex items-center gap-1">
            <kbd class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 rounded">ESC</kbd>
            <span>Close</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { Search } from 'lucide-vue-next'
import { matchSorter } from 'match-sorter'
import QuickSwitcherItem from './QuickSwitcherItem.vue'
import type { QuickSwitcherItem as QuickSwitcherItemType } from '../../composables/useQuickSwitcher'

const props = defineProps<{
  isOpen: boolean
  items: QuickSwitcherItemType[]
  recentItems: QuickSwitcherItemType[]
}>()

const emit = defineEmits<{
  close: []
}>()

const router = useRouter()
const query = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement>()

const filteredItems = computed(() => {
  if (!query.value) return []
  return matchSorter(props.items, query.value, {
    keys: ['name', 'subtitle'],
    threshold: matchSorter.rankings.CONTAINS
  }).slice(0, 10)
})

const displayedItems = computed(() => {
  return query.value ? filteredItems.value : props.recentItems.slice(0, 5)
})

// Reset selection when query changes
watch(query, () => {
  selectedIndex.value = 0
})

// Focus input when opened
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      inputRef.value?.focus()
      query.value = ''
      selectedIndex.value = 0
    })
  }
})

const close = () => {
  emit('close')
}

const selectItem = (item: QuickSwitcherItemType) => {
  router.push(item.to)
  close()
}

const handleKeydown = (e: KeyboardEvent) => {
  const items = displayedItems.value

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % items.length
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + items.length) % items.length
      break
    case 'Enter':
      e.preventDefault()
      if (items[selectedIndex.value]) {
        selectItem(items[selectedIndex.value])
      }
      break
    case 'Escape':
      e.preventDefault()
      close()
      break
  }
}
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
```

- [ ] **Step 2: Create QuickSwitcherItem subcomponent**

Create `frontend/src/components/navigation/QuickSwitcherItem.vue`:

```vue
<template>
  <div
    class="flex items-center gap-3 px-4 py-2 cursor-pointer"
    :class="{
      'bg-blue-50 dark:bg-blue-900/20': selected,
      'hover:bg-gray-50 dark:hover:bg-gray-800': !selected
    }"
    @click="$emit('click')"
  >
    <component :is="iconComponent" class="w-5 h-5 text-gray-500" />
    <div class="flex-1 min-w-0">
      <div class="text-sm font-medium truncate">{{ item.name }}</div>
      <div v-if="item.subtitle" class="text-xs text-gray-500 truncate">
        {{ item.subtitle }}
      </div>
    </div>
    <kbd v-if="selected" class="px-1.5 py-0.5 text-xs bg-gray-200 dark:bg-gray-700 rounded">
      ↵
    </kbd>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Hash, Lock, User, MessageSquare, Users } from 'lucide-vue-next'
import type { QuickSwitcherItem } from '../../composables/useQuickSwitcher'

const props = defineProps<{
  item: QuickSwitcherItem
  selected: boolean
}>()

defineEmits<{
  click: []
}>()

const iconMap: Record<string, any> = {
  Hash,
  Lock,
  User,
  MessageSquare,
  Users
}

const iconComponent = computed(() => iconMap[props.item.icon] || Hash)
</script>
```

- [ ] **Step 3: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add frontend/src/components/navigation/QuickSwitcherModal.vue frontend/src/components/navigation/QuickSwitcherItem.vue
git commit -m "feat: add QuickSwitcherModal component"
```

---

### Task 24: Create useQuickSwitcher Composable

**Files:**
- Create: `frontend/src/composables/useQuickSwitcher.ts`

- [ ] **Step 1: Create composable file**

```typescript
/**
 * Quick Switcher Composable - Manages Cmd+K quick navigation
 */

import { ref, computed } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { useTeamStore } from '../features/teams/stores/teamStore'
import { useChannelStore } from '../features/channels/stores/channelStore'

export interface QuickSwitcherItem {
  id: string
  type: 'channel' | 'dm' | 'thread' | 'team'
  name: string
  subtitle?: string
  icon: string
  to: RouteLocationRaw
}

const isOpen = ref(false)
const recentItemIds = ref<string[]>([])

export function useQuickSwitcher() {
  const teamStore = useTeamStore()
  const channelStore = useChannelStore()

  const allItems = computed<QuickSwitcherItem[]>(() => {
    const items: QuickSwitcherItem[] = []

    // Add all channels from all teams
    for (const channel of channelStore.allChannels) {
      const team = teamStore.allTeams.find(t => t.id === channel.teamId)
      items.push({
        id: `channel-${channel.id}`,
        type: 'channel',
        name: channel.name,
        subtitle: team?.name,
        icon: channel.type === 'public' ? 'Hash' : 'Lock',
        to: `/channels/${channel.id}`
      })
    }

    // Add teams
    for (const team of teamStore.allTeams) {
      items.push({
        id: `team-${team.id}`,
        type: 'team',
        name: team.name,
        icon: 'Users',
        to: `/teams/${team.id}`
      })
    }

    return items
  })

  const recentItems = computed<QuickSwitcherItem[]>(() => {
    return recentItemIds.value
      .map(id => allItems.value.find(item => item.id === id))
      .filter((item): item is QuickSwitcherItem => item !== undefined)
  })

  const open = () => {
    isOpen.value = true
  }

  const close = () => {
    isOpen.value = false
  }

  const toggle = () => {
    isOpen.value = !isOpen.value
  }

  const addRecentItem = (id: string) => {
    // Remove if exists, add to front, keep max 10
    recentItemIds.value = [id, ...recentItemIds.value.filter(i => i !== id)].slice(0, 10)
    // Persist to localStorage
    localStorage.setItem('quickSwitcherRecent', JSON.stringify(recentItemIds.value))
  }

  // Load from localStorage on init
  const loadRecent = () => {
    try {
      const saved = localStorage.getItem('quickSwitcherRecent')
      if (saved) {
        recentItemIds.value = JSON.parse(saved)
      }
    } catch {
      // Ignore localStorage errors
    }
  }

  // Initialize
  loadRecent()

  return {
    isOpen,
    allItems,
    recentItems,
    open,
    close,
    toggle,
    addRecentItem
  }
}
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/composables/useQuickSwitcher.ts
git commit -m "feat: add useQuickSwitcher composable"
```

---

### Task 25: Integrate Quick Switcher into MainLayout

**Files:**
- Modify: Main layout file

- [ ] **Step 1: Add Quick Switcher to layout**

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import QuickSwitcherModal from '../components/navigation/QuickSwitcherModal.vue'
import { useQuickSwitcher } from '../composables/useQuickSwitcher'

const quickSwitcher = useQuickSwitcher()

// Global Cmd+K handler
const handleKeydown = (e: KeyboardEvent) => {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault()
    quickSwitcher.toggle()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <!-- Rest of layout -->

  <!-- Quick Switcher -->
  <QuickSwitcherModal
    :is-open="quickSwitcher.isOpen"
    :items="quickSwitcher.allItems"
    :recent-items="quickSwitcher.recentItems"
    @close="quickSwitcher.close"
  />
</template>
```

- [ ] **Step 2: Run TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add frontend/src/layouts/MainLayout.vue
git commit -m "feat: integrate Quick Switcher with Cmd+K shortcut"
```

---

### Task 26: Create Backend Integration Tests

**Files:**
- Create: `backend/tests/api_activity.rs`

- [ ] **Step 1: Create test file**

```rust
//! Integration tests for Activity Feed API

mod common;
use common::spawn_app;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn test_get_activity_feed_requires_auth() {
    let app = spawn_app().await;

    let response = app
        .api_client
        .get(&format!("{}/api/v4/users/test-user-id/activity", app.address))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_get_activity_feed_returns_activities() {
    let app = spawn_app().await;

    // 1. Create Organization
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind("Test Org")
        .execute(&app.db_pool)
        .await
        .expect("Failed to create organization");

    // 2. Register & Login User
    let user_data = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "Password123!",
        "display_name": "Test User",
        "org_id": org_id
    });

    app.api_client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&user_data)
        .send()
        .await
        .expect("Failed to register.");

    let login_data = serde_json::json!({
        "email": "test@example.com",
        "password": "Password123!"
    });

    let login_res = app
        .api_client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&login_data)
        .send()
        .await
        .expect("Failed to login.");

    let login_body: Value = login_res.json().await.unwrap();
    let token = login_body["token"].as_str().unwrap();
    let user_id = login_body["user"]["id"].as_str().unwrap();
    let user_uuid = Uuid::parse_str(user_id).unwrap();

    // 3. Create Actor User
    let actor_data = serde_json::json!({
        "username": "actor",
        "email": "actor@example.com",
        "password": "Password123!",
        "display_name": "Actor User",
        "org_id": org_id
    });

    app.api_client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&actor_data)
        .send()
        .await
        .expect("Failed to register actor.");

    let actor_login = app
        .api_client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&serde_json::json!({
            "email": "actor@example.com",
            "password": "Password123!"
        }))
        .send()
        .await
        .expect("Failed to login actor.");

    let actor_body: Value = actor_login.json().await.unwrap();
    let actor_id = actor_body["user"]["id"].as_str().unwrap();
    let actor_uuid = Uuid::parse_str(actor_id).unwrap();

    // 4. Create Team
    let team_id = Uuid::new_v4();
    sqlx::query("INSERT INTO teams (id, org_id, name, display_name, allow_open_invite) VALUES ($1, $2, $3, $4, $5)")
        .bind(team_id)
        .bind(org_id)
        .bind("test-team")
        .bind("Test Team")
        .bind(true)
        .execute(&app.db_pool)
        .await
        .expect("Failed to insert team");

    // Add both users to team
    sqlx::query("INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(team_id)
        .bind(user_uuid)
        .bind("member")
        .execute(&app.db_pool)
        .await
        .expect("Failed to add user to team");

    sqlx::query("INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(team_id)
        .bind(actor_uuid)
        .bind("member")
        .execute(&app.db_pool)
        .await
        .expect("Failed to add actor to team");

    // 5. Create Channel
    let channel_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO channels (id, team_id, name, display_name, type, creator_id) VALUES ($1, $2, $3, $4, $5::channel_type, $6)",
    )
    .bind(channel_id)
    .bind(team_id)
    .bind("test-channel")
    .bind("Test Channel")
    .bind("public")
    .bind(user_uuid)
    .execute(&app.db_pool)
    .await
    .expect("Failed to insert channel");

    // Add user to channel
    sqlx::query("INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(channel_id)
        .bind(user_uuid)
        .bind("member")
        .execute(&app.db_pool)
        .await
        .expect("Failed to add user to channel");

    // 6. Create activity record
    let post_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO activities (user_id, type, actor_id, channel_id, team_id, post_id, message_text, read)
         VALUES ($1, 'mention', $2, $3, $4, $5, $6, false)"
    )
    .bind(user_uuid)
    .bind(actor_uuid)
    .bind(channel_id)
    .bind(team_id)
    .bind(post_id)
    .bind("@testuser hello!")
    .execute(&app.db_pool)
    .await
    .expect("Failed to insert activity");

    // 7. Get activity feed
    let response = app
        .api_client
        .get(&format!("{}/api/v4/users/{}/activity", app.address, user_id))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to get activity feed");

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert!(body["order"].as_array().unwrap().len() > 0);
    assert!(body["unread_count"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_cannot_view_other_users_activity() {
    let app = spawn_app().await;

    // Create first user and login
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind("Test Org")
        .execute(&app.db_pool)
        .await
        .expect("Failed to create organization");

    let user1_data = serde_json::json!({
        "username": "user1",
        "email": "user1@example.com",
        "password": "Password123!",
        "display_name": "User 1",
        "org_id": org_id
    });

    app.api_client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&user1_data)
        .send()
        .await
        .expect("Failed to register user1.");

    let login1 = app
        .api_client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&serde_json::json!({ "email": "user1@example.com", "password": "Password123!" }))
        .send()
        .await
        .expect("Failed to login user1.");

    let body1: Value = login1.json().await.unwrap();
    let token1 = body1["token"].as_str().unwrap();
    let user1_id = body1["user"]["id"].as_str().unwrap();

    // Create second user
    let user2_data = serde_json::json!({
        "username": "user2",
        "email": "user2@example.com",
        "password": "Password123!",
        "display_name": "User 2",
        "org_id": org_id
    });

    app.api_client
        .post(&format!("{}/api/v1/auth/register", app.address))
        .json(&user2_data)
        .send()
        .await
        .expect("Failed to register user2.");

    let login2 = app
        .api_client
        .post(&format!("{}/api/v1/auth/login", app.address))
        .json(&serde_json::json!({ "email": "user2@example.com", "password": "Password123!" }))
        .send()
        .await
        .expect("Failed to login user2.");

    let body2: Value = login2.json().await.unwrap();
    let user2_id = body2["user"]["id"].as_str().unwrap();

    // Try to access user2's activity feed as user1
    let response = app
        .api_client
        .get(&format!("{}/api/v4/users/{}/activity", app.address, user2_id))
        .bearer_auth(token1)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 403);
}
```

- [ ] **Step 2: Run tests to verify they compile**

Run: `cd backend && cargo test --test api_activity --no-run`
Expected: Tests compile successfully

- [ ] **Step 3: Commit**

```bash
git add backend/tests/api_activity.rs
git commit -m "test: add activity feed integration tests"
```

---

### Task 27: Final Verification

- [ ] **Step 1: Run backend tests**

Run: `cd backend && cargo test`
Expected: All tests pass (except possibly pre-existing failures like S3 tests)

- [ ] **Step 2: Run frontend TypeScript check**

Run: `cd frontend && npx vue-tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Build frontend**

Run: `cd frontend && npm run build`
Expected: Build succeeds

- [ ] **Step 4: Final commit**

```bash
git commit -m "feat: complete Phase 2 - Activity Feed and Breadcrumbs Navigation

- Add Activity Feed with real-time updates for mentions, replies, reactions
- Add Breadcrumbs navigation showing Team > Channel > Thread hierarchy
- Add Quick Switcher (Cmd+K) for fuzzy-search navigation
- Includes full backend API, frontend components, and integration tests
"
```

---

## Plan Notes

### Scope Adjustments from Spec
1. **Recent Items API**: The `GET /api/v4/users/{user_id}/recent-items` endpoint is NOT implemented in this plan. The Quick Switcher uses client-side data from existing stores (`channelStore.allChannels`, `teamStore.allTeams`) and localStorage for recent items.
2. **DM Activity Type**: The `DM` activity type is defined but not auto-created. DMs use the existing `mention` activity when users are messaged in DM channels, or a separate notification system can be added in Phase 3.

### Implementation Notes
1. **Activity Creation**: Uses service helper functions (`create_mention_activity`, `create_reply_activity`, etc.) rather than inline SQL
2. **WebSocket Handlers**: Activity handler registration added as explicit Task 19
3. **Store APIs**: Uses actual store getters (`allChannels`, `allTeams`, `getChannelById`) verified from codebase
4. **Event Types**: Located in `backend/src/realtime/events.rs`
5. **Test Pattern**: Follows existing test structure using `spawn_app()` and direct SQL setup
6. **Reaction Integration**: Add activity creation after the reaction INSERT query in `add_reaction` function

## Plan Review

After completing this plan, dispatch the plan-document-reviewer subagent to review:
1. Task granularity (bite-sized, 2-5 minutes each)
2. File paths are exact and correct
3. Code snippets are complete and runnable
4. Test commands have expected outputs
5. Commits follow conventional commit format
6. No missing steps or dependencies

Review context to provide:
- Plan path: `docs/superpowers/plans/2026-03-18-activity-feed-breadcrumbs.md`
- Spec path: `docs/superpowers/specs/2026-03-18-activity-feed-breadcrumbs-design.md`
- This is Phase 2 of WebUI Enhancement Initiative
- Backend: Rust/Axum/SQLx with existing thread/reaction infrastructure
- Frontend: Vue 3 + TypeScript + Pinia with feature-based architecture
- WebSocket handler registration is in `frontend/src/core/websocket/registerHandlers.ts`
- Note: Recent-items API endpoint removed from scope - Quick Switcher uses client-side data
