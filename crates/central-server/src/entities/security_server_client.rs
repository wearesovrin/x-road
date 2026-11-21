use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "security_server_clients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub security_server_id: i64,
    pub client_id_str: String,
    pub enabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::security_server::Entity",
        from = "Column::SecurityServerId",
        to = "super::security_server::Column::Id",
        on_delete = "Cascade"
    )]
    SecurityServer,
}

impl Related<super::security_server::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SecurityServer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
