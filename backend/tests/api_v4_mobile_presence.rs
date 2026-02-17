use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::StatusCode;
use rustchat::mattermost_compat::id::encode_mm_id;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
};
use uuid::Uuid;

use crate::common::spawn_app;

mod common;

type WsClient =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[tokio::test]
async fn websocket_disconnect_sets_offline_for_non_manual_status() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Presence Lifecycle Org").await;
    let (token, _user_id) =
        register_and_login(&app, org_id, "presence_user", "presence_user@example.com").await;

    let mut ws = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws, "hello", Duration::from_secs(5)).await;

    let online_before_disconnect = poll_my_status(&app, &token, Duration::from_secs(3)).await;
    assert_eq!(online_before_disconnect["status"], "online");
    assert_eq!(online_before_disconnect["manual"], false);

    ws.close(None)
        .await
        .expect("websocket close frame should be sent");
    drop(ws);

    let offline_status =
        wait_for_status(&app, &token, "offline", false, Duration::from_secs(8)).await;
    assert_eq!(offline_status["status"], "offline");
    assert_eq!(offline_status["manual"], false);

    let mut ws_reconnected = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws_reconnected, "hello", Duration::from_secs(5)).await;

    let online_status =
        wait_for_status(&app, &token, "online", false, Duration::from_secs(8)).await;
    assert_eq!(online_status["status"], "online");
    assert_eq!(online_status["manual"], false);

    let _ = ws_reconnected.close(None).await;
}

#[tokio::test]
async fn websocket_disconnect_preserves_manual_status() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Presence Grace Org").await;
    let (token, user_id) = register_and_login(
        &app,
        org_id,
        "presence_grace_user",
        "presence_grace_user@example.com",
    )
    .await;

    let mut ws = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws, "hello", Duration::from_secs(5)).await;

    let set_dnd = app
        .api_client
        .put(format!("{}/api/v4/users/me/status", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "status": "dnd"
        }))
        .send()
        .await
        .expect("status update should succeed");
    assert_eq!(set_dnd.status(), StatusCode::OK);

    ws.close(None)
        .await
        .expect("websocket close frame should be sent");
    drop(ws);

    tokio::time::sleep(Duration::from_secs(2)).await;

    let status_during_grace = poll_my_status(&app, &token, Duration::from_secs(3)).await;
    assert_eq!(status_during_grace["status"], "dnd");
    assert_eq!(status_during_grace["manual"], true);

    let mut ws_reconnected = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws_reconnected, "hello", Duration::from_secs(5)).await;

    let status_after_reconnect = poll_my_status(&app, &token, Duration::from_secs(3)).await;
    assert_eq!(status_after_reconnect["status"], "dnd");
    assert_eq!(status_after_reconnect["manual"], true);

    let _ = ws_reconnected.close(None).await;
}

#[tokio::test]
async fn user_stays_online_until_last_websocket_disconnects() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Presence Mobile Keep Online Org").await;
    let (token, _user_id) = register_and_login(
        &app,
        org_id,
        "presence_mobile_keep_online",
        "presence_mobile_keep_online@example.com",
    )
    .await;

    let mut ws_one = connect_ws_v4_mobile(&app.address, &token).await;
    let _ = wait_for_event(&mut ws_one, "hello", Duration::from_secs(5)).await;

    let mut ws_two = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws_two, "hello", Duration::from_secs(5)).await;

    ws_one
        .close(None)
        .await
        .expect("websocket close frame should be sent");
    drop(ws_one);

    tokio::time::sleep(Duration::from_secs(2)).await;

    let status_with_second_connection = poll_my_status(&app, &token, Duration::from_secs(3)).await;
    assert_eq!(status_with_second_connection["status"], "online");
    assert_eq!(status_with_second_connection["manual"], false);

    ws_two
        .close(None)
        .await
        .expect("websocket close frame should be sent");
    drop(ws_two);

    let offline_status =
        wait_for_status(&app, &token, "offline", false, Duration::from_secs(8)).await;
    assert_eq!(offline_status["status"], "offline");
    assert_eq!(offline_status["manual"], false);
}

async fn poll_my_status(app: &common::TestApp, token: &str, within: Duration) -> serde_json::Value {
    let deadline = Instant::now() + within;
    loop {
        let res = app
            .api_client
            .get(format!("{}/api/v4/users/me/status", app.address))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("status request should succeed");
        if res.status() == StatusCode::OK {
            return res
                .json::<serde_json::Value>()
                .await
                .expect("status response should be valid json");
        }

        assert!(
            Instant::now() < deadline,
            "timed out polling my status endpoint"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_status(
    app: &common::TestApp,
    token: &str,
    expected_status: &str,
    expected_manual: bool,
    within: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + within;

    loop {
        let status = poll_my_status(app, token, Duration::from_secs(2)).await;
        let status_value = status["status"].as_str().unwrap_or_default();
        let manual_value = status["manual"].as_bool().unwrap_or(false);
        if status_value == expected_status && manual_value == expected_manual {
            return status;
        }

        assert!(
            Instant::now() < deadline,
            "timed out waiting for status={expected_status} manual={expected_manual}; got status={status_value} manual={manual_value}"
        );
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
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

async fn connect_ws_v4(base_http_url: &str, token: &str) -> WsClient {
    connect_ws_v4_with_user_agent(base_http_url, token, None).await
}

async fn connect_ws_v4_mobile(base_http_url: &str, token: &str) -> WsClient {
    connect_ws_v4_with_user_agent(
        base_http_url,
        token,
        Some("RustChat Mobile/2.38.0+720 (Android; 16; CPH2653)"),
    )
    .await
}

async fn connect_ws_v4_with_user_agent(
    base_http_url: &str,
    token: &str,
    user_agent: Option<&str>,
) -> WsClient {
    let ws_base = base_http_url.replacen("http://", "ws://", 1);
    let ws_url = format!("{ws_base}/api/v4/websocket?token={token}");

    let mut request = ws_url
        .into_client_request()
        .expect("websocket request should be valid");
    if let Some(ua) = user_agent {
        request.headers_mut().insert(
            "User-Agent",
            HeaderValue::from_str(ua).expect("valid user-agent"),
        );
    }

    let (ws_stream, _) = connect_async(request)
        .await
        .expect("websocket connection should succeed");
    ws_stream
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
        .and_then(rustchat::mattermost_compat::id::parse_mm_or_uuid)
        .expect("user id should parse");

    (token, user_id)
}
