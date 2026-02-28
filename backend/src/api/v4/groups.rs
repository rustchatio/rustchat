use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use crate::models::channel::ChannelType;
use axum::{
    extract::{Path, State},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::FromRow;
use std::collections::HashSet;
use uuid::Uuid;

const GROUP_SOURCE_CUSTOM: &str = "custom";
const GROUP_SOURCE_LDAP: &str = "ldap";
const GROUP_SOURCE_PLUGIN_PREFIX: &str = "plugin_";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SyncableKind {
    Team,
    Channel,
}

impl SyncableKind {
    fn parse(path_value: &str) -> ApiResult<Self> {
        match path_value {
            "teams" => Ok(Self::Team),
            "channels" => Ok(Self::Channel),
            _ => Err(AppError::BadRequest(format!(
                "Invalid syncable type: {path_value}"
            ))),
        }
    }

    fn as_db_str(self) -> &'static str {
        match self {
            Self::Team => "team",
            Self::Channel => "channel",
        }
    }

    fn as_mm_type(self) -> &'static str {
        match self {
            Self::Team => "Team",
            Self::Channel => "Channel",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
struct GroupRow {
    id: Uuid,
    name: Option<String>,
    display_name: String,
    description: String,
    source: String,
    remote_id: Option<String>,
    allow_reference: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
struct GroupListRow {
    id: Uuid,
    name: Option<String>,
    display_name: String,
    description: String,
    source: String,
    remote_id: Option<String>,
    allow_reference: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
    has_syncables: bool,
    member_count: i64,
}

#[derive(Debug, Clone, FromRow)]
struct GroupSyncableRow {
    group_id: Uuid,
    syncable_type: String,
    syncable_id: Uuid,
    auto_add: bool,
    scheme_admin: bool,
    create_at: DateTime<Utc>,
    update_at: DateTime<Utc>,
    delete_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
struct TeamMetaRow {
    id: Uuid,
    name: String,
    display_name: Option<String>,
    is_public: bool,
}

#[derive(Debug, Clone, FromRow)]
struct ChannelMetaRow {
    id: Uuid,
    name: String,
    display_name: Option<String>,
    channel_type: ChannelType,
    team_id: Uuid,
    team_name: String,
    team_display_name: Option<String>,
    team_is_public: bool,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq, Hash)]
struct TrackedMembershipRow {
    target_type: String,
    target_id: Uuid,
    user_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct CreateGroupRequest {
    name: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    source: Option<String>,
    remote_id: Option<String>,
    allow_reference: Option<bool>,
    user_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PatchGroupRequest {
    name: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    allow_reference: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GroupSyncablePatch {
    auto_add: Option<bool>,
    scheme_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GroupModifyMembersRequest {
    user_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DesiredMembership {
    target_type: String,
    target_id: Uuid,
    user_id: Uuid,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/groups", get(get_groups).post(create_group))
        .route(
            "/groups/{group_id}",
            get(get_group).put(patch_group).delete(delete_group),
        )
        .route("/groups/{group_id}/patch", put(patch_group))
        .route("/groups/{group_id}/restore", post(restore_group))
        .route(
            "/groups/{group_id}/{syncable_type}/{syncable_id}/link",
            post(link_group_syncable).delete(unlink_group_syncable),
        )
        .route(
            "/groups/{group_id}/{syncable_type}/{syncable_id}",
            get(get_group_syncable),
        )
        .route(
            "/groups/{group_id}/{syncable_type}",
            get(get_group_syncables),
        )
        .route(
            "/groups/{group_id}/{syncable_type}/{syncable_id}/patch",
            put(patch_group_syncable),
        )
        .route("/groups/{group_id}/stats", get(get_group_stats))
        .route(
            "/groups/{group_id}/members",
            get(get_group_members)
                .post(add_group_members)
                .delete(delete_group_members),
        )
        .route("/groups/names", post(get_groups_by_names))
}

fn ts_millis(ts: DateTime<Utc>) -> i64 {
    ts.timestamp_millis()
}

fn team_type_value(is_public: bool) -> &'static str {
    if is_public {
        "O"
    } else {
        "I"
    }
}

fn channel_type_value(channel_type: ChannelType) -> &'static str {
    match channel_type {
        ChannelType::Public => "O",
        ChannelType::Private => "P",
        ChannelType::Direct => "D",
        ChannelType::Group => "G",
    }
}

fn group_json(row: &GroupListRow) -> Value {
    json!({
        "id": encode_mm_id(row.id),
        "name": row.name,
        "display_name": row.display_name,
        "description": row.description,
        "source": row.source,
        "remote_id": row.remote_id,
        "create_at": ts_millis(row.created_at),
        "update_at": ts_millis(row.updated_at),
        "delete_at": row.deleted_at.map(ts_millis).unwrap_or(0),
        "has_syncables": row.has_syncables,
        "member_count": row.member_count,
        "allow_reference": row.allow_reference,
    })
}

fn group_member_json(
    group_id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    delete_at: i64,
) -> Value {
    json!({
        "group_id": encode_mm_id(group_id),
        "user_id": encode_mm_id(user_id),
        "create_at": ts_millis(created_at),
        "delete_at": delete_at,
    })
}

fn can_manage_system_groups(auth: &crate::api::v4::extractors::MmAuthUser) -> bool {
    auth.has_permission(&permissions::SYSTEM_MANAGE)
        || auth.has_permission(&permissions::ADMIN_FULL)
}

fn require_system_groups_read(auth: &crate::api::v4::extractors::MmAuthUser) -> ApiResult<()> {
    if can_manage_system_groups(auth) {
        return Ok(());
    }

    Err(AppError::Forbidden(
        "Insufficient permissions to read group management data".to_string(),
    ))
}

fn require_system_groups_write(auth: &crate::api::v4::extractors::MmAuthUser) -> ApiResult<()> {
    if can_manage_system_groups(auth) {
        return Ok(());
    }

    Err(AppError::Forbidden(
        "Insufficient permissions to manage groups".to_string(),
    ))
}

async fn fetch_group_for_syncable(state: &AppState, group_id: Uuid) -> ApiResult<GroupRow> {
    let group: GroupRow = sqlx::query_as(
        r#"
        SELECT id, name, display_name, description, source, remote_id, allow_reference, created_at, updated_at, deleted_at
        FROM groups
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    Ok(group)
}

async fn is_team_admin_or_owner(state: &AppState, team_id: Uuid, user_id: Uuid) -> ApiResult<bool> {
    let is_admin: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM team_members
            WHERE team_id = $1
              AND user_id = $2
              AND role IN ('admin', 'owner', 'team_admin')
        )
        "#,
    )
    .bind(team_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(is_admin)
}

async fn can_manage_channel_syncable(
    state: &AppState,
    channel_id: Uuid,
    user_id: Uuid,
) -> ApiResult<bool> {
    let is_channel_admin: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM channel_members
            WHERE channel_id = $1
              AND user_id = $2
              AND role IN ('admin', 'channel_admin')
        )
        "#,
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    if is_channel_admin {
        return Ok(true);
    }

    let team_id: Option<Uuid> = sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(&state.db)
        .await?;

    let Some(team_id) = team_id else {
        return Ok(false);
    };

    is_team_admin_or_owner(state, team_id, user_id).await
}

fn ensure_group_is_syncable(group: &GroupRow) -> ApiResult<()> {
    if group.source == GROUP_SOURCE_LDAP || group.source.starts_with(GROUP_SOURCE_PLUGIN_PREFIX) {
        return Ok(());
    }

    Err(AppError::BadRequest(
        "Only LDAP or plugin groups can be linked to syncables".to_string(),
    ))
}

async fn verify_link_unlink_permission(
    state: &AppState,
    auth: &crate::api::v4::extractors::MmAuthUser,
    group: &GroupRow,
    kind: SyncableKind,
    syncable_id: Uuid,
) -> ApiResult<()> {
    if can_manage_system_groups(auth) {
        return Ok(());
    }

    // Non-system group managers can only link referenceable groups.
    if !group.allow_reference {
        return Err(AppError::Forbidden(
            "Insufficient permissions to link non-referenceable group".to_string(),
        ));
    }

    match kind {
        SyncableKind::Team => {
            let is_team_admin = is_team_admin_or_owner(state, syncable_id, auth.user_id).await?;
            if !is_team_admin {
                return Err(AppError::Forbidden(
                    "Insufficient permissions to link group to team".to_string(),
                ));
            }
        }
        SyncableKind::Channel => {
            let can_manage = can_manage_channel_syncable(state, syncable_id, auth.user_id).await?;
            if !can_manage {
                return Err(AppError::Forbidden(
                    "Insufficient permissions to link group to channel".to_string(),
                ));
            }
        }
    }

    Ok(())
}

async fn ensure_syncable_exists(
    state: &AppState,
    kind: SyncableKind,
    syncable_id: Uuid,
) -> ApiResult<()> {
    match kind {
        SyncableKind::Team => {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM teams WHERE id = $1 AND deleted_at IS NULL)",
            )
            .bind(syncable_id)
            .fetch_one(&state.db)
            .await?;
            if !exists {
                return Err(AppError::NotFound("Team not found".to_string()));
            }
        }
        SyncableKind::Channel => {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM channels WHERE id = $1 AND deleted_at IS NULL)",
            )
            .bind(syncable_id)
            .fetch_one(&state.db)
            .await?;
            if !exists {
                return Err(AppError::NotFound("Channel not found".to_string()));
            }
        }
    }

    Ok(())
}

async fn syncable_payload(
    state: &AppState,
    row: &GroupSyncableRow,
    kind: SyncableKind,
) -> ApiResult<Value> {
    let mut payload = json!({
        "group_id": encode_mm_id(row.group_id),
        "auto_add": row.auto_add,
        "scheme_admin": row.scheme_admin,
        "create_at": ts_millis(row.create_at),
        "update_at": ts_millis(row.update_at),
        "delete_at": row.delete_at.map(ts_millis).unwrap_or(0),
        "type": kind.as_mm_type(),
    });

    match kind {
        SyncableKind::Team => {
            let team: TeamMetaRow = sqlx::query_as(
                r#"
                SELECT id, name, display_name, is_public
                FROM teams
                WHERE id = $1
                "#,
            )
            .bind(row.syncable_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Team not found".to_string()))?;

            payload["team_id"] = json!(encode_mm_id(team.id));
            payload["team_display_name"] = json!(team.display_name.unwrap_or(team.name));
            payload["team_type"] = json!(team_type_value(team.is_public));
        }
        SyncableKind::Channel => {
            let channel: ChannelMetaRow = sqlx::query_as(
                r#"
                SELECT
                    c.id,
                    c.name,
                    c.display_name,
                    c.type as channel_type,
                    t.id as team_id,
                    t.name as team_name,
                    t.display_name as team_display_name,
                    t.is_public as team_is_public
                FROM channels c
                JOIN teams t ON t.id = c.team_id
                WHERE c.id = $1
                "#,
            )
            .bind(row.syncable_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

            payload["channel_id"] = json!(encode_mm_id(channel.id));
            payload["channel_display_name"] = json!(channel.display_name.unwrap_or(channel.name));
            payload["channel_type"] = json!(channel_type_value(channel.channel_type));
            payload["team_id"] = json!(encode_mm_id(channel.team_id));
            payload["team_display_name"] =
                json!(channel.team_display_name.unwrap_or(channel.team_name));
            payload["team_type"] = json!(team_type_value(channel.team_is_public));
        }
    }

    Ok(payload)
}

fn parse_user_ids(user_ids: &[String]) -> ApiResult<Vec<Uuid>> {
    let mut parsed = Vec::with_capacity(user_ids.len());
    for user_id in user_ids {
        let uuid = parse_mm_or_uuid(user_id)
            .ok_or_else(|| AppError::BadRequest(format!("Invalid user_id: {user_id}")))?;
        parsed.push(uuid);
    }
    Ok(parsed)
}

async fn load_group_syncables(
    state: &AppState,
    group_id: Uuid,
) -> ApiResult<Vec<GroupSyncableRow>> {
    let rows: Vec<GroupSyncableRow> = sqlx::query_as(
        r#"
        SELECT group_id, syncable_type, syncable_id, auto_add, scheme_admin, create_at, update_at, delete_at
        FROM group_syncables
        WHERE group_id = $1
          AND delete_at IS NULL
        ORDER BY create_at ASC
        "#,
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows)
}

fn spawn_reconcile_syncable(
    state: AppState,
    group_id: Uuid,
    kind: SyncableKind,
    syncable_id: Uuid,
) {
    tokio::spawn(async move {
        if let Err(err) = reconcile_group_syncable(&state, group_id, kind, syncable_id).await {
            tracing::warn!(
                group_id = %group_id,
                syncable_id = %syncable_id,
                syncable_type = %kind.as_db_str(),
                error = %err,
                "Group syncable reconciliation failed"
            );
        }
    });
}

fn spawn_reconcile_group_syncables(state: AppState, group_id: Uuid) {
    tokio::spawn(async move {
        let syncables = match load_group_syncables(&state, group_id).await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::warn!(group_id = %group_id, error = %err, "Failed to load group syncables for reconcile");
                return;
            }
        };

        for row in syncables {
            let kind = if row.syncable_type == "team" {
                SyncableKind::Team
            } else {
                SyncableKind::Channel
            };
            if let Err(err) =
                reconcile_group_syncable(&state, row.group_id, kind, row.syncable_id).await
            {
                tracing::warn!(
                    group_id = %row.group_id,
                    syncable_id = %row.syncable_id,
                    syncable_type = %row.syncable_type,
                    error = %err,
                    "Group syncable reconciliation failed"
                );
            }
        }
    });
}

async fn cleanup_tracking_membership(
    state: &AppState,
    group_id: Uuid,
    kind: SyncableKind,
    syncable_id: Uuid,
    tracked: &TrackedMembershipRow,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        DELETE FROM group_syncable_memberships
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
          AND target_type = $4
          AND target_id = $5
          AND user_id = $6
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .bind(&tracked.target_type)
    .bind(tracked.target_id)
    .bind(tracked.user_id)
    .execute(&state.db)
    .await?;

    let kept_by_other_syncable: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM group_syncable_memberships
            WHERE target_type = $1
              AND target_id = $2
              AND user_id = $3
        )
        "#,
    )
    .bind(&tracked.target_type)
    .bind(tracked.target_id)
    .bind(tracked.user_id)
    .fetch_one(&state.db)
    .await?;

    if kept_by_other_syncable {
        return Ok(());
    }

    if tracked.target_type == "team" {
        sqlx::query("DELETE FROM team_members WHERE team_id = $1 AND user_id = $2")
            .bind(tracked.target_id)
            .bind(tracked.user_id)
            .execute(&state.db)
            .await?;
    } else {
        sqlx::query("DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(tracked.target_id)
            .bind(tracked.user_id)
            .execute(&state.db)
            .await?;
    }

    Ok(())
}

async fn ensure_membership(
    state: &AppState,
    target_type: &str,
    target_id: Uuid,
    user_id: Uuid,
    scheme_admin: bool,
) -> ApiResult<bool> {
    let role = if scheme_admin { "admin" } else { "member" };

    let rows_affected = if target_type == "team" {
        sqlx::query("INSERT INTO team_members (team_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT (team_id, user_id) DO NOTHING")
            .bind(target_id)
            .bind(user_id)
            .bind(role)
            .execute(&state.db)
            .await?
            .rows_affected()
    } else {
        sqlx::query("INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, $3) ON CONFLICT (channel_id, user_id) DO NOTHING")
            .bind(target_id)
            .bind(user_id)
            .bind(role)
            .execute(&state.db)
            .await?
            .rows_affected()
    };

    Ok(rows_affected > 0)
}

async fn reconcile_group_syncable(
    state: &AppState,
    group_id: Uuid,
    kind: SyncableKind,
    syncable_id: Uuid,
) -> ApiResult<()> {
    let syncable: Option<GroupSyncableRow> = sqlx::query_as(
        r#"
        SELECT group_id, syncable_type, syncable_id, auto_add, scheme_admin, create_at, update_at, delete_at
        FROM group_syncables
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
          AND delete_at IS NULL
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(syncable) = syncable else {
        return Ok(());
    };

    let group_user_ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT user_id FROM group_members WHERE group_id = $1")
            .bind(group_id)
            .fetch_all(&state.db)
            .await?;

    let mut desired = HashSet::new();

    if syncable.auto_add {
        match kind {
            SyncableKind::Team => {
                for user_id in &group_user_ids {
                    desired.insert(DesiredMembership {
                        target_type: "team".to_string(),
                        target_id: syncable_id,
                        user_id: *user_id,
                    });
                }
            }
            SyncableKind::Channel => {
                let channel_team_id: Uuid =
                    sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
                        .bind(syncable_id)
                        .fetch_optional(&state.db)
                        .await?
                        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

                for user_id in &group_user_ids {
                    desired.insert(DesiredMembership {
                        target_type: "team".to_string(),
                        target_id: channel_team_id,
                        user_id: *user_id,
                    });
                    desired.insert(DesiredMembership {
                        target_type: "channel".to_string(),
                        target_id: syncable_id,
                        user_id: *user_id,
                    });
                }
            }
        }
    }

    let existing_tracked: Vec<TrackedMembershipRow> = sqlx::query_as(
        r#"
        SELECT target_type, target_id, user_id
        FROM group_syncable_memberships
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .fetch_all(&state.db)
    .await?;

    let mut tracked_set: HashSet<TrackedMembershipRow> = existing_tracked.iter().cloned().collect();

    for desired_membership in &desired {
        let key = TrackedMembershipRow {
            target_type: desired_membership.target_type.clone(),
            target_id: desired_membership.target_id,
            user_id: desired_membership.user_id,
        };

        if tracked_set.contains(&key) {
            continue;
        }

        let inserted = ensure_membership(
            state,
            &desired_membership.target_type,
            desired_membership.target_id,
            desired_membership.user_id,
            syncable.scheme_admin,
        )
        .await?;

        if inserted {
            sqlx::query(
                r#"
                INSERT INTO group_syncable_memberships
                    (group_id, syncable_type, syncable_id, target_type, target_id, user_id)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(group_id)
            .bind(kind.as_db_str())
            .bind(syncable_id)
            .bind(&desired_membership.target_type)
            .bind(desired_membership.target_id)
            .bind(desired_membership.user_id)
            .execute(&state.db)
            .await?;

            tracked_set.insert(key);
        }
    }

    for tracked in existing_tracked {
        let desired_key = DesiredMembership {
            target_type: tracked.target_type.clone(),
            target_id: tracked.target_id,
            user_id: tracked.user_id,
        };

        if !desired.contains(&desired_key) {
            cleanup_tracking_membership(state, group_id, kind, syncable_id, &tracked).await?;
        }
    }

    Ok(())
}

async fn cleanup_unlinked_syncable(
    state: &AppState,
    group_id: Uuid,
    kind: SyncableKind,
    syncable_id: Uuid,
) -> ApiResult<()> {
    let tracked_rows: Vec<TrackedMembershipRow> = sqlx::query_as(
        r#"
        SELECT target_type, target_id, user_id
        FROM group_syncable_memberships
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .fetch_all(&state.db)
    .await?;

    for tracked in tracked_rows {
        cleanup_tracking_membership(state, group_id, kind, syncable_id, &tracked).await?;
    }

    Ok(())
}

/// GET /api/v4/groups
async fn get_groups(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<Json<Vec<Value>>> {
    require_system_groups_read(&auth)?;

    let groups: Vec<GroupListRow> = sqlx::query_as(
        r#"
        SELECT
            g.id,
            g.name,
            g.display_name,
            g.description,
            g.source,
            g.remote_id,
            g.allow_reference,
            g.created_at,
            g.updated_at,
            g.deleted_at,
            EXISTS(
                SELECT 1
                FROM group_syncables gs
                WHERE gs.group_id = g.id
                  AND gs.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = g.id
            ) AS member_count
        FROM groups g
        WHERE g.deleted_at IS NULL
        ORDER BY g.display_name ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(groups.iter().map(group_json).collect()))
}

/// POST /api/v4/groups
async fn create_group(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Json(group): Json<CreateGroupRequest>,
) -> ApiResult<(axum::http::StatusCode, Json<Value>)> {
    require_system_groups_write(&auth)?;

    let source = group
        .source
        .as_deref()
        .unwrap_or(GROUP_SOURCE_CUSTOM)
        .to_ascii_lowercase();

    if source != GROUP_SOURCE_CUSTOM {
        return Err(AppError::BadRequest(
            "Only custom groups can be created from this endpoint".to_string(),
        ));
    }

    let allow_reference = group.allow_reference.unwrap_or(true);
    if !allow_reference {
        return Err(AppError::BadRequest(
            "Custom groups must allow references".to_string(),
        ));
    }

    if group
        .remote_id
        .as_ref()
        .is_some_and(|remote_id| !remote_id.is_empty())
    {
        return Err(AppError::BadRequest(
            "Custom groups cannot have remote_id".to_string(),
        ));
    }

    let display_name = group
        .display_name
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest("display_name is required".to_string()))?;

    let user_ids = parse_user_ids(group.user_ids.as_deref().unwrap_or(&[]))?;

    let mut tx = state.db.begin().await?;

    let created: GroupRow = sqlx::query_as(
        r#"
        INSERT INTO groups (name, display_name, description, source, remote_id, allow_reference)
        VALUES ($1, $2, $3, $4, NULL, $5)
        RETURNING id, name, display_name, description, source, remote_id, allow_reference, created_at, updated_at, deleted_at
        "#,
    )
    .bind(group.name.as_deref().map(str::trim).filter(|value| !value.is_empty()))
    .bind(display_name)
    .bind(group.description.unwrap_or_default())
    .bind(source)
    .bind(allow_reference)
    .fetch_one(&mut *tx)
    .await?;

    for user_id in user_ids {
        sqlx::query(
            "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT (group_id, user_id) DO NOTHING",
        )
        .bind(created.id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let row = GroupListRow {
        id: created.id,
        name: created.name,
        display_name: created.display_name,
        description: created.description,
        source: created.source,
        remote_id: created.remote_id,
        allow_reference: created.allow_reference,
        created_at: created.created_at,
        updated_at: created.updated_at,
        deleted_at: created.deleted_at,
        has_syncables: false,
        member_count: i64::from(group.user_ids.as_ref().map(Vec::len).unwrap_or(0) as i32),
    };

    Ok((axum::http::StatusCode::CREATED, Json(group_json(&row))))
}

/// GET /api/v4/groups/{group_id}
async fn get_group(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
) -> ApiResult<Json<Value>> {
    require_system_groups_read(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;

    let group: GroupListRow = sqlx::query_as(
        r#"
        SELECT
            g.id,
            g.name,
            g.display_name,
            g.description,
            g.source,
            g.remote_id,
            g.allow_reference,
            g.created_at,
            g.updated_at,
            g.deleted_at,
            EXISTS(
                SELECT 1
                FROM group_syncables gs
                WHERE gs.group_id = g.id
                  AND gs.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = g.id
            ) AS member_count
        FROM groups g
        WHERE g.id = $1
        "#,
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    Ok(Json(group_json(&group)))
}

/// PUT /api/v4/groups/{group_id}/patch
async fn patch_group(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
    Json(patch): Json<PatchGroupRequest>,
) -> ApiResult<Json<Value>> {
    require_system_groups_write(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;

    let current: GroupRow = sqlx::query_as(
        r#"
        SELECT id, name, display_name, description, source, remote_id, allow_reference, created_at, updated_at, deleted_at
        FROM groups
        WHERE id = $1
        "#,
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    if current.source == GROUP_SOURCE_CUSTOM && patch.allow_reference == Some(false) {
        return Err(AppError::BadRequest(
            "Custom groups must allow references".to_string(),
        ));
    }

    let updated: GroupListRow = sqlx::query_as(
        r#"
        UPDATE groups
        SET
            name = COALESCE($2, name),
            display_name = COALESCE($3, display_name),
            description = COALESCE($4, description),
            allow_reference = COALESCE($5, allow_reference),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            display_name,
            description,
            source,
            remote_id,
            allow_reference,
            created_at,
            updated_at,
            deleted_at,
            EXISTS(
                SELECT 1
                FROM group_syncables gs
                WHERE gs.group_id = groups.id
                  AND gs.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = groups.id
            ) AS member_count
        "#,
    )
    .bind(group_id)
    .bind(
        patch
            .name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(
        patch
            .display_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    )
    .bind(patch.description)
    .bind(patch.allow_reference)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    Ok(Json(group_json(&updated)))
}

/// DELETE /api/v4/groups/{group_id}
async fn delete_group(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
) -> ApiResult<Json<Value>> {
    require_system_groups_write(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;

    let affected =
        sqlx::query("UPDATE groups SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(group_id)
            .execute(&state.db)
            .await?
            .rows_affected();

    if affected == 0 {
        return Err(AppError::NotFound("Group not found".to_string()));
    }

    Ok(Json(json!({"status": "OK"})))
}

/// POST /api/v4/groups/{group_id}/restore
async fn restore_group(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
) -> ApiResult<Json<Value>> {
    require_system_groups_write(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;

    let group: GroupListRow = sqlx::query_as(
        r#"
        UPDATE groups
        SET deleted_at = NULL, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            display_name,
            description,
            source,
            remote_id,
            allow_reference,
            created_at,
            updated_at,
            deleted_at,
            EXISTS(
                SELECT 1
                FROM group_syncables gs
                WHERE gs.group_id = groups.id
                  AND gs.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = groups.id
            ) AS member_count
        "#,
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    Ok(Json(group_json(&group)))
}

/// POST /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}/link
async fn link_group_syncable(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path((group_id, syncable_type, syncable_id)): Path<(String, String, String)>,
    Json(patch): Json<GroupSyncablePatch>,
) -> ApiResult<(axum::http::StatusCode, Json<Value>)> {
    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let syncable_id = parse_mm_or_uuid(&syncable_id)
        .ok_or_else(|| AppError::BadRequest("Invalid syncable_id".to_string()))?;
    let kind = SyncableKind::parse(&syncable_type)?;

    let group = fetch_group_for_syncable(&state, group_id).await?;
    ensure_group_is_syncable(&group)?;
    ensure_syncable_exists(&state, kind, syncable_id).await?;
    verify_link_unlink_permission(&state, &auth, &group, kind, syncable_id).await?;

    let syncable: GroupSyncableRow = sqlx::query_as(
        r#"
        INSERT INTO group_syncables (group_id, syncable_type, syncable_id, auto_add, scheme_admin, delete_at)
        VALUES ($1, $2, $3, $4, $5, NULL)
        ON CONFLICT (group_id, syncable_type, syncable_id)
        DO UPDATE SET
            auto_add = EXCLUDED.auto_add,
            scheme_admin = EXCLUDED.scheme_admin,
            update_at = NOW(),
            delete_at = NULL
        RETURNING group_id, syncable_type, syncable_id, auto_add, scheme_admin, create_at, update_at, delete_at
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .bind(patch.auto_add.unwrap_or(false))
    .bind(patch.scheme_admin.unwrap_or(false))
    .fetch_one(&state.db)
    .await?;

    spawn_reconcile_syncable(state.clone(), group_id, kind, syncable_id);

    Ok((
        axum::http::StatusCode::CREATED,
        Json(syncable_payload(&state, &syncable, kind).await?),
    ))
}

/// DELETE /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}/link
async fn unlink_group_syncable(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path((group_id, syncable_type, syncable_id)): Path<(String, String, String)>,
) -> ApiResult<Json<Value>> {
    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let syncable_id = parse_mm_or_uuid(&syncable_id)
        .ok_or_else(|| AppError::BadRequest("Invalid syncable_id".to_string()))?;
    let kind = SyncableKind::parse(&syncable_type)?;
    let group = fetch_group_for_syncable(&state, group_id).await?;
    ensure_group_is_syncable(&group)?;
    ensure_syncable_exists(&state, kind, syncable_id).await?;
    verify_link_unlink_permission(&state, &auth, &group, kind, syncable_id).await?;

    let deleted = sqlx::query(
        r#"
        DELETE FROM group_syncables
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if deleted == 0 {
        return Err(AppError::NotFound("Group syncable not found".to_string()));
    }

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(err) = cleanup_unlinked_syncable(&state_clone, group_id, kind, syncable_id).await
        {
            tracing::warn!(
                group_id = %group_id,
                syncable_id = %syncable_id,
                syncable_type = %kind.as_db_str(),
                error = %err,
                "Group syncable unlink cleanup failed"
            );
        }
    });

    Ok(Json(json!({"status": "OK"})))
}

/// GET /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}
async fn get_group_syncable(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path((group_id, syncable_type, syncable_id)): Path<(String, String, String)>,
) -> ApiResult<Json<Value>> {
    require_system_groups_read(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let syncable_id = parse_mm_or_uuid(&syncable_id)
        .ok_or_else(|| AppError::BadRequest("Invalid syncable_id".to_string()))?;
    let kind = SyncableKind::parse(&syncable_type)?;

    let row: GroupSyncableRow = sqlx::query_as(
        r#"
        SELECT group_id, syncable_type, syncable_id, auto_add, scheme_admin, create_at, update_at, delete_at
        FROM group_syncables
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
          AND delete_at IS NULL
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group syncable not found".to_string()))?;

    Ok(Json(syncable_payload(&state, &row, kind).await?))
}

/// GET /api/v4/groups/{group_id}/{syncable_type}
async fn get_group_syncables(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path((group_id, syncable_type)): Path<(String, String)>,
) -> ApiResult<Json<Vec<Value>>> {
    require_system_groups_read(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let kind = SyncableKind::parse(&syncable_type)?;

    let rows: Vec<GroupSyncableRow> = sqlx::query_as(
        r#"
        SELECT group_id, syncable_type, syncable_id, auto_add, scheme_admin, create_at, update_at, delete_at
        FROM group_syncables
        WHERE group_id = $1
          AND syncable_type = $2
          AND delete_at IS NULL
        ORDER BY create_at ASC
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .fetch_all(&state.db)
    .await?;

    let mut response = Vec::with_capacity(rows.len());
    for row in rows {
        response.push(syncable_payload(&state, &row, kind).await?);
    }

    Ok(Json(response))
}

/// PUT /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}/patch
async fn patch_group_syncable(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path((group_id, syncable_type, syncable_id)): Path<(String, String, String)>,
    Json(patch): Json<GroupSyncablePatch>,
) -> ApiResult<Json<Value>> {
    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let syncable_id = parse_mm_or_uuid(&syncable_id)
        .ok_or_else(|| AppError::BadRequest("Invalid syncable_id".to_string()))?;
    let kind = SyncableKind::parse(&syncable_type)?;
    let group = fetch_group_for_syncable(&state, group_id).await?;
    ensure_group_is_syncable(&group)?;
    ensure_syncable_exists(&state, kind, syncable_id).await?;
    verify_link_unlink_permission(&state, &auth, &group, kind, syncable_id).await?;

    let row: GroupSyncableRow = sqlx::query_as(
        r#"
        UPDATE group_syncables
        SET
            auto_add = COALESCE($4, auto_add),
            scheme_admin = COALESCE($5, scheme_admin),
            update_at = NOW()
        WHERE group_id = $1
          AND syncable_type = $2
          AND syncable_id = $3
          AND delete_at IS NULL
        RETURNING group_id, syncable_type, syncable_id, auto_add, scheme_admin, create_at, update_at, delete_at
        "#,
    )
    .bind(group_id)
    .bind(kind.as_db_str())
    .bind(syncable_id)
    .bind(patch.auto_add)
    .bind(patch.scheme_admin)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Group syncable not found".to_string()))?;

    spawn_reconcile_syncable(state.clone(), group_id, kind, syncable_id);

    Ok(Json(syncable_payload(&state, &row, kind).await?))
}

/// GET /api/v4/groups/{group_id}/stats
async fn get_group_stats(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
) -> ApiResult<Json<Value>> {
    require_system_groups_read(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM group_members WHERE group_id = $1")
        .bind(group_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "group_id": encode_mm_id(group_id),
        "total_member_count": count,
    })))
}

/// GET /api/v4/groups/{group_id}/members
async fn get_group_members(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
) -> ApiResult<Json<Value>> {
    require_system_groups_read(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;

    let rows: Vec<(Uuid, DateTime<Utc>)> = sqlx::query_as(
        "SELECT user_id, created_at FROM group_members WHERE group_id = $1 ORDER BY created_at ASC",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await?;

    let members: Vec<Value> = rows
        .iter()
        .map(|(user_id, created_at)| group_member_json(group_id, *user_id, *created_at, 0))
        .collect();

    Ok(Json(json!({
        "members": members,
        "total_member_count": members.len(),
    })))
}

/// POST /api/v4/groups/{group_id}/members
async fn add_group_members(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
    Json(members): Json<GroupModifyMembersRequest>,
) -> ApiResult<(axum::http::StatusCode, Json<Vec<Value>>)> {
    require_system_groups_write(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let user_ids = parse_user_ids(&members.user_ids)?;

    fetch_group_for_syncable(&state, group_id).await?;

    let mut added = Vec::new();
    for user_id in user_ids {
        let inserted: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            INSERT INTO group_members (group_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (group_id, user_id) DO NOTHING
            RETURNING created_at
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(created_at) = inserted {
            added.push(group_member_json(group_id, user_id, created_at, 0));
        }
    }

    spawn_reconcile_group_syncables(state.clone(), group_id);

    Ok((axum::http::StatusCode::CREATED, Json(added)))
}

/// DELETE /api/v4/groups/{group_id}/members
async fn delete_group_members(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Path(group_id): Path<String>,
    Json(members): Json<GroupModifyMembersRequest>,
) -> ApiResult<Json<Vec<Value>>> {
    require_system_groups_write(&auth)?;

    let group_id = parse_mm_or_uuid(&group_id)
        .ok_or_else(|| AppError::BadRequest("Invalid group_id".to_string()))?;
    let user_ids = parse_user_ids(&members.user_ids)?;

    fetch_group_for_syncable(&state, group_id).await?;

    let mut deleted = Vec::new();
    let now_ms = Utc::now().timestamp_millis();
    for user_id in user_ids {
        let deleted_row: Option<DateTime<Utc>> = sqlx::query_scalar(
            r#"
            DELETE FROM group_members
            WHERE group_id = $1
              AND user_id = $2
            RETURNING created_at
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(created_at) = deleted_row {
            deleted.push(group_member_json(group_id, user_id, created_at, now_ms));
        }
    }

    spawn_reconcile_group_syncables(state.clone(), group_id);

    Ok(Json(deleted))
}

/// POST /api/v4/groups/names
async fn get_groups_by_names(
    State(state): State<AppState>,
    auth: crate::api::v4::extractors::MmAuthUser,
    Json(names): Json<Vec<String>>,
) -> ApiResult<Json<Vec<Value>>> {
    require_system_groups_read(&auth)?;

    if names.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let rows: Vec<GroupListRow> = sqlx::query_as(
        r#"
        SELECT
            g.id,
            g.name,
            g.display_name,
            g.description,
            g.source,
            g.remote_id,
            g.allow_reference,
            g.created_at,
            g.updated_at,
            g.deleted_at,
            EXISTS(
                SELECT 1
                FROM group_syncables gs
                WHERE gs.group_id = g.id
                  AND gs.delete_at IS NULL
            ) AS has_syncables,
            (
                SELECT COUNT(*)
                FROM group_members gm
                WHERE gm.group_id = g.id
            ) AS member_count
        FROM groups g
        WHERE g.deleted_at IS NULL
          AND g.name = ANY($1)
        ORDER BY g.display_name ASC
        "#,
    )
    .bind(names)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows.iter().map(group_json).collect()))
}
