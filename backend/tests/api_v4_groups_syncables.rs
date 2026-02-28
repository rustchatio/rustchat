use crate::common::spawn_app;
use once_cell::sync::Lazy;
use rustchat::mattermost_compat::id::parse_mm_or_uuid;
use rustchat::models::Team;
use std::sync::Mutex;
use std::time::Duration;
use uuid::Uuid;

mod common;

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

async fn register_user(app: &common::TestApp, username: &str, email: &str, password: &str) {
    let payload = serde_json::json!({
        "username": username,
        "email": email,
        "password": password,
        "display_name": username
    });

    let response = app
        .api_client
        .post(format!("{}/api/v1/auth/register", &app.address))
        .json(&payload)
        .send()
        .await
        .expect("register request failed");

    assert_eq!(
        response.status().as_u16(),
        200,
        "register should succeed, got {}",
        response.status()
    );
}

async fn login_token(app: &common::TestApp, email: &str, password: &str) -> String {
    let payload = serde_json::json!({
        "email": email,
        "password": password
    });

    let response = app
        .api_client
        .post(format!("{}/api/v1/auth/login", &app.address))
        .json(&payload)
        .send()
        .await
        .expect("login request failed");

    assert_eq!(response.status().as_u16(), 200, "login should succeed");
    let body: serde_json::Value = response.json().await.expect("invalid login response");
    body["token"].as_str().expect("missing token").to_string()
}

async fn create_team(app: &common::TestApp, owner_token: &str, name: &str) -> Team {
    let payload = serde_json::json!({
        "name": name,
        "display_name": format!("{} Display", name),
        "description": "test team"
    });

    let response = app
        .api_client
        .post(format!("{}/api/v1/teams", &app.address))
        .header("Authorization", format!("Bearer {}", owner_token))
        .json(&payload)
        .send()
        .await
        .expect("team create request failed");

    assert_eq!(
        response.status().as_u16(),
        200,
        "team create should succeed"
    );
    response.json().await.expect("invalid team response")
}

async fn insert_ldap_group(app: &common::TestApp, name: &str, display_name: &str) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO groups (name, display_name, description, source, allow_reference)
        VALUES ($1, $2, '', 'ldap', true)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(display_name)
    .fetch_one(&app.db_pool)
    .await
    .expect("failed to insert ldap group")
}

async fn add_group_member(app: &common::TestApp, group_id: Uuid, user_id: Uuid) {
    sqlx::query(
        "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&app.db_pool)
    .await
    .expect("failed to insert group member");
}

async fn set_user_role(app: &common::TestApp, email: &str, role: &str) {
    sqlx::query("UPDATE users SET role = $1 WHERE email = $2")
        .bind(role)
        .bind(email)
        .execute(&app.db_pool)
        .await
        .expect("failed to update user role");
}

async fn user_id_by_email(app: &common::TestApp, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch user id")
}

async fn wait_for_condition<F, Fut>(mut condition: F, timeout: Duration)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    loop {
        if condition().await {
            return;
        }

        if start.elapsed() >= timeout {
            panic!("condition not met within {:?}", timeout);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn v4_team_syncable_link_patch_and_retrieve() {
    let _guard = TEST_MUTEX.lock().expect("test mutex poisoned");
    let app = spawn_app().await;

    register_user(&app, "sync_admin", "sync_admin@example.com", "Password123!").await;
    register_user(&app, "sync_user", "sync_user@example.com", "Password123!").await;
    set_user_role(&app, "sync_admin@example.com", "system_admin").await;

    let admin_token = login_token(&app, "sync_admin@example.com", "Password123!").await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind("sync_user@example.com")
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch sync user id");

    let team = create_team(&app, &admin_token, "sync-team-a2").await;
    let group_id = insert_ldap_group(&app, "ldap-group-a2-team", "LDAP Group A2 Team").await;
    add_group_member(&app, group_id, user_id).await;

    let link_response = app
        .api_client
        .post(format!(
            "{}/api/v4/groups/{}/teams/{}/link",
            &app.address, group_id, team.id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "auto_add": true,
            "scheme_admin": false
        }))
        .send()
        .await
        .expect("link team syncable request failed");

    assert_eq!(
        link_response.status().as_u16(),
        201,
        "link should return 201"
    );
    let link_body: serde_json::Value = link_response.json().await.expect("invalid link body");
    assert_eq!(link_body["type"], "Team");
    assert_eq!(link_body["auto_add"], true);
    let linked_team_id = parse_mm_or_uuid(
        link_body["team_id"]
            .as_str()
            .expect("missing team_id in syncable payload"),
    )
    .expect("team_id should parse as mm/uuid");
    assert_eq!(linked_team_id, team.id);

    wait_for_condition(
        || async {
            sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
            )
            .bind(team.id)
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or(false)
        },
        Duration::from_secs(5),
    )
    .await;

    let get_one_response = app
        .api_client
        .get(format!(
            "{}/api/v4/groups/{}/teams/{}",
            &app.address, group_id, team.id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("get syncable request failed");
    assert_eq!(get_one_response.status().as_u16(), 200);
    let get_one_body: serde_json::Value = get_one_response.json().await.expect("invalid body");
    assert_eq!(get_one_body["type"], "Team");

    let patch_response = app
        .api_client
        .put(format!(
            "{}/api/v4/groups/{}/teams/{}/patch",
            &app.address, group_id, team.id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "auto_add": false
        }))
        .send()
        .await
        .expect("patch syncable request failed");
    assert_eq!(patch_response.status().as_u16(), 200);
    let patch_body: serde_json::Value = patch_response.json().await.expect("invalid patch body");
    assert_eq!(patch_body["auto_add"], false);

    wait_for_condition(
        || async {
            sqlx::query_scalar::<_, bool>(
                "SELECT NOT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
            )
            .bind(team.id)
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or(false)
        },
        Duration::from_secs(5),
    )
    .await;

    // team-scoped admins can mutate syncables, but reading syncable catalogs is system-admin scoped.
    let non_system_read = app
        .api_client
        .get(format!("{}/api/v4/groups/{}/teams", &app.address, group_id))
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                login_token(&app, "sync_user@example.com", "Password123!").await
            ),
        )
        .send()
        .await
        .expect("non-system read request failed");
    assert_eq!(non_system_read.status().as_u16(), 403);

    let get_list_response = app
        .api_client
        .get(format!("{}/api/v4/groups/{}/teams", &app.address, group_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("get syncables list failed");
    assert_eq!(get_list_response.status().as_u16(), 200);
    let list_body: Vec<serde_json::Value> = get_list_response.json().await.expect("invalid list");
    assert_eq!(list_body.len(), 1);
    assert_eq!(list_body[0]["auto_add"], false);
}

#[tokio::test]
async fn v4_channel_syncable_link_and_unlink_cleans_memberships() {
    let _guard = TEST_MUTEX.lock().expect("test mutex poisoned");
    let app = spawn_app().await;

    register_user(
        &app,
        "channel_sync_admin",
        "channel_sync_admin@example.com",
        "Password123!",
    )
    .await;
    register_user(
        &app,
        "channel_sync_user",
        "channel_sync_user@example.com",
        "Password123!",
    )
    .await;
    set_user_role(&app, "channel_sync_admin@example.com", "system_admin").await;

    let admin_token = login_token(&app, "channel_sync_admin@example.com", "Password123!").await;
    let user_id: Uuid = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind("channel_sync_user@example.com")
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch sync user id");

    let team = create_team(&app, &admin_token, "sync-team-channel-a2").await;
    let channel_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO channels (team_id, type, name, display_name)
        VALUES ($1, 'private'::channel_type, 'a2-sync-channel', 'A2 Sync Channel')
        RETURNING id
        "#,
    )
    .bind(team.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("failed to create private channel");

    let group_id = insert_ldap_group(&app, "ldap-group-a2-channel", "LDAP Group A2 Channel").await;
    add_group_member(&app, group_id, user_id).await;

    let link_response = app
        .api_client
        .post(format!(
            "{}/api/v4/groups/{}/channels/{}/link",
            &app.address, group_id, channel_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "auto_add": true,
            "scheme_admin": false
        }))
        .send()
        .await
        .expect("link channel syncable request failed");
    assert_eq!(link_response.status().as_u16(), 201);

    wait_for_condition(
        || async {
            let in_team: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
            )
            .bind(team.id)
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or(false);
            let in_channel: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
            )
            .bind(channel_id)
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or(false);
            in_team && in_channel
        },
        Duration::from_secs(5),
    )
    .await;

    let unlink_response = app
        .api_client
        .delete(format!(
            "{}/api/v4/groups/{}/channels/{}/link",
            &app.address, group_id, channel_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("unlink channel syncable request failed");
    assert_eq!(unlink_response.status().as_u16(), 200);

    wait_for_condition(
        || async {
            let not_in_channel: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
            )
            .bind(channel_id)
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or(false);
            let not_in_team: bool = sqlx::query_scalar(
                "SELECT NOT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
            )
            .bind(team.id)
            .bind(user_id)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or(false);
            not_in_channel && not_in_team
        },
        Duration::from_secs(5),
    )
    .await;
}

#[tokio::test]
async fn v4_non_admin_without_scope_cannot_link_team_syncable() {
    let _guard = TEST_MUTEX.lock().expect("test mutex poisoned");
    let app = spawn_app().await;

    register_user(
        &app,
        "team_owner_scope",
        "team_owner_scope@example.com",
        "Password123!",
    )
    .await;
    register_user(
        &app,
        "outsider_scope",
        "outsider_scope@example.com",
        "Password123!",
    )
    .await;

    let owner_token = login_token(&app, "team_owner_scope@example.com", "Password123!").await;
    let outsider_token = login_token(&app, "outsider_scope@example.com", "Password123!").await;

    let team = create_team(&app, &owner_token, "scope-check-team").await;
    let group_id =
        insert_ldap_group(&app, "ldap-group-scope-check", "LDAP Group Scope Check").await;

    let response = app
        .api_client
        .post(format!(
            "{}/api/v4/groups/{}/teams/{}/link",
            &app.address, group_id, team.id
        ))
        .header("Authorization", format!("Bearer {}", outsider_token))
        .json(&serde_json::json!({
            "auto_add": true,
            "scheme_admin": false
        }))
        .send()
        .await
        .expect("link request failed");

    assert_eq!(response.status().as_u16(), 403);
}

#[tokio::test]
async fn v4_team_admin_can_link_team_syncable_without_system_role() {
    let _guard = TEST_MUTEX.lock().expect("test mutex poisoned");
    let app = spawn_app().await;

    register_user(
        &app,
        "team_admin_scope",
        "team_admin_scope@example.com",
        "Password123!",
    )
    .await;

    let team_admin_token = login_token(&app, "team_admin_scope@example.com", "Password123!").await;
    let team = create_team(&app, &team_admin_token, "team-admin-scope-team").await;
    let group_id = insert_ldap_group(
        &app,
        "ldap-group-team-admin-scope",
        "LDAP Group Team Admin Scope",
    )
    .await;

    let response = app
        .api_client
        .post(format!(
            "{}/api/v4/groups/{}/teams/{}/link",
            &app.address, group_id, team.id
        ))
        .header("Authorization", format!("Bearer {}", team_admin_token))
        .json(&serde_json::json!({
            "auto_add": true,
            "scheme_admin": false
        }))
        .send()
        .await
        .expect("link request failed");

    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn v4_group_association_queries_and_visibility_match_expected_contract() {
    let _guard = TEST_MUTEX.lock().expect("test mutex poisoned");
    let app = spawn_app().await;

    register_user(
        &app,
        "assoc_admin",
        "assoc_admin@example.com",
        "Password123!",
    )
    .await;
    register_user(
        &app,
        "assoc_member",
        "assoc_member@example.com",
        "Password123!",
    )
    .await;
    set_user_role(&app, "assoc_admin@example.com", "system_admin").await;

    let admin_token = login_token(&app, "assoc_admin@example.com", "Password123!").await;
    let member_token = login_token(&app, "assoc_member@example.com", "Password123!").await;

    let member_id = user_id_by_email(&app, "assoc_member@example.com").await;
    let team = create_team(&app, &admin_token, "assoc-team").await;

    sqlx::query("INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(team.id)
        .bind(member_id)
        .execute(&app.db_pool)
        .await
        .expect("failed to add member to team");

    let channel_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO channels (team_id, type, name, display_name)
        VALUES ($1, 'private'::channel_type, 'assoc-private', 'Assoc Private')
        RETURNING id
        "#,
    )
    .bind(team.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("failed to create private channel");

    sqlx::query(
        "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member')",
    )
    .bind(channel_id)
    .bind(member_id)
    .execute(&app.db_pool)
    .await
    .expect("failed to add member to channel");

    let group_visible_alpha = insert_ldap_group(&app, "alpha-visible", "Alpha Visible").await;
    let group_hidden_alpha = insert_ldap_group(&app, "alpha-hidden", "Alpha Hidden").await;
    let group_visible_beta = insert_ldap_group(&app, "beta-visible", "Beta Visible").await;

    sqlx::query("UPDATE groups SET allow_reference = false WHERE id = $1")
        .bind(group_hidden_alpha)
        .execute(&app.db_pool)
        .await
        .expect("failed to mark hidden group");

    for group_id in [group_visible_alpha, group_hidden_alpha, group_visible_beta] {
        sqlx::query(
            r#"
            INSERT INTO group_syncables (group_id, syncable_type, syncable_id, auto_add, scheme_admin)
            VALUES ($1, 'team', $2, true, false)
            "#,
        )
        .bind(group_id)
        .bind(team.id)
        .execute(&app.db_pool)
        .await
        .expect("failed to insert team syncable");

        sqlx::query(
            r#"
            INSERT INTO group_syncables (group_id, syncable_type, syncable_id, auto_add, scheme_admin)
            VALUES ($1, 'channel', $2, true, false)
            "#,
        )
        .bind(group_id)
        .bind(channel_id)
        .execute(&app.db_pool)
        .await
        .expect("failed to insert channel syncable");

        add_group_member(&app, group_id, member_id).await;
    }

    let member_team_groups = app
        .api_client
        .get(format!(
            "{}/api/v4/teams/{}/groups?q=alpha&paginate=true&page=0&per_page=1",
            &app.address, team.id
        ))
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .expect("member team groups request failed");
    assert_eq!(member_team_groups.status().as_u16(), 200);
    let member_team_body: serde_json::Value = member_team_groups
        .json()
        .await
        .expect("invalid member team groups body");
    assert_eq!(member_team_body["total_group_count"], 1);
    let member_team_groups_arr = member_team_body["groups"]
        .as_array()
        .expect("groups should be array");
    assert_eq!(member_team_groups_arr.len(), 1);
    assert_eq!(member_team_groups_arr[0]["display_name"], "Alpha Visible");

    let admin_team_groups = app
        .api_client
        .get(format!(
            "{}/api/v4/teams/{}/groups?q=alpha&paginate=false&filter_allow_reference=false",
            &app.address, team.id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("admin team groups request failed");
    assert_eq!(admin_team_groups.status().as_u16(), 200);
    let admin_team_body: serde_json::Value = admin_team_groups
        .json()
        .await
        .expect("invalid admin team groups body");
    let admin_team_groups_arr = admin_team_body["groups"]
        .as_array()
        .expect("groups should be array");
    assert_eq!(admin_team_body["total_group_count"], 2);
    assert_eq!(admin_team_groups_arr.len(), 2);

    let member_channel_groups = app
        .api_client
        .get(format!(
            "{}/api/v4/channels/{}/groups?paginate=false",
            &app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .expect("member channel groups request failed");
    assert_eq!(member_channel_groups.status().as_u16(), 200);
    let member_channel_body: serde_json::Value = member_channel_groups
        .json()
        .await
        .expect("invalid member channel groups body");
    let member_channel_groups_arr = member_channel_body["groups"]
        .as_array()
        .expect("groups should be array");
    assert_eq!(member_channel_body["total_group_count"], 2);
    assert_eq!(member_channel_groups_arr.len(), 2);
    assert!(!member_channel_groups_arr
        .iter()
        .any(|group| group["display_name"] == "Alpha Hidden"));

    let member_user_groups = app
        .api_client
        .get(format!(
            "{}/api/v4/users/{}/groups",
            &app.address, member_id
        ))
        .header("Authorization", format!("Bearer {}", member_token))
        .send()
        .await
        .expect("member user groups request failed");
    assert_eq!(member_user_groups.status().as_u16(), 200);
    let member_user_groups_body: serde_json::Value = member_user_groups
        .json()
        .await
        .expect("invalid member user groups body");
    let member_user_groups_arr = member_user_groups_body
        .as_array()
        .expect("user groups should be array");
    assert_eq!(member_user_groups_arr.len(), 2);
    assert!(!member_user_groups_arr
        .iter()
        .any(|group| group["display_name"] == "Alpha Hidden"));
}

#[tokio::test]
async fn v4_group_association_endpoints_reject_non_members_without_system_scope() {
    let _guard = TEST_MUTEX.lock().expect("test mutex poisoned");
    let app = spawn_app().await;

    register_user(
        &app,
        "assoc_owner",
        "assoc_owner@example.com",
        "Password123!",
    )
    .await;
    register_user(
        &app,
        "assoc_outsider",
        "assoc_outsider@example.com",
        "Password123!",
    )
    .await;

    let owner_token = login_token(&app, "assoc_owner@example.com", "Password123!").await;
    let outsider_token = login_token(&app, "assoc_outsider@example.com", "Password123!").await;
    let owner_id = user_id_by_email(&app, "assoc_owner@example.com").await;

    let team = create_team(&app, &owner_token, "assoc-perm-team").await;
    let channel_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO channels (team_id, type, name, display_name)
        VALUES ($1, 'private'::channel_type, 'assoc-perm-private', 'Assoc Perm Private')
        RETURNING id
        "#,
    )
    .bind(team.id)
    .fetch_one(&app.db_pool)
    .await
    .expect("failed to create private channel");

    let group_id = insert_ldap_group(&app, "assoc-perm-group", "Assoc Perm Group").await;
    sqlx::query(
        r#"
        INSERT INTO group_syncables (group_id, syncable_type, syncable_id, auto_add, scheme_admin)
        VALUES ($1, 'team', $2, true, false)
        "#,
    )
    .bind(group_id)
    .bind(team.id)
    .execute(&app.db_pool)
    .await
    .expect("failed to insert team syncable");
    sqlx::query(
        r#"
        INSERT INTO group_syncables (group_id, syncable_type, syncable_id, auto_add, scheme_admin)
        VALUES ($1, 'channel', $2, true, false)
        "#,
    )
    .bind(group_id)
    .bind(channel_id)
    .execute(&app.db_pool)
    .await
    .expect("failed to insert channel syncable");

    let team_forbidden = app
        .api_client
        .get(format!("{}/api/v4/teams/{}/groups", &app.address, team.id))
        .header("Authorization", format!("Bearer {}", outsider_token))
        .send()
        .await
        .expect("outsider team groups request failed");
    assert_eq!(team_forbidden.status().as_u16(), 403);

    let channel_forbidden = app
        .api_client
        .get(format!(
            "{}/api/v4/channels/{}/groups",
            &app.address, channel_id
        ))
        .header("Authorization", format!("Bearer {}", outsider_token))
        .send()
        .await
        .expect("outsider channel groups request failed");
    assert_eq!(channel_forbidden.status().as_u16(), 403);

    let user_forbidden = app
        .api_client
        .get(format!("{}/api/v4/users/{}/groups", &app.address, owner_id))
        .header("Authorization", format!("Bearer {}", outsider_token))
        .send()
        .await
        .expect("outsider user groups request failed");
    assert_eq!(user_forbidden.status().as_u16(), 403);
}
