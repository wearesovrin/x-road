use crate::entities::{security_server, security_server_auth_cert, security_server_client};
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Clone)]
pub struct SecurityServerService {
    db: Arc<DatabaseConnection>,
}

impl SecurityServerService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub fn conn(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers a security server under a member owner.
    pub async fn register_security_server(
        &self,
        owner_id: i64,
        server_code: &str,
        address: &str,
    ) -> XRoadResult<security_server::Model> {
        let now = chrono::Utc::now();
        security_server::ActiveModel {
            owner_id: Set(owner_id),
            server_code: Set(server_code.to_string()),
            address: Set(address.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.conn())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to register security server: {e}")))
    }

    pub async fn get_security_server(
        &self,
        id: i64,
    ) -> XRoadResult<Option<security_server::Model>> {
        security_server::Entity::find_by_id(id)
            .one(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get security server: {e}")))
    }

    pub async fn delete_security_server(&self, id: i64) -> XRoadResult<bool> {
        security_server_client::Entity::delete_many()
            .filter(security_server_client::Column::SecurityServerId.eq(id))
            .exec(self.conn())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete security server clients: {e}"))
            })?;

        security_server_auth_cert::Entity::delete_many()
            .filter(security_server_auth_cert::Column::SecurityServerId.eq(id))
            .exec(self.conn())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete security server auth certs: {e}"))
            })?;

        let res = security_server::Entity::delete_by_id(id)
            .exec(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete security server: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    /// Lists all security servers.
    pub async fn list_security_servers(&self) -> XRoadResult<Vec<security_server::Model>> {
        security_server::Entity::find()
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list security servers: {e}")))
    }

    /// Associates a client with a security server.
    pub async fn add_security_server_client(
        &self,
        security_server_id: i64,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        security_server_client::ActiveModel {
            security_server_id: Set(security_server_id),
            client_id_str: Set(client_id.to_string()),
            enabled: Set(true),
            ..Default::default()
        }
        .insert(self.conn())
        .await
        .map_err(|e| {
            XRoadError::Database(format!(
                "Failed to associate client with security server: {e}"
            ))
        })?;

        Ok(())
    }

    pub async fn list_security_server_clients(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<String>> {
        let clients = security_server_client::Entity::find()
            .filter(security_server_client::Column::SecurityServerId.eq(security_server_id))
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list server clients: {e}")))?;
        Ok(clients.into_iter().map(|c| c.client_id_str).collect())
    }

    pub async fn remove_security_server_client(
        &self,
        security_server_id: i64,
        client_id_str: &str,
    ) -> XRoadResult<bool> {
        let res = security_server_client::Entity::delete_many()
            .filter(security_server_client::Column::SecurityServerId.eq(security_server_id))
            .filter(security_server_client::Column::ClientIdStr.eq(client_id_str))
            .exec(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete server client: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    pub async fn add_security_server_auth_cert(
        &self,
        security_server_id: i64,
        cert_pem: &str,
    ) -> XRoadResult<security_server_auth_cert::Model> {
        let mut hasher = Sha256::new();
        hasher.update(cert_pem.as_bytes());
        let cert_hash = format!("{:x}", hasher.finalize());

        security_server_auth_cert::ActiveModel {
            security_server_id: Set(security_server_id),
            cert_pem: Set(cert_pem.to_string()),
            cert_hash: Set(cert_hash),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.conn())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add auth cert: {e}")))
    }

    pub async fn list_security_server_auth_certs(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<security_server_auth_cert::Model>> {
        security_server_auth_cert::Entity::find()
            .filter(security_server_auth_cert::Column::SecurityServerId.eq(security_server_id))
            .all(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list auth certs: {e}")))
    }

    pub async fn delete_security_server_auth_cert(&self, cert_id: i64) -> XRoadResult<bool> {
        let res = security_server_auth_cert::Entity::delete_by_id(cert_id)
            .exec(self.conn())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete auth cert: {e}")))?;
        Ok(res.rows_affected > 0)
    }
}
