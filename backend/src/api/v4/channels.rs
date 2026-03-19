use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::de::DeserializeOwned;

use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::extractors::MmAuthUser;
use crate::api::v4::posts::reactions_for_posts;
use crate::api::AppState;
use crate::auth::policy::permissions;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::{post::PostResponse, Channel};
use serde_json::json;

mod compat;
mod helpers;
mod view;

use compat::{
    get_channel_access_control_attributes, get_channel_common_teams, get_channel_groups,
    get_channel_member_counts_by_group, get_channel_members_minus_group_members,
    get_channel_moderations, patch_channel_moderations, search_group_channels,
    update_channel_scheme,
};
use helpers::normalize_notify_props;
use view::{view_channel, view_channel_for_user};

const KEYCLOAK_GROUP_SOURCE: &str = "plugin_keycloak";

/// Verify that the caller is either a system admin or a channel admin.
/// Returns the membership row on success so callers can avoid a second query.
async fn ensure_channel_admin_or_system_manage(
    state: &AppState,
    channel_id: Uuid,
    auth: &MmAuthUser,
) -> ApiResult<()> {
    if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Ok(());
    }
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM channel_members WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?;

    match role.as_deref() {
        Some("admin") | Some("channel_admin") | Some("team_admin") => Ok(()),
        Some(_) => Err(AppError::Forbidden(
            "Channel admin privileges required".to_string(),
        )),
        None => Err(AppError::Forbidden(
            "Not a member of this channel".to_string(),
        )),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/channels/{channel_id}/posts", get(get_posts))
        .route(
            "/channels/{channel_id}",
            get(get_channel).put(update_channel).delete(delete_channel),
        )
        .route("/channels/{channel_id}/patch", put(patch_channel))
        .route(
            "/channels/{channel_id}/privacy",
            put(update_channel_privacy),
        )
        .route("/channels/{channel_id}/restore", post(restore_channel))
        .route("/channels/{channel_id}/move", post(move_channel))
        .route(
            "/channels/{channel_id}/members",
            get(get_channel_members).post(add_channel_member),
        )
        .route(
            "/channels/{channel_id}/members/me",
            get(get_channel_member_me),
        )
        .route(
            "/channels/{channel_id}/members/ids",
            post(get_channel_members_by_ids),
        )
        .route(
            "/channels/{channel_id}/members/{user_id}",
            get(get_channel_member_by_id).delete(remove_channel_member),
        )
        .route(
            "/channels/{channel_id}/members/{user_id}/roles",
            put(update_channel_member_roles),
        )
        .route(
            "/channels/{channel_id}/members/{user_id}/schemeRoles",
            put(update_channel_member_scheme_roles),
        )
        .route(
            "/channels/{channel_id}/members/{user_id}/notify_props",
            put(update_channel_member_notify_props),
        )
        // Mark as Read / Mark as Unread endpoints
        .route(
            "/channels/{channel_id}/members/{user_id}/read",
            post(mark_channel_as_read),
        )
        .route(
            "/channels/{channel_id}/members/{user_id}/set_unread",
            post(mark_channel_as_unread),
        )
        .route(
            "/channels/{channel_id}/timezones",
            get(get_channel_timezones),
        )
        .route("/channels/{channel_id}/stats", get(get_channel_stats))
        .route("/channels/{channel_id}/unread", get(get_channel_unread))
        .route("/channels/{channel_id}/pinned", get(get_pinned_posts))
        .route("/channels/{channel_id}/posts/{post_id}/pin", post(pin_post))
        .route(
            "/channels/{channel_id}/posts/{post_id}/unpin",
            post(unpin_post),
        )
        .route("/channels/members/me/view", post(view_channel))
        .route(
            "/channels/members/{user_id}/view",
            post(view_channel_for_user),
        )
        .route("/channels/direct", post(create_direct_channel))
        .route("/channels/group", post(create_group_channel))
        .route("/channels", get(get_all_channels).post(create_channel))
        .route("/channels/search", post(search_channels_compat))
        .route("/channels/group/search", post(search_group_channels))
        .route("/channels/{channel_id}/scheme", put(update_channel_scheme))
        .route(
            "/channels/{channel_id}/members_minus_group_members",
            get(get_channel_members_minus_group_members),
        )
        .route(
            "/channels/{channel_id}/member_counts_by_group",
            get(get_channel_member_counts_by_group),
        )
        .route(
            "/channels/{channel_id}/moderations",
            get(get_channel_moderations),
        )
        .route(
            "/channels/{channel_id}/moderations/patch",
            put(patch_channel_moderations),
        )
        .route(
            "/channels/{channel_id}/common_teams",
            get(get_channel_common_teams),
        )
        .route("/channels/{channel_id}/groups", get(get_channel_groups))
        .route(
            "/channels/{channel_id}/access_control/attributes",
            get(get_channel_access_control_attributes),
        )
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u64>,
    per_page: Option<u64>,
    /// Post ID to fetch posts before (for backward pagination)
    before: Option<String>,
    /// Post ID to fetch posts after (for forward pagination)  
    after: Option<String>,
    /// Timestamp in milliseconds to fetch posts since (for incremental sync)
    since: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
struct GetAllChannelsQuery {
    #[serde(default)]
    not_associated_to_group: Option<String>,
    #[serde(default)]
    page: Option<u64>,
    #[serde(default)]
    per_page: Option<u64>,
    #[serde(default)]
    exclude_default_channels: bool,
    #[serde(default)]
    include_deleted: bool,
    #[serde(default)]
    include_total_count: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct ChannelWithTeamDataRow {
    id: Uuid,
    team_id: Uuid,
    #[sqlx(rename = "type")]
    channel_type: crate::models::channel::ChannelType,
    name: String,
    display_name: Option<String>,
    purpose: Option<String>,
    header: Option<String>,
    is_archived: bool,
    creator_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    team_display_name: Option<String>,
    team_name: String,
    team_updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
struct ChannelWithTeamDataResponse {
    #[serde(flatten)]
    channel: mm::Channel,
    team_display_name: String,
    team_name: String,
    team_update_at: i64,
}

fn map_channel_with_team_data_row(row: ChannelWithTeamDataRow) -> ChannelWithTeamDataResponse {
    let team_name = row.team_name;
    let team_display_name = row
        .team_display_name
        .clone()
        .unwrap_or_else(|| team_name.clone());

    let channel = Channel {
        id: row.id,
        team_id: row.team_id,
        channel_type: row.channel_type,
        name: row.name,
        display_name: row.display_name,
        purpose: row.purpose,
        header: row.header,
        is_archived: row.is_archived,
        creator_id: row.creator_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    ChannelWithTeamDataResponse {
        channel: channel.into(),
        team_display_name,
        team_name,
        team_update_at: row.team_updated_at.timestamp_millis(),
    }
}

fn parse_body<T: DeserializeOwned>(
    headers: &axum::http::HeaderMap,
    body: &Bytes,
    message: &str,
) -> ApiResult<T> {
    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.starts_with("application/json") {
        serde_json::from_slice(body)
            .map_err(|_| crate::error::AppError::BadRequest(message.to_string()))
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        serde_urlencoded::from_bytes(body)
            .map_err(|_| crate::error::AppError::BadRequest(message.to_string()))
    } else {
        serde_json::from_slice(body)
            .or_else(|_| serde_urlencoded::from_bytes(body))
            .map_err(|_| crate::error::AppError::BadRequest(message.to_string()))
    }
}

async fn get_all_channels(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Query(query): Query<GetAllChannelsQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    if !auth.has_permission(&permissions::SYSTEM_MANAGE)
        && !auth.has_permission(&permissions::ADMIN_FULL)
    {
        return Err(AppError::Forbidden(
            "Insufficient permissions to list all channels".to_string(),
        ));
    }

    let not_associated_group_id = query
        .not_associated_to_group
        .as_deref()
        .filter(|raw| !raw.trim().is_empty())
        .map(|raw| {
            parse_mm_or_uuid(raw)
                .ok_or_else(|| AppError::BadRequest("Invalid not_associated_to_group".to_string()))
        })
        .transpose()?;

    let page = query.page.unwrap_or(0);
    let mut per_page = query.per_page.unwrap_or(60);
    if per_page == 0 {
        per_page = 60;
    }
    per_page = per_page.min(10_000);
    let offset = page.saturating_mul(per_page) as i64;

    let rows: Vec<ChannelWithTeamDataRow> = sqlx::query_as(
        r#"
        SELECT
            c.id,
            c.team_id,
            c.type,
            c.name,
            c.display_name,
            c.purpose,
            c.header,
            c.is_archived,
            c.creator_id,
            c.created_at,
            c.updated_at,
            t.display_name AS team_display_name,
            t.name AS team_name,
            t.updated_at AS team_updated_at
        FROM channels c
        JOIN teams t ON t.id = c.team_id
        WHERE
            ($1::bool OR c.is_archived = false)
            AND (NOT $2::bool OR c.name NOT IN ('town-square', 'off-topic'))
            AND (
                $3::uuid IS NULL
                OR NOT EXISTS (
                    SELECT 1
                    FROM group_syncables gs
                    WHERE gs.syncable_type = 'channel'
                      AND gs.group_id = $3
                      AND gs.syncable_id = c.id
                )
            )
        ORDER BY c.created_at ASC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(query.include_deleted)
    .bind(query.exclude_default_channels)
    .bind(not_associated_group_id)
    .bind(per_page as i64)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let channels: Vec<ChannelWithTeamDataResponse> = rows
        .into_iter()
        .map(map_channel_with_team_data_row)
        .collect();

    if query.include_total_count {
        let total_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM channels c
            WHERE
                ($1::bool OR c.is_archived = false)
                AND (NOT $2::bool OR c.name NOT IN ('town-square', 'off-topic'))
                AND (
                    $3::uuid IS NULL
                    OR NOT EXISTS (
                        SELECT 1
                        FROM group_syncables gs
                        WHERE gs.syncable_type = 'channel'
                          AND gs.group_id = $3
                          AND gs.syncable_id = c.id
                    )
                )
            "#,
        )
        .bind(query.include_deleted)
        .bind(query.exclude_default_channels)
        .bind(not_associated_group_id)
        .fetch_one(&state.db)
        .await?;

        return Ok(Json(json!({
            "channels": channels,
            "total_count": total_count
        })));
    }

    Ok(Json(json!(channels)))
}

#[derive(sqlx::FromRow, Clone)]
struct ChannelMemberCompatRow {
    channel_id: Uuid,
    user_id: Uuid,
    role: String,
    notify_props: serde_json::Value,
    last_viewed_at: Option<chrono::DateTime<chrono::Utc>>,
    last_update_at: chrono::DateTime<chrono::Utc>,
    msg_count: i64,
    mention_count: i64,
    mention_count_root: i64,
    urgent_mention_count: i64,
    msg_count_root: i64,
}

fn row_to_mm_channel_member(
    row: ChannelMemberCompatRow,
    post_priority_enabled: bool,
) -> mm::ChannelMember {
    let scheme_admin =
        row.role == "admin" || row.role == "team_admin" || row.role == "channel_admin";
    mm::ChannelMember {
        channel_id: encode_mm_id(row.channel_id),
        user_id: encode_mm_id(row.user_id),
        roles: crate::mattermost_compat::mappers::map_channel_role(&row.role),
        last_viewed_at: row
            .last_viewed_at
            .map(|t| t.timestamp_millis())
            .unwrap_or(0),
        msg_count: row.msg_count,
        mention_count: row.mention_count,
        mention_count_root: row.mention_count_root,
        urgent_mention_count: if post_priority_enabled {
            row.urgent_mention_count
        } else {
            0
        },
        msg_count_root: row.msg_count_root,
        notify_props: normalize_notify_props(row.notify_props),
        last_update_at: row.last_update_at.timestamp_millis(),
        scheme_guest: false,
        scheme_user: true,
        scheme_admin,
    }
}

async fn fetch_channel_member_compat_rows(
    state: &AppState,
    channel_id: Uuid,
    user_ids: Option<&[Uuid]>,
) -> ApiResult<Vec<ChannelMemberCompatRow>> {
    if let Some(ids) = user_ids {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query_as(
            r#"
            SELECT
                cm.channel_id,
                cm.user_id,
                cm.role,
                cm.notify_props,
                cm.last_viewed_at,
                COALESCE(cm.last_update_at, cm.created_at) AS last_update_at,
                GREATEST(
                    COUNT(*) FILTER (WHERE p.deleted_at IS NULL)
                    - COUNT(*) FILTER (
                        WHERE p.deleted_at IS NULL
                          AND p.seq > COALESCE(cr.last_read_message_id, 0)
                    ),
                    0
                )::BIGINT AS msg_count,
                COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.seq > COALESCE(cr.last_read_message_id, 0)
                      AND (
                          p.message LIKE '%@' || u.username || '%'
                          OR p.message LIKE '%@all%'
                          OR p.message LIKE '%@channel%'
                      )
                )::BIGINT AS mention_count,
                COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.seq > COALESCE(cr.last_read_message_id, 0)
                      AND p.root_post_id IS NULL
                      AND (
                          p.message LIKE '%@' || u.username || '%'
                          OR p.message LIKE '%@all%'
                          OR p.message LIKE '%@channel%'
                      )
                )::BIGINT AS mention_count_root,
                COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.seq > COALESCE(cr.last_read_message_id, 0)
                      AND (
                          p.message LIKE '%@' || u.username || '%'
                          OR p.message LIKE '%@all%'
                          OR p.message LIKE '%@channel%'
                      )
                      AND p.message LIKE '%@here%'
                )::BIGINT AS urgent_mention_count,
                GREATEST(
                    COUNT(*) FILTER (
                        WHERE p.deleted_at IS NULL
                          AND p.root_post_id IS NULL
                    )
                    - COUNT(*) FILTER (
                        WHERE p.deleted_at IS NULL
                          AND p.root_post_id IS NULL
                          AND p.seq > COALESCE(cr.last_read_message_id, 0)
                    ),
                    0
                )::BIGINT AS msg_count_root
            FROM channel_members cm
            JOIN users u ON u.id = cm.user_id
            LEFT JOIN channel_reads cr
                   ON cr.channel_id = cm.channel_id
                  AND cr.user_id = cm.user_id
            LEFT JOIN posts p
                   ON p.channel_id = cm.channel_id
            WHERE cm.channel_id = $1
              AND cm.user_id = ANY($2)
            GROUP BY
                cm.channel_id,
                cm.user_id,
                cm.role,
                cm.notify_props,
                cm.last_viewed_at,
                cm.last_update_at,
                cm.created_at,
                u.username,
                cr.last_read_message_id
            ORDER BY cm.user_id
            "#,
        )
        .bind(channel_id)
        .bind(ids)
        .fetch_all(&state.db)
        .await?;

        return Ok(rows);
    }

    let rows = sqlx::query_as(
        r#"
        SELECT
            cm.channel_id,
            cm.user_id,
            cm.role,
            cm.notify_props,
            cm.last_viewed_at,
            COALESCE(cm.last_update_at, cm.created_at) AS last_update_at,
            GREATEST(
                COUNT(*) FILTER (WHERE p.deleted_at IS NULL)
                - COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.seq > COALESCE(cr.last_read_message_id, 0)
                ),
                0
            )::BIGINT AS msg_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > COALESCE(cr.last_read_message_id, 0)
                  AND (
                      p.message LIKE '%@' || u.username || '%'
                      OR p.message LIKE '%@all%'
                      OR p.message LIKE '%@channel%'
                  )
            )::BIGINT AS mention_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > COALESCE(cr.last_read_message_id, 0)
                  AND p.root_post_id IS NULL
                  AND (
                      p.message LIKE '%@' || u.username || '%'
                      OR p.message LIKE '%@all%'
                      OR p.message LIKE '%@channel%'
                  )
            )::BIGINT AS mention_count_root,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > COALESCE(cr.last_read_message_id, 0)
                  AND (
                      p.message LIKE '%@' || u.username || '%'
                      OR p.message LIKE '%@all%'
                      OR p.message LIKE '%@channel%'
                  )
                  AND p.message LIKE '%@here%'
            )::BIGINT AS urgent_mention_count,
            GREATEST(
                COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.root_post_id IS NULL
                )
                - COUNT(*) FILTER (
                    WHERE p.deleted_at IS NULL
                      AND p.root_post_id IS NULL
                      AND p.seq > COALESCE(cr.last_read_message_id, 0)
                ),
                0
            )::BIGINT AS msg_count_root
        FROM channel_members cm
        JOIN users u ON u.id = cm.user_id
        LEFT JOIN channel_reads cr
               ON cr.channel_id = cm.channel_id
              AND cr.user_id = cm.user_id
        LEFT JOIN posts p
               ON p.channel_id = cm.channel_id
        WHERE cm.channel_id = $1
        GROUP BY
            cm.channel_id,
            cm.user_id,
            cm.role,
            cm.notify_props,
            cm.last_viewed_at,
            cm.last_update_at,
            cm.created_at,
            u.username,
            cr.last_read_message_id
        ORDER BY cm.user_id
        "#,
    )
    .bind(channel_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows)
}

async fn resolve_direct_channel_display_name(
    state: &AppState,
    channel_id: Uuid,
    viewer_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT COALESCE(NULLIF(u.display_name, ''), u.username)
        FROM channel_members cm
        JOIN users u ON u.id = cm.user_id
        WHERE cm.channel_id = $1
          AND cm.user_id <> $2
        ORDER BY u.username ASC
        LIMIT 1
        "#,
    )
    .bind(channel_id)
    .bind(viewer_id)
    .fetch_optional(&state.db)
    .await
}

async fn get_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let mut channel: crate::models::Channel =
        sqlx::query_as("SELECT * FROM channels WHERE id = $1")
            .bind(channel_id)
            .fetch_one(&state.db)
            .await?;

    // For Direct channels, ALWAYS compute display_name from the other participant
    // This ensures each user sees the other person's name, not their own
    if channel.channel_type == crate::models::channel::ChannelType::Direct {
        channel.display_name =
            resolve_direct_channel_display_name(&state, channel.id, auth.user_id)
                .await?
                .or_else(|| Some("Direct Message".to_string()));
    }

    Ok(Json(channel.into()))
}

/// GET /channels/{channel_id}/unread - Get unread counts for a channel
async fn get_channel_unread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let member: Option<crate::models::ChannelMember> =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?;

    let _member = member.ok_or_else(|| {
        crate::error::AppError::Forbidden("Not a member of this channel".to_string())
    })?;

    let team_id: Uuid = sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_one(&state.db)
        .await?;

    let username: Option<String> = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(auth.user_id)
        .fetch_optional(&state.db)
        .await?;
    let username = username.unwrap_or_default();

    let last_read_message_id: i64 = sqlx::query_scalar(
        "SELECT COALESCE(last_read_message_id, 0) FROM channel_reads WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(0);

    let (msg_count, mention_count, mention_count_root, mut urgent_mention_count, msg_count_root): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
            )::BIGINT AS msg_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND (p.message LIKE '%@' || $3 || '%' OR p.message LIKE '%@all%' OR p.message LIKE '%@channel%')
            )::BIGINT AS mention_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND p.root_post_id IS NULL
                  AND (p.message LIKE '%@' || $3 || '%' OR p.message LIKE '%@all%' OR p.message LIKE '%@channel%')
            )::BIGINT AS mention_count_root,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND (p.message LIKE '%@' || $3 || '%' OR p.message LIKE '%@all%' OR p.message LIKE '%@channel%')
                  AND p.message LIKE '%@here%'
            )::BIGINT AS urgent_mention_count,
            COUNT(*) FILTER (
                WHERE p.deleted_at IS NULL
                  AND p.seq > $2
                  AND p.root_post_id IS NULL
            )::BIGINT AS msg_count_root
        FROM posts p
        WHERE p.channel_id = $1
        "#,
    )
    .bind(channel_id)
    .bind(last_read_message_id)
    .bind(&username)
    .fetch_one(&state.db)
    .await?;

    if !state.config.unread.post_priority_enabled {
        urgent_mention_count = 0;
    }

    Ok(Json(serde_json::json!({
        "team_id": encode_mm_id(team_id),
        "channel_id": encode_mm_id(channel_id),
        "msg_count": msg_count,
        "mention_count": mention_count,
        "mention_count_root": mention_count_root,
        "msg_count_root": msg_count_root,
        "urgent_mention_count": urgent_mention_count
    })))
}

async fn get_channel_members(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<Vec<mm::ChannelMember>>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let rows = fetch_channel_member_compat_rows(&state, channel_id, None).await?;
    let mm_members = rows
        .into_iter()
        .map(|row| row_to_mm_channel_member(row, state.config.unread.post_priority_enabled))
        .collect();

    Ok(Json(mm_members))
}

async fn get_channel_member_me(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::ChannelMember>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let rows = fetch_channel_member_compat_rows(&state, channel_id, Some(&[auth.user_id])).await?;
    let row = rows.into_iter().next().ok_or_else(|| {
        crate::error::AppError::Forbidden("Not a member of this channel".to_string())
    })?;

    Ok(Json(row_to_mm_channel_member(
        row,
        state.config.unread.post_priority_enabled,
    )))
}

async fn get_channel_stats(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::ChannelStats>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(crate::error::AppError::Forbidden(
            "Not a member of this channel".to_string(),
        ));
    }

    let member_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM channel_members WHERE channel_id = $1")
            .bind(channel_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(mm::ChannelStats {
        channel_id: encode_mm_id(channel_id),
        member_count,
    }))
}

async fn get_channel_timezones(
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

#[derive(serde::Deserialize)]
struct DirectChannelRequest {
    user_ids: Vec<String>,
}

async fn create_direct_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    // Mattermost sends either a plain array ["id1", "id2"] or an object {"user_ids": ["id1", "id2"]}
    // Try parsing as plain array first, then fall back to object format
    let user_ids: Vec<String> = serde_json::from_slice::<Vec<String>>(&body).or_else(|_| {
        parse_body::<DirectChannelRequest>(&headers, &body, "Invalid user_ids")
            .map(|req| req.user_ids)
    })?;

    if user_ids.len() != 2 {
        return Err(crate::error::AppError::BadRequest(
            "Request body must contain exactly 2 user IDs".to_string(),
        ));
    }

    let ids: Vec<Uuid> = user_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    if ids.len() != 2 {
        return Err(crate::error::AppError::BadRequest(
            "Invalid user IDs provided".to_string(),
        ));
    }

    if !ids.contains(&auth.user_id) {
        return Err(crate::error::AppError::Forbidden(
            "Must include your user id".to_string(),
        ));
    }

    let other_id = if ids[0] == auth.user_id {
        ids[1]
    } else {
        ids[0]
    };

    let channel = create_direct_channel_internal(&state, auth.user_id, other_id).await?;
    Ok(Json(channel.into()))
}

async fn enforce_dm_acl_for_users(state: &AppState, user_ids: &[Uuid]) -> ApiResult<()> {
    if !state.config.messaging.dm_acl_enabled {
        return Ok(());
    }

    let mut unique_users = user_ids.to_vec();
    unique_users.sort_unstable();
    unique_users.dedup();

    if unique_users.len() < 2 {
        return Ok(());
    }

    let allowed: bool = if unique_users.len() == 2 {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM groups g
                JOIN group_dm_acl_flags gf ON gf.group_id = g.id AND gf.enabled = TRUE
                JOIN group_members gm1 ON gm1.group_id = g.id
                JOIN group_members gm2 ON gm2.group_id = g.id
                WHERE g.deleted_at IS NULL
                  AND g.source = $3
                  AND gm1.user_id = $1
                  AND gm2.user_id = $2
            )
            "#,
        )
        .bind(unique_users[0])
        .bind(unique_users[1])
        .bind(KEYCLOAK_GROUP_SOURCE)
        .fetch_one(&state.db)
        .await?
    } else {
        sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT gm.group_id
                FROM group_members gm
                JOIN groups g ON g.id = gm.group_id
                JOIN group_dm_acl_flags gf ON gf.group_id = g.id AND gf.enabled = TRUE
                WHERE g.deleted_at IS NULL
                  AND g.source = $1
                  AND gm.user_id = ANY($2)
                GROUP BY gm.group_id
                HAVING COUNT(DISTINCT gm.user_id) = $3
            )
            "#,
        )
        .bind(KEYCLOAK_GROUP_SOURCE)
        .bind(&unique_users)
        .bind(unique_users.len() as i64)
        .fetch_one(&state.db)
        .await?
    };

    if !allowed {
        return Err(crate::error::AppError::Forbidden(
            "Direct and group messaging is restricted by group policy".to_string(),
        ));
    }

    Ok(())
}

pub async fn create_direct_channel_internal(
    state: &AppState,
    creator_id: Uuid,
    other_id: Uuid,
) -> ApiResult<crate::models::channel::Channel> {
    enforce_dm_acl_for_users(state, &[creator_id, other_id]).await?;

    let canonical_name = crate::models::canonical_direct_channel_name(creator_id, other_id);
    let legacy_name = crate::models::legacy_direct_channel_name(creator_id, other_id);
    let mut ids = vec![creator_id, other_id];
    ids.sort();

    let team_id: Uuid = sqlx::query_scalar(
        "SELECT team_id FROM team_members WHERE user_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(creator_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::BadRequest("User has no team".to_string()))?;

    let display_name: String = sqlx::query_scalar(
        "SELECT COALESCE(NULLIF(display_name, ''), username) FROM users WHERE id = $1",
    )
    .bind(other_id)
    .fetch_optional(&state.db)
    .await?
    .unwrap_or_else(|| "Direct Message".to_string());

    if let Some(channel) = sqlx::query_as::<_, crate::models::Channel>(
        r#"
        SELECT *
        FROM channels
        WHERE team_id = $1
          AND type = 'direct'::channel_type
          AND (name = $2 OR name = $3)
        ORDER BY created_at ASC
        LIMIT 1
        "#,
    )
    .bind(team_id)
    .bind(&canonical_name)
    .bind(&legacy_name)
    .fetch_optional(&state.db)
    .await?
    {
        for user_id in ids {
            sqlx::query(
                "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
            )
            .bind(channel.id)
            .bind(user_id)
            .execute(&state.db)
            .await?;
        }

        return Ok(channel);
    }

    let channel: crate::models::Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (team_id, type, name, display_name, purpose, header, creator_id)
        VALUES ($1, 'direct', $2, $3, '', '', $4)
        ON CONFLICT (team_id, name) DO UPDATE SET
            name = EXCLUDED.name,
            display_name = CASE
                WHEN channels.display_name IS NULL OR channels.display_name = '' THEN EXCLUDED.display_name
                ELSE channels.display_name
            END
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(&canonical_name)
    .bind(&display_name)
    .bind(creator_id)
    .fetch_one(&state.db)
    .await?;

    for user_id in ids {
        sqlx::query(
            "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
        )
        .bind(channel.id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    }

    Ok(channel)
}

/// POST /channels/group - Create group DM (3+ users)
async fn create_group_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    // Group DMs also use array format
    let input: DirectChannelRequest = parse_body(&headers, &body, "Invalid user_ids")?;

    if input.user_ids.len() < 2 {
        return Err(crate::error::AppError::BadRequest(
            "user_ids must contain at least 2 users".to_string(),
        ));
    }

    let uuids: Vec<Uuid> = input
        .user_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    let channel = create_group_channel_internal(&state, auth.user_id, uuids).await?;
    Ok(Json(channel.into()))
}

pub async fn create_group_channel_internal(
    state: &AppState,
    creator_id: Uuid,
    user_ids: Vec<Uuid>,
) -> ApiResult<crate::models::channel::Channel> {
    let mut ids = user_ids;
    if !ids.contains(&creator_id) {
        ids.push(creator_id);
    }

    ids.sort();
    ids.dedup();
    enforce_dm_acl_for_users(state, &ids).await?;

    let name = format!(
        "gm_{}",
        ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("_")
    );

    let team_id: Uuid = sqlx::query_scalar(
        "SELECT team_id FROM team_members WHERE user_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(creator_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::BadRequest("User has no team".to_string()))?;

    // Generate display name from usernames
    let usernames: Vec<String> =
        sqlx::query_scalar("SELECT username FROM users WHERE id = ANY($1)")
            .bind(&ids)
            .fetch_all(&state.db)
            .await?;
    let display_name = usernames.join(", ");

    let channel: crate::models::Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (team_id, type, name, display_name, purpose, header, creator_id)
        VALUES ($1, 'group', $2, $3, '', '', $4)
        ON CONFLICT (team_id, name) DO UPDATE SET name = EXCLUDED.name
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(&name)
    .bind(&display_name)
    .bind(creator_id)
    .fetch_one(&state.db)
    .await?;

    for user_id in ids {
        sqlx::query(
            "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
        )
        .bind(channel.id)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    }

    Ok(channel)
}

/// POST /channels - Create a new channel
#[derive(serde::Deserialize)]
struct CreateChannelRequest {
    team_id: String,
    name: String,
    display_name: String,
    #[serde(rename = "type", default = "default_channel_type")]
    channel_type: String,
    #[serde(default)]
    purpose: String,
    #[serde(default)]
    header: String,
}

fn default_channel_type() -> String {
    "O".to_string()
}

async fn create_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    let input: CreateChannelRequest = parse_body(&headers, &body, "Invalid channel body")?;

    let team_id = parse_mm_or_uuid(&input.team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    // Verify team membership
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
    )
    .bind(team_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_member {
        return Err(crate::error::AppError::Forbidden(
            "Not a member of this team".to_string(),
        ));
    }

    // Map MM channel type to RustChat type
    let channel_type = match input.channel_type.as_str() {
        "O" => "public",
        "P" => "private",
        _ => "public",
    };

    let channel: Channel = sqlx::query_as(
        r#"
        INSERT INTO channels (team_id, type, name, display_name, purpose, header, creator_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(team_id)
    .bind(channel_type)
    .bind(&input.name)
    .bind(&input.display_name)
    .bind(&input.purpose)
    .bind(&input.header)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    // Add creator as member
    sqlx::query(
        "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'admin') ON CONFLICT DO NOTHING",
    )
    .bind(channel.id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// PUT /channels/{channel_id} - Update channel
#[derive(serde::Deserialize)]
struct UpdateChannelRequest {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    purpose: Option<String>,
    #[serde(default)]
    header: Option<String>,
}

async fn update_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let input: UpdateChannelRequest = parse_body(&headers, &body, "Invalid channel update")?;

    // Build update query dynamically
    let channel: Channel = sqlx::query_as(
        r#"
        UPDATE channels SET
            display_name = COALESCE($2, display_name),
            name = COALESCE($3, name),
            purpose = COALESCE($4, purpose),
            header = COALESCE($5, header),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(channel_id)
    .bind(&input.display_name)
    .bind(&input.name)
    .bind(&input.purpose)
    .bind(&input.header)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// DELETE /channels/{channel_id} - Delete/archive channel
async fn delete_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Only channel admins or system admins may delete a channel
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    // Soft delete the channel
    sqlx::query("UPDATE channels SET deleted_at = NOW() WHERE id = $1")
        .bind(channel_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// GET /channels/{channel_id}/pinned - Get pinned posts
async fn get_pinned_posts(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::PostList>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let mut posts: Vec<PostResponse> = sqlx::query_as(
        r#"
        SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
               p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
               p.reply_count::int8 as reply_count,
               p.last_reply_at, p.seq,
               u.username, u.avatar_url, u.email
        FROM posts p
        LEFT JOIN users u ON p.user_id = u.id
        WHERE p.channel_id = $1 AND p.is_pinned = true AND p.deleted_at IS NULL
        ORDER BY p.created_at DESC
        "#,
    )
    .bind(channel_id)
    .fetch_all(&state.db)
    .await?;

    crate::services::posts::populate_files(&state, &mut posts).await?;

    let mut order = Vec::new();
    let mut posts_map: HashMap<String, mm::Post> = HashMap::new();
    let mut post_ids = Vec::new();
    let mut id_map = Vec::new();

    for p in posts {
        let id = encode_mm_id(p.id);
        post_ids.push(p.id);
        id_map.push((p.id, id.clone()));
        order.push(id.clone());
        posts_map.insert(id, p.into());
    }

    let reactions_map = reactions_for_posts(&state, &post_ids).await?;
    for (post_uuid, post_id) in id_map {
        if let Some(reactions) = reactions_map.get(&post_uuid) {
            if !reactions.is_empty() {
                if let Some(post) = posts_map.get_mut(&post_id) {
                    let mut metadata = post.metadata.take().unwrap_or_else(|| json!({}));
                    if let Some(obj) = metadata.as_object_mut() {
                        obj.insert("reactions".to_string(), json!(reactions));
                    }
                    post.metadata = Some(metadata);
                }
            }
        }
    }

    Ok(Json(mm::PostList {
        order,
        posts: posts_map,
        next_post_id: String::new(),
        prev_post_id: String::new(),
    }))
}

/// Path for pin/unpin operations
#[derive(serde::Deserialize)]
struct PinPath {
    channel_id: String,
    post_id: String,
}

/// POST /channels/{channel_id}/posts/{post_id}/pin - Pin a post
async fn pin_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<PinPath>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let post_id = parse_mm_or_uuid(&path.post_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid post_id".to_string()))?;

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // Pin the post
    sqlx::query("UPDATE posts SET is_pinned = true WHERE id = $1 AND channel_id = $2")
        .bind(post_id)
        .bind(channel_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// POST /channels/{channel_id}/posts/{post_id}/unpin - Unpin a post
async fn unpin_post(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<PinPath>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let post_id = parse_mm_or_uuid(&path.post_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid post_id".to_string()))?;

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // Unpin the post
    sqlx::query("UPDATE posts SET is_pinned = false WHERE id = $1 AND channel_id = $2")
        .bind(post_id)
        .bind(channel_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

#[derive(serde::Deserialize)]
struct AddMemberRequest {
    user_id: String,
}

async fn add_channel_member(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<mm::ChannelMember>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let input: AddMemberRequest = parse_body(&headers, &body, "Invalid member body")?;

    let user_id = parse_mm_or_uuid(&input.user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;

    // Get channel to check type and verify caller can add members
    let channel: Option<Channel> = sqlx::query_as("SELECT * FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(&state.db)
        .await?;
    let channel = channel.ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    // For private channels, only channel admins or system admins may add members.
    // Public channels allow any member to add others.
    if channel.channel_type == crate::models::channel::ChannelType::Private {
        ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;
    } else {
        // For public/direct/group channels, verify caller is a member
        let _caller_member: crate::models::ChannelMember =
            sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
                .bind(channel_id)
                .bind(auth.user_id)
                .fetch_optional(&state.db)
                .await?
                .ok_or_else(|| {
                    crate::error::AppError::Forbidden("Not a member of this channel".to_string())
                })?;
    }

    // Add the user
    sqlx::query(
        "INSERT INTO channel_members (channel_id, user_id, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING",
    )
    .bind(channel_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    let team_id: Option<Uuid> = sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(&state.db)
        .await?;

    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::MemberAdded,
        serde_json::json!({
            "user_id": user_id,
            "channel_id": channel_id,
            "team_id": team_id,
        }),
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: Some(channel_id),
        team_id,
        user_id: Some(user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    let rows = fetch_channel_member_compat_rows(&state, channel_id, Some(&[user_id])).await?;
    let row = rows
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::AppError::NotFound("Member not found".to_string()))?;

    Ok(Json(row_to_mm_channel_member(
        row,
        state.config.unread.post_priority_enabled,
    )))
}

/// DELETE /channels/{channel_id}/members/{user_id} - Remove a member from a channel
#[derive(serde::Deserialize)]
struct ChannelMemberPath {
    channel_id: String,
    user_id: String,
}

#[derive(serde::Deserialize)]
struct ChannelMemberIdsRequest {
    user_ids: Vec<String>,
}

async fn remove_channel_member(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<ChannelMemberPath>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let user_id = parse_mm_or_uuid(&path.user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;

    // Users may remove themselves; to remove others, channel or system admin is required
    if auth.user_id != user_id {
        ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;
    }

    let team_id: Option<Uuid> = sqlx::query_scalar("SELECT team_id FROM channels WHERE id = $1")
        .bind(channel_id)
        .fetch_optional(&state.db)
        .await?;

    // Remove the user
    sqlx::query("DELETE FROM channel_members WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::MemberRemoved,
        serde_json::json!({
            "user_id": user_id,
            "channel_id": channel_id,
            "team_id": team_id,
            "remover_id": auth.user_id,
        }),
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: Some(channel_id),
        team_id,
        user_id: Some(user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn get_channel_member_by_id(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<ChannelMemberPath>,
) -> ApiResult<Json<mm::ChannelMember>> {
    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&path.user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;

    let _caller_member: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let rows = fetch_channel_member_compat_rows(&state, channel_id, Some(&[user_id])).await?;
    let row = rows
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::AppError::NotFound("Member not found".to_string()))?;

    Ok(Json(row_to_mm_channel_member(
        row,
        state.config.unread.post_priority_enabled,
    )))
}

async fn get_channel_members_by_ids(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> ApiResult<Json<Vec<mm::ChannelMember>>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let _caller_member: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let input: ChannelMemberIdsRequest = parse_body(&headers, &body, "Invalid ids body")?;
    if input.user_ids.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let mut user_ids = Vec::new();
    for id in input.user_ids {
        let parsed = parse_mm_or_uuid(&id)
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;
        user_ids.push(parsed);
    }

    let rows = fetch_channel_member_compat_rows(&state, channel_id, Some(&user_ids)).await?;
    let mm_members = rows
        .into_iter()
        .map(|row| row_to_mm_channel_member(row, state.config.unread.post_priority_enabled))
        .collect();

    Ok(Json(mm_members))
}

#[derive(serde::Deserialize)]
struct ChannelMemberRolesRequest {
    roles: String,
}

async fn update_channel_member_roles(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<ChannelMemberPath>,
    Json(input): Json<ChannelMemberRolesRequest>,
) -> ApiResult<Json<mm::ChannelMember>> {
    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&path.user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;

    // Only channel admins or system admins may change member roles
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let role = if input.roles.contains("channel_admin") {
        "channel_admin"
    } else {
        "member"
    };

    sqlx::query("UPDATE channel_members SET role = $1 WHERE channel_id = $2 AND user_id = $3")
        .bind(role)
        .bind(channel_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    let rows = fetch_channel_member_compat_rows(&state, channel_id, Some(&[user_id])).await?;
    let row = rows
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::AppError::NotFound("Member not found".to_string()))?;

    Ok(Json(row_to_mm_channel_member(
        row,
        state.config.unread.post_priority_enabled,
    )))
}

async fn update_channel_member_notify_props(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(path): Path<ChannelMemberPath>,
    Json(input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    let channel_id = parse_mm_or_uuid(&path.channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let user_id = parse_mm_or_uuid(&path.user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;

    if user_id != auth.user_id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot update another user's notify props".to_string(),
        ));
    }

    sqlx::query(
        "UPDATE channel_members SET notify_props = $1 WHERE channel_id = $2 AND user_id = $3",
    )
    .bind(&input)
    .bind(channel_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

async fn get_posts(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<Json<mm::PostList>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Check channel membership first
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let per_page = pagination.per_page.unwrap_or(60).min(200) as i64;

    // Determine query type based on pagination params
    let mut posts: Vec<PostResponse> = if let Some(since) = pagination.since {
        // Incremental sync: get posts created or edited since timestamp
        let since_time =
            chrono::DateTime::from_timestamp_millis(since).unwrap_or_else(chrono::Utc::now);

        sqlx::query_as(
            r#"
            SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
                   p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
                   p.reply_count::int8 as reply_count,
                   p.last_reply_at, p.seq,
                   u.username, u.avatar_url, u.email
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1
              AND p.deleted_at IS NULL
              AND (p.created_at >= $2 OR p.edited_at >= $2)
            ORDER BY p.created_at ASC
            LIMIT $3
            "#,
        )
        .bind(channel_id)
        .bind(since_time)
        .bind(per_page)
        .fetch_all(&state.db)
        .await?
    } else if let Some(before) = &pagination.before {
        // Cursor pagination: get posts before a specific post
        let before_id = parse_mm_or_uuid(before).ok_or_else(|| {
            crate::error::AppError::BadRequest("Invalid before post_id".to_string())
        })?;

        // Get the created_at of the before post
        let before_time: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT created_at FROM posts WHERE id = $1")
                .bind(before_id)
                .fetch_optional(&state.db)
                .await?;

        let before_time = before_time
            .ok_or_else(|| crate::error::AppError::NotFound("Before post not found".to_string()))?;

        sqlx::query_as(
            r#"
            SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
                   p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
                   p.reply_count::int8 as reply_count,
                   p.last_reply_at, p.seq,
                   u.username, u.avatar_url, u.email
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1 
              AND p.deleted_at IS NULL
              AND p.created_at < $2
            ORDER BY p.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(channel_id)
        .bind(before_time)
        .bind(per_page)
        .fetch_all(&state.db)
        .await?
    } else if let Some(after) = &pagination.after {
        // Cursor pagination: get posts after a specific post
        let after_id = parse_mm_or_uuid(after).ok_or_else(|| {
            crate::error::AppError::BadRequest("Invalid after post_id".to_string())
        })?;

        // Get the created_at of the after post
        let after_time: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT created_at FROM posts WHERE id = $1")
                .bind(after_id)
                .fetch_optional(&state.db)
                .await?;

        let after_time = after_time
            .ok_or_else(|| crate::error::AppError::NotFound("After post not found".to_string()))?;

        sqlx::query_as(
            r#"
            SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
                   p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
                   p.reply_count::int8 as reply_count,
                   p.last_reply_at, p.seq,
                   u.username, u.avatar_url, u.email
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1 
              AND p.deleted_at IS NULL
              AND p.created_at > $2
            ORDER BY p.created_at ASC
            LIMIT $3
            "#,
        )
        .bind(channel_id)
        .bind(after_time)
        .bind(per_page)
        .fetch_all(&state.db)
        .await?
    } else {
        // Standard page-based pagination
        let page = pagination.page.unwrap_or(0);
        let offset = (page * per_page as u64) as i64;

        sqlx::query_as(
            r#"
            SELECT p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
                   p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
                   p.reply_count::int8 as reply_count,
                   p.last_reply_at, p.seq,
                   u.username, u.avatar_url, u.email
            FROM posts p
            LEFT JOIN users u ON p.user_id = u.id
            WHERE p.channel_id = $1 AND p.deleted_at IS NULL
            ORDER BY p.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(channel_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    };

    crate::services::posts::populate_files(&state, &mut posts).await?;

    let mut order = Vec::new();
    let mut posts_map: HashMap<String, mm::Post> = HashMap::new();
    let mut post_ids = Vec::new();
    let mut id_map = Vec::new();

    // Determine prev/next post IDs for pagination hints
    let (prev_post_id, next_post_id) = if !posts.is_empty() {
        let first_id = encode_mm_id(posts.first().unwrap().id);
        let last_id = encode_mm_id(posts.last().unwrap().id);
        // If using before/after, provide the opposite cursor
        if pagination.before.is_some() {
            (last_id, String::new())
        } else if pagination.after.is_some() {
            (String::new(), first_id)
        } else {
            (String::new(), String::new())
        }
    } else {
        (String::new(), String::new())
    };

    for p in posts {
        let id = encode_mm_id(p.id);
        post_ids.push(p.id);
        id_map.push((p.id, id.clone()));
        order.push(id.clone());
        posts_map.insert(id, p.into());
    }

    let reactions_map = reactions_for_posts(&state, &post_ids).await?;
    for (post_uuid, post_id) in id_map {
        if let Some(reactions) = reactions_map.get(&post_uuid) {
            if !reactions.is_empty() {
                if let Some(post) = posts_map.get_mut(&post_id) {
                    let mut metadata = post.metadata.take().unwrap_or_else(|| json!({}));
                    if let Some(obj) = metadata.as_object_mut() {
                        obj.insert("reactions".to_string(), json!(reactions));
                    }
                    post.metadata = Some(metadata);
                }
            }
        }
    }

    Ok(Json(mm::PostList {
        order,
        posts: posts_map,
        next_post_id,
        prev_post_id,
    }))
}

async fn search_channels_compat(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(input): Json<HashMap<String, String>>,
) -> ApiResult<Json<Vec<mm::Channel>>> {
    let term = input.get("term").cloned().unwrap_or_default();
    let team_id = input
        .get("team_id")
        .and_then(|s| parse_mm_or_uuid(s));

    // Restrict results to channels the caller is a member of, scoped to team if provided.
    // System admins see all channels without the membership restriction.
    let channels: Vec<Channel> = if auth.has_permission(&permissions::SYSTEM_MANAGE) {
        sqlx::query_as(
            r#"
            SELECT * FROM channels
            WHERE name ILIKE $1
              AND ($2::uuid IS NULL OR team_id = $2)
            ORDER BY name
            LIMIT 100
            "#,
        )
        .bind(format!("%{}%", term))
        .bind(team_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
            SELECT c.* FROM channels c
            JOIN channel_members cm ON cm.channel_id = c.id AND cm.user_id = $2
            WHERE c.name ILIKE $1
              AND ($3::uuid IS NULL OR c.team_id = $3)
            ORDER BY c.name
            LIMIT 100
            "#,
        )
        .bind(format!("%{}%", term))
        .bind(auth.user_id)
        .bind(team_id)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(channels.into_iter().map(|c| c.into()).collect()))
}

/// PUT /channels/{channel_id}/patch - Patch channel (partial update)
#[derive(serde::Deserialize)]
struct PatchChannelRequest {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    purpose: Option<String>,
    #[serde(default)]
    header: Option<String>,
}

async fn patch_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(input): Json<PatchChannelRequest>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    let channel: Channel = sqlx::query_as(
        r#"
        UPDATE channels SET
            display_name = COALESCE($2, display_name),
            name = COALESCE($3, name),
            purpose = COALESCE($4, purpose),
            header = COALESCE($5, header),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(channel_id)
    .bind(&input.display_name)
    .bind(&input.name)
    .bind(&input.purpose)
    .bind(&input.header)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// PUT /channels/{channel_id}/privacy - Update channel privacy
#[derive(serde::Deserialize)]
struct UpdatePrivacyRequest {
    privacy: String, // "O" for public, "P" for private
}

async fn update_channel_privacy(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(input): Json<UpdatePrivacyRequest>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Only channel admins or system admins may change channel privacy
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let channel_type = match input.privacy.as_str() {
        "O" => "public",
        "P" => "private",
        _ => {
            return Err(crate::error::AppError::BadRequest(
                "Invalid privacy value".to_string(),
            ))
        }
    };

    let channel: Channel = sqlx::query_as(
        r#"UPDATE channels SET type = $2, updated_at = NOW() WHERE id = $1 RETURNING *"#,
    )
    .bind(channel_id)
    .bind(channel_type)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// POST /channels/{channel_id}/restore - Restore a deleted channel
async fn restore_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    // Only channel admins or system admins may restore a deleted channel
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    let channel: Channel = sqlx::query_as(
        r#"UPDATE channels SET deleted_at = NULL, updated_at = NOW() WHERE id = $1 RETURNING *"#,
    )
    .bind(channel_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// POST /channels/{channel_id}/move - Move channel to another team
#[derive(serde::Deserialize)]
struct MoveChannelRequest {
    team_id: String,
    #[serde(rename = "force", default)]
    _force: bool,
}

async fn move_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(channel_id): Path<String>,
    Json(input): Json<MoveChannelRequest>,
) -> ApiResult<Json<mm::Channel>> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let new_team_id = parse_mm_or_uuid(&input.team_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid team_id".to_string()))?;

    // Only system admins may move a channel between teams
    if !auth.has_permission(&permissions::SYSTEM_MANAGE) {
        return Err(AppError::Forbidden(
            "System admin privileges required to move channels between teams".to_string(),
        ));
    }

    // Verify membership in new team
    let is_team_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM team_members WHERE team_id = $1 AND user_id = $2)",
    )
    .bind(new_team_id)
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await?;

    if !is_team_member {
        return Err(crate::error::AppError::Forbidden(
            "Not a member of the target team".to_string(),
        ));
    }

    let channel: Channel = sqlx::query_as(
        r#"UPDATE channels SET team_id = $2, updated_at = NOW() WHERE id = $1 RETURNING *"#,
    )
    .bind(channel_id)
    .bind(new_team_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(channel.into()))
}

/// PUT /channels/{channel_id}/members/{user_id}/schemeRoles - Update member scheme roles
#[derive(serde::Deserialize)]
struct UpdateSchemeRolesRequest {
    #[serde(default)]
    scheme_admin: bool,
    #[serde(rename = "scheme_user", default)]
    _scheme_user: bool,
}

async fn update_channel_member_scheme_roles(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, user_id)): Path<(String, String)>,
    Json(input): Json<UpdateSchemeRolesRequest>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;
    let target_user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?;

    // Only channel admins or system admins may change member scheme roles
    ensure_channel_admin_or_system_manage(&state, channel_id, &auth).await?;

    // Update the target user's role based on scheme_admin
    let new_role = if input.scheme_admin {
        "admin"
    } else {
        "member"
    };

    sqlx::query("UPDATE channel_members SET role = $3 WHERE channel_id = $1 AND user_id = $2")
        .bind(channel_id)
        .bind(target_user_id)
        .bind(new_role)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// POST /channels/{channel_id}/members/{user_id}/read - Mark channel as read
async fn mark_channel_as_read(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, user_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let target_user_id = if user_id == "me" {
        auth.user_id
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?
    };

    // Users can only mark their own channels as read
    if target_user_id != auth.user_id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot mark channel as read for other users".to_string(),
        ));
    }

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // Update last_viewed_at to mark all messages as read
    sqlx::query(
        "UPDATE channel_members SET last_viewed_at = NOW(), manually_unread = false, last_update_at = NOW() WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    // Also update channel_reads table
    sqlx::query(
        r#"
        INSERT INTO channel_reads (user_id, channel_id, last_read_message_id, last_read_at)
        VALUES ($1, $2, (SELECT MAX(seq) FROM posts WHERE channel_id = $2), NOW())
        ON CONFLICT (user_id, channel_id)
        DO UPDATE SET last_read_message_id = EXCLUDED.last_read_message_id, last_read_at = NOW()
        "#,
    )
    .bind(auth.user_id)
    .bind(channel_id)
    .execute(&state.db)
    .await?;

    // Broadcast channel viewed event
    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::ChannelViewed,
        serde_json::json!({
            "channel_id": encode_mm_id(channel_id),
        }),
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: None,
        team_id: None,
        user_id: Some(auth.user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// POST /channels/{channel_id}/members/{user_id}/set_unread - Mark channel as unread
/// This sets the last_viewed_at to a past time to create unread state
async fn mark_channel_as_unread(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((channel_id, user_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let channel_id = parse_mm_or_uuid(&channel_id)
        .ok_or_else(|| crate::error::AppError::BadRequest("Invalid channel_id".to_string()))?;

    let target_user_id = if user_id == "me" {
        auth.user_id
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?
    };

    // Users can only mark their own channels as unread
    if target_user_id != auth.user_id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot mark channel as unread for other users".to_string(),
        ));
    }

    // Verify membership
    let _membership: crate::models::ChannelMember =
        sqlx::query_as("SELECT * FROM channel_members WHERE channel_id = $1 AND user_id = $2")
            .bind(channel_id)
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::Forbidden("Not a member of this channel".to_string())
            })?;

    // Get the oldest post in the channel to set as unread point
    // Or use a time far in the past to ensure all messages are marked as unread
    let oldest_post_time: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT MIN(created_at) FROM posts WHERE channel_id = $1 AND deleted_at IS NULL",
    )
    .bind(channel_id)
    .fetch_optional(&state.db)
    .await?;

    // Set last_viewed_at to the oldest post time, or epoch if no posts
    let mark_time = oldest_post_time.unwrap_or(chrono::DateTime::UNIX_EPOCH);

    sqlx::query(
        "UPDATE channel_members SET last_viewed_at = $3, manually_unread = true, last_update_at = NOW() WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(mark_time)
    .execute(&state.db)
    .await?;

    // Also update channel_reads table
    sqlx::query(
        "UPDATE channel_reads SET last_read_message_id = 0, last_read_at = $3 WHERE channel_id = $1 AND user_id = $2",
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .bind(mark_time)
    .execute(&state.db)
    .await?;

    // Broadcast unread update
    let broadcast = crate::realtime::WsEnvelope::event(
        crate::realtime::EventType::ChannelUnread,
        serde_json::json!({
            "channel_id": encode_mm_id(channel_id),
            "user_id": encode_mm_id(auth.user_id),
            "unread_count": 1,
        }),
        Some(channel_id),
    )
    .with_broadcast(crate::realtime::WsBroadcast {
        channel_id: None,
        team_id: None,
        user_id: Some(auth.user_id),
        exclude_user_id: None,
    });
    state.ws_hub.broadcast(broadcast).await;

    Ok(Json(serde_json::json!({
        "channel_id": encode_mm_id(channel_id),
        "user_id": encode_mm_id(auth.user_id),
        "status": "OK"
    })))
}
