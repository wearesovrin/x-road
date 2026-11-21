use crate::entities::{
    approved_ca, approved_tsa, global_group, global_group_member, member, security_server,
    security_server_client, subsystem,
};
use crate::migration::Migrator;
use common::{XRoadError, XRoadResult};
use globalconf::{
    SharedApprovedCa, SharedApprovedTsa, SharedGlobalGroup, SharedMember, SharedParams,
    SharedSecurityServer, SharedSubsystem,
};
use identifiers::ClientId;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;
use std::collections::HashMap;
use std::sync::Arc;

/// SeaORM-backed Central Server database repository.
#[derive(Clone)]
pub struct CentralServerDb {
    db: Arc<DatabaseConnection>,
}

impl CentralServerDb {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }

    /// Connects to database at `db_url` and runs migrations.
    pub async fn connect(db_url: &str) -> XRoadResult<Self> {
        let db = Database::connect(db_url)
            .await
            .map_err(|e| XRoadError::Database(format!("Database connection failed: {e}")))?;
        Migrator::up(&db, None)
            .await
            .map_err(|e| XRoadError::Database(format!("Database migration failed: {e}")))?;
        Ok(Self::new(db))
    }

    /// Connects to in-memory SQLite database and runs migrations.
    pub async fn connect_in_memory() -> XRoadResult<Self> {
        Self::connect("sqlite::memory:").await
    }

    pub fn raw_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    /// Registers a new X-Road member.
    pub async fn add_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
        name: &str,
        contact: Option<&str>,
    ) -> XRoadResult<member::Model> {
        let existing = self.get_member(instance, class, code).await?;
        if let Some(m) = existing {
            return Ok(m);
        }

        let now = chrono::Utc::now();
        member::ActiveModel {
            xroad_instance: Set(instance.to_string()),
            member_class: Set(class.to_string()),
            member_code: Set(code.to_string()),
            name: Set(name.to_string()),
            administrative_contact: Set(contact.map(ToString::to_string)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert member: {e}")))
    }

    /// Looks up a member by instance, class, and member code.
    pub async fn get_member(
        &self,
        instance: &str,
        class: &str,
        code: &str,
    ) -> XRoadResult<Option<member::Model>> {
        member::Entity::find()
            .filter(member::Column::XroadInstance.eq(instance))
            .filter(member::Column::MemberClass.eq(class))
            .filter(member::Column::MemberCode.eq(code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query member: {e}")))
    }

    /// Lists all members.
    pub async fn list_members(&self) -> XRoadResult<Vec<member::Model>> {
        member::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list members: {e}")))
    }

    /// Adds a subsystem to a member.
    pub async fn add_subsystem(
        &self,
        member_id: i64,
        subsystem_code: &str,
    ) -> XRoadResult<subsystem::Model> {
        subsystem::ActiveModel {
            member_id: Set(member_id),
            subsystem_code: Set(subsystem_code.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to insert subsystem: {e}")))
    }

    /// Lists all subsystems belonging to a member.
    pub async fn list_subsystems(&self, member_id: i64) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .filter(subsystem::Column::MemberId.eq(member_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list subsystems: {e}")))
    }

    /// Registers a security server under a member owner.
    pub async fn register_security_server(
        &self,
        owner_id: i64,
        server_code: &str,
        address: &str,
    ) -> XRoadResult<security_server::Model> {
        let now = chrono::Utc::now();
        security_server::ActiveModel {
            owner_id: Set(owner_id),
            server_code: Set(server_code.to_string()),
            address: Set(address.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to register security server: {e}")))
    }

    /// Lists all security servers.
    pub async fn list_security_servers(&self) -> XRoadResult<Vec<security_server::Model>> {
        security_server::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list security servers: {e}")))
    }

    /// Associates a client with a security server.
    pub async fn add_security_server_client(
        &self,
        security_server_id: i64,
        client_id: &ClientId,
    ) -> XRoadResult<()> {
        security_server_client::ActiveModel {
            security_server_id: Set(security_server_id),
            client_id_str: Set(client_id.to_string()),
            enabled: Set(true),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to associate client with security server: {e}")))?;

        Ok(())
    }

    /// Creates a global group.
    pub async fn create_global_group(
        &self,
        group_code: &str,
        description: &str,
    ) -> XRoadResult<global_group::Model> {
        let now = chrono::Utc::now();
        global_group::ActiveModel {
            group_code: Set(group_code.to_string()),
            description: Set(description.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create global group: {e}")))
    }

    /// Adds an identifier to a global group.
    pub async fn add_to_global_group(
        &self,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<()> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("Global group not found: {group_code}")))?;

        global_group_member::ActiveModel {
            global_group_id: Set(group.id),
            identifier: Set(identifier.to_string()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add member to global group: {e}")))?;

        Ok(())
    }

    /// Lists all member identifiers in a global group.
    pub async fn list_global_group_members(
        &self,
        group_code: &str,
    ) -> XRoadResult<Vec<String>> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("Global group not found: {group_code}")))?;

        let members = global_group_member::Entity::find()
            .filter(global_group_member::Column::GlobalGroupId.eq(group.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list group members: {e}")))?;

        Ok(members.into_iter().map(|m| m.identifier).collect())
    }

    /// Registers an Approved Certification Authority (CA).
    pub async fn add_approved_ca(
        &self,
        name: &str,
        top_ca_cert: &[u8],
        auth_only: bool,
        cert_profile: Option<&str>,
    ) -> XRoadResult<approved_ca::Model> {
        approved_ca::ActiveModel {
            name: Set(name.to_string()),
            top_ca_cert: Set(top_ca_cert.to_vec()),
            authentication_only: Set(auth_only),
            cert_profile_info: Set(cert_profile.map(ToString::to_string)),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add approved CA: {e}")))
    }

    /// Registers an Approved Time-Stamping Authority (TSA).
    pub async fn add_approved_tsa(
        &self,
        name: &str,
        url: &str,
        cert: &[u8],
    ) -> XRoadResult<approved_tsa::Model> {
        approved_tsa::ActiveModel {
            name: Set(name.to_string()),
            url: Set(url.to_string()),
            cert: Set(cert.to_vec()),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add approved TSA: {e}")))
    }

    /// Exports database entities into the standard `SharedParams` model.
    pub async fn export_shared_params(&self, instance_id: &str) -> XRoadResult<SharedParams> {
        let mut client_str_to_xml_id = HashMap::new();
        let mut shared_members = Vec::new();

        // 1. Members and subsystems
        let members = self.list_members().await?;
        for m in members {
            let member_client = ClientId::member_unchecked(instance_id, &m.member_class, &m.member_code);
            let member_xml_id = format!("id-m-{}", m.id);
            client_str_to_xml_id.insert(member_client.to_string(), member_xml_id.clone());

            let subsystems = self.list_subsystems(m.id).await?;
            let mut shared_subs = Vec::new();
            for sub in subsystems {
                let sub_client = ClientId::subsystem_unchecked(instance_id, &m.member_class, &m.member_code, &sub.subsystem_code);
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
        let servers = self.list_security_servers().await?;
        let mut shared_sec_servers = Vec::new();
        for s in servers {
            let client_links = security_server_client::Entity::find()
                .filter(security_server_client::Column::SecurityServerId.eq(s.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to list security server clients: {e}")))?;

            let mut client_xml_ids = Vec::new();
            for link in client_links {
                if let Some(xml_id) = client_str_to_xml_id.get(&link.client_id_str) {
                    client_xml_ids.push(xml_id.clone());
                }
            }

            shared_sec_servers.push(SharedSecurityServer {
                owner_id: format!("id-m-{}", s.owner_id),
                server_code: s.server_code,
                address: s.address,
                auth_cert_hashes: vec![],
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
        let groups = global_group::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global groups: {e}")))?;
        let mut shared_groups = Vec::new();
        for g in groups {
            let member_strs = self.list_global_group_members(&g.group_code).await?;
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
