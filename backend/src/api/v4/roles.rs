use crate::api::AppState;
use crate::error::ApiResult;
use crate::mattermost_compat::models::Role;
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(get_roles))
        .route("/roles/names", post(get_roles_by_names))
        .route("/roles/{role_id}", get(get_role))
        .route("/roles/name/{role_name}", get(get_role_by_name))
        .route("/roles/{role_id}/patch", put(patch_role))
}

fn get_hardcoded_role(name: &str) -> Option<Role> {
    match name {
        "system_user" => Some(Role {
            id: "system_user_id".to_string(),
            name: "system_user".to_string(),
            display_name: "System User".to_string(),
            description: "Default System User Role".to_string(),
            permissions: vec![
                "create_direct_channel".to_string(),
                "create_group_channel".to_string(),
                "list_public_teams".to_string(),
                "join_public_teams".to_string(),
                "view_team".to_string(),
                "edit_self".to_string(),
            ],
            scheme_managed: true,
        }),
        "team_user" => Some(Role {
            id: "team_user_id".to_string(),
            name: "team_user".to_string(),
            display_name: "Team User".to_string(),
            description: "Default Team User Role".to_string(),
            permissions: vec![
                "view_team".to_string(),
                "list_team_channels".to_string(),
                "join_public_channels".to_string(),
                "create_public_channel".to_string(),
                "create_private_channel".to_string(),
            ],
            scheme_managed: true,
        }),
        "channel_user" => Some(Role {
            id: "channel_user_id".to_string(),
            name: "channel_user".to_string(),
            display_name: "Channel User".to_string(),
            description: "Default Channel User Role".to_string(),
            permissions: vec![
                "read_channel".to_string(),
                "create_post".to_string(),
                "add_reaction".to_string(),
                "remove_reaction".to_string(),
                "upload_file".to_string(),
                "edit_post".to_string(),
                "delete_post".to_string(),
                "use_slash_commands".to_string(),
            ],
            scheme_managed: true,
        }),
        "system_admin" => Some(Role {
            id: "system_admin_id".to_string(),
            name: "system_admin".to_string(),
            display_name: "System Admin".to_string(),
            description: "Default System Admin Role".to_string(),
            permissions: vec!["manage_system".to_string(), "assign_system_admin_role".to_string()],
            scheme_managed: true,
        }),
        "team_admin" => Some(Role {
            id: "team_admin_id".to_string(),
            name: "team_admin".to_string(),
            display_name: "Team Admin".to_string(),
            description: "Default Team Admin Role".to_string(),
            permissions: vec![
                "view_team".to_string(),
                "list_team_channels".to_string(),
                "join_public_channels".to_string(),
                "create_public_channel".to_string(),
                "create_private_channel".to_string(),
                "manage_team".to_string(),
                "add_user_to_team".to_string(),
                "remove_user_from_team".to_string(),
            ],
            scheme_managed: true,
        }),
        "channel_admin" => Some(Role {
            id: "channel_admin_id".to_string(),
            name: "channel_admin".to_string(),
            display_name: "Channel Admin".to_string(),
            description: "Default Channel Admin Role".to_string(),
            permissions: vec![
                "read_channel".to_string(),
                "create_post".to_string(),
                "add_reaction".to_string(),
                "remove_reaction".to_string(),
                "upload_file".to_string(),
                "edit_post".to_string(),
                "delete_post".to_string(),
                "use_slash_commands".to_string(),
                "manage_channel".to_string(),
                "manage_public_channel_members".to_string(),
                "manage_private_channel_members".to_string(),
            ],
            scheme_managed: true,
        }),
        _ => None,
    }
}

/// GET /api/v4/roles
async fn get_roles(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<Vec<Role>>> {
    // Standard Mattermost roles
    let roles = vec![
        get_hardcoded_role("system_user").unwrap(),
        get_hardcoded_role("team_user").unwrap(),
        get_hardcoded_role("channel_user").unwrap(),
        get_hardcoded_role("system_admin").unwrap(),
    ];
    Ok(Json(roles))
}

/// POST /api/v4/roles/names
async fn get_roles_by_names(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Json(names): Json<Vec<String>>,
) -> ApiResult<Json<Vec<Role>>> {
    let mut roles = Vec::new();
    for name in names {
        if let Some(role) = get_hardcoded_role(&name) {
            roles.push(role);
        }
    }
    Ok(Json(roles))
}

/// GET /api/v4/roles/{role_id}
async fn get_role(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(role_id): Path<String>,
) -> ApiResult<Json<Role>> {
    // Match by ID or name for simplicity in this stub
    let role_name = match role_id.as_str() {
        "system_user_id" => "system_user",
        "team_user_id" => "team_user",
        "channel_user_id" => "channel_user",
        "system_admin_id" => "system_admin",
        _ => &role_id,
    };

    if let Some(role) = get_hardcoded_role(role_name) {
        Ok(Json(role))
    } else {
        Err(crate::error::AppError::NotFound("Role not found".to_string()))
    }
}

/// GET /api/v4/roles/name/{role_name}
async fn get_role_by_name(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(role_name): Path<String>,
) -> ApiResult<Json<Role>> {
    if let Some(role) = get_hardcoded_role(&role_name) {
        Ok(Json(role))
    } else {
        Err(crate::error::AppError::NotFound("Role not found".to_string()))
    }
}

/// PUT /api/v4/roles/{role_id}/patch
async fn patch_role(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(_role_id): Path<String>,
    Json(_patch): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    // We don't support patching hardcoded roles, just return a success-like stub
    Ok(Json(json!({"status": "OK"})))
}
