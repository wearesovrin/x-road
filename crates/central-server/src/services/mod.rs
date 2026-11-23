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
}

impl CentralServerServices {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            raw_db: Arc::new(db),
        }
    }

    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url).await.map_err(|e| {
            common::XRoadError::Database(format!("Database connection failed: {e}"))
        })?;
        crate::database::migrations::Migrator::up(&db, None)
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
        MemberClassService::add_member_class(&self.raw_db, code, description).await
    }

    pub async fn list_member_classes(&self) -> XRoadResult<Vec<mc_entity::Model>> {
        MemberClassService::list_member_classes(&self.raw_db).await
    }

    pub async fn get_member_class(&self, code: &str) -> XRoadResult<Option<mc_entity::Model>> {
        MemberClassService::get_member_class(&self.raw_db, code).await
    }

    pub async fn delete_member_class(&self, code: &str) -> XRoadResult<bool> {
        MemberClassService::delete_member_class(&self.raw_db, code).await
    }

    pub async fn add_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
        name: &str,
        contact: Option<&str>,
    ) -> XRoadResult<member_entity::Model> {
        MemberService::add_member(&self.raw_db, instance, class, code, name, contact).await
    }

    pub async fn get_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
    ) -> XRoadResult<Option<member_entity::Model>> {
        MemberService::get_member(&self.raw_db, instance, class, code).await
    }

    pub async fn get_member_by_id(&self, id: i64) -> XRoadResult<Option<member_entity::Model>> {
        MemberService::get_member_by_id(&self.raw_db, id).await
    }

    pub async fn update_member_name(
        &self,
        id: i64,
        name: &str,
    ) -> XRoadResult<member_entity::Model> {
        MemberService::update_member_name(&self.raw_db, id, name).await
    }

    pub async fn delete_member(&self, id: i64) -> XRoadResult<bool> {
        MemberService::delete_member(&self.raw_db, id).await
    }

    pub async fn list_members(&self) -> XRoadResult<Vec<member_entity::Model>> {
        MemberService::list_members(&self.raw_db).await
    }

    pub async fn list_member_global_groups(&self, member_id: i64) -> XRoadResult<Vec<String>> {
        MemberService::list_member_global_groups(&self.raw_db, member_id).await
    }

    pub async fn list_member_owned_servers(
        &self,
        member_id: i64,
    ) -> XRoadResult<Vec<ss_entity::Model>> {
        MemberService::list_member_owned_servers(&self.raw_db, member_id).await
    }

    pub async fn add_subsystem(
        &self,
        member_id: i64,
        subsystem_code: &str,
    ) -> XRoadResult<sub_entity::Model> {
        SubsystemService::add_subsystem(&self.raw_db, member_id, subsystem_code).await
    }

    pub async fn get_subsystem(&self, id: i64) -> XRoadResult<Option<sub_entity::Model>> {
        SubsystemService::get_subsystem(&self.raw_db, id).await
    }

    pub async fn list_subsystems(&self, member_id: i64) -> XRoadResult<Vec<sub_entity::Model>> {
        SubsystemService::list_subsystems(&self.raw_db, member_id).await
    }

    pub async fn list_all_subsystems(&self) -> XRoadResult<Vec<sub_entity::Model>> {
        SubsystemService::list_all_subsystems(&self.raw_db).await
    }

    pub async fn delete_subsystem(&self, id: i64) -> XRoadResult<bool> {
        SubsystemService::delete_subsystem(&self.raw_db, id).await
    }

    pub async fn list_all_clients(&self) -> XRoadResult<Vec<ClientItem>> {
        ClientService::list_all_clients(&self.raw_db).await
    }

    pub async fn register_security_server(
        &self,
        owner_id: i64,
        server_code: &str,
        address: &str,
    ) -> XRoadResult<ss_entity::Model> {
        SecurityServerService::register_security_server(
            &self.raw_db,
            owner_id,
            server_code,
            address,
        )
        .await
    }

    pub async fn get_security_server(&self, id: i64) -> XRoadResult<Option<ss_entity::Model>> {
        SecurityServerService::get_security_server(&self.raw_db, id).await
    }

    pub async fn delete_security_server(&self, id: i64) -> XRoadResult<bool> {
        SecurityServerService::delete_security_server(&self.raw_db, id).await
    }

    pub async fn list_security_servers(&self) -> XRoadResult<Vec<ss_entity::Model>> {
        SecurityServerService::list_security_servers(&self.raw_db).await
    }

    pub async fn add_security_server_client(
        &self,
        security_server_id: i64,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        SecurityServerService::add_security_server_client(
            &self.raw_db,
            security_server_id,
            client_id,
        )
        .await
    }

    pub async fn list_security_server_clients(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<String>> {
        SecurityServerService::list_security_server_clients(&self.raw_db, security_server_id).await
    }

    pub async fn remove_security_server_client(
        &self,
        security_server_id: i64,
        client_id_str: &str,
    ) -> XRoadResult<bool> {
        SecurityServerService::remove_security_server_client(
            &self.raw_db,
            security_server_id,
            client_id_str,
        )
        .await
    }

    pub async fn add_security_server_auth_cert(
        &self,
        security_server_id: i64,
        cert_pem: &str,
    ) -> XRoadResult<security_server_auth_cert::Model> {
        SecurityServerService::add_security_server_auth_cert(
            &self.raw_db,
            security_server_id,
            cert_pem,
        )
        .await
    }

    pub async fn list_security_server_auth_certs(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<security_server_auth_cert::Model>> {
        SecurityServerService::list_security_server_auth_certs(&self.raw_db, security_server_id)
            .await
    }

    pub async fn delete_security_server_auth_cert(&self, cert_id: i64) -> XRoadResult<bool> {
        SecurityServerService::delete_security_server_auth_cert(&self.raw_db, cert_id).await
    }

    pub async fn create_global_group(
        &self,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<global_group_entity::Model> {
        GlobalGroupService::create_global_group(&self.raw_db, group_code, description).await
    }

    pub async fn get_global_group(
        &self,
        group_code: &str,
    ) -> XRoadResult<Option<global_group_entity::Model>> {
        GlobalGroupService::get_global_group(&self.raw_db, group_code).await
    }

    pub async fn list_global_groups(&self) -> XRoadResult<Vec<global_group_entity::Model>> {
        GlobalGroupService::list_global_groups(&self.raw_db).await
    }

    pub async fn delete_global_group(&self, group_code: &str) -> XRoadResult<bool> {
        GlobalGroupService::delete_global_group(&self.raw_db, group_code).await
    }

    pub async fn add_to_global_group(&self, group_code: &str, identifier: &str) -> XRoadResult<()> {
        GlobalGroupService::add_to_global_group(&self.raw_db, group_code, identifier).await
    }

    pub async fn remove_from_global_group(
        &self,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<bool> {
        GlobalGroupService::remove_from_global_group(&self.raw_db, group_code, identifier).await
    }

    pub async fn list_global_group_members(&self, group_code: &str) -> XRoadResult<Vec<String>> {
        GlobalGroupService::list_global_group_members(&self.raw_db, group_code).await
    }

    pub async fn add_approved_ca(
        &self,
        name: &str,
        top_ca_cert: &[u8],
        auth_only: bool,
        cert_profile: Option<&str>,
    ) -> XRoadResult<approved_ca::Model> {
        CaService::add_approved_ca(&self.raw_db, name, top_ca_cert, auth_only, cert_profile).await
    }

    pub async fn get_approved_ca(&self, id: i64) -> XRoadResult<Option<approved_ca::Model>> {
        CaService::get_approved_ca(&self.raw_db, id).await
    }

    pub async fn list_approved_cas(&self) -> XRoadResult<Vec<approved_ca::Model>> {
        CaService::list_approved_cas(&self.raw_db).await
    }

    pub async fn delete_approved_ca(&self, id: i64) -> XRoadResult<bool> {
        CaService::delete_approved_ca(&self.raw_db, id).await
    }

    pub async fn add_approved_tsa(
        &self,
        name: &str,
        url: &str,
        cert: &[u8],
    ) -> XRoadResult<approved_tsa::Model> {
        TsaService::add_approved_tsa(&self.raw_db, name, url, cert).await
    }

    pub async fn get_approved_tsa(&self, id: i64) -> XRoadResult<Option<approved_tsa::Model>> {
        TsaService::get_approved_tsa(&self.raw_db, id).await
    }

    pub async fn list_approved_tsas(&self) -> XRoadResult<Vec<approved_tsa::Model>> {
        TsaService::list_approved_tsas(&self.raw_db).await
    }

    pub async fn delete_approved_tsa(&self, id: i64) -> XRoadResult<bool> {
        TsaService::delete_approved_tsa(&self.raw_db, id).await
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
        ManagementRequestService::create_management_request(
            &self.raw_db,
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
        ManagementRequestService::list_management_requests(&self.raw_db).await
    }

    pub async fn get_management_request(&self, id: i64) -> XRoadResult<Option<mr_entity::Model>> {
        ManagementRequestService::get_management_request(&self.raw_db, id).await
    }

    pub async fn approve_management_request(&self, id: i64) -> XRoadResult<mr_entity::Model> {
        ManagementRequestService::approve_management_request(&self.raw_db, id).await
    }

    pub async fn decline_management_request(&self, id: i64) -> XRoadResult<mr_entity::Model> {
        ManagementRequestService::decline_management_request(&self.raw_db, id).await
    }

    pub async fn get_system_parameter(&self, key: &str) -> XRoadResult<Option<String>> {
        SystemParameterService::get_system_parameter(&self.raw_db, key).await
    }

    pub async fn set_system_parameter(&self, key: &str, value: &str) -> XRoadResult<()> {
        SystemParameterService::set_system_parameter(&self.raw_db, key, value).await
    }

    pub async fn get_server_address(&self) -> XRoadResult<String> {
        SystemParameterService::get_server_address(&self.raw_db).await
    }

    pub async fn set_server_address(&self, address: &str) -> XRoadResult<()> {
        SystemParameterService::set_server_address(&self.raw_db, address).await
    }

    pub async fn generate_anchor_xml(&self, instance_id: &str) -> XRoadResult<String> {
        SystemParameterService::generate_anchor_xml(&self.raw_db, instance_id).await
    }

    pub async fn export_shared_params(&self, instance_id: &str) -> XRoadResult<SharedParams> {
        SystemParameterService::export_shared_params(&self.raw_db, instance_id).await
    }

    pub async fn generate_shared_params_xml(&self, instance_id: &str) -> XRoadResult<String> {
        SystemParameterService::generate_shared_params_xml(&self.raw_db, instance_id).await
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
}

pub type CentralServerDb = CentralServerServices;
