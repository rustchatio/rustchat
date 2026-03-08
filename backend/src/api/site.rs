//! Public site configuration and metadata
use super::AppState;
use crate::error::ApiResult;
use crate::models::server_config::SiteConfig;
use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

#[derive(Serialize)]
pub struct PublicConfig {
    pub site_name: String,
    pub logo_url: Option<String>,
    pub enable_sso: bool,
    pub require_sso: bool,
    pub post_edit_time_limit_seconds: i32,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/site/info", get(get_site_info))
}

async fn get_site_info(State(state): State<AppState>) -> ApiResult<Json<PublicConfig>> {
    let config: (sqlx::types::Json<SiteConfig>,) =
        sqlx::query_as("SELECT site FROM server_config WHERE id = 'default'")
            .fetch_one(&state.db)
            .await?;

    // Fetch authentication settings
    let auth: (sqlx::types::Json<serde_json::Value>,) =
        sqlx::query_as("SELECT authentication FROM server_config WHERE id = 'default'")
            .fetch_one(&state.db)
            .await?;

    let enable_sso = auth
        .0
        .get("enable_sso")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let require_sso = auth
        .0
        .get("require_sso")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(Json(PublicConfig {
        site_name: config.0.site_name.clone(),
        logo_url: config.0.logo_url.clone(),
        enable_sso,
        require_sso,
        post_edit_time_limit_seconds: config.0.post_edit_time_limit_seconds,
    }))
}
