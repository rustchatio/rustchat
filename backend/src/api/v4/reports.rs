use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::ApiResult;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reports/users", get(get_user_reports))
        .route("/reports/users/count", get(get_user_reports_count))
        .route(
            "/reports/users/export",
            post(export_user_reports).get(export_user_reports_legacy),
        )
        .route(
            "/reports/posts",
            post(get_post_reports).get(get_post_reports_legacy),
        )
        .route("/audit_logs/certificate", get(get_audit_cert))
}

#[derive(Debug, Deserialize, Default)]
struct UserReportsQuery {
    hide_active: Option<bool>,
    hide_inactive: Option<bool>,
    page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PostsReportRequest {
    channel_id: Option<String>,
}

fn ensure_manage_system(auth: &crate::api::v4::extractors::MmAuthUser) -> ApiResult<()> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(crate::error::AppError::Forbidden(
            "Insufficient permissions to access reports".to_string(),
        ));
    }

    Ok(())
}

fn validate_user_reports_query(query: &UserReportsQuery) -> ApiResult<()> {
    if query.hide_active.unwrap_or(false) && query.hide_inactive.unwrap_or(false) {
        return Err(crate::error::AppError::BadRequest(
            "hide_active and hide_inactive cannot both be true".to_string(),
        ));
    }

    if let Some(page_size) = query.page_size {
        if page_size <= 0 || page_size > 100 {
            return Err(crate::error::AppError::BadRequest(
                "page_size must be between 1 and 100".to_string(),
            ));
        }
    }

    Ok(())
}

async fn get_user_reports(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Query(query): Query<UserReportsQuery>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    ensure_manage_system(&auth)?;
    validate_user_reports_query(&query)?;

    Ok(Json(vec![]))
}

async fn get_user_reports_count(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Query(query): Query<UserReportsQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;
    validate_user_reports_query(&query)?;

    Ok(Json(json!(0)))
}

/// POST /api/v4/reports/users/export
async fn export_user_reports(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;

    Ok(Json(json!({ "status": "OK" })))
}

/// Temporary compatibility shim for legacy RustChat GET behavior.
async fn export_user_reports_legacy(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;
    Ok(Json(json!({ "status": "OK" })))
}

/// POST /api/v4/reports/posts
async fn get_post_reports(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Json(payload): Json<PostsReportRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;

    let channel_id = payload.channel_id.unwrap_or_default();
    if channel_id.trim().is_empty() {
        return Err(crate::error::AppError::BadRequest(
            "channel_id is required".to_string(),
        ));
    }

    Ok(Json(json!({
        "posts": {},
        "next_cursor": serde_json::Value::Null
    })))
}

/// Temporary compatibility shim for legacy RustChat GET behavior.
async fn get_post_reports_legacy(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;
    Ok(Json(json!({
        "posts": {},
        "next_cursor": serde_json::Value::Null
    })))
}

async fn get_audit_cert(
    State(_state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    ensure_manage_system(&auth)?;
    Ok(Json(json!({})))
}
