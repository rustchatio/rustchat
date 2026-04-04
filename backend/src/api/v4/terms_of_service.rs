use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::models::terms::*;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use super::extractors::MmAuthUser;

pub fn router() -> Router<AppState> {
    Router::new()
        // Public endpoints
        .route("/terms_of_service/current", get(get_current_terms))
        .route("/terms_of_service/status", get(get_terms_status))
        .route("/terms_of_service/accept", post(accept_terms))
        // Admin endpoints
        .route("/terms_of_service", get(list_terms).post(create_terms))
        .route(
            "/terms_of_service/{id}",
            get(get_terms).put(update_terms).delete(delete_terms),
        )
        .route("/terms_of_service/{id}/activate", post(activate_terms))
        .route("/terms_of_service/{id}/stats", get(get_terms_stats))
        .route("/terms_of_service/stats/summary", get(get_all_terms_stats))
}

// Public endpoints

async fn get_current_terms(
    State(state): State<AppState>,
) -> ApiResult<Json<Option<TermsOfService>>> {
    let terms = sqlx::query_as::<_, TermsOfService>(
        r#"
        SELECT * FROM terms_of_service 
        WHERE is_active = true 
        ORDER BY effective_date DESC 
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    Ok(Json(terms))
}

async fn get_terms_status(
    State(state): State<AppState>,
    auth_user: MmAuthUser,
) -> ApiResult<Json<TermsStatusResponse>> {
    // Get current active terms
    let current_terms = sqlx::query_as::<_, TermsOfService>(
        r#"
        SELECT * FROM terms_of_service 
        WHERE is_active = true 
        ORDER BY effective_date DESC 
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(ref terms) = current_terms else {
        return Ok(Json(TermsStatusResponse {
            has_accepted: true,
            current_terms: None,
            accepted_version: None,
            acceptance_required: false,
        }));
    };

    // Check if user has accepted
    let accepted: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM user_terms_acceptance 
            WHERE user_id = $1 AND terms_id = $2
        )
        "#,
    )
    .bind(auth_user.user_id)
    .bind(terms.id)
    .fetch_one(&state.db)
    .await?;

    // Get accepted version if any
    let accepted_version = if accepted {
        Some(terms.version.clone())
    } else {
        None
    };

    Ok(Json(TermsStatusResponse {
        has_accepted: accepted,
        current_terms: current_terms.clone(),
        accepted_version,
        acceptance_required: !accepted,
    }))
}

async fn accept_terms(
    State(state): State<AppState>,
    auth_user: MmAuthUser,
    Json(req): Json<TermsAcceptanceRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Verify terms exist and are active
    let terms =
        sqlx::query_as::<_, TermsOfService>(r#"SELECT * FROM terms_of_service WHERE id = $1"#)
            .bind(req.terms_id)
            .fetch_optional(&state.db)
            .await?;

    let Some(_) = terms else {
        return Err(AppError::NotFound("Terms not found".to_string()));
    };

    // Insert acceptance
    sqlx::query(
        r#"
        INSERT INTO user_terms_acceptance (user_id, terms_id, accepted_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, terms_id) DO UPDATE SET accepted_at = $3
        "#,
    )
    .bind(auth_user.user_id)
    .bind(req.terms_id)
    .bind(Utc::now())
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Terms accepted successfully"
    })))
}

// Admin endpoints

async fn list_terms(
    State(state): State<AppState>,
    _auth_user: MmAuthUser, // TODO: Check admin permission
) -> ApiResult<Json<Vec<TermsOfService>>> {
    let terms = sqlx::query_as::<_, TermsOfService>(
        r#"
        SELECT * FROM terms_of_service 
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(terms))
}

async fn get_terms(
    State(state): State<AppState>,
    _auth_user: MmAuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TermsOfService>> {
    let terms =
        sqlx::query_as::<_, TermsOfService>(r#"SELECT * FROM terms_of_service WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&state.db)
            .await?;

    match terms {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound("Terms not found".to_string())),
    }
}

async fn create_terms(
    State(state): State<AppState>,
    auth_user: MmAuthUser,
    Json(req): Json<CreateTermsRequest>,
) -> ApiResult<Json<TermsOfService>> {
    // Validate version uniqueness
    let existing = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(SELECT 1 FROM terms_of_service WHERE version = $1)"#,
    )
    .bind(&req.version)
    .fetch_one(&state.db)
    .await?;

    if existing {
        return Err(AppError::Validation("Version already exists".to_string()));
    }

    let terms = sqlx::query_as::<_, TermsOfService>(
        r#"
        INSERT INTO terms_of_service 
        (version, title, content, summary, is_active, effective_date, created_by)
        VALUES ($1, $2, $3, $4, false, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&req.version)
    .bind(&req.title)
    .bind(&req.content)
    .bind(&req.summary)
    .bind(req.effective_date)
    .bind(auth_user.user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(terms))
}

async fn update_terms(
    State(state): State<AppState>,
    _auth_user: MmAuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTermsRequest>,
) -> ApiResult<Json<TermsOfService>> {
    let terms =
        sqlx::query_as::<_, TermsOfService>(r#"SELECT * FROM terms_of_service WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&state.db)
            .await?;

    if terms.is_none() {
        return Err(AppError::NotFound("Terms not found".to_string()));
    }

    let updated = sqlx::query_as::<_, TermsOfService>(
        r#"
        UPDATE terms_of_service 
        SET 
            title = COALESCE($1, title),
            content = COALESCE($2, content),
            summary = COALESCE($3, summary),
            effective_date = COALESCE($4, effective_date)
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(req.title)
    .bind(req.content)
    .bind(req.summary)
    .bind(req.effective_date)
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(updated))
}

async fn delete_terms(
    State(state): State<AppState>,
    _auth_user: MmAuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    // Check if terms is active
    let is_active =
        sqlx::query_scalar::<_, bool>(r#"SELECT is_active FROM terms_of_service WHERE id = $1"#)
            .bind(id)
            .fetch_optional(&state.db)
            .await?;

    if is_active == Some(true) {
        return Err(AppError::Validation(
            "Cannot delete active terms. Deactivate first.".to_string(),
        ));
    }

    sqlx::query(r#"DELETE FROM terms_of_service WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

async fn activate_terms(
    State(state): State<AppState>,
    _auth_user: MmAuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TermsOfService>> {
    let terms = sqlx::query_as::<_, TermsOfService>(
        r#"
        UPDATE terms_of_service 
        SET is_active = true 
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    match terms {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound("Terms not found".to_string())),
    }
}

async fn get_terms_stats(
    State(state): State<AppState>,
    _auth_user: MmAuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TermsStats>> {
    let row = sqlx::query(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM users WHERE deleted_at IS NULL) as total_users,
            COUNT(DISTINCT uta.user_id) as accepted_count
        FROM user_terms_acceptance uta
        WHERE uta.terms_id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    let total_users: i64 = row.try_get("total_users")?;
    let accepted_count: i64 = row.try_get("accepted_count")?;
    let pending_count = total_users - accepted_count;
    let acceptance_rate = if total_users > 0 {
        (accepted_count as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(TermsStats {
        total_users,
        accepted_count,
        pending_count,
        acceptance_rate,
    }))
}

async fn get_all_terms_stats(
    State(state): State<AppState>,
    _auth_user: MmAuthUser,
) -> ApiResult<Json<serde_json::Value>> {
    // Get current active terms
    let current_terms = sqlx::query_as::<_, TermsOfService>(
        r#"
        SELECT * FROM terms_of_service 
        WHERE is_active = true 
        ORDER BY effective_date DESC 
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(terms) = current_terms else {
        return Ok(Json(serde_json::json!({
            "has_active_terms": false,
            "total_users": 0,
            "accepted_count": 0,
            "pending_count": 0,
            "acceptance_rate": 0.0,
            "pending_users": []
        })));
    };

    // Get stats
    let row = sqlx::query(
        r#"
        SELECT 
            COUNT(*) as total_users
        FROM users 
        WHERE deleted_at IS NULL
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    let total_users: i64 = row.try_get("total_users")?;

    let accepted_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT user_id) 
        FROM user_terms_acceptance 
        WHERE terms_id = $1
        "#,
    )
    .bind(terms.id)
    .fetch_one(&state.db)
    .await?;

    // Get pending users list
    let pending_users = sqlx::query_as::<_, crate::models::user::User>(
        r#"
        SELECT u.* FROM users u
        WHERE u.deleted_at IS NULL
        AND NOT EXISTS (
            SELECT 1 FROM user_terms_acceptance uta 
            WHERE uta.user_id = u.id AND uta.terms_id = $1
        )
        ORDER BY u.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(terms.id)
    .fetch_all(&state.db)
    .await?;

    let pending_count = total_users - accepted_count;
    let acceptance_rate = if total_users > 0 {
        (accepted_count as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(serde_json::json!({
        "has_active_terms": true,
        "current_terms": terms,
        "total_users": total_users,
        "accepted_count": accepted_count,
        "pending_count": pending_count,
        "acceptance_rate": acceptance_rate,
        "pending_users": pending_users
    })))
}
