use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::repos::{face_cache_repo::FaceCacheRepo, person_repo::PersonRepo};
use crate::error::AppError;
use crate::state::AppState;

pub async fn handle(
    ctx: &Arc<AppState>,
    _job_id: Uuid,
    params: &JsonValue,
) -> Result<Option<JsonValue>, AppError> {
    let source_app = params
        .get("sourceApp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing sourceApp".into()))?;
    let source_id = params
        .get("sourceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing sourceId".into()))?;

    // Delete media associations
    let deleted_media =
        PersonRepo::delete_media_by_source(&ctx.db, source_app, source_id).await?;

    // Delete face cache (CASCADE will clean person_faces)
    let deleted_cache =
        FaceCacheRepo::delete_by_source(&ctx.db, source_app, source_id).await?;

    // Clean up empty persons
    let user_ids: Vec<Uuid> = deleted_media
        .iter()
        .map(|m| m.user_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let mut affected_persons = 0u64;
    for uid in user_ids {
        affected_persons += PersonRepo::delete_empty_persons(&ctx.db, uid).await?;
    }

    Ok(Some(serde_json::json!({
        "deletedCache": deleted_cache,
        "deletedMedia": deleted_media.len(),
        "affectedPersons": affected_persons,
    })))
}
