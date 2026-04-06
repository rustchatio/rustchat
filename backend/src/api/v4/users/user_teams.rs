use axum::{extract::State, Json};
use uuid::Uuid;

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::ApiResult;
use crate::mattermost_compat::{id::encode_mm_id, models as mm};
use crate::models::Team;

pub async fn my_teams(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Team>>> {
    let teams = fetch_user_teams(&state, auth.user_id).await?;

    if teams.is_empty() {
        return Ok(Json(vec![default_team()]));
    }

    let mm_teams: Vec<mm::Team> = teams.into_iter().map(|t| t.into()).collect();
    Ok(Json(mm_teams))
}

pub async fn get_teams_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<Vec<mm::Team>>> {
    let user_id = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    let teams = fetch_user_teams(&state, user_id).await?;

    if teams.is_empty() {
        return Ok(Json(vec![default_team()]));
    }

    let mm_teams: Vec<mm::Team> = teams.into_iter().map(|t| t.into()).collect();
    Ok(Json(mm_teams))
}

pub async fn fetch_user_teams(state: &AppState, user_id: Uuid) -> ApiResult<Vec<Team>> {
    sqlx::query_as(
        r#"
        SELECT t.* FROM teams t
        JOIN team_members tm ON t.id = tm.team_id
        WHERE tm.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .map_err(Into::into)
}

pub fn default_team() -> mm::Team {
    let id = Uuid::new_v4();
    mm::Team {
        id: encode_mm_id(id),
        create_at: 0,
        update_at: 0,
        delete_at: 0,
        display_name: "RustChat".to_string(),
        name: "rustchat".to_string(),
        description: "".to_string(),
        email: "".to_string(),
        team_type: "O".to_string(),
        company_name: "".to_string(),
        allowed_domains: "".to_string(),
        invite_id: "".to_string(),
        allow_open_invite: false,
    }
}
