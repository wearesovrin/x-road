use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "local_groups")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_id: i64,
    pub group_code: String,
    pub description: String,
    pub updated: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::client::Entity",
        from = "Column::ClientId",
        to = "super::client::Column::Id",
        on_delete = "Cascade"
    )]
    Client,
    #[sea_orm(has_many = "super::group_member::Entity")]
    GroupMember,
}

impl Related<super::client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Client.def()
    }
}

impl Related<super::group_member::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupMember.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
