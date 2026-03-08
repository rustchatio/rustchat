use crate::common::spawn_app;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn reports_endpoints_require_manage_system() {
    let app = spawn_app().await;
    let token = create_user_and_login_with_role(
        &app,
        "reports_member",
        "reports_member@example.com",
        "member",
    )
    .await;

    let responses = vec![
        app.api_client
            .get(format!("{}/api/v4/reports/users", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!("{}/api/v4/reports/users/count", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/reports/users/export", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/reports/posts", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&json!({ "channel_id": "any" }))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!("{}/api/v4/audit_logs/certificate", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
    ];

    for response in responses {
        assert_eq!(StatusCode::FORBIDDEN, response.status());
    }
}

#[tokio::test]
async fn reports_methods_match_mattermost_contract_with_validation() {
    let app = spawn_app().await;
    let admin_token = create_user_and_login_with_role(
        &app,
        "reports_admin",
        "reports_admin@example.com",
        "system_admin",
    )
    .await;

    let users_res = app
        .api_client
        .get(format!("{}/api/v4/reports/users", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, users_res.status());
    let users_body: serde_json::Value = users_res.json().await.unwrap();
    assert!(users_body.as_array().is_some());

    let count_res = app
        .api_client
        .get(format!("{}/api/v4/reports/users/count", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, count_res.status());
    let count_body: serde_json::Value = count_res.json().await.unwrap();
    assert!(count_body.is_number());

    let export_res = app
        .api_client
        .post(format!("{}/api/v4/reports/users/export", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, export_res.status());
    let export_body: serde_json::Value = export_res.json().await.unwrap();
    assert_eq!(export_body["status"], "OK");

    let posts_bad_res = app
        .api_client
        .post(format!("{}/api/v4/reports/posts", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, posts_bad_res.status());

    let posts_ok_res = app
        .api_client
        .post(format!("{}/api/v4/reports/posts", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({ "channel_id": "dummy-channel-id" }))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, posts_ok_res.status());
    let posts_ok_body: serde_json::Value = posts_ok_res.json().await.unwrap();
    assert!(posts_ok_body["posts"].is_object());
    assert!(posts_ok_body["next_cursor"].is_null());
}

#[tokio::test]
async fn reports_query_validation_rejects_invalid_filters() {
    let app = spawn_app().await;
    let admin_token = create_user_and_login_with_role(
        &app,
        "reports_validation_admin",
        "reports_validation_admin@example.com",
        "system_admin",
    )
    .await;

    let users_filter_conflict_res = app
        .api_client
        .get(format!(
            "{}/api/v4/reports/users?hide_active=true&hide_inactive=true",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, users_filter_conflict_res.status());

    let users_page_size_res = app
        .api_client
        .get(format!(
            "{}/api/v4/reports/users?page_size=101",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, users_page_size_res.status());

    let count_filter_conflict_res = app
        .api_client
        .get(format!(
            "{}/api/v4/reports/users/count?hide_active=true&hide_inactive=true",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::BAD_REQUEST, count_filter_conflict_res.status());
}

async fn create_user_and_login_with_role(
    app: &common::TestApp,
    username: &str,
    email: &str,
    role: &str,
) -> String {
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind("Reports Test Org")
        .execute(&app.db_pool)
        .await
        .unwrap();

    let user_data = json!({
        "username": username,
        "email": email,
        "password": "Password123!",
        "display_name": username,
        "org_id": org_id
    });

    app.api_client
        .post(format!("{}/api/v1/auth/register", &app.address))
        .json(&user_data)
        .send()
        .await
        .unwrap();

    if role != "member" {
        sqlx::query("UPDATE users SET role = $1 WHERE email = $2")
            .bind(role)
            .bind(email)
            .execute(&app.db_pool)
            .await
            .unwrap();
    }

    let login_res = app
        .api_client
        .post(format!("{}/api/v4/users/login", &app.address))
        .json(&json!({
            "login_id": email,
            "password": "Password123!"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, login_res.status());

    login_res
        .headers()
        .get("Token")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
