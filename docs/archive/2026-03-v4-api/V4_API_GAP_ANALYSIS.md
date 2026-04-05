# RustChat v4 API Gap Analysis

**Date:** 2026-02-21  
**Purpose:** Comprehensive analysis of missing v4 API features needed for full Mattermost Mobile compatibility  
**Reference:** Compared against Mattermost Server v9.x API

---

## Executive Summary

RustChat currently implements **~60%** of the v4 APIs required for full Mattermost Mobile compatibility. While core messaging functionality works, several critical features for mobile settings, advanced notifications, and user preferences are missing or incomplete.

### Critical Gaps
1. **Theme/Preference Management** - Mobile settings page partially non-functional
2. **Channel Bookmarks** - Feature flag enabled but API missing
3. **Scheduled Posts** - Mobile UI exists but backend API incomplete
4. **Reactions API** - Partially implemented
5. **Status API** - User status management missing
6. **WebSocket Events** - Several event types not broadcast

---

## Detailed API Gap Analysis

### 1. USER MANAGEMENT APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /users/me/teams/members` | ❌ Missing | HIGH | Required for team membership display |
| `GET /users/me/channels` | ❌ Missing | HIGH | Get all channels across teams |
| `GET /users/me/channel_members` | ❌ Missing | HIGH | Channel membership info |
| `GET /users/me/posts/flagged` | ⚠️ Partial | MEDIUM | Saved posts/bookmarks |
| `PUT /users/me/status/custom` | ❌ Missing | HIGH | Custom status messages |
| `GET /users/{id}/status` | ❌ Missing | HIGH | User online/away/dnd status |
| `PUT /users/{id}/status` | ❌ Missing | HIGH | Update user status |
| `GET /users/status/ids` | ❌ Missing | HIGH | Bulk status fetch |
| `POST /users/{id}/posts/{postId}/set_unread` | ❌ Missing | MEDIUM | Mark post unread |
| `POST /users/{id}/posts/{postId}/ack` | ❌ Missing | LOW | Post acknowledgments |
| `POST /users/autocomplete` | ❌ Missing | MEDIUM | User autocomplete |
| `POST /users/known` | ❌ Missing | LOW | Known users list |
| `POST /users/{id}/demote` | ❌ Missing | LOW | Guest demotion |
| `GET /users/{id}/custom_profile_attributes` | ⚠️ Partial | MEDIUM | Extended profile |

**Files to Examine:**
- `/Users/scolak/Projects/rustchat-mobile/app/client/rest/users.ts`

---

### 2. PREFERENCES APIs (CRITICAL FOR SETTINGS)

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /users/me/preferences` | ✅ Implemented | - | Basic prefs working |
| `PUT /users/me/preferences` | ✅ Implemented | - | Update working |
| `GET /users/me/preferences/{category}` | ❌ Missing | HIGH | Theme/category specific |

**Missing Preference Categories:**
- `theme` - Currently returns empty, themes not working
- `display_settings` - Clock, timezone, CRT settings
- `notifications` - Push, email, mention preferences
- `sidebar_settings` - Category preferences

**Root Cause:**
- Preferences table exists but default preferences not seeded
- Theme preferences not properly handled

---

### 3. CHANNEL APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /channels/{id}/stats` | ❌ Missing | MEDIUM | Member count, etc. |
| `GET /channels/{id}/timezones` | ❌ Missing | LOW | Channel timezone list |
| `GET /channels/{id}/member_counts_by_group` | ❌ Missing | LOW | Group analytics |
| `GET /channels/{id}/access_control/attributes` | ❌ Missing | LOW | Access control |
| `GET /channels/{id}/common_teams` | ❌ Missing | LOW | GM common teams |
| `POST /channels/{id}/convert_to_channel` | ❌ Missing | LOW | GM to channel conversion |
| `GET /channels/{id}/bookmarks` | ❌ Missing | HIGH | Channel bookmarks |
| `POST /channels/{id}/bookmarks` | ❌ Missing | HIGH | Create bookmark |
| `PATCH /channels/{id}/bookmarks/{bid}` | ❌ Missing | HIGH | Update bookmark |
| `DELETE /channels/{id}/bookmarks/{bid}` | ❌ Missing | HIGH | Delete bookmark |
| `POST /channels/{id}/bookmarks/{bid}/sort_order` | ❌ Missing | MEDIUM | Reorder bookmarks |

**Files to Examine:**
- `/Users/scolak/Projects/rustchat-mobile/app/client/rest/channels.ts`

---

### 4. POSTS & THREADS APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /posts/{id}/files/info` | ❌ Missing | MEDIUM | File metadata |
| `POST /posts/{id}/pin` | ⚠️ Partial | MEDIUM | Pin post |
| `POST /posts/{id}/unpin` | ⚠️ Partial | MEDIUM | Unpin post |
| `POST /posts/{id}/actions/{actionId}` | ❌ Missing | LOW | Post actions |
| `DELETE /posts/{id}/burn` | ❌ Missing | LOW | Burn on Read |
| `GET /posts/{id}/reveal` | ❌ Missing | LOW | Reveal BoR |
| `POST /posts/schedule` | ❌ Missing | HIGH | Schedule post |
| `PUT /posts/schedule/{id}` | ❌ Missing | HIGH | Update scheduled |
| `DELETE /posts/schedule/{id}` | ❌ Missing | HIGH | Delete scheduled |
| `GET /posts/scheduled/team/{teamId}` | ❌ Missing | HIGH | List scheduled |

---

### 5. REACTIONS APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /posts/{id}/reactions` | ❌ Missing | HIGH | Get reactions |
| `POST /reactions` | ❌ Missing | HIGH | Add reaction |
| `DELETE /reactions` | ❌ Missing | HIGH | Remove reaction |

**Note:** Reactions are critical for mobile UX - currently completely missing.

---

### 6. SIDEBAR CATEGORIES APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /users/{uid}/teams/{tid}/channels/categories` | ✅ Implemented | - | Categories working |
| `PUT /users/{uid}/teams/{tid}/channels/categories` | ✅ Implemented | - | Update working |
| `GET /users/{uid}/teams/{tid}/channels/categories/order` | ❌ Missing | MEDIUM | Category order |

---

### 7. THREADS APIs (CRT - Collapsed Reply Threads)

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /users/{id}/teams/{tid}/threads` | ✅ Implemented | - | List threads |
| `GET /users/{id}/teams/{tid}/threads/{tid}` | ✅ Implemented | - | Thread details |
| `PUT /users/{id}/teams/{tid}/threads/{tid}/read/{ts}` | ✅ Implemented | - | Mark read |
| `POST /users/{id}/teams/{tid}/threads/{tid}/set_unread/{pid}` | ❌ Missing | MEDIUM | Mark unread |
| `PUT /users/{id}/teams/{tid}/threads/read` | ❌ Missing | MEDIUM | Mark all read |
| `PUT/DELETE /users/{id}/teams/{tid}/threads/{tid}/following` | ⚠️ Partial | MEDIUM | Follow/unfollow |

---

### 8. FILES APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `POST /files` | ✅ Implemented | - | Upload working |
| `GET /files/{id}` | ✅ Implemented | - | Download working |
| `GET /files/{id}/thumbnail` | ✅ Implemented | - | Thumbnails working |
| `GET /files/{id}/preview` | ✅ Implemented | - | Previews working |
| `GET /files/{id}/link` | ❌ Missing | LOW | Public link |
| `POST /files/search` | ❌ Missing | MEDIUM | File search |

---

### 9. EMOJI APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /emoji` | ⚠️ Partial | HIGH | List custom emoji |
| `GET /emoji/{id}` | ❌ Missing | MEDIUM | Get emoji by ID |
| `GET /emoji/name/{name}` | ❌ Missing | MEDIUM | Get emoji by name |
| `POST /emoji/search` | ❌ Missing | MEDIUM | Search emoji |
| `GET /emoji/autocomplete` | ❌ Missing | HIGH | Emoji autocomplete |

---

### 10. STATUS APIs (CRITICAL)

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /users/{id}/status` | ❌ Missing | HIGH | Get user status |
| `PUT /users/{id}/status` | ❌ Missing | HIGH | Update status |
| `POST /users/status/ids` | ❌ Missing | HIGH | Bulk status fetch |

**Note:** Status is fundamental for presence indication in mobile app.

---

### 11. CUSTOM PROFILE ATTRIBUTES APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /custom_profile_attributes/fields` | ❌ Missing | MEDIUM | Get CPA fields |
| `PATCH /custom_profile_attributes/values` | ❌ Missing | MEDIUM | Update CPA values |

---

### 12. DATA RETENTION APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /data_retention/policy` | ❌ Missing | LOW | Global policy |
| `GET /users/{id}/data_retention/team_policies` | ❌ Missing | LOW | Team policies |
| `GET /users/{id}/data_retention/channel_policies` | ❌ Missing | LOW | Channel policies |

---

### 13. PLUGIN APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /plugins/webapp` | ❌ Missing | MEDIUM | Plugin manifests |
| `GET /plugins/{id}/...` | ⚠️ Partial | - | Routes vary by plugin |

---

### 14. CALLS PLUGIN APIs (PARTIALLY IMPLEMENTED)

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /plugins/com.mattermost.calls/version` | ✅ Implemented | - | Version check |
| `GET /plugins/com.mattermost.calls/config` | ✅ Implemented | - | Config |
| `GET /plugins/com.mattermost.calls/channels` | ✅ Implemented | - | Active calls |
| `GET/POST /plugins/com.mattermost.calls/{channelId}` | ✅ Implemented | - | Enable/disable |
| `POST /plugins/com.mattermost.calls/calls/{id}/end` | ✅ Implemented | - | End call |
| `POST /plugins/com.mattermost.calls/calls/{id}/recording/start` | ✅ Implemented | - | Recording |
| `POST /plugins/com.mattermost.calls/calls/{id}/recording/stop` | ✅ Implemented | - | Stop recording |
| `POST /plugins/com.mattermost.calls/calls/{id}/dismiss-notification` | ✅ Implemented | - | Dismiss |
| `POST /plugins/com.mattermost.calls/calls/{id}/host/make` | ✅ Implemented | - | Make host |
| `POST /plugins/com.mattermost.calls/calls/{id}/host/mute` | ✅ Implemented | - | Host mute |
| `POST /plugins/com.mattermost.calls/calls/{id}/host/mute-others` | ✅ Implemented | - | Mute others |
| `POST /plugins/com.mattermost.calls/calls/{id}/host/screen-off` | ✅ Implemented | - | Screen off |
| `POST /plugins/com.mattermost.calls/calls/{id}/host/lower-hand` | ✅ Implemented | - | Lower hand |
| `POST /plugins/com.mattermost.calls/calls/{id}/host/remove` | ✅ Implemented | - | Remove user |
| `GET /plugins/com.mattermost.calls/turn-credentials` | ✅ Implemented | - | TURN creds |
| `POST /plugins/com.mattermost.calls/calls/{id}/ring` | ✅ Implemented | - | Ring users |
| `POST /plugins/com.mattermost.calls/calls/{id}/join` | ✅ Implemented | - | Join call |
| `POST /plugins/com.mattermost.calls/calls/{id}/leave` | ✅ Implemented | - | Leave call |
| `GET /plugins/com.mattermost.calls/calls/{id}` | ✅ Implemented | - | Call state |

**Note:** Calls plugin is well-implemented. Main issue is APNS configuration for push notifications.

---

### 15. SYSTEM & CONFIG APIs

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /system/ping` | ✅ Implemented | - | Health check |
| `GET /system/timezones` | ❌ Missing | MEDIUM | Timezone list |
| `POST /logs` | ❌ Missing | LOW | Client logging |
| `GET /config/client` | ✅ Implemented | - | Client config |
| `GET /license/client` | ✅ Implemented | - | License info |
| `GET /license/load_metric` | ❌ Missing | LOW | License metrics |
| `GET /data_retention/policy` | ❌ Missing | LOW | Retention policy |
| `GET /redirect_location` | ❌ Missing | LOW | URL preview |
| `POST /client_perf` | ❌ Missing | LOW | Performance reports |
| `GET /terms_of_service` | ✅ Implemented | - | TOS |
| `POST /notifications/test` | ❌ Missing | MEDIUM | Test notification |

---

### 16. GROUPS APIs (ENTERPRISE)

| Endpoint | Status | Priority | Notes |
|----------|--------|----------|-------|
| `GET /groups` | ❌ Missing | LOW | List groups |
| `GET /channels/{id}/groups` | ❌ Missing | LOW | Channel groups |
| `GET /teams/{id}/groups` | ❌ Missing | LOW | Team groups |
| `GET /users/{id}/groups` | ❌ Missing | LOW | User groups |

---

### 17. WEBSOCKET EVENTS GAPS

**Currently Implemented:**
- `posted`, `post_edited`, `post_deleted`
- `channel_created`, `channel_updated`, `channel_deleted`
- `user_added`, `user_removed`, `user_updated`
- `preference_changed`
- `status_change` (partial)
- `typing`
- Calls plugin events

**Missing Events:**
- `post_unread` - Mark post as unread
- `ephemeral_message` - System messages
- `post_acknowledgement_added/removed` - Post acks
- `post_translation_updated` - Translations
- `channel_viewed` - Channel viewed
- `multiple_channels_viewed` - Bulk view
- `channel_member_updated` - Member changes
- `sidebar_category_created/updated/deleted/order_updated`
- `thread_read_changed` - Thread read state
- `thread_follow_changed` - Thread follow state
- `reaction_added/removed` - Reactions
- `emoji_added` - New emoji
- `role_updated` - Role changes
- `plugin_enabled/disabled/statuses_changed`
- `channel_bookmark_created/updated/sorted/deleted`
- `scheduled_post_created/updated/deleted`
- `custom_profile_attributes_values_updated`
- `received_group` and related group events

---

### 18. THEME & DISPLAY SETTINGS (CRITICAL FOR MOBILE)

**Current State:**
- Theme preferences not seeded in database
- `/api/v4/config/client` returns hardcoded theme list
- Mobile app expects theme in user preferences

**Required Implementation:**
```sql
-- Seed default theme preference for users
INSERT INTO mattermost_preferences (user_id, category, name, value)
VALUES (
    'USER_ID',
    'theme',
    '',
    '{"type":"RustChat","sidebarBg":"#1A1A18",...}'
);
```

**Server Configuration Missing:**
- `AllowedThemes` - List of allowed theme keys
- `DefaultTheme` - Default theme for new users
- `EnableThemeSelection` - Toggle theme switching

---

## Mobile Settings Page Feature Gaps

### Display Settings
| Feature | Status | API Required |
|---------|--------|--------------|
| Theme Selection | ❌ Not Working | Theme preferences |
| Clock Display (12/24h) | ⚠️ Partial | User preference |
| Timezone | ⚠️ Partial | Timezone list API |
| Collapsed Reply Threads | ✅ Working | Config flag |

### Notification Settings
| Feature | Status | API Required |
|---------|--------|--------------|
| Mentions & Replies | ⚠️ Partial | Push notification config |
| Push Notifications | ⚠️ Partial | APNS/FCM configuration |
| Call Notifications | ⚠️ Partial | Calls config |
| Email Notifications | ⚠️ Partial | Email settings |
| Auto Responder | ❌ Missing | OOO responder API |

### Advanced Settings
| Feature | Status | API Required |
|---------|--------|--------------|
| Download Logs | ✅ Working | Client-side |
| Report a Problem | ✅ Working | Client-side |

---

## Priority Summary

### P0 - Critical (Mobile Non-Functional Without These)
1. **Theme Preferences** - Settings page broken
2. **Reactions API** - Can't react to posts
3. **User Status API** - No presence indication
4. **Scheduled Posts** - Feature exists but API broken

### P1 - High (Major Feature Gaps)
1. Channel Bookmarks API
2. Emoji autocomplete/search
3. Post file info
4. Thread follow/unfollow
5. WebSocket events for reactions, bookmarks

### P2 - Medium (Nice to Have)
1. Data retention policies
2. Custom profile attributes
3. Group APIs
4. Advanced search

### P3 - Low (Enterprise/Nice-to-Have)
1. Audit logging
2. Compliance exports
3. Advanced access control
4. Shared channels

---

## Database Schema Gaps

### Missing Tables
- `channel_bookmarks` - For bookmark feature
- `scheduled_posts` - For scheduled messages
- `reactions` - For post reactions (may exist but unused)

### Missing Columns
- `users.timezone` - Timezone settings
- `users.custom_status` - Custom status message
- `posts.has_reactions` - Reaction indicator

---

## Next Steps

See `V4_API_EXECUTION_PLAN.md` for detailed implementation roadmap.
