use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "api_keys")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub key_id: String,
    pub encoded_key: String,
    pub label: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::api_key_role::Entity")]
    ApiKeyRole,
}

impl Related<super::api_key_role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiKeyRole.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
