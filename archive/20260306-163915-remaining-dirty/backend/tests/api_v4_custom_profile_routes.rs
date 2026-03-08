use crate::common::spawn_app;
use reqwest::{Response, StatusCode};
use rustchat::mattermost_compat::id::encode_mm_id;
use serde_json::json;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn custom_profile_field_management_requires_system_manage() {
    let app = spawn_app().await;
    let member_token =
        create_user_and_login_with_role(&app, "cpa_member", "cpa_member@example.com", "member")
            .await;

    let responses = vec![
        app.api_client
            .post(format!(
                "{}/api/v4/custom_profile_attributes/fields",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"name": "Department", "type": "text"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .patch(format!(
                "{}/api/v4/custom_profile_attributes/fields/field-id",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", member_token))
            .json(&json!({"name": "Updated"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .delete(format!(
                "{}/api/v4/custom_profile_attributes/fields/field-id",
                &app.address
            ))
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
async fn custom_profile_routes_match_contract_and_group_is_available() {
    let app = spawn_app().await;
    let admin_token =
        create_user_and_login_with_role(&app, "cpa_admin", "cpa_admin@example.com", "system_admin")
            .await;
    let member_token = create_user_and_login_with_role(
        &app,
        "cpa_group_member",
        "cpa_group_member@example.com",
        "member",
    )
    .await;

    let create_res = app
        .api_client
        .post(format!(
            "{}/api/v4/custom_profile_attributes/fields",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"name": "Department", "type": "text"}))
        .send()
        .await
        .unwrap();
    assert_mm_not_implemented(create_res).await;

    let patch_res = app
        .api_client
        .patch(format!(
            "{}/api/v4/custom_profile_attributes/fields/field-id",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({"name": "Updated"}))
        .send()
        .await
        .unwrap();
    assert_mm_not_implemented(patch_res).await;

    let delete_res = app
        .api_client
        .delete(format!(
            "{}/api/v4/custom_profile_attributes/fields/field-id",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_mm_not_implemented(delete_res).await;

    let get_field_res = app
        .api_client
        .get(format!(
            "{}/api/v4/custom_profile_attributes/fields/field-id",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::METHOD_NOT_ALLOWED, get_field_res.status());

    let group_res = app
        .api_client
        .get(format!(
            "{}/api/v4/custom_profile_attributes/group",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, group_res.status());
    let group_body: serde_json::Value = group_res.json().await.unwrap();
    assert!(group_body["id"].is_string());
}

#[tokio::test]
async fn custom_profile_values_roundtrip_uses_mattermost_map_shape() {
    let app = spawn_app().await;
    let member_email = "cpa_values_member@example.com";
    let member_token =
        create_user_and_login_with_role(&app, "cpa_values_member", member_email, "member").await;
    let member_id = lookup_user_id_by_email(&app, member_email).await;

    let field_id_text = insert_custom_profile_field(&app, "Department").await;
    let field_id_multi = insert_custom_profile_field(&app, "Skills").await;

    let field_text_mm = encode_mm_id(field_id_text);
    let field_multi_mm = encode_mm_id(field_id_multi);

    let payload = build_values_payload(vec![
        (field_text_mm.clone(), json!("Engineering")),
        (field_multi_mm.clone(), json!(["rust", "mobile"])),
    ]);

    let patch_res = app
        .api_client
        .patch(format!(
            "{}/api/v4/custom_profile_attributes/values",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", member_token))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, patch_res.status());

    let patched_body: serde_json::Value = patch_res.json().await.unwrap();
    assert_eq!(patched_body[&field_text_mm], json!("Engineering"));
    assert_eq!(patched_body[&field_multi_mm], json!(["rust", "mobile"]));

    let stored_text: String = sqlx::query_scalar(
        "SELECT value FROM custom_profile_attributes WHERE field_id = $1 AND user_id = $2",
    )
    .bind(field_id_text)
    .bind(member_id)
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(stored_text, "\"Engineering\"");

    let stored_multi: String = sqlx::query_scalar(
        "SELECT value FROM custom_profile_attributes WHERE field_id = $1 AND user_id = $2",
    )
    .bind(field_id_multi)
    .bind(member_id)
    .fetch_one(&app.db_pool)
    .await
    .unwrap();
    assert_eq!(stored_multi, "[\"rust\",\"mobile\"]");

    let get_res = app
        .api_client
        .get(format!(
            "{}/api/v4/users/{}/custom_profile_attributes",
            &app.address,
            encode_mm_id(member_id)
        ))
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, get_res.status());

    let get_body: serde_json::Value = get_res.json().await.unwrap();
    assert_eq!(get_body[&field_text_mm], json!("Engineering"));
    assert_eq!(get_body[&field_multi_mm], json!(["rust", "mobile"]));
}

#[tokio::test]
async fn custom_profile_user_patch_route_requires_owner_or_user_manage() {
    let app = spawn_app().await;

    let target_email = "cpa_patch_target@example.com";
    let target_token =
        create_user_and_login_with_role(&app, "cpa_patch_target", target_email, "member").await;
    let target_id = lookup_user_id_by_email(&app, target_email).await;

    let other_member_token = create_user_and_login_with_role(
        &app,
        "cpa_patch_other",
        "cpa_patch_other@example.com",
        "member",
    )
    .await;
    let admin_token = create_user_and_login_with_role(
        &app,
        "cpa_patch_admin",
        "cpa_patch_admin@example.com",
        "system_admin",
    )
    .await;

    let field_id = insert_custom_profile_field(&app, "Location").await;
    let field_mm = encode_mm_id(field_id);
    let target_mm = encode_mm_id(target_id);
    let payload = build_values_payload(vec![(field_mm.clone(), json!("Berlin"))]);

    let forbidden_res = app
        .api_client
        .patch(format!(
            "{}/api/v4/users/{}/custom_profile_attributes",
            &app.address, target_mm
        ))
        .header("Authorization", format!("Bearer {}", other_member_token))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::FORBIDDEN, forbidden_res.status());

    let owner_res = app
        .api_client
        .patch(format!(
            "{}/api/v4/users/{}/custom_profile_attributes",
            &app.address, target_mm
        ))
        .header("Authorization", format!("Bearer {}", target_token))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, owner_res.status());

    let admin_payload = build_values_payload(vec![(field_mm.clone(), json!("Munich"))]);
    let admin_res = app
        .api_client
        .patch(format!(
            "{}/api/v4/users/{}/custom_profile_attributes",
            &app.address, target_mm
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&admin_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(StatusCode::OK, admin_res.status());

    let admin_body: serde_json::Value = admin_res.json().await.unwrap();
    assert_eq!(admin_body[&field_mm], json!("Munich"));
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
        .bind("Custom Profile Test Org")
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

async fn lookup_user_id_by_email(app: &common::TestApp, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db_pool)
        .await
        .unwrap()
}

async fn insert_custom_profile_field(app: &common::TestApp, name: &str) -> Uuid {
    let field_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO custom_profile_fields (id, group_id, name, field_type, attrs, target_id, target_type)
        VALUES ($1, '', $2, 'text', '{}'::jsonb, '', 'user')
        "#,
    )
    .bind(field_id)
    .bind(name)
    .execute(&app.db_pool)
    .await
    .unwrap();
    field_id
}

fn build_values_payload(entries: Vec<(String, serde_json::Value)>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, value) in entries {
        map.insert(key, value);
    }
    serde_json::Value::Object(map)
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
