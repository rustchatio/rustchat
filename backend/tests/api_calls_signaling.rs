use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use rustchat::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use crate::common::spawn_app;

mod common;

type WsClient =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[tokio::test]
async fn calls_lifecycle_events_are_delivered_over_websocket() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Calls Lifecycle Org").await;
    let (token_a, user_a) =
        register_and_login(&app, org_id, "caller_a", "caller_a@example.com").await;
    let (token_b, user_b) =
        register_and_login(&app, org_id, "caller_b", "caller_b@example.com").await;

    let channel_id = create_team_and_channel_with_members(&app, org_id, &[user_a, user_b]).await;

    let mut ws_a = connect_ws(&app.address, &token_a).await;
    let mut ws_b = connect_ws(&app.address, &token_b).await;

    wait_for_event(&mut ws_a, "hello", Duration::from_secs(5)).await;
    wait_for_event(&mut ws_b, "hello", Duration::from_secs(5)).await;
    subscribe_channel(&mut ws_a, channel_id).await;
    subscribe_channel(&mut ws_b, channel_id).await;
    let _ = wait_for_event(&mut ws_a, "channel_subscribed", Duration::from_secs(5)).await;
    let _ = wait_for_event(&mut ws_b, "channel_subscribed", Duration::from_secs(5)).await;

    let start = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/start",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("start call request failed");
    assert_eq!(start.status(), StatusCode::OK);

    let call_start = wait_for_event(
        &mut ws_b,
        "custom_com.mattermost.calls_call_start",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(call_start["channel_id"], channel_id.to_string());
    let _ = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_joined",
        Duration::from_secs(5),
    )
    .await;

    let join = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/join",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("join call request failed");
    assert_eq!(join.status(), StatusCode::OK);

    let user_joined = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_joined",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(user_joined["channel_id"], channel_id.to_string());
    assert_eq!(user_joined["user_id"], encode_mm_id(user_b));

    let leave_b = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/leave",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("leave call request failed");
    assert_eq!(leave_b.status(), StatusCode::OK);

    let user_left = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_left",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(user_left["channel_id"], channel_id.to_string());
    assert_eq!(user_left["user_id"], encode_mm_id(user_b));

    let leave_a = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/leave",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("leave call request failed");
    assert_eq!(leave_a.status(), StatusCode::OK);

    let call_end = wait_for_event(
        &mut ws_b,
        "custom_com.mattermost.calls_call_end",
        Duration::from_secs(15),
    )
    .await;
    assert!(
        call_end["channel_id"] == channel_id.to_string()
            || call_end["channel_id"] == encode_mm_id(channel_id)
    );
}

#[tokio::test]
async fn calls_start_in_direct_channel_auto_rings_other_participants() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Calls Direct Ringing Org").await;
    let (token_a, user_a) =
        register_and_login(&app, org_id, "direct_ring_a", "direct_ring_a@example.com").await;
    let (token_b, user_b) =
        register_and_login(&app, org_id, "direct_ring_b", "direct_ring_b@example.com").await;
    let channel_id = create_team_and_channel_with_members_of_type(
        &app,
        org_id,
        &[user_a, user_b],
        "direct",
    )
    .await;

    let mut ws_a = connect_ws(&app.address, &token_a).await;
    let mut ws_b = connect_ws(&app.address, &token_b).await;
    wait_for_event(&mut ws_a, "hello", Duration::from_secs(5)).await;
    wait_for_event(&mut ws_b, "hello", Duration::from_secs(5)).await;
    subscribe_channel(&mut ws_a, channel_id).await;
    subscribe_channel(&mut ws_b, channel_id).await;
    let _ = wait_for_event(&mut ws_a, "channel_subscribed", Duration::from_secs(5)).await;
    let _ = wait_for_event(&mut ws_b, "channel_subscribed", Duration::from_secs(5)).await;

    let start = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/start",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("start call request failed");
    assert_eq!(start.status(), StatusCode::OK);
    let start_body: serde_json::Value = start.json().await.expect("start call JSON");

    let ringing_event = wait_for_event(
        &mut ws_b,
        "custom_com.mattermost.calls_ringing",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(ringing_event["sender_id"], encode_mm_id(user_a));
    assert_eq!(ringing_event["call_id"], start_body["id"]);
}

#[tokio::test]
async fn offer_generates_server_signaling_event_over_websocket() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Calls Signaling Org").await;
    let (token, user_id) =
        register_and_login(&app, org_id, "signal_user", "signal_user@example.com").await;
    let channel_id = create_team_and_channel_with_members(&app, org_id, &[user_id]).await;

    let mut ws = connect_ws(&app.address, &token).await;
    wait_for_event(&mut ws, "hello", Duration::from_secs(5)).await;

    let start = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/start",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("start call request failed");
    assert_eq!(start.status(), StatusCode::OK);

    let signal_event = wait_for_event(
        &mut ws,
        "custom_com.mattermost.calls_signal",
        Duration::from_secs(8),
    )
    .await;
    assert_eq!(signal_event["channel_id_raw"], channel_id.to_string());
    assert_eq!(signal_event["signal"]["type"], "connection-state");
    assert_eq!(signal_event["signal"]["state"], "ready");
}

#[tokio::test]
async fn calls_mobile_channel_state_and_end_route_are_compatible() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Calls Mobile REST Org").await;
    let (token_a, user_a) =
        register_and_login(&app, org_id, "mobile_rest_a", "mobile_rest_a@example.com").await;
    let (token_b, user_b) =
        register_and_login(&app, org_id, "mobile_rest_b", "mobile_rest_b@example.com").await;
    let channel_id = create_team_and_channel_with_members(&app, org_id, &[user_a, user_b]).await;

    let start = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/start",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("start call request failed");
    assert_eq!(start.status(), StatusCode::OK);

    let join = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/join",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("join call request failed");
    assert_eq!(join.status(), StatusCode::OK);

    let channels = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls/channels?mobilev2=true",
            app.address
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("get channels request failed");
    assert_eq!(channels.status(), StatusCode::OK);
    let channels_body: serde_json::Value = channels.json().await.expect("channels JSON");
    let channel_state = channels_body
        .as_array()
        .expect("channels should be array")
        .iter()
        .find(|entry| entry["channel_id"] == encode_mm_id(channel_id))
        .expect("channel state should exist");
    assert_eq!(channel_state["enabled"], true);
    assert!(channel_state["call"].is_object());
    assert!(channel_state["call"]["sessions"].is_object());

    let get_channel_state = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls/{}?mobilev2=true",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("get channel state request failed");
    assert_eq!(get_channel_state.status(), StatusCode::OK);
    let get_channel_state_body: serde_json::Value =
        get_channel_state.json().await.expect("channel state JSON");
    assert_eq!(get_channel_state_body["enabled"], true);
    assert!(get_channel_state_body["call"].is_object());

    let config = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls/config",
            app.address
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("calls config request failed");
    assert_eq!(config.status(), StatusCode::OK);
    let config_body: serde_json::Value = config.json().await.expect("config JSON");
    assert_eq!(config_body["EnableRinging"], true);
    assert_eq!(config_body["HostControlsAllowed"], true);
    assert_eq!(config_body["MaxCallParticipants"], 0);
    assert_eq!(config_body["AllowScreenSharing"], true);
    assert_eq!(config_body["EnableSimulcast"], false);
    assert_eq!(config_body["EnableAV1"], false);
    assert_eq!(config_body["MaxRecordingDuration"], 60);
    assert_eq!(config_body["TranscribeAPI"], "whisper.cpp");
    assert_eq!(config_body["sku_short_name"], "starter");
    assert_eq!(config_body["EnableDCSignaling"], false);
    assert_eq!(config_body["EnableTranscriptions"], false);
    assert_eq!(config_body["EnableLiveCaptions"], false);

    let recording_start = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/recording/start",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("recording start request failed");
    assert_ne!(recording_start.status(), StatusCode::NOT_FOUND);

    let disable_calls = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/{}",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({ "enabled": false }))
        .send()
        .await
        .expect("disable calls request failed");
    assert_eq!(disable_calls.status(), StatusCode::OK);
    let disable_body: serde_json::Value = disable_calls.json().await.expect("disable JSON");
    assert_eq!(disable_body["enabled"], false);

    let end_forbidden = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/end",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("end call (non-host) request failed");
    assert_eq!(end_forbidden.status(), StatusCode::FORBIDDEN);

    let end_ok = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/end",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("end call request failed");
    assert_eq!(end_ok.status(), StatusCode::OK);
    let end_body: serde_json::Value = end_ok.json().await.expect("end JSON");
    assert_eq!(end_body["status"], "OK");

    let idle_state = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls/{}?mobilev2=true",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("idle channel state request failed");
    assert_eq!(idle_state.status(), StatusCode::OK);
    let idle_body: serde_json::Value = idle_state.json().await.expect("idle JSON");
    assert_eq!(idle_body["call"], serde_json::Value::Null);
}

#[tokio::test]
async fn calls_mobile_event_names_and_payloads_are_compatible() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Calls Mobile WS Org").await;
    let (token_a, user_a) =
        register_and_login(&app, org_id, "mobile_ws_a", "mobile_ws_a@example.com").await;
    let (token_b, user_b) =
        register_and_login(&app, org_id, "mobile_ws_b", "mobile_ws_b@example.com").await;
    let channel_id = create_team_and_channel_with_members(&app, org_id, &[user_a, user_b]).await;

    let mut ws_a = connect_ws(&app.address, &token_a).await;
    let mut ws_b = connect_ws(&app.address, &token_b).await;
    wait_for_event(&mut ws_a, "hello", Duration::from_secs(5)).await;
    wait_for_event(&mut ws_b, "hello", Duration::from_secs(5)).await;
    subscribe_channel(&mut ws_a, channel_id).await;
    subscribe_channel(&mut ws_b, channel_id).await;
    let _ = wait_for_event(&mut ws_a, "channel_subscribed", Duration::from_secs(5)).await;
    let _ = wait_for_event(&mut ws_b, "channel_subscribed", Duration::from_secs(5)).await;

    let start = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/start",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("start call request failed");
    assert_eq!(start.status(), StatusCode::OK);
    let _ = wait_for_event(
        &mut ws_b,
        "custom_com.mattermost.calls_call_start",
        Duration::from_secs(5),
    )
    .await;
    let _ = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_joined",
        Duration::from_secs(5),
    )
    .await;

    let join = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/join",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("join call request failed");
    assert_eq!(join.status(), StatusCode::OK);
    let _ = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_joined",
        Duration::from_secs(5),
    )
    .await;
    let call_state_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_call_state",
        Duration::from_secs(5),
    )
    .await;
    let call_state_payload = call_state_event["call"]
        .as_str()
        .expect("call state payload should include call JSON");
    let call_state_json: serde_json::Value =
        serde_json::from_str(call_state_payload).expect("call state payload should parse as JSON");
    assert_eq!(call_state_json["channel_id"], encode_mm_id(channel_id));

    let channel_state = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls/{}?mobilev2=true",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("get channel state request failed");
    assert_eq!(channel_state.status(), StatusCode::OK);
    let channel_state_body: serde_json::Value = channel_state.json().await.expect("state JSON");
    let b_session_id = channel_state_body["call"]["sessions"]
        .as_object()
        .expect("sessions should be object")
        .values()
        .find(|session| session["user_id"] == encode_mm_id(user_b))
        .and_then(|session| session["session_id"].as_str())
        .expect("target session id should exist")
        .to_string();

    let raise = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/raise-hand",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("raise hand request failed");
    assert_eq!(raise.status(), StatusCode::OK);
    let raised_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_raise_hand",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(
        raised_event["session_id"].as_str(),
        Some(b_session_id.as_str())
    );
    assert!(raised_event["raised_hand"].as_i64().unwrap_or_default() > 0);

    let lower = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/lower-hand",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("lower hand request failed");
    assert_eq!(lower.status(), StatusCode::OK);
    let lowered_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_unraise_hand",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(
        lowered_event["session_id"].as_str(),
        Some(b_session_id.as_str())
    );
    assert_eq!(lowered_event["raised_hand"], 0);

    let screen_on = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/screen-share",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("screen share request failed");
    assert_eq!(screen_on.status(), StatusCode::OK);
    let screen_on_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_screen_on",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(
        screen_on_event["session_id"].as_str(),
        Some(b_session_id.as_str())
    );

    let host_screen_off = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/host/screen-off",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({ "session_id": b_session_id.clone() }))
        .send()
        .await
        .expect("host screen-off request failed");
    assert_eq!(host_screen_off.status(), StatusCode::OK);
    let screen_off_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_screen_off",
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(
        screen_off_event["session_id"].as_str(),
        Some(b_session_id.as_str())
    );

    let host_make = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/host/make",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({ "new_host_id": encode_mm_id(user_b) }))
        .send()
        .await
        .expect("host make request failed");
    assert_eq!(host_make.status(), StatusCode::OK);
    let host_changed_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_call_host_changed",
        Duration::from_secs(5),
    )
    .await;
    let expected_host = encode_mm_id(user_b);
    assert_eq!(
        host_changed_event["hostID"].as_str(),
        Some(expected_host.as_str())
    );
    let _ = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_call_state",
        Duration::from_secs(5),
    )
    .await;

    let dismiss = app
        .api_client
        .post(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}/dismiss-notification",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .send()
        .await
        .expect("dismiss notification request failed");
    assert_eq!(dismiss.status(), StatusCode::OK);
    let dismissed_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_user_dismissed_notification",
        Duration::from_secs(5),
    )
    .await;
    let expected_user = encode_mm_id(user_b);
    assert_eq!(
        dismissed_event["userID"].as_str(),
        Some(expected_user.as_str())
    );
    assert!(dismissed_event["callID"].as_str().is_some());

    let dismissed_state_event = wait_for_event(
        &mut ws_a,
        "custom_com.mattermost.calls_call_state",
        Duration::from_secs(5),
    )
    .await;
    let dismissed_payload = dismissed_state_event["call"]
        .as_str()
        .expect("dismissed call state event should include call JSON");
    let dismissed_json: serde_json::Value =
        serde_json::from_str(dismissed_payload).expect("dismissed call JSON should parse");
    let dismissed_user_id = encode_mm_id(user_b);
    assert_eq!(
        dismissed_json["dismissed_notification"][dismissed_user_id.as_str()],
        true
    );

    let call_state_resp = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls/calls/{}",
            app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("get call state request failed");
    assert_eq!(call_state_resp.status(), StatusCode::OK);
    let call_state_body: serde_json::Value = call_state_resp.json().await.expect("call state JSON");
    let dismissed_user_id = encode_mm_id(user_b);
    assert_eq!(
        call_state_body["dismissed_notification"][dismissed_user_id.as_str()],
        true
    );
}

async fn insert_org(app: &common::TestApp, name: &str) -> Uuid {
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind(name)
        .execute(&app.db_pool)
        .await
        .expect("failed to create organization");
    org_id
}

async fn register_and_login(
    app: &common::TestApp,
    org_id: Uuid,
    username: &str,
    email: &str,
) -> (String, Uuid) {
    app.api_client
        .post(format!("{}/api/v1/auth/register", app.address))
        .json(&serde_json::json!({
            "username": username,
            "email": email,
            "password": "Password123!",
            "display_name": username,
            "org_id": org_id,
        }))
        .send()
        .await
        .expect("register request failed")
        .error_for_status()
        .expect("register should succeed");

    let login = app
        .api_client
        .post(format!("{}/api/v4/users/login", app.address))
        .json(&serde_json::json!({
            "login_id": email,
            "password": "Password123!",
        }))
        .send()
        .await
        .expect("login request failed")
        .error_for_status()
        .expect("login should succeed");

    let token = login
        .headers()
        .get("Token")
        .and_then(|v| v.to_str().ok())
        .expect("token header missing")
        .to_string();

    let me = app
        .api_client
        .get(format!("{}/api/v4/users/me", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("me request failed")
        .error_for_status()
        .expect("me should succeed")
        .json::<serde_json::Value>()
        .await
        .expect("me response should be JSON");

    let user_id = me["id"]
        .as_str()
        .and_then(parse_mm_or_uuid)
        .expect("user id should parse");

    (token, user_id)
}

async fn create_team_and_channel_with_members(
    app: &common::TestApp,
    org_id: Uuid,
    users: &[Uuid],
) -> Uuid {
    create_team_and_channel_with_members_of_type(app, org_id, users, "public").await
}

async fn create_team_and_channel_with_members_of_type(
    app: &common::TestApp,
    org_id: Uuid,
    users: &[Uuid],
    channel_type: &str,
) -> Uuid {
    let suffix = Uuid::new_v4().to_string().replace('-', "");
    let team_id = Uuid::new_v4();
    let channel_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO teams (id, org_id, name, display_name, allow_open_invite) VALUES ($1, $2, $3, $4, true)",
    )
    .bind(team_id)
    .bind(org_id)
    .bind(format!("team_{suffix}"))
    .bind(format!("Team {suffix}"))
    .execute(&app.db_pool)
    .await
    .expect("failed to create team");

    sqlx::query("INSERT INTO channels (id, team_id, name, type) VALUES ($1, $2, $3, $4::channel_type)")
        .bind(channel_id)
        .bind(team_id)
        .bind(format!("channel_{suffix}"))
        .bind(channel_type)
        .execute(&app.db_pool)
        .await
        .expect("failed to create channel");

    for user_id in users {
        sqlx::query("INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, 'member')")
            .bind(team_id)
            .bind(user_id)
            .execute(&app.db_pool)
            .await
            .expect("failed to add team member");

        sqlx::query("INSERT INTO channel_members (channel_id, user_id, role, notify_props) VALUES ($1, $2, 'member', '{}')")
            .bind(channel_id)
            .bind(user_id)
            .execute(&app.db_pool)
            .await
            .expect("failed to add channel member");
    }

    channel_id
}

async fn connect_ws(base_http_url: &str, token: &str) -> WsClient {
    let ws_base = base_http_url.replacen("http://", "ws://", 1);
    let ws_url = format!("{ws_base}/api/v1/ws?token={token}");
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("websocket connection should succeed");
    ws_stream
}

async fn wait_for_event(
    ws: &mut WsClient,
    expected_event: &str,
    within: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + within;

    loop {
        let now = Instant::now();
        assert!(
            now < deadline,
            "timed out waiting for websocket event {expected_event}"
        );

        let timeout_left = deadline.saturating_duration_since(now);
        let message = tokio::time::timeout(timeout_left, ws.next())
            .await
            .expect("timeout while waiting for websocket frame")
            .expect("websocket closed unexpectedly")
            .expect("websocket frame should be valid");

        if let Message::Text(text) = message {
            let parsed: serde_json::Value =
                serde_json::from_str(&text).expect("frame should be valid JSON");
            if parsed["event"] == expected_event {
                return parsed["data"].clone();
            }
        }
    }
}

async fn subscribe_channel(ws: &mut WsClient, channel_id: Uuid) {
    let command = serde_json::json!({
        "type": "command",
        "event": "subscribe_channel",
        "channel_id": channel_id,
        "data": {},
    });
    ws.send(Message::Text(command.to_string().into()))
        .await
        .expect("subscribe command should be sent");
}
