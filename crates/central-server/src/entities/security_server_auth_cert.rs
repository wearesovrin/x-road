use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "security_server_auth_certs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub security_server_id: i64,
    pub cert_pem: String,
    pub cert_hash: String,
    pub created_at: DateTime<Utc>,
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
