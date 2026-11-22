use crate::entities::management_request;
use crate::services::security_server::SecurityServerService;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;

/// Service for managing Management Requests in Central Server.
#[derive(Clone)]
pub struct ManagementRequestService {
    db: Arc<DatabaseConnection>,
    security_servers: SecurityServerService,
}

impl ManagementRequestService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            security_servers: SecurityServerService::new(db),
        }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn create_management_request(
        &self,
        request_type: &str,
        origin: &str,
        security_server_id: Option<i64>,
        client_id_str: Option<&str>,
        auth_cert_pem: Option<&str>,
        comments: Option<&str>,
    ) -> XRoadResult<management_request::Model> {
        let now = chrono::Utc::now();
        management_request::ActiveModel {
            request_type: Set(request_type.to_string()),
            origin: Set(origin.to_string()),
            security_server_id: Set(security_server_id),
            client_id_str: Set(client_id_str.map(ToString::to_string)),
            auth_cert_pem: Set(auth_cert_pem.map(ToString::to_string)),
            status: Set("WAITING_FOR_APPROVAL".to_string()),
            comments: Set(comments.map(ToString::to_string)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create management request: {e}")))
    }

    pub async fn list_management_requests(&self) -> XRoadResult<Vec<management_request::Model>> {
        management_request::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list management requests: {e}")))
    }

    pub async fn get_management_request(
        &self,
        id: i64,
    ) -> XRoadResult<Option<management_request::Model>> {
        management_request::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get management request: {e}")))
    }

    pub async fn approve_management_request(
        &self,
        id: i64,
    ) -> XRoadResult<management_request::Model> {
        let req = self.get_management_request(id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Management request not found: {id}"))
        })?;

        match req.request_type.as_str() {
            "CLIENT_REGISTRATION" | "AUTH_CERT_REGISTRATION" | "OWNER_CHANGE" => {
                if let (Some(server_id), Some(client_id_str)) =
                    (req.security_server_id, req.client_id_str.as_ref())
                {
                    if let Ok(client_id) = client_id_str.parse::<ClientId>() {
                        let _ = self
                            .security_servers
                            .add_security_server_client(server_id, &client_id)
                            .await;
                    }
                }
                if let (Some(server_id), Some(cert_pem)) =
                    (req.security_server_id, req.auth_cert_pem.as_ref())
                {
                    let _ = self
                        .security_servers
                        .add_security_server_auth_cert(server_id, cert_pem)
                        .await;
                }
            }
            "CLIENT_DELETION" => {
                if let (Some(server_id), Some(client_id_str)) =
                    (req.security_server_id, req.client_id_str.as_ref())
                {
                    let _ = self
                        .security_servers
                        .remove_security_server_client(server_id, client_id_str)
                        .await;
                }
            }
            _ => {}
        }

        let mut active: management_request::ActiveModel = req.into();
        active.status = Set("APPROVED".to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to approve management request: {e}")))
    }

    pub async fn decline_management_request(
        &self,
        id: i64,
    ) -> XRoadResult<management_request::Model> {
        let req = self.get_management_request(id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Management request not found: {id}"))
        })?;

        let mut active: management_request::ActiveModel = req.into();
        active.status = Set("DECLINED".to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to decline management request: {e}")))
    }
}
