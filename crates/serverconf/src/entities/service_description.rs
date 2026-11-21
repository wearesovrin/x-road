use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "service_descriptions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_id: i64,
    pub url: String,
    pub service_type: String,
    pub disabled: bool,
    pub disabled_notice: Option<String>,
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
    #[sea_orm(has_many = "super::service::Entity")]
    Service,
}

impl Related<super::client::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Client.def()
    }
}

impl Related<super::service::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Service.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
