use chrono::Utc;
use sea_orm::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::db::entities::image_face_cache::{self, Column, Entity};
use crate::error::AppError;

pub struct FaceCacheRepo;

#[allow(dead_code)]
impl FaceCacheRepo {
    fn vec_literal(embedding: &[f64]) -> String {
        let inner: Vec<String> = embedding.iter().map(std::string::ToString::to_string).collect();
        format!("[{}]", inner.join(","))
    }

    fn face_index(face: &JsonValue, fallback: usize) -> i32 {
        face.get("index")
            .and_then(|v| v.as_i64())
            .or_else(|| face.get("faceIndex").and_then(|v| v.as_i64()))
            .unwrap_or(fallback as i64) as i32
    }

    fn face_embedding(face: &JsonValue) -> Result<Vec<f64>, AppError> {
        let values = face
            .get("embedding")
            .and_then(|v| v.as_array())
            .ok_or_else(|| AppError::BadRequest("face embedding is required".into()))?;
        if values.len() != 512 {
            return Err(AppError::BadRequest(format!(
                "face embedding must have 512 dimensions, got {}",
                values.len()
            )));
        }
        values
            .iter()
            .map(|v| {
                v.as_f64()
                    .ok_or_else(|| AppError::BadRequest("face embedding must contain numbers".into()))
            })
            .collect()
    }

    fn face_bbox(face: &JsonValue) -> JsonValue {
        if let Some(bbox) = face.get("bbox") {
            return bbox.clone();
        }

        serde_json::json!({
            "x": face.get("x").cloned().unwrap_or(JsonValue::Null),
            "y": face.get("y").cloned().unwrap_or(JsonValue::Null),
            "w": face.get("w").cloned().unwrap_or(JsonValue::Null),
            "h": face.get("h").cloned().unwrap_or(JsonValue::Null),
            "confidence": face.get("confidence").cloned().unwrap_or(JsonValue::Null),
        })
    }

    pub async fn get_by_image_hash<C: ConnectionTrait>(
        db: &C,
        image_hash: &str,
    ) -> Result<Vec<image_face_cache::Model>, AppError> {
        Ok(Entity::find()
            .filter(Column::ImageHash.eq(image_hash))
            .order_by_asc(Column::FaceIndex)
            .all(db)
            .await?)
    }

    pub async fn get_by_image_hash_and_index<C: ConnectionTrait>(
        db: &C,
        image_hash: &str,
        face_index: i32,
    ) -> Result<Option<image_face_cache::Model>, AppError> {
        Ok(Entity::find()
            .filter(Column::ImageHash.eq(image_hash))
            .filter(Column::FaceIndex.eq(face_index))
            .one(db)
            .await?)
    }

    pub async fn get_by_id<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<Option<image_face_cache::Model>, AppError> {
        Ok(Entity::find_by_id(id).one(db).await?)
    }

    pub async fn upsert_faces<C: ConnectionTrait>(
        db: &C,
        image_hash: &str,
        source_app: &str,
        source_id: &str,
        faces: &[JsonValue],
    ) -> Result<Vec<image_face_cache::Model>, AppError> {
        if faces.is_empty() {
            return Ok(Vec::new());
        }

        let now = Utc::now().fixed_offset();
        for (i, face) in faces.iter().enumerate() {
            let face_index = Self::face_index(face, i);
            let embedding = Self::face_embedding(face)?;
            let embedding_literal = Self::vec_literal(&embedding);
            let bbox = Self::face_bbox(face);
            let stmt = Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r"
                INSERT INTO image_face_cache
                    (id, image_hash, source_app, source_id, face_index, embedding, bbox, created_at)
                VALUES
                    ($1, $2, $3, $4, $5, $6::vector, $7::jsonb, $8)
                ON CONFLICT (image_hash, face_index)
                DO UPDATE SET
                    source_app = EXCLUDED.source_app,
                    source_id = EXCLUDED.source_id,
                    embedding = EXCLUDED.embedding,
                    bbox = EXCLUDED.bbox
                ",
                vec![
                    Uuid::new_v4().into(),
                    image_hash.into(),
                    source_app.into(),
                    source_id.into(),
                    face_index.into(),
                    embedding_literal.into(),
                    bbox.into(),
                    now.into(),
                ],
            );
            db.execute_raw(stmt).await?;
        }

        Self::get_by_image_hash(db, image_hash).await
    }

    pub async fn delete_by_source<C: ConnectionTrait>(
        db: &C,
        source_app: &str,
        source_id: &str,
    ) -> Result<u64, AppError> {
        let result = Entity::delete_many()
            .filter(Column::SourceApp.eq(source_app))
            .filter(Column::SourceId.eq(source_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
