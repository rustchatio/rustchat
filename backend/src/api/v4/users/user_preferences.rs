use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use super::{parse_body, resolve_user_id, MmAuthUser};
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{id::encode_mm_id, models as mm};

const MAX_UPDATE_PREFERENCES: usize = 100;

pub async fn get_preferences(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<mm::Preference>>> {
    fetch_preferences(&state, auth.user_id).await
}

pub async fn get_preferences_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Vec<mm::Preference>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    fetch_preferences(&state, user_id).await
}

pub async fn get_my_preferences_by_category(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(category): Path<String>,
) -> ApiResult<Json<Vec<mm::Preference>>> {
    let rows = sqlx::query(
        "SELECT user_id, category, name, value FROM mattermost_preferences WHERE user_id = $1 AND category = $2",
    )
    .bind(auth.user_id)
    .bind(&category)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(map_preference_rows(rows)))
}

pub async fn get_preferences_by_category(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((user_id, category)): Path<(String, String)>,
) -> ApiResult<Json<Vec<mm::Preference>>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let rows = sqlx::query(
        "SELECT user_id, category, name, value FROM mattermost_preferences WHERE user_id = $1 AND category = $2",
    )
    .bind(user_id)
    .bind(&category)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(map_preference_rows(rows)))
}

pub async fn get_preference_by_category_and_name(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((user_id, category, preference_name)): Path<(String, String, String)>,
) -> ApiResult<Json<mm::Preference>> {
    let user_id = resolve_user_id(&user_id, &auth)?;
    let row = sqlx::query(
        "SELECT user_id, category, name, value FROM mattermost_preferences WHERE user_id = $1 AND category = $2 AND name = $3",
    )
    .bind(user_id)
    .bind(&category)
    .bind(&preference_name)
    .fetch_optional(&state.db)
    .await?;

    let row = row.ok_or_else(|| AppError::NotFound("Preference not found".to_string()))?;
    Ok(Json(map_preference_row(row)))
}

pub async fn update_preferences(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let prefs: Vec<mm::Preference> = parse_body(&headers, &body, "Invalid preferences body")?;
    validate_preferences_len(&prefs)?;
    update_preferences_internal(&state, auth.user_id, prefs).await
}

pub async fn update_preferences_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let prefs: Vec<mm::Preference> = parse_body(&headers, &body, "Invalid preferences body")?;
    validate_preferences_len(&prefs)?;
    let user_id = resolve_user_id(&user_id, &auth)?;
    update_preferences_internal(&state, user_id, prefs).await
}

pub async fn delete_preferences_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let prefs: Vec<mm::Preference> = parse_body(&headers, &body, "Invalid preferences body")?;
    validate_preferences_len(&prefs)?;
    let user_id = resolve_user_id(&user_id, &auth)?;

    let mut tx = state.db.begin().await?;
    for pref in prefs {
        sqlx::query(
            "DELETE FROM mattermost_preferences WHERE user_id = $1 AND category = $2 AND name = $3",
        )
        .bind(user_id)
        .bind(pref.category)
        .bind(pref.name)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn fetch_preferences(
    state: &AppState,
    user_id: Uuid,
) -> ApiResult<Json<Vec<mm::Preference>>> {
    let rows = sqlx::query(
        "SELECT user_id, category, name, value FROM mattermost_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(map_preference_rows(rows)))
}

fn map_preference_rows(rows: Vec<sqlx::postgres::PgRow>) -> Vec<mm::Preference> {
    rows.into_iter().map(map_preference_row).collect()
}

fn map_preference_row(row: sqlx::postgres::PgRow) -> mm::Preference {
    use sqlx::Row;
    let uid: Uuid = row.try_get("user_id").unwrap_or_default();
    mm::Preference {
        user_id: encode_mm_id(uid),
        category: row.try_get("category").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        value: row.try_get("value").unwrap_or_default(),
    }
}

fn validate_preferences_len(prefs: &[mm::Preference]) -> ApiResult<()> {
    if prefs.is_empty() || prefs.len() > MAX_UPDATE_PREFERENCES {
        return Err(AppError::BadRequest("Invalid preferences".to_string()));
    }
    Ok(())
}

async fn update_preferences_internal(
    state: &AppState,
    user_id: Uuid,
    prefs: Vec<mm::Preference>,
) -> ApiResult<impl IntoResponse> {
    let mut tx = state.db.begin().await?;

    for p in prefs {
        sqlx::query(
            r#"
            INSERT INTO mattermost_preferences (user_id, category, name, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, category, name)
            DO UPDATE SET value = $4
            "#,
        )
        .bind(user_id)
        .bind(p.category)
        .bind(p.name)
        .bind(p.value)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pref() -> mm::Preference {
        mm::Preference {
            user_id: "u".to_string(),
            category: "cat".to_string(),
            name: "name".to_string(),
            value: "value".to_string(),
        }
    }

    #[test]
    fn rejects_empty_preferences() {
        let result = validate_preferences_len(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_oversized_preferences() {
        let prefs = vec![pref(); MAX_UPDATE_PREFERENCES + 1];
        let result = validate_preferences_len(&prefs);
        assert!(result.is_err());
    }

    #[test]
    fn accepts_reasonable_preferences() {
        let prefs = vec![pref()];
        let result = validate_preferences_len(&prefs);
        assert!(result.is_ok());
    }
}
