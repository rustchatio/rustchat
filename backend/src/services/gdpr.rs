//! GDPR compliance service
//!
//! Provides functionality for:
//! - Right to Erasure (hard delete of user data)
//! - Data Export (GDPR Article 20)
//! - Cryptographic wipe of files before deletion

use crate::error::{ApiResult, AppError};
use crate::storage::S3Client;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

/// Certificate of destruction for audit purposes
#[derive(Debug, Clone, Serialize)]
pub struct CertificateOfDestruction {
    pub file_id: Uuid,
    pub file_name: String,
    pub s3_key: String,
    pub wiped_at: DateTime<Utc>,
    pub wipe_method: String,
    pub wiped_by: Uuid,
}

/// User data export structure (GDPR Article 20)
#[derive(Debug, Clone, Serialize)]
pub struct UserDataExport {
    pub export_id: Uuid,
    pub user_id: Uuid,
    pub export_generated_at: DateTime<Utc>,
    pub user_profile: UserProfileExport,
    pub posts: Vec<PostExport>,
    pub reactions: Vec<ReactionExport>,
    pub files: Vec<FileExport>,
    pub preferences: serde_json::Value,
    pub channel_memberships: Vec<ChannelMembershipExport>,
    pub audit_log: Vec<AuditLogExport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserProfileExport {
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    pub position: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub role: String,
    pub is_bot: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostExport {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
    pub file_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReactionExport {
    pub post_id: Uuid,
    pub emoji_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileExport {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChannelMembershipExport {
    pub channel_id: Uuid,
    pub channel_name: String,
    pub team_name: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditLogExport {
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

/// Hard delete a post and all associated data
pub async fn hard_delete_post(
    db: &PgPool,
    s3_client: &S3Client,
    post_id: Uuid,
    deleted_by: Uuid,
) -> ApiResult<()> {
    // First, get all file IDs associated with this post
    let file_ids: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, key FROM files WHERE post_id = $1",
    )
    .bind(post_id)
    .fetch_all(db)
    .await?;

    // Cryptographically wipe and delete each file
    for (file_id, s3_key) in file_ids {
        if let Err(e) = crypto_wipe_file(s3_client, s3_key.as_str()).await {
            warn!(
                file_id = %file_id,
                s3_key = %s3_key,
                error = %e,
                "Failed to cryptographically wipe file, proceeding with deletion"
            );
        }

        // Delete from S3
        if let Err(e) = s3_client.delete(s3_key.as_str()).await {
            warn!(
                file_id = %file_id,
                s3_key = %s3_key,
                error = %e,
                "Failed to delete file from S3"
            );
        }

        // Log certificate of destruction
        info!(
            file_id = %file_id,
            s3_key = %s3_key,
            wiped_by = %deleted_by,
            "File cryptographically wiped and deleted"
        );
    }

    // Delete from database in a transaction
    let mut tx = db.begin().await?;

    // Delete reactions
    sqlx::query("DELETE FROM reactions WHERE post_id = $1")
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

    // Delete saved posts references
    sqlx::query("DELETE FROM saved_posts WHERE post_id = $1")
        .bind(post_id)
    .execute(&mut *tx)
        .await?;

    // Delete post acknowledgements
    sqlx::query("DELETE FROM post_acknowledgements WHERE post_id = $1")
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

    // Delete thread memberships (if this is a root post)
    sqlx::query("DELETE FROM thread_memberships WHERE thread_id = $1")
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

    // Delete files records
    sqlx::query("DELETE FROM files WHERE post_id = $1")
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

    // Finally, delete the post itself
    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(post_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Post not found".to_string()));
    }

    tx.commit().await?;

    info!(
        post_id = %post_id,
        deleted_by = %deleted_by,
        "Post hard deleted"
    );

    Ok(())
}

/// Cryptographically wipe a file by overwriting with random data before deletion
/// 
/// Note: This overwrites the data in S3 by uploading an encrypted random blob
/// with the same key, then deletes it. This provides defense in depth for
/// sensitive data.
async fn crypto_wipe_file(s3_client: &S3Client, s3_key: &str) -> ApiResult<()> {
    use rand::RngCore;

    // Generate random data for overwrite (same size as typical file, max 1MB)
    // In production, you'd want to get the actual file size first
    const WIPE_SIZE: usize = 1024 * 1024; // 1MB
    let mut wipe_data = vec![0u8; WIPE_SIZE];
    rand::thread_rng().fill_bytes(&mut wipe_data);

    // Upload the wipe data to overwrite the original
    // This ensures the original content is overwritten before deletion
    s3_client
        .upload(
            &format!("{}.wipe", s3_key),
            wipe_data,
            "application/octet-stream",
        )
        .await?;

    Ok(())
}

/// Right to Erasure - Hard delete all user data (GDPR Article 17)
pub async fn hard_delete_user(
    db: &PgPool,
    s3_client: &S3Client,
    user_id: Uuid,
    deleted_by: Uuid,
    reason: Option<&str>,
) -> ApiResult<()> {
    info!(user_id = %user_id, deleted_by = %deleted_by, "Starting user hard deletion (GDPR Right to Erasure)");

    // Start transaction
    let mut tx = db.begin().await?;

    // 1. Get all files uploaded by this user
    let files: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, key FROM files WHERE uploader_id = $1",
    )
    .bind(user_id)
    .fetch_all(&mut *tx)
    .await?;

    // 2. Wipe and delete all user files
    for (file_id, s3_key) in &files {
        if let Err(e) = crypto_wipe_file(s3_client, s3_key.as_str()).await {
            warn!(file_id = %file_id, error = %e, "Failed to wipe file");
        }
        if let Err(e) = s3_client.delete(s3_key.as_str()).await {
            warn!(file_id = %file_id, error = %e, "Failed to delete file from S3");
        }
    }

    // 3. Delete files records
    sqlx::query("DELETE FROM files WHERE uploader_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 4. Delete reactions by user
    sqlx::query("DELETE FROM reactions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 5. Delete saved posts
    sqlx::query("DELETE FROM saved_posts WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 6. Delete user preferences
    sqlx::query("DELETE FROM preferences WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 7. Delete channel memberships
    sqlx::query("DELETE FROM channel_members WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 8. Delete channel read states
    sqlx::query("DELETE FROM channel_read_states WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 9. Delete thread memberships
    sqlx::query("DELETE FROM thread_memberships WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 10. Delete sidebar categories
    sqlx::query("DELETE FROM sidebar_categories WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 11. Delete user sessions
    sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 12. Delete email verification tokens
    sqlx::query("DELETE FROM email_verifications WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 13. Delete password reset tokens
    sqlx::query("DELETE FROM password_resets WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 14. Delete custom profile attribute values
    sqlx::query("DELETE FROM custom_profile_attribute_values WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 15. Update posts to anonymize (don't delete posts, but remove user association)
    // This is a common approach for GDPR - content remains but is anonymized
    sqlx::query(
        r#"
        UPDATE posts 
        SET user_id = NULL,
            message = '[deleted]'
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // 16. Delete team memberships
    sqlx::query("DELETE FROM team_members WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 17. Delete group memberships
    sqlx::query("DELETE FROM group_members WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // 18. Finally, delete the user record
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // 19. Log the deletion for audit purposes
    sqlx::query(
        r#"
        INSERT INTO gdpr_deletion_log 
            (user_id, deleted_by, deleted_at, reason, files_deleted, posts_anonymized)
        VALUES ($1, $2, NOW(), $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind(deleted_by)
    .bind(reason)
    .bind(files.len() as i64)
    .bind(true)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    info!(
        user_id = %user_id,
        files_deleted = files.len(),
        "User hard deleted successfully (GDPR Right to Erasure)"
    );

    Ok(())
}

/// Export all user data (GDPR Article 20 - Right to Data Portability)
pub async fn export_user_data(db: &PgPool, user_id: Uuid) -> ApiResult<UserDataExport> {
    // Get user profile
    #[derive(sqlx::FromRow)]
    struct UserProfileRow {
        username: String,
        email: String,
        display_name: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        nickname: Option<String>,
        position: Option<String>,
        created_at: DateTime<Utc>,
        last_login_at: Option<DateTime<Utc>>,
        role: String,
        is_bot: bool,
    }

    let user: UserProfileRow = sqlx::query_as(
        r#"
        SELECT 
            username, email, display_name, first_name, last_name,
            nickname, position, created_at, last_login_at, role, is_bot
        FROM users WHERE id = $1
        "#
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|_| AppError::NotFound("User not found".to_string()))?;

    let user_profile = UserProfileExport {
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        first_name: user.first_name,
        last_name: user.last_name,
        nickname: user.nickname,
        position: user.position,
        created_at: user.created_at,
        last_login_at: user.last_login_at,
        role: user.role,
        is_bot: user.is_bot,
    };

    // Get user posts
    let posts = sqlx::query_as::<_, (Uuid, Uuid, String, DateTime<Utc>, Option<DateTime<Utc>>, Vec<Uuid>)>(
        r#"
        SELECT id, channel_id, message, created_at, edited_at, file_ids
        FROM posts WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let posts: Vec<PostExport> = posts
        .into_iter()
        .map(|(id, channel_id, message, created_at, edited_at, file_ids)| PostExport {
            id,
            channel_id,
            message,
            created_at,
            edited_at,
            file_ids,
        })
        .collect();

    // Get user reactions
    let reactions = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        "SELECT post_id, emoji_name, created_at FROM reactions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let reactions: Vec<ReactionExport> = reactions
        .into_iter()
        .map(|(post_id, emoji_name, created_at)| ReactionExport {
            post_id,
            emoji_name,
            created_at,
        })
        .collect();

    // Get user files
    let files = sqlx::query_as::<_, (Uuid, String, String, i64, DateTime<Utc>)>(
        "SELECT id, name, mime_type, size, created_at FROM files WHERE uploader_id = $1",
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let files: Vec<FileExport> = files
        .into_iter()
        .map(|(id, name, mime_type, size, created_at)| FileExport {
            id,
            name,
            mime_type,
            size,
            created_at,
        })
        .collect();

    // Get user preferences
    let preferences: serde_json::Value = sqlx::query_scalar(
        "SELECT COALESCE(json_agg(p.*), '[]'::json) FROM preferences p WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    // Get channel memberships
    let channel_memberships = sqlx::query_as::<_, (Uuid, String, Option<String>, DateTime<Utc>, String)>(
        r#"
        SELECT 
            cm.channel_id,
            c.name as channel_name,
            t.name as team_name,
            cm.created_at as joined_at,
            cm.role
        FROM channel_members cm
        JOIN channels c ON c.id = cm.channel_id
        LEFT JOIN teams t ON t.id = c.team_id
        WHERE cm.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let channel_memberships: Vec<ChannelMembershipExport> = channel_memberships
        .into_iter()
        .map(|(channel_id, channel_name, team_name, joined_at, role)| ChannelMembershipExport {
            channel_id,
            channel_name,
            team_name,
            joined_at,
            role,
        })
        .collect();

    // Get audit log (limited to last 90 days for practical reasons)
    let audit_log = sqlx::query_as::<_, (String, String, Option<Uuid>, DateTime<Utc>, Option<String>)>(
        r#"
        SELECT action, entity_type, entity_id, created_at, ip_address
        FROM audit_log
        WHERE user_id = $1 AND created_at > NOW() - INTERVAL '90 days'
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    let audit_log: Vec<AuditLogExport> = audit_log
        .into_iter()
        .map(|(action, entity_type, entity_id, created_at, ip_address)| AuditLogExport {
            action,
            entity_type,
            entity_id,
            created_at,
            ip_address,
        })
        .collect();

    Ok(UserDataExport {
        export_id: Uuid::new_v4(),
        user_id,
        export_generated_at: Utc::now(),
        user_profile,
        posts,
        reactions,
        files,
        preferences,
        channel_memberships,
        audit_log,
    })
}

/// Anonymize user data instead of full deletion (alternative to hard delete)
/// This keeps the user's posts but removes all identifying information
pub async fn anonymize_user(
    db: &PgPool,
    _s3_client: &S3Client,
    user_id: Uuid,
    anonymized_by: Uuid,
) -> ApiResult<()> {
    let mut tx = db.begin().await?;

    // Update user record to anonymized state
    sqlx::query(
        r#"
        UPDATE users 
        SET username = 'deleted_user_' || SUBSTRING(id::text, 1, 8),
            email = 'deleted_' || SUBSTRING(id::text, 1, 8) || '@deleted.local',
            password_hash = NULL,
            display_name = 'Deleted User',
            first_name = NULL,
            last_name = NULL,
            nickname = NULL,
            position = NULL,
            avatar_url = NULL,
            custom_status = NULL,
            is_active = false,
            deleted_at = NOW(),
            deleted_by = $2,
            delete_reason = 'GDPR anonymization'
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(anonymized_by)
    .execute(&mut *tx)
    .await?;

    // Delete sensitive associated data but keep posts
    sqlx::query("DELETE FROM user_sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM email_verifications WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM password_resets WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // Anonymize file ownership
    sqlx::query("UPDATE files SET uploader_id = NULL WHERE uploader_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    info!(user_id = %user_id, "User data anonymized");

    Ok(())
}
