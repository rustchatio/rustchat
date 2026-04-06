use axum::{extract::State, Json};

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::ApiResult;

pub async fn get_user_stats(
    State(state): State<AppState>,
    _auth: MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"total_users_count": total})))
}

pub async fn get_user_stats_filtered(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    get_user_stats(State(state), auth).await
}

pub async fn get_invalid_emails() -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![]))
}
