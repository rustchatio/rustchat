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

#[tokio::test]
async fn status_ids_accepts_raw_and_wrapped_payloads() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Presence Status IDs Org").await;
    let (token, user_a) =
        register_and_login(&app, org_id, "presence_ids_a", "presence_ids_a@example.com").await;
    let (token_b, user_b) =
        register_and_login(&app, org_id, "presence_ids_b", "presence_ids_b@example.com").await;

    let user_b_mm = encode_mm_id(user_b);
    let set_away = app
        .api_client
        .put(format!("{}/api/v4/users/{}/status", app.address, user_b_mm))
        .header("Authorization", format!("Bearer {token_b}"))
        .json(&serde_json::json!({
            "user_id": user_b_mm,
            "status": "away"
        }))
        .send()
        .await
        .expect("status update should succeed");
    assert_eq!(set_away.status(), StatusCode::OK);

    let raw_res = app
        .api_client
        .post(format!("{}/api/v4/users/status/ids", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!([
            encode_mm_id(user_a),
            encode_mm_id(user_b)
        ]))
        .send()
        .await
        .expect("raw status ids request should succeed");
    assert_eq!(raw_res.status(), StatusCode::OK);
    let raw_body = raw_res
        .json::<serde_json::Value>()
        .await
        .expect("raw status ids body should be json");
    let raw_items = raw_body
        .as_array()
        .expect("status ids response should be an array");
    assert!(raw_items
        .iter()
        .any(|item| item["user_id"] == encode_mm_id(user_b) && item["status"] == "away"));

    let wrapped_res = app
        .api_client
        .post(format!("{}/api/v4/users/status/ids", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "user_ids": [encode_mm_id(user_a), encode_mm_id(user_b)]
        }))
        .send()
        .await
        .expect("wrapped status ids request should succeed");
    assert_eq!(wrapped_res.status(), StatusCode::OK);
    let wrapped_body = wrapped_res
        .json::<serde_json::Value>()
        .await
        .expect("wrapped status ids body should be json");
    assert!(wrapped_body.is_array());

    let empty_res = app
        .api_client
        .post(format!("{}/api/v4/users/status/ids", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!([]))
        .send()
        .await
        .expect("empty status ids request should return validation error");
    assert_eq!(empty_res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn custom_status_me_routes_are_supported_and_scoped() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Custom Status Me Org").await;
    let (token_a, user_a) =
        register_and_login(&app, org_id, "custom_me_a", "custom_me_a@example.com").await;
    let (token_b, _user_b) =
        register_and_login(&app, org_id, "custom_me_b", "custom_me_b@example.com").await;

    let put_me = app
        .api_client
        .put(format!("{}/api/v4/users/me/status/custom", app.address))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({
            "emoji": ":coffee:",
            "text": "Focus mode",
            "duration": "dont_clear"
        }))
        .send()
        .await
        .expect("put me custom status should succeed");
    assert_eq!(put_me.status(), StatusCode::OK);
    let put_body = put_me
        .json::<serde_json::Value>()
        .await
        .expect("custom status response should be json");
    assert_eq!(put_body["text"], "Focus mode");

    let get_recent = app
        .api_client
        .get(format!(
            "{}/api/v4/users/me/status/custom/recent",
            app.address
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("get me recent custom status should succeed");
    assert_eq!(get_recent.status(), StatusCode::OK);

    let delete_recent = app
        .api_client
        .post(format!(
            "{}/api/v4/users/me/status/custom/recent/delete",
            app.address
        ))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({
            "emoji": ":coffee:",
            "text": "Focus mode"
        }))
        .send()
        .await
        .expect("delete me recent custom status should succeed");
    assert_eq!(delete_recent.status(), StatusCode::OK);

    let clear_me = app
        .api_client
        .delete(format!("{}/api/v4/users/me/status/custom", app.address))
        .header("Authorization", format!("Bearer {token_a}"))
        .send()
        .await
        .expect("delete me custom status should succeed");
    assert_eq!(clear_me.status(), StatusCode::OK);

    let forbidden = app
        .api_client
        .put(format!(
            "{}/api/v4/users/{}/status/custom",
            app.address,
            encode_mm_id(user_a)
        ))
        .header("Authorization", format!("Bearer {token_b}"))
        .json(&serde_json::json!({
            "emoji": ":no_entry:",
            "text": "forbidden"
        }))
        .send()
        .await
        .expect("cross-user custom status should return forbidden");
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn status_change_event_reaches_other_connected_users() {
    let app = spawn_app().await;

    let org_id = insert_org(&app, "Presence Broadcast Org").await;
    let (token_a, user_a) = register_and_login(
        &app,
        org_id,
        "presence_broadcast_a",
        "presence_broadcast_a@example.com",
    )
    .await;
    let (token_b, _user_b) = register_and_login(
        &app,
        org_id,
        "presence_broadcast_b",
        "presence_broadcast_b@example.com",
    )
    .await;

    let mut ws_b = connect_ws_v4(&app.address, &token_b).await;
    let _ = wait_for_event(&mut ws_b, "hello", Duration::from_secs(5)).await;

    let update_status = app
        .api_client
        .put(format!("{}/api/v4/users/me/status", app.address))
        .header("Authorization", format!("Bearer {token_a}"))
        .json(&serde_json::json!({
            "user_id": encode_mm_id(user_a),
            "status": "away"
        }))
        .send()
        .await
        .expect("status update should succeed");
    assert_eq!(update_status.status(), StatusCode::OK);

    let event = wait_for_status_change_for_user(&mut ws_b, user_a, Duration::from_secs(5)).await;
    assert_eq!(event["user_id"], encode_mm_id(user_a));
    assert_eq!(event["status"], "away");

    let _ = ws_b.close(None).await;
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

async fn wait_for_status_change_for_user(
    ws: &mut WsClient,
    expected_user_id: Uuid,
    within: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + within;
    let expected_user_mm_id = encode_mm_id(expected_user_id);

    loop {
        let now = Instant::now();
        assert!(
            now < deadline,
            "timed out waiting for status_change for user {expected_user_mm_id}"
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
            if parsed["event"] == "status_change"
                && parsed["data"]["user_id"] == expected_user_mm_id
            {
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
    let ws_url = format!("{ws_base}/api/v4/websocket");

    let mut request = ws_url
        .into_client_request()
        .expect("websocket request should be valid");
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_str(token).expect("valid websocket subprotocol token"),
    );
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
