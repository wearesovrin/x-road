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
}

impl ServerConf {
    /// Creates a new `ServerConf` from an existing SeaORM database connection.
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            raw_db: Arc::new(db),
        }
    }

    /// Connects to database at `db_url` and runs pending migrations.
    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url)
            .await
            .map_err(|e| XRoadError::Database(format!("Database connection failed: {e}")))?;
        crate::database::migrations::Migrator::up(&db, None)
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
        ClientService::add_client(&self.raw_db, client_id, is_auth).await
    }

    pub async fn get_client(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Option<client_entity::Model>> {
        ClientService::get_client(&self.raw_db, client_id).await
    }

    pub async fn list_clients(&self) -> XRoadResult<Vec<ClientId>> {
        ClientService::list_clients(&self.raw_db).await
    }

    pub async fn add_service(
        &self,
        client_id: &ClientId,
        service_code: &str,
        service_version: Option<&str>,
        url: &str,
        service_type: ServiceType,
    ) -> XRoadResult<crate::database::entities::service_description::Model> {
        ServiceService::add_service(
            &self.raw_db,
            client_id,
            service_code,
            service_version,
            url,
            service_type,
        )
        .await
    }

    pub async fn grant_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        AccessRightService::grant_access(&self.raw_db, service_id, client_id).await
    }

    pub async fn revoke_access(
        &self,
        service_id: &ServiceId,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        AccessRightService::revoke_access(&self.raw_db, service_id, client_id).await
    }

    pub async fn create_api_key(
        &self,
        encoded_key: &str,
        label: Option<&str>,
        roles: &[&str],
    ) -> XRoadResult<String> {
        ApiKeyService::create_api_key(&self.raw_db, encoded_key, label, roles).await
    }

    pub async fn validate_api_key(&self, key_or_token: &str) -> XRoadResult<Option<Vec<String>>> {
        ApiKeyService::validate_api_key(&self.raw_db, key_or_token).await
    }

    pub async fn list_api_keys(&self) -> XRoadResult<Vec<(api_key_entity::Model, Vec<String>)>> {
        ApiKeyService::list_api_keys(&self.raw_db).await
    }

    pub async fn get_api_key(
        &self,
        id: i64,
    ) -> XRoadResult<Option<(api_key_entity::Model, Vec<String>)>> {
        ApiKeyService::get_api_key(&self.raw_db, id).await
    }

    pub async fn update_api_key_roles(&self, id: i64, roles: &[&str]) -> XRoadResult<()> {
        ApiKeyService::update_api_key_roles(&self.raw_db, id, roles).await
    }

    pub async fn revoke_api_key(&self, id: i64) -> XRoadResult<()> {
        ApiKeyService::revoke_api_key(&self.raw_db, id).await
    }

    pub async fn store_certificate(
        &self,
        client_id: &ClientId,
        cert_der: &[u8],
    ) -> XRoadResult<()> {
        CertificateService::store_certificate(&self.raw_db, client_id, cert_der).await
    }

    pub async fn get_certificates(&self, client_id: &ClientId) -> XRoadResult<Vec<Vec<u8>>> {
        CertificateService::get_certificates(&self.raw_db, client_id).await
    }

    pub async fn delete_client(&self, client_id: &ClientId) -> XRoadResult<()> {
        ClientService::delete_client(&self.raw_db, client_id).await
    }

    pub async fn delete_service_cascade(
        &self,
        client_id: &ClientId,
        service_code: &str,
    ) -> XRoadResult<()> {
        ServiceService::delete_service_cascade(&self.raw_db, client_id, service_code).await
    }

    pub async fn set_client_status(&self, client_id: &ClientId, status: &str) -> XRoadResult<()> {
        ClientService::set_client_status(&self.raw_db, client_id, status).await
    }

    pub async fn make_owner(&self, client_id: &ClientId) -> XRoadResult<()> {
        ClientService::make_owner(&self.raw_db, client_id).await
    }

    pub async fn get_owner(&self) -> XRoadResult<Option<client_entity::Model>> {
        ClientService::get_owner(&self.raw_db).await
    }

    pub async fn list_service_descriptions(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service_description::Model>> {
        ServiceService::list_service_descriptions(&self.raw_db, client_id).await
    }

    pub async fn get_service_description(
        &self,
        id: i64,
    ) -> XRoadResult<Option<service_description::Model>> {
        ServiceService::get_service_description(&self.raw_db, id).await
    }

    pub async fn delete_service_description(&self, id: i64) -> XRoadResult<()> {
        ServiceService::delete_service_description(&self.raw_db, id).await
    }

    pub async fn set_service_description_disabled(
        &self,
        id: i64,
        disabled: bool,
        notice: Option<&str>,
    ) -> XRoadResult<()> {
        ServiceService::set_service_description_disabled(&self.raw_db, id, disabled, notice).await
    }

    pub async fn list_services_for_client(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<service_entity::Model>> {
        ServiceService::list_services_for_client(&self.raw_db, client_id).await
    }

    pub async fn get_service_by_id(&self, id: i64) -> XRoadResult<Option<service_entity::Model>> {
        ServiceService::get_service_by_id(&self.raw_db, id).await
    }

    pub async fn update_service(&self, id: i64, url: &str, timeout: i32) -> XRoadResult<()> {
        ServiceService::update_service(&self.raw_db, id, url, timeout).await
    }

    pub async fn list_service_access_rights(
        &self,
        service_id_str: &str,
    ) -> XRoadResult<Vec<String>> {
        AccessRightService::list_service_access_rights(&self.raw_db, service_id_str).await
    }

    pub async fn add_endpoint(
        &self,
        client_id: &ClientId,
        service_code: &str,
        method: &str,
        path: &str,
    ) -> XRoadResult<ep_entity::Model> {
        EndpointService::add_endpoint(&self.raw_db, client_id, service_code, method, path).await
    }

    pub async fn add_service_endpoint(
        &self,
        service_id: i64,
        method: &str,
        path: &str,
    ) -> XRoadResult<ep_entity::Model> {
        EndpointService::add_service_endpoint(&self.raw_db, service_id, method, path).await
    }

    pub async fn list_service_endpoints(
        &self,
        service_code: &str,
    ) -> XRoadResult<Vec<ep_entity::Model>> {
        EndpointService::list_endpoints_by_service_code(&self.raw_db, service_code).await
    }

    pub async fn list_endpoints(&self, client_id: &ClientId) -> XRoadResult<Vec<ep_entity::Model>> {
        EndpointService::list_endpoints(&self.raw_db, client_id).await
    }

    pub async fn get_endpoint(&self, id: i64) -> XRoadResult<Option<ep_entity::Model>> {
        EndpointService::get_endpoint(&self.raw_db, id).await
    }

    pub async fn delete_endpoint(&self, id: i64) -> XRoadResult<()> {
        EndpointService::delete_endpoint(&self.raw_db, id).await
    }

    pub async fn grant_endpoint_access(
        &self,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        EndpointService::grant_endpoint_access(&self.raw_db, endpoint_id, subject_id).await
    }

    pub async fn revoke_endpoint_access(
        &self,
        endpoint_id: i64,
        subject_id: &str,
    ) -> XRoadResult<()> {
        EndpointService::revoke_endpoint_access(&self.raw_db, endpoint_id, subject_id).await
    }

    pub async fn list_endpoint_access_rights(&self, endpoint_id: i64) -> XRoadResult<Vec<String>> {
        EndpointService::list_endpoint_access_rights(&self.raw_db, endpoint_id).await
    }

    pub async fn create_local_group(
        &self,
        client_id: &ClientId,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<lg_entity::Model> {
        LocalGroupService::create_local_group(&self.raw_db, client_id, group_code, description)
            .await
    }

    pub async fn list_local_groups(
        &self,
        client_id: &ClientId,
    ) -> XRoadResult<Vec<lg_entity::Model>> {
        LocalGroupService::list_local_groups(&self.raw_db, client_id).await
    }

    pub async fn get_local_group(&self, id: i64) -> XRoadResult<Option<lg_entity::Model>> {
        LocalGroupService::get_local_group(&self.raw_db, id).await
    }

    pub async fn delete_local_group(&self, id: i64) -> XRoadResult<()> {
        LocalGroupService::delete_local_group(&self.raw_db, id).await
    }

    pub async fn add_group_member(&self, group_id: i64, member_id: &ClientId) -> XRoadResult<()> {
        LocalGroupService::add_group_member(&self.raw_db, group_id, member_id).await
    }

    pub async fn remove_group_member(
        &self,
        group_id: i64,
        member_id: &ClientId,
    ) -> XRoadResult<()> {
        LocalGroupService::remove_group_member(&self.raw_db, group_id, member_id).await
    }

    pub async fn list_group_members(&self, group_id: i64) -> XRoadResult<Vec<String>> {
        LocalGroupService::list_group_members(&self.raw_db, group_id).await
    }
}

pub type DatabaseServerConf = ServerConf;

#[tonic::async_trait]
impl ServerConfProvider for ServerConf {
    async fn is_client_allowed(&self, client_id: &ClientId) -> XRoadResult<bool> {
        let client = ClientService::get_client(&self.raw_db, client_id).await?;
        Ok(client
            .map(|c| c.client_status == common::ClientStatus::Registered.as_ref())
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
            if !group_ids.is_empty()
                && let Ok(groups) = lg_entity::Entity::find()
                    .filter(lg_entity::Column::Id.is_in(group_ids))
                    .all(self.raw_connection())
                    .await
            {
                for g in groups {
                    subject_cond = subject_cond.add(
                        ar_entity::Column::SubjectClientIdStr.eq(format!("GROUP:{}", g.group_code)),
                    );
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
            .filter(service_entity::Column::ServiceCode.eq(service_id.service_code.as_str()));

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
        let models = ServiceService::list_services_for_client(&self.raw_db, client_id).await?;
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
