use crate::entities::{access_right, api_key, api_key_role, certificate, client, service, service_description};
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
    pub async fn add_client(&self, client_id: &ClientId, is_auth: bool) -> XRoadResult<client::Model> {
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
    pub async fn grant_access(&self, service_id: &ServiceId, client_id: &ClientId) -> XRoadResult<()> {
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
    pub async fn revoke_access(&self, service_id: &ServiceId, client_id: &ClientId) -> XRoadResult<()> {
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
    pub async fn store_certificate(&self, client_id: &ClientId, cert_der: &[u8]) -> XRoadResult<()> {
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
}

#[tonic::async_trait]
impl ServerConfProvider for DatabaseServerConf {
    async fn is_client_allowed(&self, client_id: &ClientId) -> XRoadResult<bool> {
        let client = self.get_client(client_id).await?;
        Ok(client.map(|c| c.client_status == "REGISTERED").unwrap_or(false))
    }

    async fn is_service_allowed(
        &self,
        client_id: &ClientId,
        service_id: &ServiceId,
    ) -> XRoadResult<bool> {
        let service_id_str = service_id.to_string();
        let client_id_str = client_id.to_string();

        let count = access_right::Entity::find()
            .filter(access_right::Column::ServiceIdStr.eq(service_id_str))
            .filter(access_right::Column::SubjectClientIdStr.eq(client_id_str))
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
}
