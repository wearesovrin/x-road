use crate::entities::approved_tsa;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

/// Service for managing Approved Time-Stamping Authorities (TSAs).
#[derive(Clone, Copy, Default, Debug)]
pub struct TsaService;

impl TsaService {
    /// Registers an Approved Time-Stamping Authority (TSA).
    pub async fn add_approved_tsa(
        db: &DatabaseConnection,
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
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add approved TSA: {e}")))
    }

    pub async fn get_approved_tsa(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<approved_tsa::Model>> {
        approved_tsa::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get TSA: {e}")))
    }

    /// Lists all Approved Time-Stamping Authorities.
    pub async fn list_approved_tsas(
        db: &DatabaseConnection,
    ) -> XRoadResult<Vec<approved_tsa::Model>> {
        approved_tsa::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list approved TSAs: {e}")))
    }

    pub async fn delete_approved_tsa(db: &DatabaseConnection, id: i64) -> XRoadResult<bool> {
        let res = approved_tsa::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete TSA: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
