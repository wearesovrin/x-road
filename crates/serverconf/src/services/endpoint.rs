use crate::entities::{access_right, endpoint, service, service_description};
use crate::services::client::ClientService;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Service for managing endpoints and endpoint access rights in Security Server configuration.
#[derive(Clone, Copy, Default, Debug)]
pub struct EndpointService;

impl EndpointService {
    /// Registers a custom endpoint on a service.
    pub async fn add_endpoint(
        db: &DatabaseConnection,
        client_id: &ClientId,
        service_code: &str,
        method: &str,
        path: &str,
    ) -> XRoadResult<endpoint::Model> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        endpoint::ActiveModel {
            client_id: Set(client_model.id),
            service_code: Set(service_code.to_string()),
            method: Set(method.to_uppercase()),
            path: Set(path.to_string()),
            generated: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add endpoint: {e}")))
    }

    /// Adds an endpoint directly to a service by its service ID.
    pub async fn add_service_endpoint(
        db: &DatabaseConnection,
        service_id: i64,
        method: &str,
        path: &str,
    ) -> XRoadResult<endpoint::Model> {
        let svc = service::Entity::find_by_id(service_id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("Service not found: {service_id}")))?;

        let desc = service_description::Entity::find_by_id(svc.service_description_id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service description: {e}")))?
            .ok_or_else(|| XRoadError::Configuration("Service description not found".into()))?;

        endpoint::ActiveModel {
            client_id: Set(desc.client_id),
            service_code: Set(svc.service_code),
            method: Set(method.to_uppercase()),
            path: Set(path.to_string()),
            generated: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add endpoint: {e}")))
    }

    /// Lists endpoints for a specific service code.
    pub async fn list_endpoints_by_service_code(
        db: &DatabaseConnection,
        service_code: &str,
    ) -> XRoadResult<Vec<endpoint::Model>> {
        endpoint::Entity::find()
            .filter(endpoint::Column::ServiceCode.eq(service_code))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoints: {e}")))
    }

    /// Lists all endpoints for a client.
    pub async fn list_endpoints(
        db: &DatabaseConnection,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<endpoint::Model>> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        endpoint::Entity::find()
            .filter(endpoint::Column::ClientId.eq(client_model.id))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoints: {e}")))
    }

    /// Gets an endpoint by ID.
    pub async fn get_endpoint(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<endpoint::Model>> {
        endpoint::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoint: {e}")))
    }

    /// Deletes an endpoint by ID.
    pub async fn delete_endpoint(db: &DatabaseConnection, id: i64) -> XRoadResult<()> {
        let ep = Self::get_endpoint(db, id).await?;
        if let Some(endpoint) = ep {
            let ep_ref = format!(
                "EP:{}:{}:{}",
                endpoint.service_code, endpoint.method, endpoint.path
            );
            let _ = access_right::Entity::delete_many()
                .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
                .exec(db)
                .await;

            endpoint::Entity::delete_by_id(id)
                .exec(db)
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete endpoint: {e}")))?;
        }
        Ok(())
    }

    /// Grants access for a client to an endpoint.
    pub async fn grant_endpoint_access(
        db: &DatabaseConnection,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        let ep = Self::get_endpoint(db, endpoint_id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Endpoint not found: {endpoint_id}"))
        })?;

        let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
        let existing = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(&ep_ref))
            .filter(access_right::Column::SubjectClientIdStr.eq(subject_id))
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoint access: {e}")))?;

        if existing.is_none() {
            access_right::ActiveModel {
                service_id_str: Set(ep_ref),
                subject_client_id_str: Set(subject_id.to_string()),
                rights_given: Set(chrono::Utc::now()),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to grant endpoint access: {e}")))?;
        }
        Ok(())
    }

    /// Revokes access for a client to an endpoint.
    pub async fn revoke_endpoint_access(
        db: &DatabaseConnection,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        let ep = Self::get_endpoint(db, endpoint_id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Endpoint not found: {endpoint_id}"))
        })?;

        let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
        access_right::Entity::delete_many()
            .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
            .filter(access_right::Column::SubjectClientIdStr.eq(subject_id))
            .exec(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to revoke endpoint access: {e}")))?;
        Ok(())
    }

    /// Lists clients with access rights to an endpoint.
    pub async fn list_endpoint_access_rights(
        db: &DatabaseConnection,
        endpoint_id: i64,
    ) -> XRoadResult<Vec<String>> {
        let ep = Self::get_endpoint(db, endpoint_id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Endpoint not found: {endpoint_id}"))
        })?;

        let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
        let rights = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoint access: {e}")))?;

        Ok(rights
            .into_iter()
            .map(|r| r.subject_client_id_str)
            .collect())
    }
}
