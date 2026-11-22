use crate::entities::approved_ca;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

/// Service for managing Approved Certification Authorities (CAs).
#[derive(Clone)]
pub struct CaService {
    db: Arc<DatabaseConnection>,
}

impl CaService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers an Approved Certification Authority (CA).
    pub async fn add_approved_ca(
        &self,
        name: &str,
        top_ca_cert: &[u8],
        auth_only: bool,
        cert_profile: Option<&str>,
    ) -> XRoadResult<approved_ca::Model> {
        approved_ca::ActiveModel {
            name: Set(name.to_string()),
            top_ca_cert: Set(top_ca_cert.to_vec()),
            authentication_only: Set(auth_only),
            cert_profile_info: Set(cert_profile.map(ToString::to_string)),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add approved CA: {e}")))
    }

    pub async fn get_approved_ca(&self, id: i64) -> XRoadResult<Option<approved_ca::Model>> {
        approved_ca::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get CA: {e}")))
    }

    /// Lists all Approved Certification Authorities.
    pub async fn list_approved_cas(&self) -> XRoadResult<Vec<approved_ca::Model>> {
        approved_ca::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list approved CAs: {e}")))
    }

    pub async fn delete_approved_ca(&self, id: i64) -> XRoadResult<bool> {
        let res = approved_ca::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete CA: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
