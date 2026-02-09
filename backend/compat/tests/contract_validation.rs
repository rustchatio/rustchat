//! Mattermost compatibility contract tests
//!
//! These tests verify that RustChat API responses match
//! the expected shapes for Mattermost Mobile compatibility.

use serde_json::Value;

/// Verify User response has all required fields for mobile
pub fn validate_user_response(user: &Value) -> Vec<String> {
    let mut errors = vec![];
    
    let required_fields = [
        "id",
        "create_at", 
        "update_at",
        "username",
        "auth_service",
        "roles",
        "nickname",
    ];
    
    for field in required_fields {
        if user.get(field).is_none() {
            errors.push(format!("Missing required field: {}", field));
        }
    }
    
    // Validate id format (26-char base62)
    if let Some(id) = user.get("id").and_then(|v| v.as_str()) {
        if id.len() != 26 {
            errors.push(format!("Invalid id length: expected 26, got {}", id.len()));
        }
    }
    
    // Validate timestamps are numbers
    for field in ["create_at", "update_at", "delete_at"] {
        if let Some(val) = user.get(field) {
            if !val.is_i64() && !val.is_u64() {
                errors.push(format!("{} should be integer timestamp", field));
            }
        }
    }
    
    errors
}

/// Verify Team response has all required fields
pub fn validate_team_response(team: &Value) -> Vec<String> {
    let mut errors = vec![];
    
    let required_fields = [
        "id",
        "create_at",
        "update_at",
        "name",
        "display_name",
        "type",
    ];
    
    for field in required_fields {
        if team.get(field).is_none() {
            errors.push(format!("Missing required field: {}", field));
        }
    }
    
    // Validate team type
    if let Some(team_type) = team.get("type").and_then(|v| v.as_str()) {
        if team_type != "O" && team_type != "I" {
            errors.push(format!("Invalid team type: {}", team_type));
        }
    }
    
    errors
}

/// Verify Channel response has all required fields
pub fn validate_channel_response(channel: &Value) -> Vec<String> {
    let mut errors = vec![];
    
    let required_fields = [
        "id",
        "create_at",
        "update_at",
        "team_id",
        "type",
        "name",
        "display_name",
    ];
    
    for field in required_fields {
        if channel.get(field).is_none() {
            errors.push(format!("Missing required field: {}", field));
        }
    }
    
    // Validate channel type
    if let Some(ch_type) = channel.get("type").and_then(|v| v.as_str()) {
        let valid_types = ["O", "P", "D", "G"];
        if !valid_types.contains(&ch_type) {
            errors.push(format!("Invalid channel type: {}", ch_type));
        }
    }
    
    errors
}

/// Verify Post response has all required fields
pub fn validate_post_response(post: &Value) -> Vec<String> {
    let mut errors = vec![];
    
    let required_fields = [
        "id",
        "create_at",
        "update_at",
        "user_id",
        "channel_id",
        "message",
        "type",
    ];
    
    for field in required_fields {
        if post.get(field).is_none() {
            errors.push(format!("Missing required field: {}", field));
        }
    }
    
    errors
}

/// Verify error response matches MM format
pub fn validate_error_response(error: &Value) -> Vec<String> {
    let mut errors = vec![];
    
    let required_fields = [
        "id",
        "message",
        "status_code",
    ];
    
    for field in required_fields {
        if error.get(field).is_none() {
            errors.push(format!("Missing required error field: {}", field));
        }
    }
    
    // Validate status_code is number
    if let Some(status) = error.get("status_code") {
        if !status.is_i64() && !status.is_u64() {
            errors.push("status_code should be integer".to_string());
        }
    }
    
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_user_response() {
        let user = json!({
            "id": "12345678901234567890123456",
            "create_at": 1700000000000i64,
            "update_at": 1700000000000i64,
            "delete_at": 0,
            "username": "testuser",
            "auth_service": "",
            "email": "test@example.com",
            "nickname": "",
            "first_name": "",
            "last_name": "",
            "roles": "system_user",
            "locale": "en",
        });
        
        let errors = validate_user_response(&user);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_invalid_user_missing_fields() {
        let user = json!({
            "id": "12345678901234567890123456",
            "username": "testuser",
        });
        
        let errors = validate_user_response(&user);
        assert!(errors.iter().any(|e| e.contains("create_at")));
        assert!(errors.iter().any(|e| e.contains("roles")));
    }

    #[test]
    fn test_valid_error_response() {
        let error = json!({
            "id": "api.user.login.invalid_credentials",
            "message": "Invalid credentials",
            "detailed_error": "",
            "request_id": "abc123",
            "status_code": 401
        });
        
        let errors = validate_error_response(&error);
        assert!(errors.is_empty(), "Expected no errors, got: {:?}", errors);
    }
}
