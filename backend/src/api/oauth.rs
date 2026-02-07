//! OAuth2/OIDC authentication handlers

use axum::{
    extract::{Path, Query, State},
    response::Redirect,
    routing::get,
    Json, Router,
};
use deadpool_redis::redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::error::{ApiResult, AppError};

const OAUTH_STATE_PREFIX: &str = "rustchat:oauth:state:";
const OAUTH_STATE_TTL_SECONDS: u64 = 300;
const DEFAULT_OAUTH_REDIRECT_PATH: &str = "/oauth/callback";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/{provider}/login", get(oauth_login))
        .route("/oauth2/{provider}/callback", get(oauth_callback))
        .route("/oauth2/providers", get(list_providers))
}

/// List available OAuth providers
async fn list_providers(State(state): State<AppState>) -> ApiResult<Json<Vec<OAuthProvider>>> {
    // Query enabled SSO configs from DB
    let configs: Vec<SsoConfigRow> =
        sqlx::query_as("SELECT * FROM sso_configs WHERE is_active = true")
            .fetch_all(&state.db)
            .await?;

    let providers: Vec<OAuthProvider> = configs
        .into_iter()
        .map(|c| OAuthProvider {
            id: c.provider.clone(),
            name: c.display_name.unwrap_or(c.provider),
            icon_url: None,
        })
        .collect();

    Ok(Json(providers))
}

#[derive(Debug, Serialize)]
pub struct OAuthProvider {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct SsoConfigRow {
    id: Uuid,
    org_id: Uuid,
    provider: String,
    display_name: Option<String>,
    issuer_url: Option<String>,
    client_id: Option<String>,
    client_secret_encrypted: Option<String>,
    is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct OAuthLoginQuery {
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthStatePayload {
    provider: String,
    redirect_after: String,
}

fn oauth_state_key(state: &str) -> String {
    format!("{}{}", OAUTH_STATE_PREFIX, state)
}

fn sanitize_redirect_path(redirect_uri: Option<String>) -> String {
    match redirect_uri {
        Some(path) if path.starts_with('/') && !path.starts_with("//") => path,
        _ => DEFAULT_OAUTH_REDIRECT_PATH.to_string(),
    }
}

fn append_token_query(path: &str, token: &str) -> String {
    let separator = if path.contains('?') { '&' } else { '?' };
    format!("{}{}token={}", path, separator, urlencoding::encode(token))
}

/// Initiate OAuth login - redirects to provider
async fn oauth_login(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthLoginQuery>,
) -> Result<Redirect, AppError> {
    // Get SSO config for provider
    let config: SsoConfigRow =
        sqlx::query_as("SELECT * FROM sso_configs WHERE provider = $1 AND is_active = true")
            .bind(&provider)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "OAuth provider '{}' not found or disabled",
                    provider
                ))
            })?;

    let issuer = config.issuer_url.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider issuer_url is not configured".to_string())
    })?;
    let client_id = config.client_id.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider client_id is not configured".to_string())
    })?;

    // Generate and persist state parameter for CSRF protection
    let oauth_state = Uuid::new_v4().to_string();
    let oauth_state_payload = OAuthStatePayload {
        provider: provider.clone(),
        redirect_after: sanitize_redirect_path(query.redirect_uri),
    };
    let serialized_state = serde_json::to_string(&oauth_state_payload)
        .map_err(|e| AppError::Internal(format!("Failed to serialize OAuth state: {}", e)))?;

    let mut redis_conn =
        state.redis.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to acquire Redis connection: {}", e))
        })?;
    let _: () = redis_conn
        .set_ex(
            oauth_state_key(&oauth_state),
            serialized_state,
            OAUTH_STATE_TTL_SECONDS,
        )
        .await?;

    let callback_url = format!(
        "{}/api/v1/oauth2/{}/callback",
        std::env::var("RUSTCHAT_SITE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        provider
    );

    let auth_url = format!(
        "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope=openid%20profile%20email&state={}",
        issuer,
        urlencoding::encode(&client_id),
        urlencoding::encode(&callback_url),
        oauth_state
    );

    Ok(Redirect::temporary(&auth_url))
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    #[serde(alias = "_state")]
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    id_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfoResponse {
    sub: String,
    email: Option<String>,
    name: Option<String>,
    preferred_username: Option<String>,
    picture: Option<String>,
}

/// Handle OAuth callback from provider
async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    // Check for errors from provider
    if let Some(error) = query.error {
        let desc = query.error_description.unwrap_or_else(|| error.clone());
        return Ok(Redirect::temporary(&format!(
            "/login?error={}",
            urlencoding::encode(&desc)
        )));
    }

    let code = query
        .code
        .ok_or_else(|| AppError::BadRequest("Missing authorization code".to_string()))?;
    let oauth_state = query
        .state
        .ok_or_else(|| AppError::BadRequest("Missing OAuth state parameter".to_string()))?;

    // Validate and consume OAuth state to prevent CSRF/replay.
    let mut redis_conn =
        state.redis.get().await.map_err(|e| {
            AppError::Internal(format!("Failed to acquire Redis connection: {}", e))
        })?;
    let state_key = oauth_state_key(&oauth_state);
    let stored_state_json: Option<String> = redis_conn.get(&state_key).await?;
    let _: usize = redis_conn.del(&state_key).await?;
    let stored_state_json = stored_state_json
        .ok_or_else(|| AppError::BadRequest("Invalid or expired OAuth state".to_string()))?;
    let stored_state: OAuthStatePayload = serde_json::from_str(&stored_state_json)
        .map_err(|e| AppError::Internal(format!("Invalid OAuth state payload: {}", e)))?;
    if stored_state.provider != provider {
        return Err(AppError::BadRequest(
            "OAuth state provider mismatch".to_string(),
        ));
    }

    // Get SSO config
    let config: SsoConfigRow =
        sqlx::query_as("SELECT * FROM sso_configs WHERE provider = $1 AND is_active = true")
            .bind(&provider)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("OAuth provider '{}' not found", provider))
            })?;

    let issuer = config.issuer_url.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider issuer_url is not configured".to_string())
    })?;
    let client_id = config.client_id.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider client_id is not configured".to_string())
    })?;
    let secret_raw = config.client_secret_encrypted.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider client_secret is not configured".to_string())
    })?;
    let client_secret = match crate::crypto::decrypt(&secret_raw, &state.config.encryption_key) {
        Ok(secret) => secret,
        Err(_) if state.config.is_production() => {
            return Err(AppError::Internal(
                "Failed to decrypt OAuth client secret in production mode".to_string(),
            ));
        }
        Err(_) => {
            tracing::warn!(
                provider = %provider,
                "Using non-encrypted OAuth client secret fallback (development mode)"
            );
            secret_raw
        }
    };

    let callback_url = format!(
        "{}/api/v1/oauth2/{}/callback",
        std::env::var("RUSTCHAT_SITE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
        provider
    );

    // Exchange code for tokens
    let client = reqwest::Client::new();
    let token_url = format!("{}/token", issuer);

    let token_response = client
        .post(&token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &callback_url),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Token exchange failed: {}", e)))?;

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "Token exchange failed: {}",
            error_text
        )));
    }

    let tokens: TokenResponse = token_response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse token response: {}", e)))?;

    // Get user info
    let userinfo_url = format!("{}/userinfo", issuer);
    let userinfo_response: reqwest::Response = client
        .get(&userinfo_url)
        .bearer_auth(&tokens.access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Userinfo request failed: {}", e)))?;

    let userinfo: UserInfoResponse = userinfo_response
        .json::<UserInfoResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse userinfo: {}", e)))?;

    // Find or create user
    let email = userinfo
        .email
        .ok_or_else(|| AppError::BadRequest("Email not provided by OAuth provider".to_string()))?;

    let user: Option<crate::models::User> = sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.db)
        .await?;

    let user = match user {
        Some(u) => u,
        None => {
            // Create new user from OAuth info
            let username = userinfo
                .preferred_username
                .or(userinfo.name.clone())
                .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());

            sqlx::query_as(
                r#"
                INSERT INTO users (username, email, display_name, role, is_active, auth_provider)
                VALUES ($1, $2, $3, 'member', true, $4)
                ON CONFLICT (email) DO UPDATE SET last_login_at = NOW()
                RETURNING *
                "#,
            )
            .bind(&username)
            .bind(&email)
            .bind(&userinfo.name)
            .bind(&provider)
            .fetch_one(&state.db)
            .await?
        }
    };

    // Update last login
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(&state.db)
        .await?;

    // Generate JWT token
    let token = crate::auth::create_token(
        user.id,
        &user.email,
        &user.role,
        user.org_id,
        &state.jwt_secret,
        state.jwt_expiry_hours,
    )?;

    // Redirect to frontend with token
    Ok(Redirect::temporary(&append_token_query(
        &stored_state.redirect_after,
        &token,
    )))
}
