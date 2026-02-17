//! Mattermost Calls Plugin API
//!
//! Implements the com.mattermost.calls plugin interface for Mattermost Mobile compatibility.
//! Routes are mounted under /plugins/com.mattermost.calls/

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::sync::atomic::Ordering;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use crate::realtime::{EventType, WsBroadcast, WsEnvelope};

pub mod commands;
pub mod sfu;
pub mod state;
mod turn;

use flate2::read::ZlibDecoder;
use sfu::signaling::SignalingMessage;
pub use sfu::VoiceEvent;
use state::{CallState, Participant};
use turn::{TurnCredentialGenerator, TurnServerConfig};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

const CALLS_SIGNAL_EVENT: &str = "custom_com.mattermost.calls_signal";
const UNANSWERED_CALL_TIMEOUT_SECS: u64 = 20;
const EMPTY_CALL_TIMEOUT_SECS: u64 = 10;
static CHANNEL_CALLS_ENABLED: Lazy<DashMap<Uuid, bool>> = Lazy::new(DashMap::new);

/// Build the calls plugin router
pub fn router() -> Router<AppState> {
    Router::new()
        // Plugin info endpoints
        .route("/plugins/com.mattermost.calls/version", get(get_version))
        .route("/plugins/com.mattermost.calls/config", get(get_config))
        // Channels with calls enabled
        .route("/plugins/com.mattermost.calls/channels", get(get_channels))
        // Mattermost mobile compatibility: some clients call
        // /plugins/com.mattermost.calls/{channel_id}?mobilev2=true directly.
        .route(
            "/plugins/com.mattermost.calls/{channel_id}",
            get(get_channel_state_mobile).post(set_channel_calls_enabled),
        )
        // Call management endpoints
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/start",
            post(start_call),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/join",
            post(join_call),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/leave",
            post(leave_call),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/end",
            post(end_call_endpoint),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}",
            get(get_call_state),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/react",
            post(send_reaction),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/screen-share",
            post(toggle_screen_share),
        )
        // Mute/unmute endpoints
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/mute",
            post(mute_user),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/unmute",
            post(unmute_user),
        )
        // Raise/lower hand endpoints
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/raise-hand",
            post(raise_hand),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/lower-hand",
            post(lower_hand),
        )
        // WebRTC signaling endpoints
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/offer",
            post(handle_offer),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/ice",
            post(handle_ice_candidate),
        )
        // Host control endpoints
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/host/mute",
            post(host_mute),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/host/mute-others",
            post(host_mute_others),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/host/remove",
            post(host_remove_user),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/host/lower-hand",
            post(host_lower_hand),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/host/make",
            post(host_make_moderator),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/host/screen-off",
            post(host_screen_off),
        )
        // Notification endpoints
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/dismiss-notification",
            post(dismiss_notification),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/ring",
            post(ring_users),
        )
        .route(
            "/plugins/com.mattermost.calls/turn-credentials",
            get(get_turn_credentials),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/recording/start",
            post(start_recording),
        )
        .route(
            "/plugins/com.mattermost.calls/calls/{channel_id}/recording/stop",
            post(stop_recording),
        )
        // Slash commands
        .merge(commands::router())
}

/// Helper to resolve a channel ID which might be a UUID, a Mattermost encoded ID, or a DM name.
async fn resolve_channel_id(state: &AppState, channel_id: &str) -> ApiResult<Uuid> {
    let channel_id = channel_id.trim();
    if let Ok(uuid) = Uuid::parse_str(channel_id) {
        return Ok(uuid);
    }

    if let Some(uuid) = parse_mm_or_uuid(channel_id) {
        return Ok(uuid);
    }

    // Check if it's a DM name
    if crate::models::channel::parse_direct_channel_name(channel_id).is_some() {
        // Look up channel by name
        let channel_uuid: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM channels WHERE name = $1")
                .bind(channel_id)
                .fetch_optional(&state.db)
                .await?;

        if let Some(uuid) = channel_uuid {
            return Ok(uuid);
        }
    }

    Err(AppError::BadRequest("Invalid channel_id".to_string()))
}

// ============ Response Models ============

#[derive(Debug, Serialize)]
struct VersionResponse {
    version: String,
    rtcd: bool,
}

#[derive(Debug, Serialize)]
struct ConfigResponse {
    #[serde(rename = "ICEServersConfigs")]
    ice_servers_configs: Vec<IceServer>,
    #[serde(rename = "NeedsTURNCredentials")]
    needs_turn_credentials: bool,
    #[serde(rename = "DefaultEnabled")]
    default_enabled: bool,
    #[serde(rename = "AllowEnableCalls")]
    allow_enable_calls: bool,
    #[serde(rename = "GroupCallsAllowed")]
    group_calls_allowed: bool,
    #[serde(rename = "EnableRinging")]
    enable_ringing: bool,
    #[serde(rename = "HostControlsAllowed")]
    host_controls_allowed: bool,
    #[serde(rename = "EnableRecordings")]
    enable_recordings: bool,
    #[serde(rename = "MaxCallParticipants")]
    max_call_participants: i32,
    #[serde(rename = "AllowScreenSharing")]
    allow_screen_sharing: bool,
    #[serde(rename = "EnableSimulcast")]
    enable_simulcast: bool,
    #[serde(rename = "EnableAV1")]
    enable_av1: bool,
    #[serde(rename = "MaxRecordingDuration")]
    max_recording_duration: i32,
    #[serde(rename = "TranscribeAPI")]
    transcribe_api: String,
    #[serde(rename = "sku_short_name")]
    sku_short_name: String,
    #[serde(rename = "EnableDCSignaling")]
    enable_dc_signaling: bool,
    #[serde(rename = "EnableTranscriptions")]
    enable_transcriptions: bool,
    #[serde(rename = "EnableLiveCaptions")]
    enable_live_captions: bool,
}

#[derive(Debug, Serialize)]
struct IceServer {
    urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credential: Option<String>,
    #[serde(rename = "credentialType", skip_serializing_if = "Option::is_none")]
    credential_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct StartCallResponse {
    id: String,
    id_raw: String,
    channel_id: String,
    channel_id_raw: String,
    start_at: i64,
    owner_id: String,
    owner_id_raw: String,
    host_id: String,
    host_id_raw: String,
}

#[derive(Debug, Serialize)]
struct CallStateResponse {
    id: String,
    id_raw: String,
    channel_id: String,
    channel_id_raw: String,
    start_at: i64,
    owner_id: String,
    owner_id_raw: String,
    host_id: String,
    host_id_raw: String,
    participants: Vec<String>,
    participants_raw: Vec<String>,
    sessions: HashMap<String, CallSessionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen_sharing_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen_sharing_id_raw: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen_sharing_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen_sharing_session_id_raw: Option<String>,
    thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recording: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dismissed_notification: Option<HashMap<String, bool>>,
}

#[derive(Debug, Serialize)]
struct CallSessionResponse {
    session_id: String,
    session_id_raw: String,
    user_id: String,
    user_id_raw: String,
    username: String,
    display_name: String,
    unmuted: bool,
    raised_hand: i32,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ChannelEnableRequest {
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct ReactionRequest {
    emoji: String,
}

#[derive(Debug, Deserialize)]
struct HostControlRequest {
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct HostMakeRequest {
    new_host_id: String,
}

fn is_system_admin(auth: &MmAuthUser) -> bool {
    auth.role
        .split_whitespace()
        .any(|role| role == "system_admin")
}

fn can_manage_call(auth: &MmAuthUser, call: &CallState) -> bool {
    call.host_id == auth.user_id || is_system_admin(auth)
}

fn is_host_session_active(state: &AppState, call: &CallState) -> bool {
    let Some(host_participant) = call.participants.get(&call.host_id) else {
        return false;
    };
    state
        .connection_store
        .get_connection(&host_participant.session_id.to_string())
        .map(|connection| connection.is_active.load(Ordering::SeqCst))
        .unwrap_or(false)
}

// WebRTC Signaling Request/Response structs
#[derive(Debug, Deserialize)]
pub struct OfferRequest {
    pub sdp: String,
}

#[derive(Debug, Serialize)]
pub struct AnswerResponse {
    pub sdp: String,
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct IceCandidateRequest {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

// ============ Effective config from database ============

/// Effective calls config resolved from database overrides + env var defaults.
/// The admin console saves to `server_config.plugins->'calls'`; env vars are only
/// the fallback for fields that were never saved via the admin UI.
struct EffectiveCallsConfig {
    turn_server_enabled: bool,
    turn_server_url: String,
    turn_server_username: String,
    turn_server_credential: String,
    turn_static_auth_secret: String,
    stun_servers: Vec<String>,
}

fn ensure_protocol(url: &str, protocol: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return url.to_string();
    }
    let lower = url.to_lowercase();
    // For TURN, we also accept turns:
    if lower.starts_with(protocol) || (protocol == "turn:" && lower.starts_with("turns:")) {
        url.to_string()
    } else {
        format!("{}{}", protocol, url)
    }
}

async fn load_effective_calls_config(state: &AppState) -> EffectiveCallsConfig {
    // Try to read the database-saved config (same query the admin GET uses)
    let db_config: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT plugins->'calls' FROM server_config WHERE id = 'default'")
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

    if let Some((json,)) = db_config {
        if let Some(obj) = json.as_object() {
            return EffectiveCallsConfig {
                turn_server_enabled: obj
                    .get("turn_server_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(state.config.calls.turn_server_enabled),
                turn_server_url: obj
                    .get("turn_server_url")
                    .and_then(|v| v.as_str())
                    .map(|s| ensure_protocol(s, "turn:"))
                    .unwrap_or_else(|| {
                        ensure_protocol(&state.config.calls.turn_server_url, "turn:")
                    }),
                turn_server_username: obj
                    .get("turn_server_username")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| state.config.calls.turn_server_username.clone()),
                turn_server_credential: obj
                    .get("turn_server_credential")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| state.config.calls.turn_server_credential.clone()),
                turn_static_auth_secret: obj
                    .get("turn_static_auth_secret")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| state.config.calls.turn_static_auth_secret.clone()),
                stun_servers: obj
                    .get("stun_servers")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| ensure_protocol(s, "stun:"))
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        state
                            .config
                            .calls
                            .stun_servers
                            .iter()
                            .map(|s| ensure_protocol(s, "stun:"))
                            .collect()
                    }),
            };
        }
    }

    // No database overrides — use env var defaults
    EffectiveCallsConfig {
        turn_server_enabled: state.config.calls.turn_server_enabled,
        turn_server_url: state.config.calls.turn_server_url.clone(),
        turn_server_username: state.config.calls.turn_server_username.clone(),
        turn_server_credential: state.config.calls.turn_server_credential.clone(),
        turn_static_auth_secret: state.config.calls.turn_static_auth_secret.clone(),
        stun_servers: state.config.calls.stun_servers.clone(),
    }
}

// ============ Handlers ============

/// GET /plugins/com.mattermost.calls/version
/// Returns plugin version info
async fn get_version(State(_state): State<AppState>) -> ApiResult<Json<VersionResponse>> {
    Ok(Json(VersionResponse {
        version: "0.28.0".to_string(),
        rtcd: false, // We're using integrated mode
    }))
}

/// GET /plugins/com.mattermost.calls/config
/// Returns ICE server configuration.
/// TURN credentials are NOT included here — clients must call /turn-credentials separately.
async fn get_config(
    State(state): State<AppState>,
    _auth: MmAuthUser,
) -> ApiResult<Json<ConfigResponse>> {
    let effective = load_effective_calls_config(&state).await;

    // Build ice servers list — STUN only.
    // TURN is intentionally omitted from this response because including a credential-less
    // TURN entry causes browsers to attempt (and fail) auth. The client already handles
    // `NeedsTURNCredentials: true` by fetching proper creds via /turn-credentials.
    let mut ice_servers = vec![];

    for stun_url in &effective.stun_servers {
        ice_servers.push(IceServer {
            urls: vec![stun_url.clone()],
            username: None,
            credential: None,
            credential_type: None,
        });
    }

    Ok(Json(ConfigResponse {
        ice_servers_configs: ice_servers,
        needs_turn_credentials: effective.turn_server_enabled,
        default_enabled: true,
        allow_enable_calls: true,
        group_calls_allowed: true,
        enable_ringing: true,
        host_controls_allowed: true,
        enable_recordings: false,
        max_call_participants: 0,
        allow_screen_sharing: true,
        enable_simulcast: false,
        enable_av1: false,
        max_recording_duration: 60,
        transcribe_api: "whisper.cpp".to_string(),
        sku_short_name: "starter".to_string(),
        enable_dc_signaling: false,
        enable_transcriptions: false,
        enable_live_captions: false,
    }))
}

/// GET /plugins/com.mattermost.calls/turn-credentials
/// Returns TURN credentials (static from admin config, or ephemeral via HMAC)
async fn get_turn_credentials(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<IceServer>>> {
    let effective = load_effective_calls_config(&state).await;

    if !effective.turn_server_enabled {
        return Err(AppError::BadRequest("TURN server is disabled".to_string()));
    }

    let turn_config = TurnServerConfig {
        enabled: true,
        url: effective.turn_server_url.clone(),
        username: effective.turn_server_username.clone(),
        credential: effective.turn_server_credential.clone(),
    };

    // If static credentials are provided (via admin console), use them directly.
    // Otherwise, generate ephemeral HMAC-SHA1 credentials using the best available secret.
    let generator = if turn_config.username.is_empty() || turn_config.credential.is_empty() {
        // Prefer explicit TURN static auth secret; fallback to general encryption key
        let secret = if !effective.turn_static_auth_secret.is_empty() {
            effective.turn_static_auth_secret.clone()
        } else {
            state.config.encryption_key.clone()
        };

        TurnCredentialGenerator::with_rest_api(secret, state.config.calls.turn_ttl_minutes)
    } else {
        TurnCredentialGenerator::with_static_credentials(turn_config)
    };

    let credentials = generator.generate_credentials(&auth.user_id.to_string());

    Ok(Json(vec![IceServer {
        urls: vec![effective.turn_server_url],
        username: Some(credentials.username),
        credential: Some(credentials.credential),
        credential_type: Some("password".to_string()),
    }]))
}

/// GET /plugins/com.mattermost.calls/channels
/// Returns channels with calls enabled/active calls
async fn get_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<CallChannelInfo>>> {
    let call_manager = state.call_state_manager.as_ref();
    let active_calls = call_manager.get_all_calls().await;
    let mut calls_by_channel: HashMap<Uuid, Option<CallState>> = active_calls
        .into_iter()
        .map(|call| (call.channel_id, Some(call)))
        .collect();

    for entry in CHANNEL_CALLS_ENABLED.iter() {
        let override_channel_id: Uuid = *entry.key();
        calls_by_channel.entry(override_channel_id).or_insert(None);
    }

    let mut channels = Vec::new();
    for (channel_id, call) in calls_by_channel {
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
        )
        .bind(channel_id)
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);

        if !is_member {
            continue;
        }

        channels.push(
            build_call_channel_info(&state, channel_id, channel_calls_enabled(channel_id), call)
                .await?,
        );
    }

    Ok(Json(channels))
}

/// Channel call info response
#[derive(Debug, Serialize)]
struct CallChannelInfo {
    channel_id: String,
    channel_id_raw: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_id_raw: Option<String>,
    enabled: bool,
    has_call: bool,
    participant_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    call: Option<CallStateResponse>,
}

fn channel_calls_enabled(channel_id: Uuid) -> bool {
    CHANNEL_CALLS_ENABLED
        .get(&channel_id)
        .map(|entry| *entry)
        .unwrap_or(true)
}

async fn build_call_channel_info(
    state: &AppState,
    channel_uuid: Uuid,
    enabled: bool,
    call: Option<CallState>,
) -> ApiResult<CallChannelInfo> {
    let (call_id, call_id_raw, call_state, participant_count) = if let Some(call) = call {
        let participant_count = call.participants.len() as i32;
        let call_state = Some(
            build_call_state_response(state, &call, encode_mm_id(channel_uuid), channel_uuid)
                .await?,
        );
        (
            Some(encode_mm_id(call.call_id)),
            Some(call.call_id.to_string()),
            call_state,
            participant_count,
        )
    } else {
        (None, None, None, 0)
    };

    Ok(CallChannelInfo {
        channel_id: encode_mm_id(channel_uuid),
        channel_id_raw: channel_uuid.to_string(),
        call_id,
        call_id_raw,
        enabled,
        has_call: call_state.is_some(),
        participant_count,
        call: call_state,
    })
}

async fn build_call_state_response(
    state: &AppState,
    call: &CallState,
    channel_id_for_response: String,
    channel_uuid: Uuid,
) -> ApiResult<CallStateResponse> {
    let thread_id = ensure_call_thread_id(state, call).await;

    let call_participants = state
        .call_state_manager
        .get_participants(call.call_id)
        .await;

    let user_ids: Vec<Uuid> = call_participants.iter().map(|p| p.user_id).collect();
    let users_info: HashMap<Uuid, (String, String)> = if !user_ids.is_empty() {
        sqlx::query("SELECT id, username, COALESCE(display_name, '') as display_name FROM users WHERE id = ANY($1)")
            .bind(&user_ids)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|row| {
                use sqlx::Row;
                let id: Uuid = row.get(0);
                let username: String = row.get(1);
                let display_name: String = row.get(2);
                (id, (username, display_name))
            })
            .collect()
    } else {
        HashMap::new()
    };

    let participants: Vec<String> = call_participants
        .iter()
        .map(|p| encode_mm_id(p.user_id))
        .collect();
    let participants_raw: Vec<String> = call_participants
        .iter()
        .map(|p| p.user_id.to_string())
        .collect();
    let sessions: HashMap<String, CallSessionResponse> = call_participants
        .iter()
        .map(|participant| {
            let raw_session_id = participant.session_id.to_string();
            let (username, display_name) = users_info
                .get(&participant.user_id)
                .cloned()
                .unwrap_or_else(|| (participant.user_id.to_string(), String::new()));

            (
                raw_session_id.clone(),
                CallSessionResponse {
                    session_id: raw_session_id,
                    session_id_raw: participant.session_id.to_string(),
                    user_id: encode_mm_id(participant.user_id),
                    user_id_raw: participant.user_id.to_string(),
                    username,
                    display_name,
                    unmuted: !participant.muted,
                    raised_hand: if participant.hand_raised { 1 } else { 0 },
                },
            )
        })
        .collect();
    let screen_sharing_session = call.screen_sharer.and_then(|screen_sharer| {
        call_participants
            .iter()
            .find(|participant| participant.user_id == screen_sharer)
    });
    let dismissed_notification: HashMap<String, bool> = call
        .dismissed_users
        .iter()
        .map(|user_id| (encode_mm_id(*user_id), true))
        .collect();

    Ok(CallStateResponse {
        id: encode_mm_id(call.call_id),
        id_raw: call.call_id.to_string(),
        channel_id: channel_id_for_response,
        channel_id_raw: channel_uuid.to_string(),
        start_at: call.started_at,
        owner_id: encode_mm_id(call.owner_id),
        owner_id_raw: call.owner_id.to_string(),
        host_id: encode_mm_id(call.host_id),
        host_id_raw: call.host_id.to_string(),
        participants,
        participants_raw,
        sessions,
        screen_sharing_id: call.screen_sharer.map(encode_mm_id),
        screen_sharing_id_raw: call.screen_sharer.map(|id| id.to_string()),
        screen_sharing_session_id: screen_sharing_session
            .map(|participant| participant.session_id.to_string()),
        screen_sharing_session_id_raw: screen_sharing_session
            .map(|participant| participant.session_id.to_string()),
        thread_id: thread_id.map(encode_mm_id),
        recording: None,
        dismissed_notification: Some(dismissed_notification),
    })
}

#[derive(sqlx::FromRow)]
struct CallThreadPostRow {
    id: Uuid,
    created_at: chrono::DateTime<Utc>,
    seq: i64,
}

async fn create_call_thread_post(
    state: &AppState,
    call_id: Uuid,
    channel_id: Uuid,
    owner_id: Uuid,
    started_at: i64,
) -> Result<Uuid, sqlx::Error> {
    let props = serde_json::json!({
        "type": "custom_calls",
        "call_id": encode_mm_id(call_id),
        "start_at": started_at,
        "end_at": 0,
        "participants": [encode_mm_id(owner_id)],
    });

    let post: CallThreadPostRow = sqlx::query_as(
        r#"
        INSERT INTO posts (channel_id, user_id, message, props, file_ids)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, created_at, seq
        "#,
    )
    .bind(channel_id)
    .bind(owner_id)
    .bind("")
    .bind(&props)
    .bind(Vec::<Uuid>::new())
    .fetch_one(&state.db)
    .await?;

    let mm_post = crate::mattermost_compat::models::Post {
        id: encode_mm_id(post.id),
        create_at: post.created_at.timestamp_millis(),
        update_at: post.created_at.timestamp_millis(),
        delete_at: 0,
        edit_at: 0,
        user_id: encode_mm_id(owner_id),
        channel_id: encode_mm_id(channel_id),
        root_id: String::new(),
        original_id: String::new(),
        message: String::new(),
        post_type: "custom_calls".to_string(),
        props,
        hashtags: String::new(),
        file_ids: Vec::new(),
        pending_post_id: String::new(),
        metadata: None,
    };

    let broadcast = WsEnvelope::event(EventType::MessageCreated, mm_post, Some(channel_id))
        .with_broadcast(WsBroadcast {
            channel_id: Some(channel_id),
            team_id: None,
            user_id: None,
            exclude_user_id: None,
        });
    state.ws_hub.broadcast(broadcast).await;

    let _ =
        crate::services::unreads::increment_unreads(state, channel_id, owner_id, post.seq).await;

    Ok(post.id)
}

async fn ensure_call_thread_id(state: &AppState, call: &CallState) -> Option<Uuid> {
    if let Some(thread_id) = call.thread_id {
        return Some(thread_id);
    }

    match create_call_thread_post(
        state,
        call.call_id,
        call.channel_id,
        call.owner_id,
        call.started_at,
    )
    .await
    {
        Ok(thread_id) => {
            state
                .call_state_manager
                .set_thread_id(call.call_id, Some(thread_id))
                .await;
            Some(thread_id)
        }
        Err(err) => {
            warn!(
                call_id = %call.call_id,
                channel_id = %call.channel_id,
                error = %err,
                "calls failed to create call thread post"
            );
            None
        }
    }
}

/// GET /plugins/com.mattermost.calls/{channel_id}
/// Returns mobile-compatible channel call state envelope.
async fn get_channel_state_mobile(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<CallChannelInfo>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;

    let call = state
        .call_state_manager
        .get_call_by_channel(&channel_uuid)
        .await;
    let payload = build_call_channel_info(
        &state,
        channel_uuid,
        channel_calls_enabled(channel_uuid),
        call,
    )
    .await?;
    Ok(Json(payload))
}

/// POST /plugins/com.mattermost.calls/{channel_id}
/// Enable or disable calls in a channel.
async fn set_channel_calls_enabled(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<ChannelEnableRequest>,
) -> ApiResult<Json<CallChannelInfo>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;

    CHANNEL_CALLS_ENABLED.insert(channel_uuid, payload.enabled);

    broadcast_call_event(
        &state,
        if payload.enabled {
            "custom_com.mattermost.calls_channel_enable_voice"
        } else {
            "custom_com.mattermost.calls_channel_disable_voice"
        },
        &channel_uuid,
        serde_json::json!({
            "enabled": payload.enabled,
        }),
        None,
    )
    .await;

    let call = state
        .call_state_manager
        .get_call_by_channel(&channel_uuid)
        .await;
    let response = build_call_channel_info(&state, channel_uuid, payload.enabled, call).await?;
    Ok(Json(response))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/start
/// Starts a new call in a channel
async fn start_call(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StartCallResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        "calls.start_call requested"
    );

    // Check channel permissions
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;
    if !channel_calls_enabled(channel_uuid) {
        return Err(AppError::Forbidden(
            "Calls are disabled in this channel".to_string(),
        ));
    }

    // Get or initialize call state manager
    let call_manager = state.call_state_manager.as_ref();

    // Check if call already exists
    if let Some(call) = call_manager.get_call_by_channel(&channel_uuid).await {
        info!(
            user_id = %auth.user_id,
            channel_id = %channel_uuid,
            call_id = %call.call_id,
            owner_id = %call.owner_id,
            "calls.start_call reused existing active call"
        );
        return Ok(Json(StartCallResponse {
            id: encode_mm_id(call.call_id),
            id_raw: call.call_id.to_string(),
            channel_id: channel_id.clone(),
            channel_id_raw: channel_uuid.to_string(),
            start_at: call.started_at,
            owner_id: encode_mm_id(call.owner_id),
            owner_id_raw: call.owner_id.to_string(),
            host_id: encode_mm_id(call.host_id),
            host_id_raw: call.host_id.to_string(),
        }));
    }

    // Create new call
    let call_id = Uuid::new_v4();
    let now = Utc::now().timestamp_millis();

    let call = CallState {
        call_id,
        channel_id: channel_uuid,
        owner_id: auth.user_id,
        host_id: auth.user_id,
        started_at: now,
        participants: HashMap::new(),
        screen_sharer: None,
        thread_id: None,
        dismissed_users: HashSet::new(),
    };

    call_manager.add_call(call.clone()).await;
    debug!(
        call_id = %call_id,
        channel_id = %channel_uuid,
        owner_id = %auth.user_id,
        "calls.start_call call state created"
    );

    let thread_id = ensure_call_thread_id(&state, &call).await;

    // Add owner as first participant (muted by default)
    let participant = Participant {
        user_id: auth.user_id,
        session_id: Uuid::new_v4(),
        joined_at: now,
        muted: true,
        screen_sharing: false,
        hand_raised: false,
    };

    call_manager
        .add_participant(call_id, participant.clone())
        .await;
    debug!(
        call_id = %call_id,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        "calls.start_call owner participant added"
    );

    // Get or create SFU for this call
    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create SFU: {}", e)))?;

    // Add owner as participant in the SFU
    let (_, signaling_rx) = sfu
        .add_participant(auth.user_id, participant.session_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to add participant to SFU: {}", e)))?;
    spawn_signaling_forwarder(
        &state,
        channel_uuid,
        auth.user_id,
        participant.session_id,
        signaling_rx,
    );
    debug!(
        call_id = %call_id,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        "calls.start_call signaling forwarder spawned"
    );

    // Broadcast call_start event
    // Note: thread_id is used as post_id for call posts in Mattermost
    let thread_id_str = thread_id.map(encode_mm_id).unwrap_or_default();
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_call_start",
        &channel_uuid,
        serde_json::json!({
            "id": encode_mm_id(call_id),
            "channel_id": channel_id,
            "channelID": encode_mm_id(channel_uuid),
            "user_id": encode_mm_id(auth.user_id),
            "call_id": encode_mm_id(call_id),
            "start_at": now,
            "owner_id": encode_mm_id(auth.user_id),
            "host_id": encode_mm_id(auth.user_id),
            "thread_id": thread_id_str.clone(),
            "post_id": thread_id_str,  // Mobile expects post_id for navigation
        }),
        Some(auth.user_id), // Exclude sender
    )
    .await;

    // Broadcast user_joined event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_joined",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "session_id": participant.session_id.to_string(),
            "muted": true,
            "raised_hand": false,
        }),
        None,
    )
    .await;

    // Send ringing notifications to all channel members
    // This ensures push notifications are sent for calls in ALL channel types
    // (DMs, groups, and regular channels)
    broadcast_ringing_event(
        &state,
        channel_uuid,
        call_id,
        auth.user_id,
        Some(auth.user_id),
    )
    .await;

    broadcast_call_state_event(&state, channel_uuid, None).await;

    // Mattermost-compatible behavior: if nobody else joins, drop the call after a ring timeout.
    schedule_unanswered_call_timeout(&state, call_id, channel_uuid);

    info!(
        call_id = %call_id,
        channel_id = %channel_uuid,
        owner_id = %auth.user_id,
        session_id = %participant.session_id,
        "calls.start_call completed"
    );

    Ok(Json(StartCallResponse {
        id: encode_mm_id(call_id),
        id_raw: call_id.to_string(),
        channel_id: channel_id.clone(),
        channel_id_raw: channel_uuid.to_string(),
        start_at: now,
        owner_id: encode_mm_id(auth.user_id),
        owner_id_raw: auth.user_id.to_string(),
        host_id: encode_mm_id(auth.user_id),
        host_id_raw: auth.user_id.to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/join
/// Join an existing call
async fn join_call(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        "calls.join_call requested"
    );

    // Check channel permissions
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;
    if !channel_calls_enabled(channel_uuid) {
        return Err(AppError::Forbidden(
            "Calls are disabled in this channel".to_string(),
        ));
    }

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find active call in channel
    let call = match call_manager.get_call_by_channel(&channel_uuid).await {
        Some(c) => c,
        None => call_manager
            .get_call(channel_uuid)
            .await
            .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?,
    };

    // Check if user already in call
    if call_manager
        .get_participant(call.call_id, auth.user_id)
        .await
        .is_some()
    {
        info!(
            user_id = %auth.user_id,
            channel_id = %channel_uuid,
            call_id = %call.call_id,
            "calls.join_call user already in call"
        );
        return Ok(Json(StatusResponse {
            status: "OK".to_string(),
        }));
    }

    // Add participant
    let now = Utc::now().timestamp_millis();
    let participant = Participant {
        user_id: auth.user_id,
        session_id: Uuid::new_v4(),
        joined_at: now,
        muted: true,
        screen_sharing: false,
        hand_raised: false,
    };

    call_manager
        .add_participant(call.call_id, participant.clone())
        .await;
    debug!(
        call_id = %call.call_id,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        "calls.join_call participant added to call state"
    );

    // Get or create SFU for this call
    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get or create SFU: {}", e)))?;

    // Add participant to the SFU
    let (_, signaling_rx) = sfu
        .add_participant(auth.user_id, participant.session_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to add participant to SFU: {}", e)))?;
    spawn_signaling_forwarder(
        &state,
        channel_uuid,
        auth.user_id,
        participant.session_id,
        signaling_rx,
    );
    debug!(
        call_id = %call.call_id,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        "calls.join_call signaling forwarder spawned"
    );

    // Broadcast user_joined event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_joined",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "session_id": participant.session_id.to_string(),
            "muted": true,
            "raised_hand": false,
        }),
        None,
    )
    .await;
    broadcast_call_state_event(&state, channel_uuid, None).await;

    info!(
        call_id = %call.call_id,
        channel_id = %channel_uuid,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        "calls.join_call completed"
    );

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/leave
/// Leave a call
async fn leave_call(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        "calls.leave_call requested"
    );

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = match call_manager.get_call_by_channel(&channel_uuid).await {
        Some(c) => c,
        None => {
            debug!(
                channel_id = %channel_uuid,
                "calls.leave_call: no active call found, returning success"
            );
            return Ok(Json(StatusResponse {
                status: "OK".to_string(),
            }));
        }
    };

    // Get participant info before removing (for session_id)
    let participant = call_manager
        .get_participant(call.call_id, auth.user_id)
        .await;

    // Remove participant from call manager
    call_manager
        .remove_participant(call.call_id, auth.user_id)
        .await;

    // Remove participant from SFU if exists
    if let Some(participant) = participant {
        if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
            let _ = sfu.remove_participant(participant.session_id).await;
            debug!(
                call_id = %call.call_id,
                user_id = %auth.user_id,
                session_id = %participant.session_id,
                "calls.leave_call participant removed from SFU"
            );
        }
    }

    // Broadcast user_left event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_left",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
        }),
        None,
    )
    .await;
    let remaining =
        reconcile_after_participant_left(&state, call.call_id, channel_uuid, auth.user_id).await;
    if remaining <= 1 {
        schedule_empty_call_timeout(&state, call.call_id, channel_uuid);
        info!(
            call_id = %call.call_id,
            channel_id = %channel_uuid,
            remaining_participants = remaining,
            timeout_secs = EMPTY_CALL_TIMEOUT_SECS,
            "calls.leave_call scheduled no-remote-participant timeout"
        );
    } else {
        info!(
            call_id = %call.call_id,
            channel_id = %channel_uuid,
            remaining_participants = remaining,
            "calls.leave_call completed"
        );
    }

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/end
/// End a call (host only).
async fn end_call_endpoint(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_or_call_uuid = resolve_channel_id(&state, &channel_id).await?;
    let call_manager = state.call_state_manager.as_ref();

    let mut call = match call_manager
        .get_call_by_channel(&channel_or_call_uuid)
        .await
    {
        Some(c) => c,
        None => match call_manager.get_call(channel_or_call_uuid).await {
            Some(c) => c,
            None => {
                return Ok(Json(StatusResponse {
                    status: "OK".to_string(),
                }));
            }
        },
    };

    check_channel_permission(&state, auth.user_id, call.channel_id).await?;
    call = normalize_call_host_if_stale(&state, call).await;

    let caller_is_participant = call.participants.contains_key(&auth.user_id);
    let caller_is_only_participant = call.participants.len() <= 1 && caller_is_participant;
    let host_session_inactive = caller_is_participant && !is_host_session_active(&state, &call);
    if !can_manage_call(&auth, &call) && !caller_is_only_participant && !host_session_inactive {
        return Err(AppError::Forbidden(
            "Only the host can end this call".to_string(),
        ));
    }

    end_call(
        &state,
        call.call_id,
        call.channel_id,
        "ended_by_host",
        call.participants.len(),
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/recording/start
async fn start_recording(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_or_call_uuid = resolve_channel_id(&state, &channel_id).await?;
    let call = match state
        .call_state_manager
        .get_call_by_channel(&channel_or_call_uuid)
        .await
    {
        Some(c) => Some(c),
        None => {
            state
                .call_state_manager
                .get_call(channel_or_call_uuid)
                .await
        }
    };
    if let Some(call) = call {
        check_channel_permission(&state, auth.user_id, call.channel_id).await?;
    }
    Err(AppError::BadRequest(
        "Call recording is not supported by this server".to_string(),
    ))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/recording/stop
async fn stop_recording(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_or_call_uuid = resolve_channel_id(&state, &channel_id).await?;
    let call = match state
        .call_state_manager
        .get_call_by_channel(&channel_or_call_uuid)
        .await
    {
        Some(c) => Some(c),
        None => {
            state
                .call_state_manager
                .get_call(channel_or_call_uuid)
                .await
        }
    };
    if let Some(call) = call {
        check_channel_permission(&state, auth.user_id, call.channel_id).await?;
    }
    Err(AppError::BadRequest(
        "Call recording is not supported by this server".to_string(),
    ))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/host/screen-off
async fn host_screen_off(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<HostControlRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let target_session_id = parse_mm_or_uuid(&payload.session_id)
        .ok_or_else(|| AppError::BadRequest("Invalid session_id".to_string()))?;

    let call_manager = state.call_state_manager.as_ref();
    let mut call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    call = normalize_call_host_if_stale(&state, call).await;

    if !can_manage_call(&auth, &call) {
        return Err(AppError::Forbidden(
            "Only the host can stop screen sharing".to_string(),
        ));
    }

    let target_user_id = call
        .participants
        .values()
        .find(|p| p.session_id == target_session_id)
        .map(|p| p.user_id)
        .ok_or_else(|| AppError::NotFound("Participant not found in call".to_string()))?;

    call_manager
        .set_screen_sharing(call.call_id, target_user_id, false)
        .await;
    if call.screen_sharer == Some(target_user_id) {
        call_manager.set_screen_sharer(call.call_id, None).await;
    }

    if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
        sfu.set_screen_sharing(target_session_id, false).await;
    }

    broadcast_screen_share_event(
        &state,
        channel_uuid,
        target_user_id,
        target_session_id,
        false,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// GET /plugins/com.mattermost.calls/calls/{channel_id}
/// Get current call state
async fn get_call_state(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Response> {
    let normalized_channel_id = channel_id.trim();
    if normalized_channel_id.is_empty()
        || normalized_channel_id.eq_ignore_ascii_case("undefined")
        || normalized_channel_id.eq_ignore_ascii_case("null")
    {
        return Err(AppError::NotFound(
            "No active call in this channel".to_string(),
        ));
    }

    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = match call_manager.get_call_by_channel(&channel_uuid).await {
        Some(c) => c,
        None => {
            // Try looking up by Call ID as a fallback if Channel ID lookup failed
            match call_manager.get_call(channel_uuid).await {
                Some(c) => c,
                None => {
                    // Return silent 404 to avoid noisy ERROR logs for a common client polling case
                    let body = crate::error::ErrorResponse {
                        error: crate::error::ErrorBody {
                            code: "NOT_FOUND".to_string(),
                            message: "No active call in this channel".to_string(),
                            details: None,
                        },
                    };
                    return Ok((axum::http::StatusCode::NOT_FOUND, Json(body)).into_response());
                }
            }
        }
    };
    Ok(
        Json(build_call_state_response(&state, &call, channel_id.clone(), channel_uuid).await?)
            .into_response(),
    )
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/react
/// Send a reaction during call
async fn send_reaction(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<ReactionRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let timestamp = Utc::now().timestamp_millis();
    let emoji_name = crate::mattermost_compat::emoji_data::get_short_name_for_emoji(&payload.emoji);

    let session_id = state
        .call_state_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .and_then(|call| {
            call.participants
                .get(&auth.user_id)
                .map(|participant| participant.session_id.to_string())
        })
        .unwrap_or_default();

    // Broadcast reaction event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_reacted",
        &channel_uuid,
        serde_json::json!({
            "user_id": encode_mm_id(auth.user_id),
            "session_id": session_id,
            "reaction": payload.emoji,
            "timestamp": timestamp,
            "emoji": {
                "name": emoji_name,
                "literal": payload.emoji,
            },
        }),
        None,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/screen-share
/// Toggle screen sharing
async fn toggle_screen_share(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    // Check if user is in call
    let participant = call_manager
        .get_participant(call.call_id, auth.user_id)
        .await
        .ok_or_else(|| AppError::Forbidden("You are not in this call".to_string()))?;

    // Toggle screen sharing
    let is_sharing = !participant.screen_sharing;
    call_manager
        .set_screen_sharing(call.call_id, auth.user_id, is_sharing)
        .await;

    // Update SFU screen sharing state for track forwarding
    if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
        sfu.set_screen_sharing(participant.session_id, is_sharing)
            .await;
        info!(
            call_id = %call.call_id,
            session_id = %participant.session_id,
            is_sharing = is_sharing,
            "SFU screen sharing state updated"
        );
    }

    // Update global screen sharer
    if is_sharing {
        call_manager
            .set_screen_sharer(call.call_id, Some(auth.user_id))
            .await;
    } else if call.screen_sharer == Some(auth.user_id) {
        call_manager.set_screen_sharer(call.call_id, None).await;
    }

    broadcast_screen_share_event(
        &state,
        channel_uuid,
        auth.user_id,
        participant.session_id,
        is_sharing,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/mute
/// Mute self
async fn mute_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    // Get user's session_id from participants
    let participants = call_manager.get_participants(call.call_id).await;
    let session_id = participants
        .iter()
        .find(|p| p.user_id == auth.user_id)
        .map(|p| p.session_id.to_string())
        .unwrap_or_default();

    // Set muted
    call_manager
        .set_muted(call.call_id, auth.user_id, true)
        .await;

    // Broadcast user_muted event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_muted",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "session_id": session_id,
            "muted": true,
        }),
        None,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/unmute
/// Unmute self
async fn unmute_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    // Get user's session_id from participants
    let participants = call_manager.get_participants(call.call_id).await;
    let session_id = participants
        .iter()
        .find(|p| p.user_id == auth.user_id)
        .map(|p| p.session_id.to_string())
        .unwrap_or_default();

    // Set unmuted
    call_manager
        .set_muted(call.call_id, auth.user_id, false)
        .await;

    // Broadcast user_unmuted event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_unmuted",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "session_id": session_id,
            "muted": false,
        }),
        None,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/raise-hand
/// Raise hand
async fn raise_hand(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    let participant = call_manager
        .get_participant(call.call_id, auth.user_id)
        .await
        .ok_or_else(|| AppError::Forbidden("You are not in this call".to_string()))?;

    // Set hand raised
    call_manager
        .set_hand_raised(call.call_id, auth.user_id, true)
        .await;

    broadcast_raise_hand_event(
        &state,
        channel_uuid,
        auth.user_id,
        participant.session_id,
        true,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/lower-hand
/// Lower hand
async fn lower_hand(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    let participant = call_manager
        .get_participant(call.call_id, auth.user_id)
        .await
        .ok_or_else(|| AppError::Forbidden("You are not in this call".to_string()))?;

    // Set hand lowered
    call_manager
        .set_hand_raised(call.call_id, auth.user_id, false)
        .await;

    broadcast_raise_hand_event(
        &state,
        channel_uuid,
        auth.user_id,
        participant.session_id,
        false,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/host/mute
/// Mute a participant by host
async fn host_mute(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<HostControlRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let target_session_id = parse_mm_or_uuid(&payload.session_id)
        .ok_or_else(|| AppError::BadRequest("Invalid session_id".to_string()))?;

    let call_manager = state.call_state_manager.as_ref();
    let mut call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    call = normalize_call_host_if_stale(&state, call).await;

    // Authorize: Only host can mute others
    if !can_manage_call(&auth, &call) {
        return Err(AppError::Forbidden(
            "Only the host can mute other participants".to_string(),
        ));
    }

    // Find target user by session_id
    let target_user_id = call
        .participants
        .values()
        .find(|p| p.session_id == target_session_id)
        .map(|p| p.user_id)
        .ok_or_else(|| AppError::NotFound("Participant not found in call".to_string()))?;

    // Mute in state
    call_manager
        .set_muted(call.call_id, target_user_id, true)
        .await;

    // Send host_mute event to the target user
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_host_mute",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "session_id": payload.session_id,
        }),
        Some(target_user_id),
    )
    .await;

    // Also broadcast regular muted event for UI updates
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_muted",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(target_user_id),
            "session_id": payload.session_id,
            "muted": true,
        }),
        None,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/host/mute-others
/// Mute all participants except host
async fn host_mute_others(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    let call_manager = state.call_state_manager.as_ref();
    let mut call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    call = normalize_call_host_if_stale(&state, call).await;

    if !can_manage_call(&auth, &call) {
        return Err(AppError::Forbidden(
            "Only the host can mute other participants".to_string(),
        ));
    }

    for participant in call.participants.values() {
        if participant.user_id == auth.user_id {
            continue;
        }

        call_manager
            .set_muted(call.call_id, participant.user_id, true)
            .await;

        // Signal each user
        broadcast_call_event(
            &state,
            "custom_com.mattermost.calls_host_mute",
            &channel_uuid,
            serde_json::json!({
                "channel_id": channel_id,
                "session_id": participant.session_id.to_string(),
            }),
            Some(participant.user_id),
        )
        .await;

        // Broadcast for UI
        broadcast_call_event(
            &state,
            "custom_com.mattermost.calls_user_muted",
            &channel_uuid,
            serde_json::json!({
                "channel_id": channel_id,
                "user_id": encode_mm_id(participant.user_id),
                "session_id": participant.session_id.to_string(),
                "muted": true,
            }),
            None,
        )
        .await;
    }

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/host/remove
/// Remove a participant from the call
async fn host_remove_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<HostControlRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let target_session_id = parse_mm_or_uuid(&payload.session_id)
        .ok_or_else(|| AppError::BadRequest("Invalid session_id".to_string()))?;

    let call_manager = state.call_state_manager.as_ref();
    let mut call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    call = normalize_call_host_if_stale(&state, call).await;

    if !can_manage_call(&auth, &call) {
        return Err(AppError::Forbidden(
            "Only the host can remove participants".to_string(),
        ));
    }

    let target_user_id = call
        .participants
        .values()
        .find(|p| p.session_id == target_session_id)
        .map(|p| p.user_id)
        .ok_or_else(|| AppError::NotFound("Participant not found in call".to_string()))?;

    if target_user_id == auth.user_id {
        return Err(AppError::BadRequest(
            "Host cannot remove themselves with this endpoint; use leave_call instead".to_string(),
        ));
    }

    // Signal host removal to target
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_host_removed",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "session_id": payload.session_id,
        }),
        Some(target_user_id),
    )
    .await;

    // Remove from state
    call_manager
        .remove_participant(call.call_id, target_user_id)
        .await;

    // Remove from SFU
    if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
        let _ = sfu.remove_participant(target_session_id).await;
    }

    // Broadcast user_left for everyone
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_left",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(target_user_id),
            "session_id": payload.session_id,
        }),
        None,
    )
    .await;

    let remaining =
        reconcile_after_participant_left(&state, call.call_id, channel_uuid, target_user_id).await;
    if remaining <= 1 {
        schedule_empty_call_timeout(&state, call.call_id, channel_uuid);
    }

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/host/lower-hand
/// Lower a participant's hand
async fn host_lower_hand(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<HostControlRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let target_session_id = parse_mm_or_uuid(&payload.session_id)
        .ok_or_else(|| AppError::BadRequest("Invalid session_id".to_string()))?;

    let call_manager = state.call_state_manager.as_ref();
    let mut call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    call = normalize_call_host_if_stale(&state, call).await;

    if !can_manage_call(&auth, &call) {
        return Err(AppError::Forbidden(
            "Only the host can lower hands".to_string(),
        ));
    }

    let target_user_id = call
        .participants
        .values()
        .find(|p| p.session_id == target_session_id)
        .map(|p| p.user_id)
        .ok_or_else(|| AppError::NotFound("Participant not found in call".to_string()))?;

    // Lower hand in state
    call_manager
        .set_hand_raised(call.call_id, target_user_id, false)
        .await;

    // Signal target user
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_host_lower_hand",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "session_id": payload.session_id,
            "call_id": encode_mm_id(call.call_id),
            "host_id": encode_mm_id(auth.user_id),
        }),
        Some(target_user_id),
    )
    .await;

    let payload_json = serde_json::json!({
        "channel_id": channel_id,
        "user_id": encode_mm_id(target_user_id),
        "raised_hand": 0,
        "session_id": payload.session_id,
    });
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_unraise_hand",
        &channel_uuid,
        payload_json.clone(),
        None,
    )
    .await;
    // Legacy alias kept for compatibility with existing rustchat consumers.
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_lower_hand",
        &channel_uuid,
        payload_json,
        None,
    )
    .await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/host/make
/// Transfer host status
async fn host_make_moderator(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<HostMakeRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let new_host_uuid = parse_mm_or_uuid(&payload.new_host_id)
        .ok_or_else(|| AppError::BadRequest("Invalid new_host_id".to_string()))?;

    let call_manager = state.call_state_manager.as_ref();
    let mut call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    call = normalize_call_host_if_stale(&state, call).await;

    if !can_manage_call(&auth, &call) {
        return Err(AppError::Forbidden(
            "Only the host can transfer host status".to_string(),
        ));
    }

    // Verify new host is a participant
    if !call.participants.contains_key(&new_host_uuid) {
        return Err(AppError::BadRequest(
            "New host must be a participant in the call".to_string(),
        ));
    }

    // Transfer host in state
    call_manager.set_host(call.call_id, new_host_uuid).await;

    broadcast_host_changed_event(&state, channel_uuid, new_host_uuid).await;
    broadcast_call_state_event(&state, channel_uuid, None).await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/ring
/// Send ringing notification to all channel participants
async fn ring_users(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;

    // Check if call exists
    let call_manager = state.call_state_manager.as_ref();
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call to ring".to_string()))?;

    let thread_id = ensure_call_thread_id(&state, &call).await;

    // Mattermost-mobile compatibility: stock mobile clients trigger incoming call UX
    // from calls_call_start and do not handle calls_ringing directly.
    // Note: thread_id is used as post_id for call posts in Mattermost
    let thread_id_str = thread_id.map(encode_mm_id).unwrap_or_default();
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_call_start",
        &channel_uuid,
        serde_json::json!({
            "id": encode_mm_id(call.call_id),
            "channelID": encode_mm_id(channel_uuid),
            "start_at": call.started_at,
            "owner_id": encode_mm_id(call.owner_id),
            "host_id": encode_mm_id(call.host_id),
            "thread_id": thread_id_str.clone(),
            "post_id": thread_id_str,  // Mobile expects post_id for navigation
            "call_id": encode_mm_id(call.call_id),
            "channel_id": encode_mm_id(channel_uuid),
            "user_id": encode_mm_id(call.owner_id),
        }),
        Some(auth.user_id),
    )
    .await;

    broadcast_ringing_event(&state, channel_uuid, call.call_id, auth.user_id, None).await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/dismiss-notification
/// Dismiss incoming call ringing notification
async fn dismiss_notification(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    let call = state
        .call_state_manager
        .get_call_by_channel(&channel_uuid)
        .await;
    let call_id = if let Some(call) = call {
        state
            .call_state_manager
            .dismiss_user_notification(call.call_id, auth.user_id)
            .await;
        encode_mm_id(call.call_id)
    } else {
        String::new()
    };

    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_dismissed_notification",
        &channel_uuid,
        serde_json::json!({
            "userID": encode_mm_id(auth.user_id),
            "user_id": encode_mm_id(auth.user_id),
            "callID": call_id,
            "call_id": call_id,
        }),
        None,
    )
    .await;
    broadcast_call_state_event(&state, channel_uuid, None).await;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/offer
/// Receives SDP offer from client, creates peer connection in SFU, returns SDP answer
async fn handle_offer(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<OfferRequest>,
) -> ApiResult<Json<AnswerResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        sdp_len = payload.sdp.len(),
        "calls.offer received"
    );

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = match call_manager.get_call_by_channel(&channel_uuid).await {
        Some(c) => c,
        None => call_manager
            .get_call(channel_uuid)
            .await
            .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?,
    };

    // Get participant session_id
    let participant = call_manager
        .get_participant(call.call_id, auth.user_id)
        .await
        .ok_or_else(|| AppError::Forbidden("You are not in this call".to_string()))?;

    // Get or create SFU for this call. In multi-node or resumed-state scenarios,
    // call state can exist before a local SFU is hydrated.
    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get or create SFU: {}", e)))?;

    info!(call_id = %call.call_id, "SFU retrieved/created");

    // Ensure this participant is present in the SFU before handling signaling.
    // Also ensure the signaling forwarder is running to send ICE candidates to the client.
    let signaling_rx = if !sfu.has_participant(participant.session_id).await {
        warn!(
            call_id = %call.call_id,
            user_id = %auth.user_id,
            session_id = %participant.session_id,
            "calls.offer participant missing in SFU; recovering by re-registering"
        );
        let (_, rx) = sfu
            .add_participant(auth.user_id, participant.session_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add participant to SFU: {}", e)))?;
        Some(rx)
    } else {
        // Participant exists but we need to ensure signaling forwarder is running
        // Get the signaling receiver for the existing participant
        sfu.get_signaling_receiver(participant.session_id).await
    };

    // Spawn signaling forwarder if we have a receiver (new participant or reconnection)
    if let Some(rx) = signaling_rx {
        spawn_signaling_forwarder(
            &state,
            channel_uuid,
            auth.user_id,
            participant.session_id,
            rx,
        );
    }

    // Parse the offer SDP (keep raw SDP for potential retry)
    let sdp_raw = payload.sdp;
    let offer = RTCSessionDescription::offer(sdp_raw.clone())
        .map_err(|e| AppError::BadRequest(format!("Invalid SDP offer: {}", e)))?;

    // Handle the offer and get answer.
    // If it fails (e.g. dead PeerConnection), recreate the participant and retry once.
    let answer = match sfu.handle_offer(participant.session_id, offer).await {
        Ok(ans) => ans,
        Err(first_err) => {
            warn!(
                session_id = %participant.session_id,
                error = %first_err,
                "sfu.handle_offer failed; recreating PeerConnection and retrying"
            );

            let (_, signaling_rx) = sfu
                .recreate_participant(auth.user_id, participant.session_id)
                .await
                .map_err(|e| {
                    error!(session_id = %participant.session_id, error = %e, "recreate_participant failed");
                    AppError::Internal(format!("Failed to recreate participant: {}", e))
                })?;

            spawn_signaling_forwarder(
                &state,
                channel_uuid,
                auth.user_id,
                participant.session_id,
                signaling_rx,
            );

            let retry_offer = RTCSessionDescription::offer(sdp_raw)
                .map_err(|e| AppError::Internal(format!("Invalid SDP on retry: {}", e)))?;

            sfu.handle_offer(participant.session_id, retry_offer)
                .await
                .map_err(|e| {
                    error!(session_id = %participant.session_id, error = %e, "sfu.handle_offer retry also failed");
                    AppError::Internal(format!("Failed to handle offer after retry: {}", e))
                })?
        }
    };
    debug!(
        call_id = %call.call_id,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        answer_sdp_len = answer.sdp.len(),
        "calls.offer handled successfully"
    );

    // Extract SDP from answer
    let sdp = answer.sdp;
    send_signaling_event(
        &state,
        channel_uuid,
        auth.user_id,
        participant.session_id,
        SignalingMessage::Answer { sdp: sdp.clone() },
    )
    .await;

    Ok(Json(AnswerResponse {
        sdp,
        type_: "answer".to_string(),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/ice
/// Receives ICE candidate from client and adds it to the peer connection
async fn handle_ice_candidate(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<IceCandidateRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = resolve_channel_id(&state, &channel_id).await?;

    let candidate_len = payload.candidate.len();
    debug!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        candidate_len = candidate_len,
        sdp_mid = ?payload.sdp_mid,
        sdp_mline_index = ?payload.sdp_mline_index,
        "calls.ice received candidate"
    );

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = match call_manager.get_call_by_channel(&channel_uuid).await {
        Some(c) => c,
        None => match call_manager.get_call(channel_uuid).await {
            Some(c) => c,
            None => {
                warn!(
                    user_id = %auth.user_id,
                    channel_id = %channel_uuid,
                    "Ignoring ICE candidate: no active call in this channel"
                );
                return Ok(Json(StatusResponse {
                    status: "IGNORED".to_string(),
                }));
            }
        },
    };

    // Get participant session_id
    let Some(participant) = call_manager
        .get_participant(call.call_id, auth.user_id)
        .await
    else {
        warn!(
            user_id = %auth.user_id,
            call_id = %call.call_id,
            "Ignoring ICE candidate: user is not a participant of the call"
        );
        return Ok(Json(StatusResponse {
            status: "IGNORED".to_string(),
        }));
    };

    // Get SFU for this call
    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get or create SFU: {}", e)))?;

    if !sfu.has_participant(participant.session_id).await {
        warn!(
            call_id = %call.call_id,
            user_id = %auth.user_id,
            session_id = %participant.session_id,
            "calls.ice participant missing in SFU; recovering by re-registering"
        );
        let (_, signaling_rx) = sfu
            .add_participant(auth.user_id, participant.session_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to add participant to SFU: {}", e)))?;
        spawn_signaling_forwarder(
            &state,
            call.channel_id,
            auth.user_id,
            participant.session_id,
            signaling_rx,
        );
    }

    // Handle the ICE candidate
    sfu.handle_ice_candidate(
        participant.session_id,
        payload.candidate,
        payload.sdp_mid,
        payload.sdp_mline_index,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to handle ICE candidate: {}", e)))?;
    debug!(
        call_id = %call.call_id,
        user_id = %auth.user_id,
        session_id = %participant.session_id,
        candidate_len = candidate_len,
        "calls.ice handled successfully"
    );

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

// ============ Helper Functions ============

async fn broadcast_screen_share_event(
    state: &AppState,
    channel_id: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    is_on: bool,
) {
    let call = state
        .call_state_manager
        .get_call_by_channel(&channel_id)
        .await;
    let call_id = call.map(|c| c.call_id).unwrap_or_default();

    let payload = serde_json::json!({
        "user_id": encode_mm_id(user_id),
        "user_id_raw": user_id.to_string(),
        "session_id": session_id.to_string(),
        "session_id_raw": session_id.to_string(),
    });

    debug!(
        call_id = %call_id,
        channel_id = %channel_id,
        user_id = %user_id,
        session_id = %session_id,
        is_on = is_on,
        "calls.broadcast_screen_share_event"
    );

    broadcast_call_event(
        state,
        if is_on {
            "custom_com.mattermost.calls_user_screen_on"
        } else {
            "custom_com.mattermost.calls_user_screen_off"
        },
        &channel_id,
        payload.clone(),
        None,
    )
    .await;

    // Legacy aliases kept for compatibility with existing rustchat consumers.
    broadcast_call_event(
        state,
        if is_on {
            "custom_com.mattermost.calls_screen_on"
        } else {
            "custom_com.mattermost.calls_screen_off"
        },
        &channel_id,
        payload,
        None,
    )
    .await;
}

async fn broadcast_raise_hand_event(
    state: &AppState,
    channel_id: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    raised: bool,
) {
    let raised_hand = if raised {
        Utc::now().timestamp_millis()
    } else {
        0
    };
    let payload = serde_json::json!({
        "user_id": encode_mm_id(user_id),
        "session_id": session_id.to_string(),
        "raised_hand": raised_hand,
    });

    broadcast_call_event(
        state,
        if raised {
            "custom_com.mattermost.calls_user_raise_hand"
        } else {
            "custom_com.mattermost.calls_user_unraise_hand"
        },
        &channel_id,
        payload.clone(),
        None,
    )
    .await;

    // Legacy aliases kept for compatibility with existing rustchat consumers.
    broadcast_call_event(
        state,
        if raised {
            "custom_com.mattermost.calls_raise_hand"
        } else {
            "custom_com.mattermost.calls_lower_hand"
        },
        &channel_id,
        payload,
        None,
    )
    .await;
}

fn select_next_host(participants: &HashMap<Uuid, Participant>) -> Option<Uuid> {
    participants
        .values()
        .min_by_key(|participant| (participant.joined_at, participant.user_id))
        .map(|participant| participant.user_id)
}

async fn broadcast_host_changed_event(state: &AppState, channel_id: Uuid, new_host_id: Uuid) {
    let encoded_host_id = encode_mm_id(new_host_id);
    let event_payload = serde_json::json!({
        "hostID": encoded_host_id,
        "host_id": encoded_host_id,
    });

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_call_host_changed",
        &channel_id,
        event_payload.clone(),
        None,
    )
    .await;
    // Legacy alias kept for compatibility with existing rustchat consumers.
    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_host_changed",
        &channel_id,
        event_payload,
        None,
    )
    .await;
}

async fn normalize_call_host_if_stale(state: &AppState, call: CallState) -> CallState {
    if call.participants.is_empty() || call.participants.contains_key(&call.host_id) {
        return call;
    }

    let Some(new_host_id) = select_next_host(&call.participants) else {
        return call;
    };

    state
        .call_state_manager
        .set_host(call.call_id, new_host_id)
        .await;
    broadcast_host_changed_event(state, call.channel_id, new_host_id).await;
    broadcast_call_state_event(state, call.channel_id, None).await;

    state
        .call_state_manager
        .get_call(call.call_id)
        .await
        .unwrap_or(call)
}

async fn reconcile_after_participant_left(
    state: &AppState,
    call_id: Uuid,
    channel_id: Uuid,
    departed_user_id: Uuid,
) -> usize {
    let mut call = match state.call_state_manager.get_call(call_id).await {
        Some(call) => call,
        None => return 0,
    };

    if call.host_id == departed_user_id {
        if let Some(new_host_id) = select_next_host(&call.participants) {
            state
                .call_state_manager
                .set_host(call.call_id, new_host_id)
                .await;
            broadcast_host_changed_event(state, channel_id, new_host_id).await;
            if let Some(updated_call) = state.call_state_manager.get_call(call_id).await {
                call = updated_call;
            }
        }
    } else if !call.participants.is_empty() && !call.participants.contains_key(&call.host_id) {
        if let Some(new_host_id) = select_next_host(&call.participants) {
            state
                .call_state_manager
                .set_host(call.call_id, new_host_id)
                .await;
            broadcast_host_changed_event(state, channel_id, new_host_id).await;
            if let Some(updated_call) = state.call_state_manager.get_call(call_id).await {
                call = updated_call;
            }
        }
    }

    broadcast_call_state_event(state, channel_id, None).await;

    call.participants.len()
}

/// Handle websocket actions used by Mattermost mobile calls.
/// Returns `true` when the action is recognized and handled.
pub async fn handle_ws_action(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    action: &str,
    data: Option<&Value>,
) -> bool {
    let Some(call_action) = action.strip_prefix("custom_com.mattermost.calls_") else {
        return false;
    };

    let result = match call_action {
        "join" | "reconnect" => handle_ws_join_call(state, user_id, connection_id, data).await,
        "leave" => handle_ws_leave_call(state, user_id, connection_id).await,
        "sdp" => handle_ws_sdp(state, user_id, connection_id, data).await,
        "ice" => handle_ws_ice(state, user_id, connection_id, data).await,
        "mute" => handle_ws_mute(state, user_id, connection_id, data, true).await,
        "unmute" => handle_ws_mute(state, user_id, connection_id, data, false).await,
        "raise_hand" => handle_ws_raise_hand(state, user_id, connection_id, true).await,
        "unraise_hand" => handle_ws_raise_hand(state, user_id, connection_id, false).await,
        "react" => handle_ws_reaction(state, user_id, connection_id, data).await,
        "metric" => {
            debug!(
                user_id = %user_id,
                connection_id = connection_id,
                data = ?data,
                "calls.ws metric received"
            );
            Ok(())
        }
        other => {
            warn!(
                user_id = %user_id,
                connection_id = connection_id,
                action = other,
                "calls.ws unsupported action"
            );
            Ok(())
        }
    };

    if let Err(err) = result {
        error!(
            user_id = %user_id,
            connection_id = connection_id,
            action = action,
            error = %err,
            "calls.ws action failed"
        );
        send_ws_plugin_error(state, user_id, connection_id, &err).await;
    }

    true
}

async fn handle_ws_join_call(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    data: Option<&Value>,
) -> Result<(), String> {
    let data = data.ok_or_else(|| "Missing join payload".to_string())?;
    let conn_uuid = resolve_ws_session_uuid(connection_id, Some(data))?;
    let channel_uuid = parse_join_channel_id(data)?;

    check_channel_permission(state, user_id, channel_uuid)
        .await
        .map_err(|e| e.to_string())?;
    if !channel_calls_enabled(channel_uuid) {
        return Err("Calls are disabled in this channel".to_string());
    }

    let call_manager = state.call_state_manager.as_ref();
    let now = Utc::now().timestamp_millis();

    let mut created_call = false;
    let call = if let Some(call) = call_manager.get_call_by_channel(&channel_uuid).await {
        call
    } else {
        created_call = true;
        let call = CallState {
            call_id: Uuid::new_v4(),
            channel_id: channel_uuid,
            owner_id: user_id,
            host_id: user_id,
            started_at: now,
            participants: HashMap::new(),
            screen_sharer: None,
            thread_id: data
                .get("threadID")
                .and_then(|v| v.as_str())
                .and_then(parse_mm_or_uuid),
            dismissed_users: HashSet::new(),
        };
        call_manager.add_call(call.clone()).await;
        call
    };

    let mut should_add_participant = true;
    if let Some(existing) = call_manager.get_participant(call.call_id, user_id).await {
        if existing.session_id == conn_uuid {
            should_add_participant = false;
        } else {
            if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
                let _ = sfu.remove_participant(existing.session_id).await;
            }
        }
    }

    if should_add_participant {
        call_manager
            .add_participant(
                call.call_id,
                Participant {
                    user_id,
                    session_id: conn_uuid,
                    joined_at: now,
                    muted: true,
                    screen_sharing: false,
                    hand_raised: false,
                },
            )
            .await;
    }

    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| format!("Failed to get or create SFU: {e}"))?;

    if !sfu.has_participant(conn_uuid).await {
        let _ = sfu
            .add_participant(user_id, conn_uuid)
            .await
            .map_err(|e| format!("Failed to add participant to SFU: {e}"))?;
    }

    let thread_id = ensure_call_thread_id(state, &call).await;

    if created_call {
        schedule_unanswered_call_timeout(state, call.call_id, channel_uuid);
        broadcast_call_event(
            state,
            "custom_com.mattermost.calls_call_start",
            &channel_uuid,
            serde_json::json!({
                "id": encode_mm_id(call.call_id),
                "channelID": encode_mm_id(channel_uuid),
                "start_at": call.started_at,
                "owner_id": encode_mm_id(call.owner_id),
                "host_id": encode_mm_id(call.owner_id),
                "thread_id": thread_id.map(encode_mm_id),
                "call_id": encode_mm_id(call.call_id),
                "channel_id": encode_mm_id(channel_uuid),
            }),
            None,
        )
        .await;

        // Send ringing notifications via push for mobile apps
        // WebSocket join doesn't go through HTTP /start endpoint, so we need to trigger ringing here
        broadcast_ringing_event(state, channel_uuid, call.call_id, user_id, Some(user_id)).await;
    }

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_user_joined",
        &channel_uuid,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": conn_uuid.to_string(),
            "muted": true,
            "raised_hand": 0,
        }),
        None,
    )
    .await;

    broadcast_call_state_event(state, channel_uuid, None).await;

    send_ws_plugin_event(
        state,
        user_id,
        "custom_com.mattermost.calls_join",
        serde_json::json!({
            "connID": connection_id,
            "conn_id": connection_id,
            "channelID": encode_mm_id(channel_uuid),
            "channel_id": encode_mm_id(channel_uuid),
            "channel_id_raw": channel_uuid.to_string(),
            "callID": encode_mm_id(call.call_id),
            "call_id": encode_mm_id(call.call_id),
            "call_id_raw": call.call_id.to_string(),
            "sessionID": conn_uuid.to_string(),
            "session_id": conn_uuid.to_string(),
        }),
    )
    .await;

    info!(
        user_id = %user_id,
        connection_id = connection_id,
        channel_id = %channel_uuid,
        call_id = %call.call_id,
        created_call = created_call,
        "calls.ws join handled"
    );

    Ok(())
}

async fn handle_ws_sdp(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    data: Option<&Value>,
) -> Result<(), String> {
    let requested_session_id = resolve_ws_session_uuid(connection_id, data)?;

    info!(
        user_id = %user_id,
        connection_id = connection_id,
        "calls.ws sdp received"
    );

    let sdp = parse_ws_sdp_payload(data).map_err(|e| {
        error!(
            user_id = %user_id,
            connection_id = connection_id,
            error = %e,
            "Failed to parse SDP payload"
        );
        format!("Invalid SDP payload: {e}")
    })?;

    let (call, session_id) =
        resolve_call_for_ws_connection(state, user_id, requested_session_id).await?;

    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| format!("Failed to get or create SFU: {e}"))?;

    if !sfu.has_participant(session_id).await {
        info!(
            user_id = %user_id,
            session_id = %session_id,
            "Adding participant to SFU for SDP handling"
        );
        let _ = sfu
            .add_participant(user_id, session_id)
            .await
            .map_err(|e| format!("Failed to add participant to SFU: {e}"))?;
    }

    let offer = RTCSessionDescription::offer(sdp).map_err(|e| format!("Invalid offer SDP: {e}"))?;

    info!(
        user_id = %user_id,
        session_id = %session_id,
        "Processing SDP offer"
    );

    let answer = sfu
        .handle_offer(session_id, offer)
        .await
        .map_err(|e| format!("Failed to handle offer: {e}"))?;

    info!(
        user_id = %user_id,
        session_id = %session_id,
        sdp_length = answer.sdp.len(),
        "Sending SDP answer"
    );

    send_ws_plugin_signal(
        state,
        user_id,
        connection_id,
        serde_json::json!({
            "type": "answer",
            "sdp": answer.sdp,
        }),
    )
    .await;

    Ok(())
}

async fn handle_ws_ice(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    data: Option<&Value>,
) -> Result<(), String> {
    let requested_session_id = resolve_ws_session_uuid(connection_id, data)?;

    debug!(
        user_id = %user_id,
        connection_id = connection_id,
        "calls.ws ice received"
    );

    let (candidate, sdp_mid, sdp_mline_index) = parse_ws_ice_payload(data).map_err(|e| {
        error!(
            user_id = %user_id,
            connection_id = connection_id,
            error = %e,
            "Failed to parse ICE payload"
        );
        format!("Invalid ICE payload: {e}")
    })?;

    let (call, session_id) =
        resolve_call_for_ws_connection(state, user_id, requested_session_id).await?;

    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| format!("Failed to get or create SFU: {e}"))?;

    if !sfu.has_participant(session_id).await {
        info!(
            user_id = %user_id,
            session_id = %session_id,
            "Adding participant to SFU for ICE handling"
        );
        let _ = sfu
            .add_participant(user_id, session_id)
            .await
            .map_err(|e| format!("Failed to add participant to SFU: {e}"))?;
    }

    info!(
        user_id = %user_id,
        session_id = %session_id,
        candidate_len = candidate.len(),
        sdp_mid = ?sdp_mid,
        sdp_mline_index = ?sdp_mline_index,
        "Processing ICE candidate"
    );

    sfu.handle_ice_candidate(session_id, candidate, sdp_mid, sdp_mline_index)
        .await
        .map_err(|e| format!("Failed to handle ICE candidate: {e}"))?;

    Ok(())
}

async fn handle_ws_leave_call(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
) -> Result<(), String> {
    let requested_session_id = resolve_ws_session_uuid(connection_id, None)?;
    let Ok((call, session_id)) =
        resolve_call_for_ws_connection(state, user_id, requested_session_id).await
    else {
        return Ok(());
    };

    let call_manager = state.call_state_manager.as_ref();
    call_manager.remove_participant(call.call_id, user_id).await;
    if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
        let _ = sfu.remove_participant(session_id).await;
    }

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_user_left",
        &call.channel_id,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": session_id.to_string(),
        }),
        None,
    )
    .await;

    let remaining =
        reconcile_after_participant_left(state, call.call_id, call.channel_id, user_id).await;
    if remaining <= 1 {
        schedule_empty_call_timeout(state, call.call_id, call.channel_id);
    }

    Ok(())
}

/// Best-effort cleanup for abrupt websocket disconnects where no explicit
/// calls_leave websocket action was delivered.
pub async fn handle_ws_connection_closed(state: &AppState, user_id: Uuid, connection_id: &str) {
    let Ok(session_id) = Uuid::parse_str(connection_id) else {
        return;
    };
    let Some(call) = find_call_for_session(state, user_id, session_id).await else {
        return;
    };

    state
        .call_state_manager
        .remove_participant(call.call_id, user_id)
        .await;
    if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
        let _ = sfu.remove_participant(session_id).await;
    }

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_user_left",
        &call.channel_id,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": session_id.to_string(),
        }),
        None,
    )
    .await;

    let remaining =
        reconcile_after_participant_left(state, call.call_id, call.channel_id, user_id).await;
    if remaining <= 1 {
        schedule_empty_call_timeout(state, call.call_id, call.channel_id);
    }

    info!(
        user_id = %user_id,
        session_id = %session_id,
        call_id = %call.call_id,
        channel_id = %call.channel_id,
        remaining_participants = remaining,
        "calls.ws cleaned up disconnected participant"
    );
}

async fn handle_ws_mute(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    data: Option<&Value>,
    muted: bool,
) -> Result<(), String> {
    let requested_session_id = resolve_ws_session_uuid(connection_id, data)?;
    let (call, session_id) =
        resolve_call_for_ws_connection(state, user_id, requested_session_id).await?;

    if state
        .call_state_manager
        .get_participant(call.call_id, user_id)
        .await
        .is_none()
    {
        // Recover from transient reconnect races where mute/unmute arrives before join/reconnect
        // has re-associated the user participant state.
        state
            .call_state_manager
            .add_participant(
                call.call_id,
                Participant {
                    user_id,
                    session_id,
                    joined_at: Utc::now().timestamp_millis(),
                    muted: true,
                    screen_sharing: false,
                    hand_raised: false,
                },
            )
            .await;

        broadcast_call_event(
            state,
            "custom_com.mattermost.calls_user_joined",
            &call.channel_id,
            serde_json::json!({
                "user_id": encode_mm_id(user_id),
                "session_id": session_id.to_string(),
                "muted": true,
                "raised_hand": 0,
            }),
            None,
        )
        .await;
    }

    state
        .call_state_manager
        .set_muted(call.call_id, user_id, muted)
        .await;
    broadcast_call_event(
        state,
        if muted {
            "custom_com.mattermost.calls_user_muted"
        } else {
            "custom_com.mattermost.calls_user_unmuted"
        },
        &call.channel_id,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": session_id.to_string(),
            "muted": muted,
        }),
        None,
    )
    .await;

    Ok(())
}

async fn handle_ws_raise_hand(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    raised: bool,
) -> Result<(), String> {
    let requested_session_id = resolve_ws_session_uuid(connection_id, None)?;
    let (call, session_id) =
        resolve_call_for_ws_connection(state, user_id, requested_session_id).await?;

    state
        .call_state_manager
        .set_hand_raised(call.call_id, user_id, raised)
        .await;
    broadcast_call_event(
        state,
        if raised {
            "custom_com.mattermost.calls_user_raise_hand"
        } else {
            "custom_com.mattermost.calls_user_unraise_hand"
        },
        &call.channel_id,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": session_id.to_string(),
            "raised_hand": if raised { Utc::now().timestamp_millis() } else { 0 },
        }),
        None,
    )
    .await;

    Ok(())
}

async fn handle_ws_reaction(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    data: Option<&Value>,
) -> Result<(), String> {
    let requested_session_id = resolve_ws_session_uuid(connection_id, data)?;
    let (call, session_id) =
        resolve_call_for_ws_connection(state, user_id, requested_session_id).await?;
    let data = data.ok_or_else(|| "Missing reaction payload".to_string())?;
    let emoji = data
        .get("data")
        .and_then(|v| v.as_str())
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .or_else(|| data.get("data").cloned())
        .unwrap_or_else(|| serde_json::json!({}));
    let reaction = emoji
        .get("literal")
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            emoji
                .get("name")
                .and_then(|value| value.as_str())
                .map(|name| format!(":{name}:"))
        })
        .unwrap_or_default();
    let timestamp = Utc::now().timestamp_millis();

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_user_reacted",
        &call.channel_id,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": session_id.to_string(),
            "reaction": reaction,
            "timestamp": timestamp,
            "emoji": emoji,
        }),
        None,
    )
    .await;

    Ok(())
}

async fn find_call_for_session(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
) -> Option<CallState> {
    let calls = state.call_state_manager.get_all_calls().await;
    calls.into_iter().find(|call| {
        call.participants
            .get(&user_id)
            .map(|p| p.session_id == session_id)
            .unwrap_or(false)
    })
}

async fn resolve_call_for_ws_connection(
    state: &AppState,
    user_id: Uuid,
    requested_session_id: Uuid,
) -> Result<(CallState, Uuid), String> {
    if let Some(call) = find_call_for_session(state, user_id, requested_session_id).await {
        return Ok((call, requested_session_id));
    }

    let user_calls: Vec<(CallState, Uuid)> = state
        .call_state_manager
        .get_all_calls()
        .await
        .into_iter()
        .filter_map(|call| {
            let participant_session_id = call.participants.get(&user_id).map(|p| p.session_id);
            participant_session_id.map(|session_id| (call, session_id))
        })
        .collect();

    if user_calls.len() == 1 {
        let (call, participant_session_id) =
            user_calls.into_iter().next().expect("len checked above");
        warn!(
            user_id = %user_id,
            requested_session_id = %requested_session_id,
            participant_session_id = %participant_session_id,
            call_id = %call.call_id,
            "calls.ws session mismatch recovered using existing participant session"
        );
        Ok((call, participant_session_id))
    } else if user_calls.is_empty() {
        let member_calls = find_member_calls_for_user(state, user_id).await?;
        if member_calls.len() == 1 {
            let call = member_calls.into_iter().next().expect("len checked above");
            warn!(
                user_id = %user_id,
                requested_session_id = %requested_session_id,
                call_id = %call.call_id,
                "calls.ws session lookup recovered using channel membership fallback"
            );
            Ok((call, requested_session_id))
        } else if member_calls.is_empty() {
            Err("No active call found for connection".to_string())
        } else {
            Err("Multiple active calls found for user session resolution".to_string())
        }
    } else {
        Err("Multiple active calls found for user session resolution".to_string())
    }
}

async fn find_member_calls_for_user(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<CallState>, String> {
    let calls = state.call_state_manager.get_all_calls().await;
    let mut member_calls = Vec::new();

    for call in calls {
        let member: Option<(Uuid,)> = sqlx::query_as(
            "SELECT user_id FROM channel_members WHERE channel_id = $1 AND user_id = $2",
        )
        .bind(call.channel_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| format!("Database error while resolving call membership: {e}"))?;

        if member.is_some() {
            member_calls.push(call);
        }
    }

    Ok(member_calls)
}

fn resolve_ws_session_uuid(connection_id: &str, data: Option<&Value>) -> Result<Uuid, String> {
    let default_session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;

    let Some(data) = data else {
        return Ok(default_session_id);
    };

    let original_session_id = data
        .get("originalConnID")
        .or_else(|| data.get("original_conn_id"))
        .or_else(|| data.get("originalConnId"))
        .and_then(|value| value.as_str())
        .and_then(|raw| Uuid::parse_str(raw).ok());

    Ok(original_session_id.unwrap_or(default_session_id))
}

fn parse_ws_sdp_payload(data: Option<&Value>) -> Result<String, String> {
    let data = data.ok_or_else(|| "missing payload".to_string())?;
    let data_field = data
        .get("data")
        .ok_or_else(|| "missing payload.data".to_string())?;

    // Try parsing as string first (uncompressed JSON)
    if let Some(text) = data_field.as_str() {
        let parsed = serde_json::from_str::<Value>(text).map_err(|e| e.to_string())?;
        let sdp = parsed
            .get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing sdp".to_string())?;
        return Ok(sdp.to_string());
    }

    // Parse binary data (compressed)
    let bytes = parse_ws_binary_data(data_field)?;

    // Try as uncompressed UTF-8 first
    if let Ok(text) = String::from_utf8(bytes.clone()) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
            if let Some(sdp) = parsed.get("sdp").and_then(|v| v.as_str()) {
                return Ok(sdp.to_string());
            }
        }
    }

    // Try zlib decompression (mobile clients send compressed SDP)
    let mut decoder = ZlibDecoder::new(bytes.as_slice());
    let mut decoded = String::new();
    match decoder.read_to_string(&mut decoded) {
        Ok(_) => {
            let parsed = serde_json::from_str::<Value>(&decoded).map_err(|e| e.to_string())?;
            let sdp = parsed
                .get("sdp")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing sdp in decompressed data".to_string())?;
            Ok(sdp.to_string())
        }
        Err(e) => Err(format!(
            "zlib decode failed: {e}. Data may not be compressed."
        )),
    }
}

fn parse_ws_ice_payload(
    data: Option<&Value>,
) -> Result<(String, Option<String>, Option<u16>), String> {
    let data = data.ok_or_else(|| "missing payload".to_string())?;
    let raw = data
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing payload.data".to_string())?;
    let parsed = serde_json::from_str::<Value>(raw).map_err(|e| e.to_string())?;

    let candidate = parsed
        .get("candidate")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing candidate".to_string())?
        .to_string();
    let sdp_mid = parsed
        .get("sdpMid")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            parsed
                .get("sdp_mid")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
        });
    let sdp_mline_index = parsed
        .get("sdpMLineIndex")
        .and_then(|v| v.as_u64())
        .or_else(|| parsed.get("sdp_mline_index").and_then(|v| v.as_u64()))
        .and_then(|v| u16::try_from(v).ok());

    Ok((candidate, sdp_mid, sdp_mline_index))
}

fn parse_ws_binary_data(value: &Value) -> Result<Vec<u8>, String> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_u64()
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or_else(|| "binary payload contains non-byte value".to_string())
            })
            .collect(),
        Value::Object(map) if map.get("type").and_then(|v| v.as_str()) == Some("Buffer") => map
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "buffer payload missing data array".to_string())?
            .iter()
            .map(|item| {
                item.as_u64()
                    .and_then(|v| u8::try_from(v).ok())
                    .ok_or_else(|| "buffer payload contains non-byte value".to_string())
            })
            .collect(),
        _ => Err("unsupported binary payload shape".to_string()),
    }
}

async fn send_ws_plugin_event(state: &AppState, user_id: Uuid, event: &str, data: Value) {
    let envelope = WsEnvelope {
        msg_type: "event".to_string(),
        event: event.to_string(),
        seq: None,
        channel_id: None,
        data,
        broadcast: Some(WsBroadcast {
            channel_id: None,
            team_id: None,
            user_id: Some(user_id),
            exclude_user_id: None,
        }),
    };
    state.ws_hub.broadcast(envelope).await;
}

async fn send_ws_plugin_error(state: &AppState, user_id: Uuid, connection_id: &str, message: &str) {
    send_ws_plugin_event(
        state,
        user_id,
        "custom_com.mattermost.calls_error",
        serde_json::json!({
            "connID": connection_id,
            "conn_id": connection_id,
            "error": message,
        }),
    )
    .await;
}

async fn send_ws_plugin_signal(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    signal: Value,
) {
    send_ws_plugin_event(
        state,
        user_id,
        "custom_com.mattermost.calls_signal",
        serde_json::json!({
            "connID": connection_id,
            "conn_id": connection_id,
            "data": signal.to_string(),
            "signal": signal,
        }),
    )
    .await;
}

fn parse_join_channel_id(data: &Value) -> Result<Uuid, String> {
    let raw = data
        .get("channelID")
        .or_else(|| data.get("channel_id"))
        .or_else(|| data.get("channelId"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing channel ID in join payload".to_string())?;

    parse_mm_or_uuid(raw).ok_or_else(|| format!("Invalid channel ID: {raw}"))
}

/// Check if user has permission to access channel
async fn check_channel_permission(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
) -> ApiResult<()> {
    // Check if user is channel member
    let member: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM channel_members WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    if member.is_none() {
        return Err(AppError::Forbidden(
            "You are not a member of this channel".to_string(),
        ));
    }

    Ok(())
}

async fn is_dm_or_gm_channel(state: &AppState, channel_id: Uuid) -> ApiResult<bool> {
    let channel_type: Option<String> =
        sqlx::query_scalar("SELECT type::text FROM channels WHERE id = $1")
            .bind(channel_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

    let Some(channel_type) = channel_type else {
        return Ok(false);
    };

    let normalized = channel_type.trim().to_ascii_lowercase();
    Ok(matches!(
        normalized.as_str(),
        "direct" | "group" | "d" | "g"
    ))
}

/// Broadcast a call-related WebSocket event
async fn broadcast_call_event(
    state: &AppState,
    event_name: &str,
    channel_id: &Uuid,
    mut data: Value,
    exclude_user_id: Option<Uuid>,
) {
    if let Some(obj) = data.as_object_mut() {
        obj.entry("channelID".to_string())
            .or_insert_with(|| Value::String(encode_mm_id(*channel_id)));
        obj.entry("channel_id".to_string())
            .or_insert_with(|| Value::String(encode_mm_id(*channel_id)));
        obj.entry("channel_id_raw".to_string())
            .or_insert_with(|| Value::String(channel_id.to_string()));
    }

    debug!(
        event = event_name,
        channel_id = %channel_id,
        exclude_user_id = ?exclude_user_id,
        "calls.broadcast_call_event"
    );
    let envelope = WsEnvelope {
        msg_type: "event".to_string(),
        event: event_name.to_string(),
        seq: None,
        channel_id: Some(*channel_id),
        data,
        broadcast: Some(crate::realtime::WsBroadcast {
            channel_id: Some(*channel_id),
            team_id: None,
            user_id: None,
            exclude_user_id,
        }),
    };

    state.ws_hub.broadcast(envelope).await;
}

async fn broadcast_ringing_event(
    state: &AppState,
    channel_id: Uuid,
    call_id: Uuid,
    sender_id: Uuid,
    exclude_user_id: Option<Uuid>,
) {
    // Fetch sender info for better mobile client support
    let sender_info: Option<(String, String)> = sqlx::query_as(
        "SELECT username, COALESCE(display_name, '') as display_name FROM users WHERE id = $1",
    )
    .bind(sender_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let (username, display_name) =
        sender_info.unwrap_or_else(|| (encode_mm_id(sender_id), String::new()));

    info!(
        call_id = %call_id,
        channel_id = %channel_id,
        sender_id = %sender_id,
        exclude_user_id = ?exclude_user_id,
        "calls.broadcast_ringing_event STARTED - will send push notifications"
    );

    // Broadcast WebSocket event
    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_ringing",
        &channel_id,
        serde_json::json!({
            "call_id": encode_mm_id(call_id),
            "call_id_raw": call_id.to_string(),
            "sender_id": encode_mm_id(sender_id),
            "sender_id_raw": sender_id.to_string(),
            "username": username,
            "display_name": display_name,
        }),
        exclude_user_id,
    )
    .await;

    // Also send push notifications to offline/mobile users
    // Get channel members to notify
    let members: Vec<(Uuid,)> =
        sqlx::query_as("SELECT user_id FROM channel_members WHERE channel_id = $1")
            .bind(channel_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();

    info!(
        member_count = members.len(),
        "Found channel members for push notification"
    );

    let caller_name = if !display_name.is_empty() {
        display_name.clone()
    } else {
        username.clone()
    };

    for (user_id,) in members {
        // Skip the sender
        if Some(user_id) == exclude_user_id {
            info!(user_id = %user_id, "Skipping sender for push notification");
            continue;
        }

        // Skip users who have dismissed this notification
        if state
            .call_state_manager
            .is_notification_dismissed(call_id, user_id)
            .await
        {
            info!(user_id = %user_id, "Skipping user who dismissed notification");
            continue;
        }

        info!(user_id = %user_id, caller_name = %caller_name, "Sending push notification to user");

        // Send push notification asynchronously (don't block)
        let state_clone = state.clone();
        let caller_name_clone = caller_name.clone();
        tokio::spawn(async move {
            match crate::services::push_notifications::send_call_ringing_notification(
                &state_clone,
                user_id,
                channel_id,
                call_id,
                caller_name_clone,
            )
            .await
            {
                Ok(count) if count > 0 => {
                    info!(
                        user_id = %user_id,
                        count = count,
                        "Sent push notification for incoming call"
                    );
                }
                Ok(_) => {
                    // No devices to notify
                }
                Err(e) => {
                    debug!(
                        user_id = %user_id,
                        error = %e,
                        "Failed to send push notification for call"
                    );
                }
            }
        });
    }
}

async fn broadcast_call_state_event(
    state: &AppState,
    channel_id: Uuid,
    exclude_user_id: Option<Uuid>,
) {
    let Some(call) = state
        .call_state_manager
        .get_call_by_channel(&channel_id)
        .await
    else {
        return;
    };

    let call_state =
        match build_call_state_response(state, &call, encode_mm_id(channel_id), channel_id).await {
            Ok(state_payload) => state_payload,
            Err(err) => {
                warn!(
                    call_id = %call.call_id,
                    channel_id = %channel_id,
                    error = %err,
                    "calls.call_state failed to build call state payload"
                );
                return;
            }
        };

    let call_json = match serde_json::to_string(&call_state) {
        Ok(payload) => payload,
        Err(err) => {
            warn!(
                call_id = %call.call_id,
                channel_id = %channel_id,
                error = %err,
                "calls.call_state failed to serialize call state payload"
            );
            return;
        }
    };

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_call_state",
        &channel_id,
        serde_json::json!({
            "call": call_json,
            "call_id": encode_mm_id(call.call_id),
            "callID": encode_mm_id(call.call_id),
        }),
        exclude_user_id,
    )
    .await;
}

fn spawn_signaling_forwarder(
    state: &AppState,
    channel_id: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    mut rx: mpsc::UnboundedReceiver<SignalingMessage>,
) {
    let state = state.clone();
    tokio::spawn(async move {
        send_signaling_event(
            &state,
            channel_id,
            user_id,
            session_id,
            SignalingMessage::ConnectionState {
                state: "ready".to_string(),
            },
        )
        .await;

        while let Some(signal) = rx.recv().await {
            send_signaling_event(&state, channel_id, user_id, session_id, signal).await;
        }
    });
}

fn schedule_unanswered_call_timeout(state: &AppState, call_id: Uuid, channel_id: Uuid) {
    let state = state.clone();
    tokio::spawn(async move {
        info!(
            call_id = %call_id,
            channel_id = %channel_id,
            timeout_secs = UNANSWERED_CALL_TIMEOUT_SECS,
            "calls.timeout scheduled unanswered-call timeout"
        );
        sleep(Duration::from_secs(UNANSWERED_CALL_TIMEOUT_SECS)).await;
        end_call_if_still_unanswered(&state, call_id).await;
    });
}

fn schedule_empty_call_timeout(state: &AppState, call_id: Uuid, channel_id: Uuid) {
    let state = state.clone();
    tokio::spawn(async move {
        info!(
            call_id = %call_id,
            channel_id = %channel_id,
            timeout_secs = EMPTY_CALL_TIMEOUT_SECS,
            "calls.timeout scheduled empty-call timeout"
        );
        sleep(Duration::from_secs(EMPTY_CALL_TIMEOUT_SECS)).await;
        end_call_if_still_empty(&state, call_id).await;
    });
}

async fn end_call_if_still_unanswered(state: &AppState, call_id: Uuid) {
    let Some(call) = state.call_state_manager.get_call(call_id).await else {
        return;
    };

    let participant_count = call.participants.len();
    if participant_count > 1 {
        debug!(
            call_id = %call_id,
            participant_count = participant_count,
            "calls.timeout unanswered-call timeout skipped"
        );
        return;
    }

    end_call(
        state,
        call.call_id,
        call.channel_id,
        "unanswered_timeout",
        participant_count,
    )
    .await;
}

async fn end_call_if_still_empty(state: &AppState, call_id: Uuid) {
    let Some(call) = state.call_state_manager.get_call(call_id).await else {
        return;
    };

    let participant_count = call.participants.len();
    if participant_count > 1 {
        debug!(
            call_id = %call_id,
            participant_count = participant_count,
            "calls.timeout no-remote-participant timeout skipped"
        );
        return;
    }

    end_call(
        state,
        call.call_id,
        call.channel_id,
        "no_remote_participant_timeout",
        participant_count,
    )
    .await;
}

async fn end_call(
    state: &AppState,
    call_id: Uuid,
    channel_id: Uuid,
    reason: &'static str,
    participant_count: usize,
) {
    state.call_state_manager.remove_call(call_id).await;
    state.sfu_manager.remove_sfu(call_id).await;

    let encoded_channel_id = encode_mm_id(channel_id);
    let encoded_call_id = encode_mm_id(call_id);
    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_call_end",
        &channel_id,
        serde_json::json!({
            "id": encoded_call_id,
            "channelID": encoded_channel_id,
            "call_id": encoded_call_id,
            "channel_id": encoded_channel_id,
        }),
        None,
    )
    .await;

    info!(
        call_id = %call_id,
        channel_id = %channel_id,
        reason = reason,
        participant_count = participant_count,
        "calls.call ended"
    );
}

async fn send_signaling_event(
    state: &AppState,
    channel_id: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    signal: SignalingMessage,
) {
    let signal_kind = match &signal {
        SignalingMessage::Offer { .. } => "offer",
        SignalingMessage::Answer { .. } => "answer",
        SignalingMessage::IceCandidate { .. } => "ice-candidate",
        SignalingMessage::IceConnectionState { .. } => "ice-state",
        SignalingMessage::ConnectionState { .. } => "connection-state",
        SignalingMessage::Error { .. } => "error",
    };
    debug!(
        channel_id = %channel_id,
        user_id = %user_id,
        session_id = %session_id,
        signal_kind = signal_kind,
        "calls.send_signaling_event"
    );
    let signal_payload = serde_json::to_value(signal).unwrap_or_else(|_| {
        serde_json::json!({
            "type": "error",
            "message": "failed to serialize signaling payload"
        })
    });

    let envelope = WsEnvelope {
        msg_type: "event".to_string(),
        event: CALLS_SIGNAL_EVENT.to_string(),
        seq: None,
        channel_id: Some(channel_id),
        data: serde_json::json!({
            "connID": session_id.to_string(),
            "conn_id": session_id.to_string(),
            "data": signal_payload.to_string(),
            "channel_id": encode_mm_id(channel_id),
            "channel_id_raw": channel_id.to_string(),
            "user_id": encode_mm_id(user_id),
            "user_id_raw": user_id.to_string(),
            "session_id": session_id.to_string(),
            "session_id_raw": session_id.to_string(),
            "signal": signal_payload,
        }),
        broadcast: Some(WsBroadcast {
            channel_id: None,
            team_id: None,
            user_id: Some(user_id),
            exclude_user_id: None,
        }),
    };

    state.ws_hub.broadcast(envelope).await;
}

/// Start a background task to listen for voice events from the SFU and broadcast them via WebSockets
pub async fn start_voice_event_listener(
    state: AppState,
    mut rx: mpsc::UnboundedReceiver<VoiceEvent>,
) {
    info!("Starting Calls Voice Event Listener");
    while let Some(event) = rx.recv().await {
        match event {
            VoiceEvent::VoiceOn {
                call_id,
                session_id,
            } => {
                let Some(call) = state.call_state_manager.get_call(call_id).await else {
                    continue;
                };
                broadcast_call_event(
                    &state,
                    "custom_com.mattermost.calls_user_voice_on",
                    &call.channel_id,
                    serde_json::json!({
                        "session_id": session_id.to_string(),
                    }),
                    None,
                )
                .await;
            }
            VoiceEvent::VoiceOff {
                call_id,
                session_id,
            } => {
                let Some(call) = state.call_state_manager.get_call(call_id).await else {
                    continue;
                };
                broadcast_call_event(
                    &state,
                    "custom_com.mattermost.calls_user_voice_off",
                    &call.channel_id,
                    serde_json::json!({
                        "session_id": session_id.to_string(),
                    }),
                    None,
                )
                .await;
            }
        }
    }
    warn!("Calls Voice Event Listener stopped");
}
