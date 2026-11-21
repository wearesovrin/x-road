use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "xroad_members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub xroad_instance: String,
    pub member_class: String,
    pub member_code: String,
    pub name: String,
    pub administrative_contact: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::subsystem::Entity")]
    Subsystem,
    #[sea_orm(has_many = "super::security_server::Entity")]
    SecurityServer,
}

impl Related<super::subsystem::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subsystem.def()
    }
}

impl Related<super::security_server::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SecurityServer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
