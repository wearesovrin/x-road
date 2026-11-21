pub mod db;
pub mod entities;
pub mod migration;

use poem::{
    EndpointExt, Response, Route, Server, get, handler, http::StatusCode, listener::TcpListener,
    web::Data,
};
use poem_openapi::{Object, OpenApi, OpenApiService, payload::Json};

pub use db::CentralServerDb;
pub use migration::Migrator;

#[derive(Clone)]
pub struct CentralServerState {
    pub instance_id: String,
    pub db: CentralServerDb,
}

#[handler]
async fn handle_shared_params_xml(
    Data(state): Data<&CentralServerState>,
) -> poem::Result<Response> {
    let xml = state
        .db
        .generate_shared_params_xml(&state.instance_id)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Response::builder()
        .content_type("application/xml")
        .body(xml))
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct CentralServerInfo {
    pub instance_identifier: String,
    pub status: String,
    pub member_count: usize,
    pub security_server_count: usize,
}

pub struct CentralServerApi {
    pub instance_id: String,
    pub db: CentralServerDb,
}

#[OpenApi]
impl CentralServerApi {
    /// Returns the Central Server operational status and counts.
    #[oai(path = "/internalconf", method = "get")]
    async fn internal_conf(&self) -> Json<CentralServerInfo> {
        let members = self.db.list_members().await.unwrap_or_default();
        let servers = self.db.list_security_servers().await.unwrap_or_default();

        Json(CentralServerInfo {
            instance_identifier: self.instance_id.clone(),
            status: "READY".to_string(),
            member_count: members.len(),
            security_server_count: servers.len(),
        })
    }
}

pub fn central_server_app(instance_id: String, db: CentralServerDb) -> Route {
    let state = CentralServerState {
        instance_id: instance_id.clone(),
        db: db.clone(),
    };

    let api_service = OpenApiService::new(
        CentralServerApi {
            instance_id: instance_id.clone(),
            db: db.clone(),
        },
        "X-Road Central Server Admin API",
        "0.1.0",
    )
    .server("/api");
    let ui = api_service.swagger_ui();

    Route::new()
        .nest("/api", api_service)
        .nest("/swagger-ui", ui)
        .at(
            "/internalconf/shared-params.xml",
            get(handle_shared_params_xml).data(state.clone()),
        )
        .at(
            "/api/v1/globalconf/shared-params.xml",
            get(handle_shared_params_xml).data(state),
        )
}

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

        // 1. Add member
        let member = db
            .add_member("DEV", "COM", "12345678", "Acme Corp", Some("admin@acme.com"))
            .await
            .unwrap();
        assert_eq!(member.member_code, "12345678");

        // 2. Add subsystem
        let sub = db.add_subsystem(member.id, "MANAGEMENT").await.unwrap();
        assert_eq!(sub.subsystem_code, "MANAGEMENT");

        let subsystems = db.list_subsystems(member.id).await.unwrap();
        assert_eq!(subsystems.len(), 1);

        // 3. Register security server
        let sec_server = db
            .register_security_server(member.id, "SS1", "https://ss1.acme.com:4000")
            .await
            .unwrap();
        assert_eq!(sec_server.server_code, "SS1");

        let client = ClientId::subsystem_unchecked("DEV", "COM", "12345678", "MANAGEMENT");
        db.add_security_server_client(sec_server.id, &client)
            .await
            .unwrap();

        // 4. Global group
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

        // 5. Approved CA and TSA
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

        // 6. Generate shared-params.xml and verify with GlobalConf
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
    }
}
