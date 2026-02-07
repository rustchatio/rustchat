use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use image::{GenericImageView, ImageFormat};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use uuid::Uuid;

use super::extractors::MmAuthUser;
use crate::api::AppState;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::{
    id::{encode_mm_id, parse_mm_or_uuid},
    models as mm,
};
use crate::models::FileInfo;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files", post(upload_file))
        .route("/files/{file_id}", get(get_file))
        .route("/files/{file_id}/info", get(get_file_info))
        .route("/files/{file_id}/thumbnail", get(get_thumbnail))
        .route("/files/{file_id}/preview", get(get_preview))
        .route("/files/{file_id}/link", get(get_link))
}

async fn upload_file(
    State(state): State<AppState>,
    auth: MmAuthUser,
    mut multipart: Multipart,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    let mut channel_id: Option<Uuid> = None;
    let mut client_ids: Vec<String> = Vec::new();

    struct PendingFile {
        filename: String,
        content_type: String,
        data: Vec<u8>,
    }

    let mut pending_files: Vec<PendingFile> = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "channel_id" {
            let txt = field.text().await.unwrap_or_default();
            if let Some(id) = parse_mm_or_uuid(&txt) {
                channel_id = Some(id);
            }
        } else if name == "client_ids" {
            let txt = field.text().await.unwrap_or_default();
            client_ids.push(txt);
        } else if !name.is_empty() {
            // Accept multiple field names: "files", "file", "attachment", or unnamed
            // React Native network client may use different field names
            if field.file_name().is_some() || field.content_type().is_some() {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Read error: {}", e)))?
                    .to_vec();

                pending_files.push(PendingFile {
                    filename,
                    content_type,
                    data,
                });
            }
        }
    }

    let mut file_infos: Vec<mm::FileInfo> = Vec::new();

    for file in pending_files {
        let file_id = Uuid::new_v4();
        let extension = file.filename.rsplit('.').next().unwrap_or("").to_string();
        let key = format!("files/{}/{}.{}", auth.user_id, file_id, extension);

        let mut hasher = Sha256::new();
        hasher.update(&file.data);
        let hash = hex::encode(hasher.finalize());
        let size = file.data.len() as i64;

        state
            .s3_client
            .upload(&key, file.data.clone(), &file.content_type)
            .await?;

        // Image processing (Blocking offloaded)
        let (width, height, thumbnail_data, preview_data) =
            if file.content_type.starts_with("image/") {
                let data_clone = file.data.clone();

                tokio::task::spawn_blocking(move || {
                    if let Ok(img) = image::load_from_memory(&data_clone) {
                        let (w, h) = img.dimensions();
                        let w_out = Some(w as i32);
                        let h_out = Some(h as i32);

                        // Generate thumbnail (400x400 max) as JPEG for Mattermost mobile compatibility
                        let thumb_data = if w > 400 || h > 400 {
                            let thumb = img.thumbnail(400, 400);
                            let mut buf = Vec::new();
                            // Use JPEG format for mobile compatibility (Mattermost expects image/jpeg)
                            if thumb
                                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
                                .is_ok()
                            {
                                Some(buf)
                            } else {
                                None
                            }
                        } else {
                            // For small images, generate JPEG thumbnail for consistency
                            let mut buf = Vec::new();
                            if img
                                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
                                .is_ok()
                            {
                                Some(buf)
                            } else {
                                None
                            }
                        };

                        // Generate preview (1024x1024 max) as JPEG for Mattermost mobile compatibility
                        let preview_data = if w > 1024 || h > 1024 {
                            let preview = img.thumbnail(1024, 1024);
                            let mut buf = Vec::new();
                            // Use JPEG format for mobile compatibility (Mattermost expects image/jpeg)
                            if preview
                                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
                                .is_ok()
                            {
                                Some(buf)
                            } else {
                                None
                            }
                        } else {
                            // If smaller than 1024, generate JPEG preview for consistency
                            let mut buf = Vec::new();
                            if img
                                .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
                                .is_ok()
                            {
                                Some(buf)
                            } else {
                                None
                            }
                        };

                        (w_out, h_out, thumb_data, preview_data)
                    } else {
                        (None, None, None, None)
                    }
                })
                .await
                .unwrap_or((None, None, None, None))
            } else {
                (None, None, None, None)
            };

        let mut thumbnail_key = None;
        if let Some(t_data) = thumbnail_data {
            // Store as .jpg with image/jpeg content type for Mattermost mobile compatibility
            let t_key = format!("thumbnails/{}/{}.jpg", auth.user_id, file_id);
            if state
                .s3_client
                .upload(&t_key, t_data, "image/jpeg")
                .await
                .is_ok()
            {
                thumbnail_key = Some(t_key);
            }
        }

        // Store preview as JPEG for Mattermost mobile compatibility
        if let Some(p_data) = preview_data {
            let p_key = format!("previews/{}/{}.jpg", auth.user_id, file_id);
            let _ = state.s3_client.upload(&p_key, p_data, "image/jpeg").await;
        }

        let has_thumbnail = thumbnail_key.is_some();

        let _file_info: FileInfo = sqlx::query_as(
            r#"
            INSERT INTO files (id, uploader_id, channel_id, name, key, mime_type, size, sha256, width, height, has_thumbnail, thumbnail_key)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(file_id)
        .bind(auth.user_id)
        .bind(channel_id)
        .bind(&file.filename)
        .bind(&key)
        .bind(&file.content_type)
        .bind(size)
        .bind(&hash)
        .bind(width)
        .bind(height)
        .bind(has_thumbnail)
        .bind(thumbnail_key)
        .fetch_one(&state.db)
        .await?;

        file_infos.push(mm::FileInfo {
            id: encode_mm_id(file_id),
            user_id: encode_mm_id(auth.user_id),
            post_id: "".to_string(),
            channel_id: channel_id.map(encode_mm_id).unwrap_or_default(),
            create_at: Utc::now().timestamp_millis(),
            update_at: Utc::now().timestamp_millis(),
            delete_at: 0,
            name: file.filename,
            extension,
            size,
            mime_type: file.content_type,
            width: width.unwrap_or(0),
            height: height.unwrap_or(0),
            has_preview_image: has_thumbnail,
            mini_preview: None,
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "file_infos": file_infos,
            "client_ids": client_ids
        })),
    ))
}

async fn get_file(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(file_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let file_id = parse_mm_or_uuid(&file_id)
        .ok_or_else(|| AppError::BadRequest("Invalid file_id".to_string()))?;
    let file: FileInfo = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let data = state.s3_client.download(&file.key).await?;

    Ok((
        [
            (header::CONTENT_TYPE, file.mime_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", file.name),
            ),
            (
                header::CACHE_CONTROL,
                "max-age=2592000, private".to_string(),
            ),
            // Security headers matching Mattermost server
            (
                header::HeaderName::from_static("x-content-type-options"),
                "nosniff".to_string(),
            ),
            (
                header::HeaderName::from_static("x-frame-options"),
                "DENY".to_string(),
            ),
            (
                header::HeaderName::from_static("content-security-policy"),
                "Frame-ancestors 'none'".to_string(),
            ),
        ],
        data,
    ))
}

/// GET /files/{file_id}/info - Get file metadata
async fn get_file_info(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(file_id): Path<String>,
) -> ApiResult<Json<mm::FileInfo>> {
    let file_id = parse_mm_or_uuid(&file_id)
        .ok_or_else(|| AppError::BadRequest("Invalid file_id".to_string()))?;
    let file: FileInfo = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    // Get file extension from name
    let extension = file.name.rsplit('.').next().unwrap_or("").to_string();

    Ok(Json(mm::FileInfo {
        id: encode_mm_id(file.id),
        user_id: encode_mm_id(file.uploader_id),
        post_id: file.post_id.map(encode_mm_id).unwrap_or_default(),
        channel_id: file.channel_id.map(encode_mm_id).unwrap_or_default(),
        create_at: file.created_at.timestamp_millis(),
        update_at: file.created_at.timestamp_millis(),
        delete_at: 0,
        name: file.name,
        extension,
        size: file.size,
        mime_type: file.mime_type,
        width: file.width.unwrap_or(0),
        height: file.height.unwrap_or(0),
        has_preview_image: file.has_thumbnail,
        mini_preview: None,
    }))
}

async fn get_thumbnail(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(file_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let file_id = parse_mm_or_uuid(&file_id)
        .ok_or_else(|| AppError::BadRequest("Invalid file_id".to_string()))?;
    let file: FileInfo = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    if file.has_thumbnail {
        if let Some(key) = file.thumbnail_key {
            let data = state.s3_client.download(&key).await?;
            return Ok((
                [
                    // Use image/jpeg for Mattermost mobile compatibility
                    (header::CONTENT_TYPE, "image/jpeg".to_string()),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("inline; filename=\"thumb_{}\"", file.name),
                    ),
                    (
                        header::CACHE_CONTROL,
                        "max-age=2592000, private".to_string(),
                    ),
                    // Security headers
                    (
                        header::HeaderName::from_static("x-content-type-options"),
                        "nosniff".to_string(),
                    ),
                ],
                data,
            )
                .into_response());
        }
    }

    // Fallback to original if no thumbnail or just 404?
    // MM returns 404 if no thumbnail.
    Err(AppError::NotFound("Thumbnail not found".to_string()))
}

async fn get_preview(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(file_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let file_id = parse_mm_or_uuid(&file_id)
        .ok_or_else(|| AppError::BadRequest("Invalid file_id".to_string()))?;
    let file: FileInfo = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    if file.mime_type.starts_with("image/") {
        if file.has_thumbnail {
            // Derive preview key from convention (now using .jpg for JPEG format)
            let preview_key = format!("previews/{}/{}.jpg", file.uploader_id, file.id);
            if let Ok(data) = state.s3_client.download(&preview_key).await {
                return Ok((
                    [
                        // Use image/jpeg for Mattermost mobile compatibility
                        (header::CONTENT_TYPE, "image/jpeg".to_string()),
                        (
                            header::CONTENT_DISPOSITION,
                            format!("inline; filename=\"preview_{}\"", file.name),
                        ),
                        (
                            header::CACHE_CONTROL,
                            "max-age=2592000, private".to_string(),
                        ),
                        // Security headers
                        (
                            header::HeaderName::from_static("x-content-type-options"),
                            "nosniff".to_string(),
                        ),
                    ],
                    data,
                )
                    .into_response());
            }

            // If preview not found but thumbnail exists, try to serve thumbnail as fallback
            if let Some(thumb_key) = &file.thumbnail_key {
                if let Ok(data) = state.s3_client.download(thumb_key).await {
                    return Ok((
                        [
                            (header::CONTENT_TYPE, "image/jpeg".to_string()),
                            (
                                header::CONTENT_DISPOSITION,
                                format!("inline; filename=\"preview_{}\"", file.name),
                            ),
                            (
                                header::CACHE_CONTROL,
                                "max-age=2592000, private".to_string(),
                            ),
                            (
                                header::HeaderName::from_static("x-content-type-options"),
                                "nosniff".to_string(),
                            ),
                        ],
                        data,
                    )
                        .into_response());
                }
            }
        }
    }

    // If we can't serve a preview image, return 404 or 400.
    // Mattermost returns 404 if no preview (e.g. non-images).
    // Redirecting non-images to S3 presigned URL for "preview" endpoint confuses mobile client
    // because it expects an image or an error.
    Err(AppError::NotFound("Preview not available".to_string()))
}

async fn get_link(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(file_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let file_id = parse_mm_or_uuid(&file_id)
        .ok_or_else(|| AppError::BadRequest("Invalid file_id".to_string()))?;
    let file: FileInfo = sqlx::query_as("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    let url = state
        .s3_client
        .presigned_download_url(&file.key, 3600)
        .await?;

    Ok(Json(serde_json::json!({"link": url})))
}
