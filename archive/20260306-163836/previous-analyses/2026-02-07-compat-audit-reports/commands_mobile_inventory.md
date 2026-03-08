# Commands Mobile Endpoint Inventory

## Deliverable A: Mobile API Calls for Slash Commands

Source: `mattermost-mobile/app/client/rest/`

| # | Method | Path Template | Mobile File | Line | Headers | Request Schema | Response Schema | Notes |
|---|--------|---------------|-------------|------|---------|----------------|-----------------|-------|
| 1 | GET | `/api/v4/commands` | integrations.ts | 21 | Auth token | Query: `team_id` | `Command[]` | Lists team's custom commands |
| 2 | POST | `/api/v4/commands` | integrations.ts | 49 | Auth token | `CreateCommand` | `Command` | Create new slash command |
| 3 | POST | `/api/v4/commands/execute` | integrations.ts | 42 | Auth token | `CommandArgs` | `CommandResponse` | Execute slash command |
| 4 | GET | `/api/v4/teams/{team_id}/commands/autocomplete` | integrations.ts | 36 | Auth token | Query: `page`, `per_page` | `Command[]` | List autocomplete-enabled commands |
| 5 | GET | `/api/v4/teams/{team_id}/commands/autocomplete_suggestions` | integrations.ts | 29 | Auth token | Query: `user_input`, `channel_id`, `root_id` | `Suggestion[]` | Get suggestions for partial input |
| 6 | PUT | `/api/v4/commands/{command_id}` | base.ts | 130 | Auth token | `Command` | `Command` | Update command (inferred) |
| 7 | DELETE | `/api/v4/commands/{command_id}` | base.ts | 130 | Auth token | - | `{status: "OK"}` | Delete command (inferred) |
| 8 | GET | `/api/v4/commands/{command_id}` | base.ts | 130 | Auth token | - | `Command` | Get single command (inferred) |
| 9 | PUT | `/api/v4/commands/{command_id}/move` | base.ts | 130 | Auth token | `{team_id}` | `{status: "OK"}` | Move command to team |
| 10 | PUT | `/api/v4/commands/{command_id}/regen_token` | base.ts | 130 | Auth token | - | `{token}` | Regenerate command token |

---

## Request/Response Schemas

### CommandArgs (Execute Request)
```json
{
  "command": "/trigger arguments",  // Required, starts with /
  "channel_id": "26char_id",        // Required
  "team_id": "26char_id",           // Optional, derived from channel if not DM/GM
  "root_id": "26char_id",           // Optional, for threaded commands
  "parent_id": "26char_id"          // Optional, alias for root_id
}
```
Source: `mattermost-mobile/app/actions/remote/command.ts:41-46`

### CommandResponse
```json
{
  "response_type": "ephemeral|in_channel",
  "text": "Response message",
  "username": "",
  "icon_url": "",
  "type": "",
  "props": {},
  "goto_location": "",
  "trigger_id": "",
  "skip_slack_parsing": false,
  "attachments": [],
  "extra_responses": []
}
```
Source: `mattermost/server/public/model/command_response.go:19-32`

### Command (Model)
```json
{
  "id": "26char_id",
  "token": "26char_token",
  "create_at": 1700000000000,
  "update_at": 1700000000000,
  "delete_at": 0,
  "creator_id": "26char_id",
  "team_id": "26char_id",
  "trigger": "remind",
  "method": "P",          // P=POST, G=GET
  "username": "",
  "icon_url": "",
  "auto_complete": true,
  "auto_complete_desc": "Set a reminder",
  "auto_complete_hint": "[time] [message]",
  "display_name": "Reminder",
  "description": "Creates reminders",
  "url": "http://integrations/remind",
  "plugin_id": ""
}
```
Source: `mattermost/server/public/model/command.go:18-42`

### AutocompleteSuggestion
```json
{
  "Complete": "/remind",
  "Suggestion": "remind",
  "Hint": "[time] [message]",
  "Description": "Set a reminder",
  "IconData": ""
}
```
Source: `mattermost/server/channels/api4/command.go:493`

---

## Mobile Usage Flow

### 1. Command Execution (command.ts:24)
```typescript
export const executeCommand = async (serverUrl, intl, message, channelId, rootId) => {
  // 1. Get team_id from channel or current team
  // 2. Build CommandArgs
  // 3. Check if it's an App command (plugin system)
  // 4. Parse /code special case (preserve trailing spaces)
  // 5. POST to /commands/execute
  // 6. Handle trigger_id for interactive dialogs
}
```

### 2. Autocomplete Suggestions (command.ts:175)
```typescript
export const fetchSuggestions = async (serverUrl, term, teamId, channelId, rootId) => {
  // GET /teams/{teamId}/commands/autocomplete_suggestions
  // Query: user_input, channel_id, root_id
}
```

### 3. List Commands (command.ts:165)
```typescript
export const fetchCommands = async (serverUrl, teamId) => {
  // GET /commands?team_id={teamId}
}
```

---

## Error Handling

Mobile expects standard Mattermost error format:
```json
{
  "id": "api.command.execute_command.start.app_error",
  "message": "Human readable message",
  "detailed_error": "Stack trace or details",
  "request_id": "abc123",
  "status_code": 400
}
```

Common error cases:
- 400: Invalid command format (doesn't start with /)
- 400: Invalid channel_id
- 403: No permission to execute in channel
- 404: Command not found (for custom commands)
