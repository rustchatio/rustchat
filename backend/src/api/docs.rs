//! OpenAPI 3.1.1 documentation
//!
//! Provides API specification at /api/openapi.json
//!
//! Note: Swagger UI integration is pending due to axum version compatibility.
//! The OpenAPI spec can be viewed using external tools like Swagger Editor.

use axum::{
    response::Json,
    routing::get,
    Router,
};
use utoipa::OpenApi;

use crate::api::AppState;

/// Main API documentation struct
/// 
/// This struct defines all the API endpoints, schemas, and security schemes
/// for the RustChat API. It uses utoipa's derive macro to generate the
/// OpenAPI 3.1.1 specification at compile time.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "RustChat API",
        version = "5.0.0",
        description = "Self-hosted, enterprise-ready team collaboration platform API. Mattermost-compatible.",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT"),
        contact(name = "RustChat Team", url = "https://github.com/rustchat/rustchat"),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management"),
        (name = "channels", description = "Channel operations"),
        (name = "posts", description = "Posts and messaging"),
        (name = "teams", description = "Team/Workspace management"),
        (name = "files", description = "File upload and download"),
        (name = "search", description = "Search functionality"),
        (name = "system", description = "System health and status"),
        (name = "mcp", description = "Model Context Protocol for AI agents"),
        (name = "a2a", description = "Agent-to-Agent protocol for multi-agent collaboration"),
    ),
)]
pub struct ApiDoc;

/// Handler for OpenAPI JSON endpoint
async fn openapi_json() -> Json<serde_json::Value> {
    let spec = ApiDoc::openapi();
    let json_str = match spec.to_json() {
        Ok(s) => s,
        Err(_) => return Json(serde_json::Value::Null),
    };
    Json(serde_json::from_str(&json_str).unwrap_or_default())
}

/// Create routes for API documentation
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_json))
}

/// Export OpenAPI spec to JSON string
/// 
/// This can be used to generate SDKs or for documentation
#[allow(dead_code)]
pub fn export_openapi_spec() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap_or_default()
}

/// Export OpenAPI spec to file
/// 
/// Used for SDK generation pipeline
#[allow(dead_code)]
pub fn write_openapi_spec_to_file(path: &str) -> std::io::Result<()> {
    let spec = export_openapi_spec();
    std::fs::write(path, spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_valid() {
        let spec = ApiDoc::openapi();
        let json = spec.to_json().expect("Failed to serialize spec");
        assert!(!json.is_empty());
        
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");
        assert_eq!(parsed["openapi"], "3.1.0");
        assert_eq!(parsed["info"]["title"], "RustChat API");
        assert_eq!(parsed["info"]["version"], "5.0.0");
    }

    #[test]
    fn test_tags_present() {
        let spec = ApiDoc::openapi();
        let json = spec.to_json().expect("Failed to serialize spec");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        let tags = parsed["tags"].as_array().expect("Tags should be an array");
        assert!(tags.len() >= 5, "Should have at least 5 tags");
        
        let tag_names: Vec<&str> = tags
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        
        assert!(tag_names.contains(&"auth"));
        assert!(tag_names.contains(&"users"));
        assert!(tag_names.contains(&"channels"));
        assert!(tag_names.contains(&"posts"));
        assert!(tag_names.contains(&"mcp"));
        assert!(tag_names.contains(&"a2a"));
    }
}
