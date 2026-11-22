use crate::entities::{approved_ca, approved_tsa, security_server_client, system_parameter};
use crate::services::global_group::GlobalGroupService;
use crate::services::member::MemberService;
use crate::services::security_server::SecurityServerService;
use crate::services::subsystem::SubsystemService;
use common::{XRoadError, XRoadResult};
use globalconf::{
    SharedApprovedCa, SharedApprovedTsa, SharedGlobalGroup, SharedMember, SharedParams,
    SharedSecurityServer, SharedSubsystem,
};
use identifiers::ClientId;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;
use std::sync::Arc;

/// Service for managing System Parameters, configuration anchor, and shared-params export.
#[derive(Clone)]
pub struct SystemParameterService {
    db: Arc<DatabaseConnection>,
    members: MemberService,
    subsystems: SubsystemService,
    security_servers: SecurityServerService,
    global_groups: GlobalGroupService,
}

impl SystemParameterService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            db: db.clone(),
            members: MemberService::new(db.clone()),
            subsystems: SubsystemService::new(db.clone()),
            security_servers: SecurityServerService::new(db.clone()),
            global_groups: GlobalGroupService::new(db),
        }
    }

    fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn get_system_parameter(&self, key: &str) -> XRoadResult<Option<String>> {
        system_parameter::Entity::find_by_id(key.to_string())
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get system parameter: {e}")))
            .map(|opt| opt.map(|m| m.value))
    }

    pub async fn set_system_parameter(&self, key: &str, value: &str) -> XRoadResult<()> {
        let existing = system_parameter::Entity::find_by_id(key.to_string())
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to check system parameter: {e}")))?;

        if let Some(m) = existing {
            let mut active: system_parameter::ActiveModel = m.into();
            active.value = Set(value.to_string());
            active.update(self.raw_connection()).await.map_err(|e| {
                XRoadError::Database(format!("Failed to update system parameter: {e}"))
            })?;
        } else {
            system_parameter::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to insert system parameter: {e}")))?;
        }
        Ok(())
    }

    pub async fn get_server_address(&self) -> XRoadResult<String> {
        Ok(self
            .get_system_parameter("server_address")
            .await?
            .unwrap_or_else(|| "127.0.0.1".to_string()))
    }

    pub async fn set_server_address(&self, address: &str) -> XRoadResult<()> {
        self.set_system_parameter("server_address", address).await
    }

    pub async fn generate_anchor_xml(&self, instance_id: &str) -> XRoadResult<String> {
        let addr = self.get_server_address().await?;
        let anchor = globalconf::ConfigurationAnchor {
            instance_identifier: instance_id.to_string(),
            generated_at: chrono::Utc::now(),
            download_urls: vec![format!("http://{addr}:4000/internalconf")],
            verification_cert_hashes: vec!["MIIB0DCCAXqgAwIBAgIUQz3y".to_string()],
        };
        anchor.to_xml()
    }

    /// Exports database entities into the standard `SharedParams` model.
    pub async fn export_shared_params(&self, instance_id: &str) -> XRoadResult<SharedParams> {
        let mut client_str_to_xml_id = HashMap::new();
        let mut shared_members = Vec::new();

        // 1. Members and subsystems
        let members = self.members.list_members().await?;
        for m in members {
            let member_client =
                ClientId::member_unchecked(instance_id, &m.member_class, &m.member_code);
            let member_xml_id = format!("id-m-{}", m.id);
            client_str_to_xml_id.insert(member_client.to_string(), member_xml_id.clone());

            let subsystems = self.subsystems.list_subsystems(m.id).await?;
            let mut shared_subs = Vec::new();
            for sub in subsystems {
                let sub_client = ClientId::subsystem_unchecked(
                    instance_id,
                    &m.member_class,
                    &m.member_code,
                    &sub.subsystem_code,
                );
                let sub_xml_id = format!("id-s-{}", sub.id);
                client_str_to_xml_id.insert(sub_client.to_string(), sub_xml_id.clone());

                shared_subs.push(SharedSubsystem {
                    id: sub_xml_id,
                    subsystem_code: sub.subsystem_code,
                    subsystem_name: None,
                });
            }

            shared_members.push(SharedMember {
                id: member_xml_id,
                member_class: m.member_class,
                member_code: m.member_code,
                name: m.name,
                subsystems: shared_subs,
            });
        }

        // 2. Security Servers
        let servers = self.security_servers.list_security_servers().await?;
        let mut shared_sec_servers = Vec::new();
        for s in servers {
            let client_links = security_server_client::Entity::find()
                .filter(security_server_client::Column::SecurityServerId.eq(s.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| {
                    XRoadError::Database(format!("Failed to list security server clients: {e}"))
                })?;

            let mut client_xml_ids = Vec::new();
            for link in client_links {
                if let Some(xml_id) = client_str_to_xml_id.get(&link.client_id_str) {
                    client_xml_ids.push(xml_id.clone());
                }
            }

            let auth_certs = self
                .security_servers
                .list_security_server_auth_certs(s.id)
                .await
                .unwrap_or_default();
            let auth_cert_hashes: Vec<String> =
                auth_certs.into_iter().map(|c| c.cert_hash).collect();

            shared_sec_servers.push(SharedSecurityServer {
                owner_id: format!("id-m-{}", s.owner_id),
                server_code: s.server_code,
                address: s.address,
                auth_cert_hashes,
                client_ids: client_xml_ids,
            });
        }

        // 3. Approved CAs
        let cas = approved_ca::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query approved CAs: {e}")))?;
        let shared_cas: Vec<SharedApprovedCa> = cas
            .into_iter()
            .map(|ca| SharedApprovedCa {
                name: ca.name,
                authentication_only: ca.authentication_only,
                top_ca_cert: ca.top_ca_cert,
                ocsp_url: None,
                ocsp_cert: None,
                certificate_profile_info: ca.cert_profile_info,
                acme_server_url: None,
            })
            .collect();

        // 4. Approved TSAs
        let tsas = approved_tsa::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query approved TSAs: {e}")))?;
        let shared_tsas: Vec<SharedApprovedTsa> = tsas
            .into_iter()
            .map(|tsa| SharedApprovedTsa {
                name: tsa.name,
                url: tsa.url,
                cert: tsa.cert,
            })
            .collect();

        // 5. Global Groups
        let groups = self.global_groups.list_global_groups().await?;
        let mut shared_groups = Vec::new();
        for g in groups {
            let member_strs = self
                .global_groups
                .list_global_group_members(&g.group_code)
                .await?;
            let members: Vec<ClientId> = member_strs
                .into_iter()
                .filter_map(|s| s.parse::<ClientId>().ok())
                .collect();

            shared_groups.push(SharedGlobalGroup {
                group_code: g.group_code,
                description: g.description,
                members,
            });
        }

        Ok(SharedParams {
            instance_identifier: instance_id.to_string(),
            approved_cas: shared_cas,
            approved_tsas: shared_tsas,
            members: shared_members,
            security_servers: shared_sec_servers,
            global_groups: shared_groups,
            ocsp_freshness_seconds: Some(3600),
        })
    }

    /// Generates standard `shared-params.xml` for this instance.
    pub async fn generate_shared_params_xml(&self, instance_id: &str) -> XRoadResult<String> {
        let params = self.export_shared_params(instance_id).await?;
        Ok(params.to_xml())
    }
}
