use axum::{extract::Path, extract::State, Json};

use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::ApiResult;

pub async fn autocomplete_team_commands(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_team_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}
