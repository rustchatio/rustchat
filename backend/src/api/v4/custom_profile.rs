//! Custom Profile Attributes API endpoints for Mattermost mobile compatibility

use axum::{
    extract::{Path, State},
    routing::{get, patch},
    Json, Router,
};
use std::collections::HashMap;

use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use crate::models::{
    CustomProfileAttribute, CustomProfileField, CustomProfileFieldResponse,
    UserCustomProfileAttributeSimple,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/custom_profile_attributes/fields",
            get(get_custom_profile_fields),
        )
        .route(
            "/custom_profile_attributes/values",
            patch(update_custom_profile_values),
        )
        .route(
            "/users/{user_id}/custom_profile_attributes",
            get(get_user_custom_profile_attributes),
        )
}

/// GET /custom_profile_attributes/fields - Get all custom profile field definitions
async fn get_custom_profile_fields(
    State(state): State<AppState>,
    _auth: MmAuthUser,
) -> ApiResult<Json<Vec<CustomProfileFieldResponse>>> {
    let fields: Vec<CustomProfileField> = sqlx::query_as(
        r#"
        SELECT id, group_id, name, field_type, attrs, target_id, target_type,
               created_at, updated_at, deleted_at
        FROM custom_profile_fields
        WHERE deleted_at IS NULL
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let response: Vec<CustomProfileFieldResponse> = fields.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

/// GET /users/{user_id}/custom_profile_attributes - Get a user's custom profile attributes
async fn get_user_custom_profile_attributes(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(user_id): Path<String>,
) -> ApiResult<Json<UserCustomProfileAttributeSimple>> {
    let user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user_id".to_string()))?;

    let attrs: Vec<CustomProfileAttribute> = sqlx::query_as(
        r#"
        SELECT id, field_id, user_id, value
        FROM custom_profile_attributes
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let mut result: HashMap<String, serde_json::Value> = HashMap::new();
    for attr in attrs {
        result.insert(
            encode_mm_id(attr.field_id),
            serde_json::Value::String(attr.value),
        );
    }

    Ok(Json(result))
}

/// PATCH /custom_profile_attributes/values - Update custom profile attribute values
async fn update_custom_profile_values(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(values): Json<UserCustomProfileAttributeSimple>,
) -> ApiResult<Json<serde_json::Value>> {
    for (field_id_str, value) in values {
        let field_id = parse_mm_or_uuid(&field_id_str)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid field_id: {}", field_id_str)))?;

        let value_str = match value {
            serde_json::Value::String(s) => s,
            serde_json::Value::Array(arr) => {
                // Convert array to comma-separated string
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            }
            other => other.to_string(),
        };

        sqlx::query(
            r#"
            INSERT INTO custom_profile_attributes (field_id, user_id, value)
            VALUES ($1, $2, $3)
            ON CONFLICT (field_id, user_id)
            DO UPDATE SET value = $3
            "#,
        )
        .bind(field_id)
        .bind(auth.user_id)
        .bind(&value_str)
        .execute(&state.db)
        .await?;
    }

    Ok(Json(serde_json::json!({"status": "OK"})))
}
