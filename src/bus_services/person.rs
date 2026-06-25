use std::sync::Arc;

use tokimo_bus_client::BusClientBuilder;
use tokimo_bus_protocol::{BusError, CallerCtx, HttpMethod, MethodDecl};
use uuid::Uuid;

use crate::{
    db::repos::{face_cache_repo::FaceCacheRepo, person_repo::PersonRepo},
    services::person_events::emit_person_event,
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
    let ctx_persons_by_ids = ctx.clone();
    let ctx_update = ctx.clone();
    let ctx_merge = ctx.clone();
    let ctx_assign = ctx.clone();
    let ctx_create_from_face = ctx.clone();
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
                emit_person_event(
                    &ctx,
                    user_id,
                    if matched.is_new { "created" } else { "faces_changed" },
                    matched.person_id,
                    vec![matched.person_id],
                    &["faces", "media"],
                )
                .await;

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
            "persons_by_ids",
            "Fetch persons by id for a user",
        ))
        .on_invoke("persons_by_ids", move |req| {
            let ctx = ctx_persons_by_ids.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    person_ids: Vec<Uuid>,
                }
                let body: Req = decode_json(&req.payload)?;
                let user_id = get_user_id(&req.caller)?;
                let persons = PersonRepo::list_by_ids(&ctx.db, user_id, &body.person_ids)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;
                let person_ids: Vec<Uuid> = persons.iter().map(|person| person.id).collect();
                let media_counts = PersonRepo::media_counts(&ctx.db, &person_ids)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;
                #[derive(serde::Serialize)]
                struct PersonItem {
                    id: Uuid,
                    name: Option<String>,
                    avatar_url: Option<String>,
                    face_count: i32,
                    media_count: i32,
                }
                let items: Vec<PersonItem> = persons
                    .into_iter()
                    .map(|person| PersonItem {
                        media_count: media_counts.get(&person.id).copied().unwrap_or(0),
                        id: person.id,
                        name: person.name,
                        avatar_url: person.avatar_url,
                        face_count: person.face_count,
                    })
                    .collect();
                serde_json::to_vec(&items).map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "update_person",
            "Update a person profile for a user",
        ))
        .on_invoke("update_person", move |req| {
            let ctx = ctx_update.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    person_id: Uuid,
                    name: Option<String>,
                    avatar_url: Option<String>,
                }
                let body: Req = decode_json(&req.payload)?;
                let user_id = get_user_id(&req.caller)?;
                let mut changed_fields = Vec::new();
                if body.name.is_some() {
                    changed_fields.push("name");
                }
                if body.avatar_url.is_some() {
                    changed_fields.push("avatarUrl");
                }
                let person = PersonRepo::update(&ctx.db, body.person_id, user_id, body.name, body.avatar_url)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?
                    .ok_or_else(|| BusError::BadRequest("person not found".into()))?;
                emit_person_event(&ctx, user_id, "updated", person.id, vec![person.id], &changed_fields).await;
                serde_json::to_vec(&serde_json::json!({
                    "id": person.id,
                    "name": person.name,
                    "avatar_url": person.avatar_url,
                    "face_count": person.face_count,
                    "media_count": 0,
                }))
                .map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "merge_persons",
            "Merge one person into another for a user",
        ))
        .on_invoke("merge_persons", move |req| {
            let ctx = ctx_merge.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    source_id: Uuid,
                    target_id: Uuid,
                }
                let body: Req = decode_json(&req.payload)?;
                let user_id = get_user_id(&req.caller)?;
                PersonRepo::merge_persons(&ctx.db, user_id, body.source_id, body.target_id)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;
                emit_person_event(
                    &ctx,
                    user_id,
                    "merged",
                    body.target_id,
                    vec![body.target_id, body.source_id],
                    &["identity", "faces", "media"],
                )
                .await;
                serde_json::to_vec(&serde_json::json!({"success": true}))
                    .map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "assign_face",
            "Assign a cached face to an existing person",
        ))
        .on_invoke("assign_face", move |req| {
            let ctx = ctx_assign.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    person_id: Uuid,
                    image_hash: String,
                    face_index: i32,
                }
                let body: Req = decode_json(&req.payload)?;
                let user_id = get_user_id(&req.caller)?;
                let face = FaceCacheRepo::get_by_image_hash_and_index(&ctx.db, &body.image_hash, body.face_index)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?
                    .ok_or_else(|| BusError::BadRequest("face not found in cache".into()))?;
                let matched = PersonRepo::assign_face(&ctx.db, user_id, body.person_id, face.id)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;
                emit_person_event(
                    &ctx,
                    user_id,
                    "faces_changed",
                    matched.person_id,
                    vec![matched.person_id],
                    &["faces", "media"],
                )
                .await;
                serde_json::to_vec(&serde_json::json!({
                    "face_cache_id": matched.face_cache_id,
                    "person_id": matched.person_id,
                    "bbox": matched.bbox,
                    "is_new": matched.is_new,
                    "similarity": matched.similarity,
                }))
                .map_err(|e| BusError::Internal(e.to_string()))
            }
        })
        .method(decl(
            "create_person_from_face",
            "Create a person and assign a cached face to it",
        ))
        .on_invoke("create_person_from_face", move |req| {
            let ctx = ctx_create_from_face.clone();
            async move {
                #[derive(serde::Deserialize)]
                struct Req {
                    name: Option<String>,
                    image_hash: String,
                    face_index: i32,
                }
                let body: Req = decode_json(&req.payload)?;
                let user_id = get_user_id(&req.caller)?;
                let face = FaceCacheRepo::get_by_image_hash_and_index(&ctx.db, &body.image_hash, body.face_index)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?
                    .ok_or_else(|| BusError::BadRequest("face not found in cache".into()))?;
                let matched = PersonRepo::create_person_from_face(&ctx.db, user_id, face.id, body.name)
                    .await
                    .map_err(|e| BusError::Internal(e.to_string()))?;
                emit_person_event(
                    &ctx,
                    user_id,
                    "created",
                    matched.person_id,
                    vec![matched.person_id],
                    &["identity", "faces", "media"],
                )
                .await;
                serde_json::to_vec(&serde_json::json!({
                    "face_cache_id": matched.face_cache_id,
                    "person_id": matched.person_id,
                    "bbox": matched.bbox,
                    "is_new": matched.is_new,
                    "similarity": matched.similarity,
                }))
                .map_err(|e| BusError::Internal(e.to_string()))
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
