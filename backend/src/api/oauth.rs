//! OAuth2/OIDC authentication handlers
//!
//! Supports three provider types:
//! - github: OAuth2 with GitHub (no OIDC discovery)
//! - google: OIDC with discovery
//! - oidc: Generic OIDC with discovery (Keycloak, ZITADEL, Authentik, etc.)

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
use crate::crypto;
use crate::error::{ApiResult, AppError};
use crate::models::{OAuthProviderInfo, SsoConfig, SsoProviderType};
use crate::services::oidc_discovery::{find_signing_key, OidcDiscoveryService};

const OAUTH_STATE_PREFIX: &str = "rustchat:oauth:state:";
const OAUTH_STATE_TTL_SECONDS: u64 = 300; // 5 minutes
const DEFAULT_OAUTH_REDIRECT_PATH: &str = "/";

// GitHub OAuth endpoints (no OIDC discovery)
const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_API_URL: &str = "https://api.github.com";

// Google OAuth endpoints (as fallback if discovery fails)
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

/// State parameter stored in Redis
#[derive(Debug, Serialize, Deserialize)]
struct OAuthStatePayload {
    provider_key: String,
    redirect_after: String,
    created_at: i64,
    // OIDC-specific fields
    nonce: Option<String>,
    // PKCE
    code_verifier: Option<String>,
    code_challenge_method: Option<String>,
}

/// OAuth callback query parameters
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// OAuth login query parameters
#[derive(Debug, Deserialize)]
pub struct OAuthLoginQuery {
    pub redirect_uri: Option<String>,
}

/// Token response from OAuth provider
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<i64>,
    refresh_token: Option<String>,
    id_token: Option<String>,
    scope: Option<String>,
}

/// User info from OIDC userinfo endpoint
#[derive(Debug, Deserialize)]
struct UserInfoResponse {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    preferred_username: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Claims from ID token
#[derive(Debug, Deserialize)]
struct IdTokenClaims {
    iss: String,
    sub: String,
    aud: String,
    exp: i64,
    iat: i64,
    nonce: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    preferred_username: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    groups: Option<Vec<String>>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// GitHub user info
#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

/// GitHub email info
#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/{provider_key}/login", get(oauth_login))
        .route("/oauth2/{provider_key}/callback", get(oauth_callback))
        .route("/oauth2/providers", get(list_providers))
}

/// Generate Redis key for OAuth state
fn oauth_state_key(state: &str) -> String {
    format!("{}{}", OAUTH_STATE_PREFIX, state)
}

/// Sanitize redirect path - only allow relative paths starting with /
fn sanitize_redirect_path(redirect_uri: Option<String>) -> String {
    match redirect_uri {
        Some(path) => {
            // Must start with / and not be // or contain ..
            if path.starts_with('/')
                && !path.starts_with("//")
                && !path.contains("..")
                && !path.contains('\0')
            {
                path
            } else {
                DEFAULT_OAUTH_REDIRECT_PATH.to_string()
            }
        }
        _ => DEFAULT_OAUTH_REDIRECT_PATH.to_string(),
    }
}

/// Append token to redirect URL
fn append_token_query(path: &str, token: &str) -> String {
    let encoded_token = urlencoding::encode(token);
    if path.contains('?') {
        format!("{}&token={}", path, encoded_token)
    } else {
        format!("{}?token={}", path, encoded_token)
    }
}

/// Generate PKCE code verifier (43-128 chars per RFC 7636)
fn generate_code_verifier() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..128)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

/// Generate PKCE code challenge from verifier (S256 method)
fn generate_code_challenge(verifier: &str) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use sha2::{Digest, Sha256};

    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate nonce for OIDC
fn generate_nonce() -> String {
    Uuid::new_v4().to_string()
}

/// Get site URL from database (server config) with fallback to environment variable
async fn get_site_url(db: &sqlx::PgPool) -> String {
    // Try to get from database first
    let result: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT site FROM server_config WHERE id = 'default'")
            .fetch_optional(db)
            .await
            .ok()
            .flatten();

    let db_url = result
        .and_then(|(site,)| {
            site.get("site_url")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
        .filter(|s| !s.is_empty());

    // Fall back to environment variable if not set in database
    db_url.unwrap_or_else(|| {
        std::env::var("RUSTCHAT_SITE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string())
    })
}

/// List available OAuth providers for login
async fn list_providers(State(state): State<AppState>) -> ApiResult<Json<Vec<OAuthProviderInfo>>> {
    // Check if SSO is enabled globally
    let config_row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT authentication FROM server_config WHERE id = 'default'")
            .fetch_optional(&state.db)
            .await?;

    let sso_enabled = config_row
        .and_then(|(json,)| json.get("enable_sso").and_then(|v| v.as_bool()))
        .unwrap_or(false);

    if !sso_enabled {
        return Ok(Json(vec![]));
    }

    // Query active SSO configs
    let configs: Vec<SsoConfig> = sqlx::query_as(
        r#"
        SELECT 
            id, org_id, provider, provider_key, provider_type, display_name,
            issuer_url, client_id, client_secret_encrypted, scopes,
            idp_metadata_url, idp_entity_id, is_active, auto_provision,
            default_role, allow_domains, github_org, github_team,
            groups_claim, role_mappings, created_at, updated_at
        FROM sso_configs 
        WHERE is_active = true
        ORDER BY display_name, provider_key
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let site_url = get_site_url(&state.db).await;
    let providers: Vec<OAuthProviderInfo> = configs
        .into_iter()
        .map(|c| {
            let display_name = c
                .display_name
                .clone()
                .unwrap_or_else(|| match c.provider_type.as_str() {
                    "github" => "GitHub".to_string(),
                    "google" => "Google".to_string(),
                    "oidc" => "SSO".to_string(),
                    _ => c.provider_key.clone(),
                });

            OAuthProviderInfo {
                id: c.id.to_string(),
                provider_key: c.provider_key.clone(),
                provider_type: c.provider_type.clone(),
                display_name,
                login_url: format!("{}/api/v1/oauth2/{}/login", site_url, c.provider_key),
            }
        })
        .collect();

    Ok(Json(providers))
}

/// Initiate OAuth login - redirects to provider
async fn oauth_login(
    State(state): State<AppState>,
    Path(provider_key): Path<String>,
    Query(query): Query<OAuthLoginQuery>,
) -> Result<Redirect, AppError> {
    // Load provider config
    let config: SsoConfig = sqlx::query_as(
        r#"
        SELECT 
            id, org_id, provider, provider_key, provider_type, display_name,
            issuer_url, client_id, client_secret_encrypted, scopes,
            idp_metadata_url, idp_entity_id, is_active, auto_provision,
            default_role, allow_domains, github_org, github_team,
            groups_claim, role_mappings, created_at, updated_at
        FROM sso_configs 
        WHERE provider_key = $1 AND is_active = true
        "#,
    )
    .bind(&provider_key)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(format!(
            "OAuth provider '{}' not found or disabled",
            provider_key
        ))
    })?;

    let client_id = config.client_id.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider client_id not configured".to_string())
    })?;

    let provider_type =
        SsoProviderType::from_str(&config.provider_type).ok_or_else(|| {
            AppError::Internal(format!("Unknown provider type: {}", config.provider_type))
        })?;

    // Generate state parameter
    let oauth_state = Uuid::new_v4().to_string();
    let redirect_after = sanitize_redirect_path(query.redirect_uri);

    // Generate PKCE and nonce for OIDC providers
    let (code_verifier, code_challenge, nonce) = match provider_type {
        SsoProviderType::GitHub => (None, None, None),
        _ => {
            let verifier = generate_code_verifier();
            let challenge = generate_code_challenge(&verifier);
            (Some(verifier), Some(challenge), Some(generate_nonce()))
        }
    };

    let state_payload = OAuthStatePayload {
        provider_key: provider_key.clone(),
        redirect_after,
        created_at: chrono::Utc::now().timestamp(),
        nonce: nonce.clone(),
        code_verifier: code_verifier.clone(),
        code_challenge_method: code_challenge.as_ref().map(|_| "S256".to_string()),
    };

    // Store state in Redis
    let serialized_state = serde_json::to_string(&state_payload)
        .map_err(|e| AppError::Internal(format!("Failed to serialize OAuth state: {}", e)))?;

    let mut redis_conn = state
        .redis
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))?;

    let _: () = redis_conn
        .set_ex(
            oauth_state_key(&oauth_state),
            serialized_state,
            OAUTH_STATE_TTL_SECONDS,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to store OAuth state: {}", e)))?;

    let callback_url = format!("{}/api/v1/oauth2/{}/callback", get_site_url(&state.db).await, provider_key);
    let scopes = if config.scopes.is_empty() {
        provider_type.default_scopes()
    } else {
        config.scopes.clone()
    };
    let scope_str = scopes.join(" ");

    // Build authorization URL based on provider type
    let auth_url = match provider_type {
        SsoProviderType::GitHub => {
            format!(
                "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                GITHUB_AUTH_URL,
                urlencoding::encode(&client_id),
                urlencoding::encode(&callback_url),
                urlencoding::encode(&scope_str),
                oauth_state
            )
        }
        SsoProviderType::Google | SsoProviderType::Oidc => {
            let issuer = config.issuer_url.clone().ok_or_else(|| {
                AppError::BadRequest("OIDC provider issuer_url not configured".to_string())
            })?;

            // Use OIDC discovery to get authorization endpoint
            let discovery = OidcDiscoveryService::new();
            let discovery_result = discovery.discover(&issuer).await.map_err(|e| {
                AppError::Internal(format!("OIDC discovery failed for '{}': {}", issuer, e))
            })?;

            let mut url = format!(
                "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
                discovery_result.authorization_endpoint,
                urlencoding::encode(&client_id),
                urlencoding::encode(&callback_url),
                urlencoding::encode(&scope_str),
                oauth_state
            );

            // Add PKCE code challenge
            if let Some(challenge) = code_challenge {
                url.push_str(&format!(
                    "&code_challenge={}&code_challenge_method=S256",
                    urlencoding::encode(&challenge)
                ));
            }

            // Add nonce for ID token validation
            if let Some(n) = nonce {
                url.push_str(&format!("&nonce={}", urlencoding::encode(&n)));
            }

            url
        }
        SsoProviderType::Saml => {
            return Err(AppError::BadRequest(
                "SAML is not supported via OAuth endpoints".to_string(),
            ));
        }
    };

    Ok(Redirect::temporary(&auth_url))
}

/// Handle OAuth callback from provider
async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider_key): Path<String>,
    Query(query): Query<OAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    // Handle provider error
    if let Some(error) = query.error {
        let desc = query
            .error_description
            .unwrap_or_else(|| error.clone());
        tracing::warn!(
            provider = %provider_key,
            error = %error,
            "OAuth provider returned error"
        );
        return Ok(Redirect::temporary(&format!(
            "/login?error={}",
            urlencoding::encode(&desc)
        )));
    }

    let code = query.code.ok_or_else(|| {
        AppError::BadRequest("Missing authorization code".to_string())
    })?;
    let oauth_state = query.state.ok_or_else(|| {
        AppError::BadRequest("Missing OAuth state parameter".to_string())
    })?;

    // Validate and consume state from Redis (one-time use)
    let mut redis_conn = state
        .redis
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))?;

    let state_key = oauth_state_key(&oauth_state);
    let stored_state_json: Option<String> = redis_conn
        .get::<_, Option<String>>(&state_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read OAuth state: {}", e)))?;

    // Delete state immediately (one-time use)
    let _: () = redis_conn
        .del(&state_key)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete OAuth state: {}", e)))?;

    let stored_state_json = stored_state_json.ok_or_else(|| {
        AppError::BadRequest("Invalid or expired OAuth state".to_string())
    })?;

    let stored_state: OAuthStatePayload = serde_json::from_str(&stored_state_json)
        .map_err(|e| AppError::Internal(format!("Invalid OAuth state payload: {}", e)))?;

    if stored_state.provider_key != provider_key {
        return Err(AppError::BadRequest(
            "OAuth state provider mismatch".to_string(),
        ));
    }

    // Check state age (prevent replay attacks with stolen states)
    let state_age = chrono::Utc::now().timestamp() - stored_state.created_at;
    if state_age > OAUTH_STATE_TTL_SECONDS as i64 {
        return Err(AppError::BadRequest("OAuth state expired".to_string()));
    }

    // Load provider config
    let config: SsoConfig = sqlx::query_as(
        r#"
        SELECT 
            id, org_id, provider, provider_key, provider_type, display_name,
            issuer_url, client_id, client_secret_encrypted, scopes,
            idp_metadata_url, idp_entity_id, is_active, auto_provision,
            default_role, allow_domains, github_org, github_team,
            groups_claim, role_mappings, created_at, updated_at
        FROM sso_configs 
        WHERE provider_key = $1 AND is_active = true
        "#,
    )
    .bind(&provider_key)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        AppError::NotFound(format!("OAuth provider '{}' not found", provider_key))
    })?;

    let provider_type =
        SsoProviderType::from_str(&config.provider_type).ok_or_else(|| {
            AppError::Internal(format!("Unknown provider type: {}", config.provider_type))
        })?;

    let client_id = config.client_id.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider client_id not configured".to_string())
    })?;

    // Decrypt client secret
    let secret_raw = config.client_secret_encrypted.clone().ok_or_else(|| {
        AppError::BadRequest("OAuth provider client_secret not configured".to_string())
    })?;

    let client_secret = match crypto::decrypt(&secret_raw, &state.config.encryption_key) {
        Ok(secret) => secret,
        Err(_) if state.config.is_production() => {
            return Err(AppError::Internal(
                "Failed to decrypt OAuth client secret".to_string(),
            ));
        }
        Err(e) => {
            tracing::warn!(
                provider = %provider_key,
                error = %e,
                "Failed to decrypt secret, using raw value (dev mode)"
            );
            secret_raw
        }
    };

    let callback_url = format!("{}/api/v1/oauth2/{}/callback", get_site_url(&state.db).await, provider_key);

    // Exchange code for token and get user info based on provider type
    let (email, user_info) = match provider_type {
        SsoProviderType::GitHub => {
            exchange_github_token(
                &code,
                &client_id,
                &client_secret,
                &callback_url,
                &config,
            )
            .await?
        }
        SsoProviderType::Google | SsoProviderType::Oidc => {
            let issuer = config.issuer_url.clone().ok_or_else(|| {
                AppError::BadRequest("OIDC provider issuer_url not configured".to_string())
            })?;

            exchange_oidc_token(
                &code,
                &client_id,
                &client_secret,
                &callback_url,
                &issuer,
                stored_state.code_verifier.as_deref(),
                stored_state.nonce.as_deref(),
                &config,
            )
            .await?
        }
        SsoProviderType::Saml => {
            return Err(AppError::BadRequest("SAML not supported".to_string()));
        }
    };

    // Find or create user
    let _user = find_or_create_user(&state, &email, &user_info, &config, &provider_key).await?;

    // Generate JWT
    let token = crate::auth::create_token(
        _user.id,
        &_user.email,
        &_user.role,
        _user.org_id,
        &state.jwt_secret,
        state.jwt_expiry_hours,
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    // Redirect with token
    Ok(Redirect::temporary(&append_token_query(
        &stored_state.redirect_after,
        &token,
    )))
}

/// User info extracted from OAuth provider
struct UserInfo {
    email: String,
    name: Option<String>,
    preferred_username: Option<String>,
    groups: Vec<String>,
    external_id: Option<String>, // Provider's user ID (e.g., Google 'sub', GitHub 'id')
}

/// Exchange code for GitHub token and get user info
async fn exchange_github_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    config: &SsoConfig,
) -> Result<(String, UserInfo), AppError> {
    let client = reqwest::Client::new();

    // Exchange code for token
    let token_response = client
        .post(GITHUB_TOKEN_URL)
        .header("Accept", "application/json")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub token exchange failed: {}", e)))?;

    if !token_response.status().is_success() {
        let status = token_response.status();
        let body = token_response
            .text()
            .await
            .unwrap_or_default();
        return Err(AppError::Internal(format!(
            "GitHub token exchange failed: {} - {}",
            status, body
        )));
    }

    let tokens: TokenResponse = token_response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub token: {}", e)))?;

    // Get user info
    let user_response = client
        .get(format!("{}/user", GITHUB_API_URL))
        .header("Authorization", format!("token {}", tokens.access_token))
        .header("User-Agent", "RustChat")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("GitHub user request failed: {}", e)))?;

    if !user_response.status().is_success() {
        return Err(AppError::Internal(
            "Failed to fetch GitHub user info".to_string(),
        ));
    }

    let github_user: GitHubUser = user_response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse GitHub user: {}", e)))?;

    // Get primary verified email (GitHub may return null for email in user endpoint)
    let email = if github_user.email.is_some() {
        github_user.email.unwrap()
    } else {
        // Fetch emails from /user/emails endpoint
        let emails_response = client
            .get(format!("{}/user/emails", GITHUB_API_URL))
            .header("Authorization", format!("token {}", tokens.access_token))
            .header("User-Agent", "RustChat")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("GitHub emails request failed: {}", e)))?;

        if !emails_response.status().is_success() {
            return Err(AppError::Internal(
                "Failed to fetch GitHub emails".to_string(),
            ));
        }

        let emails: Vec<GitHubEmail> = emails_response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse GitHub emails: {}", e)))?;

        // Find primary verified email
        let primary_email = emails
            .iter()
            .find(|e| e.primary && e.verified)
            .or_else(|| emails.iter().find(|e| e.verified))
            .ok_or_else(|| {
                AppError::BadRequest(
                    "No verified email found for GitHub account".to_string(),
                )
            })?;

        primary_email.email.clone()
    };

    // Check GitHub organization/team restrictions if configured
    if let Some(ref org) = config.github_org {
        let org_check = client
            .get(format!("{}/orgs/{}/members/{}", GITHUB_API_URL, org, github_user.login))
            .header("Authorization", format!("token {}", tokens.access_token))
            .header("User-Agent", "RustChat")
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("GitHub org check failed: {}", e)))?;

        if org_check.status() != reqwest::StatusCode::NO_CONTENT {
            return Err(AppError::Forbidden(format!(
                "User is not a member of required GitHub organization: {}",
                org
            )));
        }

        // Check team if specified
        if let Some(ref team) = config.github_team {
            // First get the team ID by name
            let teams_response = client
                .get(format!("{}/orgs/{}/teams", GITHUB_API_URL, org))
                .header("Authorization", format!("token {}", tokens.access_token))
                .header("User-Agent", "RustChat")
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("GitHub teams request failed: {}", e)))?;

            if !teams_response.status().is_success() {
                return Err(AppError::Internal(
                    "Failed to fetch GitHub teams".to_string(),
                ));
            }

            let teams: Vec<serde_json::Value> = teams_response
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to parse GitHub teams: {}", e)))?;

            let team_id = teams
                .iter()
                .find(|t| t.get("slug").and_then(|s| s.as_str()) == Some(team))
                .and_then(|t| t.get("id").and_then(|id| id.as_i64()))
                .ok_or_else(|| {
                    AppError::Internal(format!("GitHub team '{}' not found in org '{}'", team, org))
                })?;

            let team_check = client
                .get(format!(
                    "{}/teams/{}/memberships/{}",
                    GITHUB_API_URL, team_id, github_user.login
                ))
                .header("Authorization", format!("token {}", tokens.access_token))
                .header("User-Agent", "RustChat")
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("GitHub team check failed: {}", e)))?;

            if !team_check.status().is_success() {
                return Err(AppError::Forbidden(format!(
                    "User is not a member of required GitHub team: {}",
                    team
                )));
            }
        }
    }

    let user_info = UserInfo {
        email: email.clone(),
        name: github_user.name,
        preferred_username: Some(github_user.login),
        groups: vec![],
        external_id: Some(github_user.id.to_string()),
    };

    Ok((email, user_info))
}

/// Exchange code for OIDC token and validate ID token
async fn exchange_oidc_token(
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    issuer: &str,
    code_verifier: Option<&str>,
    expected_nonce: Option<&str>,
    config: &SsoConfig,
) -> Result<(String, UserInfo), AppError> {
    let client = reqwest::Client::new();
    let discovery = OidcDiscoveryService::new();

    // Get OIDC configuration
    let discovery_result = discovery
        .discover(issuer)
        .await
        .map_err(|e| AppError::Internal(format!("OIDC discovery failed: {}", e)))?;

    // Build token request
    let mut form_params = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    // Add PKCE code verifier if present
    if let Some(verifier) = code_verifier {
        form_params.push(("code_verifier", verifier));
    }

    // Exchange code for tokens
    let token_response = client
        .post(&discovery_result.token_endpoint)
        .form(&form_params)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("OIDC token exchange failed: {}", e)))?;

    if !token_response.status().is_success() {
        let status = token_response.status();
        let body = token_response
            .text()
            .await
            .unwrap_or_default();
        return Err(AppError::Internal(format!(
            "OIDC token exchange failed: {} - {}",
            status, body
        )));
    }

    let tokens: TokenResponse = token_response
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse OIDC token: {}", e)))?;

    // Validate ID token if present
    let mut claims: Option<IdTokenClaims> = None;

    if let Some(ref id_token) = tokens.id_token {
        claims = Some(
            validate_id_token(
                id_token,
                &discovery_result.jwks_uri,
                client_id,
                issuer,
                expected_nonce,
            )
            .await?,
        );
    }

    // Get user info from ID token claims or userinfo endpoint
    let user_info = if let Some(ref c) = claims {
        // Use claims from ID token
        UserInfo {
            email: c.email.clone().unwrap_or_default(),
            name: c.name.clone(),
            preferred_username: c.preferred_username.clone(),
            groups: extract_groups(c, config.groups_claim.as_deref()),
            external_id: Some(c.sub.clone()),
        }
    } else if let Some(ref userinfo_url) = discovery_result.userinfo_endpoint {
        // Fall back to userinfo endpoint
        let userinfo_response = client
            .get(userinfo_url)
            .bearer_auth(&tokens.access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("UserInfo request failed: {}", e)))?;

        if !userinfo_response.status().is_success() {
            return Err(AppError::Internal(
                "Failed to fetch UserInfo".to_string(),
            ));
        }

        let userinfo: UserInfoResponse = userinfo_response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse UserInfo: {}", e)))?;

        let email = userinfo.email.clone().unwrap_or_default();
        let name = userinfo.name.clone();
        let preferred_username = userinfo.preferred_username.clone();
        let groups = extract_groups_from_userinfo(&userinfo, config.groups_claim.as_deref());

        UserInfo {
            email,
            name,
            preferred_username,
            groups,
            external_id: Some(userinfo.sub.clone()),
        }
    } else {
        return Err(AppError::Internal(
            "No ID token or UserInfo endpoint available".to_string(),
        ));
    };

    let email = user_info.email.clone();

    // Check email_verified claim if present
    if let Some(ref c) = claims {
        if c.email_verified == Some(false) {
            return Err(AppError::Forbidden(
                "Email not verified with OAuth provider".to_string(),
            ));
        }
    }

    // Check domain restrictions for Google
    if config.provider_type == "google" {
        if let Some(ref allowed_domains) = config.allow_domains {
            // Empty array means no restrictions (same as None)
            if !allowed_domains.is_empty() {
                let email_domain = email
                    .split('@')
                    .nth(1)
                    .ok_or_else(|| AppError::BadRequest("Invalid email format".to_string()))?;

                if !allowed_domains.contains(&email_domain.to_string()) {
                    return Err(AppError::Forbidden(format!(
                        "Email domain '{}' not allowed",
                        email_domain
                    )));
                }
            }
        }
    }

    Ok((email, user_info))
}

/// Extract groups from ID token claims
fn extract_groups(claims: &IdTokenClaims, groups_claim: Option<&str>) -> Vec<String> {
    let claim_name = groups_claim.unwrap_or("groups");

    // Try standard groups field first
    if let Some(ref groups) = claims.groups {
        return groups.clone();
    }

    // Try to find in extra claims
    if let Some(value) = claims.extra.get(claim_name) {
        if let Some(arr) = value.as_array() {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
    }

    vec![]
}

/// Extract groups from userinfo response
fn extract_groups_from_userinfo(
    userinfo: &UserInfoResponse,
    groups_claim: Option<&str>,
) -> Vec<String> {
    let claim_name = groups_claim.unwrap_or("groups");

    if let Some(value) = userinfo.extra.get(claim_name) {
        if let Some(arr) = value.as_array() {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
    }

    vec![]
}

/// Validate ID token signature and claims
async fn validate_id_token(
    id_token: &str,
    jwks_uri: &str,
    client_id: &str,
    expected_issuer: &str,
    expected_nonce: Option<&str>,
) -> Result<IdTokenClaims, AppError> {
    use jsonwebtoken::{decode, decode_header, Algorithm, Validation};

    // Decode header to get key ID
    let header = decode_header(id_token).map_err(|e| {
        AppError::Internal(format!("Failed to decode ID token header: {}", e))
    })?;

    // Fetch JWKS
    let discovery = OidcDiscoveryService::new();
    let jwks = discovery
        .fetch_jwks(jwks_uri)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch JWKS: {}", e)))?;

    // Find signing key
    let jwk = find_signing_key(&jwks, header.kid.as_deref()).ok_or_else(|| {
        AppError::Internal("No suitable signing key found in JWKS".to_string())
    })?;

    // Build decoding key from JWK
    let decoding_key = jwk_to_decoding_key(jwk)?;

    // Determine algorithm from header
    let algorithm = match header.alg {
        jsonwebtoken::Algorithm::RS256 => Algorithm::RS256,
        jsonwebtoken::Algorithm::RS384 => Algorithm::RS384,
        jsonwebtoken::Algorithm::RS512 => Algorithm::RS512,
        jsonwebtoken::Algorithm::ES256 => Algorithm::ES256,
        jsonwebtoken::Algorithm::ES384 => Algorithm::ES384,
        _ => Algorithm::RS256,
    };

    // Validate token
    let mut validation = Validation::new(algorithm);
    validation.set_audience(&[client_id]);
    validation.set_issuer(&[expected_issuer]);

    let token_data = decode::<IdTokenClaims>(id_token, &decoding_key, &validation).map_err(
        |e| AppError::Internal(format!("ID token validation failed: {}", e)),
    )?;

    let claims = token_data.claims;

    // Validate nonce if provided
    if let Some(expected) = expected_nonce {
        let actual = claims.nonce.as_deref().unwrap_or("");
        if actual != expected {
            return Err(AppError::Internal(
                "ID token nonce mismatch".to_string(),
            ));
        }
    }

    // Check token expiration
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(AppError::Internal("ID token expired".to_string()));
    }

    Ok(claims)
}

/// Convert JWK to DecodingKey
fn jwk_to_decoding_key(jwk: &crate::services::oidc_discovery::Jwk) -> Result<jsonwebtoken::DecodingKey, AppError> {
    use jsonwebtoken::DecodingKey;

    match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk.n.as_ref().ok_or_else(|| {
                AppError::Internal("RSA key missing modulus".to_string())
            })?;
            let e = jwk.e.as_ref().ok_or_else(|| {
                AppError::Internal("RSA key missing exponent".to_string())
            })?;
            DecodingKey::from_rsa_components(n, e).map_err(|e| {
                AppError::Internal(format!("Failed to build RSA decoding key: {}", e))
            })
        }
        "EC" => {
            // For EC keys, we need to use the x5c certificate chain or build from components
            if let Some(ref x5c) = jwk.x5c {
                if let Some(cert) = x5c.first() {
                    return DecodingKey::from_ec_pem(format!("-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n", cert).as_bytes())
                        .map_err(|e| AppError::Internal(format!("Failed to build EC decoding key from cert: {}", e)));
                }
            }
            Err(AppError::Internal(
                "EC key format not supported".to_string(),
            ))
        }
        _ => Err(AppError::Internal(format!(
            "Unsupported key type: {}",
            jwk.kty
        ))),
    }
}

/// Convert algorithm string to jsonwebtoken Algorithm
/// Find or create user from OAuth info
async fn find_or_create_user(
    state: &AppState,
    email: &str,
    user_info: &UserInfo,
    config: &SsoConfig,
    provider_key: &str,
) -> Result<crate::models::User, AppError> {
    use crate::models::User;

    // Look up existing user by email
    let existing: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&state.db)
        .await?;

    if let Some(user) = existing {
        // Update last login
        sqlx::query(
            r#"
            UPDATE users 
            SET last_login_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user.id)
        .execute(&state.db)
        .await?;

        return Ok(user);
    }

    // User not found - check if auto-provisioning is enabled
    if !config.auto_provision {
        return Err(AppError::Forbidden(
            "Account does not exist and auto-provisioning is disabled".to_string(),
        ));
    }

    // Determine role from mappings or use default
    let role = if let Some(ref mappings) = config.role_mappings {
        // Map groups to role
        let mappings_obj = mappings.as_object().unwrap_or(&serde_json::Map::new()).clone();
        let mut assigned_role = config.default_role.clone().unwrap_or_else(|| "member".to_string());

        for group in &user_info.groups {
            if let Some(role_val) = mappings_obj.get(group) {
                if let Some(role_str) = role_val.as_str() {
                    assigned_role = role_str.to_string();
                    break;
                }
            }
        }
        assigned_role
    } else {
        config.default_role.clone().unwrap_or_else(|| "member".to_string())
    };

    // Generate username from preferred_username, name, or email
    let username = user_info
        .preferred_username
        .clone()
        .or_else(|| {
            user_info.name.as_ref().map(|n| {
                n.to_lowercase()
                    .replace(' ', "_")
                    .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "")
            })
        })
        .unwrap_or_else(|| {
            email
                .split('@')
                .next()
                .unwrap_or("user")
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "")
        });

    // Ensure username is unique by appending numbers if needed
    let unique_username = generate_unique_username(&state.db, &username).await?;

    // Create new user (OAuth users have NULL password_hash)
    let user: User = sqlx::query_as(
        r#"
        INSERT INTO users (
            username, email, display_name, role, 
            is_active, auth_provider, auth_provider_id, org_id
        )
        VALUES ($1, $2, $3, $4, true, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(&unique_username)
    .bind(email)
    .bind(user_info.name.as_ref())
    .bind(&role)
    .bind(provider_key)
    .bind(user_info.external_id.as_ref())
    .bind(config.org_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create user: {}", e)))?;

    Ok(user)
}

/// Generate a unique username by appending numbers if needed
async fn generate_unique_username(
    db: &sqlx::PgPool,
    base_username: &str,
) -> Result<String, AppError> {
    // First try the base username
    let exists: Option<(bool,)> = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
        .bind(base_username)
        .fetch_optional(db)
        .await?;

    if exists.map_or(true, |(e,)| !e) {
        return Ok(base_username.to_string());
    }

    // Try appending numbers
    for i in 1..1000 {
        let candidate = format!("{}{}", base_username, i);
        let exists: Option<(bool,)> = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(&candidate)
            .fetch_optional(db)
            .await?;

        if exists.map_or(true, |(e,)| !e) {
            return Ok(candidate);
        }
    }

    // Fallback to UUID suffix
    let unique_suffix = Uuid::new_v4().to_string().split('-').next().unwrap_or("user").to_string();
    Ok(format!("{}_{}", base_username, unique_suffix))
}
