use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "persons")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    #[sea_orm(column_type = "Text", nullable)]
    pub name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub avatar_url: Option<String>,
    pub face_count: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::person_faces::Entity")]
    PersonFaces,
    #[sea_orm(has_many = "super::person_media::Entity")]
    PersonMedia,
}

impl Related<super::person_faces::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PersonFaces.def()
    }
}

impl Related<super::person_media::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PersonMedia.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
