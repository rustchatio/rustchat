//! Terms of Service Enforcement Middleware
//!
//! Provides utilities for checking terms acceptance status.

use crate::api::AppState;

/// Check if a user needs to accept terms (for use in login/auth flows)
pub async fn user_needs_terms_acceptance(
    state: &AppState,
    user_id: uuid::Uuid,
) -> Result<Option<crate::models::terms::TermsOfService>, sqlx::Error> {
    // Get current active terms
    let terms = sqlx::query_as::<_, crate::models::terms::TermsOfService>(
        r#"
        SELECT * FROM terms_of_service 
        WHERE is_active = true 
        ORDER BY effective_date DESC 
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?;

    let Some(terms) = terms else {
        return Ok(None);
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
    .bind(user_id)
    .bind(terms.id)
    .fetch_one(&state.db)
    .await?;

    if accepted {
        Ok(None)
    } else {
        Ok(Some(terms))
    }
}
