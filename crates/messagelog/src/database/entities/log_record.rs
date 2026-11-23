use chrono::{DateTime, Utc};
use common::MessageProtocol;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "log_records")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub query_id: String,
    pub x_request_id: Option<String>,
    pub client_id_str: Option<String>,
    pub service_id_str: Option<String>,
    pub discriminator: String,
    pub message_hash: String,
    pub signature_hash: Option<String>,
    pub message_body: Option<String>,
    pub signature_data: Option<Vec<u8>>,
    pub hash_chain: Option<String>,
    pub hash_chain_result: Option<String>,
    pub timestamp_record_id: Option<i64>,
    pub timestamp_hash_chain: Option<String>,
    pub response: bool,
    pub archived: bool,
    pub time: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::attachment::Entity")]
    Attachment,
}

impl Related<super::attachment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attachment.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Attempts to parse the client identifier string into a structured ClientId.
    pub fn client_id(&self) -> Option<identifiers::ClientId> {
        self.client_id_str.as_deref().and_then(|s| s.parse().ok())
    }

    /// Returns the parsed MessageProtocol enum for this record.
    pub fn protocol(&self) -> MessageProtocol {
        self.discriminator.parse().unwrap_or(MessageProtocol::Rest)
    }
}
