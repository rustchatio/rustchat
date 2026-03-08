# Commands Test Plan

## Deliverable E: Verification Strategy

---

## 1. Contract Tests (Schema Validation)

### Test Location
`compat/tests/commands_contract.rs`

### Tests

```rust
#[cfg(test)]
mod commands_contract_tests {
    use serde_json::json;
    use jsonschema::{Draft, JSONSchema};
    
    // Golden schemas derived from Mattermost responses
    
    #[test]
    fn test_command_response_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "response_type": {"type": "string", "enum": ["ephemeral", "in_channel"]},
                "text": {"type": "string"},
                "username": {"type": ["string", "null"]},
                "icon_url": {"type": ["string", "null"]},
                "channel_id": {"type": ["string", "null"]},
                "goto_location": {"type": ["string", "null"]},
                "trigger_id": {"type": ["string", "null"]},
                "skip_slack_parsing": {"type": ["boolean", "null"]},
                "attachments": {"type": ["array", "null"]},
                "extra_responses": {"type": ["array", "null"]}
            },
            "required": ["response_type", "text"]
        });
        
        let response = execute_command("/echo hello");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&response));
    }
    
    #[test]
    fn test_command_model_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "minLength": 26, "maxLength": 26},
                "token": {"type": "string", "minLength": 26, "maxLength": 26},
                "create_at": {"type": "integer"},
                "update_at": {"type": "integer"},
                "delete_at": {"type": "integer"},
                "creator_id": {"type": "string"},
                "team_id": {"type": "string"},
                "trigger": {"type": "string", "minLength": 1, "maxLength": 128},
                "method": {"type": "string", "enum": ["P", "G"]},
                "auto_complete": {"type": "boolean"},
                "url": {"type": "string", "format": "uri"}
            },
            "required": ["id", "token", "trigger", "team_id", "url", "method"]
        });
        
        let command = get_command(command_id);
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&command));
    }
    
    #[test]
    fn test_autocomplete_suggestion_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "Complete": {"type": "string", "pattern": "^/.*"},
                "Suggestion": {"type": "string"},
                "Hint": {"type": "string"},
                "Description": {"type": "string"},
                "IconData": {"type": "string"}
            },
            "required": ["Complete", "Suggestion"]
        });
        
        let suggestions = get_suggestions("/ec");
        for suggestion in suggestions {
            let compiled = JSONSchema::compile(&schema).unwrap();
            assert!(compiled.is_valid(&suggestion));
        }
    }
    
    #[test]
    fn test_execute_command_error_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "message": {"type": "string"},
                "detailed_error": {"type": ["string", "null"]},
                "request_id": {"type": ["string", "null"]},
                "status_code": {"type": "integer"}
            },
            "required": ["id", "message", "status_code"]
        });
        
        // Test invalid command
        let error = execute_command_expect_error("not-a-command");
        let compiled = JSONSchema::compile(&schema).unwrap();
        assert!(compiled.is_valid(&error));
    }
}
```

### Run Command
```bash
cd /Users/scolak/Projects/rustchat/backend
cargo test commands_contract
```

---

## 2. Behavioral Tests (End-to-End)

### Test Location
`compat/tests/commands_e2e.rs`

### Tests

```rust
#[tokio::test]
async fn test_builtin_echo_command() {
    let client = setup_test_client().await;
    
    // Login and get token
    let token = login(&client, "testuser", "password").await;
    
    // Execute /echo command
    let response = client.post("/api/v4/commands/execute")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "command": "/echo Hello World",
            "channel_id": CHANNEL_ID
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: CommandResponse = response.json().await.unwrap();
    assert_eq!(body.response_type, "ephemeral");
    assert_eq!(body.text, "Echo: Hello World");
}

#[tokio::test]
async fn test_builtin_shrug_command_creates_post() {
    let client = setup_test_client().await;
    let token = login(&client, "testuser", "password").await;
    
    // Execute /shrug command (in_channel type)
    let response = client.post("/api/v4/commands/execute")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "command": "/shrug whatever",
            "channel_id": CHANNEL_ID
        }))
        .send()
        .await
        .unwrap();
    
    let body: CommandResponse = response.json().await.unwrap();
    assert_eq!(body.response_type, "in_channel");
    assert!(body.text.contains("¯\\_(ツ)_/¯"));
    
    // Verify post was created in channel
    let posts = get_channel_posts(&client, &token, CHANNEL_ID).await;
    assert!(posts.iter().any(|p| p.message.contains("¯\\_(ツ)_/¯")));
}

#[tokio::test]
async fn test_custom_command_http_execution() {
    // Start mock server
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/webhook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response_type": "ephemeral",
            "text": "Custom response"
        })))
        .mount(&mock_server)
        .await;
    
    let client = setup_test_client().await;
    let token = login(&client, "admin", "password").await;
    
    // Create custom command
    let create_resp = client.post("/api/v4/commands")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "team_id": TEAM_ID,
            "trigger": "custom",
            "url": format!("{}/webhook", mock_server.uri()),
            "method": "P"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(create_resp.status(), 201);
    
    // Execute custom command
    let exec_resp = client.post("/api/v4/commands/execute")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "command": "/custom test args",
            "channel_id": CHANNEL_ID,
            "team_id": TEAM_ID
        }))
        .send()
        .await
        .unwrap();
    
    let body: CommandResponse = exec_resp.json().await.unwrap();
    assert_eq!(body.text, "Custom response");
    
    // Verify mock received correct payload
    let received = mock_server.received_requests().await.unwrap();
    assert_eq!(received.len(), 1);
    let req = &received[0];
    assert!(req.body_string().contains("token="));
    assert!(req.body_string().contains("text=test%20args"));
}

#[tokio::test]
async fn test_autocomplete_returns_matching_commands() {
    let client = setup_test_client().await;
    let token = login(&client, "testuser", "password").await;
    
    let response = client.get(&format!(
        "/api/v4/teams/{}/commands/autocomplete_suggestions?user_input=/ec&channel_id={}",
        TEAM_ID, CHANNEL_ID
    ))
    .header("Authorization", format!("Bearer {}", token))
    .send()
    .await
    .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let suggestions: Vec<AutocompleteSuggestion> = response.json().await.unwrap();
    assert!(suggestions.iter().any(|s| s.Suggestion == "echo"));
    
    for s in &suggestions {
        assert!(s.Complete.starts_with("/"));
        assert!(s.Suggestion.starts_with("ec"));
    }
}

#[tokio::test]
async fn test_command_permission_denied() {
    let client = setup_test_client().await;
    let token = login(&client, "testuser", "password").await;
    
    // Try to execute in channel user is not member of
    let response = client.post("/api/v4/commands/execute")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "command": "/echo test",
            "channel_id": "nonmember_channel_id"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 403);
}
```

### Run Command
```bash
cargo test commands_e2e -- --test-threads=1
```

---

## 3. Trace-Based Replay Tests

### Capture Trace from Mattermost
```bash
# Start mitmproxy in reverse mode
mitmproxy --mode reverse:https://mattermost.example.com -w commands_trace.mitm

# Configure mobile app to use proxy
# Execute various commands:
# - /echo test
# - /shrug whatever  
# - /help
# - Invalid command
# - Custom command
```

### Replay Against RustChat
```bash
cd /Users/scolak/Projects/rustchat/backend/compat

python3 replay_traces.py \
  --trace traces/commands_trace.mitm \
  --target http://localhost:8080 \
  --filter "/api/v4/commands" \
  --output reports/commands_trace_diff.json
```

### Diff Analysis Script
```python
# replay_commands.py
import json

def analyze_diff(diff_file):
    with open(diff_file) as f:
        diffs = json.load(f)
    
    for diff in diffs:
        # Ignore expected differences
        if is_expected_diff(diff):
            continue
        
        print(f"MISMATCH: {diff['endpoint']}")
        print(f"  Expected: {diff['expected']}")
        print(f"  Actual: {diff['actual']}")

def is_expected_diff(diff):
    # IDs and timestamps are expected to differ
    if 'id' in diff['field'] or 'at' in diff['field']:
        return True
    return False
```

---

## 4. Unit Tests

### Location
`src/services/commands.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_command_trigger() {
        assert_eq!(parse_trigger("/echo hello"), ("echo", "hello"));
        assert_eq!(parse_trigger("/call"), ("call", ""));
        assert_eq!(parse_trigger("/remind in 1 hour check email"), ("remind", "in 1 hour check email"));
    }
    
    #[test]
    fn test_command_validation() {
        assert!(is_valid_command("/echo"));
        assert!(!is_valid_command("echo"));
        assert!(!is_valid_command(""));
        assert!(!is_valid_command("/"));
    }
    
    #[test]
    fn test_trigger_id_format() {
        let id = generate_trigger_id();
        assert_eq!(id.len(), 26);
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric()));
    }
    
    #[test]
    fn test_command_method_conversion() {
        assert_eq!(to_mm_method("POST"), "P");
        assert_eq!(to_mm_method("GET"), "G");
        assert_eq!(from_mm_method("P"), "POST");
        assert_eq!(from_mm_method("G"), "GET");
    }
}
```

---

## 5. CI Integration

### GitHub Actions Workflow
```yaml
# .github/workflows/commands-compat.yml
name: Commands Compatibility

on:
  push:
    paths:
      - 'src/api/v4/commands.rs'
      - 'src/api/integrations.rs'
      - 'src/services/commands.rs'
      - 'compat/tests/commands_*.rs'

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        ports:
          - 5432:5432
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Run contract tests
        run: cargo test commands_contract
      
      - name: Run e2e tests
        run: cargo test commands_e2e -- --test-threads=1
      
      - name: Run trace replay
        run: |
          cargo run --release &
          sleep 5
          cd compat
          python3 replay_traces.py --trace traces/commands_baseline.mitm --target http://localhost:8080
```

---

## Test Files Summary

| File | Type | Description |
|------|------|-------------|
| `compat/tests/commands_contract.rs` | Contract | JSON schema validation |
| `compat/tests/commands_e2e.rs` | E2E | Full flow behavioral tests |
| `compat/traces/commands_trace.mitm` | Trace | Captured mobile session |
| `compat/replay_commands.py` | Script | Trace replay and diff |
| `src/services/commands.rs` | Unit | Function-level unit tests |
| `.github/workflows/commands-compat.yml` | CI | Automated test pipeline |

---

## Runnable Commands

```bash
# All commands tests
cargo test commands

# Specific test categories
cargo test commands_contract
cargo test commands_e2e
cargo test commands_unit

# With coverage
cargo tarpaulin --out Html --include-files "*commands*"

# Trace replay
cd compat && python3 replay_traces.py --trace traces/commands.mitm --target http://localhost:8080
```
