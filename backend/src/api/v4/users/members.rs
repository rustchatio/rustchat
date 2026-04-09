use axum::{extract::State, Json};
use uuid::Uuid;

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::ApiResult;
use crate::mattermost_compat::{id::encode_mm_id, models as mm};

pub async fn my_team_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::TeamMember>>> {
    let members: Vec<(Uuid, Uuid, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT tm.team_id, tm.user_id, tm.role, u.presence
        FROM team_members tm
        JOIN users u ON tm.user_id = u.id
        WHERE tm.user_id = $1
        "#,
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let mm_members = members
        .into_iter()
        .map(|(team_id, user_id, role, presence)| mm::TeamMember {
            team_id: encode_mm_id(team_id),
            user_id: encode_mm_id(user_id),
            roles: crate::mattermost_compat::mappers::map_team_role(&role),
            delete_at: 0,
            scheme_guest: false,
            scheme_user: true,
            scheme_admin: role == "admin" || role == "team_admin",
            presence: presence.filter(|value| !value.is_empty()),
        })
        .collect();

    Ok(Json(mm_members))
}

pub async fn get_team_members_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> ApiResult<Json<Vec<mm::TeamMember>>> {
    let user_id = super::user_sidebar_categories::resolve_user_id(&user_id, &auth)?;
    let members: Vec<(Uuid, Uuid, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT tm.team_id, tm.user_id, tm.role, u.presence
        FROM team_members tm
        JOIN users u ON tm.user_id = u.id
        WHERE tm.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let mm_members = members
        .into_iter()
        .map(|(team_id, user_id, role, presence)| mm::TeamMember {
            team_id: encode_mm_id(team_id),
            user_id: encode_mm_id(user_id),
            roles: crate::mattermost_compat::mappers::map_team_role(&role),
            delete_at: 0,
            scheme_guest: false,
            scheme_user: true,
            scheme_admin: role == "admin" || role == "team_admin",
            presence: presence.filter(|value| !value.is_empty()),
        })
        .collect();

    Ok(Json(mm_members))
}
