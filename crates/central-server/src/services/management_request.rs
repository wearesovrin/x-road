use crate::entities::management_request;
use crate::services::security_server::SecurityServerService;
use common::{ManagementRequestStatus, ManagementRequestType, XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

/// Service for managing Management Requests in Central Server.
#[derive(Clone, Copy, Default, Debug)]
pub struct ManagementRequestService;

impl ManagementRequestService {
    pub async fn create_management_request(
        db: &DatabaseConnection,
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
            status: Set(ManagementRequestStatus::WaitingForApproval.to_string()),
            comments: Set(comments.map(ToString::to_string)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create management request: {e}")))
    }

    pub async fn list_management_requests(
        db: &DatabaseConnection,
    ) -> XRoadResult<Vec<management_request::Model>> {
        management_request::Entity::find()
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list management requests: {e}")))
    }

    pub async fn get_management_request(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<management_request::Model>> {
        management_request::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get management request: {e}")))
    }

    pub async fn approve_management_request(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<management_request::Model> {
        let req = Self::get_management_request(db, id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Management request not found: {id}"))
        })?;

        match req.request_type.parse::<ManagementRequestType>() {
            Ok(ManagementRequestType::ClientRegistration)
            | Ok(ManagementRequestType::AuthCertRegistration)
            | Ok(ManagementRequestType::OwnerChange) => {
                if let (Some(server_id), Some(client_id_str)) =
                    (req.security_server_id, req.client_id_str.as_ref())
                {
                    if let Ok(client_id) = client_id_str.parse::<ClientId>() {
                        let _ = SecurityServerService::add_security_server_client(
                            db, server_id, &client_id,
                        )
                        .await;
                    }
                }
                if let (Some(server_id), Some(cert_pem)) =
                    (req.security_server_id, req.auth_cert_pem.as_ref())
                {
                    let _ = SecurityServerService::add_security_server_auth_cert(
                        db, server_id, cert_pem,
                    )
                    .await;
                }
            }
            Ok(ManagementRequestType::ClientDeletion) => {
                if let (Some(server_id), Some(client_id_str)) =
                    (req.security_server_id, req.client_id_str.as_ref())
                {
                    let _ = SecurityServerService::remove_security_server_client(
                        db,
                        server_id,
                        client_id_str,
                    )
                    .await;
                }
            }
            _ => {}
        }

        let mut active: management_request::ActiveModel = req.into();
        active.status = Set(ManagementRequestStatus::Approved.to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to approve management request: {e}")))
    }

    pub async fn decline_management_request(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<management_request::Model> {
        let req = Self::get_management_request(db, id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Management request not found: {id}"))
        })?;

        let mut active: management_request::ActiveModel = req.into();
        active.status = Set(ManagementRequestStatus::Declined.to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to decline management request: {e}")))
    }
}
