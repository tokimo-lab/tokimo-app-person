use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::repos::face_cache_repo::FaceCacheRepo;
use crate::error::AppError;
use crate::state::AppState;

pub async fn handle(
    ctx: &Arc<AppState>,
    _job_id: Uuid,
    params: &JsonValue,
) -> Result<Option<JsonValue>, AppError> {
    let image_hash = params
        .get("imageHash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing imageHash".into()))?;
    let source_app = params
        .get("sourceApp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing sourceApp".into()))?;
    let source_id = params
        .get("sourceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing sourceId".into()))?;

    let faces_json = params
        .get("faces")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::BadRequest("missing faces array".into()))?;

    let mut faces = Vec::new();
    for face in faces_json {
        let index = face
            .get("index")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let bbox = face
            .get("bbox")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        faces.push((index, vec![], bbox));
    }

    let result =
        FaceCacheRepo::upsert_faces(&ctx.db, image_hash, source_app, source_id, faces).await?;

    Ok(Some(serde_json::json!({
        "cached": result.len(),
    })))
}
