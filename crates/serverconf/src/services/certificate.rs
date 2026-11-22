use crate::entities::certificate;
use crate::services::client::ClientService;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Service for managing client certificates in Security Server configuration.
#[derive(Clone, Copy, Default, Debug)]
pub struct CertificateService;

impl CertificateService {
    /// Stores a certificate for a client.
    pub async fn store_certificate(
        db: &DatabaseConnection,
        client_id: &ClientId,
        cert_der: &[u8],
    ) -> XRoadResult<()> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        certificate::ActiveModel {
            client_id: Set(client_model.id),
            data: Set(cert_der.to_vec()),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to store certificate: {e}")))?;

        Ok(())
    }

    /// Retrieves all certificates for a client.
    pub async fn get_certificates(
        db: &DatabaseConnection,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<Vec<u8>>> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let certs = certificate::Entity::find()
            .filter(certificate::Column::ClientId.eq(client_model.id))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query certificates: {e}")))?;

        Ok(certs.into_iter().map(|c| c.data).collect())
    }
}
