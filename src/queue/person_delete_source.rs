use serde_json::Value as JsonValue;
use std::sync::Arc;
use uuid::Uuid;

use crate::db::repos::person_repo::PersonRepo;
use crate::error::AppError;
use crate::state::AppState;

pub async fn handle(ctx: &Arc<AppState>, _job_id: Uuid, params: &JsonValue) -> Result<Option<JsonValue>, AppError> {
    let source_app = params
        .get("sourceApp")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing sourceApp".into()))?;
    let source_id = params
        .get("sourceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("missing sourceId".into()))?;

    let deleted = PersonRepo::delete_source(&ctx.db, source_app, source_id).await?;

    Ok(Some(serde_json::json!({
        "deletedCache": deleted.deleted_cache,
        "deletedMedia": deleted.deleted_media,
        "affectedPersons": deleted.affected_persons,
    })))
}
