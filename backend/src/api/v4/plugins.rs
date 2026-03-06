use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::ApiResult;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

const CALLS_PLUGIN_ID: &str = "com.mattermost.calls";
const CALLS_PLUGIN_VERSION: &str = "0.28.0";
const CALLS_PLUGIN_MIN_SERVER_VERSION: &str = "7.0.0";
const CALLS_PLUGIN_NAME: &str = "Calls";
const CALLS_PLUGIN_DESCRIPTION: &str = "Mattermost Calls plugin for voice and video conferencing";
const FIRST_ADMIN_VISIT_MARKETPLACE_KEY: &str = "FirstAdminVisitMarketplace";
const FIRST_ADMIN_VISIT_MARKETPLACE_EVENT: &str = "first_admin_visit_marketplace_status_received";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/plugins", get(get_plugins).post(upload_plugin))
        .route("/plugins/install_from_url", post(install_plugin_from_url))
        .route(
            "/plugins/{plugin_id}",
            get(get_plugin_status).delete(remove_plugin),
        )
        .route("/plugins/{plugin_id}/enable", post(enable_plugin))
        .route("/plugins/{plugin_id}/disable", post(disable_plugin))
        .route("/plugins/statuses", get(get_plugin_statuses))
        .route("/plugins/webapp", get(get_webapp_plugins))
        .route(
            "/plugins/marketplace",
            get(get_marketplace_plugins).post(install_marketplace_plugin),
        )
        .route(
            "/plugins/marketplace/first_admin_visit",
            get(get_first_admin_visit_marketplace).post(first_admin_visit_marketplace),
        )
}

fn ensure_manage_system(auth: &crate::api::v4::extractors::MmAuthUser) -> ApiResult<()> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(crate::error::AppError::Forbidden(
            "Insufficient permissions to manage system plugins".to_string(),
        ));
    }

    Ok(())
}

/// GET /api/v4/plugins
async fn get_plugins(
    State(state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let calls_enabled = calls_enabled(&state).await?;
    let calls_summary = calls_plugin_summary();

    let (active, inactive) = if calls_enabled {
        (vec![calls_summary], vec![])
    } else {
        (vec![], vec![calls_summary])
    };

    Ok(Json(json!({
        "active": active,
        "inactive": inactive
    })))
}

/// POST /api/v4/plugins
async fn upload_plugin(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.plugins.upload.not_implemented.app_error",
        "Plugin upload is not implemented.",
        "POST /api/v4/plugins is not supported in this server.",
    ))
}

/// POST /api/v4/plugins/install_from_url
async fn install_plugin_from_url(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.plugins.install_from_url.not_implemented.app_error",
        "Plugin installation from URL is not implemented.",
        "POST /api/v4/plugins/install_from_url is not supported in this server.",
    ))
}

/// GET /api/v4/plugins/{plugin_id}
async fn get_plugin_status(
    State(state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(plugin_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    if plugin_id != CALLS_PLUGIN_ID {
        return Err(crate::error::AppError::NotFound(format!(
            "Plugin {plugin_id} not found"
        )));
    }

    let is_active = calls_enabled(&state).await?;
    Ok(Json(json!({
        "id": CALLS_PLUGIN_ID,
        "name": CALLS_PLUGIN_NAME,
        "description": CALLS_PLUGIN_DESCRIPTION,
        "version": CALLS_PLUGIN_VERSION,
        "active": is_active
    })))
}

/// DELETE /api/v4/plugins/{plugin_id}
async fn remove_plugin(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(_plugin_id): Path<String>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.plugins.remove.not_implemented.app_error",
        "Plugin removal is not implemented.",
        "DELETE /api/v4/plugins/{plugin_id} is not supported in this server.",
    ))
}

/// POST /api/v4/plugins/{plugin_id}/enable
async fn enable_plugin(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(_plugin_id): Path<String>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.plugins.enable.not_implemented.app_error",
        "Plugin enable is not implemented.",
        "POST /api/v4/plugins/{plugin_id}/enable is not supported in this server.",
    ))
}

/// POST /api/v4/plugins/{plugin_id}/disable
async fn disable_plugin(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
    Path(_plugin_id): Path<String>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.plugins.disable.not_implemented.app_error",
        "Plugin disable is not implemented.",
        "POST /api/v4/plugins/{plugin_id}/disable is not supported in this server.",
    ))
}

/// GET /api/v4/plugins/statuses
async fn get_plugin_statuses(
    State(state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let is_active = calls_enabled(&state).await?;
    Ok(Json(vec![json!({
        "plugin_id": CALLS_PLUGIN_ID,
        "name": CALLS_PLUGIN_NAME,
        "version": CALLS_PLUGIN_VERSION,
        "is_active": is_active,
        "state": if is_active { 2 } else { 0 }
    })]))
}

/// GET /api/v4/plugins/webapp
/// Returns manifests for webapp plugins that should be loaded by clients
async fn get_webapp_plugins(
    State(state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let mut plugins = Vec::new();
    if calls_enabled(&state).await? {
        plugins.push(calls_plugin_webapp_manifest());
    }

    Ok(Json(plugins))
}

/// GET /api/v4/plugins/marketplace
async fn get_marketplace_plugins(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    ensure_manage_system(&auth)?;

    Ok(Json(vec![]))
}

/// POST /api/v4/plugins/marketplace
async fn install_marketplace_plugin(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    ensure_manage_system(&auth)?;

    Ok(crate::api::v4::mm_not_implemented(
        "api.plugins.marketplace.install.not_implemented.app_error",
        "Marketplace plugin installation is not implemented.",
        "POST /api/v4/plugins/marketplace is not supported in this server.",
    ))
}

/// GET /api/v4/plugins/marketplace/first_admin_visit
async fn get_first_admin_visit_marketplace(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;

    let db_value: Option<Option<String>> = sqlx::query_scalar(
        "SELECT plugins->'marketplace'->>'first_admin_visit' FROM server_config WHERE id = 'default'",
    )
    .fetch_optional(&state.db)
    .await?;

    let visited = db_value
        .flatten()
        .map(|raw| raw.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    Ok(Json(json!({
        "name": FIRST_ADMIN_VISIT_MARKETPLACE_KEY,
        "value": if visited { "true" } else { "false" }
    })))
}

/// POST /api/v4/plugins/marketplace/first_admin_visit
async fn first_admin_visit_marketplace(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    ensure_manage_system(&auth)?;

    sqlx::query(
        r#"
        UPDATE server_config
        SET
            plugins = jsonb_set(
                COALESCE(plugins, '{}'::jsonb),
                '{marketplace}',
                COALESCE(plugins->'marketplace', '{}'::jsonb) || '{"first_admin_visit": true}'::jsonb,
                true
            ),
            updated_at = NOW()
        WHERE id = 'default'
        "#,
    )
    .execute(&state.db)
    .await?;

    state
        .ws_hub
        .broadcast(crate::realtime::WsEnvelope {
            msg_type: "event".to_string(),
            event: FIRST_ADMIN_VISIT_MARKETPLACE_EVENT.to_string(),
            seq: None,
            channel_id: None,
            data: json!({
                "firstAdminVisitMarketplaceStatus": "true"
            }),
            broadcast: None,
        })
        .await;

    Ok((StatusCode::OK, Json(json!({ "status": "OK" }))))
}

fn calls_plugin_summary() -> serde_json::Value {
    json!({
        "id": CALLS_PLUGIN_ID,
        "name": CALLS_PLUGIN_NAME,
        "description": CALLS_PLUGIN_DESCRIPTION,
        "version": CALLS_PLUGIN_VERSION,
        "min_server_version": CALLS_PLUGIN_MIN_SERVER_VERSION
    })
}

fn calls_plugin_webapp_manifest() -> serde_json::Value {
    json!({
        "id": CALLS_PLUGIN_ID,
        "name": CALLS_PLUGIN_NAME,
        "description": CALLS_PLUGIN_DESCRIPTION,
        "version": CALLS_PLUGIN_VERSION,
        "min_server_version": CALLS_PLUGIN_MIN_SERVER_VERSION,
        "server": {},
        "webapp": {
            "bundle_path": "/static/plugins/com.mattermost.calls/webapp/main.js"
        }
    })
}

async fn calls_enabled(state: &AppState) -> ApiResult<bool> {
    let db_value: Option<String> = sqlx::query_scalar(
        "SELECT plugins->'calls'->>'enabled' FROM server_config WHERE id = 'default'",
    )
    .fetch_optional(&state.db)
    .await?;

    Ok(db_value
        .as_ref()
        .map(|v| v.parse::<bool>().unwrap_or(state.config.calls.enabled))
        .unwrap_or(state.config.calls.enabled))
}
