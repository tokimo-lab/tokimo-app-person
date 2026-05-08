use crate::db::entities::items::{self, ActiveModel, Column, Entity};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

pub struct ItemsRepo;

impl ItemsRepo {
    pub async fn list_by_user(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<items::Model>, DbErr> {
        Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .limit(100)
            .all(db)
            .await
    }

    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid, user_id: Uuid) -> Result<Option<items::Model>, DbErr> {
        Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    pub async fn create(db: &DatabaseConnection, user_id: Uuid, content: String) -> Result<items::Model, DbErr> {
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            content: Set(content),
            user_id: Set(user_id),
            created_at: Set(Utc::now().into()),
        };
        am.insert(db).await
    }

    pub async fn update(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
        content: String,
    ) -> Result<Option<items::Model>, DbErr> {
        let Some(model) = Self::find_by_id(db, id, user_id).await? else {
            return Ok(None);
        };
        let mut am: ActiveModel = model.into();
        am.content = Set(content);
        Ok(Some(am.update(db).await?))
    }

    pub async fn delete(db: &DatabaseConnection, id: Uuid, user_id: Uuid) -> Result<u64, DbErr> {
        let result = Entity::delete_many()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
