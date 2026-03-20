# Teams API Stub Implementation Design

**Date:** 2026-03-20
**Scope:** Implement 10 stub endpoints in `backend/src/api/v4/teams.rs`

## Overview

This specification details the implementation of 10 currently stubbed team management endpoints in the rustchat backend. These endpoints currently return `{"status": "OK"}` without performing any actual operations.

## Endpoints to Implement

### 1. PUT /teams/{team_id}/privacy — `update_team_privacy`

**Purpose:** Change team visibility between open (anyone can join) and invite-only.

**Request Body:**
```json
{
  "privacy": "O" | "I"
}
```

**Implementation:**
- Parse privacy value: "O" → "open", "I" → "invite"
- Validate caller has team admin role or SYSTEM_MANAGE permission
- Update `teams.privacy` column
- Return updated team object

**Database Migration:**
```sql
ALTER TABLE teams ADD COLUMN privacy TEXT NOT NULL DEFAULT 'open';
```

**Authorization:** `ensure_team_admin_or_system_manage()`

---

### 2. POST /teams/{team_id}/restore — `restore_team`

**Purpose:** Restore a previously archived (soft-deleted) team.

**Implementation:**
- Validate team exists and `deleted_at IS NOT NULL`
- Validate caller has team admin role or SYSTEM_MANAGE permission
- Set `teams.deleted_at = NULL`
- Return restored team object

**Authorization:** `ensure_team_admin_or_system_manage()`

---

### 3. POST /teams/{team_id}/image — `update_team_icon`

**Purpose:** Upload and set team icon/image.

**Request:** Multipart form with image file (PNG, JPG, max 1MB)

**Implementation:**
- Validate caller is team admin
- Generate unique filename: `teams/{team_id}/icon_{timestamp}.{ext}`
- Store in configured file storage (S3 or local)
- Generate thumbnail variants (64x64, 128x128)
- Update `teams.icon_path` with primary image path
- Return `{ "status": "OK" }`

**Database Migration:**
```sql
ALTER TABLE teams ADD COLUMN icon_path TEXT;
```

**Authorization:** Team admin only

---

### 4. DELETE /teams/{team_id}/image — `delete_team_icon`

**Purpose:** Remove team icon.

**Implementation:**
- Validate caller is team admin
- Delete file from storage if `icon_path` exists
- Set `teams.icon_path = NULL`
- Return `{ "status": "OK" }`

**Authorization:** Team admin only

---

### 5. PUT /teams/{team_id}/members/{user_id}/scheme_roles — `update_team_member_scheme_roles`

**Purpose:** Update a member's scheme-derived roles (admin/user/guest).

**Request Body:**
```json
{
  "scheme_admin": true | false,
  "scheme_user": true | false,
  "scheme_guest": true | false
}
```

**Implementation:**
- Validate caller has team admin or SYSTEM_MANAGE permission
- Validate target user is a member of the team
- Update `team_members.role` based on scheme flags:
  - `scheme_admin: true` → role = "admin"
  - `scheme_user: true` → role = "member"
  - `scheme_guest: true` → role = "guest"
- Return updated team member object

**Authorization:** `ensure_team_admin_or_system_manage()`

---

### 6. POST /teams/{team_id}/invite — `invite_users_to_team`

**Purpose:** Invite existing users to join the team by username.

**Request Body:**
```json
{
  "user_ids": ["user_id_1", "user_id_2"]
}
```

**Implementation:**
- Validate caller is team member with invite permission (or admin)
- For each user_id:
  - Verify user exists and is not already a team member
  - Create `team_invitations` record with:
    - `team_id`, `user_id`, `invited_by`, `token` (secure random)
    - `expires_at = NOW() + 7 days`, `used = false`
  - Send in-app notification to invited user
- Return array of invited user objects

**Database Migration:**
```sql
CREATE TABLE team_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invited_by UUID NOT NULL REFERENCES users(id),
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, user_id, used) WHERE used = false
);
CREATE INDEX idx_team_invitations_user ON team_invitations(user_id);
CREATE INDEX idx_team_invitations_token ON team_invitations(token);
```

**Authorization:** Team member with invite permission (team setting)

---

### 7. POST /teams/{team_id}/invite/guests — `invite_guests_to_team`

**Purpose:** Invite external guests (limited-access users) to the team.

**Request Body:**
```json
{
  "emails": ["guest@example.com"],
  "channels": ["channel_id_1"],
  "message": "Optional welcome message"
}
```

**Implementation:**
- Validate caller is team admin
- For each email:
  - Check if user exists; if not, create provisional guest user record
  - Create `team_invitations` record with type = 'guest'
  - Send email invitation with secure token
- Return array of invitation objects

**Authorization:** Team admin only

---

### 8. POST /teams/{team_id}/invite/email — `invite_users_to_team_by_email`

**Purpose:** Send email invitations to non-registered users.

**Request Body:**
```json
{
  "emails": ["newuser@example.com"],
  "site_url": "https://chat.example.com"
}
```

**Implementation:**
- Validate caller is team admin or has invite permission
- For each email:
  - Generate secure invitation token
  - Create `team_invitations` record
  - Send email with invitation link containing token
- Return array of sent invitations

**Email Template:** Use existing email service with new template `templates/emails/team_invitation.html`

**Authorization:** Team admin or member with invite permission

---

### 9. POST /teams/{team_id}/import — `import_team`

**Purpose:** Import team data from a Mattermost export file.

**Request:** Multipart form with `.zip` export file

**Implementation:**
- Validate caller has SYSTEM_MANAGE permission
- Parse export file structure
- Import in transaction:
  - Channels (with archived status)
  - Channel memberships
  - Posts (respecting `create_at` timestamps)
  - Files (download and store)
- Return import summary: `{ "channels": N, "posts": N, "errors": [] }`

**Note:** This is a complex operation; initial implementation may support basic channel/post import only.

**Authorization:** SYSTEM_MANAGE only

---

### 10. GET/PUT /teams/{team_id}/scheme — `get_team_scheme` / `update_team_scheme`

**Purpose:** Get or update the team's permission scheme.

**GET Response:**
```json
{
  "team_id": "...",
  "scheme_id": "...",
  "default_team_admin_role": "...",
  "default_team_user_role": "...",
  "default_team_guest_role": "..."
}
```

**Implementation:**
- GET: Return team scheme metadata from `team_schemes` table
- PUT: Update team's assigned scheme (requires SYSTEM_MANAGE)

**Database Migration:**
```sql
-- team_schemes table likely exists; add relationship if needed
ALTER TABLE teams ADD COLUMN scheme_id UUID REFERENCES schemes(id);
```

**Authorization:**
- GET: Team member
- PUT: SYSTEM_MANAGE only

---

## 11. GET /teams/{team_id}/members_minus_group_members — `get_team_members_minus_group_members`

**Purpose:** Get team members not in a specific group/channel.

**Query Parameters:** `group_id`, `channel_id` (mutually exclusive)

**Implementation:**
- Validate caller is team member
- If `group_id` provided: exclude members of that group
- If `channel_id` provided: exclude members of that channel
- Return filtered member list

**Authorization:** Team member

---

## Common Patterns

### Error Handling

All endpoints return consistent error responses:

```json
{
  "id": "api.team.update_privacy.app_error",
  "message": "Failed to update team privacy",
  "detailed_error": "",
  "request_id": "...",
  "status_code": 403
}
```

### Authorization Helpers

Use existing helpers:
- `ensure_team_member(state, team_id, user_id)` — verify membership
- `ensure_team_admin_or_system_manage(state, team_id, auth)` — admin operations
- `auth.has_permission(&permissions::SYSTEM_MANAGE)` — system-level ops

### Database Transactions

Mutating operations should use transactions:

```rust
let mut tx = state.db.begin().await?;
// ... operations ...
tx.commit().await?;
```

---

## Implementation Order

1. **Database migrations** — Run first, all other work depends on schema
2. **Basic CRUD endpoints** — `restore_team`, `update_team_privacy`, `update_team_member_scheme_roles`
3. **Icon management** — `update_team_icon`, `delete_team_icon` (requires file storage)
4. **Invitations** — `invite_users_to_team`, `invite_users_to_team_by_email`, `invite_guests_to_team` (requires email service)
5. **Advanced features** — `import_team`, scheme endpoints

---

## Testing Strategy

### Unit Tests
- Test each endpoint's authorization logic
- Test database query correctness
- Test error conditions (invalid UUIDs, missing resources)

### Integration Tests
- Test full invitation flow (invite → accept → join)
- Test icon upload/download cycle
- Test team import with sample export file

---

## Files to Modify

1. `backend/src/api/v4/teams.rs` — Main implementation
2. `backend/migrations/` — Database schema changes
3. `backend/src/services/email.rs` — Add invitation email template
4. `backend/src/models/` — Add invitation model if needed
