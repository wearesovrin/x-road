pub mod access_right;
pub mod api_key;
pub mod certificate;
pub mod client;
pub mod endpoint;
pub mod local_group;
pub mod service;

pub use access_right::AccessRightService;
pub use api_key::ApiKeyService;
pub use certificate::CertificateService;
pub use client::ClientService;
pub use endpoint::EndpointService;
pub use local_group::LocalGroupService;
pub use service::ServiceService;

use crate::entities::{
    access_right as ar_entity, api_key as api_key_entity, client as client_entity,
    endpoint as ep_entity, group_member, local_group as lg_entity, service as service_entity,
    service_description,
};
use crate::{ServerConfProvider, ServiceType};
use common::{XRoadError, XRoadResult};
use identifiers::{ClientId, ServiceId};
use sea_orm::{
    ColumnTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

/// SeaORM-backed Security Server configuration services provider.
#[derive(Clone)]
pub struct ServerConf {
    pub raw_db: Arc<DatabaseConnection>,
    pub clients: ClientService,
    pub services: ServiceService,
    pub access_rights: AccessRightService,
    pub endpoints: EndpointService,
    pub local_groups: LocalGroupService,
    pub certificates: CertificateService,
    pub api_keys: ApiKeyService,
}

impl ServerConf {
    /// Creates a new `ServerConf` from an existing SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        let db_arc = Arc::new(db);
        Self {
            clients: ClientService::new(db_arc.clone()),
            services: ServiceService::new(db_arc.clone()),
            access_rights: AccessRightService::new(db_arc.clone()),
            endpoints: EndpointService::new(db_arc.clone()),
            local_groups: LocalGroupService::new(db_arc.clone()),
            certificates: CertificateService::new(db_arc.clone()),
            api_keys: ApiKeyService::new(db_arc.clone()),
            raw_db: db_arc,
        }
    }

    /// Connects to database at `db_url` and runs pending migrations.
    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url)
            .await
            .map_err(|e| XRoadError::Database(format!("Database connection failed: {e}")))?;
        crate::migration::Migrator::up(&db, None)
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
        &self.raw_db
    }

    // Direct access helper methods
    pub async fn add_client(
        &self,
        client_id: &ClientId,
        is_auth: bool,
    ) -> XRoadResult<client_entity::Model> {
        self.clients.add_client(client_id, is_auth).await
    }

    pub async fn get_client(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Option<client_entity::Model>> {
        self.clients.get_client(client_id).await
    }

    pub async fn list_clients(&self) -> XRoadResult<Vec<ClientId>> {
        self.clients.list_clients().await
    }

    pub async fn add_service(
        &self,
        client_id: &ClientId,
        service_code: &str,
        service_version: Option<&str>,
        url: &str,
        service_type: ServiceType,
    ) -> XRoadResult<()> {
        self.services
            .add_service(client_id, service_code, service_version, url, service_type)
            .await
    }

    pub async fn grant_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        self.access_rights.grant_access(service_id, client_id).await
    }

    pub async fn revoke_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        self.access_rights
            .revoke_access(service_id, client_id)
            .await
    }

    pub async fn create_api_key(
        &self,
        encoded_key: &str,
        label: Option<&str>,
        roles: &[&str],
    ) -> XRoadResult<String> {
        self.api_keys
            .create_api_key(encoded_key, label, roles)
            .await
    }

    pub async fn validate_api_key(&self, key_or_token: &str) -> XRoadResult<Option<Vec<String>>> {
        self.api_keys.validate_api_key(key_or_token).await
    }

    pub async fn list_api_keys(&self) -> XRoadResult<Vec<(api_key_entity::Model, Vec<String>)>> {
        self.api_keys.list_api_keys().await
    }

    pub async fn get_api_key(
        &self,
        id: i64,
    ) -> XRoadResult<Option<(api_key_entity::Model, Vec<String>)>> {
        self.api_keys.get_api_key(id).await
    }

    pub async fn update_api_key_roles(&self, id: i64, roles: &[&str]) -> XRoadResult<()> {
        self.api_keys.update_api_key_roles(id, roles).await
    }

    pub async fn revoke_api_key(&self, id: i64) -> XRoadResult<()> {
        self.api_keys.revoke_api_key(id).await
    }

    pub async fn store_certificate(
        &self,
        client_id: &ClientId,
        cert_der: &[u8],
    ) -> XRoadResult<()> {
        self.certificates
            .store_certificate(client_id, cert_der)
            .await
    }

    pub async fn get_certificates(&self, client_id: &ClientId) -> XRoadResult<Vec<Vec<u8>>> {
        self.certificates.get_certificates(client_id).await
    }

    pub async fn delete_client(&self, client_id: &ClientId) -> XRoadResult<()> {
        self.clients.delete_client(client_id).await
    }

    pub async fn delete_service_cascade(
        &self,
        client_id: &ClientId,
        service_code: &str,
    ) -> XRoadResult<()> {
        self.services
            .delete_service_cascade(client_id, service_code)
            .await
    }

    pub async fn set_client_status(&self, client_id: &ClientId, status: &str) -> XRoadResult<()> {
        self.clients.set_client_status(client_id, status).await
    }

    pub async fn make_owner(&self, client_id: &ClientId) -> XRoadResult<()> {
        self.clients.make_owner(client_id).await
    }

    pub async fn get_owner(&self) -> XRoadResult<Option<client_entity::Model>> {
        self.clients.get_owner().await
    }

    pub async fn list_service_descriptions(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service_description::Model>> {
        self.services.list_service_descriptions(client_id).await
    }

    pub async fn get_service_description(
        &self,
        id: i64,
    ) -> XRoadResult<Option<service_description::Model>> {
        self.services.get_service_description(id).await
    }

    pub async fn delete_service_description(&self, id: i64) -> XRoadResult<()> {
        self.services.delete_service_description(id).await
    }

    pub async fn set_service_description_disabled(
        &self,
        id: i64,
        disabled: bool,
        notice: Option<&str>,
    ) -> XRoadResult<()> {
        self.services
            .set_service_description_disabled(id, disabled, notice)
            .await
    }

    pub async fn list_services_for_client(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service_entity::Model>> {
        self.services.list_services_for_client(client_id).await
    }

    pub async fn get_service_by_id(&self, id: i64) -> XRoadResult<Option<service_entity::Model>> {
        self.services.get_service_by_id(id).await
    }

    pub async fn update_service(&self, id: i64, url: &str, timeout: i32) -> XRoadResult<()> {
        self.services.update_service(id, url, timeout).await
    }

    pub async fn list_service_access_rights(
        &self,
        service_id_str: &str,
    ) -> XRoadResult<Vec<String>> {
        self.access_rights
            .list_service_access_rights(service_id_str)
            .await
    }

    pub async fn add_endpoint(
        &self,
        client_id: &ClientId,
        service_code: &str,
        method: &str,
        path: &str,
    ) -> XRoadResult<ep_entity::Model> {
        self.endpoints
            .add_endpoint(client_id, service_code, method, path)
            .await
    }

    pub async fn list_endpoints(&self, client_id: &ClientId) -> XRoadResult<Vec<ep_entity::Model>> {
        self.endpoints.list_endpoints(client_id).await
    }

    pub async fn get_endpoint(&self, id: i64) -> XRoadResult<Option<ep_entity::Model>> {
        self.endpoints.get_endpoint(id).await
    }

    pub async fn delete_endpoint(&self, id: i64) -> XRoadResult<()> {
        self.endpoints.delete_endpoint(id).await
    }

    pub async fn grant_endpoint_access(
        &self,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        self.endpoints
            .grant_endpoint_access(endpoint_id, subject_id)
            .await
    }

    pub async fn revoke_endpoint_access(
        &self,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        self.endpoints
            .revoke_endpoint_access(endpoint_id, subject_id)
            .await
    }

    pub async fn list_endpoint_access_rights(&self, endpoint_id: i64) -> XRoadResult<Vec<String>> {
        self.endpoints
            .list_endpoint_access_rights(endpoint_id)
            .await
    }

    pub async fn create_local_group(
        &self,
        client_id: &ClientId,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<lg_entity::Model> {
        self.local_groups
            .create_local_group(client_id, group_code, description)
            .await
    }

    pub async fn list_local_groups(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<lg_entity::Model>> {
        self.local_groups.list_local_groups(client_id).await
    }

    pub async fn get_local_group(&self, id: i64) -> XRoadResult<Option<lg_entity::Model>> {
        self.local_groups.get_local_group(id).await
    }

    pub async fn delete_local_group(&self, id: i64) -> XRoadResult<()> {
        self.local_groups.delete_local_group(id).await
    }

    pub async fn add_group_member(&self, group_id: i64, member_id: &ClientId) -> XRoadResult<()> {
        self.local_groups
            .add_group_member(group_id, member_id)
            .await
    }

    pub async fn remove_group_member(
        &self,
        group_id: i64,
        member_id: &ClientId,
    ) -> XRoadResult<()> {
        self.local_groups
            .remove_group_member(group_id, member_id)
            .await
    }

    pub async fn list_group_members(&self, group_id: i64) -> XRoadResult<Vec<String>> {
        self.local_groups.list_group_members(group_id).await
    }
}

pub type DatabaseServerConf = ServerConf;

#[tonic::async_trait]
impl ServerConfProvider for ServerConf {
    async fn is_client_allowed(&self, client_id: &ClientId) -> XRoadResult<bool> {
        let client = self.clients.get_client(client_id).await?;
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
            .add(ar_entity::Column::ServiceIdStr.eq(&service_id_str));

        if service_id.service_version.is_none() {
            cond =
                cond.add(ar_entity::Column::ServiceIdStr.starts_with(format!("{service_id_str}/")));
        } else {
            let without_version = ServiceId::new_unchecked(
                service_id.client.clone(),
                &service_id.service_code,
                None::<&str>,
            )
            .to_string();
            cond = cond.add(ar_entity::Column::ServiceIdStr.eq(without_version));
        }

        let mut subject_cond = sea_orm::sea_query::Condition::any()
            .add(ar_entity::Column::SubjectClientIdStr.eq(&client_id_str));

        // Check local group memberships
        if let Ok(memberships) = group_member::Entity::find()
            .filter(group_member::Column::MemberClientIdStr.eq(&client_id_str))
            .all(self.raw_connection())
            .await
        {
            let group_ids: Vec<i64> = memberships.into_iter().map(|m| m.local_group_id).collect();
            if !group_ids.is_empty() {
                if let Ok(groups) = lg_entity::Entity::find()
                    .filter(lg_entity::Column::Id.is_in(group_ids))
                    .all(self.raw_connection())
                    .await
                {
                    for g in groups {
                        subject_cond = subject_cond.add(
                            ar_entity::Column::SubjectClientIdStr
                                .eq(format!("GROUP:{}", g.group_code)),
                        );
                    }
                }
            }
        }

        let count = ar_entity::Entity::find()
            .filter(cond)
            .filter(subject_cond)
            .count(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to check service access: {e}")))?;

        Ok(count > 0)
    }

    async fn get_service_url(&self, service_id: &ServiceId) -> XRoadResult<Option<String>> {
        let mut query = service_entity::Entity::find()
            .filter(service_entity::Column::ServiceCode.eq(&service_id.service_code));

        if let Some(ref version) = service_id.service_version {
            query = query.filter(service_entity::Column::ServiceVersion.eq(Some(version.clone())));
        }

        let svc = query
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get service URL: {e}")))?;

        Ok(svc.map(|s| s.url))
    }

    async fn list_services(&self, client_id: &ClientId) -> XRoadResult<Vec<ServiceId>> {
        let models = self.services.list_services_for_client(client_id).await?;
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
