//! Mattermost Calls Plugin API
//!
//! Implements the com.mattermost.calls plugin interface for Mattermost Mobile compatibility.
//! Routes are mounted under /plugins/com.mattermost.calls/

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::api::v4::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use crate::realtime::{WsBroadcast, WsEnvelope};

pub mod commands;
pub mod sfu;
pub mod state;
mod turn;

use flate2::read::ZlibDecoder;
use sfu::signaling::SignalingMessage;
use state::{CallState, Participant};
use turn::{TurnCredentialGenerator, TurnServerConfig};
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

const CALLS_SIGNAL_EVENT: &str = "custom_com.mattermost.calls_signal";

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
            get(get_call_state),
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
        .route(
            "/plugins/com.mattermost.calls/turn-credentials",
            get(get_turn_credentials),
        )
        // Slash commands
        .merge(commands::router())
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
}

#[derive(Debug, Serialize)]
struct IceServer {
    urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credential: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    participants: Vec<String>,
    participants_raw: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    screen_sharing_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ReactionRequest {
    emoji: String,
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
/// Returns ICE server configuration with TURN credentials
async fn get_config(
    State(state): State<AppState>,
    _auth: MmAuthUser,
) -> ApiResult<Json<ConfigResponse>> {
    // Build ice servers list
    let mut ice_servers = vec![];

    // Add STUN servers if configured
    for stun_url in &state.config.calls.stun_servers {
        ice_servers.push(IceServer {
            urls: vec![stun_url.clone()],
            username: None,
            credential: None,
            credential_type: None,
        });
    }

    // Add TURN server if enabled
    if state.config.calls.turn_server_enabled {
        // Build basic server info without credentials (mobile will fetch them later via /turn-credentials)
        ice_servers.push(IceServer {
            urls: vec![state.config.calls.turn_server_url.clone()],
            username: None,
            credential: None,
            credential_type: Some("password".to_string()),
        });
    }
    let needs_turn = state.config.calls.turn_server_enabled;

    Ok(Json(ConfigResponse { 
        ice_servers_configs: ice_servers,
        needs_turn_credentials: needs_turn,
    }))
}

/// GET /plugins/com.mattermost.calls/turn-credentials
/// Returns ephemeral TURN credentials
async fn get_turn_credentials(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<Vec<IceServer>>> {
    if !state.config.calls.turn_server_enabled {
        return Err(AppError::BadRequest("TURN server is disabled".to_string()));
    }

    let turn_config = TurnServerConfig {
        enabled: true,
        url: state.config.calls.turn_server_url.clone(),
        username: state.config.calls.turn_server_username.clone(),
        credential: state.config.calls.turn_server_credential.clone(),
    };

    // If static credentials are NOT provided, use the application encryption key as the secret for ephemeral ones
    let generator = if turn_config.username.is_empty() || turn_config.credential.is_empty() {
        TurnCredentialGenerator::with_rest_api(
            state.config.encryption_key.clone(),
            state.config.calls.turn_ttl_minutes
        )
    } else {
        TurnCredentialGenerator::with_static_credentials(turn_config)
    };

    let credentials = generator.generate_credentials(&auth.user_id.to_string());
    
    Ok(Json(vec![IceServer {
        urls: vec![state.config.calls.turn_server_url.clone()],
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
    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Get all active calls
    let active_calls = call_manager.get_all_calls().await;

    // Build response with channels that have active calls
    let mut channels = Vec::new();
    for call in active_calls {
        // Check if user is a member of this channel
        let is_member: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
        )
        .bind(call.channel_id)
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);

        if is_member {
            let participant_count = call_manager.get_participant_count(call.call_id).await;

            channels.push(CallChannelInfo {
                channel_id: encode_mm_id(call.channel_id),
                channel_id_raw: call.channel_id.to_string(),
                call_id: Some(encode_mm_id(call.call_id)),
                call_id_raw: Some(call.call_id.to_string()),
                enabled: true,
                has_call: participant_count > 0,
                participant_count: participant_count as i32,
            });
        }
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
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/start
/// Starts a new call in a channel
async fn start_call(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StartCallResponse>> {
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        "calls.start_call requested"
    );

    // Check channel permissions
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;

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
        }));
    }

    // Create new call
    let call_id = Uuid::new_v4();
    let now = Utc::now().timestamp_millis();

    let call = CallState {
        call_id,
        channel_id: channel_uuid,
        owner_id: auth.user_id,
        started_at: now,
        participants: HashMap::new(),
        screen_sharer: None,
        thread_id: None,
    };

    call_manager.add_call(call.clone()).await;
    debug!(
        call_id = %call_id,
        channel_id = %channel_uuid,
        owner_id = %auth.user_id,
        "calls.start_call call state created"
    );

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
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_call_start",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "call_id": encode_mm_id(call_id),
            "start_at": now,
            "owner_id": encode_mm_id(auth.user_id),
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
            "session_id": encode_mm_id(participant.session_id),
            "muted": true,
            "raised_hand": false,
        }),
        None,
    )
    .await;
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
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/join
/// Join an existing call
async fn join_call(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        "calls.join_call requested"
    );

    // Check channel permissions
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find active call in channel
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

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
            "session_id": encode_mm_id(participant.session_id),
            "muted": true,
            "raised_hand": false,
        }),
        None,
    )
    .await;
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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        "calls.leave_call requested"
    );

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

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

    // If no participants left, end the call
    let participants = call_manager.get_participants(call.call_id).await;
    if participants.is_empty() {
        call_manager.remove_call(call.call_id).await;

        // Remove the SFU for this call
        state.sfu_manager.remove_sfu(call.call_id).await;

        // Broadcast call_end event
        broadcast_call_event(
            &state,
            "custom_com.mattermost.calls_call_end",
            &channel_uuid,
            serde_json::json!({
                "channel_id": channel_id,
                "call_id": encode_mm_id(call.call_id),
            }),
            None,
        )
        .await;
        info!(
            call_id = %call.call_id,
            channel_id = %channel_uuid,
            "calls.leave_call ended call because no participants remain"
        );
    } else {
        info!(
            call_id = %call.call_id,
            channel_id = %channel_uuid,
            remaining_participants = participants.len(),
            "calls.leave_call completed"
        );
    }

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
) -> ApiResult<Json<CallStateResponse>> {
    let normalized_channel_id = channel_id.trim();
    if normalized_channel_id.is_empty()
        || normalized_channel_id.eq_ignore_ascii_case("undefined")
        || normalized_channel_id.eq_ignore_ascii_case("null")
    {
        return Err(AppError::NotFound(
            "No active call in this channel".to_string(),
        ));
    }

    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;
    debug!(
        channel_id = %channel_uuid,
        call_id = %call.call_id,
        owner_id = %call.owner_id,
        "calls.get_call_state found active call"
    );

    let call_participants = call_manager.get_participants(call.call_id).await;
    let participants: Vec<String> = call_participants
        .iter()
        .map(|p| encode_mm_id(p.user_id))
        .collect();
    let participants_raw: Vec<String> = call_participants
        .iter()
        .map(|p| p.user_id.to_string())
        .collect();

    Ok(Json(CallStateResponse {
        id: encode_mm_id(call.call_id),
        id_raw: call.call_id.to_string(),
        channel_id: channel_id.clone(),
        channel_id_raw: channel_uuid.to_string(),
        start_at: call.started_at,
        owner_id: encode_mm_id(call.owner_id),
        owner_id_raw: call.owner_id.to_string(),
        participants,
        participants_raw,
        screen_sharing_id: call.screen_sharer.map(encode_mm_id),
        thread_id: call.thread_id.map(encode_mm_id),
    }))
}

/// POST /plugins/com.mattermost.calls/calls/{channel_id}/react
/// Send a reaction during call
async fn send_reaction(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(payload): Json<ReactionRequest>,
) -> ApiResult<Json<StatusResponse>> {
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Broadcast reaction event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_user_reacted",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "emoji": payload.emoji,
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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

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

    // Update global screen sharer
    if is_sharing {
        call_manager
            .set_screen_sharer(call.call_id, Some(auth.user_id))
            .await;
    } else if call.screen_sharer == Some(auth.user_id) {
        call_manager.set_screen_sharer(call.call_id, None).await;
    }

    // Broadcast event
    let event_name = if is_sharing {
        "custom_com.mattermost.calls_screen_on"
    } else {
        "custom_com.mattermost.calls_screen_off"
    };

    broadcast_call_event(
        &state,
        event_name,
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
        }),
        None,
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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    // Set hand raised
    call_manager
        .set_hand_raised(call.call_id, auth.user_id, true)
        .await;

    // Broadcast raise_hand event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_raise_hand",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "raised": true,
        }),
        None,
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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

    // Set hand lowered
    call_manager
        .set_hand_raised(call.call_id, auth.user_id, false)
        .await;

    // Broadcast lower_hand event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_lower_hand",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "raised": false,
        }),
        None,
    )
    .await;

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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;
    info!(
        user_id = %auth.user_id,
        channel_id = %channel_uuid,
        sdp_len = payload.sdp.len(),
        "calls.offer received"
    );

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

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

    // Ensure this participant is present in the SFU before handling signaling.
    if !sfu.has_participant(participant.session_id).await {
        warn!(
            call_id = %call.call_id,
            user_id = %auth.user_id,
            session_id = %participant.session_id,
            "calls.offer participant missing in SFU; recovering by re-registering"
        );
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
    }

    // Parse the offer SDP
    let offer = RTCSessionDescription::offer(payload.sdp)
        .map_err(|e| AppError::BadRequest(format!("Invalid SDP offer: {}", e)))?;

    // Handle the offer and get answer
    let answer = sfu
        .handle_offer(participant.session_id, offer)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to handle offer: {}", e)))?;
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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

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
    let Some(call) = call_manager.get_call_by_channel(&channel_uuid).await else {
        warn!(
            user_id = %auth.user_id,
            channel_id = %channel_uuid,
            "Ignoring ICE candidate: no active call in this channel"
        );
        return Ok(Json(StatusResponse {
            status: "IGNORED".to_string(),
        }));
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
        "mute" => handle_ws_mute(state, user_id, connection_id, true).await,
        "unmute" => handle_ws_mute(state, user_id, connection_id, false).await,
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
    let conn_uuid = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let data = data.ok_or_else(|| "Missing join payload".to_string())?;
    let channel_uuid = parse_join_channel_id(data)?;

    check_channel_permission(state, user_id, channel_uuid)
        .await
        .map_err(|e| e.to_string())?;

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
            started_at: now,
            participants: HashMap::new(),
            screen_sharer: None,
            thread_id: data
                .get("threadID")
                .and_then(|v| v.as_str())
                .and_then(parse_mm_or_uuid),
        };
        call_manager.add_call(call.clone()).await;
        call
    };

    let mut should_add_participant = true;
    if let Some(existing) = call_manager.get_participant(call.call_id, user_id).await {
        if existing.session_id == conn_uuid {
            should_add_participant = false;
        } else {
            call_manager.remove_participant(call.call_id, user_id).await;
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

    if created_call {
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
                "thread_id": call.thread_id.map(encode_mm_id),
                "call_id": encode_mm_id(call.call_id),
                "channel_id": encode_mm_id(channel_uuid),
            }),
            None,
        )
        .await;
    }

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_user_joined",
        &channel_uuid,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": connection_id,
            "muted": true,
            "raised_hand": 0,
        }),
        None,
    )
    .await;

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
            "sessionID": connection_id,
            "session_id": connection_id,
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
    let session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let sdp = parse_ws_sdp_payload(data).map_err(|e| format!("Invalid SDP payload: {e}"))?;
    let call = find_call_for_session(state, user_id, session_id)
        .await
        .ok_or_else(|| "No active call found for connection".to_string())?;

    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| format!("Failed to get or create SFU: {e}"))?;

    if !sfu.has_participant(session_id).await {
        let _ = sfu
            .add_participant(user_id, session_id)
            .await
            .map_err(|e| format!("Failed to add participant to SFU: {e}"))?;
    }

    let offer = RTCSessionDescription::offer(sdp).map_err(|e| format!("Invalid offer SDP: {e}"))?;
    let answer = sfu
        .handle_offer(session_id, offer)
        .await
        .map_err(|e| format!("Failed to handle offer: {e}"))?;

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
    let session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let (candidate, sdp_mid, sdp_mline_index) =
        parse_ws_ice_payload(data).map_err(|e| format!("Invalid ICE payload: {e}"))?;
    let call = find_call_for_session(state, user_id, session_id)
        .await
        .ok_or_else(|| "No active call found for connection".to_string())?;

    let sfu = state
        .sfu_manager
        .get_or_create_sfu(call.call_id)
        .await
        .map_err(|e| format!("Failed to get or create SFU: {e}"))?;

    if !sfu.has_participant(session_id).await {
        let _ = sfu
            .add_participant(user_id, session_id)
            .await
            .map_err(|e| format!("Failed to add participant to SFU: {e}"))?;
    }

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
    let session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let Some(call) = find_call_for_session(state, user_id, session_id).await else {
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
            "session_id": connection_id,
        }),
        None,
    )
    .await;

    if call_manager.get_participants(call.call_id).await.is_empty() {
        call_manager.remove_call(call.call_id).await;
        state.sfu_manager.remove_sfu(call.call_id).await;
        broadcast_call_event(
            state,
            "custom_com.mattermost.calls_call_end",
            &call.channel_id,
            serde_json::json!({
                "id": encode_mm_id(call.call_id),
                "channelID": encode_mm_id(call.channel_id),
                "call_id": encode_mm_id(call.call_id),
            }),
            None,
        )
        .await;
    }

    Ok(())
}

async fn handle_ws_mute(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    muted: bool,
) -> Result<(), String> {
    let session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let call = find_call_for_session(state, user_id, session_id)
        .await
        .ok_or_else(|| "No active call found for connection".to_string())?;

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
            "session_id": connection_id,
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
    let session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let call = find_call_for_session(state, user_id, session_id)
        .await
        .ok_or_else(|| "No active call found for connection".to_string())?;

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
            "session_id": connection_id,
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
    let session_id = Uuid::parse_str(connection_id)
        .map_err(|_| format!("Invalid connection ID: {connection_id}"))?;
    let call = find_call_for_session(state, user_id, session_id)
        .await
        .ok_or_else(|| "No active call found for connection".to_string())?;
    let data = data.ok_or_else(|| "Missing reaction payload".to_string())?;
    let emoji = data
        .get("data")
        .and_then(|v| v.as_str())
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    broadcast_call_event(
        state,
        "custom_com.mattermost.calls_user_reacted",
        &call.channel_id,
        serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "session_id": connection_id,
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

fn parse_ws_sdp_payload(data: Option<&Value>) -> Result<String, String> {
    let data = data.ok_or_else(|| "missing payload".to_string())?;
    let data_field = data
        .get("data")
        .ok_or_else(|| "missing payload.data".to_string())?;

    if let Some(text) = data_field.as_str() {
        let parsed = serde_json::from_str::<Value>(text).map_err(|e| e.to_string())?;
        let sdp = parsed
            .get("sdp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing sdp".to_string())?;
        return Ok(sdp.to_string());
    }

    let bytes = parse_ws_binary_data(data_field)?;
    if let Ok(text) = String::from_utf8(bytes.clone()) {
        if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
            if let Some(sdp) = parsed.get("sdp").and_then(|v| v.as_str()) {
                return Ok(sdp.to_string());
            }
        }
    }

    let mut decoder = ZlibDecoder::new(bytes.as_slice());
    let mut decoded = String::new();
    decoder
        .read_to_string(&mut decoded)
        .map_err(|e| format!("zlib decode failed: {e}"))?;

    let parsed = serde_json::from_str::<Value>(&decoded).map_err(|e| e.to_string())?;
    let sdp = parsed
        .get("sdp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing sdp".to_string())?;
    Ok(sdp.to_string())
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
            "channel_id": encode_mm_id(channel_id),
            "channel_id_raw": channel_id.to_string(),
            "user_id": encode_mm_id(user_id),
            "user_id_raw": user_id.to_string(),
            "session_id": encode_mm_id(session_id),
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
