use axum::{body::Bytes, http::HeaderMap, Json};

use super::MmAuthUser;
use crate::error::ApiResult;

pub async fn get_user_tokens(
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _ = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    Ok(Json(vec![]))
}

pub async fn get_tokens() -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

pub async fn revoke_token(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = super::parse_body(&headers, &body, "Invalid revoke body")?;
    Ok(super::status_ok())
}

pub async fn get_token(
    axum::extract::Path(_token_id): axum::extract::Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({})))
}

pub async fn disable_token(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = super::parse_body(&headers, &body, "Invalid disable body")?;
    Ok(super::status_ok())
}

pub async fn enable_token(headers: HeaderMap, body: Bytes) -> ApiResult<Json<serde_json::Value>> {
    let _value: serde_json::Value = super::parse_body(&headers, &body, "Invalid enable body")?;
    Ok(super::status_ok())
}

pub async fn search_tokens(
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let _value: serde_json::Value = super::parse_body(&headers, &body, "Invalid search body")?;
    Ok(Json(vec![]))
}
