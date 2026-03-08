# Commands Implementation Plan

## Deliverable D: Detailed Implementation Tasks

---

## Overview

Implement full Mattermost slash commands compatibility in RustChat by:
1. Aligning data models with Mattermost
2. Completing v4 API route handlers
3. Implementing custom command HTTP execution
4. Adding missing built-in commands
5. Proper autocomplete support

---

## Phase 1: Data Model Alignment

### Task 1.1: Update Command Model

**File**: `src/models/integration.rs`

Add missing fields to `SlashCommand`:

```rust
pub struct SlashCommand {
    pub id: Uuid,
    pub token: String,
    pub create_at: i64,         // ADD: milliseconds
    pub update_at: i64,         // ADD: milliseconds
    pub delete_at: i64,         // ADD: soft delete
    pub creator_id: Uuid,
    pub team_id: Uuid,
    pub trigger: String,
    pub method: String,         // CHANGE: "P" or "G" format
    pub username: Option<String>,  // ADD
    pub icon_url: Option<String>,
    pub auto_complete: bool,
    pub auto_complete_desc: Option<String>,  // ADD
    pub auto_complete_hint: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub plugin_id: Option<String>,  // ADD (always empty for user commands)
}
```

### Task 1.2: Update CommandResponse Model

**File**: `src/models/integration.rs`

```rust
pub struct CommandResponse {
    pub response_type: String,
    pub text: String,
    pub username: Option<String>,
    pub channel_id: Option<String>,    // ADD
    pub icon_url: Option<String>,
    #[serde(rename = "type")]
    pub response_subtype: Option<String>,  // ADD
    pub props: Option<serde_json::Value>,  // ADD
    pub goto_location: Option<String>,
    pub trigger_id: Option<String>,        // ADD - critical
    pub skip_slack_parsing: Option<bool>,  // ADD
    pub attachments: Option<Vec<SlackAttachment>>,  // CHANGE type
    pub extra_responses: Option<Vec<CommandResponse>>,  // ADD
}
```

### Task 1.3: Add SlackAttachment Type

**File**: `src/models/integration.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAttachment {
    pub fallback: Option<String>,
    pub color: Option<String>,
    pub pretext: Option<String>,
    pub author_name: Option<String>,
    pub author_link: Option<String>,
    pub author_icon: Option<String>,
    pub title: Option<String>,
    pub title_link: Option<String>,
    pub text: Option<String>,
    pub fields: Option<Vec<AttachmentField>>,
    pub image_url: Option<String>,
    pub thumb_url: Option<String>,
    pub footer: Option<String>,
    pub footer_icon: Option<String>,
    pub actions: Option<Vec<AttachmentAction>>,
}
```

### Task 1.4: Database Migration

**File**: `migrations/YYYYMMDDHHMMSS_update_slash_commands.sql`

```sql
ALTER TABLE slash_commands
ADD COLUMN IF NOT EXISTS delete_at BIGINT DEFAULT 0,
ADD COLUMN IF NOT EXISTS username VARCHAR(64),
ADD COLUMN IF NOT EXISTS auto_complete_desc TEXT,
ADD COLUMN IF NOT EXISTS plugin_id VARCHAR(64);

-- Update method format
UPDATE slash_commands SET method = 'P' WHERE method = 'POST';
UPDATE slash_commands SET method = 'G' WHERE method = 'GET';

-- Add index for autocomplete lookups
CREATE INDEX IF NOT EXISTS idx_slash_commands_team_trigger 
ON slash_commands(team_id, trigger) WHERE delete_at = 0;
```

---

## Phase 2: V4 API Route Handlers

### Task 2.1: Complete list_commands

**File**: `src/api/v4/commands.rs`

Replace stub with:

```rust
async fn list_commands(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<CommandsQuery>,
) -> ApiResult<Json<Vec<MmCommand>>> {
    let team_id = query.team_id
        .and_then(|t| parse_mm_or_uuid(&t))
        .ok_or_else(|| AppError::BadRequest("team_id required".into()))?;
    
    // Check team membership
    // ...
    
    // Fetch custom commands
    let custom: Vec<SlashCommand> = sqlx::query_as(
        "SELECT * FROM slash_commands WHERE team_id = $1 AND delete_at = 0 ORDER BY trigger"
    )
    .bind(team_id)
    .fetch_all(&state.db)
    .await?;
    
    // Add built-in commands
    let mut commands: Vec<MmCommand> = get_builtin_commands();
    commands.extend(custom.into_iter().map(to_mm_command));
    
    Ok(Json(commands))
}
```

### Task 2.2: Add listAutocompleteCommands endpoint

**File**: `src/api/v4/commands.rs`

Add route in `router()`:

```rust
.route("/teams/{team_id}/commands/autocomplete", get(list_autocomplete_commands))
```

Handler:

```rust
async fn list_autocomplete_commands(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(team): Path<TeamPath>,
    Query(query): Query<PaginationQuery>,
) -> ApiResult<Json<Vec<MmCommand>>> {
    let team_id = parse_mm_or_uuid(&team.team_id)?;
    
    // Get only commands with auto_complete = true
    let commands: Vec<SlashCommand> = sqlx::query_as(
        "SELECT * FROM slash_commands 
         WHERE team_id = $1 AND delete_at = 0 AND auto_complete = true 
         ORDER BY trigger
         LIMIT $2 OFFSET $3"
    )
    .bind(team_id)
    .bind(query.per_page.unwrap_or(60))
    .bind(query.page.unwrap_or(0) * query.per_page.unwrap_or(60))
    .fetch_all(&state.db)
    .await?;
    
    let mut result = get_builtin_commands_autocomplete();
    result.extend(commands.into_iter().map(to_mm_command));
    
    Ok(Json(result))
}
```

### Task 2.3: Complete autocomplete_suggestions

**File**: `src/api/v4/commands.rs`

Replace partial implementation:

```rust
async fn autocomplete_suggestions(
    State(state): State<AppState>,
    Path(team): Path<TeamPath>,
    Query(query): Query<AutocompleteQuery>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<AutocompleteSuggestion>>> {
    let team_id = parse_mm_or_uuid(&team.team_id)?;
    let input = query.user_input.trim_start_matches('/');
    
    // Get all autocomplete commands
    let commands: Vec<SlashCommand> = sqlx::query_as(
        "SELECT * FROM slash_commands 
         WHERE team_id = $1 AND delete_at = 0 AND auto_complete = true 
         AND trigger LIKE $2
         ORDER BY trigger"
    )
    .bind(team_id)
    .bind(format!("{}%", input))
    .fetch_all(&state.db)
    .await?;
    
    let mut suggestions = Vec::new();
    
    // Add matching built-ins
    for builtin in get_builtin_commands() {
        if builtin.trigger.starts_with(input) {
            suggestions.push(AutocompleteSuggestion {
                complete: format!("/{}", builtin.trigger),
                suggestion: builtin.trigger.clone(),
                hint: builtin.auto_complete_hint.unwrap_or_default(),
                description: builtin.auto_complete_desc.unwrap_or_default(),
                icon_data: String::new(),
            });
        }
    }
    
    // Add matching custom commands
    for cmd in commands {
        suggestions.push(AutocompleteSuggestion {
            complete: format!("/{}", cmd.trigger),
            suggestion: cmd.trigger,
            hint: cmd.auto_complete_hint.unwrap_or_default(),
            description: cmd.auto_complete_desc.or(cmd.description).unwrap_or_default(),
            icon_data: String::new(),
        });
    }
    
    Ok(Json(suggestions))
}
```

### Task 2.4: Wire createCommand to v4 router

**File**: `src/api/v4/commands.rs`

Add route:

```rust
.route("/commands", get(list_commands).post(create_command))
```

Handler uses existing `integrations.rs::create_slash_command` logic but with v4 auth.

---

## Phase 3: Custom Command HTTP Execution

### Task 3.1: Create CommandExecutor Service

**File**: `src/services/commands.rs` (NEW)

```rust
use reqwest::{Client, header};
use std::time::Duration;

pub struct CommandExecutor {
    client: Client,
}

impl CommandExecutor {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap();
        Self { client }
    }
    
    pub async fn execute(
        &self,
        command: &SlashCommand,
        args: &CommandArgs,
        user: &User,
        channel: &Channel,
        team: &Team,
    ) -> Result<CommandResponse, AppError> {
        let payload = OutgoingCommandPayload {
            token: command.token.clone(),
            team_id: encode_mm_id(team.id),
            team_domain: team.name.clone(),
            channel_id: encode_mm_id(channel.id),
            channel_name: channel.name.clone(),
            user_id: encode_mm_id(user.id),
            user_name: user.username.clone(),
            command: format!("/{}", command.trigger),
            text: args.text.clone(),
            response_url: format!("{}/hooks/commands/{}", args.site_url, command.id),
            trigger_id: generate_trigger_id(),
        };
        
        let response = match command.method.as_str() {
            "G" => self.client.get(&command.url)
                .query(&payload)
                .send()
                .await,
            _ => self.client.post(&command.url)
                .form(&payload)
                .send()
                .await,
        };
        
        match response {
            Ok(resp) => {
                let text = resp.text().await?;
                parse_command_response(&text)
            }
            Err(e) => Ok(CommandResponse {
                response_type: "ephemeral".into(),
                text: format!("Command error: {}", e),
                ..Default::default()
            }),
        }
    }
}
```

### Task 3.2: Integrate into execute_command

**File**: `src/api/integrations.rs`

Update `execute_command_internal`:

```rust
// After built-in command check:
// Look up custom command
let custom_command: Option<SlashCommand> = sqlx::query_as(
    "SELECT * FROM slash_commands WHERE team_id = $1 AND trigger = $2 AND delete_at = 0"
)
.bind(team_id)
.bind(trigger)
.fetch_optional(&state.db)
.await?;

if let Some(cmd) = custom_command {
    return state.command_executor.execute(&cmd, &args, &user, &channel, &team).await;
}
```

---

## Phase 4: Built-in Commands

### Task 4.1: Create BuiltinCommands Module

**File**: `src/services/builtin_commands.rs` (NEW)

```rust
pub fn get_builtin_commands() -> Vec<BuiltinCommand> {
    vec![
        BuiltinCommand::new("call", "Start a video call", "[end]"),
        BuiltinCommand::new("echo", "Echo your message", "[message]"),
        BuiltinCommand::new("shrug", "Append ¯\\_(ツ)_/¯", "[message]"),
        BuiltinCommand::new("me", "Action text formatting", "[action]"),
        BuiltinCommand::new("join", "Join a channel", "[channel]"),
        BuiltinCommand::new("leave", "Leave current channel", ""),
        BuiltinCommand::new("msg", "Send direct message", "@[username] [message]"),
        BuiltinCommand::new("mute", "Mute channel notifications", ""),
        BuiltinCommand::new("help", "Show available commands", ""),
        BuiltinCommand::new("away", "Set status to away", ""),
        BuiltinCommand::new("online", "Set status to online", ""),
        BuiltinCommand::new("dnd", "Set do not disturb", ""),
        BuiltinCommand::new("offline", "Set status to offline", ""),
    ]
}

pub async fn execute_builtin(
    state: &AppState,
    auth: &CommandAuth,
    trigger: &str,
    args: &str,
    channel_id: Uuid,
) -> Option<ApiResult<CommandResponse>> {
    match trigger {
        "me" => Some(execute_me(args)),
        "join" => Some(execute_join(state, auth, args).await),
        "leave" => Some(execute_leave(state, auth, channel_id).await),
        "msg" => Some(execute_msg(state, auth, args).await),
        "mute" => Some(execute_mute(state, auth, channel_id).await),
        "help" => Some(execute_help()),
        "away" | "online" | "dnd" | "offline" => Some(execute_status(state, auth, trigger).await),
        // ... existing call, echo, shrug from integrations.rs
        _ => None,
    }
}
```

---

## Phase 5: Response Handling

### Task 5.1: Ephemeral Post Support

**File**: `src/services/posts.rs`

```rust
pub async fn create_ephemeral_post(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
    message: String,
) -> ApiResult<()> {
    // Send via WebSocket to specific user only
    let event = WsEnvelope {
        event: "ephemeral_message".into(),
        channel_id: Some(channel_id),
        data: serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "channel_id": encode_mm_id(channel_id),
            "message": message,
        }),
        broadcast: Some(WsBroadcast {
            user_id: Some(user_id),
            ..Default::default()
        }),
    };
    state.ws_hub.broadcast(event).await;
    Ok(())
}
```

### Task 5.2: Trigger ID Generation

**File**: `src/services/commands.rs`

```rust
fn generate_trigger_id() -> String {
    // 26-char ID compatible with Mattermost
    crate::mattermost_compat::id::encode_mm_id(Uuid::new_v4())
}
```

---

## Files Summary

| File | Action | Description |
|------|--------|-------------|
| `src/models/integration.rs` | MODIFY | Add fields to SlashCommand, CommandResponse |
| `migrations/YYYYMMDD_update_slash_commands.sql` | NEW | Schema updates |
| `src/api/v4/commands.rs` | MODIFY | Complete all stub handlers |
| `src/services/commands.rs` | NEW | CommandExecutor for HTTP calls |
| `src/services/builtin_commands.rs` | NEW | Built-in command implementations |
| `src/services/posts.rs` | MODIFY | Add ephemeral post support |
| `src/api/integrations.rs` | MODIFY | Wire custom command execution |

---

## Effort Estimate

| Phase | Tasks | Hours |
|-------|-------|-------|
| Phase 1: Models | 4 | 2-3 |
| Phase 2: Routes | 4 | 3-4 |
| Phase 3: HTTP Execution | 2 | 2-3 |
| Phase 4: Built-ins | 1 | 2-3 |
| Phase 5: Response | 2 | 1-2 |
| **Total** | 13 | **10-15** |
