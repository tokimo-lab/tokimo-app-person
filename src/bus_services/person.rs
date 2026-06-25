use std::sync::Arc;

use tokimo_bus_client::BusClientBuilder;
use tokimo_bus_protocol::{BusError, CallerCtx, HttpMethod, MethodDecl};
use uuid::Uuid;

use crate::{
    db::repos::{face_cache_repo::FaceCacheRepo, person_repo::PersonRepo},
    state::AppState,
};

fn decl(name: &str, description: &str) -> MethodDecl {
    MethodDecl {
        name: name.into(),
        description: Some(description.into()),
        requires_auth: false,
        streaming: false,
        http_method: HttpMethod::Post,
        path: None,
    }
}

fn decode_json<T: serde::de::DeserializeOwned>(raw: &[u8]) -> Result<T, BusError> {
    serde_json::from_slice(raw).map_err(|e| BusError::BadRequest(format!("json decode: {e}")))
}

fn get_user_id(caller: &CallerCtx) -> Result<Uuid, BusError> {
    caller
        .user_id
        .as_deref()
        .ok_or_else(|| BusError::BadRequest("missing user_id".into()))
        .and_then(|s| Uuid::parse_str(s).map_err(|e| BusError::BadRequest(format!("user_id: {e}"))))
}

pub fn register(builder: BusClientBuilder, ctx: Arc<AppState>) -> BusClientBuilder {
    let ctx_register = ctx.clone();
    let ctx_match = ctx.clone();
    let ctx_delete = ctx.clone();
    let ctx_dispatch_delete = ctx.clone();
    let ctx_dispatch_register = ctx.clone();

    builder
        .method(decl(
            "register_faces",
            "Register face detections into the shared image_face_cache",
        ))
        .on_invoke("register_faces", move |req| {
            let ctx = ctx_register.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    image_hash: String,
                    source_app: String,
                    source_id: String,
                    faces: Vec<serde_json::Value>,
                }
                let body: Req = decode_json(&req.payload)?;
                let cached = FaceCacheRepo::upsert_faces(
                    &ctx.db,
                    &body.image_hash,
                    &body.source_app,
                    &body.source_id,
                    &body.faces,
                )
                .await
                .map_err(|e| BusError::Internal(e.to_string()))?;
                serde_json::to_vec(&cached).map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "match_face",
            "Match a face against known persons for a user",
        ))
        .on_invoke("match_face", move |req| {
            let ctx = ctx_match.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    image_hash: String,
                    face_index: i32,
                }
                let body: Req = decode_json(&req.payload)?;
                let user_id = get_user_id(&req.caller)?;

                let faces = FaceCacheRepo::get_by_image_hash(&ctx.db, &body.image_hash)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;
                let face = faces
                    .into_iter()
                    .find(|f| f.face_index == body.face_index)
                    .ok_or_else(|| BusError::BadRequest("face not found in cache".into()))?;

                let matched = PersonRepo::match_face(&ctx.db, user_id, face.id, 0.68)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;

                #[derive(serde::Serialize)]
                struct Resp {
                    face_cache_id: Uuid,
                    person_id: Option<Uuid>,
                    bbox: serde_json::Value,
                    is_new: bool,
                    similarity: f64,
                }
                let resp = Resp {
                    face_cache_id: matched.face_cache_id,
                    person_id: Some(matched.person_id),
                    bbox: matched.bbox,
                    is_new: matched.is_new,
                    similarity: matched.similarity,
                };
                serde_json::to_vec(&resp).map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "delete_source",
            "Delete cached faces and media by source (GC)",
        ))
        .on_invoke("delete_source", move |req| {
            let ctx = ctx_delete.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    source_app: String,
                    source_id: String,
                }
                let body: Req = decode_json(&req.payload)?;

                let deleted = PersonRepo::delete_source(&ctx.db, &body.source_app, &body.source_id)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;

                serde_json::to_vec(&serde_json::json!({
                    "deleted_faces": deleted.deleted_cache,
                    "deleted_media": deleted.deleted_media,
                    "affected_persons": deleted.affected_persons,
                }))
                .map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        // Job dispatch methods (called by job worker for async processing with retry)
        .method(decl(
            "dispatch_person_delete_source",
            "Job handler: delete cached faces by source",
        ))
        .on_invoke("dispatch_person_delete_source", move |req| {
            let ctx = ctx_dispatch_delete.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct JobReq {
                    job: JobPayload,
                }
                #[derive(serde::Deserialize)]
                struct JobPayload {
                    id: String,
                    params: serde_json::Value,
                }
                let body: JobReq = decode_json(&req.payload)?;
                let job_id = Uuid::parse_str(&body.job.id)
                    .map_err(|e| BusError::BadRequest(format!("job id: {e}")))?;

                let result = crate::queue::person_delete_source::handle(&ctx, job_id, &body.job.params)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;

                serde_json::to_vec(&result.unwrap_or_default())
                    .map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "dispatch_person_register_faces",
            "Job handler: register faces into shared cache",
        ))
        .on_invoke("dispatch_person_register_faces", move |req| {
            let ctx = ctx_dispatch_register.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct JobReq {
                    job: JobPayload,
                }
                #[derive(serde::Deserialize)]
                struct JobPayload {
                    id: String,
                    params: serde_json::Value,
                }
                let body: JobReq = decode_json(&req.payload)?;
                let job_id = Uuid::parse_str(&body.job.id)
                    .map_err(|e| BusError::BadRequest(format!("job id: {e}")))?;

                let result = crate::queue::person_register_faces::handle(&ctx, job_id, &body.job.params)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;

                serde_json::to_vec(&result.unwrap_or_default())
                    .map_err(|e| BusError::Internal(e.to_string()))
            }
        })
}
