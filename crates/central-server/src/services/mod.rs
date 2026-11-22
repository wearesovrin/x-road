pub mod api_key;
pub mod ca;
pub mod client;
pub mod global_group;
pub mod management_request;
pub mod member;
pub mod member_class;
pub mod security_server;
pub mod subsystem;
pub mod system_parameter;
pub mod tsa;

pub use api_key::ApiKeyService;
pub use ca::CaService;
pub use client::{ClientItem, ClientService};
pub use global_group::GlobalGroupService;
pub use management_request::ManagementRequestService;
pub use member::MemberService;
pub use member_class::MemberClassService;
pub use security_server::SecurityServerService;
pub use subsystem::SubsystemService;
pub use system_parameter::SystemParameterService;
pub use tsa::TsaService;

use crate::entities::{
    api_key as api_key_entity, approved_ca, approved_tsa, global_group as global_group_entity,
    management_request as mr_entity, member as member_entity, member_class as mc_entity,
    security_server as ss_entity, security_server_auth_cert, subsystem as sub_entity,
};
use common::XRoadResult;
use globalconf::SharedParams;
use identifiers::ClientId;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

/// Central Server services container allowing direct access to domain service methods.
#[derive(Clone)]
pub struct CentralServerServices {
    pub raw_db: Arc<DatabaseConnection>,
    pub member_classes: MemberClassService,
    pub members: MemberService,
    pub subsystems: SubsystemService,
    pub clients: ClientService,
    pub security_servers: SecurityServerService,
    pub global_groups: GlobalGroupService,
    pub cas: CaService,
    pub tsas: TsaService,
    pub management_requests: ManagementRequestService,
    pub system_parameters: SystemParameterService,
    pub api_keys: ApiKeyService,
}

impl CentralServerServices {
    pub fn new(db: DatabaseConnection) -> Self {
        let db_arc = Arc::new(db);
        Self {
            member_classes: MemberClassService::new(db_arc.clone()),
            members: MemberService::new(db_arc.clone()),
            subsystems: SubsystemService::new(db_arc.clone()),
            clients: ClientService::new(db_arc.clone()),
            security_servers: SecurityServerService::new(db_arc.clone()),
            global_groups: GlobalGroupService::new(db_arc.clone()),
            cas: CaService::new(db_arc.clone()),
            tsas: TsaService::new(db_arc.clone()),
            management_requests: ManagementRequestService::new(db_arc.clone()),
            system_parameters: SystemParameterService::new(db_arc.clone()),
            api_keys: ApiKeyService::new(db_arc.clone()),
            raw_db: db_arc,
        }
    }

    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url).await.map_err(|e| {
            common::XRoadError::Database(format!("Database connection failed: {e}"))
        })?;
        crate::migration::Migrator::up(&db, None)
            .await
            .map_err(|e| common::XRoadError::Database(format!("Database migration failed: {e}")))?;
        Ok(Self::new(db))
    }

    pub async fn connect_in_memory() -> XRoadResult<Self> {
        Self::connect("sqlite::memory:").await
    }

    pub fn raw_connection(&self) -> &DatabaseConnection {
        &self.raw_db
    }

    // Direct service method delegations
    pub async fn add_member_class(
        &self,
        code: &str,
        description: &str,
    ) -> XRoadResult<mc_entity::Model> {
        self.member_classes
            .add_member_class(code, description)
            .await
    }

    pub async fn list_member_classes(&self) -> XRoadResult<Vec<mc_entity::Model>> {
        self.member_classes.list_member_classes().await
    }

    pub async fn get_member_class(&self, code: &str) -> XRoadResult<Option<mc_entity::Model>> {
        self.member_classes.get_member_class(code).await
    }

    pub async fn delete_member_class(&self, code: &str) -> XRoadResult<bool> {
        self.member_classes.delete_member_class(code).await
    }

    pub async fn add_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
        name: &str,
        contact: Option<&str>,
    ) -> XRoadResult<member_entity::Model> {
        self.members
            .add_member(instance, class, code, name, contact)
            .await
    }

    pub async fn get_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
    ) -> XRoadResult<Option<member_entity::Model>> {
        self.members.get_member(instance, class, code).await
    }

    pub async fn get_member_by_id(&self, id: i64) -> XRoadResult<Option<member_entity::Model>> {
        self.members.get_member_by_id(id).await
    }

    pub async fn update_member_name(
        &self,
        id: i64,
        name: &str,
    ) -> XRoadResult<member_entity::Model> {
        self.members.update_member_name(id, name).await
    }

    pub async fn delete_member(&self, id: i64) -> XRoadResult<bool> {
        self.members.delete_member(id).await
    }

    pub async fn list_members(&self) -> XRoadResult<Vec<member_entity::Model>> {
        self.members.list_members().await
    }

    pub async fn list_member_global_groups(&self, member_id: i64) -> XRoadResult<Vec<String>> {
        self.members.list_member_global_groups(member_id).await
    }

    pub async fn list_member_owned_servers(
        &self,
        member_id: i64,
    ) -> XRoadResult<Vec<ss_entity::Model>> {
        self.members.list_member_owned_servers(member_id).await
    }

    pub async fn add_subsystem(
        &self,
        member_id: i64,
        subsystem_code: &str,
    ) -> XRoadResult<sub_entity::Model> {
        self.subsystems
            .add_subsystem(member_id, subsystem_code)
            .await
    }

    pub async fn get_subsystem(&self, id: i64) -> XRoadResult<Option<sub_entity::Model>> {
        self.subsystems.get_subsystem(id).await
    }

    pub async fn list_subsystems(&self, member_id: i64) -> XRoadResult<Vec<sub_entity::Model>> {
        self.subsystems.list_subsystems(member_id).await
    }

    pub async fn list_all_subsystems(&self) -> XRoadResult<Vec<sub_entity::Model>> {
        self.subsystems.list_all_subsystems().await
    }

    pub async fn delete_subsystem(&self, id: i64) -> XRoadResult<bool> {
        self.subsystems.delete_subsystem(id).await
    }

    pub async fn list_all_clients(&self) -> XRoadResult<Vec<ClientItem>> {
        self.clients.list_all_clients().await
    }

    pub async fn register_security_server(
        &self,
        owner_id: i64,
        server_code: &str,
        address: &str,
    ) -> XRoadResult<ss_entity::Model> {
        self.security_servers
            .register_security_server(owner_id, server_code, address)
            .await
    }

    pub async fn get_security_server(&self, id: i64) -> XRoadResult<Option<ss_entity::Model>> {
        self.security_servers.get_security_server(id).await
    }

    pub async fn delete_security_server(&self, id: i64) -> XRoadResult<bool> {
        self.security_servers.delete_security_server(id).await
    }

    pub async fn list_security_servers(&self) -> XRoadResult<Vec<ss_entity::Model>> {
        self.security_servers.list_security_servers().await
    }

    pub async fn add_security_server_client(
        &self,
        security_server_id: i64,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        self.security_servers
            .add_security_server_client(security_server_id, client_id)
            .await
    }

    pub async fn list_security_server_clients(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<String>> {
        self.security_servers
            .list_security_server_clients(security_server_id)
            .await
    }

    pub async fn remove_security_server_client(
        &self,
        security_server_id: i64,
        client_id_str: &str,
    ) -> XRoadResult<bool> {
        self.security_servers
            .remove_security_server_client(security_server_id, client_id_str)
            .await
    }

    pub async fn add_security_server_auth_cert(
        &self,
        security_server_id: i64,
        cert_pem: &str,
    ) -> XRoadResult<security_server_auth_cert::Model> {
        self.security_servers
            .add_security_server_auth_cert(security_server_id, cert_pem)
            .await
    }

    pub async fn list_security_server_auth_certs(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<security_server_auth_cert::Model>> {
        self.security_servers
            .list_security_server_auth_certs(security_server_id)
            .await
    }

    pub async fn delete_security_server_auth_cert(&self, cert_id: i64) -> XRoadResult<bool> {
        self.security_servers
            .delete_security_server_auth_cert(cert_id)
            .await
    }

    pub async fn create_global_group(
        &self,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<global_group_entity::Model> {
        self.global_groups
            .create_global_group(group_code, description)
            .await
    }

    pub async fn get_global_group(
        &self,
        group_code: &str,
    ) -> XRoadResult<Option<global_group_entity::Model>> {
        self.global_groups.get_global_group(group_code).await
    }

    pub async fn list_global_groups(&self) -> XRoadResult<Vec<global_group_entity::Model>> {
        self.global_groups.list_global_groups().await
    }

    pub async fn delete_global_group(&self, group_code: &str) -> XRoadResult<bool> {
        self.global_groups.delete_global_group(group_code).await
    }

    pub async fn add_to_global_group(&self, group_code: &str, identifier: &str) -> XRoadResult<()> {
        self.global_groups
            .add_to_global_group(group_code, identifier)
            .await
    }

    pub async fn remove_from_global_group(
        &self,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<bool> {
        self.global_groups
            .remove_from_global_group(group_code, identifier)
            .await
    }

    pub async fn list_global_group_members(&self, group_code: &str) -> XRoadResult<Vec<String>> {
        self.global_groups
            .list_global_group_members(group_code)
            .await
    }

    pub async fn add_approved_ca(
        &self,
        name: &str,
        top_ca_cert: &[u8],
        auth_only: bool,
        cert_profile: Option<&str>,
    ) -> XRoadResult<approved_ca::Model> {
        self.cas
            .add_approved_ca(name, top_ca_cert, auth_only, cert_profile)
            .await
    }

    pub async fn get_approved_ca(&self, id: i64) -> XRoadResult<Option<approved_ca::Model>> {
        self.cas.get_approved_ca(id).await
    }

    pub async fn list_approved_cas(&self) -> XRoadResult<Vec<approved_ca::Model>> {
        self.cas.list_approved_cas().await
    }

    pub async fn delete_approved_ca(&self, id: i64) -> XRoadResult<bool> {
        self.cas.delete_approved_ca(id).await
    }

    pub async fn add_approved_tsa(
        &self,
        name: &str,
        url: &str,
        cert: &[u8],
    ) -> XRoadResult<approved_tsa::Model> {
        self.tsas.add_approved_tsa(name, url, cert).await
    }

    pub async fn get_approved_tsa(&self, id: i64) -> XRoadResult<Option<approved_tsa::Model>> {
        self.tsas.get_approved_tsa(id).await
    }

    pub async fn list_approved_tsas(&self) -> XRoadResult<Vec<approved_tsa::Model>> {
        self.tsas.list_approved_tsas().await
    }

    pub async fn delete_approved_tsa(&self, id: i64) -> XRoadResult<bool> {
        self.tsas.delete_approved_tsa(id).await
    }

    pub async fn create_management_request(
        &self,
        request_type: &str,
        origin: &str,
        security_server_id: Option<i64>,
        client_id_str: Option<&str>,
        auth_cert_pem: Option<&str>,
        comments: Option<&str>,
    ) -> XRoadResult<mr_entity::Model> {
        self.management_requests
            .create_management_request(
                request_type,
                origin,
                security_server_id,
                client_id_str,
                auth_cert_pem,
                comments,
            )
            .await
    }

    pub async fn list_management_requests(&self) -> XRoadResult<Vec<mr_entity::Model>> {
        self.management_requests.list_management_requests().await
    }

    pub async fn get_management_request(&self, id: i64) -> XRoadResult<Option<mr_entity::Model>> {
        self.management_requests.get_management_request(id).await
    }

    pub async fn approve_management_request(&self, id: i64) -> XRoadResult<mr_entity::Model> {
        self.management_requests
            .approve_management_request(id)
            .await
    }

    pub async fn decline_management_request(&self, id: i64) -> XRoadResult<mr_entity::Model> {
        self.management_requests
            .decline_management_request(id)
            .await
    }

    pub async fn get_system_parameter(&self, key: &str) -> XRoadResult<Option<String>> {
        self.system_parameters.get_system_parameter(key).await
    }

    pub async fn set_system_parameter(&self, key: &str, value: &str) -> XRoadResult<()> {
        self.system_parameters
            .set_system_parameter(key, value)
            .await
    }

    pub async fn get_server_address(&self) -> XRoadResult<String> {
        self.system_parameters.get_server_address().await
    }

    pub async fn set_server_address(&self, address: &str) -> XRoadResult<()> {
        self.system_parameters.set_server_address(address).await
    }

    pub async fn generate_anchor_xml(&self, instance_id: &str) -> XRoadResult<String> {
        self.system_parameters
            .generate_anchor_xml(instance_id)
            .await
    }

    pub async fn export_shared_params(&self, instance_id: &str) -> XRoadResult<SharedParams> {
        self.system_parameters
            .export_shared_params(instance_id)
            .await
    }

    pub async fn generate_shared_params_xml(&self, instance_id: &str) -> XRoadResult<String> {
        self.system_parameters
            .generate_shared_params_xml(instance_id)
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
}

pub type CentralServerDb = CentralServerServices;
