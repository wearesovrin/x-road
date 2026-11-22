use crate::entities::approved_tsa;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

/// Service for managing Approved Time-Stamping Authorities (TSAs).
#[derive(Clone)]
pub struct TsaService {
    db: Arc<DatabaseConnection>,
}

impl TsaService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers an Approved Time-Stamping Authority (TSA).
    pub async fn add_approved_tsa(
        &self,
        name: &str,
        url: &str,
        cert: &[u8],
    ) -> XRoadResult<approved_tsa::Model> {
        approved_tsa::ActiveModel {
            name: Set(name.to_string()),
            url: Set(url.to_string()),
            cert: Set(cert.to_vec()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add approved TSA: {e}")))
    }

    pub async fn get_approved_tsa(&self, id: i64) -> XRoadResult<Option<approved_tsa::Model>> {
        approved_tsa::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get TSA: {e}")))
    }

    /// Lists all Approved Time-Stamping Authorities.
    pub async fn list_approved_tsas(&self) -> XRoadResult<Vec<approved_tsa::Model>> {
        approved_tsa::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list approved TSAs: {e}")))
    }

    pub async fn delete_approved_tsa(&self, id: i64) -> XRoadResult<bool> {
        let res = approved_tsa::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete TSA: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
