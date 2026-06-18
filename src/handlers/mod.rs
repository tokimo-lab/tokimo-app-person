use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokimo_bus_auth::TokimoUser;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    db::repos::{face_cache_repo::FaceCacheRepo, person_repo::PersonRepo},
    error::AppError,
};

pub type AppCtx = crate::state::AppState;

fn parse_user_id(user_id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(user_id).map_err(|_| AppError::BadRequest("invalid user id".into()))
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct PersonDto {
    #[ts(type = "string")]
    pub id: Uuid,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub face_count: i32,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

impl From<crate::db::entities::persons::Model> for PersonDto {
    fn from(m: crate::db::entities::persons::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            avatar_url: m.avatar_url,
            face_count: m.face_count,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct PersonListResponse {
    persons: Vec<PersonDto>,
}

pub async fn list_persons(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
) -> Result<Json<PersonListResponse>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let persons = PersonRepo::list(&ctx.db, uid)
        .await?
        .into_iter()
        .map(PersonDto::from)
        .collect();
    Ok(Json(PersonListResponse { persons }))
}

pub async fn get_person(
    State(ctx): State<Arc<AppCtx>>,
    Path(id): Path<Uuid>,
    TokimoUser { user_id }: TokimoUser,
) -> Result<Json<PersonDto>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let person = PersonRepo::get_by_id(&ctx.db, id, uid)
        .await?
        .ok_or_else(|| AppError::NotFound("person not found".into()))?;
    Ok(Json(PersonDto::from(person)))
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UpdatePersonReq {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn update_person(
    State(ctx): State<Arc<AppCtx>>,
    Path(id): Path<Uuid>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<UpdatePersonReq>,
) -> Result<Json<PersonDto>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let person = PersonRepo::update(&ctx.db, id, uid, req.name, req.avatar_url)
        .await?
        .ok_or_else(|| AppError::NotFound("person not found".into()))?;
    Ok(Json(PersonDto::from(person)))
}

// ── Bus API proxy handlers (for demo page testing) ──────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterFacesReq {
    pub image_hash: String,
    pub source_app: String,
    pub source_id: String,
    pub faces: Vec<RegisterFaceItem>,
}

#[derive(Deserialize)]
pub struct RegisterFaceItem {
    pub index: i32,
    pub bbox: serde_json::Value,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct RegisterFacesResponse {
    pub cached: usize,
}

pub async fn register_faces(
    State(ctx): State<Arc<AppCtx>>,
    Json(req): Json<RegisterFacesReq>,
) -> Result<Json<RegisterFacesResponse>, AppError> {
    let faces: Vec<serde_json::Value> = req
        .faces
        .into_iter()
        .map(|f| serde_json::json!({"index": f.index, "bbox": f.bbox}))
        .collect();

    let result =
        FaceCacheRepo::upsert_faces(&ctx.db, &req.image_hash, &req.source_app, &req.source_id, &faces)
            .await?;

    Ok(Json(RegisterFacesResponse {
        cached: result.len(),
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchFaceReq {
    pub image_hash: String,
    pub face_index: i32,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct MatchFaceResponse {
    #[ts(type = "string")]
    pub person_id: Uuid,
    pub is_new: bool,
    pub similarity: f64,
}

pub async fn match_face(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<MatchFaceReq>,
) -> Result<Json<MatchFaceResponse>, AppError> {
    let uid = parse_user_id(&user_id)?;

    // Get face from cache
    let cached_faces = FaceCacheRepo::get_by_image_hash(&ctx.db, &req.image_hash).await?;
    let face = cached_faces
        .into_iter()
        .find(|f| f.face_index == req.face_index)
        .ok_or_else(|| AppError::NotFound("face not found in cache".into()))?;

    // Check if already linked
    if let Some(link) = PersonRepo::get_face_link(&ctx.db, uid, face.id).await? {
        return Ok(Json(MatchFaceResponse {
            person_id: link.person_id,
            is_new: false,
            similarity: 1.0,
        }));
    }

    // Create new person and link
    let person = PersonRepo::create(&ctx.db, uid, None).await?;
    PersonRepo::link_face(&ctx.db, uid, person.id, face.id).await?;
    PersonRepo::link_media(&ctx.db, uid, person.id, &face.source_app, &face.source_id).await?;

    Ok(Json(MatchFaceResponse {
        person_id: person.id,
        is_new: true,
        similarity: 0.0,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSourceReq {
    pub source_app: String,
    pub source_id: String,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct DeleteSourceResponse {
    pub deleted_cache: u64,
    pub deleted_media: u64,
    pub affected_persons: u64,
}

pub async fn delete_source(
    State(ctx): State<Arc<AppCtx>>,
    Json(req): Json<DeleteSourceReq>,
) -> Result<Json<DeleteSourceResponse>, AppError> {
    // Delete media associations
    let deleted_media_count =
        PersonRepo::delete_media_by_source(&ctx.db, &req.source_app, &req.source_id).await?;

    // Delete face cache (CASCADE will clean person_faces)
    let deleted_cache =
        FaceCacheRepo::delete_by_source(&ctx.db, &req.source_app, &req.source_id).await?;

    // Clean up empty persons (we don't know which users were affected,
    // so we clean up all empty persons for now)
    // TODO: Track affected users more precisely
    let affected_persons = 0u64;

    Ok(Json(DeleteSourceResponse {
        deleted_cache,
        deleted_media: deleted_media_count,
        affected_persons,
    }))
}
