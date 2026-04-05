# RustChat v4 API Implementation Execution Plan

**Date:** 2026-02-21  
**Goal:** Achieve full Mattermost Mobile compatibility  
**Estimated Effort:** 4-6 weeks

---

## Phase 1: Critical Fixes (Week 1)

### 1.1 Theme & Preferences System
**Priority:** P0  
**Effort:** 2-3 days  
**Files:**
- `backend/src/api/v4/users/preferences.rs` (extend)
- `backend/src/models/preference.rs` (new)

**Tasks:**
1. Create migration to add default theme preference for all users
2. Add `GET /users/me/preferences/{category}` endpoint
3. Seed default preferences on user creation
4. Add server config options: `AllowedThemes`, `DefaultTheme`

**Database Migration:**
```sql
-- Migration: add_default_themes.sql
INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    id as user_id,
    'theme' as category,
    '' as name,
    '{"type":"RustChat","sidebarBg":"#1A1A18","sidebarText":"#ffffff","centerChannelBg":"#121213","centerChannelColor":"#e3e4e8","linkColor":"#00FFC2","buttonBg":"#00FFC2","buttonColor":"#121213","errorTextColor":"#da6c6e","mentionHighlightBg":"#0d6e6e","mentionHighlightLink":"#a4f4f4","codeTheme":"monokai"}'::jsonb as value
FROM users
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences 
    WHERE user_id = users.id AND category = 'theme'
);
```

**Test:** Mobile settings → Display → Theme should show options

---

### 1.2 Reactions API
**Priority:** P0  
**Effort:** 2-3 days  
**Files:**
- `backend/src/api/v4/reactions.rs` (new)
- `backend/src/models/reaction.rs` (new)

**Endpoints to Implement:**
```rust
// GET /api/v4/posts/{id}/reactions
async fn get_post_reactions(State(state): State<AppState>, Path(post_id): Path<String>) -> ApiResult<Json<Vec<Reaction>>> {}

// POST /api/v4/reactions
async fn add_reaction(State(state): State<AppState>, auth: MmAuthUser, Json(body): Json<AddReactionRequest>) -> ApiResult<Json<Reaction>> {}

// DELETE /api/v4/users/{user_id}/posts/{post_id}/reactions/{emoji_name}
async fn remove_reaction(State(state): State<AppState>, auth: MmAuthUser, Path((user_id, post_id, emoji_name)): Path<(String, String, String)>) -> ApiResult<impl IntoResponse> {}
```

**Database:**
```sql
CREATE TABLE IF NOT EXISTS reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji_name VARCHAR(64) NOT NULL,
    create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    UNIQUE(post_id, user_id, emoji_name)
);
CREATE INDEX idx_reactions_post_id ON reactions(post_id);
CREATE INDEX idx_reactions_user_id ON reactions(user_id);
```

**WebSocket Events:**
- `reaction_added` - Broadcast when reaction added
- `reaction_removed` - Broadcast when reaction removed

**Test:** Long-press a post → Add reaction → Should show reactions

---

### 1.3 User Status API
**Priority:** P0  
**Effort:** 2-3 days  
**Files:**
- `backend/src/api/v4/status.rs` (new)

**Endpoints:**
```rust
// GET /api/v4/users/{id}/status
async fn get_user_status(State(state): State<AppState>, Path(user_id): Path<String>) -> ApiResult<Json<UserStatus>> {}

// PUT /api/v4/users/{id}/status
async fn update_user_status(State(state): State<AppState>, auth: MmAuthUser, Path(user_id): Path<String>, Json(body): Json<UpdateStatusRequest>) -> ApiResult<impl IntoResponse> {}

// POST /api/v4/users/status/ids
async fn get_user_statuses_by_ids(State(state): State<AppState>, Json(user_ids): Json<Vec<String>>) -> ApiResult<Json<Vec<UserStatus>>> {}
```

**Status Types:** `online`, `away`, `dnd`, `offline`

**Database:** Add to existing `users` table or create `user_status` table

**WebSocket:** `status_change` event

**Test:** User profile should show online/away/dnd indicator

---

## Phase 2: High Priority Features (Week 2-3)

### 2.1 Channel Bookmarks API
**Priority:** P1  
**Effort:** 3-4 days  
**Files:**
- `backend/src/api/v4/channel_bookmarks.rs` (new)

**Endpoints:**
```rust
// GET /api/v4/channels/{id}/bookmarks
// POST /api/v4/channels/{id}/bookmarks
// PATCH /api/v4/channels/{id}/bookmarks/{bookmark_id}
// DELETE /api/v4/channels/{id}/bookmarks/{bookmark_id}
// POST /api/v4/channels/{id}/bookmarks/{bookmark_id}/sort_order
```

**Database:**
```sql
CREATE TABLE channel_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_id UUID REFERENCES files(id) ON DELETE SET NULL,
    display_name VARCHAR(128),
    sort_order INTEGER DEFAULT 0,
    link_url TEXT,
    image_url TEXT,
    emoji VARCHAR(64),
    type VARCHAR(32) NOT NULL, -- 'link', 'file'
    create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    update_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    delete_at BIGINT DEFAULT 0
);
```

**WebSocket Events:**
- `channel_bookmark_created`
- `channel_bookmark_updated`
- `channel_bookmark_deleted`
- `channel_bookmark_sorted`

**Test:** Channel info → Bookmarks → Add bookmark

---

### 2.2 Emoji APIs
**Priority:** P1  
**Effort:** 2-3 days  
**Files:**
- `backend/src/api/v4/emoji.rs` (extend)

**Endpoints to Add:**
```rust
// GET /api/v4/emoji/name/{name}
async fn get_emoji_by_name(State(state): State<AppState>, Path(name): Path<String>) -> ApiResult<Json<Emoji>> {}

// POST /api/v4/emoji/search
async fn search_emoji(State(state): State<AppState>, Json(query): Json<SearchEmojiRequest>) -> ApiResult<Json<Vec<Emoji>>> {}

// GET /api/v4/emoji/autocomplete
async fn autocomplete_emoji(State(state): State<AppState>, Query(name): Query<String>) -> ApiResult<Json<Vec<Emoji>>> {}
```

**Test:** Emoji picker search and autocomplete

---

### 2.3 Scheduled Posts API
**Priority:** P1  
**Effort:** 3-4 days  
**Files:**
- `backend/src/api/v4/scheduled_posts.rs` (new)
- `backend/src/services/scheduler.rs` (new)

**Endpoints:**
```rust
// POST /api/v4/posts/schedule
// PUT /api/v4/posts/schedule/{id}
// DELETE /api/v4/posts/schedule/{id}
// GET /api/v4/posts/scheduled/team/{team_id}
```

**Database:**
```sql
CREATE TABLE scheduled_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    root_id UUID REFERENCES posts(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    file_ids TEXT[],
    scheduled_at BIGINT NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    error_code VARCHAR(64),
    create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    update_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000
);
```

**Background Job:** Process scheduled posts every minute

**Test:** Long press send button → Schedule post

---

## Phase 3: Medium Priority Features (Week 4-5)

### 3.1 Advanced Thread Features
**Priority:** P2  
**Effort:** 2-3 days

- `POST /users/{id}/teams/{tid}/threads/{tid}/set_unread/{pid}`
- `PUT /users/{id}/teams/{tid}/threads/read`
- Complete thread follow/unfollow

### 3.2 File Enhancements
**Priority:** P2  
**Effort:** 1-2 days

- `GET /files/{id}/link` - Public file links
- `POST /files/search` - File search

### 3.3 Post Actions
**Priority:** P2  
**Effort:** 2-3 days

- Pin/unpin posts
- Post acknowledgments
- Mark post as unread

### 3.4 System APIs
**Priority:** P2  
**Effort:** 2-3 days

- `GET /system/timezones` - Timezone list
- `POST /notifications/test` - Test push notification
- `POST /client_perf` - Performance metrics

---

## Phase 4: WebSocket Event Completeness (Week 5-6)

### 4.1 Missing WebSocket Events

**Post Events:**
```rust
// Add to realtime/events.rs
pub const POST_UNREAD: &str = "post_unread";
pub const POST_ACKNOWLEDGEMENT_ADDED: &str = "post_acknowledgement_added";
pub const POST_ACKNOWLEDGEMENT_REMOVED: &str = "post_acknowledgement_removed";
pub const EPHEMERAL_MESSAGE: &str = "ephemeral_message";
```

**Reaction Events:**
```rust
pub const REACTION_ADDED: &str = "reaction_added";
pub const REACTION_REMOVED: &str = "reaction_removed";
pub const EMOJI_ADDED: &str = "emoji_added";
```

**Bookmark Events:**
```rust
pub const CHANNEL_BOOKMARK_CREATED: &str = "channel_bookmark_created";
pub const CHANNEL_BOOKMARK_UPDATED: &str = "channel_bookmark_updated";
pub const CHANNEL_BOOKMARK_DELETED: &str = "channel_bookmark_deleted";
pub const CHANNEL_BOOKMARK_SORTED: &str = "channel_bookmark_sorted";
```

**Thread Events:**
```rust
pub const THREAD_READ_CHANGED: &str = "thread_read_changed";
pub const THREAD_FOLLOW_CHANGED: &str = "thread_follow_changed";
```

---

## Implementation Checklist

### Week 1: Critical Fixes
- [ ] 1.1.1 Create theme preference migration
- [ ] 1.1.2 Add `GET /users/me/preferences/{category}` endpoint
- [ ] 1.1.3 Seed default preferences on user creation
- [ ] 1.2.1 Create reactions table migration
- [ ] 1.2.2 Implement `GET /posts/{id}/reactions`
- [ ] 1.2.3 Implement `POST /reactions`
- [ ] 1.2.4 Implement `DELETE /reactions`
- [ ] 1.2.5 Add WebSocket events for reactions
- [ ] 1.3.1 Implement status endpoints
- [ ] 1.3.2 Add status change WebSocket events

### Week 2: Bookmarks & Emoji
- [ ] 2.1.1 Create channel_bookmarks table
- [ ] 2.1.2 Implement bookmark CRUD endpoints
- [ ] 2.1.3 Add bookmark WebSocket events
- [ ] 2.2.1 Extend emoji endpoints
- [ ] 2.2.2 Implement emoji search/autocomplete

### Week 3: Scheduled Posts
- [ ] 2.3.1 Create scheduled_posts table
- [ ] 2.3.2 Implement scheduled post endpoints
- [ ] 2.3.3 Create scheduler background job
- [ ] 2.3.4 Add scheduled post WebSocket events

### Week 4: Threads & Files
- [ ] 3.1.1 Implement thread unread endpoints
- [ ] 3.1.2 Implement mark all threads read
- [ ] 3.2.1 Implement file public links
- [ ] 3.2.2 Implement file search

### Week 5: Post Actions & System
- [ ] 3.3.1 Implement pin/unpin posts
- [ ] 3.3.2 Implement post acknowledgments
- [ ] 3.3.3 Implement mark post unread
- [ ] 3.4.1 Implement timezone list
- [ ] 3.4.2 Implement test notification

### Week 6: WebSocket Completeness
- [ ] 4.1 Add all missing WebSocket event types
- [ ] 4.2 Ensure all events broadcast correctly
- [ ] 4.3 Testing and bug fixes

---

## Testing Strategy

### Manual Testing Checklist

**Settings Page:**
- [ ] Theme selection shows all themes
- [ ] Theme changes apply immediately
- [ ] Clock format toggle works
- [ ] Timezone selection works
- [ ] CRT toggle works

**Reactions:**
- [ ] Can add reaction to post
- [ ] Can remove reaction
- [ ] Reactions show count
- [ ] Multiple users can react

**Status:**
- [ ] User status shows online/away/dnd/offline
- [ ] Status changes broadcast to other users
- [ ] Can set custom status

**Bookmarks:**
- [ ] Can add link bookmark
- [ ] Can add file bookmark
- [ ] Can reorder bookmarks
- [ ] Can delete bookmarks
- [ ] Bookmarks sync across devices

**Scheduled Posts:**
- [ ] Can schedule post for future
- [ ] Can edit scheduled post
- [ ] Can delete scheduled post
- [ ] Scheduled posts send at correct time

---

## Dependencies & Blockers

### Database Migrations
All features require database migrations. Plan downtime or use zero-downtime migration strategy.

### APNS Configuration
For call notifications to work:
- Apple Developer account required
- APNS key generation
- Push proxy configuration

### Mobile App Rebuild
After backend changes:
- Mobile app needs ringback.mp3 added
- Some features may require mobile app patches

---

## Success Criteria

1. **Settings Page:** All display settings functional
2. **Reactions:** 100% of reaction features working
3. **Status:** Real-time status updates working
4. **Bookmarks:** Full CRUD operations working
5. **Scheduled Posts:** Posts schedule and send correctly
6. **WebSocket:** All events broadcast and received

---

## Appendix: Mobile App Patches Required

### Ringback Tone
Add `ringback.mp3` to mobile app resources:
- Android: `android/app/src/main/res/raw/ringback.mp3`
- iOS: `ios/RustChat/Resources/ringback.mp3`

### Call Notifications
Ensure mobile app has proper CallKit/ConnectionService integration for incoming calls.

---

## Resources

- Mattermost API Reference: https://api.mattermost.com/
- Mobile App API Client: `/Users/scolak/Projects/rustchat-mobile/app/client/rest/`
- RustChat v4 API: `/Users/scolak/Projects/rustchat/backend/src/api/v4/`
