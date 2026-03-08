use crate::common::spawn_app;
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn access_control_endpoints_require_manage_system() {
    let app = spawn_app().await;
    let member_token = create_user_and_login_with_role(
        &app,
        "access_member",
        "access_member@example.com",
        "member",
    )
    .await;

    let policy_id = "policy-test";

    let responses = vec![
        app.api_client
            .put(format!("{}/api/v4/access_control_policies", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"type": "parent"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!("{}/api/v4/access_control_policies", &app.address))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/cel/check",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"expression": "true"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/cel/validate_requester",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"expression": "true"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/cel/test",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"expression": "true"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/search",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"term": "policy"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!(
                "{}/api/v4/access_control_policies/cel/autocomplete/fields?limit=10",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!(
                "{}/api/v4/access_control_policies/{}",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .delete(format!(
                "{}/api/v4/access_control_policies/{}",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!(
                "{}/api/v4/access_control_policies/{}/activate?active=true",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/{}/activate",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/{}/assign",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"channel_ids": ["channel-1"]}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .delete(format!(
                "{}/api/v4/access_control_policies/{}/unassign",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"channel_ids": ["channel-1"]}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/{}/unassign",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"channel_ids": ["channel-1"]}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!(
                "{}/api/v4/access_control_policies/{}/resources/channels?limit=10",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/{}/resources/channels/search",
                &app.address, policy_id
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"term": "test"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/cel/visual_ast",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"expression": "true"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .get(format!(
                "{}/api/v4/access_control_policies/cel/visual_ast",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .put(format!(
                "{}/api/v4/access_control_policies/activate",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"entries": []}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/access_control_policies/activate",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"entries": []}))
            .send()
            .await
            .unwrap(),
    ];

    for response in responses {
        assert_eq!(StatusCode::FORBIDDEN, response.status());
    }
}

#[tokio::test]
async fn access_control_canonical_methods_and_legacy_shims_work_for_system_admin() {
    let app = spawn_app().await;
    let admin_token = create_user_and_login_with_role(
        &app,
        "access_admin",
        "access_admin@example.com",
        "system_admin",
    )
    .await;

    let policy_id = "policy-admin";

    let create_res = app
        .api_client
        .put(format!("{}/api/v4/access_control_policies", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"type": "parent"}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, create_res.status());
    let create_body: serde_json::Value = create_res.json().await.unwrap();
    assert!(create_body.is_object());

    let create_post_res = app
        .api_client
        .post(format!("{}/api/v4/access_control_policies", &app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"type": "parent"}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::METHOD_NOT_ALLOWED, create_post_res.status());

    let delete_res = app
        .api_client
        .delete(format!(
            "{}/api/v4/access_control_policies/{}",
            &app.address, policy_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, delete_res.status());
    let delete_body: serde_json::Value = delete_res.json().await.unwrap();
    assert_eq!(delete_body["status"], "OK");

    let single_activate_res = app
        .api_client
        .get(format!(
            "{}/api/v4/access_control_policies/{}/activate?active=true",
            &app.address, policy_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, single_activate_res.status());

    let batch_activate_res = app
        .api_client
        .put(format!(
            "{}/api/v4/access_control_policies/activate",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"entries": []}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, batch_activate_res.status());

    let unassign_delete_res = app
        .api_client
        .delete(format!(
            "{}/api/v4/access_control_policies/{}/unassign",
            &app.address, policy_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"channel_ids": ["channel-1"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, unassign_delete_res.status());

    let visual_ast_res = app
        .api_client
        .post(format!(
            "{}/api/v4/access_control_policies/cel/visual_ast",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"expression": "true"}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, visual_ast_res.status());

    let legacy_activate_res = app
        .api_client
        .post(format!(
            "{}/api/v4/access_control_policies/{}/activate",
            &app.address, policy_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, legacy_activate_res.status());

    let legacy_unassign_res = app
        .api_client
        .post(format!(
            "{}/api/v4/access_control_policies/{}/unassign",
            &app.address, policy_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"channel_ids": ["channel-1"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, legacy_unassign_res.status());

    let legacy_visual_ast_res = app
        .api_client
        .get(format!(
            "{}/api/v4/access_control_policies/cel/visual_ast",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, legacy_visual_ast_res.status());

    let legacy_batch_activate_res = app
        .api_client
        .post(format!(
            "{}/api/v4/access_control_policies/activate",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"entries": []}))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, legacy_batch_activate_res.status());
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
        .bind("Access Control Test Org")
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
