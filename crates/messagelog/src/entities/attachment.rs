use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "message_attachments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub logrecord_id: i64,
    pub attachment_no: i32,
    pub attachment_hash: String,
    pub data: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::log_record::Entity",
        from = "Column::LogrecordId",
        to = "super::log_record::Column::Id",
        on_delete = "Cascade"
    )]
    LogRecord,
}

impl Related<super::log_record::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LogRecord.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
