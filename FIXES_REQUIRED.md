# Required Fixes for RustChat UI

This document contains a prioritized list of fixes required to match SPEC.md requirements.
Each fix includes the file path, current state, required changes, and code examples.

---

## 🔴 CRITICAL FIXES

### 1. Error Code Compatibility - Mattermost-compatible Error IDs

**Files to Modify:**
- `backend/src/error/mod.rs`
- `backend/src/api/v4/posts.rs` (lines 1272-1293)

**Current Problem:**
Error responses use generic codes like `BAD_REQUEST`, `FORBIDDEN` without Mattermost-compatible `id` fields.

**Required Changes:**

#### Step 1: Update ErrorBody struct in `backend/src/error/mod.rs` (line 53-65)
```rust
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub id: String,           // ADD THIS FIELD - Mattermost-compatible error ID
    pub message: String,
    pub detailed_error: String, // ADD THIS FIELD
    pub status_code: u16,     // ADD THIS FIELD
    pub request_id: String,   // ADD THIS FIELD
}
```

#### Step 2: Add Mattermost-compatible error IDs to AppError
Add new error variants or extend existing ones to carry Mattermost error IDs:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    // ... existing variants
    
    #[error("Unauthorized")]
    SessionExpired,  // Returns api.context.session_expired.app_error
    
    #[error("Forbidden")]
    PermissionsError, // Returns api.context.permissions.app_error
    
    #[error("Edit time limit expired")]
    PostEditTimeLimitExpired, // Returns api.post.update_post.permissions_time_limit.app_error
}
```

#### Step 3: Implement Mattermost-compatible error mapping
```rust
impl AppError {
    /// Get Mattermost-compatible error ID
    pub fn mm_error_id(&self) -> &'static str {
        match self {
            AppError::SessionExpired | AppError::Unauthorized(_) => 
                "api.context.session_expired.app_error",
            AppError::PermissionsError | AppError::Forbidden(_) => 
                "api.context.permissions.app_error",
            AppError::PostEditTimeLimitExpired => 
                "api.post.update_post.permissions_time_limit.app_error",
            _ => "api.error.generic",
        }
    }
}
```

#### Step 4: Update IntoResponse to include all required fields
```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();
        let request_id = generate_request_id(); // Generate unique request ID

        let body = ErrorResponse {
            error: ErrorBody {
                id: self.mm_error_id().to_string(),
                message: message.clone(),
                detailed_error: message,
                status_code: status.as_u16(),
                request_id,
            },
        };

        (status, Json(body)).into_response()
    }
}
```

#### Step 5: Update edit policy errors in `backend/src/api/v4/posts.rs` (lines 1272-1293)
```rust
// CURRENT CODE (lines 1272-1293):
if post_user_id != acting_user_id {
    return Err(AppError::Forbidden("Cannot edit others' posts".to_string()));
}
if message != original_message {
    let post_edit_time_limit_seconds = load_post_edit_time_limit_seconds(state).await?;
    if post_edit_time_limit_seconds == 0 {
        return Err(AppError::BadRequest(
            "Message editing is disabled by server policy".to_string(),
        ));
    }
    if post_edit_time_limit_seconds > 0 {
        let post_age_seconds = Utc::now()
            .signed_duration_since(post_created_at)
            .num_seconds();
        if post_age_seconds >= post_edit_time_limit_seconds {
            return Err(AppError::BadRequest(format!(
                "Message edit window expired after {} seconds",
                post_edit_time_limit_seconds
            )));
        }
    }
}

// REPLACE WITH:
if post_user_id != acting_user_id {
    return Err(AppError::PermissionsError); // or Forbidden with proper ID mapping
}
if message != original_message {
    let post_edit_time_limit_seconds = load_post_edit_time_limit_seconds(state).await?;
    if post_edit_time_limit_seconds == 0 {
        return Err(AppError::PostEditTimeLimitExpired); // Use specific variant
    }
    if post_edit_time_limit_seconds > 0 {
        let post_age_seconds = Utc::now()
            .signed_duration_since(post_created_at)
            .num_seconds();
        if post_age_seconds >= post_edit_time_limit_seconds {
            return Err(AppError::PostEditTimeLimitExpired); // Use specific variant
        }
    }
}
```

---

### 2. Frontend Edit Policy Consumption

**Files to Create/Modify:**
- `frontend-solid/src/api/config.ts` (CREATE)
- `frontend-solid/src/stores/config.ts` (CREATE)
- `frontend-solid/src/components/messages/MessageActions.tsx` (MODIFY)
- `frontend-solid/src/components/messages/Message.tsx` (MODIFY)

**Current Problem:**
Frontend never calls `/api/v4/config/client?format=old`, so edit policy is not enforced in UI.

**Required Changes:**

#### Step 1: Create `frontend-solid/src/api/config.ts`
```typescript
// ============================================
// Config API - Server Configuration Client
// ============================================

import { client } from './client';

export interface ClientConfig {
  AllowEditPost: 'true' | 'false' | 'always' | 'never' | 'time_limit';
  PostEditTimeLimit: string; // seconds as string
  EnableUserTypingMessages: string;
  // ... other config fields as needed
}

export async function getClientConfig(): Promise<ClientConfig> {
  const response = await client.get<ClientConfig>(
    '/api/v4/config/client?format=old'
  );
  return response.data;
}
```

#### Step 2: Create `frontend-solid/src/stores/config.ts`
```typescript
// ============================================
// Config Store - Server Configuration State
// ============================================

import { createStore } from 'solid-js/store';
import { createEffect } from 'solid-js';
import { getClientConfig } from '../api/config';

export interface ConfigState {
  allowEditPost: 'always' | 'never' | 'time_limit';
  postEditTimeLimitSeconds: number;
  isLoading: boolean;
  error: string | null;
}

const [configState, setConfigState] = createStore<ConfigState>({
  allowEditPost: 'always',
  postEditTimeLimitSeconds: -1,
  isLoading: false,
  error: null,
});

export async function loadClientConfig(): Promise<void> {
  setConfigState('isLoading', true);
  setConfigState('error', null);
  
  try {
    const config = await getClientConfig();
    
    // Parse AllowEditPost
    const allowEdit = config.AllowEditPost;
    let editMode: 'always' | 'never' | 'time_limit';
    
    if (allowEdit === 'never' || allowEdit === 'false') {
      editMode = 'never';
    } else if (allowEdit === 'time_limit') {
      editMode = 'time_limit';
    } else {
      editMode = 'always';
    }
    
    setConfigState({
      allowEditPost: editMode,
      postEditTimeLimitSeconds: parseInt(config.PostEditTimeLimit, 10) || -1,
      isLoading: false,
    });
  } catch (err) {
    setConfigState('error', 'Failed to load config');
    setConfigState('isLoading', false);
  }
}

// Check if a post can be edited based on policy
export function canEditPost(postCreatedAt: string | Date): boolean {
  const mode = configState.allowEditPost;
  
  if (mode === 'never') {
    return false;
  }
  
  if (mode === 'always') {
    return true;
  }
  
  // time_limited mode
  const limitSeconds = configState.postEditTimeLimitSeconds;
  if (limitSeconds <= 0) {
    return false; // 0 or negative means disabled
  }
  
  const createdAt = new Date(postCreatedAt).getTime();
  const now = Date.now();
  const ageSeconds = Math.floor((now - createdAt) / 1000);
  
  return ageSeconds < limitSeconds;
}

export const configStore = {
  state: configState,
  loadClientConfig,
  canEditPost,
};
```

#### Step 3: Load config on app startup
Modify `frontend-solid/src/App.tsx` (in RouterLayout onMount):
```typescript
import { loadClientConfig } from './stores/config';

function RouterLayout(props: RouteSectionProps) {
  onMount(() => {
    setupInterceptors();
    loadClientConfig(); // ADD THIS LINE
  });
  // ... rest of component
}
```

#### Step 4: Update MessageActions.tsx to check edit policy
In `frontend-solid/src/components/messages/MessageActions.tsx` (around line 327-334):
```typescript
import { configStore, canEditPost } from '../../stores/config';

// In the component render:
<Show when={props.message.userId === authStore.user()?.id && canEditPost(props.message.createdAt)}>
  <button aria-label="Edit message" onClick={...}>
    {/* Edit icon */}
  </button>
</Show>
```

---

### 3. Header Notification Bell - Connect to Unread Store

**File to Modify:**
- `frontend-solid/src/components/layout/Header.tsx` (lines 59-64, 136-143)

**Current Problem:**
Uses hardcoded mock data:
```typescript
const notifications = () => [
  { id: '1', title: 'New mention', message: 'Alice mentioned you in #general', read: false, time: '2m ago' },
  { id: '2', title: 'Direct message', message: 'Bob sent you a message', read: false, time: '15m ago' },
];
const unreadCount = () => notifications().filter((n) => !n.read).length;
```

**Required Changes:**
```typescript
// REPLACE lines 59-64 with:
import { totalUnreadCount, totalMentionCount } from '../../stores/unreads';

// Remove mock notifications() array
// Use actual unread counts:
const unreadCount = totalUnreadCount; // This is already a memo from unreads store
const mentionCount = totalMentionCount;
```

Update the bell icon badge (around line 141):
```typescript
// CURRENT:
<Show when={unreadCount() > 0}>
  <span class="badge">{unreadCount()}</span>
</Show>

// Can remain similar, but now using real data:
<Show when={unreadCount() > 0}>
  <span class="absolute -top-1 -right-1 w-5 h-5 bg-brand text-white text-xs rounded-full flex items-center justify-center">
    {unreadCount() > 99 ? '99+' : unreadCount()}
  </span>
</Show>
```

---

### 4. MobileNav Badge - Connect to Unread Store

**File to Modify:**
- `frontend-solid/src/components/layout/MobileNav.tsx` (line 44)

**Current Problem:**
```typescript
badge: () => 2, // Mock unread count
```

**Required Changes:**
```typescript
import { totalUnreadCount } from '../../stores/unreads';

// Replace line 44:
badge: totalUnreadCount, // Remove the mock, use actual store
```

---

### 5. Fix Unread Store Mutations - Use SolidJS Setters

**File to Modify:**
- `frontend-solid/src/hooks/useWebSocket.ts` (lines 211-220, 243-246)

**Current Problem:**
Direct store mutation breaks SolidJS reactivity:
```typescript
// Line 245-246 - BAD:
unreadStore.channelUnreads[unread.channel_id] = unread.msg_count ?? 0;
unreadStore.channelMentions[unread.channel_id] = unread.mention_count ?? 0;
```

**Required Changes:**
```typescript
// Import the setter functions:
import { setChannelUnreads, setChannelMentions } from '../stores/unreads';

// REPLACE the handleUnreadCountsUpdated function:
function handleUnreadCountsUpdated(data: UnreadCountsEvent) {
  const rawUnread = data;
  if (rawUnread.channel_id) {
    const unread = rawUnread as { channel_id: string; msg_count?: number; mention_count?: number };
    // USE SETTERS instead of direct mutation:
    setChannelUnreads(unread.channel_id, unread.msg_count ?? 0);
    setChannelMentions(unread.channel_id, unread.mention_count ?? 0);
  }
}
```

**Note:** You may need to export these setters from `frontend-solid/src/stores/unreads.ts` if not already available.

---

## 🟡 MEDIUM PRIORITY

### 6. Verify Reaction Toggle Logic

**File to Check:**
- `frontend-solid/src/components/messages/Reactions.tsx` or wherever reaction button is rendered

**Requirement:**
- First click: add reaction
- Second click by same user: remove reaction
- Counts should decrement correctly while preserving others

**What to Verify:**
```typescript
// The component should check if current user already reacted:
const hasUserReacted = (emoji: string) => {
  const userId = authStore.user()?.id;
  return message.reactions?.some(r => r.emoji === emoji && r.userId === userId);
};

const handleReactionClick = (emoji: string) => {
  if (hasUserReacted(emoji)) {
    removeReaction(message.id, emoji);
  } else {
    addReaction(message.id, emoji);
  }
};
```

---

## ✅ VERIFICATION CHECKLIST

After implementing fixes, verify:

### Error Codes
- [ ] Edit outside time limit returns `400` with `api.post.update_post.permissions_time_limit.app_error`
- [ ] Edit others' post returns `403` with `api.context.permissions.app_error`
- [ ] Unauthenticated edit returns `401` with `api.context.session_expired.app_error`
- [ ] Response includes `id`, `message`, `detailed_error`, `status_code`, `request_id`

### Edit Policy
- [ ] Frontend calls `/api/v4/config/client?format=old` on startup
- [ ] Edit button hidden when `AllowEditPost: never`
- [ ] Edit disabled after time limit expires
- [ ] Edit allowed within time limit

### Notification Bell
- [ ] Bell shows actual unread count from `unreadStore`
- [ ] Badge clears when channel is marked read
- [ ] Updates in real-time via websocket

### Mobile Nav
- [ ] Shows actual unread count badge

### Unread Store
- [ ] WebSocket updates use proper SolidJS setters
- [ ] Reactivity works (UI updates when unread counts change)

---

## 📋 SPEC REFERENCES

- **FR-API-001**: Edit policy enforcement with time limits
- **FR-API-002**: Config client compatibility keys (`AllowEditPost`, `PostEditTimeLimit`)
- **FR-API-004**: Error semantics with Mattermost-compatible IDs
- **FR-WS-001**: `post_edited` websocket event
- **FR-WS-002**: `reaction_added`/`reaction_removed` events
- **FR-WS-003**: `unread_counts_updated` event
