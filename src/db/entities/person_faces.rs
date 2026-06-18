use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "person_faces")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub person_id: Uuid,
    pub face_cache_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::persons::Entity",
        from = "Column::PersonId",
        to = "super::persons::Column::Id"
    )]
    Person,
    #[sea_orm(
        belongs_to = "super::image_face_cache::Entity",
        from = "Column::FaceCacheId",
        to = "super::image_face_cache::Column::Id"
    )]
    FaceCache,
}

impl Related<super::persons::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Person.def()
    }
}

impl Related<super::image_face_cache::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FaceCache.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
