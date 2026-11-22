use crate::entities::{
    api_key, api_key_role, approved_ca, approved_tsa, global_group, global_group_member,
    management_request, member, member_class, security_server, security_server_auth_cert,
    security_server_client, subsystem, system_parameter,
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
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientItem {
    pub client_id: String,
    pub member_name: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub status: String,
}

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

    // ==========================================
    // Member Classes
    // ==========================================

    pub async fn add_member_class(
        &self,
        code: &str,
        description: &str,
    ) -> XRoadResult<member_class::Model> {
        member_class::ActiveModel {
            code: Set(code.to_string()),
            description: Set(description.to_string()),
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add member class: {e}")))
    }

    pub async fn list_member_classes(&self) -> XRoadResult<Vec<member_class::Model>> {
        member_class::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list member classes: {e}")))
    }

    pub async fn get_member_class(&self, code: &str) -> XRoadResult<Option<member_class::Model>> {
        member_class::Entity::find_by_id(code.to_string())
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get member class: {e}")))
    }

    pub async fn delete_member_class(&self, code: &str) -> XRoadResult<bool> {
        let res = member_class::Entity::delete_by_id(code.to_string())
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete member class: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    // ==========================================
    // Members
    // ==========================================

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

    pub async fn get_member_by_id(&self, id: i64) -> XRoadResult<Option<member::Model>> {
        member::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query member by ID: {e}")))
    }

    pub async fn update_member_name(&self, id: i64, name: &str) -> XRoadResult<member::Model> {
        let m = self
            .get_member_by_id(id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Member not found: {id}")))?;
        let mut active: member::ActiveModel = m.into();
        active.name = Set(name.to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to update member: {e}")))
    }

    pub async fn delete_member(&self, id: i64) -> XRoadResult<bool> {
        subsystem::Entity::delete_many()
            .filter(subsystem::Column::MemberId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete member subsystems: {e}"))
            })?;

        let res = member::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete member: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    /// Lists all members.
    pub async fn list_members(&self) -> XRoadResult<Vec<member::Model>> {
        member::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list members: {e}")))
    }

    pub async fn list_member_global_groups(&self, member_id: i64) -> XRoadResult<Vec<String>> {
        let member = self
            .get_member_by_id(member_id)
            .await?
            .ok_or_else(|| XRoadError::Configuration(format!("Member not found: {member_id}")))?;
        let member_prefix = format!(
            "{}/{}/{}",
            member.xroad_instance, member.member_class, member.member_code
        );

        let group_members = global_group_member::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query group members: {e}")))?;

        let mut matched_group_ids = std::collections::HashSet::new();
        for gm in group_members {
            if gm.identifier.starts_with(&member_prefix) {
                matched_group_ids.insert(gm.global_group_id);
            }
        }

        let mut groups = Vec::new();
        for gid in matched_group_ids {
            if let Ok(Some(g)) = global_group::Entity::find_by_id(gid)
                .one(self.raw_connection())
                .await
            {
                groups.push(g.group_code);
            }
        }
        Ok(groups)
    }

    pub async fn list_member_owned_servers(
        &self,
        member_id: i64,
    ) -> XRoadResult<Vec<security_server::Model>> {
        security_server::Entity::find()
            .filter(security_server::Column::OwnerId.eq(member_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to query member security servers: {e}"))
            })
    }

    // ==========================================
    // Subsystems
    // ==========================================

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

    pub async fn get_subsystem(&self, id: i64) -> XRoadResult<Option<subsystem::Model>> {
        subsystem::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get subsystem: {e}")))
    }

    /// Lists all subsystems belonging to a member.
    pub async fn list_subsystems(&self, member_id: i64) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .filter(subsystem::Column::MemberId.eq(member_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list subsystems: {e}")))
    }

    pub async fn list_all_subsystems(&self) -> XRoadResult<Vec<subsystem::Model>> {
        subsystem::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list all subsystems: {e}")))
    }

    pub async fn delete_subsystem(&self, id: i64) -> XRoadResult<bool> {
        let res = subsystem::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete subsystem: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    // ==========================================
    // Clients Aggregation
    // ==========================================

    pub async fn list_all_clients(&self) -> XRoadResult<Vec<ClientItem>> {
        let members = self.list_members().await?;
        let mut clients = Vec::new();

        for m in members {
            let member_id_str =
                format!("{}/{}/{}", m.xroad_instance, m.member_class, m.member_code);
            clients.push(ClientItem {
                client_id: member_id_str,
                member_name: m.name.clone(),
                member_class: m.member_class.clone(),
                member_code: m.member_code.clone(),
                subsystem_code: None,
                status: "REGISTERED".to_string(),
            });

            let subs = self.list_subsystems(m.id).await?;
            for s in subs {
                let sub_id_str = format!(
                    "{}/{}/{}/{}",
                    m.xroad_instance, m.member_class, m.member_code, s.subsystem_code
                );
                clients.push(ClientItem {
                    client_id: sub_id_str,
                    member_name: m.name.clone(),
                    member_class: m.member_class.clone(),
                    member_code: m.member_code.clone(),
                    subsystem_code: Some(s.subsystem_code),
                    status: "REGISTERED".to_string(),
                });
            }
        }

        Ok(clients)
    }

    // ==========================================
    // Security Servers
    // ==========================================

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

    pub async fn get_security_server(
        &self,
        id: i64,
    ) -> XRoadResult<Option<security_server::Model>> {
        security_server::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get security server: {e}")))
    }

    pub async fn delete_security_server(&self, id: i64) -> XRoadResult<bool> {
        security_server_client::Entity::delete_many()
            .filter(security_server_client::Column::SecurityServerId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete security server clients: {e}"))
            })?;

        security_server_auth_cert::Entity::delete_many()
            .filter(security_server_auth_cert::Column::SecurityServerId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete security server auth certs: {e}"))
            })?;

        let res = security_server::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete security server: {e}")))?;
        Ok(res.rows_affected > 0)
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
        .map_err(|e| {
            XRoadError::Database(format!(
                "Failed to associate client with security server: {e}"
            ))
        })?;

        Ok(())
    }

    pub async fn list_security_server_clients(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<String>> {
        let clients = security_server_client::Entity::find()
            .filter(security_server_client::Column::SecurityServerId.eq(security_server_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list server clients: {e}")))?;
        Ok(clients.into_iter().map(|c| c.client_id_str).collect())
    }

    pub async fn remove_security_server_client(
        &self,
        security_server_id: i64,
        client_id_str: &str,
    ) -> XRoadResult<bool> {
        let res = security_server_client::Entity::delete_many()
            .filter(security_server_client::Column::SecurityServerId.eq(security_server_id))
            .filter(security_server_client::Column::ClientIdStr.eq(client_id_str))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete server client: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    pub async fn add_security_server_auth_cert(
        &self,
        security_server_id: i64,
        cert_pem: &str,
    ) -> XRoadResult<security_server_auth_cert::Model> {
        let mut hasher = Sha256::new();
        hasher.update(cert_pem.as_bytes());
        let cert_hash = format!("{:x}", hasher.finalize());

        security_server_auth_cert::ActiveModel {
            security_server_id: Set(security_server_id),
            cert_pem: Set(cert_pem.to_string()),
            cert_hash: Set(cert_hash),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to add auth cert: {e}")))
    }

    pub async fn list_security_server_auth_certs(
        &self,
        security_server_id: i64,
    ) -> XRoadResult<Vec<security_server_auth_cert::Model>> {
        security_server_auth_cert::Entity::find()
            .filter(security_server_auth_cert::Column::SecurityServerId.eq(security_server_id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list auth certs: {e}")))
    }

    pub async fn delete_security_server_auth_cert(&self, cert_id: i64) -> XRoadResult<bool> {
        let res = security_server_auth_cert::Entity::delete_by_id(cert_id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete auth cert: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    // ==========================================
    // Global Groups
    // ==========================================

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

    pub async fn get_global_group(
        &self,
        group_code: &str,
    ) -> XRoadResult<Option<global_group::Model>> {
        global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get global group: {e}")))
    }

    pub async fn list_global_groups(&self) -> XRoadResult<Vec<global_group::Model>> {
        global_group::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list global groups: {e}")))
    }

    pub async fn delete_global_group(&self, group_code: &str) -> XRoadResult<bool> {
        if let Some(g) = self.get_global_group(group_code).await? {
            global_group_member::Entity::delete_many()
                .filter(global_group_member::Column::GlobalGroupId.eq(g.id))
                .exec(self.raw_connection())
                .await
                .map_err(|e| {
                    XRoadError::Database(format!("Failed to delete group members: {e}"))
                })?;

            let res = global_group::Entity::delete_by_id(g.id)
                .exec(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to delete global group: {e}")))?;
            Ok(res.rows_affected > 0)
        } else {
            Ok(false)
        }
    }

    /// Adds an identifier to a global group.
    pub async fn add_to_global_group(&self, group_code: &str, identifier: &str) -> XRoadResult<()> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Global group not found: {group_code}"))
            })?;

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

    pub async fn remove_from_global_group(
        &self,
        group_code: &str,
        identifier: &str,
    ) -> XRoadResult<bool> {
        if let Some(g) = self.get_global_group(group_code).await? {
            let res = global_group_member::Entity::delete_many()
                .filter(global_group_member::Column::GlobalGroupId.eq(g.id))
                .filter(global_group_member::Column::Identifier.eq(identifier))
                .exec(self.raw_connection())
                .await
                .map_err(|e| {
                    XRoadError::Database(format!("Failed to remove member from global group: {e}"))
                })?;
            Ok(res.rows_affected > 0)
        } else {
            Ok(false)
        }
    }

    /// Lists all member identifiers in a global group.
    pub async fn list_global_group_members(&self, group_code: &str) -> XRoadResult<Vec<String>> {
        let group = global_group::Entity::find()
            .filter(global_group::Column::GroupCode.eq(group_code))
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to query global group: {e}")))?
            .ok_or_else(|| {
                XRoadError::Configuration(format!("Global group not found: {group_code}"))
            })?;

        let members = global_group_member::Entity::find()
            .filter(global_group_member::Column::GlobalGroupId.eq(group.id))
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list group members: {e}")))?;

        Ok(members.into_iter().map(|m| m.identifier).collect())
    }

    // ==========================================
    // Approved Certification Authorities (CAs)
    // ==========================================

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

    pub async fn get_approved_ca(&self, id: i64) -> XRoadResult<Option<approved_ca::Model>> {
        approved_ca::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get CA: {e}")))
    }

    /// Lists all Approved Certification Authorities.
    pub async fn list_approved_cas(&self) -> XRoadResult<Vec<approved_ca::Model>> {
        approved_ca::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list approved CAs: {e}")))
    }

    pub async fn delete_approved_ca(&self, id: i64) -> XRoadResult<bool> {
        let res = approved_ca::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete CA: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    // ==========================================
    // Approved Time-Stamping Authorities (TSAs)
    // ==========================================

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

    pub async fn get_approved_tsa(&self, id: i64) -> XRoadResult<Option<approved_tsa::Model>> {
        approved_tsa::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get TSA: {e}")))
    }

    /// Lists all Approved Time-Stamping Authorities.
    pub async fn list_approved_tsas(&self) -> XRoadResult<Vec<approved_tsa::Model>> {
        approved_tsa::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list approved TSAs: {e}")))
    }

    pub async fn delete_approved_tsa(&self, id: i64) -> XRoadResult<bool> {
        let res = approved_tsa::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete TSA: {e}")))?;
        Ok(res.rows_affected > 0)
    }

    // ==========================================
    // Management Requests
    // ==========================================

    pub async fn create_management_request(
        &self,
        request_type: &str,
        origin: &str,
        security_server_id: Option<i64>,
        client_id_str: Option<&str>,
        auth_cert_pem: Option<&str>,
        comments: Option<&str>,
    ) -> XRoadResult<management_request::Model> {
        let now = chrono::Utc::now();
        management_request::ActiveModel {
            request_type: Set(request_type.to_string()),
            origin: Set(origin.to_string()),
            security_server_id: Set(security_server_id),
            client_id_str: Set(client_id_str.map(ToString::to_string)),
            auth_cert_pem: Set(auth_cert_pem.map(ToString::to_string)),
            status: Set("WAITING_FOR_APPROVAL".to_string()),
            comments: Set(comments.map(ToString::to_string)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(self.raw_connection())
        .await
        .map_err(|e| XRoadError::Database(format!("Failed to create management request: {e}")))
    }

    pub async fn list_management_requests(&self) -> XRoadResult<Vec<management_request::Model>> {
        management_request::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list management requests: {e}")))
    }

    pub async fn get_management_request(
        &self,
        id: i64,
    ) -> XRoadResult<Option<management_request::Model>> {
        management_request::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get management request: {e}")))
    }

    pub async fn approve_management_request(
        &self,
        id: i64,
    ) -> XRoadResult<management_request::Model> {
        let req = self.get_management_request(id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Management request not found: {id}"))
        })?;

        match req.request_type.as_str() {
            "CLIENT_REGISTRATION" | "AUTH_CERT_REGISTRATION" | "OWNER_CHANGE" => {
                if let (Some(server_id), Some(client_id_str)) =
                    (req.security_server_id, req.client_id_str.as_ref())
                {
                    if let Ok(client_id) = client_id_str.parse::<ClientId>() {
                        let _ = self.add_security_server_client(server_id, &client_id).await;
                    }
                }
                if let (Some(server_id), Some(cert_pem)) =
                    (req.security_server_id, req.auth_cert_pem.as_ref())
                {
                    let _ = self
                        .add_security_server_auth_cert(server_id, cert_pem)
                        .await;
                }
            }
            "CLIENT_DELETION" => {
                if let (Some(server_id), Some(client_id_str)) =
                    (req.security_server_id, req.client_id_str.as_ref())
                {
                    let _ = self
                        .remove_security_server_client(server_id, client_id_str)
                        .await;
                }
            }
            _ => {}
        }

        let mut active: management_request::ActiveModel = req.into();
        active.status = Set("APPROVED".to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to approve management request: {e}")))
    }

    pub async fn decline_management_request(
        &self,
        id: i64,
    ) -> XRoadResult<management_request::Model> {
        let req = self.get_management_request(id).await?.ok_or_else(|| {
            XRoadError::Configuration(format!("Management request not found: {id}"))
        })?;

        let mut active: management_request::ActiveModel = req.into();
        active.status = Set("DECLINED".to_string());
        active.updated_at = Set(chrono::Utc::now());
        active
            .update(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to decline management request: {e}")))
    }

    // ==========================================
    // System Parameters & Configuration Anchor
    // ==========================================

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
        let generated_at = chrono::Utc::now().to_rfc3339();
        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<configurationAnchor>
    <instanceIdentifier>{instance_id}</instanceIdentifier>
    <generatedAt>{generated_at}</generatedAt>
    <source>
        <downloadURL>http://{addr}:4000/internalconf</downloadURL>
        <verificationCert>MIIB0DCCAXqgAwIBAgIUQz3y</verificationCert>
    </source>
</configurationAnchor>"#
        ))
    }

    // ==========================================
    // Export Shared Parameters
    // ==========================================

    /// Exports database entities into the standard `SharedParams` model.
    pub async fn export_shared_params(&self, instance_id: &str) -> XRoadResult<SharedParams> {
        let mut client_str_to_xml_id = HashMap::new();
        let mut shared_members = Vec::new();

        // 1. Members and subsystems
        let members = self.list_members().await?;
        for m in members {
            let member_client =
                ClientId::member_unchecked(instance_id, &m.member_class, &m.member_code);
            let member_xml_id = format!("id-m-{}", m.id);
            client_str_to_xml_id.insert(member_client.to_string(), member_xml_id.clone());

            let subsystems = self.list_subsystems(m.id).await?;
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
        let servers = self.list_security_servers().await?;
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

    // -------------------------------------------------------------------------
    // API Keys Management
    // -------------------------------------------------------------------------

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
    /// Checks both plaintext token (hashed with SHA-256) and direct encoded key.
    pub async fn validate_api_key(&self, key_or_token: &str) -> XRoadResult<Option<Vec<String>>> {
        let hashed_token = format!("{:x}", Sha256::digest(key_or_token.as_bytes()));
        let key = api_key::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(api_key::Column::EncodedKey.eq(key_or_token))
                    .add(api_key::Column::EncodedKey.eq(&hashed_token)),
            )
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

    /// Lists all registered API keys with their assigned roles.
    pub async fn list_api_keys(&self) -> XRoadResult<Vec<(api_key::Model, Vec<String>)>> {
        let keys = api_key::Entity::find()
            .all(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to list API keys: {e}")))?;

        let mut result = Vec::with_capacity(keys.len());
        for key in keys {
            let roles = api_key_role::Entity::find()
                .filter(api_key_role::Column::ApiKeyId.eq(key.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query API key roles: {e}")))?;
            result.push((key, roles.into_iter().map(|r| r.role).collect()));
        }
        Ok(result)
    }

    /// Gets an API key and its assigned roles by its database ID.
    pub async fn get_api_key(&self, id: i64) -> XRoadResult<Option<(api_key::Model, Vec<String>)>> {
        let key = api_key::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to get API key: {e}")))?;

        if let Some(key_model) = key {
            let roles = api_key_role::Entity::find()
                .filter(api_key_role::Column::ApiKeyId.eq(key_model.id))
                .all(self.raw_connection())
                .await
                .map_err(|e| XRoadError::Database(format!("Failed to query API key roles: {e}")))?;
            Ok(Some((
                key_model,
                roles.into_iter().map(|r| r.role).collect(),
            )))
        } else {
            Ok(None)
        }
    }

    /// Updates the assigned roles of an API key.
    pub async fn update_api_key_roles(&self, id: i64, roles: &[&str]) -> XRoadResult<()> {
        let _key = api_key::Entity::find_by_id(id)
            .one(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to find API key: {e}")))?
            .ok_or_else(|| XRoadError::Configuration(format!("API key {id} not found")))?;

        // Remove old roles
        api_key_role::Entity::delete_many()
            .filter(api_key_role::Column::ApiKeyId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| {
                XRoadError::Database(format!("Failed to delete old API key roles: {e}"))
            })?;

        // Insert new roles
        for role in roles {
            api_key_role::ActiveModel {
                api_key_id: Set(id),
                role: Set(role.to_string()),
                ..Default::default()
            }
            .insert(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to insert API key role: {e}")))?;
        }

        Ok(())
    }

    /// Revokes (deletes) an API key by its database ID.
    pub async fn revoke_api_key(&self, id: i64) -> XRoadResult<()> {
        api_key_role::Entity::delete_many()
            .filter(api_key_role::Column::ApiKeyId.eq(id))
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete API key roles: {e}")))?;

        api_key::Entity::delete_by_id(id)
            .exec(self.raw_connection())
            .await
            .map_err(|e| XRoadError::Database(format!("Failed to delete API key: {e}")))?;

        Ok(())
    }
}
