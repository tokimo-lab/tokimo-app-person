use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{sea_query::Expr, *};
use serde_json::json;
use uuid::Uuid;

use crate::db::entities::{
    image_face_cache,
    person_faces::{self as pf, ActiveModel as PfActiveModel},
    person_media::{self as pm, ActiveModel as PmActiveModel},
    persons::{self, ActiveModel, Column, Entity},
};
use crate::error::AppError;

pub struct PersonRepo;

#[derive(Debug, Clone)]
pub struct FaceMatch {
    pub face_cache_id: Uuid,
    pub person_id: Uuid,
    pub bbox: serde_json::Value,
    pub is_new: bool,
    pub similarity: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SourceDelete {
    pub deleted_cache: u64,
    pub deleted_media: u64,
    pub affected_persons: u64,
}

#[allow(dead_code)]
impl PersonRepo {
    pub async fn list<C: ConnectionTrait>(db: &C, user_id: Uuid) -> Result<Vec<persons::Model>, AppError> {
        Ok(Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::UpdatedAt)
            .all(db)
            .await?)
    }

    pub async fn list_by_ids<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        ids: &[Uuid],
    ) -> Result<Vec<persons::Model>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        Ok(Entity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Id.eq_any(ids.to_vec()))
            .all(db)
            .await?)
    }

    pub async fn media_counts<C: ConnectionTrait>(db: &C, person_ids: &[Uuid]) -> Result<HashMap<Uuid, i32>, AppError> {
        let mut counts = HashMap::new();
        if person_ids.is_empty() {
            return Ok(counts);
        }

        let rows = pm::Entity::find()
            .select_only()
            .column(pm::Column::PersonId)
            .column_as(pm::Column::Id.count(), "cnt")
            .filter(pm::Column::PersonId.eq_any(person_ids.to_vec()))
            .group_by(pm::Column::PersonId)
            .into_tuple::<(Uuid, i64)>()
            .all(db)
            .await?;

        for (person_id, count) in rows {
            counts.insert(person_id, Ord::min(count, i32::MAX as i64) as i32);
        }

        Ok(counts)
    }

    pub async fn get_by_id<C: ConnectionTrait>(
        db: &C,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<persons::Model>, AppError> {
        Ok(Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await?)
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        name: Option<String>,
    ) -> Result<persons::Model, AppError> {
        let now = Utc::now().fixed_offset();
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            name: Set(name),
            avatar_url: Set(None),
            face_count: Set(0),
            metadata: Set(json!({})),
            created_at: Set(now),
            updated_at: Set(now),
        };
        Ok(am.insert(db).await?)
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<Option<persons::Model>, AppError> {
        let Some(model) = Self::get_by_id(db, id, user_id).await? else {
            return Ok(None);
        };
        let mut am: ActiveModel = model.into();
        if let Some(n) = name {
            am.name = Set(Some(n));
        }
        if let Some(url) = avatar_url {
            am.avatar_url = Set(Some(url));
        }
        am.updated_at = Set(Utc::now().fixed_offset());
        Ok(Some(am.update(db).await?))
    }

    pub async fn increment_face_count<C: ConnectionTrait>(db: &C, person_id: Uuid) -> Result<(), AppError> {
        Entity::update_many()
            .col_expr(Column::FaceCount, Expr::col(Column::FaceCount).add(1))
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now().fixed_offset()))
            .filter(Column::Id.eq(person_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn decrement_face_count<C: ConnectionTrait>(db: &C, person_id: Uuid) -> Result<(), AppError> {
        Entity::update_many()
            .col_expr(Column::FaceCount, Expr::cust("GREATEST(face_count - 1, 0)"))
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now().fixed_offset()))
            .filter(Column::Id.eq(person_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_empty_persons<C: ConnectionTrait>(db: &C, user_id: Uuid) -> Result<u64, AppError> {
        let result = Entity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::FaceCount.eq(0))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn link_face<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        person_id: Uuid,
        face_cache_id: Uuid,
    ) -> Result<pf::Model, AppError> {
        let am = PfActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            person_id: Set(person_id),
            face_cache_id: Set(face_cache_id),
            created_at: Set(Utc::now().fixed_offset()),
        };
        pf::Entity::insert(am)
            .on_conflict(
                sea_query::OnConflict::columns([pf::Column::UserId, pf::Column::FaceCacheId])
                    .do_nothing()
                    .to_owned(),
            )
            .exec(db)
            .await?;
        Self::get_face_link(db, user_id, face_cache_id)
            .await?
            .ok_or_else(|| AppError::Internal("failed to link face".into()))
    }

    pub async fn get_face_link<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        face_cache_id: Uuid,
    ) -> Result<Option<pf::Model>, AppError> {
        Ok(pf::Entity::find()
            .filter(pf::Column::UserId.eq(user_id))
            .filter(pf::Column::FaceCacheId.eq(face_cache_id))
            .one(db)
            .await?)
    }

    pub async fn get_person_faces<C: ConnectionTrait>(
        db: &C,
        person_id: Uuid,
    ) -> Result<Vec<(pf::Model, image_face_cache::Model)>, AppError> {
        Ok(pf::Entity::find()
            .filter(pf::Column::PersonId.eq(person_id))
            .find_also_related(image_face_cache::Entity)
            .all(db)
            .await?
            .into_iter()
            .filter_map(|(pf_model, cache)| cache.map(|c| (pf_model, c)))
            .collect())
    }

    pub async fn link_media<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        person_id: Uuid,
        source_app: &str,
        source_id: &str,
    ) -> Result<pm::Model, AppError> {
        let am = PmActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_id),
            person_id: Set(person_id),
            source_app: Set(source_app.to_string()),
            source_id: Set(source_id.to_string()),
            created_at: Set(Utc::now().fixed_offset()),
        };
        pm::Entity::insert(am)
            .on_conflict(
                sea_query::OnConflict::columns([
                    pm::Column::UserId,
                    pm::Column::PersonId,
                    pm::Column::SourceApp,
                    pm::Column::SourceId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await?;
        pm::Entity::find()
            .filter(pm::Column::UserId.eq(user_id))
            .filter(pm::Column::PersonId.eq(person_id))
            .filter(pm::Column::SourceApp.eq(source_app))
            .filter(pm::Column::SourceId.eq(source_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::Internal("failed to link media".into()))
    }

    pub async fn get_person_media<C: ConnectionTrait>(db: &C, person_id: Uuid) -> Result<Vec<pm::Model>, AppError> {
        Ok(pm::Entity::find()
            .filter(pm::Column::PersonId.eq(person_id))
            .order_by_desc(pm::Column::CreatedAt)
            .all(db)
            .await?)
    }

    pub async fn delete_media_by_source<C: ConnectionTrait>(
        db: &C,
        source_app: &str,
        source_id: &str,
    ) -> Result<u64, AppError> {
        let result = pm::Entity::delete_many()
            .filter(pm::Column::SourceApp.eq(source_app))
            .filter(pm::Column::SourceId.eq(source_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    async fn delete_person_media_if_source_empty<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        person_id: Uuid,
        source_app: &str,
        source_id: &str,
    ) -> Result<(), AppError> {
        let remaining = pf::Entity::find()
            .filter(pf::Column::UserId.eq(user_id))
            .filter(pf::Column::PersonId.eq(person_id))
            .inner_join(image_face_cache::Entity)
            .filter(image_face_cache::Column::SourceApp.eq(source_app))
            .filter(image_face_cache::Column::SourceId.eq(source_id))
            .count(db)
            .await?;
        if remaining == 0 {
            pm::Entity::delete_many()
                .filter(pm::Column::UserId.eq(user_id))
                .filter(pm::Column::PersonId.eq(person_id))
                .filter(pm::Column::SourceApp.eq(source_app))
                .filter(pm::Column::SourceId.eq(source_id))
                .exec(db)
                .await?;
        }
        Ok(())
    }

    pub async fn delete_source<C: ConnectionTrait + TransactionTrait>(
        db: &C,
        source_app: &str,
        source_id: &str,
    ) -> Result<SourceDelete, AppError> {
        let txn = db.begin().await?;

        let affected_person_ids = pf::Entity::find()
            .select_only()
            .column(pf::Column::PersonId)
            .inner_join(image_face_cache::Entity)
            .filter(image_face_cache::Column::SourceApp.eq(source_app))
            .filter(image_face_cache::Column::SourceId.eq(source_id))
            .distinct()
            .into_tuple::<Uuid>()
            .all(&txn)
            .await?;

        let deleted_media = Self::delete_media_by_source(&txn, source_app, source_id).await?;
        let deleted_cache = image_face_cache::Entity::delete_many()
            .filter(image_face_cache::Column::SourceApp.eq(source_app))
            .filter(image_face_cache::Column::SourceId.eq(source_id))
            .exec(&txn)
            .await?
            .rows_affected;

        for person_id in &affected_person_ids {
            Self::recount_person(&txn, *person_id).await?;
        }

        if !affected_person_ids.is_empty() {
            Entity::delete_many()
                .filter(Column::Id.eq_any(affected_person_ids.clone()))
                .filter(Column::FaceCount.eq(0))
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;
        Ok(SourceDelete {
            deleted_cache,
            deleted_media,
            affected_persons: affected_person_ids.len() as u64,
        })
    }

    pub async fn merge_persons<C: ConnectionTrait + TransactionTrait>(
        db: &C,
        user_id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), AppError> {
        let txn = db.begin().await?;

        let _source = Self::get_by_id(&txn, source_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("source person not found".into()))?;
        let _target = Self::get_by_id(&txn, target_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("target person not found".into()))?;

        pf::Entity::update_many()
            .col_expr(pf::Column::PersonId, Expr::value(target_id))
            .filter(pf::Column::PersonId.eq(source_id))
            .filter(pf::Column::UserId.eq(user_id))
            .exec(&txn)
            .await?;

        let copy_media = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r"
            INSERT INTO person_media (id, user_id, person_id, source_app, source_id, created_at)
            SELECT gen_random_uuid(), user_id, $1, source_app, source_id, created_at
            FROM person_media
            WHERE user_id = $2 AND person_id = $3
            ON CONFLICT (user_id, person_id, source_app, source_id) DO NOTHING
            ",
            [target_id.into(), user_id.into(), source_id.into()],
        );
        txn.execute_raw(copy_media).await?;

        pm::Entity::delete_many()
            .filter(pm::Column::UserId.eq(user_id))
            .filter(pm::Column::PersonId.eq(source_id))
            .exec(&txn)
            .await?;

        Self::recount_person(&txn, target_id).await?;

        Entity::update_many()
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now().fixed_offset()))
            .filter(Column::Id.eq(target_id))
            .filter(Column::UserId.eq(user_id))
            .exec(&txn)
            .await?;

        Entity::delete_many()
            .filter(Column::Id.eq(source_id))
            .filter(Column::UserId.eq(user_id))
            .exec(&txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }

    pub async fn assign_face<C: ConnectionTrait + TransactionTrait>(
        db: &C,
        user_id: Uuid,
        person_id: Uuid,
        face_cache_id: Uuid,
    ) -> Result<FaceMatch, AppError> {
        let txn = db.begin().await?;

        let _target = Self::get_by_id(&txn, person_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("target person not found".into()))?;
        let face = image_face_cache::Entity::find_by_id(face_cache_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("face not found in cache".into()))?;

        let existing = Self::get_face_link(&txn, user_id, face_cache_id).await?;
        match existing {
            Some(link) if link.person_id == person_id => {
                Self::link_media(&txn, user_id, person_id, &face.source_app, &face.source_id).await?;
                Self::recount_person(&txn, person_id).await?;
            }
            Some(link) => {
                pf::Entity::update_many()
                    .col_expr(pf::Column::PersonId, Expr::value(person_id))
                    .filter(pf::Column::UserId.eq(user_id))
                    .filter(pf::Column::FaceCacheId.eq(face_cache_id))
                    .exec(&txn)
                    .await?;
                Self::link_media(&txn, user_id, person_id, &face.source_app, &face.source_id).await?;
                Self::recount_person(&txn, link.person_id).await?;
                Self::recount_person(&txn, person_id).await?;
                Self::delete_person_media_if_source_empty(
                    &txn,
                    user_id,
                    link.person_id,
                    &face.source_app,
                    &face.source_id,
                )
                .await?;
                Entity::delete_many()
                    .filter(Column::Id.eq(link.person_id))
                    .filter(Column::UserId.eq(user_id))
                    .filter(Column::FaceCount.eq(0))
                    .exec(&txn)
                    .await?;
            }
            None => {
                Self::link_face(&txn, user_id, person_id, face_cache_id).await?;
                Self::link_media(&txn, user_id, person_id, &face.source_app, &face.source_id).await?;
                Self::recount_person(&txn, person_id).await?;
            }
        }

        txn.commit().await?;
        Ok(FaceMatch {
            face_cache_id: face.id,
            person_id,
            bbox: face.bbox,
            is_new: false,
            similarity: 1.0,
        })
    }

    pub async fn create_person_from_face<C: ConnectionTrait + TransactionTrait>(
        db: &C,
        user_id: Uuid,
        face_cache_id: Uuid,
        name: Option<String>,
    ) -> Result<FaceMatch, AppError> {
        let txn = db.begin().await?;

        let face = image_face_cache::Entity::find_by_id(face_cache_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("face not found in cache".into()))?;
        let old_link = Self::get_face_link(&txn, user_id, face_cache_id).await?;
        let person = Self::create(&txn, user_id, name).await?;

        match old_link {
            Some(link) => {
                pf::Entity::update_many()
                    .col_expr(pf::Column::PersonId, Expr::value(person.id))
                    .filter(pf::Column::UserId.eq(user_id))
                    .filter(pf::Column::FaceCacheId.eq(face_cache_id))
                    .exec(&txn)
                    .await?;
                Self::recount_person(&txn, link.person_id).await?;
                Self::delete_person_media_if_source_empty(
                    &txn,
                    user_id,
                    link.person_id,
                    &face.source_app,
                    &face.source_id,
                )
                .await?;
                Entity::delete_many()
                    .filter(Column::Id.eq(link.person_id))
                    .filter(Column::UserId.eq(user_id))
                    .filter(Column::FaceCount.eq(0))
                    .exec(&txn)
                    .await?;
            }
            None => {
                Self::link_face(&txn, user_id, person.id, face_cache_id).await?;
            }
        }

        Self::link_media(&txn, user_id, person.id, &face.source_app, &face.source_id).await?;
        Self::recount_person(&txn, person.id).await?;

        txn.commit().await?;
        Ok(FaceMatch {
            face_cache_id: face.id,
            person_id: person.id,
            bbox: face.bbox,
            is_new: true,
            similarity: 1.0,
        })
    }

    pub async fn match_face<C: ConnectionTrait + TransactionTrait>(
        db: &C,
        user_id: Uuid,
        face_cache_id: Uuid,
        threshold: f64,
    ) -> Result<FaceMatch, AppError> {
        let txn = db.begin().await?;

        let face = image_face_cache::Entity::find_by_id(face_cache_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("face not found in cache".into()))?;

        if let Some(link) = Self::get_face_link(&txn, user_id, face.id).await? {
            txn.commit().await?;
            return Ok(FaceMatch {
                face_cache_id: face.id,
                person_id: link.person_id,
                bbox: face.bbox,
                is_new: false,
                similarity: 1.0,
            });
        }

        let (person_id, is_new, similarity) = if let Some((person_id, similarity)) =
            Self::find_closest_person(&txn, user_id, face.id, threshold).await?
        {
            (person_id, false, similarity)
        } else {
            let person = Self::create(&txn, user_id, None).await?;
            (person.id, true, 0.0)
        };

        Self::link_face(&txn, user_id, person_id, face.id).await?;
        Self::increment_face_count(&txn, person_id).await?;
        Self::link_media(&txn, user_id, person_id, &face.source_app, &face.source_id).await?;

        txn.commit().await?;
        Ok(FaceMatch {
            face_cache_id: face.id,
            person_id,
            bbox: face.bbox,
            is_new,
            similarity,
        })
    }

    async fn find_closest_person<C: ConnectionTrait>(
        db: &C,
        user_id: Uuid,
        face_cache_id: Uuid,
        threshold: f64,
    ) -> Result<Option<(Uuid, f64)>, AppError> {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r"
            WITH query_face AS (
                SELECT embedding
                FROM image_face_cache
                WHERE id = $2
            ),
            nearest AS (
                SELECT pf.person_id,
                       1 - (ifc.embedding <=> q.embedding) AS similarity
                FROM person_faces pf
                JOIN image_face_cache ifc ON ifc.id = pf.face_cache_id
                CROSS JOIN query_face q
                WHERE pf.user_id = $1
                  AND ifc.id <> $2
                ORDER BY ifc.embedding <=> q.embedding
                LIMIT 50
            )
            SELECT person_id,
                   AVG(similarity)::float8 AS avg_sim
            FROM nearest
            GROUP BY person_id
            HAVING AVG(similarity) > $3
            ORDER BY AVG(similarity) DESC
            LIMIT 1
            ",
            [user_id.into(), face_cache_id.into(), threshold.into()],
        );
        let Some(row) = db.query_one_raw(stmt).await? else {
            return Ok(None);
        };
        let person_id: Uuid = row.try_get("", "person_id")?;
        let similarity: f64 = row.try_get("", "avg_sim")?;
        Ok(Some((person_id, similarity)))
    }

    async fn recount_person<C: ConnectionTrait>(db: &C, person_id: Uuid) -> Result<(), AppError> {
        let count = Ord::min(
            pf::Entity::find()
                .filter(pf::Column::PersonId.eq(person_id))
                .count(db)
                .await?,
            i32::MAX as u64,
        ) as i32;
        Entity::update_many()
            .col_expr(Column::FaceCount, Expr::value(count))
            .col_expr(Column::UpdatedAt, Expr::value(Utc::now().fixed_offset()))
            .filter(Column::Id.eq(person_id))
            .exec(db)
            .await?;
        Ok(())
    }
}
