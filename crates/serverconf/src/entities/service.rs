use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "services")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub service_description_id: i64,
    pub service_code: String,
    pub service_version: Option<String>,
    pub title: Option<String>,
    pub url: String,
    pub timeout: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::service_description::Entity",
        from = "Column::ServiceDescriptionId",
        to = "super::service_description::Column::Id",
        on_delete = "Cascade"
    )]
    ServiceDescription,
}

impl Related<super::service_description::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ServiceDescription.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
