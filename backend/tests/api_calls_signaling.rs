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
        Duration::from_secs(5),
    )
    .await;
    assert_eq!(call_end["channel_id"], channel_id.to_string());
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

    sqlx::query("INSERT INTO channels (id, team_id, name, type) VALUES ($1, $2, $3, 'public')")
        .bind(channel_id)
        .bind(team_id)
        .bind(format!("channel_{suffix}"))
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
