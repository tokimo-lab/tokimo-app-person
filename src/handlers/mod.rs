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
    pub media_count: i32,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

impl From<crate::db::entities::persons::Model> for PersonDto {
    fn from(m: crate::db::entities::persons::Model) -> Self {
        Self::from_model(m, 0)
    }
}

impl PersonDto {
    fn from_model(m: crate::db::entities::persons::Model, media_count: i32) -> Self {
        Self {
            id: m.id,
            name: m.name,
            avatar_url: m.avatar_url,
            face_count: m.face_count,
            media_count,
            created_at: m.created_at.with_timezone(&Utc),
            updated_at: m.updated_at.with_timezone(&Utc),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct PersonListResponse {
    items: Vec<PersonDto>,
    total: u64,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PersonsByIdsReq {
    #[ts(type = "string[]")]
    pub person_ids: Vec<Uuid>,
}

pub async fn list_persons(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
) -> Result<Json<PersonListResponse>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let persons = PersonRepo::list(&ctx.db, uid).await?;
    let total = persons.len() as u64;
    let person_ids: Vec<Uuid> = persons.iter().map(|person| person.id).collect();
    let media_counts = PersonRepo::media_counts(&ctx.db, &person_ids).await?;
    let items = persons
        .into_iter()
        .map(|person| {
            let media_count = media_counts.get(&person.id).copied().unwrap_or(0);
            PersonDto::from_model(person, media_count)
        })
        .collect();
    Ok(Json(PersonListResponse { items, total }))
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

pub async fn persons_by_ids(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<PersonsByIdsReq>,
) -> Result<Json<PersonListResponse>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let persons = PersonRepo::list_by_ids(&ctx.db, uid, &req.person_ids).await?;
    let total = persons.len() as u64;
    let person_ids: Vec<Uuid> = persons.iter().map(|person| person.id).collect();
    let media_counts = PersonRepo::media_counts(&ctx.db, &person_ids).await?;
    let items = persons
        .into_iter()
        .map(|person| {
            let media_count = media_counts.get(&person.id).copied().unwrap_or(0);
            PersonDto::from_model(person, media_count)
        })
        .collect();
    Ok(Json(PersonListResponse { items, total }))
}

// ── Person detail with faces ─────────────────────────────────────────────────

#[derive(Serialize, TS)]
#[ts(export)]
pub struct FaceDetailDto {
    #[ts(type = "string")]
    pub id: String,
    pub image_hash: String,
    pub face_index: i32,
    pub bbox: serde_json::Value,
    pub source_app: String,
    pub source_id: String,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SourceMediaDto {
    #[ts(type = "string")]
    pub id: String,
    pub source_app: String,
    pub source_id: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct PersonDetailDto {
    #[ts(type = "string")]
    pub id: Uuid,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub face_count: i32,
    pub media_count: i32,
    pub faces: Vec<FaceDetailDto>,
    pub media: Vec<SourceMediaDto>,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

pub async fn get_person_detail(
    State(ctx): State<Arc<AppCtx>>,
    Path(id): Path<Uuid>,
    TokimoUser { user_id }: TokimoUser,
) -> Result<Json<PersonDetailDto>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let person = PersonRepo::get_by_id(&ctx.db, id, uid)
        .await?
        .ok_or_else(|| AppError::NotFound("person not found".into()))?;

    let face_links = PersonRepo::get_person_faces(&ctx.db, id).await?;
    let media_rows = PersonRepo::get_person_media(&ctx.db, id).await?;
    let faces: Vec<FaceDetailDto> = face_links
        .into_iter()
        .map(|(pf, cache)| FaceDetailDto {
            id: pf.id.to_string(),
            image_hash: cache.image_hash.clone(),
            face_index: cache.face_index,
            bbox: cache.bbox.clone(),
            source_app: cache.source_app.clone(),
            source_id: cache.source_id.clone(),
        })
        .collect();
    let media: Vec<SourceMediaDto> = media_rows
        .into_iter()
        .map(|row| SourceMediaDto {
            id: row.id.to_string(),
            source_app: row.source_app,
            source_id: row.source_id,
            created_at: row.created_at.with_timezone(&Utc),
        })
        .collect();

    Ok(Json(PersonDetailDto {
        id: person.id,
        name: person.name,
        avatar_url: person.avatar_url,
        face_count: person.face_count,
        media_count: media.len().min(i32::MAX as usize) as i32,
        faces,
        media,
        created_at: person.created_at.with_timezone(&Utc),
        updated_at: person.updated_at.with_timezone(&Utc),
    }))
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

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MergePersonsReq {
    #[ts(type = "string")]
    pub target_id: Uuid,
    #[ts(type = "string")]
    pub source_id: Uuid,
}

pub async fn merge_persons(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<MergePersonsReq>,
) -> Result<Json<PersonDto>, AppError> {
    let uid = parse_user_id(&user_id)?;
    PersonRepo::merge_persons(&ctx.db, uid, req.source_id, req.target_id).await?;
    let person = PersonRepo::get_by_id(&ctx.db, req.target_id, uid)
        .await?
        .ok_or_else(|| AppError::NotFound("target person not found".into()))?;
    Ok(Json(PersonDto::from(person)))
}

// ── Bus API proxy handlers (for demo page testing) ──────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterFacesReq {
    pub image_hash: String,
    pub source_app: String,
    pub source_id: String,
    pub faces: Vec<serde_json::Value>,
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
    let result =
        FaceCacheRepo::upsert_faces(&ctx.db, &req.image_hash, &req.source_app, &req.source_id, &req.faces).await?;

    Ok(Json(RegisterFacesResponse { cached: result.len() }))
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
    #[ts(type = "string")]
    pub face_cache_id: Uuid,
    pub bbox: serde_json::Value,
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

    let matched = PersonRepo::match_face(&ctx.db, uid, face.id, 0.68).await?;

    Ok(Json(MatchFaceResponse {
        person_id: matched.person_id,
        face_cache_id: matched.face_cache_id,
        bbox: matched.bbox,
        is_new: matched.is_new,
        similarity: matched.similarity,
    }))
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssignFaceReq {
    #[ts(type = "string")]
    pub person_id: Uuid,
    pub image_hash: String,
    pub face_index: i32,
}

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreatePersonFromFaceReq {
    pub name: Option<String>,
    pub image_hash: String,
    pub face_index: i32,
}

async fn resolve_face_cache_id(ctx: &AppCtx, image_hash: &str, face_index: i32) -> Result<Uuid, AppError> {
    let face = FaceCacheRepo::get_by_image_hash_and_index(&ctx.db, image_hash, face_index)
        .await?
        .ok_or_else(|| AppError::NotFound("face not found in cache".into()))?;
    Ok(face.id)
}

pub async fn assign_face(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<AssignFaceReq>,
) -> Result<Json<MatchFaceResponse>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let face_cache_id = resolve_face_cache_id(&ctx, &req.image_hash, req.face_index).await?;
    let matched = PersonRepo::assign_face(&ctx.db, uid, req.person_id, face_cache_id).await?;
    Ok(Json(MatchFaceResponse {
        person_id: matched.person_id,
        face_cache_id: matched.face_cache_id,
        bbox: matched.bbox,
        is_new: matched.is_new,
        similarity: matched.similarity,
    }))
}

pub async fn create_person_from_face(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<CreatePersonFromFaceReq>,
) -> Result<Json<MatchFaceResponse>, AppError> {
    let uid = parse_user_id(&user_id)?;
    let face_cache_id = resolve_face_cache_id(&ctx, &req.image_hash, req.face_index).await?;
    let matched = PersonRepo::create_person_from_face(&ctx.db, uid, face_cache_id, req.name).await?;
    Ok(Json(MatchFaceResponse {
        person_id: matched.person_id,
        face_cache_id: matched.face_cache_id,
        bbox: matched.bbox,
        is_new: matched.is_new,
        similarity: matched.similarity,
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
    let deleted = PersonRepo::delete_source(&ctx.db, &req.source_app, &req.source_id).await?;

    Ok(Json(DeleteSourceResponse {
        deleted_cache: deleted.deleted_cache,
        deleted_media: deleted.deleted_media,
        affected_persons: deleted.affected_persons,
    }))
}
