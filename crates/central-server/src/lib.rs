pub mod entities;
pub mod handlers;
pub mod middleware;
pub mod migration;
pub mod services;

pub use handlers::*;
pub use middleware::{CentralServerAuthEndpoint, CentralServerAuthMiddleware, init_tracer};
pub use migration::Migrator;
pub use services::{CentralServerDb, CentralServerServices, ClientItem};

use poem::Server;
use poem::listener::TcpListener;

/// Launches the Central Server administration and configuration service on the given address.
pub async fn run_central_server(
    bind_addr: &str,
    instance: &str,
    db_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = match db_url {
        Some(url) => CentralServerDb::connect(url).await?,
        None => CentralServerDb::connect_in_memory().await?,
    };

    tracing::info!(
        "Starting X-Road Central Server on {} (Swagger UI at /swagger-ui)...",
        bind_addr
    );
    let app = central_server_app(instance.to_string(), db);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use identifiers::ClientId;

    #[tokio::test]
    async fn test_central_server_db_in_memory() {
        let db = CentralServerDb::connect_in_memory()
            .await
            .expect("Failed to connect to in-memory central server db");

        // 1. Member class
        let mc = db
            .add_member_class("COM", "Commercial companies")
            .await
            .unwrap();
        assert_eq!(mc.code, "COM");
        let classes = db.list_member_classes().await.unwrap();
        assert_eq!(classes.len(), 1);

        // 2. Add member
        let member = db
            .add_member(
                "DEV",
                "COM",
                "12345678",
                "Acme Corp",
                Some("admin@acme.com"),
            )
            .await
            .unwrap();
        assert_eq!(member.member_code, "12345678");

        // 3. Add subsystem
        let sub = db.add_subsystem(member.id, "MANAGEMENT").await.unwrap();
        assert_eq!(sub.subsystem_code, "MANAGEMENT");

        let subsystems = db.list_subsystems(member.id).await.unwrap();
        assert_eq!(subsystems.len(), 1);

        // 4. Register security server
        let sec_server = db
            .register_security_server(member.id, "SS1", "https://ss1.acme.com:4000")
            .await
            .unwrap();
        assert_eq!(sec_server.server_code, "SS1");

        let client = ClientId::subsystem_unchecked("DEV", "COM", "12345678", "MANAGEMENT");
        db.add_security_server_client(sec_server.id, &client)
            .await
            .unwrap();

        // 5. Auth cert for security server
        let auth_cert = db
            .add_security_server_auth_cert(
                sec_server.id,
                "-----BEGIN CERTIFICATE-----\nMIIC...\n-----END CERTIFICATE-----",
            )
            .await
            .unwrap();
        assert!(!auth_cert.cert_hash.is_empty());

        // 6. Global group
        let group = db
            .create_global_group("security-officers", "Security officers group")
            .await
            .unwrap();
        assert_eq!(group.group_code, "security-officers");

        db.add_to_global_group("security-officers", &client.to_string())
            .await
            .unwrap();
        let group_members = db
            .list_global_group_members("security-officers")
            .await
            .unwrap();
        assert_eq!(group_members.len(), 1);
        assert_eq!(group_members[0], client.to_string());

        // 7. Approved CA and TSA
        let ca = db
            .add_approved_ca("TestCA", b"fake-ca-cert", false, None)
            .await
            .unwrap();
        assert_eq!(ca.name, "TestCA");

        let tsa = db
            .add_approved_tsa("TestTSA", "http://tsa.example.com", b"fake-tsa-cert")
            .await
            .unwrap();
        assert_eq!(tsa.name, "TestTSA");

        // 8. Management request and approval
        let m_req = db
            .create_management_request(
                "CLIENT_REGISTRATION",
                "SECURITY_SERVER",
                Some(sec_server.id),
                Some(&client.to_string()),
                None,
                Some("Test registration request"),
            )
            .await
            .unwrap();
        assert_eq!(
            m_req.status,
            common::ManagementRequestStatus::WaitingForApproval.to_string()
        );

        let approved = db.approve_management_request(m_req.id).await.unwrap();
        assert_eq!(
            approved.status,
            common::ManagementRequestStatus::Approved.to_string()
        );

        // 9. Server address & anchor
        db.set_server_address("central.example.com").await.unwrap();
        let anchor_xml = db.generate_anchor_xml("DEV").await.unwrap();
        assert!(anchor_xml.contains("central.example.com"));

        // 10. Generate shared-params.xml and verify with GlobalConf
        let xml = db.generate_shared_params_xml("DEV").await.unwrap();
        assert!(xml.contains("<instanceIdentifier>DEV</instanceIdentifier>"));
        assert!(xml.contains("Acme Corp"));
        assert!(xml.contains("MANAGEMENT"));
        assert!(xml.contains("SS1"));

        let conf = globalconf::GlobalConf::from_shared_params_xml(&xml).unwrap();
        assert_eq!(conf.instance_identifier, "DEV");
        assert!(conf.is_member_registered(&client));
        assert!(conf.is_member_in_group(&client, "security-officers"));
        assert_eq!(conf.approved_cas.len(), 1);
        assert_eq!(conf.approved_tsps.len(), 1);

        // 11. API Keys Management
        let key_id = db
            .create_api_key(
                "hash-xyz-123",
                Some("Dev Admin Key"),
                &["XROAD_SECURITY_OFFICER"],
            )
            .await
            .unwrap();
        assert!(!key_id.is_empty());

        let keys = db.list_api_keys().await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].0.label.as_deref(), Some("Dev Admin Key"));
        assert_eq!(keys[0].1, vec!["XROAD_SECURITY_OFFICER"]);

        // Validate API key
        let roles = db.validate_api_key("hash-xyz-123").await.unwrap();
        assert_eq!(roles, Some(vec!["XROAD_SECURITY_OFFICER".to_string()]));

        // Update roles
        db.update_api_key_roles(keys[0].0.id, &["XROAD_SYSTEM_ADMINISTRATOR"])
            .await
            .unwrap();
        let updated_roles = db.validate_api_key("hash-xyz-123").await.unwrap();
        assert_eq!(
            updated_roles,
            Some(vec!["XROAD_SYSTEM_ADMINISTRATOR".to_string()])
        );

        // Revoke
        db.revoke_api_key(keys[0].0.id).await.unwrap();
        let revoked_roles = db.validate_api_key("hash-xyz-123").await.unwrap();
        assert!(revoked_roles.is_none());
    }
}
