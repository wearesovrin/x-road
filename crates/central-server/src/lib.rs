pub mod db;
pub mod entities;
pub mod migration;

use poem::{
    EndpointExt, Response, Route, Server, get, handler, http::StatusCode, listener::TcpListener,
    web::Data,
};
use poem_openapi::{
    Object, OpenApi, OpenApiService,
    param::Path,
    payload::Json,
};

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

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct MemberDto {
    pub id: i64,
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub name: String,
    pub administrative_contact: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddMemberRequest {
    pub member_class: String,
    pub member_code: String,
    pub name: String,
    pub administrative_contact: Option<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SubsystemDto {
    pub id: i64,
    pub member_id: i64,
    pub subsystem_code: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddSubsystemRequest {
    pub subsystem_code: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SecurityServerDto {
    pub id: i64,
    pub owner_id: i64,
    pub server_code: String,
    pub address: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct RegisterSecurityServerRequest {
    pub owner_id: i64,
    pub server_code: String,
    pub address: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct LinkClientRequest {
    pub security_server_id: i64,
    pub client_id_str: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ApprovedCaDto {
    pub id: i64,
    pub name: String,
    pub authentication_only: bool,
    pub cert_profile_info: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddCaRequest {
    pub name: String,
    pub top_ca_cert_pem: String,
    pub authentication_only: bool,
    pub cert_profile_info: Option<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ApprovedTsaDto {
    pub id: i64,
    pub name: String,
    pub url: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddTsaRequest {
    pub name: String,
    pub url: String,
    pub cert_pem: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct GlobalGroupDto {
    pub group_code: String,
    pub description: String,
    pub member_count: usize,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct CreateGlobalGroupRequest {
    pub group_code: String,
    pub description: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddGroupMemberRequest {
    pub group_code: String,
    pub client_id_str: String,
}

pub struct CentralServerApi {
    pub instance_id: String,
    pub db: CentralServerDb,
}

#[OpenApi]
impl CentralServerApi {
    /// Returns the Central Server operational status and counts.
    #[oai(path = "/v1/status", method = "get")]
    async fn status(&self) -> Json<CentralServerInfo> {
        let members = self.db.list_members().await.unwrap_or_default();
        let servers = self.db.list_security_servers().await.unwrap_or_default();

        Json(CentralServerInfo {
            instance_identifier: self.instance_id.clone(),
            status: "READY".to_string(),
            member_count: members.len(),
            security_server_count: servers.len(),
        })
    }

    /// Lists all registered X-Road members.
    #[oai(path = "/v1/members", method = "get")]
    async fn list_members(&self) -> Json<Vec<MemberDto>> {
        let members = self.db.list_members().await.unwrap_or_default();
        let dtos = members
            .into_iter()
            .map(|m| MemberDto {
                id: m.id,
                instance: m.xroad_instance,
                member_class: m.member_class,
                member_code: m.member_code,
                name: m.name,
                administrative_contact: m.administrative_contact,
            })
            .collect();
        Json(dtos)
    }

    /// Registers a new X-Road member.
    #[oai(path = "/v1/members", method = "post")]
    async fn add_member(&self, body: Json<AddMemberRequest>) -> poem::Result<Json<MemberDto>> {
        let req = body.0;
        let member = self
            .db
            .add_member(
                &self.instance_id,
                &req.member_class,
                &req.member_code,
                &req.name,
                req.administrative_contact.as_deref(),
            )
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(MemberDto {
            id: member.id,
            instance: member.xroad_instance,
            member_class: member.member_class,
            member_code: member.member_code,
            name: member.name,
            administrative_contact: member.administrative_contact,
        }))
    }

    /// Lists all subsystems belonging to a member.
    #[oai(path = "/v1/members/:member_id/subsystems", method = "get")]
    async fn list_subsystems(&self, member_id: Path<i64>) -> poem::Result<Json<Vec<SubsystemDto>>> {
        let subs = self
            .db
            .list_subsystems(member_id.0)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        let dtos = subs
            .into_iter()
            .map(|s| SubsystemDto {
                id: s.id,
                member_id: s.member_id,
                subsystem_code: s.subsystem_code,
            })
            .collect();
        Ok(Json(dtos))
    }

    /// Adds a subsystem to a member.
    #[oai(path = "/v1/members/:member_id/subsystems", method = "post")]
    async fn add_subsystem(
        &self,
        member_id: Path<i64>,
        body: Json<AddSubsystemRequest>,
    ) -> poem::Result<Json<SubsystemDto>> {
        let sub = self
            .db
            .add_subsystem(member_id.0, &body.0.subsystem_code)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SubsystemDto {
            id: sub.id,
            member_id: sub.member_id,
            subsystem_code: sub.subsystem_code,
        }))
    }

    /// Lists all registered Security Servers.
    #[oai(path = "/v1/security-servers", method = "get")]
    async fn list_security_servers(&self) -> Json<Vec<SecurityServerDto>> {
        let servers = self.db.list_security_servers().await.unwrap_or_default();
        let dtos = servers
            .into_iter()
            .map(|s| SecurityServerDto {
                id: s.id,
                owner_id: s.owner_id,
                server_code: s.server_code,
                address: s.address,
            })
            .collect();
        Json(dtos)
    }

    /// Registers a new Security Server.
    #[oai(path = "/v1/security-servers", method = "post")]
    async fn register_security_server(
        &self,
        body: Json<RegisterSecurityServerRequest>,
    ) -> poem::Result<Json<SecurityServerDto>> {
        let req = body.0;
        let server = self
            .db
            .register_security_server(req.owner_id, &req.server_code, &req.address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SecurityServerDto {
            id: server.id,
            owner_id: server.owner_id,
            server_code: server.server_code,
            address: server.address,
        }))
    }

    /// Links an X-Road client to a Security Server.
    #[oai(path = "/v1/security-servers/client", method = "post")]
    async fn link_client(&self, body: Json<LinkClientRequest>) -> poem::Result<Json<bool>> {
        let req = body.0;
        let client_id: identifiers::ClientId = req
            .client_id_str
            .parse()
            .map_err(|e| poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST))?;

        self.db
            .add_security_server_client(req.security_server_id, &client_id)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(true))
    }

    /// Lists Approved Certification Authorities (CAs).
    #[oai(path = "/v1/cas", method = "get")]
    async fn list_cas(&self) -> Json<Vec<ApprovedCaDto>> {
        let cas = self.db.list_approved_cas().await.unwrap_or_default();
        let dtos = cas
            .into_iter()
            .map(|ca| ApprovedCaDto {
                id: ca.id,
                name: ca.name,
                authentication_only: ca.authentication_only,
                cert_profile_info: ca.cert_profile_info,
            })
            .collect();
        Json(dtos)
    }

    /// Registers an Approved CA.
    #[oai(path = "/v1/cas", method = "post")]
    async fn add_ca(&self, body: Json<AddCaRequest>) -> poem::Result<Json<ApprovedCaDto>> {
        let req = body.0;
        let ca = self
            .db
            .add_approved_ca(
                &req.name,
                req.top_ca_cert_pem.as_bytes(),
                req.authentication_only,
                req.cert_profile_info.as_deref(),
            )
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ApprovedCaDto {
            id: ca.id,
            name: ca.name,
            authentication_only: ca.authentication_only,
            cert_profile_info: ca.cert_profile_info,
        }))
    }

    /// Lists Approved Time-Stamping Authorities (TSAs).
    #[oai(path = "/v1/tsas", method = "get")]
    async fn list_tsas(&self) -> Json<Vec<ApprovedTsaDto>> {
        let tsas = self.db.list_approved_tsas().await.unwrap_or_default();
        let dtos = tsas
            .into_iter()
            .map(|t| ApprovedTsaDto {
                id: t.id,
                name: t.name,
                url: t.url,
            })
            .collect();
        Json(dtos)
    }

    /// Registers an Approved TSA.
    #[oai(path = "/v1/tsas", method = "post")]
    async fn add_tsa(&self, body: Json<AddTsaRequest>) -> poem::Result<Json<ApprovedTsaDto>> {
        let req = body.0;
        let tsa = self
            .db
            .add_approved_tsa(&req.name, &req.url, req.cert_pem.as_bytes())
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ApprovedTsaDto {
            id: tsa.id,
            name: tsa.name,
            url: tsa.url,
        }))
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
