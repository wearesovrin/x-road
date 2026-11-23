use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "clients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub xroad_instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub client_status: String,
    pub is_authentication: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::certificate::Entity")]
    Certificate,
    #[sea_orm(has_many = "super::service_description::Entity")]
    ServiceDescription,
    #[sea_orm(has_many = "super::endpoint::Entity")]
    Endpoint,
    #[sea_orm(has_many = "super::local_group::Entity")]
    LocalGroup,
}

impl Related<super::certificate::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Certificate.def()
    }
}

impl Related<super::service_description::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceDescription.def()
    }
}

impl Related<super::endpoint::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Endpoint.def()
    }
}

impl Related<super::local_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LocalGroup.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
