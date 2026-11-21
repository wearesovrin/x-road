use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "global_group_members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub global_group_id: i64,
    pub identifier: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::global_group::Entity",
        from = "Column::GlobalGroupId",
        to = "super::global_group::Column::Id",
        on_delete = "Cascade"
    )]
    GlobalGroup,
}

impl Related<super::global_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GlobalGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
