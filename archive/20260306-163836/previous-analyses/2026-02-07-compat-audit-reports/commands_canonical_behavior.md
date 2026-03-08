# Commands Canonical Behavior

## Deliverable B: Mattermost Server Command Implementation

Source: `mattermost/server/channels/api4/command.go`

---

## Route Registration (line 16-29)

```go
api.BaseRoutes.Commands.Handle("", createCommand).Methods(POST)
api.BaseRoutes.Commands.Handle("", listCommands).Methods(GET)
api.BaseRoutes.Commands.Handle("/execute", executeCommand).Methods(POST)
api.BaseRoutes.Command.Handle("", getCommand).Methods(GET)
api.BaseRoutes.Command.Handle("", updateCommand).Methods(PUT)
api.BaseRoutes.Command.Handle("/move", moveCommand).Methods(PUT)
api.BaseRoutes.Command.Handle("", deleteCommand).Methods(DELETE)
api.BaseRoutes.Team.Handle("/commands/autocomplete", listAutocompleteCommands).Methods(GET)
api.BaseRoutes.Team.Handle("/commands/autocomplete_suggestions", listCommandAutocompleteSuggestions).Methods(GET)
api.BaseRoutes.Command.Handle("/regen_token", regenCommandToken).Methods(PUT)
```

---

## 1. POST /commands/execute (line 357-429)

### Validation
```go
// Line 364
if len(commandArgs.Command) <= 1 || strings.Index(commandArgs.Command, "/") != 0 || !model.IsValidId(commandArgs.ChannelId) {
    c.Err = model.NewAppError("executeCommand", "api.command.execute_command.start.app_error", nil, "", http.StatusBadRequest)
}
```

### Permission Check
```go
// Line 374 - Must have PermissionCreatePost in channel
if ok, _ := c.App.SessionHasPermissionToChannel(c.AppContext, *c.AppContext.Session(), commandArgs.ChannelId, model.PermissionCreatePost); !ok {
    c.SetPermissionError(model.PermissionCreatePost)
}
```

### Team Resolution
```go
// Line 390-414
if channel.Type != model.ChannelTypeDirect && channel.Type != model.ChannelTypeGroup {
    // Take team_id from channel (ignore client-provided team_id)
    commandArgs.TeamId = channel.TeamId
} else {
    // DM/GM: validate user is member of the team_id they specified
}
```

### Execution Pipeline
1. Parse command trigger from `commandArgs.Command`
2. Check if built-in command → execute internally
3. Check if custom command for team → HTTP call to command URL
4. Return `CommandResponse` with `response_type`, `text`, `attachments`

### Response Status Codes
- 200: Success
- 400: Invalid command format, deleted channel
- 403: No permission to post in channel
- 500: Command execution failure

---

## 2. GET /commands (line 260-319)

### Parameters
- `team_id` (required query param)
- `custom_only` (optional, filters to user/team custom commands)

### Permission Checks
```go
// Line 269 - Must view team
if !c.App.SessionHasPermissionToTeam(*c.AppContext.Session(), teamId, model.PermissionViewTeam)

// Line 277 - Custom commands require ManageOwnSlashCommands
if !c.App.SessionHasPermissionToTeam(*c.AppContext.Session(), teamId, model.PermissionManageOwnSlashCommands)
```

### Logic
- If `custom_only=true`: Return only custom commands user can manage
- If `custom_only=false` and no ManageOwnSlashCommands: Return only system/autocomplete commands
- If `custom_only=false` and has ManageOwnSlashCommands: Return all (filtered by user if not ManageOthers)

---

## 3. POST /commands (line 31-81)

### Creation Logic
```go
// Line 43 - Require ManageOwnSlashCommands permission
if !c.App.SessionHasPermissionToTeam(*c.AppContext.Session(), cmd.TeamId, model.PermissionManageOwnSlashCommands)

// Line 49-62 - Handle creator_id assignment
if cmd.CreatorId != "" && cmd.CreatorId != userId {
    // Require ManageOthersSlashCommands to create for others
}
cmd.CreatorId = userId
```

### Response
- 201 Created on success
- Returns full Command object with generated `id` and `token`

---

## 4. GET /teams/{team_id}/commands/autocomplete_suggestions (line 454-502)

### Parameters
```go
// Line 470-474
userInput := query.Get("user_input")  // Required
channelId := query.Get("channel_id")
rootId := query.Get("root_id")
```

### Logic
```go
// Line 475
userInput = strings.TrimPrefix(userInput, "/")  // Strip leading slash

// Line 477 - Get all autocomplete commands for team
commands, appErr := c.App.ListAutocompleteCommands(c.Params.TeamId, c.AppContext.T)

// Line 493 - Generate suggestions based on partial input
suggestions := c.App.GetSuggestions(c.AppContext, commandArgs, commands, roleId)
```

### Response Format
```go
// Returns []AutocompleteSuggestion
{
    Complete: "/remind",      // Full command (starts with /)
    Suggestion: "remind",     // Trigger word only
    Hint: "[time] [message]",
    Description: "Set a reminder",
    IconData: ""
}
```

---

## 5. GET /teams/{team_id}/commands/autocomplete (line 432-451)

### Permission
```go
// Line 438 - Must view team
if !c.App.SessionHasPermissionToTeam(*c.AppContext.Session(), c.Params.TeamId, model.PermissionViewTeam)
```

### Response
Returns `[]Command` - all commands with `auto_complete=true` for the team.

---

## 6. PUT /commands/{command_id}/regen_token (line 505-552)

### Permission Checks
```go
// Line 524 - Require ManageOwnSlashCommands
// Line 532 - If not creator, require ManageOthersSlashCommands
```

### Response
```json
{"token": "new26chartoken"}
```

---

## Built-in Commands

Located in `mattermost/server/channels/app/command*.go`:

| Command | File | Behavior |
|---------|------|----------|
| `/away` | command_away.go | Set status to away |
| `/code` | command_code.go | Format as code block |
| `/collapse` | command_expand.go | Collapse image previews |
| `/dnd` | command_dnd.go | Set status to DND |
| `/echo` | command_echo.go | Echo message (ephemeral) |
| `/expand` | command_expand.go | Expand image previews |
| `/header` | command_channel_header.go | Set channel header |
| `/help` | command_help.go | Show help (ephemeral) |
| `/invite` | command_invite.go | Invite user to channel |
| `/join` | command_join.go | Join a channel |
| `/kick` | command_remove.go | Remove user from channel |
| `/leave` | command_leave.go | Leave current channel |
| `/logout` | command_logout.go | Logout session |
| `/me` | command_me.go | /me action formatting |
| `/msg` | command_msg.go | Direct message user |
| `/mute` | command_mute.go | Mute channel |
| `/offline` | command_offline.go | Set status offline |
| `/online` | command_online.go | Set status online |
| `/open` | command_join.go | Alias for /join |
| `/purpose` | command_channel_purpose.go | Set channel purpose |
| `/rename` | command_channel_rename.go | Rename channel |
| `/search` | command_search.go | Search messages |
| `/settings` | command_settings.go | Open settings |
| `/shortcuts` | command_shortcuts.go | Show keyboard shortcuts |
| `/shrug` | command_shrug.go | Append shrug emoji |
| `/status` | command_status.go | Set custom status |

---

## Custom Command Execution

```go
// mattermost/server/channels/app/command.go
func (a *App) ExecuteCommand(c request.CTX, commandArgs *model.CommandArgs) (*model.CommandResponse, *model.AppError) {
    // 1. Parse trigger from commandArgs.Command
    // 2. Check built-in commands first
    // 3. Look up custom command by (team_id, trigger)
    // 4. If custom command found:
    //    - Build outgoing payload with user/channel/team context
    //    - HTTP request to command.URL (GET or POST based on command.Method)
    //    - Parse response as CommandResponse
    //    - Handle response_type (ephemeral vs in_channel)
    // 5. Return response
}
```

### Outgoing Payload (to custom command URL)
```
token=WEBHOOK_TOKEN
team_id=...
team_domain=...
channel_id=...
channel_name=...
user_id=...
user_name=...
command=/trigger
text=arguments after trigger
response_url=...  // For delayed responses
trigger_id=...    // For interactive dialogs
```
