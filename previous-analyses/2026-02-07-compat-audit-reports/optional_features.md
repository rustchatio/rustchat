# Optional Features Truth Table

## Methodology

For each "optional" (not implemented) endpoint, determine:
- **Used in Default Flow?**: Does mobile call this in normal usage (login, browse, message)?
- **Priority**: If yes → must fix. If no → can defer.

---

## Truth Table

| # | Endpoint | Mobile File | Used in Default Flow | Evidence | Priority |
|---|----------|-------------|---------------------|----------|----------|
| 1 | POST /oauth/intune | users.ts:174 | **NO** | Requires Intune enterprise config | PARK |
| 2 | GET /custom_profile_attributes/fields | users.ts:311 | **NO** | Only when feature flag enabled | PARK |
| 3 | GET /channels/{id}/access_control/attributes | channels.ts:292 | **NO** | Enterprise access control only | PARK |
| 4 | POST /notifications/test | posts.ts:227 | **NO** | Only from Push Notification Troubleshooter | PARK |
| 5 | GET /posts/{id}/reveal | posts.ts:234 | **NO** | "Blast of Random" feature only | PARK |
| 6 | GET /license/load_metric | general.ts:75 | **NO** | Enterprise license metrics only | PARK |
| 7 | POST /client_perf | general.ts:125 | **NO** | Performance metrics opt-in | PARK |
| 8 | POST /scheduled_posts | scheduled_post.ts:21 | **YES** | Users can schedule posts | PRIORITIZE |
| 9 | PUT /scheduled_posts/{id} | scheduled_post.ts:37 | **YES** | Edit scheduled posts | PRIORITIZE |
| 10 | GET /scheduled_posts/team/{id} | scheduled_post.ts:48 | **YES** | List scheduled posts | PRIORITIZE |
| 11 | DELETE /scheduled_posts/{id} | scheduled_post.ts:59 | **YES** | Cancel scheduled posts | PRIORITIZE |

---

## Summary

| Category | Count | Action |
|----------|-------|--------|
| **PRIORITIZE** | 4 | Scheduled Posts feature |
| **PARK** | 7 | Enterprise/optional features |

---

## Detailed Analysis

### PRIORITIZE: Scheduled Posts (4 endpoints)

**User Impact**: Users clicking "Schedule Send" will get errors.

**Evidence from mattermost-mobile**:
- `app/screens/compose/components/schedule_post_picker.tsx` - UI exists
- Called when user picks "Schedule for later" in post composer

**Canonical Behavior** (Mattermost Server):
- File: `server/channels/api4/scheduled_post.go`
- Stores posts with `scheduled_at` timestamp
- Background job sends at scheduled time

**Required RustChat Changes**:
1. Create `scheduled_posts` table migration
2. Add `scheduled_posts` model
3. Implement CRUD handlers in `api/v4/scheduled_posts.rs`
4. Add background job to send scheduled posts

**Tests to Add**:
```rust
#[test]
fn test_create_scheduled_post() { ... }
#[test]
fn test_list_scheduled_posts_for_team() { ... }
#[test]
fn test_update_scheduled_post() { ... }
#[test]
fn test_delete_scheduled_post() { ... }
```

---

### PARK: Enterprise Features (7 endpoints)

These endpoints are behind feature flags or enterprise configuration:

| Endpoint | Reason to Park |
|----------|----------------|
| /oauth/intune | Microsoft Intune enterprise SSO |
| /custom_profile_attributes | Enterprise profile customization |
| /access_control/attributes | Enterprise access policies |
| /notifications/test | Debug/troubleshooting only |
| /posts/{id}/reveal | Blind or Random feature |
| /license/load_metric | License usage metrics |
| /client_perf | Optional performance reporting |

**Mobile Behavior When Missing**: 
- Features are hidden or gracefully degraded
- No error messages shown to users
- App continues functioning normally

---

## Recommendation

### Immediate Action
None required for default flows to work.

### When Users Request Scheduled Posts
Implement the 4 scheduled post endpoints:
- POST /scheduled_posts
- PUT /scheduled_posts/{id}
- GET /scheduled_posts/team/{id}
- DELETE /scheduled_posts/{id}

Estimated effort: 4-6 hours including tests.
