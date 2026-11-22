use crate::entities::certificate;
use crate::services::client::ClientService;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

/// Service for managing client certificates in Security Server configuration.
#[derive(Clone)]
pub struct CertificateService {
    db: Arc<DatabaseConnection>,
    clients: ClientService,
}

impl CertificateService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            clients: ClientService::new(db),
        }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Stores a certificate for a client.
    pub async fn store_certificate(
        &self,
        client_id: &ClientId,
        cert_der: &[u8],
    ) -> XRoadResult<()> {
        let client_model =
            self.clients.get_client(client_id).await?.ok_or_else(|| {
                XRoadError::Configuration(format!("Client not found: {client_id}"))
            })?;

        certificate::ActiveModel {
            client_id: Set(client_model.id),
            data: Set(cert_der.to_vec()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to store certificate: {e}")))?;

        Ok(())
    }

    /// Retrieves all certificates for a client.
    pub async fn get_certificates(&self, client_id: &ClientId) -> XRoadResult<Vec<Vec<u8>>> {
        let client_model =
            self.clients.get_client(client_id).await?.ok_or_else(|| {
                XRoadError::Configuration(format!("Client not found: {client_id}"))
            })?;

        let certs = certificate::Entity::find()
            .filter(certificate::Column::ClientId.eq(client_model.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query certificates: {e}")))?;

        Ok(certs.into_iter().map(|c| c.data).collect())
    }
}
