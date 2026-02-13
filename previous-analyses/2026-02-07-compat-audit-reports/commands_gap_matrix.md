# Commands Gap Analysis Matrix

## Deliverable C: RustChat vs Mattermost Commands Comparison

---

## Endpoint Coverage

| # | Endpoint | Mattermost (command.go) | RustChat (commands.rs) | Status | Gap | Priority |
|---|----------|------------------------|------------------------|--------|-----|----------|
| 1 | POST /commands | line 17, createCommand | NOT EXPOSED via v4 | **NO** | Missing route | P1 |
| 2 | GET /commands | line 18, listCommands | line 18, list_commands (stub) | **PARTIAL** | Returns hardcoded data | P1 |
| 3 | POST /commands/execute | line 19 | line 19 | **YES** | Working via integrations | P0 ✅ |
| 4 | GET /commands/{id} | line 21 | line 22, get_command (stub) | **PARTIAL** | Returns empty {} | P2 |
| 5 | PUT /commands/{id} | line 22 | line 22, update_command (stub) | **PARTIAL** | Returns empty {} | P2 |
| 6 | DELETE /commands/{id} | line 24 | line 22, delete_command (stub) | **PARTIAL** | No actual deletion | P2 |
| 7 | PUT /commands/{id}/move | line 23 | line 24, move_command (stub) | **PARTIAL** | Returns empty {} | P3 |
| 8 | GET /teams/{id}/commands/autocomplete | line 26 | **MISSING** | **NO** | Endpoint not registered | P1 |
| 9 | GET /teams/{id}/commands/autocomplete_suggestions | line 27 | line 30, autocomplete_suggestions | **PARTIAL** | Only /call suggestion | P1 |
| 10 | PUT /commands/{id}/regen_token | line 28 | line 27, regenerate_command_token (stub) | **PARTIAL** | Returns empty token | P3 |

---

## Model Comparison

### Mattermost Command Model (command.go:18-42)
| Field | Type | Required | RustChat SlashCommand | Status |
|-------|------|----------|----------------------|--------|
| id | string(26) | yes | id: Uuid | ⚠️ UUID vs MM ID |
| token | string(26) | yes | token: String | ✅ |
| create_at | int64 | yes | created_at: DateTime | ⚠️ Different name |
| update_at | int64 | yes | updated_at: DateTime | ⚠️ Different name |
| delete_at | int64 | yes | **MISSING** | ❌ |
| creator_id | string(26) | yes | creator_id: Uuid | ⚠️ UUID vs MM ID |
| team_id | string(26) | yes | team_id: Uuid | ⚠️ UUID vs MM ID |
| trigger | string | yes | trigger: String | ✅ |
| method | string (P/G) | yes | method: String | ⚠️ Uses POST vs P |
| username | string | no | **MISSING** | ❌ |
| icon_url | string | no | icon_url: Option | ✅ |
| auto_complete | bool | yes | auto_complete: bool | ✅ |
| auto_complete_desc | string | no | **MISSING** | ❌ Uses description |
| auto_complete_hint | string | no | hint: Option | ✅ |
| display_name | string | no | display_name: Option | ✅ |
| description | string | no | description: Option | ✅ |
| url | string | yes | url: String | ✅ |
| plugin_id | string | no | **MISSING** | ❌ Not needed |

### Mattermost CommandResponse (command_response.go:19-32)
| Field | Type | RustChat CommandResponse | Status |
|-------|------|-------------------------|--------|
| response_type | string | response_type: String | ✅ |
| text | string | text: String | ✅ |
| username | string | username: Option | ✅ |
| channel_id | string | **MISSING** | ❌ |
| icon_url | string | icon_url: Option | ✅ |
| type | string | **MISSING** | ❌ |
| props | StringInterface | **MISSING** | ❌ |
| goto_location | string | goto_location: Option | ✅ |
| trigger_id | string | **MISSING** | ❌ Critical for dialogs |
| skip_slack_parsing | bool | **MISSING** | ❌ |
| attachments | SlackAttachment[] | attachments: Option<Value> | ⚠️ Type mismatch |
| extra_responses | CommandResponse[] | **MISSING** | ❌ |

---

## Behavioral Gaps

### 1. Execute Command Pipeline

| Behavior | Mattermost | RustChat | Gap |
|----------|------------|----------|-----|
| Command validation | `/` prefix + length check | `/` prefix check | ✅ Similar |
| Channel permission | PermissionCreatePost | MmAuthUser check | ⚠️ Less granular |
| Team resolution | From channel for non-DM | Optional team_id | ⚠️ Not auto-resolved |
| Built-in commands | 25+ commands | 4 commands (call, echo, shrug, invite) | ❌ **Missing 21** |
| Custom command lookup | DB by (team_id, trigger) | integrations.rs has it | ⚠️ Not in v4 router |
| Custom command HTTP | reqwest with timeout | **NOT IMPLEMENTED** | ❌ **Critical** |
| Ephemeral posts | PostEphemeral() | Response only | ⚠️ No post created |
| In-channel posts | CreatePost() | Manual in some cases | ⚠️ Inconsistent |
| Trigger ID | Generated for dialogs | **MISSING** | ❌ |

### 2. Autocomplete Behavior

| Behavior | Mattermost | RustChat | Gap |
|----------|------------|----------|-----|
| Built-in suggestions | All from registered | Only /call | ❌ Hardcoded |
| Custom command suggestions | From DB auto_complete=true | None | ❌ |
| Permission filtering | By role (admin sees more) | None | ❌ |
| Partial match | Prefix match on trigger | Only /call | ❌ |

### 3. Permission Model

| Permission | Mattermost | RustChat | Gap |
|------------|------------|----------|-----|
| PermissionManageOwnSlashCommands | Create/edit own | role == "system_admin" | ⚠️ Simplified |
| PermissionManageOthersSlashCommands | Create/edit others | role == "system_admin" | ⚠️ Simplified |
| PermissionCreatePost | Execute in channel | MmAuthUser (member) | ⚠️ Less granular |

---

## Priority Classification

### P0 - Blocks Basic Use
- None at critical level (execute works)

### P1 - Major Gaps (Must Fix)
1. **Custom command HTTP execution** - integrations.rs has code but not connected
2. **listAutocompleteCommands endpoint** - Missing entirely
3. **list_commands returning real data** - Currently hardcoded
4. **createCommand via v4 API** - Needed for admin panel

### P2 - Moderate Gaps (Should Fix)  
1. Command model field alignment (delete_at, method format)
2. CommandResponse trigger_id for interactive dialogs
3. CRUD operations returning real data
4. Built-in commands (at least /join, /leave, /me, /msg)

### P3 - Minor Gaps (Nice to Have)
1. moveCommand proper implementation
2. regenCommandToken proper implementation
3. All 25+ built-in commands
4. Extra responses support
5. Skip slack parsing flag

---

## Summary Metrics

| Category | Count | Status |
|----------|-------|--------|
| Endpoints | 10 | 1 ✅, 6 PARTIAL, 3 ❌ |
| Command fields | 17 | 10 ✅, 4 ⚠️, 3 ❌ |
| Response fields | 12 | 6 ✅, 1 ⚠️, 5 ❌ |
| Built-in commands | 25+ | 4 implemented |
| Custom command execution | required | ❌ Not implemented |
