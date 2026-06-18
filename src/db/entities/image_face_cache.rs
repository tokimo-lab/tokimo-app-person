use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "image_face_cache")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text")]
    pub image_hash: String,
    #[sea_orm(column_type = "Text")]
    pub source_app: String,
    #[sea_orm(column_type = "Text")]
    pub source_id: String,
    pub face_index: i32,
    #[sea_orm(column_type = "JsonBinary")]
    pub bbox: Json,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::person_faces::Entity")]
    PersonFaces,
}

impl Related<super::person_faces::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PersonFaces.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
