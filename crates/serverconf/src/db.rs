use crate::entities::{
    access_right, api_key, api_key_role, certificate, client, endpoint, group_member, local_group,
    service, service_description,
};
use crate::migration::Migrator;
use crate::{ServerConfProvider, ServiceType};
use common::{XRoadError, XRoadResult};
use identifiers::{ClientId, ServiceId};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

/// SeaORM-backed Security Server configuration database provider.
#[derive(Clone)]
pub struct DatabaseServerConf {
    db: Arc<DatabaseConnection>,
}

impl DatabaseServerConf {
    /// Creates a new `DatabaseServerConf` from an existing SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Connects to database at `db_url` and runs pending migrations.
    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url)
            .await
            .map_err(|e| XRoadError::Database(format!("Database connection failed: {e}")))?;
        Migrator::up(&db, None)
            .await
            .map_err(|e| XRoadError::Database(format!("Database migration failed: {e}")))?;
        Ok(Self::new(db))
    }

    /// Connects to an in-memory SQLite database and runs all migrations. Ideal for testing.
    pub async fn connect_in_memory() -> XRoadResult<Self> {
        Self::connect("sqlite::memory:").await
    }

    /// Access the underlying SeaORM database connection.
    pub fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers a client in the server configuration.
    pub async fn add_client(
        &self,
        client_id: &ClientId,
        is_auth: bool,
    ) -> XRoadResult<client::Model> {
        let existing = self.get_client(client_id).await?;
        if let Some(c) = existing {
            return Ok(c);
        }

        let new_client = client::ActiveModel {
            xroad_instance: Set(client_id.xroad_instance.clone()),
            member_class: Set(client_id.member_class.clone()),
            member_code: Set(client_id.member_code.clone()),
            subsystem_code: Set(client_id.subsystem_code.clone()),
            client_status: Set("REGISTERED".to_string()),
            is_authentication: Set(is_auth),
            ..Default::default()
        };

        new_client
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to register client: {e}")))
    }

    /// Look up client record by identifier.
    pub async fn get_client(&self, client_id: &ClientId) -> XRoadResult<Option<client::Model>> {
        let mut query = client::Entity::find()
            .filter(client::Column::XroadInstance.eq(&client_id.xroad_instance))
            .filter(client::Column::MemberClass.eq(&client_id.member_class))
            .filter(client::Column::MemberCode.eq(&client_id.member_code));

        if let Some(ref subsystem) = client_id.subsystem_code {
            query = query.filter(client::Column::SubsystemCode.eq(Some(subsystem.clone())));
        } else {
            query = query.filter(client::Column::SubsystemCode.is_null());
        }

        query
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query client: {e}")))
    }

    /// Lists all registered clients.
    pub async fn list_clients(&self) -> XRoadResult<Vec<ClientId>> {
        let clients = client::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list clients: {e}")))?;

        let mut result = Vec::new();
        for c in clients {
            let id = if let Some(sub) = c.subsystem_code {
                ClientId::subsystem_unchecked(c.xroad_instance, c.member_class, c.member_code, sub)
            } else {
                ClientId::member_unchecked(c.xroad_instance, c.member_class, c.member_code)
            };
            result.push(id);
        }

        Ok(result)
    }

    /// Registers a service for a client.
    pub async fn add_service(
        &self,
        client_id: &ClientId,
        service_code: &str,
        service_version: Option<&str>,
        url: &str,
        service_type: ServiceType,
    ) -> XRoadResult<()> {
        let client_model = self
            .get_client(client_id)
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
        .insert(self.raw_connection())
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
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert service: {e}")))?;

        Ok(())
    }

    /// Grants access for a client to invoke a service.
    pub async fn grant_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        let service_id_str = service_id.to_string();
        let client_id_str = client_id.to_string();

        let existing = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(&service_id_str))
            .filter(access_right::Column::SubjectClientIdStr.eq(&client_id_str))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Access right query failed: {e}")))?;

        if existing.is_none() {
            access_right::ActiveModel {
                service_id_str: Set(service_id_str),
                subject_client_id_str: Set(client_id_str),
                rights_given: Set(chrono::Utc::now()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to grant access: {e}")))?;
        }

        Ok(())
    }

    /// Revokes access for a client to invoke a service.
    pub async fn revoke_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        access_right::Entity::delete_many()
            .filter(access_right::Column::ServiceIdStr.eq(service_id.to_string()))
            .filter(access_right::Column::SubjectClientIdStr.eq(client_id.to_string()))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to revoke access: {e}")))?;

        Ok(())
    }

    /// Creates an API key with assigned roles.
    pub async fn create_api_key(
        &self,
        encoded_key: &str,
        label: Option<&str>,
        roles: &[&str],
    ) -> XRoadResult<String> {
        let key_id = uuid::Uuid::new_v4().to_string();

        let key_model = api_key::ActiveModel {
            key_id: Set(key_id.clone()),
            encoded_key: Set(encoded_key.to_string()),
            label: Set(label.map(ToString::to_string)),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert API key: {e}")))?;

        for role in roles {
            api_key_role::ActiveModel {
                api_key_id: Set(key_model.id),
                role: Set(role.to_string()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to insert API key role: {e}")))?;
        }

        Ok(key_id)
    }

    /// Validates an API key and returns its assigned roles if valid.
    pub async fn validate_api_key(&self, encoded_key: &str) -> XRoadResult<Option<Vec<String>>> {
        let key = api_key::Entity::find()
            .filter(api_key::Column::EncodedKey.eq(encoded_key))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to validate API key: {e}")))?;

        if let Some(key_model) = key {
            let roles = api_key_role::Entity::find()
                .filter(api_key_role::Column::ApiKeyId.eq(key_model.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query API key roles: {e}")))?;

            Ok(Some(roles.into_iter().map(|r| r.role).collect()))
        } else {
            Ok(None)
        }
    }

    /// Stores a certificate for a client.
    pub async fn store_certificate(
        &self,
        client_id: &ClientId,
        cert_der: &[u8],
    ) -> XRoadResult<()> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

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
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let certs = certificate::Entity::find()
            .filter(certificate::Column::ClientId.eq(client_model.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query certificates: {e}")))?;

        Ok(certs.into_iter().map(|c| c.data).collect())
    }

    /// Deletes a client and cascading dependencies.
    pub async fn delete_client(&self, client_id: &ClientId) -> XRoadResult<()> {
        let client_model = self.get_client(client_id).await?;
        if let Some(c) = client_model {
            client::Entity::delete_by_id(c.id)
                .exec(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete client: {e}")))?;
        }
        Ok(())
    }

    /// Updates status of a client (e.g., "REGISTERED", "DISABLED", "SAVED").
    pub async fn set_client_status(&self, client_id: &ClientId, status: &str) -> XRoadResult<()> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let mut active: client::ActiveModel = client_model.into();
        active.client_status = Set(status.to_string());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update client status: {e}")))?;
        Ok(())
    }

    /// Designates a client as the Security Server owner.
    pub async fn make_owner(&self, client_id: &ClientId) -> XRoadResult<()> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        let mut active: client::ActiveModel = client_model.into();
        active.is_authentication = Set(true);
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to set owner client: {e}")))?;
        Ok(())
    }

    /// Gets the current Security Server owner client.
    pub async fn get_owner(&self) -> XRoadResult<Option<client::Model>> {
        client::Entity::find()
            .filter(client::Column::IsAuthentication.eq(true))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query owner client: {e}")))
    }

    /// Lists service descriptions for a client.
    pub async fn list_service_descriptions(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service_description::Model>> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        service_description::Entity::find()
            .filter(service_description::Column::ClientId.eq(client_model.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service descriptions: {e}")))
    }

    /// Gets a service description by ID.
    pub async fn get_service_description(
        &self,
        id: i64,
    ) -> XRoadResult<Option<service_description::Model>> {
        service_description::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service description: {e}")))
    }

    /// Deletes a service description and its services.
    pub async fn delete_service_description(&self, id: i64) -> XRoadResult<()> {
        service::Entity::delete_many()
            .filter(service::Column::ServiceDescriptionId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete associated services: {e}"))
            })?;

        service_description::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete service description: {e}"))
            })?;
        Ok(())
    }

    /// Enables or disables a service description.
    pub async fn set_service_description_disabled(
        &self,
        id: i64,
        disabled: bool,
        notice: Option<&str>,
    ) -> XRoadResult<()> {
        let desc = service_description::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service description: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Service description not found: {id}"))
            })?;

        let mut active: service_description::ActiveModel = desc.into();
        active.disabled = Set(disabled);
        active.disabled_notice = Set(notice.map(ToString::to_string));
        active.update(self.raw_connection()).await.map_err(|e| {
            XRoadError::Database(format!("Failed to update service description: {e}"))
        })?;
        Ok(())
    }

    /// Lists services registered under a client.
    pub async fn list_services_for_client(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service::Model>> {
        let descriptions = self.list_service_descriptions(client_id).await?;
        let desc_ids: Vec<i64> = descriptions.into_iter().map(|d| d.id).collect();

        service::Entity::find()
            .filter(service::Column::ServiceDescriptionId.is_in(desc_ids))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list client services: {e}")))
    }

    /// Gets service by ID.
    pub async fn get_service_by_id(&self, id: i64) -> XRoadResult<Option<service::Model>> {
        service::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service: {e}")))
    }

    /// Updates service URL and timeout.
    pub async fn update_service(&self, id: i64, url: &str, timeout: i32) -> XRoadResult<()> {
        let svc = service::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query service: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("Service not found: {id}")))?;

        let mut active: service::ActiveModel = svc.into();
        active.url = Set(url.to_string());
        active.timeout = Set(timeout);
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update service: {e}")))?;
        Ok(())
    }

    /// Lists subjects (clients) with access rights to a service.
    pub async fn list_service_access_rights(
        &self,
        service_id_str: &str,
    ) -> XRoadResult<Vec<String>> {
        let rights = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(service_id_str))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query access rights: {e}")))?;

        Ok(rights
            .into_iter()
            .map(|r| r.subject_client_id_str)
            .collect())
    }

    /// Registers a custom endpoint on a service.
    pub async fn add_endpoint(
        &self,
        client_id: &ClientId,
        service_code: &str,
        method: &str,
        path: &str,
    ) -> XRoadResult<endpoint::Model> {
        let client_model = self
            .get_client(client_id)
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
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add endpoint: {e}")))
    }

    /// Lists all endpoints for a client.
    pub async fn list_endpoints(&self, client_id: &ClientId) -> XRoadResult<Vec<endpoint::Model>> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        endpoint::Entity::find()
            .filter(endpoint::Column::ClientId.eq(client_model.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoints: {e}")))
    }

    /// Gets an endpoint by ID.
    pub async fn get_endpoint(&self, id: i64) -> XRoadResult<Option<endpoint::Model>> {
        endpoint::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoint: {e}")))
    }

    /// Deletes an endpoint by ID.
    pub async fn delete_endpoint(&self, id: i64) -> XRoadResult<()> {
        let ep = self.get_endpoint(id).await?;
        if let Some(endpoint) = ep {
            let ep_ref = format!(
                "EP:{}:{}:{}",
                endpoint.service_code, endpoint.method, endpoint.path
            );
            let _ = access_right::Entity::delete_many()
                .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
                .exec(self.raw_connection())
                .await;

            endpoint::Entity::delete_by_id(id)
                .exec(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete endpoint: {e}")))?;
        }
        Ok(())
    }

    /// Grants access for a client to an endpoint.
    pub async fn grant_endpoint_access(
        &self,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        let ep = self.get_endpoint(endpoint_id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Endpoint not found: {endpoint_id}"))
        })?;

        let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
        let existing = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(&ep_ref))
            .filter(access_right::Column::SubjectClientIdStr.eq(subject_id))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoint access: {e}")))?;

        if existing.is_none() {
            access_right::ActiveModel {
                service_id_str: Set(ep_ref),
                subject_client_id_str: Set(subject_id.to_string()),
                rights_given: Set(chrono::Utc::now()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to grant endpoint access: {e}")))?;
        }
        Ok(())
    }

    /// Revokes access for a client to an endpoint.
    pub async fn revoke_endpoint_access(
        &self,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        let ep = self.get_endpoint(endpoint_id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Endpoint not found: {endpoint_id}"))
        })?;

        let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
        access_right::Entity::delete_many()
            .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
            .filter(access_right::Column::SubjectClientIdStr.eq(subject_id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to revoke endpoint access: {e}")))?;
        Ok(())
    }

    /// Lists clients with access rights to an endpoint.
    pub async fn list_endpoint_access_rights(&self, endpoint_id: i64) -> XRoadResult<Vec<String>> {
        let ep = self.get_endpoint(endpoint_id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Endpoint not found: {endpoint_id}"))
        })?;

        let ep_ref = format!("EP:{}:{}:{}", ep.service_code, ep.method, ep.path);
        let rights = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(ep_ref))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query endpoint access: {e}")))?;

        Ok(rights
            .into_iter()
            .map(|r| r.subject_client_id_str)
            .collect())
    }

    /// Creates a local group for a client.
    pub async fn create_local_group(
        &self,
        client_id: &ClientId,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<local_group::Model> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        local_group::ActiveModel {
            client_id: Set(client_model.id),
            group_code: Set(group_code.to_string()),
            description: Set(description.to_string()),
            updated: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create local group: {e}")))
    }

    /// Lists local groups for a client.
    pub async fn list_local_groups(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<local_group::Model>> {
        let client_model = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Client not found: {client_id}")))?;

        local_group::Entity::find()
            .filter(local_group::Column::ClientId.eq(client_model.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query local groups: {e}")))
    }

    /// Gets a local group by ID.
    pub async fn get_local_group(&self, id: i64) -> XRoadResult<Option<local_group::Model>> {
        local_group::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query local group: {e}")))
    }

    /// Deletes a local group by ID.
    pub async fn delete_local_group(&self, id: i64) -> XRoadResult<()> {
        group_member::Entity::delete_many()
            .filter(group_member::Column::LocalGroupId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete group members: {e}")))?;

        local_group::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete local group: {e}")))?;
        Ok(())
    }

    /// Adds a member client to a local group.
    pub async fn add_group_member(&self, group_id: i64, member_id: &ClientId) -> XRoadResult<()> {
        let member_str = member_id.to_string();
        let existing = group_member::Entity::find()
            .filter(group_member::Column::LocalGroupId.eq(group_id))
            .filter(group_member::Column::MemberClientIdStr.eq(&member_str))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query group member: {e}")))?;

        if existing.is_none() {
            group_member::ActiveModel {
                local_group_id: Set(group_id),
                member_client_id_str: Set(member_str),
                added: Set(chrono::Utc::now()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to add group member: {e}")))?;
        }
        Ok(())
    }

    /// Removes a member client from a local group.
    pub async fn remove_group_member(
        &self,
        group_id: i64,
        member_id: &ClientId,
    ) -> XRoadResult<()> {
        let member_str = member_id.to_string();
        group_member::Entity::delete_many()
            .filter(group_member::Column::LocalGroupId.eq(group_id))
            .filter(group_member::Column::MemberClientIdStr.eq(member_str))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to remove group member: {e}")))?;
        Ok(())
    }

    /// Lists member clients of a local group.
    pub async fn list_group_members(&self, group_id: i64) -> XRoadResult<Vec<String>> {
        let members = group_member::Entity::find()
            .filter(group_member::Column::LocalGroupId.eq(group_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query group members: {e}")))?;

        Ok(members
            .into_iter()
            .map(|m| m.member_client_id_str)
            .collect())
    }
}

#[tonic::async_trait]
impl ServerConfProvider for DatabaseServerConf {
    async fn is_client_allowed(&self, client_id: &ClientId) -> XRoadResult<bool> {
        let client = self.get_client(client_id).await?;
        Ok(client
            .map(|c| c.client_status == "REGISTERED")
            .unwrap_or(false))
    }

    async fn is_service_allowed(
        &self,
        client_id: &ClientId,
        service_id: &ServiceId,
    ) -> XRoadResult<bool> {
        let service_id_str = service_id.to_string();
        let client_id_str = client_id.to_string();

        let mut cond = sea_orm::sea_query::Condition::any()
            .add(access_right::Column::ServiceIdStr.eq(&service_id_str));

        if service_id.service_version.is_none() {
            cond = cond
                .add(access_right::Column::ServiceIdStr.starts_with(format!("{service_id_str}/")));
        } else {
            let without_version = ServiceId::new_unchecked(
                service_id.client.clone(),
                &service_id.service_code,
                None::<&str>,
            )
            .to_string();
            cond = cond.add(access_right::Column::ServiceIdStr.eq(without_version));
        }

        let mut subject_cond = sea_orm::sea_query::Condition::any()
            .add(access_right::Column::SubjectClientIdStr.eq(&client_id_str));

        // Check local group memberships
        if let Ok(memberships) = group_member::Entity::find()
            .filter(group_member::Column::MemberClientIdStr.eq(&client_id_str))
            .all(self.raw_connection())
            .await
        {
            let group_ids: Vec<i64> = memberships.into_iter().map(|m| m.local_group_id).collect();
            if !group_ids.is_empty() {
                if let Ok(groups) = local_group::Entity::find()
                    .filter(local_group::Column::Id.is_in(group_ids))
                    .all(self.raw_connection())
                    .await
                {
                    for g in groups {
                        subject_cond = subject_cond.add(
                            access_right::Column::SubjectClientIdStr
                                .eq(format!("GROUP:{}", g.group_code)),
                        );
                    }
                }
            }
        }

        let count = access_right::Entity::find()
            .filter(cond)
            .filter(subject_cond)
            .count(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to check service access: {e}")))?;

        Ok(count > 0)
    }

    async fn get_service_url(&self, service_id: &ServiceId) -> XRoadResult<Option<String>> {
        let mut query = service::Entity::find()
            .filter(service::Column::ServiceCode.eq(&service_id.service_code));

        if let Some(ref version) = service_id.service_version {
            query = query.filter(service::Column::ServiceVersion.eq(Some(version.clone())));
        }

        let svc = query
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get service URL: {e}")))?;

        Ok(svc.map(|s| s.url))
    }

    async fn list_services(&self, client_id: &ClientId) -> XRoadResult<Vec<ServiceId>> {
        let models = self.list_services_for_client(client_id).await?;
        let mut services = Vec::new();
        for m in models {
            if let Ok(svc) = ServiceId::new(
                client_id.clone(),
                &m.service_code,
                m.service_version.as_deref(),
            ) {
                services.push(svc);
            }
        }
        Ok(services)
    }

    async fn get_allowed_services(
        &self,
        producer: &ClientId,
        consumer: &ClientId,
    ) -> XRoadResult<Vec<ServiceId>> {
        let all_services = self.list_services(producer).await?;
        let mut allowed = Vec::new();
        for svc in all_services {
            if self
                .is_service_allowed(consumer, &svc)
                .await
                .unwrap_or(false)
            {
                allowed.push(svc);
            }
        }
        Ok(allowed)
    }
}
