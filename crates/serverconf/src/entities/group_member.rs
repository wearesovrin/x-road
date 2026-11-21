use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "group_members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub local_group_id: i64,
    pub member_client_id_str: String,
    pub added: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::local_group::Entity",
        from = "Column::LocalGroupId",
        to = "super::local_group::Column::Id",
        on_delete = "Cascade"
    )]
    LocalGroup,
}

impl Related<super::local_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LocalGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
