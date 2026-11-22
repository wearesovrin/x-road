use crate::entities::approved_ca;
use common::{XRoadError, XRoadResult};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

/// Service for managing Approved Certification Authorities (CAs).
#[derive(Clone, Copy, Default, Debug)]
pub struct CaService;

impl CaService {
    /// Registers an Approved Certification Authority (CA).
    pub async fn add_approved_ca(
        db: &DatabaseConnection,
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
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add approved CA: {e}")))
    }

    pub async fn get_approved_ca(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<approved_ca::Model>> {
        approved_ca::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get CA: {e}")))
    }

    /// Lists all Approved Certification Authorities.
    pub async fn list_approved_cas(
        db: &DatabaseConnection,
    ) -> XRoadResult<Vec<approved_ca::Model>> {
        approved_ca::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list approved CAs: {e}")))
    }

    pub async fn delete_approved_ca(db: &DatabaseConnection, id: i64) -> XRoadResult<bool> {
        let res = approved_ca::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete CA: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
