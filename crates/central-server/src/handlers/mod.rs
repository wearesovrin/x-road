use crate::services::CentralServerDb;
use poem::{EndpointExt, IntoResponse, Response, Route, get, handler, http::StatusCode, web::Data};
use poem_openapi::{
    Object, OpenApi, OpenApiService,
    param::Path,
    payload::{Attachment, Json, PlainText},
};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct CentralServerState {
    pub instance_id: String,
    pub db: CentralServerDb,
}

/// Helper wrapper for raw pre-rendered XML HTTP responses in Poem.
pub struct RawXml(pub String);

impl IntoResponse for RawXml {
    fn into_response(self) -> Response {
        Response::builder()
            .content_type("application/xml; charset=utf-8")
            .body(self.0)
    }
}

#[handler]
pub async fn handle_shared_params_xml(
    Data(state): Data<&CentralServerState>,
) -> poem::Result<RawXml> {
    let xml = state
        .db
        .generate_shared_params_xml(&state.instance_id)
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(RawXml(xml))
}

// ==========================================
// DTOs
// ==========================================

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InitializationStatusDto {
    pub is_initialized: bool,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct InitialRegistrationDto {
    pub central_server_address: String,
    pub software_token_pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MemberClassDto {
    pub code: String,
    pub description: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddMemberClassRequest {
    pub code: String,
    pub description: String,
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

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateMemberRequest {
    pub name: String,
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
pub struct ClientDto {
    pub client_id: String,
    pub member_name: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub status: String,
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

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddServerClientRequest {
    pub client_id_str: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SecurityServerAuthCertDto {
    pub id: i64,
    pub security_server_id: i64,
    pub cert_hash: String,
    pub created_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct AddAuthCertRequest {
    pub cert_pem: String,
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
    pub client_id_str: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ConfigurationAnchorDto {
    pub instance_identifier: String,
    pub generated_at: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerAddressDto {
    pub server_address: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct SystemVersionDto {
    pub version: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ManagementRequestDto {
    pub id: i64,
    pub request_type: String,
    pub origin: String,
    pub security_server_id: Option<i64>,
    pub client_id_str: Option<String>,
    pub status: String,
    pub comments: Option<String>,
    pub created_at: String,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct CreateManagementRequestDto {
    pub request_type: String,
    pub origin: String,
    pub security_server_id: Option<i64>,
    pub client_id_str: Option<String>,
    pub auth_cert_pem: Option<String>,
    pub comments: Option<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct BackupDto {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: u64,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct CreateApiKeyReq {
    pub roles: Vec<String>,
    pub label: Option<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct UpdateApiKeyReq {
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ApiKeyDto {
    pub id: i64,
    pub key_id: String,
    pub key: Option<String>,
    pub label: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Deserialize, Debug, Clone)]
pub struct ValidateApiKeyReq {
    pub key: String,
}

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ValidateApiKeyResp {
    pub valid: bool,
    pub roles: Vec<String>,
}

// ==========================================
// Central Server REST API Implementation
// ==========================================

pub struct CentralServerApi {
    pub instance_id: String,
    pub db: CentralServerDb,
}

#[OpenApi]
impl CentralServerApi {
    // ------------------------------------------
    // Initialization
    // ------------------------------------------

    #[oai(path = "/v1/initialization/status", method = "get")]
    async fn get_initialization_status(&self) -> Json<InitializationStatusDto> {
        let is_init = self
            .db
            .get_system_parameter("is_initialized")
            .await
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(false);
        Json(InitializationStatusDto {
            is_initialized: is_init,
        })
    }

    #[oai(path = "/v1/initialization", method = "post")]
    async fn initialize(
        &self,
        body: Json<InitialRegistrationDto>,
    ) -> poem::Result<Json<InitializationStatusDto>> {
        self.db
            .set_server_address(&body.0.central_server_address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        self.db
            .set_system_parameter("is_initialized", "true")
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(InitializationStatusDto {
            is_initialized: true,
        }))
    }

    // ------------------------------------------
    // Member Classes
    // ------------------------------------------

    #[oai(path = "/v1/member-classes", method = "get")]
    async fn list_member_classes(&self) -> Json<Vec<MemberClassDto>> {
        let classes = self.db.list_member_classes().await.unwrap_or_default();
        Json(
            classes
                .into_iter()
                .map(|c| MemberClassDto {
                    code: c.code,
                    description: c.description,
                })
                .collect(),
        )
    }

    #[oai(path = "/v1/member-classes", method = "post")]
    async fn add_member_class(
        &self,
        body: Json<AddMemberClassRequest>,
    ) -> poem::Result<Json<MemberClassDto>> {
        let mc = self
            .db
            .add_member_class(&body.0.code, &body.0.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(MemberClassDto {
            code: mc.code,
            description: mc.description,
        }))
    }

    #[oai(path = "/v1/member-classes/:code", method = "get")]
    async fn get_member_class(&self, code: Path<String>) -> poem::Result<Json<MemberClassDto>> {
        let mc = self
            .db
            .get_member_class(&code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Member class not found", StatusCode::NOT_FOUND)
            })?;
        Ok(Json(MemberClassDto {
            code: mc.code,
            description: mc.description,
        }))
    }

    #[oai(path = "/v1/member-classes/:code", method = "delete")]
    async fn delete_member_class(&self, code: Path<String>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_member_class(&code.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    // ------------------------------------------
    // Members
    // ------------------------------------------

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

    #[oai(path = "/v1/members/:member_id", method = "get")]
    async fn get_member(&self, member_id: Path<i64>) -> poem::Result<Json<MemberDto>> {
        let m = self
            .db
            .get_member_by_id(member_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("Member not found", StatusCode::NOT_FOUND))?;
        Ok(Json(MemberDto {
            id: m.id,
            instance: m.xroad_instance,
            member_class: m.member_class,
            member_code: m.member_code,
            name: m.name,
            administrative_contact: m.administrative_contact,
        }))
    }

    #[oai(path = "/v1/members/:member_id", method = "put")]
    async fn update_member(
        &self,
        member_id: Path<i64>,
        body: Json<UpdateMemberRequest>,
    ) -> poem::Result<Json<MemberDto>> {
        let m = self
            .db
            .update_member_name(member_id.0, &body.0.name)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(MemberDto {
            id: m.id,
            instance: m.xroad_instance,
            member_class: m.member_class,
            member_code: m.member_code,
            name: m.name,
            administrative_contact: m.administrative_contact,
        }))
    }

    #[oai(path = "/v1/members/:member_id", method = "delete")]
    async fn delete_member(&self, member_id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_member(member_id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/members/:member_id/subsystems", method = "get")]
    async fn list_subsystems(&self, member_id: Path<i64>) -> poem::Result<Json<Vec<SubsystemDto>>> {
        let subs = self.db.list_subsystems(member_id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;

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

    #[oai(path = "/v1/members/:member_id/global-groups", method = "get")]
    async fn list_member_global_groups(
        &self,
        member_id: Path<i64>,
    ) -> poem::Result<Json<Vec<String>>> {
        let groups = self
            .db
            .list_member_global_groups(member_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(groups))
    }

    #[oai(path = "/v1/members/:member_id/owned-servers", method = "get")]
    async fn list_member_owned_servers(
        &self,
        member_id: Path<i64>,
    ) -> poem::Result<Json<Vec<SecurityServerDto>>> {
        let servers = self
            .db
            .list_member_owned_servers(member_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(
            servers
                .into_iter()
                .map(|s| SecurityServerDto {
                    id: s.id,
                    owner_id: s.owner_id,
                    server_code: s.server_code,
                    address: s.address,
                })
                .collect(),
        ))
    }

    // ------------------------------------------
    // Subsystems
    // ------------------------------------------

    #[oai(path = "/v1/subsystems", method = "get")]
    async fn list_all_subsystems(&self) -> Json<Vec<SubsystemDto>> {
        let subs = self.db.list_all_subsystems().await.unwrap_or_default();
        Json(
            subs.into_iter()
                .map(|s| SubsystemDto {
                    id: s.id,
                    member_id: s.member_id,
                    subsystem_code: s.subsystem_code,
                })
                .collect(),
        )
    }

    #[oai(path = "/v1/subsystems/:subsystem_id", method = "get")]
    async fn get_subsystem(&self, subsystem_id: Path<i64>) -> poem::Result<Json<SubsystemDto>> {
        let sub = self
            .db
            .get_subsystem(subsystem_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Subsystem not found", StatusCode::NOT_FOUND)
            })?;
        Ok(Json(SubsystemDto {
            id: sub.id,
            member_id: sub.member_id,
            subsystem_code: sub.subsystem_code,
        }))
    }

    #[oai(path = "/v1/subsystems/:subsystem_id", method = "delete")]
    async fn delete_subsystem(&self, subsystem_id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self
            .db
            .delete_subsystem(subsystem_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    // ------------------------------------------
    // Clients
    // ------------------------------------------

    #[oai(path = "/v1/clients", method = "get")]
    async fn list_all_clients(&self) -> poem::Result<Json<Vec<ClientDto>>> {
        let clients = self.db.list_all_clients().await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(
            clients
                .into_iter()
                .map(|c| ClientDto {
                    client_id: c.client_id,
                    member_name: c.member_name,
                    member_class: c.member_class,
                    member_code: c.member_code,
                    subsystem_code: c.subsystem_code,
                    status: c.status,
                })
                .collect(),
        ))
    }

    // ------------------------------------------
    // Security Servers
    // ------------------------------------------

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

    #[oai(path = "/v1/security-servers/:server_id", method = "get")]
    async fn get_security_server(
        &self,
        server_id: Path<i64>,
    ) -> poem::Result<Json<SecurityServerDto>> {
        let s = self
            .db
            .get_security_server(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Security Server not found", StatusCode::NOT_FOUND)
            })?;
        Ok(Json(SecurityServerDto {
            id: s.id,
            owner_id: s.owner_id,
            server_code: s.server_code,
            address: s.address,
        }))
    }

    #[oai(path = "/v1/security-servers/:server_id", method = "delete")]
    async fn delete_security_server(&self, server_id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self
            .db
            .delete_security_server(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/security-servers/:server_id/clients", method = "get")]
    async fn list_server_clients(&self, server_id: Path<i64>) -> poem::Result<Json<Vec<String>>> {
        let clients = self
            .db
            .list_security_server_clients(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(clients))
    }

    #[oai(path = "/v1/security-servers/:server_id/clients", method = "post")]
    async fn add_server_client(
        &self,
        server_id: Path<i64>,
        body: Json<AddServerClientRequest>,
    ) -> poem::Result<Json<bool>> {
        let client_id: identifiers::ClientId = body.0.client_id_str.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        self.db
            .add_security_server_client(server_id.0, &client_id)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(true))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/clients/:client_id",
        method = "delete"
    )]
    async fn remove_server_client(
        &self,
        server_id: Path<i64>,
        client_id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let ok = self
            .db
            .remove_security_server_client(server_id.0, &client_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/security-servers/client", method = "post")]
    async fn link_client(&self, body: Json<LinkClientRequest>) -> poem::Result<Json<bool>> {
        let req = body.0;
        let client_id: identifiers::ClientId = req.client_id_str.parse().map_err(|e| {
            poem::Error::from_string(format!("Invalid ClientId: {e}"), StatusCode::BAD_REQUEST)
        })?;

        self.db
            .add_security_server_client(req.security_server_id, &client_id)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(true))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/authentication-certificates",
        method = "get"
    )]
    async fn list_server_auth_certs(
        &self,
        server_id: Path<i64>,
    ) -> poem::Result<Json<Vec<SecurityServerAuthCertDto>>> {
        let certs = self
            .db
            .list_security_server_auth_certs(server_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Json(
            certs
                .into_iter()
                .map(|c| SecurityServerAuthCertDto {
                    id: c.id,
                    security_server_id: c.security_server_id,
                    cert_hash: c.cert_hash,
                    created_at: c.created_at.to_rfc3339(),
                })
                .collect(),
        ))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/authentication-certificates",
        method = "post"
    )]
    async fn add_server_auth_cert(
        &self,
        server_id: Path<i64>,
        body: Json<AddAuthCertRequest>,
    ) -> poem::Result<Json<SecurityServerAuthCertDto>> {
        let cert = self
            .db
            .add_security_server_auth_cert(server_id.0, &body.0.cert_pem)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(SecurityServerAuthCertDto {
            id: cert.id,
            security_server_id: cert.security_server_id,
            cert_hash: cert.cert_hash,
            created_at: cert.created_at.to_rfc3339(),
        }))
    }

    #[oai(
        path = "/v1/security-servers/:server_id/authentication-certificates/:cert_id",
        method = "delete"
    )]
    async fn delete_server_auth_cert(
        &self,
        _server_id: Path<i64>,
        cert_id: Path<i64>,
    ) -> poem::Result<Json<bool>> {
        let ok = self
            .db
            .delete_security_server_auth_cert(cert_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    // ------------------------------------------
    // Global Groups
    // ------------------------------------------

    #[oai(path = "/v1/global-groups", method = "get")]
    async fn list_global_groups(&self) -> Json<Vec<GlobalGroupDto>> {
        let groups = self.db.list_global_groups().await.unwrap_or_default();
        let mut dtos = Vec::new();
        for g in groups {
            let members = self
                .db
                .list_global_group_members(&g.group_code)
                .await
                .unwrap_or_default();
            dtos.push(GlobalGroupDto {
                group_code: g.group_code,
                description: g.description,
                member_count: members.len(),
            });
        }
        Json(dtos)
    }

    #[oai(path = "/v1/global-groups", method = "post")]
    async fn create_global_group(
        &self,
        body: Json<CreateGlobalGroupRequest>,
    ) -> poem::Result<Json<GlobalGroupDto>> {
        let req = body.0;
        let g = self
            .db
            .create_global_group(&req.group_code, &req.description)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(GlobalGroupDto {
            group_code: g.group_code,
            description: g.description,
            member_count: 0,
        }))
    }

    #[oai(path = "/v1/global-groups/:group_code", method = "get")]
    async fn get_global_group(
        &self,
        group_code: Path<String>,
    ) -> poem::Result<Json<GlobalGroupDto>> {
        let g = self
            .db
            .get_global_group(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Global group not found", StatusCode::NOT_FOUND)
            })?;

        let members = self
            .db
            .list_global_group_members(&g.group_code)
            .await
            .unwrap_or_default();

        Ok(Json(GlobalGroupDto {
            group_code: g.group_code,
            description: g.description,
            member_count: members.len(),
        }))
    }

    #[oai(path = "/v1/global-groups/:group_code", method = "delete")]
    async fn delete_global_group(&self, group_code: Path<String>) -> poem::Result<Json<bool>> {
        let ok = self
            .db
            .delete_global_group(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/global-groups/:group_code/members", method = "get")]
    async fn list_global_group_members(
        &self,
        group_code: Path<String>,
    ) -> poem::Result<Json<Vec<String>>> {
        let members = self
            .db
            .list_global_group_members(&group_code.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(members))
    }

    #[oai(path = "/v1/global-groups/:group_code/members", method = "post")]
    async fn add_global_group_member(
        &self,
        group_code: Path<String>,
        body: Json<AddGroupMemberRequest>,
    ) -> poem::Result<Json<bool>> {
        self.db
            .add_to_global_group(&group_code.0, &body.0.client_id_str)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(true))
    }

    #[oai(
        path = "/v1/global-groups/:group_code/members/:client_id",
        method = "delete"
    )]
    async fn remove_global_group_member(
        &self,
        group_code: Path<String>,
        client_id: Path<String>,
    ) -> poem::Result<Json<bool>> {
        let ok = self
            .db
            .remove_from_global_group(&group_code.0, &client_id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(Json(ok))
    }

    // ------------------------------------------
    // Certification Services (CAs)
    // ------------------------------------------

    #[oai(path = "/v1/certification-services", method = "get")]
    async fn list_certification_services(&self) -> Json<Vec<ApprovedCaDto>> {
        self.list_cas().await
    }

    #[oai(path = "/v1/certification-services", method = "post")]
    async fn add_certification_service(
        &self,
        body: Json<AddCaRequest>,
    ) -> poem::Result<Json<ApprovedCaDto>> {
        self.add_ca(body).await
    }

    #[oai(path = "/v1/certification-services/:id", method = "get")]
    async fn get_certification_service(&self, id: Path<i64>) -> poem::Result<Json<ApprovedCaDto>> {
        let ca = self
            .db
            .get_approved_ca(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("CA not found", StatusCode::NOT_FOUND))?;
        Ok(Json(ApprovedCaDto {
            id: ca.id,
            name: ca.name,
            authentication_only: ca.authentication_only,
            cert_profile_info: ca.cert_profile_info,
        }))
    }

    #[oai(path = "/v1/certification-services/:id", method = "delete")]
    async fn delete_certification_service(&self, id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_approved_ca(id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

    #[oai(path = "/v1/certification-services/:id/certificate", method = "get")]
    async fn download_ca_certificate(&self, id: Path<i64>) -> poem::Result<PlainText<String>> {
        let ca = self
            .db
            .get_approved_ca(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("CA not found", StatusCode::NOT_FOUND))?;
        Ok(PlainText(
            String::from_utf8_lossy(&ca.top_ca_cert).to_string(),
        ))
    }

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

    // ------------------------------------------
    // Timestamping Services (TSAs)
    // ------------------------------------------

    #[oai(path = "/v1/timestamping-services", method = "get")]
    async fn list_timestamping_services(&self) -> Json<Vec<ApprovedTsaDto>> {
        self.list_tsas().await
    }

    #[oai(path = "/v1/timestamping-services", method = "post")]
    async fn add_timestamping_service(
        &self,
        body: Json<AddTsaRequest>,
    ) -> poem::Result<Json<ApprovedTsaDto>> {
        self.add_tsa(body).await
    }

    #[oai(path = "/v1/timestamping-services/:id", method = "get")]
    async fn get_timestamping_service(&self, id: Path<i64>) -> poem::Result<Json<ApprovedTsaDto>> {
        let tsa = self
            .db
            .get_approved_tsa(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| poem::Error::from_string("TSA not found", StatusCode::NOT_FOUND))?;
        Ok(Json(ApprovedTsaDto {
            id: tsa.id,
            name: tsa.name,
            url: tsa.url,
        }))
    }

    #[oai(path = "/v1/timestamping-services/:id", method = "delete")]
    async fn delete_timestamping_service(&self, id: Path<i64>) -> poem::Result<Json<bool>> {
        let ok = self.db.delete_approved_tsa(id.0).await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ok))
    }

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

    // ------------------------------------------
    // Configuration Sources & Anchors
    // ------------------------------------------

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor",
        method = "get"
    )]
    async fn get_configuration_anchor(
        &self,
        _configuration_type: Path<String>,
    ) -> poem::Result<Json<ConfigurationAnchorDto>> {
        Ok(Json(ConfigurationAnchorDto {
            instance_identifier: self.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor/download",
        method = "get"
    )]
    async fn download_configuration_anchor(
        &self,
        _configuration_type: Path<String>,
    ) -> poem::Result<Attachment<Vec<u8>>> {
        let xml = self
            .db
            .generate_anchor_xml(&self.instance_id)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Attachment::new(xml.into_bytes())
            .filename(format!("configuration-anchor-{}.xml", self.instance_id)))
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor/re-create",
        method = "post"
    )]
    async fn recreate_configuration_anchor(
        &self,
        _configuration_type: Path<String>,
    ) -> poem::Result<Json<ConfigurationAnchorDto>> {
        Ok(Json(ConfigurationAnchorDto {
            instance_identifier: self.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    // ------------------------------------------
    // System Information
    // ------------------------------------------

    #[oai(path = "/v1/system/server-address", method = "get")]
    async fn get_server_address(&self) -> poem::Result<Json<ServerAddressDto>> {
        let addr = self.db.get_server_address().await.map_err(|e| {
            poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(Json(ServerAddressDto {
            server_address: addr,
        }))
    }

    #[oai(path = "/v1/system/server-address", method = "put")]
    async fn update_server_address(
        &self,
        body: Json<ServerAddressDto>,
    ) -> poem::Result<Json<ServerAddressDto>> {
        self.db
            .set_server_address(&body.0.server_address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        Ok(Json(ServerAddressDto {
            server_address: body.0.server_address,
        }))
    }

    #[oai(path = "/v1/system/status", method = "get")]
    async fn system_status(&self) -> Json<CentralServerInfo> {
        self.status().await
    }

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

    #[oai(path = "/v1/system/version", method = "get")]
    async fn system_version(&self) -> Json<SystemVersionDto> {
        Json(SystemVersionDto {
            version: "7.5.0-rust".to_string(),
        })
    }

    // ------------------------------------------
    // Management Requests
    // ------------------------------------------

    #[oai(path = "/v1/management-requests", method = "get")]
    async fn list_management_requests(&self) -> Json<Vec<ManagementRequestDto>> {
        let reqs = self.db.list_management_requests().await.unwrap_or_default();
        Json(
            reqs.into_iter()
                .map(|r| ManagementRequestDto {
                    id: r.id,
                    request_type: r.request_type,
                    origin: r.origin,
                    security_server_id: r.security_server_id,
                    client_id_str: r.client_id_str,
                    status: r.status,
                    comments: r.comments,
                    created_at: r.created_at.to_rfc3339(),
                })
                .collect(),
        )
    }

    #[oai(path = "/v1/management-requests", method = "post")]
    async fn create_management_request(
        &self,
        body: Json<CreateManagementRequestDto>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let req = body.0;
        let created = self
            .db
            .create_management_request(
                &req.request_type,
                &req.origin,
                req.security_server_id,
                req.client_id_str.as_deref(),
                req.auth_cert_pem.as_deref(),
                req.comments.as_deref(),
            )
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ManagementRequestDto {
            id: created.id,
            request_type: created.request_type,
            origin: created.origin,
            security_server_id: created.security_server_id,
            client_id_str: created.client_id_str,
            status: created.status,
            comments: created.comments,
            created_at: created.created_at.to_rfc3339(),
        }))
    }

    #[oai(path = "/v1/management-requests/:id", method = "get")]
    async fn get_management_request(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let r = self
            .db
            .get_management_request(id.0)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?
            .ok_or_else(|| {
                poem::Error::from_string("Management request not found", StatusCode::NOT_FOUND)
            })?;

        Ok(Json(ManagementRequestDto {
            id: r.id,
            request_type: r.request_type,
            origin: r.origin,
            security_server_id: r.security_server_id,
            client_id_str: r.client_id_str,
            status: r.status,
            comments: r.comments,
            created_at: r.created_at.to_rfc3339(),
        }))
    }

    #[oai(path = "/v1/management-requests/:id/approval", method = "put")]
    async fn approve_management_request(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let r = self
            .db
            .approve_management_request(id.0)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ManagementRequestDto {
            id: r.id,
            request_type: r.request_type,
            origin: r.origin,
            security_server_id: r.security_server_id,
            client_id_str: r.client_id_str,
            status: r.status,
            comments: r.comments,
            created_at: r.created_at.to_rfc3339(),
        }))
    }

    #[oai(path = "/v1/management-requests/:id/decline", method = "post")]
    async fn decline_management_request(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<ManagementRequestDto>> {
        let r = self
            .db
            .decline_management_request(id.0)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(ManagementRequestDto {
            id: r.id,
            request_type: r.request_type,
            origin: r.origin,
            security_server_id: r.security_server_id,
            client_id_str: r.client_id_str,
            status: r.status,
            comments: r.comments,
            created_at: r.created_at.to_rfc3339(),
        }))
    }

    // ------------------------------------------
    // Backups
    // ------------------------------------------

    #[oai(path = "/v1/backups", method = "get")]
    async fn list_backups(&self) -> Json<Vec<BackupDto>> {
        Json(vec![BackupDto {
            filename: "backup_latest.tar.gz".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
        }])
    }

    #[oai(path = "/v1/backups", method = "post")]
    async fn create_backup(&self) -> Json<BackupDto> {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Json(BackupDto {
            filename: format!("conf_backup_{ts}.tar.gz"),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
        })
    }

    #[oai(path = "/v1/backups/:filename/download", method = "get")]
    async fn download_backup(&self, filename: Path<String>) -> Attachment<Vec<u8>> {
        Attachment::new(vec![0x1f, 0x8b, 0x08, 0x00]).filename(filename.0)
    }

    #[oai(path = "/v1/backups/:filename/restore", method = "post")]
    async fn restore_backup(&self, _filename: Path<String>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/backups/:filename", method = "delete")]
    async fn delete_backup(&self, _filename: Path<String>) -> Json<bool> {
        Json(true)
    }

    // -------------------------------------------------------------------------
    // API Keys Management
    // -------------------------------------------------------------------------

    /// Creates a new API key with assigned roles. Returns plaintext key token.
    #[oai(path = "/v1/api-keys", method = "post")]
    async fn create_api_key(&self, req: Json<CreateApiKeyReq>) -> Json<ApiKeyDto> {
        let plaintext_key = uuid::Uuid::new_v4().to_string();
        let encoded_key = format!("{:x}", Sha256::digest(plaintext_key.as_bytes()));
        let role_strs: Vec<&str> = req.roles.iter().map(String::as_str).collect();

        let key_id = self
            .db
            .create_api_key(&encoded_key, req.label.as_deref(), &role_strs)
            .await
            .unwrap_or_default();

        let all_keys = self.db.list_api_keys().await.unwrap_or_default();
        let db_id = all_keys
            .iter()
            .find(|(k, _)| k.key_id == key_id)
            .map(|(k, _)| k.id)
            .unwrap_or(0);

        Json(ApiKeyDto {
            id: db_id,
            key_id,
            key: Some(plaintext_key),
            label: req.label.clone(),
            roles: req.roles.clone(),
        })
    }

    /// Lists all registered API keys.
    #[oai(path = "/v1/api-keys", method = "get")]
    async fn list_api_keys(&self) -> Json<Vec<ApiKeyDto>> {
        let keys = self.db.list_api_keys().await.unwrap_or_default();
        let list = keys
            .into_iter()
            .map(|(k, roles)| ApiKeyDto {
                id: k.id,
                key_id: k.key_id,
                key: None,
                label: k.label,
                roles,
            })
            .collect();
        Json(list)
    }

    /// Gets an API key and its assigned roles by database ID.
    #[oai(path = "/v1/api-keys/:id", method = "get")]
    async fn get_api_key(&self, id: Path<i64>) -> poem::Result<Json<ApiKeyDto>> {
        match self.db.get_api_key(id.0).await {
            Ok(Some((k, roles))) => Ok(Json(ApiKeyDto {
                id: k.id,
                key_id: k.key_id,
                key: None,
                label: k.label,
                roles,
            })),
            _ => Err(poem::error::NotFoundError.into()),
        }
    }

    /// Updates the assigned roles of an API key.
    #[oai(path = "/v1/api-keys/:id", method = "put")]
    async fn update_api_key(
        &self,
        id: Path<i64>,
        req: Json<UpdateApiKeyReq>,
    ) -> poem::Result<Json<ApiKeyDto>> {
        let role_strs: Vec<&str> = req.roles.iter().map(String::as_str).collect();
        match self.db.update_api_key_roles(id.0, &role_strs).await {
            Ok(()) => match self.db.get_api_key(id.0).await {
                Ok(Some((k, roles))) => Ok(Json(ApiKeyDto {
                    id: k.id,
                    key_id: k.key_id,
                    key: None,
                    label: k.label,
                    roles,
                })),
                _ => Err(poem::error::NotFoundError.into()),
            },
            Err(e) => Err(poem::error::BadRequest(e).into()),
        }
    }

    /// Revokes (deletes) an API key by database ID.
    #[oai(path = "/v1/api-keys/:id", method = "delete")]
    async fn revoke_api_key(&self, id: Path<i64>) -> Json<bool> {
        let _ = self.db.revoke_api_key(id.0).await;
        Json(true)
    }

    /// Validates an API key and returns assigned roles.
    #[oai(path = "/v1/api-keys/validate", method = "post")]
    async fn validate_api_key(&self, req: Json<ValidateApiKeyReq>) -> Json<ValidateApiKeyResp> {
        match self.db.validate_api_key(&req.key).await {
            Ok(Some(roles)) => Json(ValidateApiKeyResp { valid: true, roles }),
            _ => Json(ValidateApiKeyResp {
                valid: false,
                roles: vec![],
            }),
        }
    }
}

pub struct CentralServerAuthMiddleware {
    pub db: CentralServerDb,
}

impl<E: poem::Endpoint> poem::Middleware<E> for CentralServerAuthMiddleware {
    type Output = CentralServerAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        CentralServerAuthEndpoint {
            ep,
            db: self.db.clone(),
        }
    }
}

pub struct CentralServerAuthEndpoint<E> {
    ep: E,
    db: CentralServerDb,
}

impl<E: poem::Endpoint> poem::Endpoint for CentralServerAuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: poem::Request) -> poem::Result<Self::Output> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        if let Some(header) = auth_header {
            if let Some(token) = common::headers::parse_api_key_header(header) {
                match self.db.validate_api_key(token).await {
                    Ok(Some(roles)) => {
                        req.extensions_mut()
                            .insert(common::AuthenticatedUser::new(None, roles));
                    }
                    _ => {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .content_type("application/json")
                            .body(r#"{"error":{"code":"invalid_api_key","message":"The API key was not found or is invalid"}}"#));
                    }
                }
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/json")
                    .body(r#"{"error":{"code":"invalid_authorization_header","message":"Invalid Authorization header format"}}"#));
            }
        } else if std::env::var("XROAD_REQUIRE_API_KEY").as_deref() == Ok("true") {
            let path = req.uri().path();
            if !path.starts_with("/swagger-ui")
                && !path.ends_with("/openapi.json")
                && !path.contains("shared-params.xml")
                && !path.ends_with("/status")
            {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/json")
                    .body(r#"{"error":{"code":"missing_api_key","message":"Missing required Authorization header"}}"#));
            }
        }

        let resp = self.ep.call(req).await?;
        Ok(resp.into_response())
    }
}

pub fn central_server_app(instance_id: String, db: CentralServerDb) -> Route {
    let state = CentralServerState {
        instance_id: instance_id.clone(),
        db: db.clone(),
    };

    let auth_middleware = CentralServerAuthMiddleware { db: db.clone() };

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
    let auth_api = api_service.with(auth_middleware);

    Route::new()
        .nest("/api", auth_api)
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
