use crate::api::AppState;
use axum::Router;
use serde_json::Value;
use tracing::debug;
use uuid::Uuid;

pub mod commands;
pub mod sfu;
pub mod state;
pub mod turn;

pub fn router() -> Router<AppState> {
    Router::new().merge(commands::router())
}

pub async fn start_voice_event_listener(
    _state: AppState,
    mut voice_event_rx: tokio::sync::mpsc::Receiver<sfu::VoiceEvent>,
) {
    while let Some(event) = voice_event_rx.recv().await {
        match event {
            sfu::VoiceEvent::VoiceOn {
                call_id,
                session_id,
            } => {
                debug!(call_id = %call_id, session_id = %session_id, "voice activity on");
            }
            sfu::VoiceEvent::VoiceOff {
                call_id,
                session_id,
            } => {
                debug!(call_id = %call_id, session_id = %session_id, "voice activity off");
            }
        }
    }
}

pub async fn handle_ws_action(
    state: &AppState,
    user_id: Uuid,
    connection_id: &str,
    action: &str,
    _data: Option<&Value>,
) -> bool {
    match action {
        "mute" | "custom_com.mattermost.calls_mute" => {
            apply_participant_mutation(state, user_id, |call_id| async move {
                state
                    .call_state_manager
                    .set_muted(call_id, user_id, true)
                    .await;
            })
            .await;
            true
        }
        "unmute" | "custom_com.mattermost.calls_unmute" => {
            apply_participant_mutation(state, user_id, |call_id| async move {
                state
                    .call_state_manager
                    .set_muted(call_id, user_id, false)
                    .await;
            })
            .await;
            true
        }
        "raise_hand" | "custom_com.mattermost.calls_raise_hand" => {
            apply_participant_mutation(state, user_id, |call_id| async move {
                state
                    .call_state_manager
                    .set_hand_raised(call_id, user_id, true)
                    .await;
            })
            .await;
            true
        }
        "unraise_hand" | "custom_com.mattermost.calls_unraise_hand" => {
            apply_participant_mutation(state, user_id, |call_id| async move {
                state
                    .call_state_manager
                    .set_hand_raised(call_id, user_id, false)
                    .await;
            })
            .await;
            true
        }
        "leave" | "custom_com.mattermost.calls_leave" => {
            handle_ws_connection_closed(state, user_id, connection_id).await;
            true
        }
        _ => false,
    }
}

pub async fn handle_ws_connection_closed(state: &AppState, user_id: Uuid, _connection_id: &str) {
    let calls = state.call_state_manager.get_all_calls().await;

    for call in calls {
        let participant = call.participants.get(&user_id).cloned();
        let Some(participant) = participant else {
            continue;
        };

        state
            .call_state_manager
            .remove_participant(call.call_id, user_id)
            .await;

        if let Some(sfu) = state.sfu_manager.get_sfu(call.call_id).await {
            let _ = sfu.remove_participant(participant.session_id).await;
        }

        if state
            .call_state_manager
            .get_participant_count(call.call_id)
            .await
            == 0
        {
            state.call_state_manager.remove_call(call.call_id).await;
            state.sfu_manager.remove_sfu(call.call_id).await;
        }
    }
}

async fn apply_participant_mutation<F, Fut>(state: &AppState, user_id: Uuid, mutator: F)
where
    F: Fn(Uuid) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let calls = state.call_state_manager.get_all_calls().await;

    for call in calls {
        if call.participants.contains_key(&user_id) {
            mutator(call.call_id).await;
        }
    }
}
