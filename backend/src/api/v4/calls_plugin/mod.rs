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
use tokio::sync::mpsc;
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
    ice_servers: Vec<IceServer>,
}

#[derive(Debug, Serialize)]
struct IceServer {
    urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    credential: Option<String>,
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
    auth: MmAuthUser,
) -> ApiResult<Json<ConfigResponse>> {
    // Build ice servers list
    let mut ice_servers = vec![];

    // Add STUN servers if configured
    for stun_url in &state.config.calls.stun_servers {
        ice_servers.push(IceServer {
            urls: vec![stun_url.clone()],
            username: None,
            credential: None,
        });
    }

    // Add TURN server if enabled
    if state.config.calls.turn_server_enabled {
        // Configure TURN server with static credentials
        let turn_config = TurnServerConfig {
            enabled: true,
            url: state.config.calls.turn_server_url.clone(),
            username: state.config.calls.turn_server_username.clone(),
            credential: state.config.calls.turn_server_credential.clone(),
        };

        let turn_generator = TurnCredentialGenerator::with_static_credentials(turn_config);
        let credentials = turn_generator.generate_credentials(&auth.user_id.to_string());

        // Add TURN server with credentials
        if let Some(turn_url) = turn_generator.get_turn_url() {
            ice_servers.push(IceServer {
                urls: vec![turn_url],
                username: Some(credentials.username.clone()),
                credential: Some(credentials.credential.clone()),
            });
        }
    }

    Ok(Json(ConfigResponse { ice_servers }))
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

    // Check channel permissions
    check_channel_permission(&state, auth.user_id, channel_uuid).await?;

    // Get or initialize call state manager
    let call_manager = state.call_state_manager.as_ref();

    // Check if call already exists
    if let Some(call) = call_manager.get_call_by_channel(&channel_uuid).await {
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

    // Broadcast call_start event
    broadcast_call_event(
        &state,
        "custom_com.mattermost.calls_call_start",
        &channel_uuid,
        serde_json::json!({
            "channel_id": channel_id,
            "user_id": encode_mm_id(auth.user_id),
            "call_id": encode_mm_id(call_id),
            "start_at": now.to_string(),
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
    let channel_uuid = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Get call manager
    let call_manager = state.call_state_manager.as_ref();

    // Find call
    let call = call_manager
        .get_call_by_channel(&channel_uuid)
        .await
        .ok_or_else(|| AppError::NotFound("No active call in this channel".to_string()))?;

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

    // Get SFU for this call
    let sfu = state
        .sfu_manager
        .get_sfu(call.call_id)
        .await
        .ok_or_else(|| AppError::NotFound("SFU not found for this call".to_string()))?;

    // Parse the offer SDP
    let offer = RTCSessionDescription::offer(payload.sdp)
        .map_err(|e| AppError::BadRequest(format!("Invalid SDP offer: {}", e)))?;

    // Handle the offer and get answer
    let answer = sfu
        .handle_offer(participant.session_id, offer)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to handle offer: {}", e)))?;

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

    // Get SFU for this call
    let sfu = state
        .sfu_manager
        .get_sfu(call.call_id)
        .await
        .ok_or_else(|| AppError::NotFound("SFU not found for this call".to_string()))?;

    // Handle the ICE candidate
    sfu.handle_ice_candidate(
        participant.session_id,
        payload.candidate,
        payload.sdp_mid,
        payload.sdp_mline_index,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to handle ICE candidate: {}", e)))?;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
    }))
}

// ============ Helper Functions ============

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
    data: Value,
    exclude_user_id: Option<Uuid>,
) {
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
