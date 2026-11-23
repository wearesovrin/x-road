use crate::ServiceType;
use crate::entities::{access_right, service, service_description};
use crate::services::client::ClientService;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

/// Service for managing services and service descriptions in Security Server configuration.
#[derive(Clone, Copy, Default, Debug)]
pub struct ServiceService;

impl ServiceService {
    /// Registers a service for a client.
    pub async fn add_service(
        db: &DatabaseConnection,
        client_id: &ClientId,
        service_code: &str,
        service_version: Option<&str>,
        url: &str,
        service_type: ServiceType,
    ) -> XRoadResult<service_description::Model> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let desc = service_description::ActiveModel {
            client_id: Set(client_model.id),
            url: Set(url.to_string()),
            service_type: Set(service_type.to_string()),
            disabled: Set(false),
            disabled_notice: Set(None),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert service description: {e}")))?;

        service::ActiveModel {
            service_description_id: Set(desc.id),
            service_code: Set(service_code.to_string()),
            service_version: Set(service_version.map(ToString::to_string)),
            title: Set(Some(service_code.to_string())),
            url: Set(url.to_string()),
            timeout: Set(60),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert service: {e}")))?;

        Ok(desc)
    }

    /// Lists service descriptions for a client.
    pub async fn list_service_descriptions(
        db: &DatabaseConnection,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service_description::Model>> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        service_description::Entity::find()
            .filter(service_description::Column::ClientId.eq(client_model.id))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service descriptions: {e}")))
    }

    /// Gets a service description by ID.
    pub async fn get_service_description(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<service_description::Model>> {
        service_description::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service description: {e}")))
    }

    /// Deletes a service description and its services.
    pub async fn delete_service_description(db: &DatabaseConnection, id: i64) -> XRoadResult<()> {
        service::Entity::delete_many()
            .filter(service::Column::ServiceDescriptionId.eq(id))
            .exec(db)
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete associated services: {e}"))
            })?;

        service_description::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete service description: {e}"))
            })?;
        Ok(())
    }

    /// Enables or disables a service description.
    pub async fn set_service_description_disabled(
        db: &DatabaseConnection,
        id: i64,
        disabled: bool,
        notice: Option<&str>,
    ) -> XRoadResult<()> {
        let desc = service_description::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service description: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Service description not found: {id}"))
            })?;

        let mut active: service_description::ActiveModel = desc.into();
        active.disabled = Set(disabled);
        active.disabled_notice = Set(notice.map(ToString::to_string));
        active.update(db).await.map_err(|e| {
            XRoadError::Database(format!("Failed to update service description: {e}"))
        })?;
        Ok(())
    }

    /// Lists services registered under a client.
    pub async fn list_services_for_client(
        db: &DatabaseConnection,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service::Model>> {
        let descriptions = Self::list_service_descriptions(db, client_id).await?;
        let desc_ids: Vec<i64> = descriptions.into_iter().map(|d| d.id).collect();

        service::Entity::find()
            .filter(service::Column::ServiceDescriptionId.is_in(desc_ids))
            .all(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list client services: {e}")))
    }

    /// Gets service by ID.
    pub async fn get_service_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> XRoadResult<Option<service::Model>> {
        service::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service: {e}")))
    }

    /// Updates service URL and timeout.
    pub async fn update_service(
        db: &DatabaseConnection,
        id: i64,
        url: &str,
        timeout: i32,
    ) -> XRoadResult<()> {
        let svc = service::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("Service not found: {id}")))?;

        let mut active: service::ActiveModel = svc.into();
        active.url = Set(url.to_string());
        active.timeout = Set(timeout);
        active
            .update(db)
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update service: {e}")))?;
        Ok(())
    }

    /// Deletes a service and its cascading access rights.
    pub async fn delete_service_cascade(
        db: &DatabaseConnection,
        client_id: &ClientId,
        service_code: &str,
    ) -> XRoadResult<()> {
        let client_model = ClientService::get_client(db, client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let descriptions = service_description::Entity::find()
            .filter(service_description::Column::ClientId.eq(client_model.id))
            .all(db)
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to query service descriptions: {e}"))
            })?;

        for desc in descriptions {
            let svc = service::Entity::find()
                .filter(service::Column::ServiceDescriptionId.eq(desc.id))
                .filter(service::Column::ServiceCode.eq(service_code))
                .one(db)
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query service: {e}")))?;

            if let Some(s) = svc {
                let svc_prefix = format!("{}/{}", client_id, service_code);
                let _ = access_right::Entity::delete_many()
                    .filter(access_right::Column::ServiceIdStr.contains(&svc_prefix))
                    .exec(db)
                    .await;

                service::Entity::delete_by_id(s.id)
                    .exec(db)
                    .await
                    .map_err(|e| XRoadError::Database(format!("Failed to delete service: {e}")))?;
            }
        }
        Ok(())
    }
}
