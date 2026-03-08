use crate::common::spawn_app;
use reqwest::{Response, StatusCode};
use serde_json::json;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn license_management_endpoints_require_system_manage() {
    let app = spawn_app().await;
    let member_token = create_user_and_login_with_role(
        &app,
        "license_member",
        "license_member@example.com",
        "member",
    )
    .await;

    let responses = vec![
        app.api_client
            .post(format!("{}/api/v4/license", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .delete(format!("{}/api/v4/license", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!("{}/api/v4/license", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!("{}/api/v4/license/renewal", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/license/renewal", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/trial-license", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({ "users": 10 }))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!("{}/api/v4/trial-license/prev", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
    ];

    for response in responses {
        assert_eq!(StatusCode::FORBIDDEN, response.status());
    }
}

#[tokio::test]
async fn license_canonical_methods_and_renewal_shape_match_contract() {
    let app = spawn_app().await;
    let admin_token = create_user_and_login_with_role(
        &app,
        "license_admin",
        "license_admin@example.com",
        "system_admin",
    )
    .await;

    let upload_res = app
        .api_client
        .post(format!("{}/api/v4/license", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_mm_not_implemented(upload_res).await;

    let remove_res = app
        .api_client
        .delete(format!("{}/api/v4/license", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_mm_not_implemented(remove_res).await;

    let renewal_get_res = app
        .api_client
        .get(format!("{}/api/v4/license/renewal", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, renewal_get_res.status());
    let renewal_get_body: serde_json::Value = renewal_get_res.json().await.unwrap();
    assert!(renewal_get_body["renewal_link"].is_string());

    let renewal_post_res = app
        .api_client
        .post(format!("{}/api/v4/license/renewal", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, renewal_post_res.status());
    let renewal_post_body: serde_json::Value = renewal_post_res.json().await.unwrap();
    assert!(renewal_post_body["renewal_link"].is_string());

    let trial_res = app
        .api_client
        .post(format!("{}/api/v4/trial-license", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({ "users": 10 }))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, trial_res.status());
    let trial_body: serde_json::Value = trial_res.json().await.unwrap();
    assert_eq!(trial_body["status"], "OK");
}

#[tokio::test]
async fn license_load_metric_remains_available_to_logged_in_users() {
    let app = spawn_app().await;
    let member_token = create_user_and_login_with_role(
        &app,
        "license_metric_member",
        "license_metric_member@example.com",
        "member",
    )
    .await;

    let response = app
        .api_client
        .get(format!("{}/api/v4/license/load_metric", &app.address))
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, response.status());
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.is_object());
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
        .bind("License Test Org")
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

async fn assert_mm_not_implemented(response: Response) {
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["id"].is_string());
    assert!(body["message"].is_string());
    assert!(body["detailed_error"].is_string());
    assert!(body["request_id"].is_string());
    assert_eq!(body["status_code"], 501);
}
