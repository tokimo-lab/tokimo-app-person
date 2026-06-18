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

    let result =
        FaceCacheRepo::upsert_faces(&ctx.db, image_hash, source_app, source_id, faces_json).await?;

    Ok(Some(serde_json::json!({
        "cached": result.len(),
    })))
}
