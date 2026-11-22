use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "management_requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub request_type: String,
    pub origin: String,
    pub security_server_id: Option<i64>,
    pub client_id_str: Option<String>,
    pub auth_cert_pem: Option<String>,
    pub status: String,
    pub comments: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
